//! Physical async speech opening, flow control and retirement in android-sandbox.
use serde_json::{json, Value};
use std::{
    future::{poll_fn, Future},
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread,
    time::{Duration, Instant},
};
use t3_protocol::{
    speech::{
        CancellableSpeech, LiveEvent, LiveFailure, RequestCancellation, TranscriptionOptions,
    },
    Connection, Credential,
};
use tungstenite::Message;
fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}
fn fixture(
    run: impl FnOnce(&TcpListener) + Send + 'static,
) -> (CancellableSpeech, thread::JoinHandle<()>) {
    assert!(std::env::var_os("ANDROID_SANDBOX").is_some());
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        run(&listener);
        assert!(listener.accept().is_err(), "No reconnect or ticket replay");
    });
    // Build/drop the existing blocking Connection outside the caller runtime.
    let client = Connection::new(
        &base,
        None,
        Credential::bearer("private-async-voice").unwrap(),
    )
    .unwrap()
    .cancellable_speech()
    .unwrap();
    (client, server)
}
fn accept(listener: &TcpListener) -> TcpStream {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match listener.accept() {
            Ok((socket, _)) => {
                socket
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                socket
                    .set_write_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                return socket;
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(Instant::now() < deadline);
                thread::sleep(Duration::from_millis(2))
            }
            Err(error) => panic!("{error}"),
        }
    }
}
fn header(socket: &mut TcpStream) -> String {
    let mut bytes = vec![];
    while !bytes.ends_with(b"\r\n\r\n") {
        let mut byte = [0];
        socket.read_exact(&mut byte).unwrap();
        bytes.push(byte[0]);
        assert!(bytes.len() < 65536)
    }
    String::from_utf8(bytes).unwrap()
}
fn ticket(listener: &TcpListener) {
    let mut socket = accept(listener);
    let header = header(&mut socket);
    assert!(header.starts_with("POST /api/auth/websocket-ticket/media "));
    assert!(header
        .to_ascii_lowercase()
        .contains("authorization: bearer private-async-voice\r\n"));
    let body = json!({"ticket":"private-async-ticket"}).to_string();
    write!(socket,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
}
struct Upgrade;
impl tungstenite::handshake::server::Callback for Upgrade {
    fn on_request(
        self,
        request: &tungstenite::handshake::server::Request,
        response: tungstenite::handshake::server::Response,
    ) -> Result<
        tungstenite::handshake::server::Response,
        tungstenite::handshake::server::ErrorResponse,
    > {
        assert_eq!(
            request.uri().to_string(),
            "/ws/voice?wsTicket=private-async-ticket"
        );
        assert!(request.headers().get("authorization").is_none());
        Ok(response)
    }
}
fn socket(listener: &TcpListener) -> tungstenite::WebSocket<TcpStream> {
    ticket(listener);
    let mut socket = tungstenite::accept_hdr(accept(listener), Upgrade).unwrap();
    let message = socket.read().unwrap();
    let start: Value = serde_json::from_str(message.to_text().unwrap()).unwrap();
    assert_eq!(
        start,
        json!({"type":"start","options":serde_json::from_str::<Value>(&TranscriptionOptions::default().encode_json()).unwrap()})
    );
    socket
}
fn send(socket: &mut tungstenite::WebSocket<TcpStream>, value: Value) {
    socket
        .send(Message::Text(value.to_string().into()))
        .unwrap()
}
fn closed(socket: &mut TcpStream) {
    match socket.read(&mut [0]) {
        Ok(0) => {}
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::ConnectionAborted
            ) => {}
        value => panic!("Owned socket must close: {value:?}"),
    }
}
#[test]
fn actual_async_ticket_window_equal_revisions_and_final_text() {
    let (client, server) = fixture(|listener| {
        let mut socket = socket(listener);
        send(&mut socket, json!({"type":"ready"}));
        for _ in 0..2 {
            let frame = socket.read().unwrap();
            assert!(frame.is_binary());
            assert_eq!(frame.len(), 64000);
            assert!(frame
                .into_data()
                .as_chunks::<4>()
                .0
                .iter()
                .all(|chunk| chunk == &0.125f32.to_le_bytes()));
        }
        socket
            .get_mut()
            .set_read_timeout(Some(Duration::from_millis(80)))
            .unwrap();
        assert!(
            matches!(socket.read(),Err(tungstenite::Error::Io(error))if matches!(error.kind(),std::io::ErrorKind::WouldBlock|std::io::ErrorKind::TimedOut))
        );
        socket
            .get_mut()
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        send(
            &mut socket,
            json!({"type":"update","revision":1,"text":{"committed":"Synthetic ","tentative":"text"}}),
        );
        assert_eq!(socket.read().unwrap().len(), 64000);
        send(
            &mut socket,
            json!({"type":"update","revision":1,"text":null}),
        );
        send(
            &mut socket,
            json!({"type":"update","revision":2,"text":{"committed":"Synthetic text","tentative":""}}),
        );
        assert_eq!(
            serde_json::from_str::<Value>(socket.read().unwrap().to_text().unwrap()).unwrap(),
            json!({"type":"finish"})
        );
        send(
            &mut socket,
            json!({"type":"finished","text":"Synthetic text"}),
        );
    });
    runtime().block_on(async {
        let mut stream = client
            .speech_stream(
                &TranscriptionOptions::default(),
                &RequestCancellation::default(),
                Arc::new(|| true),
            )
            .await
            .unwrap();
        stream.feed(&vec![0.125; 48000]).await.unwrap();
        stream.request_finish().await.unwrap();
        let mut updates = vec![];
        loop {
            match stream.next_event().await.unwrap() {
                LiveEvent::Ready => {}
                LiveEvent::Update { revision, text } => updates.push((revision, text)),
                LiveEvent::Finished(text) => {
                    assert_eq!(text, "Synthetic text");
                    break;
                }
            }
        }
        assert_eq!(updates.len(), 3);
        assert!(updates[1].1.is_none());
        assert!(!format!("{:?}", updates[0].1).contains("Synthetic"));
    });
    server.join().unwrap();
}
fn cancelled_opening(handshake: bool) {
    let (entered, ready) = mpsc::channel();
    let (client, server) = fixture(move |listener| {
        let mut pending = if handshake {
            ticket(listener);
            let mut socket = accept(listener);
            let request = header(&mut socket);
            assert!(request.starts_with("GET /ws/voice?wsTicket=private-async-ticket "));
            assert!(!request.to_ascii_lowercase().contains("authorization:"));
            socket
        } else {
            let mut socket = accept(listener);
            assert!(header(&mut socket).starts_with("POST /api/auth/websocket-ticket/media "));
            socket
        };
        entered.send(()).unwrap();
        closed(&mut pending);
    });
    let cancel = RequestCancellation::default();
    let stopper = cancel.clone();
    let interrupt = thread::spawn(move || {
        ready.recv_timeout(Duration::from_secs(5)).unwrap();
        stopper.cancel();
    });
    runtime().block_on(async {
        assert!(matches!(
            client
                .speech_stream(&TranscriptionOptions::default(), &cancel, Arc::new(|| true))
                .await,
            Err(LiveFailure::Cancelled)
        ));
        tokio::task::spawn_blocking(move || server.join().unwrap())
            .await
            .unwrap();
    });
    interrupt.join().unwrap();
}
#[test]
fn cancellation_closes_actual_media_ticket_wait() {
    cancelled_opening(false)
}
#[test]
fn cancellation_closes_actual_pending_websocket_handshake() {
    cancelled_opening(true)
}
#[test]
fn dropping_an_opening_future_closes_its_actual_handshake_socket() {
    let (observed, entered) = mpsc::channel();
    let (client, server) = fixture(move |listener| {
        ticket(listener);
        let mut socket = accept(listener);
        assert!(header(&mut socket).starts_with("GET /ws/voice?wsTicket=private-async-ticket "));
        observed.send(()).unwrap();
        closed(&mut socket);
    });
    let cancel = RequestCancellation::default();
    runtime().block_on(async {
        let mut entered = tokio::task::spawn_blocking(move || {
            entered.recv_timeout(Duration::from_secs(5)).unwrap()
        });
        let options = TranscriptionOptions::default();
        let mut opening = Box::pin(client.speech_stream(&options, &cancel, Arc::new(|| true)));
        poll_fn(|context| {
            assert!(opening.as_mut().poll(context).is_pending());
            Pin::new(&mut entered)
                .poll(context)
                .map(|result| result.unwrap())
        })
        .await;
        drop(opening);
        assert!(!cancel.is_cancelled());
        tokio::task::spawn_blocking(move || server.join().unwrap())
            .await
            .unwrap();
    });
}
#[test]
fn token_closes_an_idle_connected_stream_without_runtime_polling() {
    let (client, server) = fixture(|listener| {
        let mut socket = socket(listener);
        send(&mut socket, json!({"type":"ready"}));
        closed(socket.get_mut());
    });
    let cancel = RequestCancellation::default();
    let rt = runtime();
    let mut stream = rt.block_on(async {
        let mut stream = client
            .speech_stream(&TranscriptionOptions::default(), &cancel, Arc::new(|| true))
            .await
            .unwrap();
        assert!(matches!(
            stream.next_event().await.unwrap(),
            LiveEvent::Ready
        ));
        stream
    });
    cancel.cancel();
    server.join().unwrap();
    assert!(matches!(
        rt.block_on(stream.feed(&[0.125])),
        Err(LiveFailure::Cancelled)
    ));
}
#[test]
fn dropping_an_entered_reader_retires_its_socket_without_cancelling_the_shared_token() {
    let (observed, entered) = mpsc::channel();
    let (client, server) = fixture(move |listener| {
        let mut socket = socket(listener);
        send(&mut socket, json!({"type":"ready"}));
        socket.send(Message::Ping(vec![7].into())).unwrap();
        assert!(matches!(socket.read().unwrap(),Message::Pong(bytes)if bytes.as_ref()==[7]));
        observed.send(()).unwrap();
        closed(socket.get_mut());
    });
    let cancel = RequestCancellation::default();
    runtime().block_on(async {
        let mut stream = client
            .speech_stream(&TranscriptionOptions::default(), &cancel, Arc::new(|| true))
            .await
            .unwrap();
        assert!(matches!(
            stream.next_event().await.unwrap(),
            LiveEvent::Ready
        ));
        let mut entered = tokio::task::spawn_blocking(move || {
            entered.recv_timeout(Duration::from_secs(5)).unwrap()
        });
        let mut read = Box::pin(stream.next_event());
        poll_fn(|context| {
            assert!(read.as_mut().poll(context).is_pending());
            Pin::new(&mut entered)
                .poll(context)
                .map(|result| result.unwrap())
        })
        .await;
        drop(read);
        assert!(!cancel.is_cancelled());
        assert!(matches!(
            stream.feed(&[0.125]).await,
            Err(LiveFailure::Cancelled)
        ));
        tokio::task::spawn_blocking(move || server.join().unwrap())
            .await
            .unwrap();
    });
}
#[test]
fn precancelled_and_unconsented_openings_never_connect() {
    assert!(std::env::var_os("ANDROID_SANDBOX").is_some());
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let client = Connection::new(
        &base,
        None,
        Credential::bearer("private-no-connect").unwrap(),
    )
    .unwrap()
    .cancellable_speech()
    .unwrap();
    let cancel = RequestCancellation::default();
    cancel.cancel();
    let rt = runtime();
    assert!(matches!(
        rt.block_on(client.speech_stream(
            &TranscriptionOptions::default(),
            &cancel,
            Arc::new(|| true)
        )),
        Err(LiveFailure::Cancelled)
    ));
    assert!(matches!(
        rt.block_on(client.speech_stream(
            &TranscriptionOptions::default(),
            &RequestCancellation::default(),
            Arc::new(|| false)
        )),
        Err(LiveFailure::ConsentRevoked)
    ));
    assert!(listener.accept().is_err());
}
#[test]
fn current_consent_revocation_refuses_audio_on_a_real_connected_stream() {
    let (client, server) = fixture(|listener| {
        let mut socket = socket(listener);
        send(&mut socket, json!({"type":"ready"}));
        closed(socket.get_mut());
    });
    let gate = Arc::new(AtomicBool::new(true));
    let check = gate.clone();
    runtime().block_on(async {
        let mut stream = client
            .speech_stream(
                &TranscriptionOptions::default(),
                &RequestCancellation::default(),
                Arc::new(move || check.load(Ordering::Acquire)),
            )
            .await
            .unwrap();
        assert!(matches!(
            stream.next_event().await.unwrap(),
            LiveEvent::Ready
        ));
        gate.store(false, Ordering::Release);
        assert!(matches!(
            stream.feed(&[0.25]).await,
            Err(LiveFailure::ConsentRevoked)
        ));
    });
    server.join().unwrap();
}
#[test]
fn a_websocket_redirect_cannot_reach_another_origin_or_replay_its_ticket() {
    let other = TcpListener::bind("127.0.0.1:0").unwrap();
    other.set_nonblocking(true).unwrap();
    let location = format!("ws://{}/foreign", other.local_addr().unwrap());
    let (client, server) = fixture(move |listener| {
        ticket(listener);
        let mut socket = accept(listener);
        let request = header(&mut socket);
        assert!(request.starts_with("GET /ws/voice?wsTicket=private-async-ticket "));
        write!(socket,"HTTP/1.1 302 Found\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
    });
    assert!(matches!(
        runtime().block_on(client.speech_stream(
            &TranscriptionOptions::default(),
            &RequestCancellation::default(),
            Arc::new(|| true)
        )),
        Err(LiveFailure::Transport)
    ));
    server.join().unwrap();
    assert!(other.accept().is_err());
}
#[test]
fn binary_server_events_fail_typed_and_retire_the_owned_socket() {
    let (client, server) = fixture(|listener| {
        let mut socket = socket(listener);
        send(&mut socket, json!({"type":"ready"}));
        assert!(socket.read().unwrap().is_binary());
        socket.send(Message::Binary(vec![1, 2, 3].into())).unwrap();
        closed(socket.get_mut());
    });
    runtime().block_on(async {
        let mut stream = client
            .speech_stream(
                &TranscriptionOptions::default(),
                &RequestCancellation::default(),
                Arc::new(|| true),
            )
            .await
            .unwrap();
        assert!(matches!(
            stream.next_event().await.unwrap(),
            LiveEvent::Ready
        ));
        stream.feed(&[0.125]).await.unwrap();
        assert!(matches!(
            stream.next_event().await,
            Err(LiveFailure::InvalidResponse)
        ));
    });
    server.join().unwrap();
}
#[test]
fn cancellation_retires_a_maximum_recording_waiting_for_acknowledgements() {
    let (client, server) = fixture(|listener| {
        let mut socket = socket(listener);
        send(&mut socket, json!({"type":"ready"}));
        let mut bytes = 0;
        while let Ok(frame) = socket.read() {
            assert!(frame.is_binary());
            bytes += frame.len();
            assert!(
                bytes <= 128000,
                "Only two native frames may be in flight without acknowledgements"
            );
        }
        assert!(bytes > 0 && bytes < t3_protocol::speech::MAX_PCM_BYTES);
    });
    let cancel = RequestCancellation::default();
    let rt = runtime();
    let mut stream = rt.block_on(async {
        let mut stream = client
            .speech_stream(&TranscriptionOptions::default(), &cancel, Arc::new(|| true))
            .await
            .unwrap();
        assert!(matches!(
            stream.next_event().await.unwrap(),
            LiveEvent::Ready
        ));
        stream.feed(&vec![0.125; 4_800_000]).await.unwrap();
        stream
    });
    cancel.cancel();
    server.join().unwrap();
    assert!(matches!(
        rt.block_on(stream.feed(&[0.125])),
        Err(LiveFailure::Cancelled)
    ));
}
#[test]
fn shared_token_closes_multiple_idle_streams_after_a_registered_slot_is_reused() {
    let server = |listener: &TcpListener| {
        let mut socket = socket(listener);
        send(&mut socket, json!({"type":"ready"}));
        closed(socket.get_mut());
    };
    let (first, first_server) = fixture(server);
    let (second, second_server) = fixture(server);
    let (third, third_server) = fixture(server);
    let cancel = RequestCancellation::default();
    let rt = runtime();
    let open = |client: &CancellableSpeech| {
        rt.block_on(async {
            let mut stream = client
                .speech_stream(&TranscriptionOptions::default(), &cancel, Arc::new(|| true))
                .await
                .unwrap();
            assert!(matches!(
                stream.next_event().await.unwrap(),
                LiveEvent::Ready
            ));
            stream
        })
    };
    let first = open(&first);
    let mut second = open(&second);
    drop(first);
    first_server.join().unwrap();
    assert!(!cancel.is_cancelled());
    let mut third = open(&third);
    cancel.cancel();
    second_server.join().unwrap();
    third_server.join().unwrap();
    assert!(matches!(
        rt.block_on(second.feed(&[0.125])),
        Err(LiveFailure::Cancelled)
    ));
    assert!(matches!(
        rt.block_on(third.feed(&[0.125])),
        Err(LiveFailure::Cancelled)
    ));
}
