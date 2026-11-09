//! Genuine authenticated WebSocket frames; run only in the canonical sandbox.
use serde_json::value::RawValue;
use std::{
    cell::{Cell, RefCell},
    net::TcpListener,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};
use t3_protocol::{wtf16::Wtf16String, Connection, Credential, RawStreamExit, RawStreamTick};
use tungstenite::{
    handshake::server::{Callback, ErrorResponse, Request, Response},
    Message,
};
struct Upgrade;
impl Callback for Upgrade {
    fn on_request(self, request: &Request, response: Response) -> Result<Response, ErrorResponse> {
        assert_eq!(
            request.headers()["authorization"],
            "Bearer private-raw-stream"
        );
        Ok(response)
    }
}
fn raw(text: &str) -> Box<RawValue> {
    RawValue::from_string(text.into()).unwrap()
}
fn connection(endpoint: &str) -> Connection {
    Connection::new(
        endpoint,
        None,
        Credential::bearer("private-raw-stream").unwrap(),
    )
    .unwrap()
}
fn peer(
    run: impl FnOnce(tungstenite::WebSocket<std::net::TcpStream>) + Send + 'static,
) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    let task = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut socket = tungstenite::accept_hdr(stream, Upgrade).unwrap();
        let Message::Text(request) = socket.read().unwrap() else {
            panic!("No actual stream Request")
        };
        let request: serde_json::Value = serde_json::from_str(&request).unwrap();
        assert_eq!(request["_tag"], "Request");
        assert_eq!(request["id"], "1");
        run(socket)
    });
    (endpoint, task)
}
fn frame(socket: &mut tungstenite::WebSocket<std::net::TcpStream>, text: &str) {
    socket.send(Message::Text(text.to_owned().into())).unwrap()
}
fn read_json(socket: &mut tungstenite::WebSocket<std::net::TcpStream>) -> serde_json::Value {
    loop {
        match socket.read().unwrap() {
            Message::Text(text) => return serde_json::from_str(&text).unwrap(),
            Message::Ping(bytes) => socket.send(Message::Pong(bytes)).unwrap(),
            other => panic!("Unexpected stream frame {other:?}"),
        }
    }
}
#[test]
fn shell_snapshot_patch_title_units_chunk_order_and_real_ack() {
    let count = Arc::new(AtomicUsize::new(0));
    let observed = count.clone();
    let (endpoint, server) = peer(move |mut socket| {
        frame(
            &mut socket,
            r#"{"_tag":"Chunk","requestId":"other","values":["\ud800"]}"#,
        );
        frame(&mut socket, r#"{"_tag":"Ping"}"#);
        assert_eq!(read_json(&mut socket)["_tag"], "Pong");
        socket.send(Message::Ping(vec![1, 2, 3].into())).unwrap();
        assert!(matches!(socket.read().unwrap(),Message::Pong(bytes)if bytes.as_ref()==[1,2,3]));
        frame(
            &mut socket,
            r#"{"_tag":"Ch\u0075nk","requestId":"\u0031","values":[null,{"kind":"snapshot","snapshot":{"snapshotSequence":10,"threads":[{"id":"t","title":"ab\ud800"}]}},{"kind":"thread.updated","sequence":11,"thread":{"id":"t","title":"ab\udc00"}}]}"#,
        );
        assert_eq!(read_json(&mut socket)["_tag"], "Ack");
        assert_eq!(observed.load(Ordering::SeqCst), 4);
        frame(
            &mut socket,
            r#"{"_tag":"Exit","requestId":"1","exit":{"_tag":"Success"}}"#,
        );
    });
    let items = RefCell::new(Vec::new());
    let result = connection(&endpoint)
        .subscribe_raw(
            "orchestration.subscribeShell",
            &raw("{}"),
            || false,
            |tick| {
                match tick {
                    RawStreamTick::Item(item) => {
                        items.borrow_mut().push(item);
                        count.fetch_add(1, Ordering::SeqCst);
                    }
                    RawStreamTick::ChunkEnd => {
                        count.fetch_add(1, Ordering::SeqCst);
                    }
                    RawStreamTick::Idle => {}
                }
                Ok(())
            },
        )
        .unwrap();
    server.join().unwrap();
    assert!(matches!(result, RawStreamExit::Success(None)));
    let items = items.into_inner();
    assert_eq!(items.len(), 3);
    assert_eq!(items[0].get(), "null");
    let fields: std::collections::BTreeMap<String, Box<RawValue>> =
        serde_json::from_str(items[2].get()).unwrap();
    let fields: std::collections::BTreeMap<String, Box<RawValue>> =
        serde_json::from_str(fields["thread"].get()).unwrap();
    assert_eq!(
        Wtf16String::from_json(&fields["title"]).unwrap().units(),
        [97, 98, 0xdc00]
    );
    assert!(serde_json::from_str::<serde_json::Value>(items[1].get()).is_err());
}
#[test]
fn native_failure_cause_and_explicit_null_remain_raw() {
    for (failure, wire) in [
        (
            true,
            r#"{"_tag":"Exit","requestId":"1","exit":{"_tag":"Failure","cause":{"message":"ab\udc00"}}}"#,
        ),
        (
            false,
            r#"{"_tag":"Exit","requestId":"1","exit":{"_tag":"Success","value":null}}"#,
        ),
    ] {
        let (endpoint, server) = peer(move |mut socket| frame(&mut socket, wire));
        let result = connection(&endpoint)
            .subscribe_raw(
                "orchestration.subscribeShell",
                &raw("{}"),
                || false,
                |_| panic!("Exit is not an item"),
            )
            .unwrap();
        server.join().unwrap();
        match result {
            RawStreamExit::Failure(cause) if failure => {
                let fields: std::collections::BTreeMap<String, Box<RawValue>> =
                    serde_json::from_str(cause.get()).unwrap();
                assert_eq!(
                    Wtf16String::from_json(&fields["message"]).unwrap().units(),
                    [97, 98, 0xdc00]
                );
            }
            RawStreamExit::Success(Some(value)) if !failure => assert_eq!(value.get(), "null"),
            other => panic!("Wrong raw exit {other:?}"),
        }
    }
}
#[test]
fn cancellation_preserves_chunk_ack_then_interrupt_and_never_retries() {
    let (endpoint, server) = peer(|mut socket| {
        frame(
            &mut socket,
            r#"{"_tag":"Chunk","requestId":"1","values":["\ud800"]}"#,
        );
        assert_eq!(read_json(&mut socket)["_tag"], "Ack");
        assert_eq!(read_json(&mut socket)["_tag"], "Interrupt");
    });
    let stop = Cell::new(false);
    let result = connection(&endpoint)
        .subscribe_raw(
            "orchestration.subscribeShell",
            &raw("{}"),
            || stop.get(),
            |tick| {
                if matches!(tick, RawStreamTick::ChunkEnd) {
                    stop.set(true)
                }
                Ok(())
            },
        )
        .unwrap();
    server.join().unwrap();
    assert!(matches!(result, RawStreamExit::Cancelled));
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let result = connection(&format!("http://{}", listener.local_addr().unwrap()))
        .subscribe_raw(
            "orchestration.subscribeShell",
            &raw("{}"),
            || true,
            |_| panic!("Cancelled before factory"),
        )
        .unwrap();
    assert!(matches!(result, RawStreamExit::Cancelled));
    assert!(matches!(listener.accept(),Err(error)if error.kind()==std::io::ErrorKind::WouldBlock));
}
#[test]
fn callback_failure_refuses_ack_and_retires_its_only_socket() {
    let (endpoint, server) = peer(|mut socket| {
        frame(
            &mut socket,
            r#"{"_tag":"Chunk","requestId":"1","values":["\ud800","second"]}"#,
        );
        match socket.read() {
            Ok(Message::Close(_)) => {}
            Err(
                tungstenite::Error::ConnectionClosed
                | tungstenite::Error::AlreadyClosed
                | tungstenite::Error::Protocol(_),
            ) => {}
            Err(tungstenite::Error::Io(error))
                if !matches!(
                    error.kind(),
                    std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                ) => {}
            other => panic!("Callback failure acknowledged or leaked {other:?}"),
        }
    });
    let count = Cell::new(0);
    let result = connection(&endpoint).subscribe_raw(
        "orchestration.subscribeShell",
        &raw("{}"),
        || false,
        |tick| {
            assert!(matches!(tick, RawStreamTick::Item(_)));
            count.set(count.get() + 1);
            Err("private_callback_refusal".into())
        },
    );
    server.join().unwrap();
    assert_eq!(result.unwrap_err(), "private_callback_refusal");
    assert_eq!(count.get(), 1);
}
#[test]
fn malformed_wire_is_an_error_instead_of_a_reconnect() {
    let (endpoint, server) = peer(|mut socket| {
        frame(
            &mut socket,
            r#"{"_tag":"Chunk","requestId":"1","values":["\uxxxx"]}"#,
        )
    });
    let result = connection(&endpoint).subscribe_raw(
        "orchestration.subscribeShell",
        &raw("{}"),
        || false,
        |_| panic!("Invalid JSON delivered"),
    );
    server.join().unwrap();
    assert_eq!(result.unwrap_err(), "malformed_stream");
}
#[test]
fn wake_probe_uses_original_absolute_three_second_budget() {
    let (endpoint, server) = peer(|mut socket| {
        assert_eq!(read_json(&mut socket)["_tag"], "Ping");
        match socket.read() {
            Ok(Message::Close(_)) => {}
            Err(
                tungstenite::Error::ConnectionClosed
                | tungstenite::Error::AlreadyClosed
                | tungstenite::Error::Protocol(_),
            ) => {}
            Err(tungstenite::Error::Io(error))
                if !matches!(
                    error.kind(),
                    std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                ) => {}
            other => panic!("Probe did not close its owned socket {other:?}"),
        }
    });
    let asked = Cell::new(false);
    let started = Instant::now();
    let result = connection(&endpoint).subscribe_raw_probed(
        "orchestration.subscribeShell",
        &raw("{}"),
        || false,
        || {
            if asked.replace(true) {
                None
            } else {
                Some(started)
            }
        },
        |_| Ok(()),
    );
    server.join().unwrap();
    assert_eq!(result.unwrap_err(), "stream_timeout");
    assert!(
        started.elapsed() >= Duration::from_secs(3) && started.elapsed() < Duration::from_secs(5)
    );
}
#[test]
fn bounded_thread_opt_in_keeps_the_existing_sixty_four_mib_factory() {
    let (endpoint, server) = peer(|mut socket| {
        let item = "s".repeat(8 * 1024 * 1024 + 1);
        frame(
            &mut socket,
            &format!(
                "{{\"_tag\":\"Chunk\",\"requestId\":\"1\",\"values\":[{}]}}",
                serde_json::to_string(&item).unwrap()
            ),
        );
        assert_eq!(read_json(&mut socket)["_tag"], "Ack");
        frame(
            &mut socket,
            r#"{"_tag":"Exit","requestId":"1","exit":{"_tag":"Success"}}"#,
        );
    });
    let count = Cell::new(0);
    let mut sub = t3_protocol::api::streams::ThreadSub::new("t");
    sub.accept_bounded_snapshot = Some(true);
    let result = t3_protocol::api::streams::subscribe_thread_raw(
        &connection(&endpoint),
        &sub,
        || false,
        || None,
        |tick| {
            if let RawStreamTick::Item(item) = tick {
                assert_eq!(item.get().len(), 8 * 1024 * 1024 + 3);
                count.set(count.get() + 1)
            }
            Ok(())
        },
    )
    .unwrap();
    server.join().unwrap();
    assert!(matches!(result, RawStreamExit::Success(None)));
    assert_eq!(count.get(), 1);
}

#[test]
fn typed_raw_shell_and_thread_keep_original_admission_before_the_socket() {
    use t3_protocol::api::streams::{self, ShellSub, ThreadSub};
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let conn = connection(&format!("http://{}", listener.local_addr().unwrap()));
    let shell = ShellSub {
        after_sequence: Some(1 << 53),
        ..ShellSub::default()
    };
    assert_eq!(
        streams::subscribe_shell_raw(
            &conn,
            &shell,
            || false,
            || None,
            |_| panic!("Invalid shell admitted")
        )
        .unwrap_err(),
        "invalid_after_sequence"
    );
    assert_eq!(
        streams::subscribe_thread_raw(
            &conn,
            &ThreadSub::new("../invalid"),
            || false,
            || None,
            |_| panic!("Invalid thread admitted")
        )
        .unwrap_err(),
        "invalid_thread_id"
    );
    assert!(matches!(listener.accept(),Err(error)if error.kind()==std::io::ErrorKind::WouldBlock));
}
