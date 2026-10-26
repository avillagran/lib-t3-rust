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
#[test]
fn restores_exact_native_local_prefix_before_inherited_tail_only_when_marked() {
    use t3_protocol::bounded_snapshot::restore;
    let legacy = json!({"snapshotSequence":7,"historyCursor":"opaque","projection":{"thread":{"id":"t"},"turnItems":[{"id":"one"},{"id":"two"},{"id":"tail"}],"visibleTurnItems":[{"visibility":"inherited","sourceThreadId":"parent","item":{"id":"inherited"}},{"visibility":"local","sourceThreadId":"t","item":{"id":"one"}},{"visibility":"synthetic","sourceThreadId":"t","item":{"id":"two"}}]}});
    assert_eq!(restore(legacy.clone()).unwrap(), legacy);
    let mut compact = legacy.clone();
    compact["turnItemsOmitLocalVisible"] = json!(true);
    compact["projection"]["turnItems"] = json!([{"id":"tail"}]);
    let restored = restore(compact.clone()).unwrap();
    assert_eq!(restored, legacy);
    assert_eq!(restore(restored.clone()).unwrap(), restored);
    compact["turnItemsOmitLocalVisible"] = json!(false);
    assert_eq!(restore(compact.clone()).unwrap_err(), "invalid_compact_snapshot");
    compact["turnItemsOmitLocalVisible"] = json!(true);
    let mut no_thread = compact.clone();
    no_thread["projection"]["thread"] = json!({});
    assert_eq!(restore(no_thread).unwrap_err(), "invalid_compact_snapshot");
    let mut bad_item = compact.clone();
    bad_item["projection"]["visibleTurnItems"][1]["item"] = json!("one");
    assert_eq!(restore(bad_item).unwrap_err(), "invalid_compact_snapshot");
    compact["projection"]["visibleTurnItems"] = Value::Null;
    assert_eq!(restore(compact).unwrap_err(), "invalid_compact_snapshot");
}
#[test]
fn bounded_thread_path_validates_and_encodes_the_id() {
    use t3_protocol::bounded_snapshot::path;
    assert_eq!(path("t-1").unwrap(), "/api/orchestration/threads/t-1/bounded?compactTurnItems=1&dedupeBodies=1&compactIds=1");
    // Segment encoding, as ot3 did: a literal % is escaped, never decoded into a slash.
    assert_eq!(path("a:b%2F.c_d").unwrap(), "/api/orchestration/threads/a:b%252F.c_d/bounded?compactTurnItems=1&dedupeBodies=1&compactIds=1");
    let long = "a".repeat(2049);
    for bad in ["", "-t", "a/b", "a?b", "a b", "../x", long.as_str()] {
        assert_eq!(path(bad).unwrap_err(), "invalid_id", "{bad}");
    }
}
#[test]
fn bounded_thread_reads_a_large_compact_snapshot_and_restores_it() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let item = json!({"id":"large","text":"x".repeat(9_000_000)});
    let compact = json!({"snapshotSequence":3,"turnItemsOmitLocalVisible":true,"projection":{"thread":{"id":"t"},"messages":[],"turnItems":[{"id":"tail"}],"visibleTurnItems":[{"visibility":"local","sourceThreadId":"t","item":item}]}}).to_string();
    assert!(compact.len() > 8 * 1024 * 1024);
    let server = thread::spawn(move || {
        let (mut s, _) = listener.accept().unwrap();
        let h = header(&mut s);
        assert!(h.starts_with("GET /api/orchestration/threads/t/bounded?compactTurnItems=1&dedupeBodies=1&compactIds=1 HTTP"), "{h}");
        assert!(h.to_lowercase().contains("x-t3-orchestration-protocol: 2"));
        write!(s, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", compact.len()).unwrap();
        let _ = s.write_all(compact.as_bytes());
    });
    let c = Connection::new(&base, None, Credential::bearer("synthetic").unwrap()).unwrap();
    let snapshot = c.bounded_thread("t").unwrap();
    assert!(snapshot.get("turnItemsOmitLocalVisible").is_none());
    let items = snapshot["projection"]["turnItems"].as_array().unwrap();
    assert_eq!((items[0]["id"].as_str(), items[1]["id"].as_str()), (Some("large"), Some("tail")));
    server.join().unwrap();
    assert_eq!(c.bounded_thread("../shell").unwrap_err(), "invalid_id");
}
