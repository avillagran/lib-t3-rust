//! Its own test binary: the proxy variables below would leak into other tests.
mod common;
use common::{json, serve};
use std::{net::TcpListener, time::Duration};
use t3_protocol::{Connection, ConnectionOptions, Credential};
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
    let (base, server) = serve(vec![json("{\"direct\":true}"), json("{\"direct\":true}")]);
    let token = Credential::bearer("synthetic-token").unwrap();
    let c = Connection::new(&base, None, token.clone()).unwrap();
    assert_eq!(c.request_json("GET", "/api/auth/session", None, false).unwrap()["direct"], true);
    let o = ConnectionOptions::default().with_request_timeout(Duration::from_secs(8));
    let c = Connection::with_options(&base, None, token, &o).unwrap();
    assert_eq!(c.request_json("GET", "/api/auth/session", None, false).unwrap()["direct"], true);
    server.join().unwrap();
    assert!(proxy.accept().is_err(), "a request went through the proxy");
}
