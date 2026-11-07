//! Physical cancellation of actual SDK requests on private loopback sockets.
use serde_json::{json, Value};
use std::{
    future::{poll_fn, Future},
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    task::Poll,
    thread,
    time::{Duration, Instant},
};
use t3_protocol::{
    speech::{
        decode_pcm_request, CloudProvider, ControlFailure, RequestCancellation, RequestFailure,
        SpeechFailure, TranscriptionOptions,
    },
    Connection, Credential,
};
fn isolated() {
    assert!(std::path::Path::new("/run/.containerenv").exists());
}
fn accept(listener: &TcpListener) -> (TcpStream, std::net::SocketAddr) {
    listener.set_nonblocking(true).unwrap();
    let end = Instant::now() + Duration::from_secs(3);
    loop {
        match listener.accept() {
            Ok(pair) => return pair,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(Instant::now() < end, "Private request admission deadline");
                thread::sleep(Duration::from_millis(2));
            }
            Err(error) => panic!("{error}"),
        }
    }
}
fn listener() -> (TcpListener, String) {
    isolated();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    (listener, base)
}
fn client(base: &str) -> t3_protocol::speech::CancellableSpeech {
    Connection::new(
        base,
        None,
        Credential::bearer("private-request-fixture").unwrap(),
    )
    .unwrap()
    .cancellable_speech()
    .unwrap()
}
fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}
fn always() -> bool {
    true
}
fn assert_closed(socket: &mut TcpStream) {
    match socket.read(&mut [0]) {
        Ok(0) => {}
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::ConnectionAborted
            ) => {}
        value => panic!("Actual pending HTTP connection must close: {value:?}"),
    }
}
fn receive_headers(socket: &mut TcpStream) -> (String, usize) {
    socket
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    socket
        .set_write_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    let mut header = vec![];
    let mut byte = [0];
    while !header.ends_with(b"\r\n\r\n") {
        socket.read_exact(&mut byte).unwrap();
        header.push(byte[0]);
        assert!(header.len() < 65536);
    }
    let header = String::from_utf8(header).unwrap();
    assert!(header
        .to_ascii_lowercase()
        .contains("authorization: bearer private-request-fixture"));
    let size = header
        .lines()
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
        .map(|(_, size)| size.trim().parse::<usize>().unwrap())
        .unwrap_or(0);
    assert!(size <= 20 * 1024 * 1024);
    (header, size)
}
fn receive(socket: &mut TcpStream) -> (String, Vec<u8>) {
    let (header, size) = receive_headers(socket);
    let mut body = vec![0; size];
    socket.read_exact(&mut body).unwrap();
    (header, body)
}
#[test]
fn cancellation_stops_a_backpressured_maximum_pcm_upload() {
    maximum_upload_is_retired(false);
}
#[test]
fn dropping_the_future_stops_its_upload_without_cancelling_sibling_operations() {
    maximum_upload_is_retired(true);
}
fn maximum_upload_is_retired(drop_future: bool) {
    isolated();
    // Set receive capacity before accepting so an entire recording cannot be
    // mistaken for a still-running upload already buffered by the receiver.
    let socket = socket2::Socket::new(
        socket2::Domain::IPV4,
        socket2::Type::STREAM,
        Some(socket2::Protocol::TCP),
    )
    .unwrap();
    socket.set_recv_buffer_size(8192).unwrap();
    socket
        .bind(
            &"127.0.0.1:0"
                .parse::<std::net::SocketAddr>()
                .unwrap()
                .into(),
        )
        .unwrap();
    socket.listen(4).unwrap();
    let listener: TcpListener = socket.into();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let (entered, ready) = mpsc::channel();
    let (completed, result_ready) = mpsc::channel();
    let server = thread::spawn(move || {
        let (mut socket, _) = accept(&listener);
        assert!(socket2::SockRef::from(&socket).recv_buffer_size().unwrap() <= 16 * 1024);
        let (_, declared) = receive_headers(&mut socket);
        assert!(declared > t3_protocol::speech::MAX_PCM_BYTES);
        let mut prefix = [0; 4096];
        socket.read_exact(&mut prefix).unwrap();
        entered.send(()).unwrap();
        // Deliberate TCP backpressure: no remaining audio is read until the
        // actual request reports cancellation. Then count all buffered bytes.
        result_ready.recv_timeout(Duration::from_secs(2)).unwrap();
        let mut observed = prefix.len();
        let mut bytes = [0; 65536];
        loop {
            match socket.read(&mut bytes) {
                Ok(0) => break,
                Ok(count) => observed += count,
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::ConnectionAborted
                    ) =>
                {
                    break
                }
                Err(error) => panic!("Cancelled upload did not close: {error}"),
            }
        }
        assert!(
            observed < declared,
            "A cancelled upload must not finish its recording"
        );
    });
    let client = client(&base);
    let rt = runtime();
    let cancel = RequestCancellation::default();
    let pcm = vec![0; t3_protocol::speech::MAX_PCM_BYTES];
    if drop_future {
        rt.block_on(async {
            let mut entered = tokio::task::spawn_blocking(move || {
                ready.recv_timeout(Duration::from_secs(3)).unwrap();
            });
            let options = TranscriptionOptions::default();
            let mut operation = Box::pin(client.transcribe(&pcm, &options, None, &cancel, always));
            poll_fn(|context| {
                assert!(operation.as_mut().poll(context).is_pending());
                std::pin::Pin::new(&mut entered)
                    .poll(context)
                    .map(|result| result.unwrap())
            })
            .await;
            drop(operation);
            assert!(!cancel.is_cancelled());
            completed.send(()).unwrap();
            tokio::task::spawn_blocking(move || server.join().unwrap())
                .await
                .unwrap();
        });
        return;
    }
    let stopper = cancel.clone();
    let interrupt = thread::spawn(move || {
        ready.recv_timeout(Duration::from_secs(3)).unwrap();
        stopper.cancel();
    });
    let result = rt.block_on(async {
        let result = client
            .transcribe(
                &pcm,
                &TranscriptionOptions::default(),
                None,
                &cancel,
                || true,
            )
            .await;
        completed.send(()).unwrap();
        tokio::task::spawn_blocking(move || server.join().unwrap())
            .await
            .unwrap();
        result
    });
    assert_eq!(result, Err(RequestFailure::Cancelled));
    interrupt.join().unwrap();
}
fn respond(socket: &mut TcpStream, status: &str, value: Value) {
    let body = value.to_string();
    write!(socket,"HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
}
#[test]
fn precancelled_or_unconsented_requests_never_connect() {
    let (listener, base) = listener();
    listener.set_nonblocking(true).unwrap();
    let client = client(&base);
    let rt = runtime();
    let cancel = RequestCancellation::default();
    cancel.cancel();
    assert_eq!(
        rt.block_on(client.status(&cancel, || true)),
        Err(RequestFailure::Cancelled)
    );
    assert_eq!(
        rt.block_on(client.transcribe(
            &[0; 4],
            &TranscriptionOptions::default(),
            None,
            &RequestCancellation::default(),
            || false
        )),
        Err(RequestFailure::ConsentRevoked)
    );
    assert!(matches!(listener.accept(),Err(error)if error.kind()==std::io::ErrorKind::WouldBlock));
}
#[test]
fn actual_binary_envelope_cloud_errors_and_control_errors_match_existing_codecs() {
    let (listener, base) = listener();
    let server = thread::spawn(move || {
        let (mut socket, _) = accept(&listener);
        let (header, body) = receive(&mut socket);
        assert!(header.starts_with("GET /api/auth/session ") && body.is_empty());
        respond(
            &mut socket,
            "200 OK",
            json!({"authenticated":true,"permissions":["orchestration:operate"]}),
        );
        let (mut socket, _) = accept(&listener);
        let (header, body) = receive(&mut socket);
        assert!(header.starts_with("POST /api/voice/transcribe "));
        assert!(header
            .to_ascii_lowercase()
            .contains("content-type: application/octet-stream"));
        let (options, pcm) = decode_pcm_request(&body).unwrap();
        assert_eq!(options, TranscriptionOptions::default());
        assert_eq!(pcm, [0; 8]);
        respond(&mut socket, "200 OK", json!({"text":"native transcript"}));
        let (mut socket, _) = accept(&listener);
        let (header, _) = receive(&mut socket);
        assert!(header.contains("/api/voice/cloud/openrouter/transcribe?model=author%2Fmodel-v1"));
        respond(
            &mut socket,
            "422 Unprocessable Entity",
            json!({"_tag":"CloudSpeechError","provider":"openrouter","reason":"key-changed"}),
        );
        let (mut socket, _) = accept(&listener);
        let (header, _) = receive(&mut socket);
        assert!(header.starts_with("GET /api/voice/status "));
        respond(
            &mut socket,
            "400 Bad Request",
            json!({"_tag":"EnvironmentRequestInvalidError","code":"invalid_request","reason":"speech_busy","traceId":" fixed-trace "}),
        );
        let (mut socket, _) = accept(&listener);
        let (header, _) = receive(&mut socket);
        assert!(header.starts_with("POST /api/voice/models/prepare "));
        write!(
            socket,
            "HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n"
        )
        .unwrap();
        let (mut socket, _) = accept(&listener);
        let (header, _) = receive(&mut socket);
        assert!(header.starts_with("GET /api/voice/cloud/keys "));
        respond(
            &mut socket,
            "200 OK",
            json!({"openai":true,"xai":false,"openrouter":false}),
        );
    });
    let client = client(&base);
    let rt = runtime();
    let cancel = RequestCancellation::default();
    let session = rt.block_on(client.session(&cancel, always)).unwrap();
    assert_eq!(session["authenticated"], true);
    assert_eq!(session["permissions"], json!(["orchestration:operate"]));
    assert_eq!(
        rt.block_on(client.transcribe(
            &[0; 8],
            &TranscriptionOptions::default(),
            None,
            &cancel,
            || true
        ))
        .unwrap(),
        "native transcript"
    );
    assert_eq!(
        rt.block_on(client.transcribe(
            &[0; 4],
            &TranscriptionOptions::default(),
            Some((CloudProvider::OpenRouter, Some("author/model-v1"))),
            &cancel,
            || true
        )),
        Err(RequestFailure::Speech(SpeechFailure::Cloud {
            provider: CloudProvider::OpenRouter,
            reason: "key-changed".into()
        }))
    );
    assert_eq!(
        rt.block_on(client.status(&cancel, || true)),
        Err(RequestFailure::Control(ControlFailure::InvalidRequest {
            reason: "speech_busy".into(),
            trace_id: "fixed-trace".into()
        }))
    );
    assert_eq!(rt.block_on(client.prepare(&cancel, || true)), Ok(()));
    let keys = rt.block_on(client.cloud_keys(&cancel, || true)).unwrap();
    assert!(keys.openai);
    assert!(!keys.xai);
    assert_eq!(keys.openrouter, Some(false));
    server.join().unwrap();
}
fn cancellation_case(partial_body: bool) {
    let (listener, base) = listener();
    let (started, ready) = mpsc::channel();
    let (cancelled, signal) = mpsc::channel();
    let server = thread::spawn(move || {
        let (mut socket, _) = accept(&listener);
        receive(&mut socket);
        if partial_body {
            socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 1000\r\n\r\n{\"text\":\"").unwrap();
        }
        started.send(()).unwrap();
        assert_closed(&mut socket);
        cancelled.send(()).unwrap();
    });
    let client = client(&base);
    let rt = runtime();
    let cancel = RequestCancellation::default();
    let clone = cancel.clone();
    let interrupter = thread::spawn(move || {
        ready.recv_timeout(Duration::from_secs(3)).unwrap();
        let at = Instant::now();
        clone.cancel();
        at
    });
    let result = rt.block_on(async {
        let result = client
            .transcribe(
                &[0; 4],
                &TranscriptionOptions::default(),
                None,
                &cancel,
                || true,
            )
            .await;
        // Keep the caller's runtime driving its actual HTTP connection task
        // while the independent server verifies EOF/reset. No timing sleep.
        tokio::task::spawn_blocking(move || signal.recv_timeout(Duration::from_secs(2)).unwrap())
            .await
            .unwrap();
        result
    });
    let at = interrupter.join().unwrap();
    assert_eq!(result, Err(RequestFailure::Cancelled));
    assert!(at.elapsed() < Duration::from_millis(750));
    server.join().unwrap();
}
#[test]
fn cancellation_interrupts_a_real_inference_wait_and_closes_its_socket() {
    cancellation_case(false);
}
#[test]
fn cancellation_interrupts_a_real_partial_response_and_discards_text() {
    cancellation_case(true);
}
#[test]
fn shared_cancellation_wakes_all_entered_requests() {
    let (listener, base) = listener();
    let cancel = RequestCancellation::default();
    let stop = cancel.clone();
    let server = thread::spawn(move || {
        let mut sockets = vec![];
        for _ in 0..2 {
            let (mut socket, _) = accept(&listener);
            receive(&mut socket);
            sockets.push(socket);
        }
        stop.cancel();
        for mut socket in sockets {
            assert_closed(&mut socket);
        }
    });
    let client = client(&base);
    let rt = runtime();
    let mut requests = [
        Some(Box::pin(client.status(&cancel, always as fn() -> bool))),
        Some(Box::pin(client.status(&cancel, always as fn() -> bool))),
    ];
    let mut results = [None, None];
    rt.block_on(async {
        poll_fn(|context| {
            for (index, request) in requests.iter_mut().enumerate() {
                if let Some(work) = request {
                    if let Poll::Ready(result) = work.as_mut().poll(context) {
                        results[index] = Some(result);
                        *request = None;
                    }
                }
            }
            if results.iter().all(Option::is_some) {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
        tokio::task::spawn_blocking(move || server.join().unwrap())
            .await
            .unwrap();
    });
    for result in results {
        assert_eq!(result.unwrap(), Err(RequestFailure::Cancelled));
    }
}
#[test]
fn a_revoked_guard_discards_a_completed_response_without_replay() {
    let (listener, base) = listener();
    let allowed = Arc::new(AtomicBool::new(true));
    let change = allowed.clone();
    let server = thread::spawn(move || {
        let (mut socket, _) = accept(&listener);
        receive(&mut socket);
        change.store(false, Ordering::Release);
        respond(
            &mut socket,
            "200 OK",
            json!({"text":"must not reach the draft"}),
        );
    });
    let client = client(&base);
    let rt = runtime();
    assert_eq!(
        rt.block_on(client.transcribe(
            &[0; 4],
            &TranscriptionOptions::default(),
            None,
            &RequestCancellation::default(),
            || allowed.load(Ordering::Acquire)
        )),
        Err(RequestFailure::ConsentRevoked)
    );
    server.join().unwrap();
}
#[test]
fn redirects_do_not_replay_audio() {
    let (listener, base) = listener();
    let server = thread::spawn(move || {
        let (mut socket, _) = accept(&listener);
        receive(&mut socket);
        write!(socket,"HTTP/1.1 307 Temporary Redirect\r\nLocation: /second-upload\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
        drop(socket);
        listener.set_nonblocking(true).unwrap();
        listener
    });
    let client = client(&base);
    let rt = runtime();
    assert_eq!(
        rt.block_on(client.transcribe(
            &[0; 4],
            &TranscriptionOptions::default(),
            None,
            &RequestCancellation::default(),
            || true
        )),
        Err(RequestFailure::Transport("http_307".into()))
    );
    let listener = server.join().unwrap();
    assert!(matches!(listener.accept(),Err(error)if error.kind()==std::io::ErrorKind::WouldBlock));
}
