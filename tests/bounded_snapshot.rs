use serde_json::{json, Value};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::atomic::{AtomicBool, Ordering},
    thread,
};
use t3_protocol::{Connection, Credential};
fn header(s: &mut TcpStream) -> String {
    let mut header = vec![];
    let mut b = [0];
    while !header.ends_with(b"\r\n\r\n") {
        s.read_exact(&mut b).unwrap();
        header.push(b[0]);
    }
    String::from_utf8(header).unwrap()
}
#[test]
fn http_keeps_the_generic_limit_and_accepts_large_bounded_thread_snapshots() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let body = json!({"snapshotSequence":3,"projection":{"thread":{"id":"t"},"turnItems":[{"id":"large","text":"x".repeat(9_000_000)}]}}).to_string();
    assert!(body.len() > 8 * 1024 * 1024);
    let server = thread::spawn(move || {
        for path in [
            "GET /api/orchestration/shell HTTP",
            "GET /api/orchestration/threads/t/bounded?compactTurnItems=1 HTTP",
            "GET /api/orchestration/threads/t/bounded HTTP",
        ] {
            let (mut s, _) = listener.accept().unwrap();
            let h = header(&mut s);
            assert!(h.starts_with(path), "{h}");
            write!(s, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).unwrap();
            let _ = s.write_all(body.as_bytes());
        }
    });
    let c = Connection::new(&base, None, Credential::bearer("synthetic").unwrap()).unwrap();
    assert_eq!(c.request_json("GET", "/api/orchestration/shell", None, true).unwrap_err(), "too_large");
    let snapshot = c.request_json("GET", "/api/orchestration/threads/t/bounded?compactTurnItems=1", None, true).unwrap();
    assert_eq!(snapshot["projection"]["turnItems"][0]["text"].as_str().unwrap().len(), 9_000_000);
    // Only protocol reads of a thread snapshot get the larger allowance.
    assert_eq!(c.request_json("GET", "/api/orchestration/threads/t/bounded", None, false).unwrap_err(), "too_large");
    server.join().unwrap();
}
#[test]
fn bounded_snapshot_still_rejects_responses_over_64_mib() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        let (mut s, _) = listener.accept().unwrap();
        header(&mut s);
        write!(s, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", 65 * 1024 * 1024).unwrap();
        let chunk = vec![b' '; 1024 * 1024];
        for _ in 0..65 {
            if s.write_all(&chunk).is_err() {
                break;
            }
        }
    });
    let c = Connection::new(&base, None, Credential::bearer("synthetic").unwrap()).unwrap();
    assert_eq!(c.request_json("GET", "/api/orchestration/threads/t/bounded?compactTurnItems=1", None, true).unwrap_err(), "too_large");
    server.join().unwrap();
}
#[test]
fn large_live_snapshot_is_allowed_only_for_bounded_thread_subscriptions() {
    for (tag, payload, allowed) in [
        ("device.subscribeState", json!({"acceptBoundedSnapshot":true}), false),
        ("orchestration.subscribeThread", json!({"threadId":"t"}), false),
        ("orchestration.subscribeThread", json!({"threadId":"t","acceptBoundedSnapshot":true}), true),
    ] {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let server = thread::spawn(move || {
            let (socket, _) = listener.accept().unwrap();
            let mut ws = tungstenite::accept(socket).unwrap();
            let _: Value = serde_json::from_str(ws.read().unwrap().to_text().unwrap()).unwrap();
            let item = json!({"kind":"snapshot","projection":{"padding":"x".repeat(9_000_000)}});
            let _ = ws.send(tungstenite::Message::Text(json!({"_tag":"Chunk","requestId":"1","values":[item]}).to_string().into()));
            let _ = ws.read();
        });
        let c = Connection::new(&base, None, Credential::bearer("synthetic").unwrap()).unwrap();
        let received = AtomicBool::new(false);
        let result = c.subscribe(tag, payload, || received.load(Ordering::Relaxed), |item| {
            if let Some(item) = item {
                assert_eq!(item["projection"]["padding"].as_str().unwrap().len(), 9_000_000);
                received.store(true, Ordering::Relaxed);
            }
            Ok(())
        });
        assert_eq!(received.load(Ordering::Relaxed), allowed);
        assert_eq!(result.is_ok(), allowed);
        server.join().unwrap();
    }
}
