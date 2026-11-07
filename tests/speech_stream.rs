use serde_json::{json, Value};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};
use t3_protocol::{
    speech::{LiveEvent, LiveFailure, SpeechStream, TranscriptionOptions},
    Connection, Credential,
};
use tungstenite::Message;

fn isolated() {
    assert!(std::env::var_os("ANDROID_SANDBOX").is_some());
}
fn accept(listener: &TcpListener) -> TcpStream {
    listener.set_nonblocking(true).unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        match listener.accept() {
            Ok((socket, _)) => return socket,
            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock
                    && std::time::Instant::now() < deadline =>
            {
                thread::sleep(Duration::from_millis(2))
            }
            Err(error) => panic!("Private listener did not receive its owned client: {error}"),
        }
    }
}
struct Upgrade;
impl tungstenite::handshake::server::Callback for Upgrade {
    fn on_request(
        self,
        request: &tungstenite::handshake::server::Request,
        response: tungstenite::handshake::server::Response,
    ) -> std::result::Result<
        tungstenite::handshake::server::Response,
        tungstenite::handshake::server::ErrorResponse,
    > {
        assert_eq!(
            request.uri().to_string(),
            "/ws/voice?wsTicket=synthetic-stream-ticket"
        );
        assert!(request.headers().get("authorization").is_none());
        Ok(response)
    }
}
fn http(socket: &mut TcpStream) -> String {
    socket
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let mut header = vec![];
    let mut one = [0];
    while !header.ends_with(b"\r\n\r\n") {
        socket.read_exact(&mut one).unwrap();
        header.push(one[0]);
        assert!(header.len() < 65536);
    }
    String::from_utf8(header).unwrap()
}
fn server<F: FnOnce(tungstenite::WebSocket<TcpStream>) + Send + 'static>(
    run: F,
) -> (Connection, thread::JoinHandle<()>) {
    isolated();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let task = thread::spawn(move || {
        let mut socket = accept(&listener);
        let header = http(&mut socket);
        assert!(header.starts_with("POST /api/auth/websocket-ticket/media "));
        assert!(header
            .to_lowercase()
            .contains("authorization: bearer synthetic-voice"));
        let body = json!({"ticket":"synthetic-stream-ticket"}).to_string();
        write!(socket,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
        drop(socket);
        let socket = accept(&listener);
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        socket
            .set_write_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut ws = tungstenite::accept_hdr(socket, Upgrade).unwrap();
        let start = ws.read().unwrap();
        let value: Value = serde_json::from_str(start.to_text().unwrap()).unwrap();
        assert_eq!(value["type"], "start");
        assert_eq!(value["options"]["speechLanguage"], "auto");
        run(ws);
        listener.set_nonblocking(true).unwrap();
        assert!(
            listener.accept().is_err(),
            "Stream must not reconnect or remint a ticket"
        );
    });
    (
        Connection::new(&url, None, Credential::bearer("synthetic-voice").unwrap()).unwrap(),
        task,
    )
}
fn send(ws: &mut tungstenite::WebSocket<TcpStream>, value: Value) {
    ws.send(Message::Text(value.to_string().into())).unwrap();
}
fn stream(c: &Connection) -> SpeechStream {
    c.speech_stream(&TranscriptionOptions::default(), Arc::new(|| true))
        .unwrap()
}

#[test]
fn physical_ticket_socket_ack_window_and_final_text_match_native() {
    let (c, task) = server(|mut ws| {
        send(&mut ws, json!({"type":"ready"}));
        for _ in 0..2 {
            let frame = ws.read().unwrap();
            let bytes = frame.into_data();
            assert_eq!(bytes.len(), 64000);
            assert!(bytes
                .as_chunks::<4>()
                .0
                .iter()
                .all(|v| v == &0.125f32.to_le_bytes()));
        }
        ws.get_mut()
            .set_read_timeout(Some(Duration::from_millis(80)))
            .unwrap();
        assert!(
            matches!(ws.read(),Err(tungstenite::Error::Io(error))if matches!(error.kind(),std::io::ErrorKind::WouldBlock|std::io::ErrorKind::TimedOut))
        );
        ws.get_mut()
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        send(
            &mut ws,
            json!({"type":"update","revision":1,"text":{"committed":"Synthetic ","tentative":"text"}}),
        );
        let frame = ws.read().unwrap();
        assert!(frame.is_binary());
        assert_eq!(frame.len(), 64000);
        send(&mut ws, json!({"type":"update","revision":1,"text":null}));
        send(
            &mut ws,
            json!({"type":"update","revision":2,"text":{"committed":"Synthetic text","tentative":""}}),
        );
        let finish = ws.read().unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(finish.to_text().unwrap()).unwrap(),
            json!({"type":"finish"})
        );
        send(&mut ws, json!({"type":"finished","text":"Synthetic text"}));
    });
    let mut s = stream(&c);
    s.feed(&vec![0.125; 48000]).unwrap();
    s.request_finish().unwrap();
    let mut updates = vec![];
    loop {
        match s.next_event().unwrap() {
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
    task.join().unwrap();
}
#[test]
fn independent_cancel_interrupts_a_blocked_read_without_reconnection() {
    let (observed, ready) = std::sync::mpsc::sync_channel(1);
    let (c, task) = server(move |mut ws| {
        send(&mut ws, json!({"type":"ready"}));
        ws.send(Message::Ping(vec![9].into())).unwrap();
        assert!(matches!(ws.read().unwrap(),Message::Pong(data)if data.as_ref()==[9]));
        observed.send(()).unwrap();
        let _ = ws.read();
    });
    let mut s = stream(&c);
    assert!(matches!(s.next_event().unwrap(), LiveEvent::Ready));
    let cancel = s.cancellation();
    let read = thread::spawn(move || s.next_event());
    // An observed Pong proves the reader is active; cancellation cannot win
    // merely because its flag was set before the read owner started.
    ready.recv_timeout(Duration::from_secs(5)).unwrap();
    cancel.cancel();
    assert!(matches!(read.join().unwrap(), Err(LiveFailure::Cancelled)));
    task.join().unwrap();
}
#[test]
fn bounded_poll_accepts_capture_between_reads_and_rejects_binary_events() {
    let (c, task) = server(|mut ws| {
        send(&mut ws, json!({"type":"ready"}));
        let audio = ws.read().unwrap();
        assert!(audio.is_binary());
        ws.send(Message::Binary(vec![1, 2, 3].into())).unwrap();
        assert!(ws.read().unwrap().is_close());
    });
    let mut s = stream(&c);
    assert!(matches!(s.next_event().unwrap(), LiveEvent::Ready));
    let started = std::time::Instant::now();
    assert!(s.poll_event().unwrap().is_none());
    assert!(started.elapsed() < Duration::from_secs(2));
    s.feed(&[0.25]).unwrap();
    assert!(matches!(s.next_event(), Err(LiveFailure::InvalidResponse)));
    task.join().unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let c = Connection::new(
        &format!("http://{}", listener.local_addr().unwrap()),
        None,
        Credential::bearer("synthetic").unwrap(),
    )
    .unwrap();
    assert!(matches!(
        c.speech_stream(&TranscriptionOptions::default(), Arc::new(|| false)),
        Err(LiveFailure::ConsentRevoked)
    ));
    assert!(listener.accept().is_err());
}
#[test]
fn current_consent_fences_audio_and_ticket_redirects_never_reach_another_origin() {
    let gate = Arc::new(AtomicBool::new(true));
    let g = gate.clone();
    let (c, task) = server(|mut ws| {
        send(&mut ws, json!({"type":"ready"}));
        let frame = ws.read().unwrap();
        assert!(frame.is_close());
    });
    let mut s = c
        .speech_stream(
            &TranscriptionOptions::default(),
            Arc::new(move || g.load(Ordering::Acquire)),
        )
        .unwrap();
    assert!(matches!(s.next_event().unwrap(), LiveEvent::Ready));
    gate.store(false, Ordering::Release);
    assert!(matches!(s.feed(&[0.1]), Err(LiveFailure::ConsentRevoked)));
    task.join().unwrap();
    let target = TcpListener::bind("127.0.0.1:0").unwrap();
    target.set_nonblocking(true).unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let location = format!(
        "http://{}/must-not-receive-credential",
        target.local_addr().unwrap()
    );
    let task = thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        http(&mut socket);
        write!(socket,"HTTP/1.1 307 Temporary Redirect\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
    });
    let c = Connection::new(&base, None, Credential::bearer("synthetic").unwrap()).unwrap();
    assert!(matches!(
        c.speech_stream(&TranscriptionOptions::default(), Arc::new(|| true)),
        Err(LiveFailure::Transport)
    ));
    task.join().unwrap();
    assert!(target.accept().is_err());
}
