mod common;
use common::{read_request, response, Seen};
use serde_json::{json, Value};
use std::{
    io::Write,
    net::TcpListener,
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use t3_protocol::{assets::attachment_resource, AssetOptions, Connection, Credential};
struct Served {
    resource: Value,
    gets: Vec<Seen>,
    listener: TcpListener,
}
/// One `assets.createUrl` grant answering `relative`, then one signed GET per answer.
fn asset_server(relative: Value, answers: Vec<Vec<u8>>) -> (Connection, JoinHandle<Served>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        let (s, _) = listener.accept().unwrap();
        let mut ws = tungstenite::accept(s).unwrap();
        let r: Value = serde_json::from_str(ws.read().unwrap().to_text().unwrap()).unwrap();
        assert_eq!(r["tag"], "assets.createUrl");
        ws.send(tungstenite::Message::Text(json!({"_tag":"Exit","requestId":"1","exit":{"_tag":"Success","value":{"relativeUrl":relative}}}).to_string().into())).unwrap();
        let mut gets = vec![];
        for answer in answers {
            let (mut s, _) = listener.accept().unwrap();
            gets.push(read_request(&mut s));
            let _ = s.write_all(&answer);
        }
        listener.set_nonblocking(true).unwrap();
        Served { resource: r["payload"]["resource"].clone(), gets, listener }
    });
    (Connection::new(&base, None, Credential::bearer("synthetic-token").unwrap()).unwrap(), server)
}
fn ok(mime: &str, body: &[u8]) -> Vec<u8> {
    response("200 OK", &[("Content-Type", mime)], body)
}
fn no_more_requests(served: &Served) {
    assert_eq!(served.listener.accept().unwrap_err().kind(), std::io::ErrorKind::WouldBlock);
}
#[test]
fn asset_bytes_reads_a_signed_attachment_without_credentials() {
    let (c, server) = asset_server(json!("/api/assets/tok/clip.mp4?sig=1"), vec![ok("Video/MP4; codecs=avc1", b"movie-bytes")]);
    let resource = attachment_resource("att-1", "clip.mp4", "video/mp4", true).unwrap();
    let asset = c.asset_bytes(resource.clone(), 50 * 1024 * 1024).unwrap();
    assert_eq!((asset.bytes.as_slice(), asset.mime.as_str()), (&b"movie-bytes"[..], "video/mp4"));
    let served = server.join().unwrap();
    assert_eq!(served.resource, json!({"_tag":"attachment","attachmentId":"att-1","fileName":"clip.mp4","mimeType":"video/mp4","disposition":"inline"}));
    assert!(served.gets[0].head.starts_with("GET /api/assets/tok/clip.mp4?sig=1 HTTP"));
    assert!(served.gets[0].header("authorization").is_none());
    assert!(served.gets[0].header("dpop").is_none());
}
#[test]
fn attachment_resource_without_inline_omits_disposition_and_validates_sizes() {
    assert_eq!(
        attachment_resource("a", "a.txt", "text/plain", false).unwrap(),
        json!({"_tag":"attachment","attachmentId":"a","fileName":"a.txt","mimeType":"text/plain"})
    );
    let long_id = "a".repeat(257);
    let long_name = "n".repeat(513);
    let long_mime = "m".repeat(129);
    for (id, name, mime) in [("", "a", "text/plain"), (long_id.as_str(), "a", "text/plain"), ("a", long_name.as_str(), "text/plain"), ("a", "a", long_mime.as_str())] {
        assert_eq!(attachment_resource(id, name, mime, true).unwrap_err(), "invalid_attachment");
    }
}
#[test]
fn limit_is_exact_so_a_host_can_require_the_recorded_size() {
    let (c, server) = asset_server(json!("/api/assets/tok/a.bin"), vec![ok("application/octet-stream", b"0123456789")]);
    assert_eq!(c.asset_bytes(json!({"_tag":"attachment"}), 10).unwrap().bytes.len(), 10);
    server.join().unwrap();
    let (c, server) = asset_server(json!("/api/assets/tok/a.bin"), vec![ok("application/octet-stream", b"0123456789A")]);
    assert_eq!(c.asset_bytes(json!({"_tag":"attachment"}), 10).err().unwrap(), "asset_too_large");
    server.join().unwrap();
}
#[test]
fn grants_outside_the_asset_route_or_origin_are_refused_before_any_get() {
    for relative in [
        json!(null),
        json!("/api/attachments/upload/x"),
        json!("api/assets/x"),
        json!("//evil.example/api/assets/x"),
        json!("http://evil.example/api/assets/x"),
        json!("/api/assets/../orchestration/shell"),
        json!("/api/assets/x#fragment"),
        json!("/api/assets\\x"),
    ] {
        let (c, server) = asset_server(relative.clone(), vec![]);
        assert_eq!(c.asset_bytes(json!({"_tag":"attachment"}), 1024).err().unwrap(), "invalid_asset_url", "{relative}");
        no_more_requests(&server.join().unwrap());
    }
}
#[test]
fn failed_or_redirected_gets_are_unavailable() {
    for answer in [
        response("404 Not Found", &[], b""),
        response("302 Found", &[("Location", "/api/assets/elsewhere")], b""),
    ] {
        let (c, server) = asset_server(json!("/api/assets/tok/a"), vec![answer]);
        assert_eq!(c.asset_bytes(json!({"_tag":"attachment"}), 1024).err().unwrap(), "asset_unavailable");
        no_more_requests(&server.join().unwrap());
    }
}
#[test]
fn a_silent_asset_server_is_bounded_by_the_get_deadline() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        let (s, _) = listener.accept().unwrap();
        let mut ws = tungstenite::accept(s).unwrap();
        ws.read().unwrap();
        ws.send(tungstenite::Message::Text(json!({"_tag":"Exit","requestId":"1","exit":{"_tag":"Success","value":{"relativeUrl":"/api/assets/slow"}}}).to_string().into())).unwrap();
        let (s, _) = listener.accept().unwrap();
        thread::sleep(Duration::from_secs(2));
        drop(s);
    });
    let c = Connection::new(&base, None, Credential::bearer("synthetic-token").unwrap()).unwrap();
    let start = Instant::now();
    let options = AssetOptions::new(1024).with_timeout(Duration::from_millis(300));
    assert_eq!(c.asset_bytes_with(json!({"_tag":"attachment"}), &options).err().unwrap(), "asset_unreachable");
    assert!(start.elapsed() < Duration::from_millis(1500), "{:?}", start.elapsed());
    server.join().unwrap();
}
#[test]
fn invalid_limits_and_deadlines_fail_before_connecting() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let c = Connection::new(&format!("http://{}", listener.local_addr().unwrap()), None, Credential::bearer("synthetic-token").unwrap()).unwrap();
    for limit in [0, 64 * 1024 * 1024 + 1] {
        assert_eq!(c.asset_bytes(json!({}), limit).err().unwrap(), "invalid_asset_limit");
    }
    for timeout in [Duration::ZERO, Duration::from_secs(301)] {
        assert_eq!(c.asset_bytes_with(json!({}), &AssetOptions::new(1).with_timeout(timeout)).err().unwrap(), "invalid_asset_timeout");
    }
    assert_eq!(c.project_favicon("", None).err().unwrap(), "invalid_asset_resource");
    assert_eq!(c.project_favicon("/repo", Some("a\n.png")).err().unwrap(), "invalid_asset_resource");
    assert_eq!(listener.accept().unwrap_err().kind(), std::io::ErrorKind::WouldBlock);
}
#[test]
fn project_favicon_reads_the_icon_and_reports_a_missing_one_without_fetching_it() {
    let (c, server) = asset_server(json!("/api/assets/tok/favicon.svg"), vec![ok("image/svg+xml", b"<svg/>")]);
    let icon = c.project_favicon("/repo", Some("public/favicon.svg")).unwrap().unwrap();
    assert_eq!((icon.bytes.as_slice(), icon.mime.as_str()), (&b"<svg/>"[..], "image/svg+xml"));
    assert_eq!(server.join().unwrap().resource, json!({"_tag":"project-favicon","cwd":"/repo","path":"public/favicon.svg"}));

    let (c, server) = asset_server(json!("/api/assets/tok/project-favicon-missing"), vec![]);
    assert!(c.project_favicon("/repo", None).unwrap().is_none());
    let served = server.join().unwrap();
    assert_eq!(served.resource, json!({"_tag":"project-favicon","cwd":"/repo"}));
    no_more_requests(&served);

    let (c, server) = asset_server(json!("/api/assets/tok/favicon.png"), vec![ok("image/png", &vec![0; 2 * 1024 * 1024 + 1])]);
    assert_eq!(c.project_favicon("/repo", None).err().unwrap(), "asset_too_large");
    server.join().unwrap();
}
