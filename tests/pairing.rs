mod common;
use common::{json, response, serve};
use serde_json::{json, Value};
use std::net::TcpListener;
use t3_protocol::{
    fetch_descriptor, fetch_descriptor_with, pair_bearer, verify_environment, Connection,
    ConnectionOptions, Credential, PairingClient,
};
const ENV: &str = "00000000-0000-4000-8000-000000000001";
fn descriptor(env: &str) -> Vec<u8> {
    json(&json!({"environmentId":env,"label":"Studio","platform":{"os":"linux","arch":"x64"},"serverVersion":"0.1.2","capabilities":{}}).to_string())
}
fn form(body: &str) -> Vec<(String, String)> {
    let url = reqwest::Url::parse(&format!("http://form.invalid/?{body}")).unwrap();
    url.query_pairs().into_owned().collect()
}
fn ot3() -> PairingClient {
    PairingClient::default().with_label("ot3").with_device_type("desktop").with_os("linux")
}
#[test]
fn descriptor_is_read_without_credentials_or_protocol_header() {
    let (base, server) = serve(vec![descriptor(ENV), descriptor(ENV), descriptor("other")]);
    assert_eq!(fetch_descriptor(&base).unwrap()["label"], "Studio");
    let o = ConnectionOptions::default().with_request_timeout(std::time::Duration::from_secs(5));
    assert_eq!(verify_environment(&format!("{base}/"), ENV, &o).unwrap()["environmentId"], ENV);
    assert_eq!(verify_environment(&base, ENV, &o).unwrap_err(), "t3_environment_mismatch");
    let seen = server.join().unwrap();
    for s in &seen {
        assert!(s.head.starts_with("GET /.well-known/t3/environment HTTP"), "{}", s.head);
        assert!(s.header("authorization").is_none());
        assert!(s.header("x-t3-orchestration-protocol").is_none());
        assert_eq!(s.header("accept-encoding").unwrap(), "identity");
    }
}
#[test]
fn descriptor_failures_keep_transport_codes() {
    let (base, server) = serve(vec![
        response("404 Not Found", &[], b""),
        response("302 Found", &[("Location", "http://evil.example/")], b""),
        json("{\"label\":\"no id\"}"),
        response("200 OK", &[("Content-Type", "text/html")], b"<html>"),
    ]);
    assert_eq!(fetch_descriptor(&base).unwrap_err(), "http_404");
    assert_eq!(fetch_descriptor(&base).unwrap_err(), "http_302");
    assert_eq!(fetch_descriptor(&base).unwrap_err(), "malformed");
    assert_eq!(fetch_descriptor(&base).unwrap_err(), "not_json");
    assert_eq!(server.join().unwrap().len(), 4);
    for bad in ["ftp://host", "https://host/prefix", "https://user@host", "https://host/?q=1", "not a url"] {
        assert_eq!(fetch_descriptor(bad).unwrap_err(), "invalid_endpoint", "{bad}");
    }
    let closed = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", closed.local_addr().unwrap());
    drop(closed);
    assert_eq!(fetch_descriptor_with(&base, &ConnectionOptions::default()).unwrap_err(), "unreachable");
}
#[test]
fn pair_bearer_checks_the_descriptor_then_exchanges_the_secret() {
    let token = json!({"access_token":"paired-bearer-token","issued_token_type":"urn:ietf:params:oauth:token-type:access_token","token_type":"Bearer","expires_in":2592000,"scope":"orchestration:read orchestration:operate"});
    let (base, server) = serve(vec![descriptor(ENV), json(&token.to_string()), json("{\"authenticated\":true}")]);
    let paired = pair_bearer(&base, " pairing-secret-1 ", Some(ENV), &ot3(), &ConnectionOptions::default()).unwrap();
    assert_eq!(paired.access_token.as_str(), "paired-bearer-token");
    assert_eq!(paired.environment_id, ENV);
    assert_eq!(paired.descriptor["label"], "Studio");
    assert_eq!(paired.expires_in, Some(2592000));
    assert_eq!(paired.scope, "orchestration:read orchestration:operate");
    let session = paired.connection(&ConnectionOptions::default()).unwrap();
    assert_eq!(session.request_json("GET", "/api/auth/session", None, false).unwrap()["authenticated"], true);
    let seen = server.join().unwrap();
    assert!(seen[1].head.starts_with("POST /oauth/token HTTP"));
    assert!(seen[1].header("authorization").is_none() && seen[1].header("dpop").is_none());
    assert_eq!(seen[1].header("content-type").unwrap(), "application/x-www-form-urlencoded");
    let expected: Vec<(String, String)> = [
        ("grant_type", "urn:ietf:params:oauth:grant-type:token-exchange"),
        ("subject_token", "pairing-secret-1"),
        ("subject_token_type", "urn:t3:params:oauth:token-type:environment-bootstrap"),
        ("requested_token_type", "urn:ietf:params:oauth:token-type:access_token"),
        ("client_label", "ot3"),
        ("client_device_type", "desktop"),
        ("client_os", "linux"),
    ]
    .iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect();
    assert_eq!(form(&seen[1].body), expected);
    assert_eq!(seen[2].header("authorization").unwrap(), "Bearer paired-bearer-token");
}
#[test]
fn pairing_without_client_metadata_sends_only_the_exchange() {
    let token = json!({"access_token":"paired-bearer-token","token_type":"Bearer","expires_in":60,"scope":"orchestration:read"});
    let (base, server) = serve(vec![descriptor(ENV), json(&token.to_string())]);
    pair_bearer(&base, "pairing-secret-1", None, &PairingClient::default(), &ConnectionOptions::default()).unwrap();
    let seen = server.join().unwrap();
    assert_eq!(form(&seen[1].body).len(), 4);
}
#[test]
fn the_secret_is_never_sent_to_an_unverified_environment() {
    for (env, error) in [("11111111-1111-4111-8111-111111111111", "t3_environment_mismatch"), ("not-a-uuid", "invalid_t3_environment_id")] {
        let (base, server) = serve(vec![descriptor(env)]);
        let result = pair_bearer(&base, "pairing-secret-1", Some(ENV), &ot3(), &ConnectionOptions::default());
        assert_eq!(result.err().unwrap(), error);
        // Only the descriptor was requested.
        assert_eq!(server.join().unwrap().len(), 1);
    }
}
#[test]
fn rejected_or_unexpected_exchanges_have_stable_codes() {
    let dpop = json!({"access_token":"paired-bearer-token","token_type":"DPoP","expires_in":60,"scope":"x"});
    let short = json!({"access_token":"short","token_type":"Bearer","expires_in":60,"scope":"x"});
    for (answer, error) in [
        (response("400 Bad Request", &[("Content-Type", "application/json")], b"{\"error\":\"invalid_grant\"}"), "pairing_rejected"),
        (response("401 Unauthorized", &[], b""), "pairing_rejected"),
        (response("500 Internal Server Error", &[], b""), "pairing_rejected"),
        (json(&dpop.to_string()), "unsupported_pairing_token"),
        (json(&short.to_string()), "unsupported_pairing_token"),
        (json("{\"token_type\":\"Bearer\"}"), "malformed"),
    ] {
        let (base, server) = serve(vec![descriptor(ENV), answer]);
        let result = pair_bearer(&base, "pairing-secret-1", None, &ot3(), &ConnectionOptions::default());
        assert_eq!(result.err().unwrap(), error);
        server.join().unwrap();
    }
}
#[test]
fn invalid_secrets_and_metadata_fail_before_connecting() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let o = ConnectionOptions::default();
    let long = "s".repeat(4097);
    for secret in ["", "   ", "two words", "line\nbreak", long.as_str()] {
        assert_eq!(pair_bearer(&base, secret, None, &ot3(), &o).err().unwrap(), "invalid_pairing_code");
    }
    for client in [
        PairingClient::default().with_device_type("phone"),
        PairingClient::default().with_label(" "),
        PairingClient::default().with_os("linux\n"),
    ] {
        assert_eq!(pair_bearer(&base, "pairing-secret-1", None, &client, &o).err().unwrap(), "invalid_pairing_client");
    }
    assert_eq!(listener.accept().unwrap_err().kind(), std::io::ErrorKind::WouldBlock);
}
#[test]
fn revoke_stale_sessions_revokes_what_the_host_policy_selects() {
    let clients = json!([{"sessionId":"keep","current":true},{"sessionId":"old-1"},{"sessionId":"old-2"}]);
    let (base, server) = serve(vec![
        json(&clients.to_string()),
        json("{\"revoked\":true}"),
        response("404 Not Found", &[], b""),
    ]);
    let c = Connection::new(&base, None, Credential::bearer("synthetic-token").unwrap()).unwrap();
    let revoked = c
        .revoke_stale_sessions(|list: &Value| {
            let mut ids: Vec<String> = list.as_array().unwrap().iter().filter(|c| c["current"] != true).map(|c| c["sessionId"].as_str().unwrap().into()).collect();
            ids.extend(["".to_string(), "x".repeat(129)]);
            ids
        })
        .unwrap();
    assert_eq!(revoked, 1);
    let seen = server.join().unwrap();
    assert_eq!(seen.len(), 3);
    assert!(seen[0].head.starts_with("GET /api/auth/clients HTTP"));
    for (s, id) in seen[1..].iter().zip(["old-1", "old-2"]) {
        assert!(s.head.starts_with("POST /api/auth/clients/revoke HTTP"));
        assert_eq!(s.header("authorization").unwrap(), "Bearer synthetic-token");
        assert_eq!(serde_json::from_str::<Value>(&s.body).unwrap(), json!({"sessionId":id}));
    }
}
#[test]
fn revoke_stale_sessions_fails_when_the_list_is_unavailable() {
    let (base, server) = serve(vec![response("403 Forbidden", &[], b"")]);
    let c = Connection::new(&base, None, Credential::bearer("synthetic-token").unwrap()).unwrap();
    assert_eq!(c.revoke_stale_sessions(|_| vec!["never".into()]).unwrap_err(), "unauthorized");
    server.join().unwrap();
}
