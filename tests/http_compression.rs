//! Compressed responses against a local server that negotiates like T3's: what each
//! HTTP path accepts, what it decodes, and that limits apply after decoding.
mod common;
use common::{read_request, response, serve, Seen};
use serde_json::{json, Value};
use std::{
    io::Write,
    net::TcpListener,
    sync::Arc,
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use t3_protocol::{assets::attachment_resource, Connection, Credential, DpopKey};

const MIB: usize = 1024 * 1024;

fn gzip(body: &[u8]) -> Vec<u8> {
    let mut out = flate2::write::GzEncoder::new(vec![], flate2::Compression::default());
    out.write_all(body).unwrap();
    out.finish().unwrap()
}
fn brotli(body: &[u8], quality: u32) -> Vec<u8> {
    let mut out = vec![];
    let mut writer = brotli::CompressorWriter::new(&mut out, 4096, quality, 22);
    writer.write_all(body).unwrap();
    drop(writer);
    out
}
type Encoder = (&'static str, fn(&[u8]) -> Vec<u8>);
const ENCODERS: [Encoder; 2] = [("gzip", gzip), ("br", |b| brotli(b, 4))];
fn encoded(content_type: &str, encoding: &str, body: &[u8]) -> Vec<u8> {
    response(
        "200 OK",
        &[
            ("Content-Type", content_type),
            ("Content-Encoding", encoding),
        ],
        body,
    )
}
fn json_answer((encoding, encode): Encoder, body: &[u8]) -> Vec<u8> {
    encoded("application/json", encoding, &encode(body))
}
fn bearer() -> Credential {
    Credential::bearer("synthetic-token").unwrap()
}
fn accepts_compression(seen: &Seen) -> bool {
    let accepted = seen.header("accept-encoding").unwrap_or_default();
    accepted.contains("gzip") && accepted.contains("br")
}
/// A JSON document of exactly `len` bytes made of `filler`, valid after the limit check.
fn json_of_len(len: usize, filler: char) -> Vec<u8> {
    format!("\"{}\"", filler.to_string().repeat(len - 2)).into_bytes()
}

#[test]
fn shell_snapshot_and_http_rpc_paths_decode_both_encodings() {
    for encoder in ENCODERS {
        let body = br#"{"threads":[{"id":"t1","title":"compressed"}]}"#;
        let (base, server) = serve(vec![
            json_answer(encoder, body),
            json_answer(encoder, body),
            json_answer(encoder, body),
        ]);
        let c = Connection::new(&base, None, bearer()).unwrap();
        let expected: Value = serde_json::from_slice(body).unwrap();
        let shell = c
            .request_json("GET", "/api/orchestration/shell", None, true)
            .unwrap();
        assert_eq!(shell, expected);
        let snapshot = c
            .request_json("GET", "/api/orchestration/threads/t1/bounded", None, true)
            .unwrap();
        assert_eq!(snapshot, expected);
        let posted = c
            .request_json(
                "POST",
                "/api/pull-requests/diff",
                Some(&json!({"a":1})),
                false,
            )
            .unwrap();
        assert_eq!(posted, expected);
        let seen = server.join().unwrap();
        assert!(seen.iter().all(accepts_compression), "{}", seen[0].head);
        assert_eq!(seen[2].body, r#"{"a":1}"#);
    }
}

/// One `assets.createUrl` grant, then the signed GETs. The grant is the only
/// WebSocket exchange; each answer here is plain HTTP.
fn asset_server(answers: Vec<Vec<u8>>) -> (Connection, JoinHandle<Vec<Seen>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let grants = answers.len();
    let server = thread::spawn(move || {
        let mut seen = vec![];
        for answer in answers {
            let (s, _) = listener.accept().unwrap();
            let mut ws = tungstenite::accept(s).unwrap();
            ws.read().unwrap();
            let grant = json!({"_tag":"Exit","requestId":"1","exit":{"_tag":"Success","value":{"relativeUrl":"/api/assets/tok/a.txt?sig=1"}}});
            ws.send(tungstenite::Message::Text(grant.to_string().into()))
                .unwrap();
            let (mut s, _) = listener.accept().unwrap();
            seen.push(read_request(&mut s));
            let _ = s.write_all(&answer);
        }
        assert_eq!(seen.len(), grants);
        seen
    });
    (Connection::new(&base, None, bearer()).unwrap(), server)
}
fn resource() -> Value {
    attachment_resource("att-1", "a.txt", "text/plain", true).unwrap()
}

#[test]
fn signed_assets_and_documents_decode_and_keep_their_limits() {
    for (encoding, encode) in ENCODERS {
        let body = vec![b'x'; 2000];
        let answer = encoded("text/plain", encoding, &encode(&body));
        let (c, server) = asset_server(vec![answer.clone(), answer.clone(), answer]);
        let asset = c.asset_bytes(resource(), 2000).unwrap();
        assert_eq!(
            (asset.bytes.as_slice(), asset.mime.as_str()),
            (body.as_slice(), "text/plain")
        );
        assert_eq!(
            c.asset_bytes(resource(), 1999).err().unwrap(),
            "asset_too_large"
        );
        let document = c.document(resource()).unwrap();
        assert_eq!(document["contents"].as_str().unwrap().len(), 2000);
        let seen = server.join().unwrap();
        assert!(seen.iter().all(accepts_compression), "{}", seen[0].head);
        assert!(seen[0].header("authorization").is_none());
    }
}

#[test]
fn documents_apply_their_own_limit_after_decoding() {
    for (encoding, encode) in ENCODERS {
        let answer = encoded("text/plain", encoding, &encode(&vec![b'x'; 4 * MIB + 1]));
        let (c, server) = asset_server(vec![answer]);
        assert_eq!(c.document(resource()).unwrap_err(), "document_limit_4MiB");
        server.join().unwrap();
    }
}

#[test]
fn the_limit_is_exact_after_decoding() {
    let limit = 64 * 1024;
    for encoder in ENCODERS {
        let (base, server) = serve(vec![
            json_answer(encoder, &json_of_len(limit, 'a')),
            json_answer(encoder, &json_of_len(limit + 1, 'a')),
        ]);
        let c = Connection::new(&base, None, bearer()).unwrap();
        let fits = c
            .request_json_with_limit("GET", "/api/a", None, false, limit as u64)
            .unwrap();
        assert_eq!(fits.as_str().unwrap().len(), limit - 2);
        let over = c.request_json_with_limit("GET", "/api/b", None, false, limit as u64);
        assert_eq!(over.unwrap_err(), "too_large");
        server.join().unwrap();
    }
}

#[test]
fn a_compression_bomb_fails_cleanly_at_the_default_limit() {
    // 64 MiB of zeros in a body that would pass any check on its wire size.
    let bomb = vec![0; 64 * MIB];
    for encoder in ENCODERS {
        let answer = json_answer(encoder, &bomb);
        assert!(answer.len() < MIB, "{} bytes on the wire", answer.len());
        let (base, server) = serve(vec![answer]);
        let c = Connection::new(&base, None, bearer()).unwrap();
        let started = Instant::now();
        assert_eq!(
            c.request_json("GET", "/api/orchestration/shell", None, true)
                .unwrap_err(),
            "too_large"
        );
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "{:?}",
            started.elapsed()
        );
        server.join().unwrap();
    }
}

#[test]
fn corrupt_and_truncated_streams_fail_like_any_broken_body() {
    let body = json_of_len(32 * 1024, 'q');
    for (encoding, encode) in ENCODERS {
        let whole = encode(&body);
        let (base, server) = serve(vec![
            encoded(
                "application/json",
                encoding,
                b"not a compressed stream at all",
            ),
            encoded("application/json", encoding, &whole[..whole.len() / 2]),
        ]);
        let c = Connection::new(&base, None, bearer()).unwrap();
        assert_eq!(
            c.request_json("GET", "/api/a", None, false).unwrap_err(),
            "unreachable",
            "{encoding}"
        );
        assert_eq!(
            c.request_json("GET", "/api/b", None, false).unwrap_err(),
            "unreachable",
            "{encoding}"
        );
        server.join().unwrap();
    }
    let answer = encoded("text/plain", "br", b"not a compressed stream at all");
    let (c, server) = asset_server(vec![answer]);
    assert_eq!(
        c.asset_bytes(resource(), 1024).err().unwrap(),
        "asset_unavailable"
    );
    server.join().unwrap();
}

#[test]
fn credentials_tickets_and_the_websocket_are_unchanged() {
    let key = Arc::new(DpopKey::generate());
    let dpop = Credential::dpop("synthetic-token", key, 4_102_444_800).unwrap();
    let ticket = response(
        "200 OK",
        &[("Content-Type", "application/json")],
        br#"{"ticket":"t"}"#,
    );
    let (base, server) = serve(vec![
        json_answer(ENCODERS[1], b"{}"),
        ticket.clone(),
        ticket,
        response("401 Unauthorized", &[], b""),
    ]);
    let c = Connection::new(&base, None, dpop).unwrap();
    c.request_json("GET", "/api/orchestration/shell", None, true)
        .unwrap();
    c.request_json("POST", "/api/auth/websocket-ticket", None, false)
        .unwrap();
    assert!(c.rpc("server.getConfig", json!({})).is_err());
    let seen = server.join().unwrap();
    // The proof and the token ride on a compressed read exactly as on a plain one.
    assert!(accepts_compression(&seen[0]));
    assert!(seen[0].header("dpop").is_some());
    assert_eq!(
        seen[0].header("authorization").unwrap(),
        "DPoP synthetic-token"
    );
    for request in &seen[1..3] {
        assert_eq!(
            request.header("accept-encoding").unwrap(),
            "identity",
            "{}",
            request.head
        );
        assert!(request.header("dpop").is_some());
    }
    let upgrade = &seen[3];
    assert!(
        upgrade
            .head
            .starts_with("GET /ws?orchestrationProtocol=2&wsTicket=t HTTP"),
        "{}",
        upgrade.head
    );
    assert!(
        upgrade.header("accept-encoding").is_none(),
        "{}",
        upgrade.head
    );
    assert!(
        upgrade.header("sec-websocket-extensions").is_none(),
        "{}",
        upgrade.head
    );
}

#[test]
fn the_endless_mjpeg_stream_is_never_offered_compression() {
    let ticket = response(
        "200 OK",
        &[("Content-Type", "application/json")],
        br#"{"ticket":"t"}"#,
    );
    let frames = response(
        "200 OK",
        &[("Content-Type", "multipart/x-mixed-replace")],
        b"",
    );
    let (base, server) = serve(vec![ticket, frames]);
    let c = Connection::new(&base, None, bearer()).unwrap();
    let access = c.device_hub("/api/device-hub", "host-1").unwrap();
    c.device_mjpeg(&access, "UDID").unwrap();
    assert_eq!(
        server.join().unwrap()[1].header("accept-encoding").unwrap(),
        "identity"
    );
}

/// A deterministic thread shell of exactly `len` bytes: ids, timestamps, repeated
/// keys and prose, in the proportions of a real `/api/orchestration/shell`.
fn shell_body(len: usize) -> Vec<u8> {
    const WORDS: [&str; 12] = [
        "review",
        "android",
        "bridge",
        "fix",
        "sidebar",
        "reconnect",
        "theme",
        "plan",
        "tests",
        "release",
        "gateway",
        "shell",
    ];
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let mut out = String::with_capacity(len);
    out.push_str(r#"{"sequence":48213,"threads":["#);
    while out.len() + 1024 < len {
        let (id, project) = (next(), next() % 24);
        let title = (0..3 + next() % 5)
            .map(|_| WORDS[(next() % 12) as usize])
            .collect::<Vec<_>>()
            .join(" ");
        let (day, hour, minute) = (1 + next() % 28, next() % 24, next() % 60);
        out.push_str(&format!(
            r#"{{"id":"{:016x}-{:016x}","projectId":"project-{project:02}","title":"{title}","archived":false,"status":"idle","createdAt":"2026-10-{day:02}T{hour:02}:{minute:02}:00.000Z","updatedAt":"2026-10-{day:02}T{hour:02}:{minute:02}:30.000Z","modelSelection":{{"provider":"claude","model":"sonnet"}},"branch":"feat/{}","latestTurnId":"turn-{:08x}","hasPendingApprovals":false}},"#,
            id, next(), WORDS[(id % 12) as usize], next() as u32,
        ));
    }
    out.pop();
    out.push_str(r#"],"note":""#);
    let tail = "\"}";
    while out.len() + tail.len() < len {
        out.push('.');
    }
    out.push_str(tail);
    out.into_bytes()
}
/// Negotiates `Accept-Encoding` as T3 does and counts the bytes it writes to the socket.
fn negotiating_server(
    body: Arc<Vec<u8>>,
    brotli_quality: u32,
) -> (String, JoinHandle<(usize, String)>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        let (mut s, _) = listener.accept().unwrap();
        let accepted = read_request(&mut s)
            .header("accept-encoding")
            .unwrap_or_default();
        let offers = |name: &str| accepted.split(',').any(|e| e.trim() == name);
        let answer = if offers("br") {
            encoded("application/json", "br", &brotli(&body, brotli_quality))
        } else if offers("gzip") {
            encoded("application/json", "gzip", &gzip(&body))
        } else {
            response("200 OK", &[("Content-Type", "application/json")], &body)
        };
        s.write_all(&answer).unwrap();
        (answer.len(), accepted)
    });
    (base, server)
}
fn wire_bytes(
    client: reqwest::blocking::Client,
    body: &Arc<Vec<u8>>,
    brotli_quality: u32,
) -> (usize, String) {
    let (base, server) = negotiating_server(body.clone(), brotli_quality);
    let c = Connection::with_http_client(client, &base, None, bearer()).unwrap();
    let shell = c
        .request_json("GET", "/api/orchestration/shell", None, true)
        .unwrap();
    assert_eq!(shell, serde_json::from_slice::<Value>(body).unwrap());
    server.join().unwrap()
}

#[test]
fn a_cold_start_shell_costs_a_fraction_of_its_size_on_the_wire() {
    let size = 1_139_497; // The shell measured on the validation host.
    let body = Arc::new(shell_body(size));
    assert_eq!(body.len(), size);
    let builder = || reqwest::blocking::Client::builder().no_proxy();
    let plain = builder().no_gzip().no_brotli().build().unwrap();
    let (plain, _) = wire_bytes(plain, &body, 4);
    let gzip_only = builder().no_brotli().build().unwrap();
    let (gzip_only, offered) = wire_bytes(gzip_only, &body, 4);
    assert_eq!(offered, "gzip");
    let (fast, offered) = wire_bytes(builder().build().unwrap(), &body, 4);
    assert!(
        offered.contains("gzip") && offered.contains("br"),
        "{offered}"
    );
    let (best, _) = wire_bytes(builder().build().unwrap(), &body, 11);
    println!("wire bytes for {size} decoded: identity {plain}, gzip {gzip_only}, br q4 {fast}, br q11 {best}");
    assert!(plain > size);
    assert!(gzip_only * 3 < plain && fast * 3 < plain && best * 3 < plain);
}
