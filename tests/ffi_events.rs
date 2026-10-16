use serde_json::{json, Value};
use std::{
    ffi::{c_void, CStr},
    io::{Read, Write},
    net::TcpListener,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};
use t3_protocol::ffi::*;
struct Client(usize);
impl Client {
    fn new() -> Self {
        Self(t3_client_new() as usize)
    }
    fn call(&self, value: Value) -> Value {
        invoke(self.0, value)
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        unsafe { t3_client_free(self.0 as *mut c_void) }
    }
}
fn invoke(handle: usize, value: Value) -> Value {
    let bytes = serde_json::to_vec(&value).unwrap();
    unsafe {
        let raw = t3_client_call(handle as *mut c_void, bytes.as_ptr(), bytes.len());
        let value = serde_json::from_slice(CStr::from_ptr(raw).to_bytes()).unwrap();
        t3_string_free(raw);
        value
    }
}
#[test]
fn long_poll_does_not_block_requests_and_cancel_interrupts_native_stream() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let (ready_tx, ready_rx) = mpsc::channel();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut ws = tungstenite::accept(stream).unwrap();
        let request: Value = serde_json::from_str(ws.read().unwrap().to_text().unwrap()).unwrap();
        assert_eq!(request["tag"], "orchestration.subscribeThread");
        assert_eq!(request["payload"]["afterSequence"], 9);
        ready_tx.send(()).unwrap();
        let ws_reader = thread::spawn(move || {
            let message: Value =
                serde_json::from_str(ws.read().unwrap().to_text().unwrap()).unwrap();
            assert_eq!(message["_tag"], "Interrupt");
            assert_eq!(message["requestId"], "1");
        });
        let (mut http, _) = listener.accept().unwrap();
        http.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        let mut bytes = vec![];
        let mut b = [0];
        while !bytes.ends_with(b"\r\n\r\n") {
            http.read_exact(&mut b).unwrap();
            bytes.push(b[0]);
        }
        assert!(String::from_utf8(bytes)
            .unwrap()
            .starts_with("GET /health "));
        http.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}").unwrap();
        ws_reader.join().unwrap();
    });
    let client = Client::new();
    assert_eq!(
        client.call(json!({"op":"connectBearer","httpBaseUrl":base,"token":"fixture-token"}))["ok"],
        true
    );
    let start = client.call(json!({"op":"subscribe","tag":"orchestration.subscribeThread","payload":{"threadId":"fixture","afterSequence":9}}));
    let id = start["data"]["subscriptionId"].as_str().unwrap().to_owned();
    ready_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let handle = client.0;
    let poll_id = id.clone();
    let polling = thread::spawn(move || {
        invoke(
            handle,
            json!({"op":"pollSubscription","subscriptionId":poll_id,"timeoutMs":1000}),
        )
    });
    thread::sleep(Duration::from_millis(20));
    let now = Instant::now();
    assert_eq!(
        client.call(json!({"op":"request","path":"/health","method":"GET"}))["ok"],
        true
    );
    assert!(
        now.elapsed() < Duration::from_millis(700),
        "long-poll held the connection lock"
    );
    assert_eq!(
        client.call(json!({"op":"cancelSubscription","subscriptionId":id}))["data"]["cancelled"],
        true
    );
    assert_eq!(polling.join().unwrap()["data"]["closed"], true);
    assert_eq!(
        client.call(json!({"op":"cancelSubscription","subscriptionId":id}))["data"]["cancelled"],
        false
    );
    server.join().unwrap();
}
#[test]
fn foreign_stream_preserves_items_and_terminal_error_without_replay() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut ws = tungstenite::accept(stream).unwrap();
        ws.read().unwrap();
        ws.send(tungstenite::Message::Text(json!({"_tag":"Chunk","requestId":"1","values":[{"kind":"event","sequence":10,"text":"á 🦀"}]}).to_string().into())).unwrap();
        let ack: Value = serde_json::from_str(ws.read().unwrap().to_text().unwrap()).unwrap();
        assert_eq!(ack["_tag"], "Ack");
        ws.send(tungstenite::Message::Text(
            json!({"_tag":"Exit","requestId":"1","exit":{"_tag":"Success"}})
                .to_string()
                .into(),
        ))
        .unwrap();
    });
    let client = Client::new();
    client.call(json!({"op":"connectBearer","httpBaseUrl":base,"token":"fixture-token"}));
    let id = client
        .call(json!({"op":"subscribe","tag":"orchestration.subscribeThread","payload":{}}))["data"]
        ["subscriptionId"]
        .clone();
    let mut events = vec![];
    for _ in 0..10 {
        let poll =
            client.call(json!({"op":"pollSubscription","subscriptionId":id,"timeoutMs":1000}));
        events.extend(poll["data"]["items"].as_array().unwrap().iter().cloned());
        if poll["data"]["closed"] == true {
            assert_eq!(poll["data"]["error"], "stream_closed");
            break;
        }
    }
    assert_eq!(
        events,
        vec![json!({"kind":"event","sequence":10,"text":"á 🦀"})]
    );
    assert_eq!(
        client.call(json!({"op":"pollSubscription","subscriptionId":id,"timeoutMs":0}))["data"]
            ["items"],
        json!([])
    );
    server.join().unwrap();
}
static RELEASED: AtomicUsize = AtomicUsize::new(0);
unsafe extern "C" fn read_unused(_: *mut c_void, _: *mut u8, _: usize) -> isize {
    panic!("must not read on invalid input")
}
unsafe extern "C" fn release(_: *mut c_void) {
    RELEASED.fetch_add(1, Ordering::SeqCst);
}
#[test]
fn upload_context_released_once_even_before_a_connection_exists() {
    let client = Client::new();
    let metadata = br#"{"name":"file.txt","mimeType":"text/plain","sizeBytes":7}"#;
    unsafe {
        let raw = t3_client_upload(
            client.0 as *mut c_void,
            metadata.as_ptr(),
            metadata.len(),
            Some(read_unused),
            Some(release),
            std::ptr::null_mut(),
        );
        let value: Value = serde_json::from_slice(CStr::from_ptr(raw).to_bytes()).unwrap();
        t3_string_free(raw);
        assert_eq!(value["error"], "not_connected");
    }
    assert_eq!(RELEASED.load(Ordering::SeqCst), 1);
}
struct HostReader {
    input: std::io::Cursor<Vec<u8>>,
    released: std::sync::Arc<AtomicUsize>,
}
unsafe extern "C" fn host_read(context: *mut c_void, buffer: *mut u8, len: usize) -> isize {
    let reader = &mut *context.cast::<HostReader>();
    reader
        .input
        .read(std::slice::from_raw_parts_mut(buffer, len))
        .map(|n| n as isize)
        .unwrap_or(-1)
}
unsafe extern "C" fn host_release(context: *mut c_void) {
    let reader = Box::from_raw(context.cast::<HostReader>());
    reader.released.fetch_add(1, Ordering::SeqCst);
}
fn read_http(stream: &mut std::net::TcpStream) -> String {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let mut bytes = vec![];
    let mut b = [0];
    while !bytes.ends_with(b"\r\n\r\n") {
        stream.read_exact(&mut b).unwrap();
        bytes.push(b[0]);
    }
    String::from_utf8(bytes).unwrap()
}
fn grant(listener: &TcpListener, tag: &str, value: Value) {
    let (stream, _) = listener.accept().unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let mut ws = tungstenite::accept(stream).unwrap();
    let request: Value = serde_json::from_str(ws.read().unwrap().to_text().unwrap()).unwrap();
    assert_eq!(request["tag"], tag);
    ws.send(tungstenite::Message::Text(
        json!({"_tag":"Exit","requestId":"1","exit":{"_tag":"Success","value":value}})
            .to_string()
            .into(),
    ))
    .unwrap();
}
#[test]
fn callback_upload_streams_exact_bytes_without_sending_auth_to_signed_post() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        grant(
            &listener,
            "attachments.createUploadUrl",
            json!({"attachmentId":"fixture-file","relativeUrl":"/api/attachments/upload/signed-fixture"}),
        );
        let (mut stream, _) = listener.accept().unwrap();
        let header = read_http(&mut stream).to_ascii_lowercase();
        assert!(header.starts_with("post /api/attachments/upload/signed-fixture "));
        assert!(!header.contains("authorization:"));
        assert!(header.contains("content-length: 7"));
        let mut body = [0; 7];
        stream.read_exact(&mut body).unwrap();
        assert_eq!(&body, "hi 🦀".as_bytes());
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
            .unwrap();
    });
    let client = Client::new();
    client.call(json!({"op":"connectBearer","httpBaseUrl":base,"token":"fixture-token"}));
    let released = std::sync::Arc::new(AtomicUsize::new(0));
    let context = Box::into_raw(Box::new(HostReader {
        input: std::io::Cursor::new("hi 🦀".as_bytes().to_vec()),
        released: released.clone(),
    }));
    let metadata =
        serde_json::to_vec(&json!({"name":"á.txt","mimeType":"text/plain","sizeBytes":7})).unwrap();
    unsafe {
        let raw = t3_client_upload(
            client.0 as *mut c_void,
            metadata.as_ptr(),
            metadata.len(),
            Some(host_read),
            Some(host_release),
            context.cast(),
        );
        let value: Value = serde_json::from_slice(CStr::from_ptr(raw).to_bytes()).unwrap();
        t3_string_free(raw);
        assert_eq!(value["ok"], true, "{value}");
        assert_eq!(value["data"]["id"], "fixture-file");
        assert_eq!(value["data"]["sizeBytes"], 7);
    }
    assert_eq!(released.load(Ordering::SeqCst), 1);
    server.join().unwrap();
}
unsafe extern "C" fn image_received(context: *mut c_void, bytes: *const u8, len: usize) {
    *context.cast::<Vec<u8>>() = std::slice::from_raw_parts(bytes, len).to_vec();
}
#[test]
fn native_image_callback_gets_validated_bytes_from_the_signed_grant() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        grant(
            &listener,
            "assets.createUrl",
            json!({"relativeUrl":"/api/assets/signed-fixture"}),
        );
        let (mut stream, _) = listener.accept().unwrap();
        let header = read_http(&mut stream).to_ascii_lowercase();
        assert!(header.starts_with("get /api/assets/signed-fixture "));
        assert!(!header.contains("authorization:"));
        stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 8\r\nConnection: close\r\n\r\n\x89PNG\r\n\x1a\n").unwrap();
    });
    let client = Client::new();
    client.call(json!({"op":"connectBearer","httpBaseUrl":base,"token":"fixture-token"}));
    let metadata = br#"{"id":"fixture-image","name":"image.png","mimeType":"image/png"}"#;
    let mut image: Vec<u8> = vec![];
    unsafe {
        let raw = t3_client_image(
            client.0 as *mut c_void,
            metadata.as_ptr(),
            metadata.len(),
            Some(image_received),
            (&mut image as *mut Vec<u8>).cast(),
        );
        let value: Value = serde_json::from_slice(CStr::from_ptr(raw).to_bytes()).unwrap();
        t3_string_free(raw);
        assert_eq!(value["ok"], true);
    }
    assert_eq!(image, b"\x89PNG\r\n\x1a\n");
    server.join().unwrap();
}
