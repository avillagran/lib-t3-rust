use serde_json::{json, Value};
use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
};
use t3_protocol::{Connection, Credential};
const PNG: &[u8] = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR";
/// One signed asset grant over the RPC socket, then the unauthenticated signed GET.
fn asset_server(resource: Value, content_type: &'static str, body: &'static [u8]) -> (Connection, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        let (s, _) = listener.accept().unwrap();
        let mut ws = tungstenite::accept(s).unwrap();
        let r: Value = serde_json::from_str(ws.read().unwrap().to_text().unwrap()).unwrap();
        assert_eq!(r["tag"], "assets.createUrl");
        assert_eq!(r["payload"]["resource"], resource);
        ws.send(tungstenite::Message::Text(json!({"_tag":"Exit","requestId":"1","exit":{"_tag":"Success","value":{"relativeUrl":"/api/assets/signed?signature=fixture"}}}).to_string().into())).unwrap();
        let (mut s, _) = listener.accept().unwrap();
        let mut header = vec![];
        let mut b = [0];
        while !header.ends_with(b"\r\n\r\n") {
            s.read_exact(&mut b).unwrap();
            header.push(b[0]);
        }
        let header = String::from_utf8(header).unwrap().to_lowercase();
        assert!(header.starts_with("get /api/assets/signed?signature=fixture "));
        assert!(!header.contains("authorization:"));
        write!(s, "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).unwrap();
        s.write_all(body).unwrap();
    });
    (Connection::new(&base, None, Credential::bearer("synthetic").unwrap()).unwrap(), server)
}
#[test]
fn media_image_uses_the_exact_thread_and_path_and_reports_the_served_type() {
    let (c, server) = asset_server(json!({"_tag":"media-file","threadId":"remote-thread","path":"/home/other/a.png"}), "image/png; charset=binary", PNG);
    let (bytes, mime) = c.media_image("remote-thread", "/home/other/a.png").unwrap();
    assert_eq!((bytes.as_slice(), mime.as_str()), (PNG, "image/png"));
    server.join().unwrap();
}
#[test]
fn media_image_rejects_bytes_that_do_not_match_the_served_type() {
    let (c, server) = asset_server(json!({"_tag":"media-file","threadId":"t","path":"notes.txt"}), "image/png", b"not an image");
    assert_eq!(c.media_image("t", "notes.txt").unwrap_err(), "invalid_image");
    server.join().unwrap();
}
#[test]
fn media_image_validates_scope_before_connecting() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let c = Connection::new(&format!("http://{}", listener.local_addr().unwrap()), None, Credential::bearer("synthetic").unwrap()).unwrap();
    let long = "a".repeat(4097);
    for (thread, path) in [("", "/a.png"), ("t", ""), ("t", "/a\n.png"), ("t", long.as_str())] {
        assert_eq!(c.media_image(thread, path).unwrap_err(), "invalid_image");
    }
    assert_eq!(listener.accept().unwrap_err().kind(), std::io::ErrorKind::WouldBlock);
}
#[test]
fn attachment_image_requires_the_served_type_to_match_the_attachment() {
    let (c, server) = asset_server(json!({"_tag":"attachment","attachmentId":"a","fileName":"a.png","mimeType":"image/png","disposition":"inline"}), "image/jpeg", PNG);
    assert_eq!(c.attachment_image("a", "a.png", "image/png").unwrap_err(), "invalid_image");
    server.join().unwrap();
}
#[test]
fn attachment_image_reads_heic_heif_and_avif_by_their_ftyp_box() {
    const AVIF: &[u8] = b"\0\0\0\x1cftypavif\0\0\0\0avifmif1";
    let (c, server) = asset_server(json!({"_tag":"attachment","attachmentId":"a","fileName":"a.avif","mimeType":"image/avif","disposition":"inline"}), "image/avif", AVIF);
    assert_eq!(c.attachment_image("a", "a.avif", "image/avif").unwrap(), AVIF);
    server.join().unwrap();
    let (c, server) = asset_server(json!({"_tag":"attachment","attachmentId":"h","fileName":"h.heic","mimeType":"image/heic","disposition":"inline"}), "image/heic", b"\0\0\0\x18fytpheic");
    assert_eq!(c.attachment_image("h", "h.heic", "image/heic").unwrap_err(), "invalid_image");
    server.join().unwrap();
    let (c, server) = asset_server(json!({"_tag":"media-file","threadId":"t","path":"/a.heif"}), "image/heif", b"\0\0\0\x18ftypmif1");
    assert_eq!(c.media_image("t", "/a.heif").unwrap().1, "image/heif");
    server.join().unwrap();
    assert_eq!(c.attachment_image("a", "a.tiff", "image/tiff").unwrap_err(), "invalid_image");
}
