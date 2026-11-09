//! Lossless HTTP through the original same-origin/authenticated bounded factory.
use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
    time::Duration,
};
use t3_protocol::{wtf16::Wtf16String, Connection, Credential, RawJson};
fn connection(endpoint: &str) -> Connection {
    Connection::new(
        endpoint,
        None,
        Credential::bearer("private-raw-http").unwrap(),
    )
    .unwrap()
}
fn peer(
    body: Vec<u8>,
    status: &'static str,
    media: &'static str,
    encoding: &'static str,
) -> (String, thread::JoinHandle<(String, Vec<u8>)>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    let task = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut header = Vec::new();
        while !header.ends_with(b"\r\n\r\n") {
            let mut byte = [0];
            stream.read_exact(&mut byte).unwrap();
            header.push(byte[0]);
            assert!(header.len() < 16384)
        }
        let header = String::from_utf8(header).unwrap();
        let lower = header.to_ascii_lowercase();
        assert!(lower.contains("authorization: bearer private-raw-http\r\n"));
        assert!(lower.contains("accept-encoding: identity\r\n"));
        let length = lower
            .lines()
            .find_map(|line| {
                line.strip_prefix("content-length:")
                    .map(|value| value.trim().parse::<usize>().unwrap())
            })
            .unwrap_or(0);
        let mut input = vec![0; length];
        stream.read_exact(&mut input).unwrap();
        let response=format!("HTTP/1.1 {status}\r\nContent-Type: {media}\r\nContent-Encoding: {encoding}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",body.len());
        stream.write_all(response.as_bytes()).unwrap();
        let _ = stream.write_all(&body);
        (header, input)
    });
    (endpoint, task)
}
#[test]
fn original_http_factory_preserves_shell_titles_and_raw_post_body() {
    let text = r#"{"threads":[{"id":"t","title":"ab\ud800"}],"projectTitle":"p\udc00"}"#;
    for method in ["GET", "POST"] {
        let (endpoint, server) = peer(
            text.as_bytes().to_vec(),
            "200 OK",
            "Application/JSON; charset=utf-8",
            "identity",
        );
        let payload = RawJson::from_string(text.into()).unwrap();
        let result = connection(&endpoint)
            .request_json_raw(
                method,
                "/api/orchestration/shell",
                (method == "POST").then_some(payload.as_ref()),
                true,
            )
            .unwrap();
        let (header, input) = server.join().unwrap();
        assert!(header
            .to_ascii_lowercase()
            .contains("x-t3-orchestration-protocol: 2\r\n"));
        assert_eq!(result.get(), text);
        if method == "POST" {
            assert_eq!(input, text.as_bytes())
        } else {
            assert!(input.is_empty())
        }
        let fields: std::collections::BTreeMap<String, Box<RawJson>> =
            serde_json::from_str(result.get()).unwrap();
        assert_eq!(
            Wtf16String::from_json(&fields["projectTitle"])
                .unwrap()
                .units(),
            [112, 0xdc00]
        );
    }
}
#[test]
fn content_status_encoding_json_and_limit_failures_keep_existing_rules() {
    let cases = vec![
        (
            b"{}".to_vec(),
            "401 Unauthorized",
            "application/json",
            "identity",
            "unauthorized",
            1024,
        ),
        (
            b"{}".to_vec(),
            "200 OK",
            "text/plain",
            "identity",
            "not_json",
            1024,
        ),
        (
            b"{}".to_vec(),
            "200 OK",
            "application/json",
            "gzip",
            "unsupported_encoding",
            1024,
        ),
        (
            b"{\"title\":\"\\uxxxx\"}".to_vec(),
            "200 OK",
            "application/json",
            "identity",
            "malformed",
            1024,
        ),
        (
            vec![0xff],
            "200 OK",
            "application/json",
            "identity",
            "malformed",
            1024,
        ),
        (
            vec![32; 256],
            "200 OK",
            "application/json",
            "identity",
            "too_large",
            128,
        ),
    ];
    for (body, status, media, encoding, expected, limit) in cases {
        let (endpoint, server) = peer(body, status, media, encoding);
        let result = connection(&endpoint).request_json_raw_with_limit(
            "GET",
            "/api/orchestration/shell",
            None,
            true,
            limit,
        );
        server.join().unwrap();
        assert_eq!(result.unwrap_err(), expected)
    }
}
#[test]
fn invalid_endpoint_and_invalid_explicit_limit_do_not_open_a_socket() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let connection = connection(&format!("http://{}", listener.local_addr().unwrap()));
    for path in [
        "//foreign.invalid/private",
        "https://foreign.invalid/",
        "/api\\private",
        "/api#private",
    ] {
        assert_eq!(
            connection
                .request_json_raw("GET", path, None, false)
                .unwrap_err(),
            "invalid_endpoint"
        )
    }
    for limit in [0, 64 * 1024 * 1024 + 1] {
        assert_eq!(
            connection
                .request_json_raw_with_limit("GET", "/api/orchestration/shell", None, true, limit)
                .unwrap_err(),
            "invalid_max_bytes"
        )
    }
    assert!(matches!(listener.accept(),Err(error)if error.kind()==std::io::ErrorKind::WouldBlock));
}
#[test]
fn bounded_snapshot_has_its_original_sixty_four_mib_budget_only_when_admitted() {
    let body = serde_json::to_vec(&"s".repeat(8 * 1024 * 1024 + 1)).unwrap();
    for protocol in [false, true] {
        let (endpoint, server) = peer(body.clone(), "200 OK", "application/json", "identity");
        let result = connection(&endpoint).request_json_raw(
            "GET",
            "/api/orchestration/threads/t/bounded",
            None,
            protocol,
        );
        server.join().unwrap();
        if protocol {
            assert_eq!(result.unwrap().get().len(), body.len())
        } else {
            assert_eq!(result.unwrap_err(), "too_large")
        }
    }
}
