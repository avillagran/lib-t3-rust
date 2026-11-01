mod common;
use common::{json, response, serve};
use serde_json::json;
use std::{
    net::TcpListener,
    thread,
    time::{Duration, Instant},
};
use t3_protocol::{Connection, ConnectionOptions, Credential};
fn bearer() -> Credential {
    Credential::bearer("synthetic-token").unwrap()
}
#[test]
fn defaults_are_the_historical_policy() {
    let o = ConnectionOptions::default();
    assert_eq!(o.request_timeout(), Duration::from_secs(12));
    assert_eq!(o.connect_timeout(), None);
    assert_eq!(o.pool_idle_timeout(), Some(Duration::from_secs(90)));
    assert_eq!(o.max_bytes(), 8 * 1024 * 1024);
    let c = Connection::new("http://127.0.0.1:9", None, bearer()).unwrap();
    assert_eq!(c.max_bytes(), 8 * 1024 * 1024);
}
#[test]
fn invalid_options_are_rejected_before_any_request() {
    for o in [
        ConnectionOptions::default().with_request_timeout(Duration::ZERO),
        ConnectionOptions::default().with_request_timeout(Duration::from_secs(301)),
        ConnectionOptions::default().with_connect_timeout(Some(Duration::ZERO)),
        ConnectionOptions::default().with_max_bytes(0),
        ConnectionOptions::default().with_max_bytes(64 * 1024 * 1024 + 1),
    ] {
        let error = Connection::with_options("http://127.0.0.1:9", None, bearer(), &o).err();
        assert_eq!(error.as_deref(), Some("invalid_connection_options"));
    }
}
#[test]
fn content_type_is_case_insensitive_and_may_carry_parameters() {
    let (base, server) = serve(vec![
        response("200 OK", &[("Content-Type", "Application/JSON; charset=UTF-8")], b"{\"a\":1}"),
        response("200 OK", &[("Content-Type", "application/json"), ("Content-Encoding", "Identity")], b"{\"b\":2}"),
        response("200 OK", &[("Content-Type", "text/plain")], b"{}"),
        response("200 OK", &[("Content-Type", "application/json"), ("Content-Encoding", "zstd")], b"{}"),
    ]);
    let c = Connection::new(&base, None, bearer()).unwrap();
    assert_eq!(c.request_json("GET", "/api/one", None, false).unwrap(), json!({"a":1}));
    assert_eq!(c.request_json("GET", "/api/two", None, false).unwrap(), json!({"b":2}));
    assert_eq!(c.request_json("GET", "/api/three", None, false).unwrap_err(), "not_json");
    assert_eq!(c.request_json("GET", "/api/four", None, false).unwrap_err(), "unsupported_encoding");
    server.join().unwrap();
}
#[test]
fn status_codes_map_to_stable_errors_and_redirects_are_not_followed() {
    let (base, server) = serve(vec![
        response("401 Unauthorized", &[], b""),
        response("403 Forbidden", &[], b""),
        response("500 Internal Server Error", &[("Content-Type", "application/json")], b"{\"secret\":1}"),
        response("302 Found", &[("Location", "/api/elsewhere")], b""),
    ]);
    let c = Connection::new(&base, None, bearer()).unwrap();
    assert_eq!(c.request_json("GET", "/api/a", None, false).unwrap_err(), "unauthorized");
    assert_eq!(c.request_json("GET", "/api/b", None, false).unwrap_err(), "unauthorized");
    assert_eq!(c.request_json("GET", "/api/c", None, false).unwrap_err(), "http_500");
    assert_eq!(c.request_json("GET", "/api/d", None, false).unwrap_err(), "http_302");
    // Exactly four connections: the redirect target was never requested.
    let seen = server.join().unwrap();
    assert_eq!(seen.len(), 4);
    assert!(seen[0].head.starts_with("GET /api/a HTTP"));
    assert_eq!(seen[0].header("authorization").unwrap(), "Bearer synthetic-token");
    assert_eq!(seen[0].header("accept").unwrap(), "application/json");
    let accepted = seen[0].header("accept-encoding").unwrap();
    assert!(accepted.contains("gzip") && accepted.contains("br"), "{accepted}");
    assert!(seen[0].header("x-t3-orchestration-protocol").is_none());
}
#[test]
fn request_ok_accepts_any_2xx_body_and_keeps_error_codes() {
    let (base, server) = serve(vec![
        response("204 No Content", &[], b""),
        response("200 OK", &[("Content-Type", "text/plain")], b"done"),
        response("200 OK", &[("Content-Type", "application/json")], b"{\"revoked\":true}"),
        response("404 Not Found", &[], b""),
        response("403 Forbidden", &[], b""),
    ]);
    let c = Connection::new(&base, None, bearer()).unwrap();
    let body = json!({"sessionId":"s-1"});
    c.request_ok("POST", "/api/auth/clients/revoke", Some(&body), false).unwrap();
    c.request_ok("POST", "/api/auth/clients/revoke", Some(&body), false).unwrap();
    c.request_ok("POST", "/api/auth/clients/revoke", Some(&body), false).unwrap();
    assert_eq!(c.request_ok("POST", "/api/x", None, false).unwrap_err(), "http_404");
    assert_eq!(c.request_ok("POST", "/api/x", None, true).unwrap_err(), "unauthorized");
    let seen = server.join().unwrap();
    assert!(seen[0].head.starts_with("POST /api/auth/clients/revoke HTTP"));
    assert_eq!(serde_json::from_str::<serde_json::Value>(&seen[0].body).unwrap(), body);
    assert_eq!(seen[4].header("x-t3-orchestration-protocol").unwrap(), "2");
}
#[test]
fn response_limit_is_configurable_per_connection_and_per_call() {
    let body = format!("{{\"text\":\"{}\"}}", "x".repeat(64));
    let (base, server) = serve(vec![json(&body), json(&body), json(&body), json(&body)]);
    let small = ConnectionOptions::default().with_max_bytes(32);
    let c = Connection::with_options(&base, None, bearer(), &small).unwrap();
    assert_eq!(c.max_bytes(), 32);
    assert_eq!(c.request_json("GET", "/api/orchestration/shell", None, true).unwrap_err(), "too_large");
    // A bounded thread snapshot keeps its own 64 MiB allowance.
    assert!(c.request_json("GET", "/api/orchestration/threads/t/bounded?compactTurnItems=1", None, true).is_ok());
    assert!(c.request_json_with_limit("GET", "/api/orchestration/shell", None, true, 1024).is_ok());
    assert_eq!(
        c.request_json_with_limit("GET", "/api/orchestration/threads/t/bounded", None, true, 16).unwrap_err(),
        "too_large"
    );
    for limit in [0, 64 * 1024 * 1024 + 1] {
        assert_eq!(c.request_json_with_limit("GET", "/api/x", None, true, limit).unwrap_err(), "invalid_max_bytes");
    }
    server.join().unwrap();
}
#[test]
fn request_timeout_bounds_a_silent_server() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        let (s, _) = listener.accept().unwrap();
        thread::sleep(Duration::from_secs(3));
        drop(s);
    });
    let o = ConnectionOptions::default()
        .with_request_timeout(Duration::from_millis(300))
        .with_connect_timeout(Some(Duration::from_millis(200)))
        .with_pool_idle_timeout(None);
    let c = Connection::with_options(&base, None, bearer(), &o).unwrap();
    let start = Instant::now();
    assert_eq!(c.request_json("GET", "/api/auth/session", None, false).unwrap_err(), "unreachable");
    assert!(start.elapsed() < Duration::from_secs(2), "{:?}", start.elapsed());
    server.join().unwrap();
}
#[test]
fn with_credential_shares_endpoints_and_limits() {
    let (base, server) = serve(vec![json("{}"), json("{}")]);
    let o = ConnectionOptions::default().with_max_bytes(4096);
    let read = Connection::with_options(&base, None, Credential::bearer("read-token-1").unwrap(), &o).unwrap();
    let write = read.with_credential(Credential::bearer("write-token-1").unwrap());
    assert_eq!(write.base_url(), read.base_url());
    assert_eq!(write.max_bytes(), 4096);
    read.request_json("GET", "/api/auth/session", None, false).unwrap();
    write.request_json("GET", "/api/auth/session", None, false).unwrap();
    let seen = server.join().unwrap();
    assert_eq!(seen[0].header("authorization").unwrap(), "Bearer read-token-1");
    assert_eq!(seen[1].header("authorization").unwrap(), "Bearer write-token-1");
}
