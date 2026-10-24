use t3_protocol as protocol;
use serde_json::{json, Value};
use std::{
    net::TcpListener,
    thread,
    time::{Duration, Instant},
};

/// Fake server for one subscription: answers the client's Ping with a Pong only when `answer`.
fn server(answer: bool) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let handle = thread::spawn(move || {
        let (socket, _) = listener.accept().unwrap();
        socket.set_read_timeout(Some(Duration::from_secs(8))).unwrap();
        let mut ws = tungstenite::accept(socket).unwrap();
        let request: Value = serde_json::from_str(ws.read().unwrap().to_text().unwrap()).unwrap();
        assert_eq!(request["tag"], "orchestration.subscribeShell");
        while let Ok(message) = ws.read() {
            let Ok(text) = message.to_text() else { continue };
            let Ok(v) = serde_json::from_str::<Value>(text) else { continue };
            if v["_tag"] == "Interrupt" {
                break;
            }
            if v["_tag"] == "Ping" && answer {
                ws.send(tungstenite::Message::Text(json!({"_tag":"Pong"}).to_string().into())).unwrap();
            }
        }
    });
    (base, handle)
}

#[test]
fn a_wakeup_probe_drops_a_silent_socket_within_seconds() {
    let (base, handle) = server(false);
    let c = protocol::Connection::new(&base, None, protocol::Credential::bearer("synthetic").unwrap()).unwrap();
    let start = Instant::now();
    let mut asked = false;
    let result = c.subscribe_probed("orchestration.subscribeShell", json!({}), || false, || !std::mem::replace(&mut asked, true), |_| Ok(()));
    assert_eq!(result.unwrap_err(), "stream_timeout");
    assert!(start.elapsed() < Duration::from_secs(5), "{:?}", start.elapsed());
    drop(c);
    let _ = handle.join();
}

#[test]
fn a_wakeup_probe_keeps_a_socket_that_answers() {
    let (base, handle) = server(true);
    let c = protocol::Connection::new(&base, None, protocol::Credential::bearer("synthetic").unwrap()).unwrap();
    let start = Instant::now();
    let mut asked = false;
    let result = c.subscribe_probed("orchestration.subscribeShell", json!({}), || start.elapsed() > Duration::from_secs(5), || !std::mem::replace(&mut asked, true), |_| Ok(()));
    assert!(result.is_ok(), "{result:?}");
    handle.join().unwrap();
}

#[test]
fn a_silent_subscription_pings_every_five_seconds_and_drops_after_twenty() {
    assert_eq!(protocol::stream::STREAM_PING_INTERVAL, Duration::from_secs(5));
    assert_eq!(protocol::stream::STREAM_IDLE_TIMEOUT, Duration::from_secs(20));
    assert_eq!(protocol::stream::STREAM_PROBE_TIMEOUT, Duration::from_secs(3));
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        let (socket, _) = listener.accept().unwrap();
        socket.set_read_timeout(Some(Duration::from_secs(30))).unwrap();
        let mut ws = tungstenite::accept(socket).unwrap();
        let _request = ws.read().unwrap();
        let start = Instant::now();
        let mut pings = vec![];
        // Never answers: only the client's own pings cross the socket until it gives up.
        while let Ok(message) = ws.read() {
            let v: Value = serde_json::from_str(message.to_text().unwrap()).unwrap();
            assert_eq!(v["_tag"], "Ping");
            pings.push(start.elapsed());
        }
        pings
    });
    let c = protocol::Connection::new(&base, None, protocol::Credential::bearer("synthetic").unwrap()).unwrap();
    let start = Instant::now();
    let result = c.subscribe("orchestration.subscribeThread", json!({"threadId":"t"}), || false, |_| Ok(()));
    let elapsed = start.elapsed();
    assert_eq!(result.unwrap_err(), "stream_timeout");
    assert!(elapsed >= Duration::from_secs(20) && elapsed < Duration::from_secs(23), "{elapsed:?}");
    let pings = server.join().unwrap();
    assert!((3..=4).contains(&pings.len()), "{pings:?}");
    let first = pings[0];
    assert!(first >= Duration::from_millis(4800) && first < Duration::from_millis(6000), "{pings:?}");
    for pair in pings.windows(2) {
        let gap = pair[1] - pair[0];
        assert!(gap >= Duration::from_millis(4800) && gap < Duration::from_millis(5600), "{pings:?}");
    }
}
