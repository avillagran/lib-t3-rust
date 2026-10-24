use serde_json::{json, Value};
use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    thread,
};
use t3_protocol::{
    device_hub::{input_message, parse_semu, Platform},
    Connection, Credential,
};
/// Answers the ticket request, then one hub request with `status`; returns the paths seen.
fn hub(status: &'static str) -> (Connection, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        let mut seen = vec![];
        for _ in 0..2 {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let mut headers = String::new();
            loop {
                let mut h = String::new();
                reader.read_line(&mut h).unwrap();
                if h == "\r\n" {
                    break;
                }
                headers.push_str(&h.to_lowercase());
            }
            let mut out = stream;
            seen.push(format!("{} {}", line.split_whitespace().next().unwrap(), line.split_whitespace().nth(1).unwrap()));
            let (status, kind, body): (&str, &str, &[u8]) = if line.contains("/api/auth/websocket-ticket") {
                assert!(headers.contains("authorization: bearer synthetic"), "{headers}");
                ("200 OK", "application/json", br#"{"ticket":"one-ticket"}"#)
            } else {
                // Media requests carry the ticket, never the credential.
                assert!(!headers.contains("authorization:"), "{headers}");
                (status, "image/png", b"\x89PNG\r\n\x1a\nandroid")
            };
            write!(out, "HTTP/1.1 {status}\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).unwrap();
            out.write_all(body).unwrap();
        }
        seen
    });
    (Connection::new(&base, None, Credential::bearer("synthetic").unwrap()).unwrap(), server)
}
#[test]
fn android_screenshot_goes_through_the_serve_emu_proxy_with_ticket_and_host() {
    let (c, server) = hub("200 OK");
    let access = c.device_hub("/api/device-hub", "host-1").unwrap();
    assert_eq!(c.device_screenshot(&access, Platform::Android, "emulator-5554").unwrap(), b"\x89PNG\r\n\x1a\nandroid");
    assert_eq!(server.join().unwrap(), vec![
        "POST /api/auth/websocket-ticket".to_string(),
        "POST /api/device-hub/vendor/serve-emu/api/screenshot?device=emulator-5554&hostId=host-1&wsTicket=one-ticket".to_string(),
    ]);
}
#[test]
fn hub_refusals_map_to_stable_codes() {
    let (c, server) = hub("403 Forbidden");
    let access = c.device_hub("/api/device-hub", "host-1").unwrap();
    assert_eq!(c.device_screenshot(&access, Platform::Ios, "UDID").unwrap_err(), "unauthorized");
    server.join().unwrap();
    let (c, server) = hub("404 Not Found");
    let access = c.device_hub("/api/device-hub", "host-1").unwrap();
    assert_eq!(c.device_screenshot(&access, Platform::Android, "gone").unwrap_err(), "device_unavailable");
    server.join().unwrap();
}
#[test]
fn hub_paths_and_ids_are_validated_before_any_request() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let c = Connection::new(&format!("http://{}", listener.local_addr().unwrap()), None, Credential::bearer("synthetic").unwrap()).unwrap();
    for (base, host) in [("/api/device-hub/", "h"), ("/api/../admin", "h"), ("/device-hub", "h"), ("/api/device-hub", "a/b"), ("/api/device-hub", "")] {
        assert!(c.device_hub(base, host).is_err(), "{base} {host}");
    }
    assert_eq!(listener.accept().unwrap_err().kind(), std::io::ErrorKind::WouldBlock);
    assert_eq!(Platform::parse("android").unwrap(), Platform::Android);
    assert_eq!(Platform::parse("ios").unwrap(), Platform::Ios);
    assert_eq!(Platform::parse("windows").unwrap_err(), "invalid_device");
}
#[test]
fn semu_header_is_sixteen_bytes_with_a_keyframe_flag() {
    let mut packet = b"SEMU".to_vec();
    packet.extend_from_slice(&[1, 0, 0, 0]);
    packet.extend_from_slice(&123_456u64.to_be_bytes());
    assert_eq!(parse_semu(&packet), Some((false, &[][..])), "header only");
    packet[5] = 0b11;
    packet.extend_from_slice(&[0, 0, 0, 1, 0x67]);
    assert_eq!(parse_semu(&packet), Some((true, &[0, 0, 0, 1, 0x67][..])));
    assert!(parse_semu(&packet[..15]).is_none());
    assert!(parse_semu(b"SEMX\x01\x01\0\0\0\0\0\0\0\0\0\0").is_none());
}
#[test]
fn android_input_is_serve_emu_json_and_ios_keys_use_hid_usages() {
    let text = |platform, event: Value| match input_message(platform, &event).unwrap() {
        tungstenite::Message::Text(t) => serde_json::from_str::<Value>(&t).unwrap(),
        tungstenite::Message::Binary(b) => json!({"tag":b[0],"body":serde_json::from_slice::<Value>(&b[1..]).unwrap()}),
        other => panic!("{other:?}"),
    };
    assert_eq!(text(Platform::Android, json!({"type":"button","button":"back"})), json!({"type":"back"}));
    assert_eq!(text(Platform::Android, json!({"type":"key","keycode":66})), json!({"type":"key","keycode":66}));
    assert_eq!(text(Platform::Android, json!({"type":"text","text":"hola"})), json!({"type":"text","text":"hola"}));
    assert_eq!(text(Platform::Android, json!({"type":"keyframe"})), json!({"type":"reset-video","ack":false}));
    assert_eq!(text(Platform::Android, json!({"type":"touch","phase":"up","x":1.0,"y":0.0})), json!({"type":"touch","action":"up","x":1.0,"y":0.0}));
    assert_eq!(text(Platform::Ios, json!({"type":"key","usage":4,"down":true})), json!({"tag":6,"body":{"type":"down","usage":4}}));
    assert_eq!(text(Platform::Ios, json!({"type":"button","button":"power"})), json!({"tag":4,"body":{"button":"lock"}}));
    for (platform, bad) in [
        (Platform::Android, json!({"type":"key","keycode":0})),
        (Platform::Android, json!({"type":"text","text":""})),
        (Platform::Android, json!({"type":"text","text":"x".repeat(1025)})),
        (Platform::Android, json!({"type":"touch","phase":"down","x":-0.1,"y":0.5})),
        (Platform::Android, json!({"type":"touch","phase":"tap","x":0.1,"y":0.5})),
        (Platform::Ios, json!({"type":"key","usage":0x10000})),
        (Platform::Ios, json!({"type":"text","text":"x"})),
    ] {
        assert_eq!(input_message(platform, &bad).unwrap_err(), "invalid_device_input", "{bad}");
    }
}
