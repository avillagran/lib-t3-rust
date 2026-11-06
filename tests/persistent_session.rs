use serde_json::{json, Value};
use std::{
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};
use t3_protocol::{Connection, Credential, SessionOptions};
use tungstenite::{accept, Message, WebSocket};
fn peer(
    body: impl FnOnce(WebSocket<TcpStream>) + Send + 'static,
) -> (Connection, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let join = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        body(accept(stream).unwrap())
    });
    (
        Connection::new(&base, None, Credential::bearer("synthetic-token").unwrap()).unwrap(),
        join,
    )
}
fn read(ws: &mut WebSocket<TcpStream>) -> Value {
    loop {
        if let Message::Text(t) = ws.read().unwrap() {
            return serde_json::from_str(&t).unwrap();
        }
    }
}
fn reply(ws: &mut WebSocket<TcpStream>, id: &Value, value: Value) {
    ws.send(Message::Text(
        json!({"_tag":"Exit","requestId":id,"exit":{"_tag":"Success","value":value}})
            .to_string()
            .into(),
    ))
    .unwrap()
}
#[test]
fn concurrent_calls_correlate_out_of_order_on_one_physical_socket() {
    let (c, server) = peer(|mut ws| {
        let first = read(&mut ws);
        let second = read(&mut ws);
        assert_ne!(first["id"], second["id"]);
        reply(&mut ws, &second["id"], second["payload"].clone());
        reply(&mut ws, &first["id"], first["payload"].clone());
        let third = read(&mut ws);
        reply(&mut ws, &third["id"], json!({"third":true}));
        let _ = ws.read();
    });
    let s = c.rpc_session(SessionOptions::default()).unwrap();
    let id = s.id().to_owned();
    assert_eq!(s.clone().id(), id);
    let first = s
        .start(
            "filesystem.getMetadata",
            json!({"first":true}),
            Duration::from_secs(2),
        )
        .unwrap();
    let second = s
        .start(
            "server.getConfig",
            json!({"second":true}),
            Duration::from_secs(2),
        )
        .unwrap();
    assert_eq!(first.wait().unwrap(), json!({"first":true}));
    assert_eq!(second.wait().unwrap(), json!({"second":true}));
    assert_eq!(
        s.rpc("server.getConfig", json!({}), Duration::from_secs(2))
            .unwrap(),
        json!({"third":true})
    );
    s.close();
    server.join().unwrap();
}
#[test]
fn transmitted_mutation_failure_is_uncertain_and_never_replayed() {
    let (c, server) = peer(|mut ws| {
        let call = read(&mut ws);
        assert_eq!(call["tag"], "terminal.write");
        ws.close(None).unwrap();
    });
    let s = c.rpc_session(SessionOptions::default()).unwrap();
    assert_eq!(
        s.rpc(
            "terminal.write",
            json!({"data":"once"}),
            Duration::from_secs(2)
        )
        .unwrap_err(),
        "rpc_outcome_unknown"
    );
    assert!(!s.is_open());
    assert!(s.failure().is_some());
    assert_eq!(
        s.start("terminal.write", json!({}), Duration::from_secs(1))
            .err()
            .unwrap(),
        "rpc_session_closed"
    );
    server.join().unwrap();
}
#[test]
fn timeout_interrupts_only_its_request_and_session_stays_usable() {
    let (c, server) = peer(|mut ws| {
        let call = read(&mut ws);
        let interrupt = read(&mut ws);
        assert_eq!(
            interrupt,
            json!({"_tag":"Interrupt","requestId":call["id"]})
        );
        reply(&mut ws, &call["id"], json!("late"));
        let next = read(&mut ws);
        reply(&mut ws, &next["id"], json!("next"));
        let _ = ws.read();
    });
    let s = c.rpc_session(SessionOptions::default()).unwrap();
    assert_eq!(
        s.rpc("terminal.write", json!({}), Duration::from_millis(500))
            .unwrap_err(),
        "rpc_outcome_unknown"
    );
    assert_eq!(
        s.rpc("server.getConfig", json!({}), Duration::from_secs(2))
            .unwrap(),
        json!("next")
    );
    s.close();
    server.join().unwrap();
}
#[test]
fn dropping_or_cancelling_a_call_interrupts_without_replaying() {
    let (seen_tx, seen_rx) = std::sync::mpsc::channel();
    let (c, server) = peer(move |mut ws| {
        let call = read(&mut ws);
        seen_tx.send(()).unwrap();
        let interrupt = read(&mut ws);
        assert_eq!(interrupt["_tag"], "Interrupt");
        assert_eq!(interrupt["requestId"], call["id"]);
        let next = read(&mut ws);
        reply(&mut ws, &next["id"], json!(true));
        let _ = ws.read();
    });
    let s = c.rpc_session(SessionOptions::default()).unwrap();
    let call = s
        .start("terminal.write", json!({}), Duration::from_secs(2))
        .unwrap();
    seen_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    call.cancel();
    assert_eq!(call.wait().unwrap_err(), "rpc_cancelled_outcome_unknown");
    assert_eq!(
        s.rpc("server.getConfig", json!({}), Duration::from_secs(2))
            .unwrap(),
        json!(true)
    );
    s.close();
    server.join().unwrap();
}
#[test]
fn bounded_unread_results_hold_capacity_until_consumed() {
    let (c, server) = peer(|mut ws| {
        let first = read(&mut ws);
        reply(&mut ws, &first["id"], json!(true));
        let next = read(&mut ws);
        reply(&mut ws, &next["id"], json!(false));
        let _ = ws.read();
    });
    let s = c
        .rpc_session(SessionOptions {
            max_in_flight: 1,
            ..Default::default()
        })
        .unwrap();
    let call = s
        .start("server.getConfig", json!({}), Duration::from_secs(2))
        .unwrap();
    thread::sleep(Duration::from_millis(80));
    assert_eq!(
        s.start("server.getConfig", json!({}), Duration::from_secs(2))
            .err()
            .unwrap(),
        "rpc_session_busy"
    );
    assert_eq!(call.wait().unwrap(), json!(true));
    assert_eq!(
        s.rpc("server.getConfig", json!({}), Duration::from_secs(2))
            .unwrap(),
        json!(false)
    );
    s.close();
    server.join().unwrap();
}
#[test]
fn session_replacement_has_a_new_opaque_identity() {
    let (c, a) = peer(|mut ws| {
        let _ = ws.read();
    });
    let first = c.rpc_session(Default::default()).unwrap();
    let id = first.id().to_owned();
    first.close();
    a.join().unwrap();
    let (c, b) = peer(|mut ws| {
        let _ = ws.read();
    });
    let next = c.rpc_session(Default::default()).unwrap();
    assert_ne!(id, next.id());
    assert!(!next.id().contains("fixture"));
    next.close();
    b.join().unwrap();
}
#[test]
fn local_option_and_frame_bounds_do_not_emit_a_request() {
    let (c, server) = peer(|mut ws| {
        let end = ws.read();
        assert!(matches!(end, Ok(Message::Close(_)) | Err(_)));
    });
    assert!(c
        .rpc_session(SessionOptions {
            max_in_flight: 0,
            ..Default::default()
        })
        .is_err());
    let s = c
        .rpc_session(SessionOptions {
            max_message_bytes: 128,
            max_queued_bytes: 128,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(
        s.start(
            "terminal.write",
            json!({"data":"x".repeat(256)}),
            Duration::from_secs(1)
        )
        .err()
        .unwrap(),
        "rpc_request_too_large"
    );
    assert!(s.start("bad\n", json!({}), Duration::from_secs(1)).is_err());
    s.close();
    server.join().unwrap();
}
#[test]
fn application_failure_and_both_heartbeat_forms_keep_the_session_usable() {
    let (c, server) = peer(|mut ws| {
        let call = read(&mut ws);
        ws.send(Message::Text(json!({"_tag":"Ping"}).to_string().into()))
            .unwrap();
        assert_eq!(read(&mut ws), json!({"_tag":"Pong"}));
        ws.send(Message::Ping(vec![1, 2, 3].into())).unwrap();
        assert!(matches!(ws.read().unwrap(),Message::Pong(p) if p.as_ref()==[1,2,3]));
        ws.send(Message::Text(json!({"_tag":"Exit","requestId":call["id"],"exit":{"_tag":"Failure","cause":{"_tag":"Fail","error":{"_tag":"PermissionDenied"}}}}).to_string().into())).unwrap();
        let next = read(&mut ws);
        reply(&mut ws, &next["id"], json!(true));
        let _ = ws.read();
    });
    let s = c.rpc_session(Default::default()).unwrap();
    assert!(s.rpc("denied", json!({}), Duration::from_secs(2)).is_err());
    assert!(s.is_open());
    assert_eq!(
        s.rpc("next", json!({}), Duration::from_secs(2)).unwrap(),
        json!(true)
    );
    s.close();
    server.join().unwrap();
}
#[test]
fn malformed_transport_fails_all_transmitted_calls_without_reconnect() {
    let (c, server) = peer(|mut ws| {
        read(&mut ws);
        read(&mut ws);
        ws.send(Message::Text("invalid-json".into())).unwrap();
        let _ = ws.read();
    });
    let s = c.rpc_session(Default::default()).unwrap();
    let a = s.start("one", json!({}), Duration::from_secs(2)).unwrap();
    let b = s.start("two", json!({}), Duration::from_secs(2)).unwrap();
    assert_eq!(a.wait().unwrap_err(), "rpc_outcome_unknown");
    assert_eq!(b.wait().unwrap_err(), "rpc_outcome_unknown");
    assert_eq!(s.failure().as_deref(), Some("malformed_rpc"));
    server.join().unwrap();
}
#[test]
fn queued_byte_rejection_releases_the_request_permit() {
    let (c, server) = peer(|mut ws| {
        let call = read(&mut ws);
        assert_eq!(call["tag"], "small");
        reply(&mut ws, &call["id"], json!(true));
        let _ = ws.read();
    });
    let s = c
        .rpc_session(SessionOptions {
            max_in_flight: 1,
            max_message_bytes: 4096,
            max_queued_bytes: 128,
        })
        .unwrap();
    assert_eq!(
        s.start(
            "large",
            json!({"data":"x".repeat(1024)}),
            Duration::from_secs(1)
        )
        .err()
        .unwrap(),
        "rpc_session_busy"
    );
    assert_eq!(
        s.rpc("small", json!({}), Duration::from_secs(2)).unwrap(),
        json!(true)
    );
    s.close();
    server.join().unwrap();
}
#[test]
fn dropping_the_last_session_owner_completes_pending_calls() {
    let (c, server) = peer(|mut ws| {
        let _ = ws.read();
    });
    let s = c.rpc_session(Default::default()).unwrap();
    let call = s.start("one", json!({}), Duration::from_secs(2)).unwrap();
    drop(s);
    let result = call.wait();
    assert!(matches!(
        result.as_ref().map_err(String::as_str),
        Err("rpc_session_closed" | "rpc_outcome_unknown")
    ));
    server.join().unwrap();
}
#[test]
fn typed_function_validation_and_payloads_share_the_existing_registry() {
    let (c, server) = peer(|mut ws| {
        let call = read(&mut ws);
        assert_eq!(call["tag"], "filesystem.getMetadata");
        assert_eq!(call["payload"], json!({"paths":["/work/a.ts"]}));
        reply(&mut ws, &call["id"], json!({"entries":[{"kind":"file"}]}));
        let _ = ws.read();
    });
    let s = c.rpc_session(Default::default()).unwrap();
    assert!(s
        .start_function("filesystem.getMetadata", &json!({"paths":[]}), None)
        .is_err());
    assert_eq!(
        s.start_function("device.subscribeDeviceState", &json!({}), None)
            .err()
            .unwrap(),
        "stream_function"
    );
    assert_eq!(
        s.call_function(
            "filesystem.getMetadata",
            &json!({"paths":["/work/a.ts"]}),
            None
        )
        .unwrap(),
        json!({"entries":[{"kind":"file"}]})
    );
    s.close();
    server.join().unwrap();
}

#[test]
fn admission_comparison_is_private_and_does_not_substitute_for_socket_identity() {
    let a = Connection::new(
        "http://127.0.0.1:1234",
        None,
        Credential::bearer("synthetic-token").unwrap(),
    )
    .unwrap();
    let same = Connection::new(
        "http://127.0.0.1:1234",
        None,
        Credential::bearer("synthetic-token").unwrap(),
    )
    .unwrap();
    let auth = Connection::new(
        "http://127.0.0.1:1234",
        None,
        Credential::bearer("another-synthetic-token").unwrap(),
    )
    .unwrap();
    let route = Connection::new(
        "http://127.0.0.1:1235",
        None,
        Credential::bearer("synthetic-token").unwrap(),
    )
    .unwrap();
    assert!(a.same_session_admission(&same));
    assert!(a.same_session_admission(&a.clone()));
    assert!(!a.same_session_admission(&auth));
    assert!(!a.same_session_admission(&route));
}

#[test]
fn lifecycle_observer_does_not_keep_the_last_owner_alive() {
    let (c, server) = peer(|mut ws| {
        assert!(matches!(ws.read(), Ok(Message::Close(_)) | Err(_)));
    });
    let s = c.rpc_session(Default::default()).unwrap();
    let closed = s.closed();
    drop(s);
    assert_eq!(closed.wait().unwrap(), None);
    server.join().unwrap();
}
