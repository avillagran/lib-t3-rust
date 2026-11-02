use serde_json::{json, Value};
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::atomic::{AtomicBool, Ordering},
    thread,
};
use t3_protocol::{Connection, Credential};

fn shell() -> Value {
    let threads: Vec<_> = (0..10_000)
        .map(|id| json!({"id":format!("t-{id}"),"title":"x".repeat(910)}))
        .collect();
    json!({"schemaVersion":2,"snapshotSequence":7,"projects":[],"threads":threads})
}

#[test]
fn large_shell_loads_over_http_and_subscription_including_reconnect() {
    let snapshot = shell();
    assert!(snapshot.to_string().len() > 8 * 1024 * 1024);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let expected = snapshot.clone();
    let server = thread::spawn(move || {
        for path in [
            "/api/orchestration/shell",
            "/api/orchestration/shell?ignored=1",
        ] {
            let (mut s, _) = listener.accept().unwrap();
            let mut h = Vec::new();
            while !h.ends_with(b"\r\n\r\n") {
                let mut b = [0];
                s.read_exact(&mut b).unwrap();
                h.push(b[0]);
            }
            assert!(String::from_utf8(h)
                .unwrap()
                .starts_with(&format!("GET {path} HTTP")));
            let body = snapshot.to_string();
            write!(s, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).unwrap();
            s.write_all(body.as_bytes()).unwrap();
        }
        for after in [None, Some(7)] {
            let (s, _) = listener.accept().unwrap();
            let mut ws = tungstenite::accept(s).unwrap();
            let request: Value =
                serde_json::from_str(ws.read().unwrap().to_text().unwrap()).unwrap();
            assert_eq!(request["payload"]["afterSequence"].as_u64(), after);
            let frame = json!({"_tag":"Chunk","requestId":"1","values":[{"kind":"snapshot","snapshot":snapshot}]});
            ws.send(tungstenite::Message::Text(frame.to_string().into()))
                .unwrap();
            let _ = ws.read();
        }
    });
    let c = Connection::new(&base, None, Credential::bearer("synthetic").unwrap()).unwrap();
    for path in [
        "/api/orchestration/shell",
        "/api/orchestration/shell?ignored=1",
    ] {
        assert_eq!(c.request_json("GET", path, None, true).unwrap(), expected);
    }
    for payload in [json!({}), json!({"afterSequence":7})] {
        let received = AtomicBool::new(false);
        c.subscribe(
            "orchestration.subscribeShell",
            payload,
            || received.load(Ordering::Relaxed),
            |item| {
                if let Some(item) = item {
                    assert_eq!(item["snapshot"], expected);
                    received.store(true, Ordering::Relaxed);
                }
                Ok(())
            },
        )
        .unwrap();
        assert!(received.load(Ordering::Relaxed));
    }
    server.join().unwrap();
}
