//! Its own test binary: the proxy variables below would leak into other tests.
mod common;
use common::{json, serve};
use std::{net::TcpListener, time::Duration};
use t3_protocol::{fetch_descriptor, pair_bearer, Connection, ConnectionOptions, Credential, PairingClient};
#[test]
fn system_proxy_variables_are_ignored() {
    // A proxy that would see the request if the library honoured these variables.
    let proxy = TcpListener::bind("127.0.0.1:0").unwrap();
    proxy.set_nonblocking(true).unwrap();
    let address = format!("http://{}", proxy.local_addr().unwrap());
    for name in ["HTTP_PROXY", "http_proxy", "HTTPS_PROXY", "https_proxy", "ALL_PROXY", "all_proxy"] {
        std::env::set_var(name, &address);
    }
    std::env::remove_var("NO_PROXY");
    std::env::remove_var("no_proxy");
    let env = "00000000-0000-4000-8000-000000000001";
    let descriptor = format!("{{\"environmentId\":\"{env}\"}}");
    let token = "{\"access_token\":\"paired-bearer-token\",\"token_type\":\"Bearer\",\"expires_in\":60,\"scope\":\"x\"}";
    let (base, server) = serve(vec![
        json("{\"direct\":true}"),
        json("{\"direct\":true}"),
        json(&descriptor),
        json(&descriptor),
        json(token),
    ]);
    let token = Credential::bearer("synthetic-token").unwrap();
    let c = Connection::new(&base, None, token.clone()).unwrap();
    assert_eq!(c.request_json("GET", "/api/auth/session", None, false).unwrap()["direct"], true);
    let o = ConnectionOptions::default().with_request_timeout(Duration::from_secs(8));
    let c = Connection::with_options(&base, None, token, &o).unwrap();
    assert_eq!(c.request_json("GET", "/api/auth/session", None, false).unwrap()["direct"], true);
    assert_eq!(fetch_descriptor(&base).unwrap()["environmentId"], env);
    let paired = pair_bearer(&base, "pairing-secret-1", Some(env), &PairingClient::default(), &o).unwrap();
    assert_eq!(paired.access_token.as_str(), "paired-bearer-token");
    assert_eq!(server.join().unwrap().len(), 5);
    assert!(proxy.accept().is_err(), "a request went through the proxy");
}
