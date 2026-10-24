use serde_json::{json, Value};
use std::{net::TcpListener, thread};
use t3_protocol::*;
#[test]
fn rejections_keep_the_servers_reason_but_no_other_cause_data() {
    // Effect 4 encodes a cause as its list of reasons.
    let cause = json!([{"_tag":"Fail","error":{"_tag":"ProjectNotFound","message":"Project p-1 was not found.\nstack","request":"PRIVATE"}}]);
    let error = rejection_for_test(&cause);
    assert_eq!(error, "rpc_rejected: Project p-1 was not found.");
    assert_eq!(error_code(&error), "rpc_rejected");
    assert_eq!(rejection_for_test(&json!({"private":"PRIVATE"})), "rpc_rejected");
    assert_eq!(rejection_for_test(&json!({"_tag":"Die","defect":"boom"})), "rpc_rejected: boom");
    assert_eq!(rejection_for_test(&json!({"_tag":"Fail","error":{"_tag":"AuthScopeError"}})), "write_not_authorized: AuthScopeError");
    let long = rejection_for_test(&json!({"_tag":"Fail","error":{"message":"x".repeat(400)}}));
    assert_eq!(long.chars().count(), "rpc_rejected: ".len() + 301);
    assert!(long.ends_with('…'));
    assert_eq!(error_code("Not a code: text"), "Not a code: text");
    assert_eq!(error_code("invalid_id"), "invalid_id");
}
#[test]
fn rpc_failure_returns_code_and_first_line_of_the_reason() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        let (s, _) = listener.accept().unwrap();
        let mut ws = tungstenite::accept(s).unwrap();
        let r: Value = serde_json::from_str(ws.read().unwrap().to_text().unwrap()).unwrap();
        assert_eq!(r["tag"], "projects.remove");
        ws.send(tungstenite::Message::Text(json!({"_tag":"Exit","requestId":"1","exit":{"_tag":"Failure","cause":[{"_tag":"Fail","error":{"_tag":"ProjectBusy","message":"Project has active runs\nat stack","secret":"PRIVATE"}}]}}).to_string().into())).unwrap();
        let _ = ws.read();
    });
    let c = Connection::new(&base, None, Credential::bearer("synthetic").unwrap()).unwrap();
    let error = c.rpc("projects.remove", json!({"projectId":"p"})).unwrap_err();
    assert_eq!(error, "rpc_rejected: Project has active runs");
    assert!(!error.contains("PRIVATE"));
    server.join().unwrap();
}
#[test]
fn failed_websocket_handshake_keeps_its_reason() {
    use std::io::{Read, Write};
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        let (mut s, _) = listener.accept().unwrap();
        let mut header = vec![];
        let mut b = [0];
        while !header.ends_with(b"\r\n\r\n") {
            s.read_exact(&mut b).unwrap();
            header.push(b[0]);
        }
        write!(s, "HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
    });
    let c = Connection::new(&base, None, Credential::bearer("synthetic").unwrap()).unwrap();
    let error = c.rpc("server.getConfig", json!({})).unwrap_err();
    assert_eq!(error_code(&error), "rpc_connection_failed");
    assert!(error.len() > "rpc_connection_failed: ".len(), "{error}");
    server.join().unwrap();
}
