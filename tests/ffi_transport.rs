mod common;
use common::{json, read_request, response, serve};
use serde_json::{json, Value};
use std::{
    ffi::{c_void, CStr},
    io::Write,
    net::TcpListener,
    thread,
};
use t3_protocol::ffi::*;
const ENV: &str = "00000000-0000-4000-8000-000000000001";
struct Client(*mut c_void);
impl Client {
    fn new() -> Self {
        Self(t3_client_new())
    }
    fn call(&self, value: Value) -> Value {
        let bytes = serde_json::to_vec(&value).unwrap();
        unsafe { read(t3_client_call(self.0, bytes.as_ptr(), bytes.len())) }
    }
    /// The binary path: returns the JSON status and the bytes the callback received.
    fn binary(&self, value: Value) -> (Value, Option<Vec<u8>>) {
        unsafe extern "C" fn receive(context: *mut c_void, bytes: *const u8, len: usize) {
            let out = &mut *context.cast::<Option<Vec<u8>>>();
            *out = Some(std::slice::from_raw_parts(bytes, len).to_vec());
        }
        let bytes = serde_json::to_vec(&value).unwrap();
        let mut out: Option<Vec<u8>> = None;
        let status = unsafe {
            read(t3_client_image(self.0, bytes.as_ptr(), bytes.len(), Some(receive), (&mut out as *mut Option<Vec<u8>>).cast()))
        };
        (status, out)
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        unsafe { t3_client_free(self.0) }
    }
}
unsafe fn read(raw: *mut std::ffi::c_char) -> Value {
    let value = serde_json::from_slice(CStr::from_ptr(raw).to_bytes()).unwrap();
    t3_string_free(raw);
    value
}
#[test]
fn capabilities_list_the_new_operations_under_abi_3() {
    let caps = Client::new().call(json!({"op":"capabilities"}))["data"].clone();
    // The transport ops are additive; ABI 3 comes from the typed API (call/describe).
    assert_eq!(caps["abiVersion"], 3);
    for op in ["connectBearer", "request", "requestOk", "fetchDescriptor", "pairBearer", "boundedThread", "assetBytes", "projectFavicon"] {
        assert!(caps["operations"].as_array().unwrap().contains(&json!(op)), "{op}");
    }
    assert_eq!(caps["binaryOperations"], json!(["image", "assetBytes", "projectFavicon", "registryIconBytes"]));
    assert_eq!(caps["maxAssetBytes"], 64 * 1024 * 1024);
}
#[test]
fn connect_options_are_validated_and_applied() {
    let c = Client::new();
    for options in [
        json!({"requestTimeoutMs":0}),
        json!({"requestTimeoutMs":"8000"}),
        json!({"maxBytes":0}),
        json!({"requestTimeout":8000}),
        json!([8000]),
    ] {
        let r = c.call(json!({"op":"connectBearer","httpBaseUrl":"http://127.0.0.1:9","token":"synthetic-token","options":options}));
        assert_eq!(r["error"], "invalid_connection_options", "{options}");
    }
    let body = format!("{{\"text\":\"{}\"}}", "x".repeat(64));
    let (base, server) = serve(vec![json(&body), json(&body), response("204 No Content", &[], b"")]);
    let options = json!({"requestTimeoutMs":8000,"connectTimeoutMs":3000,"poolIdleTimeoutMs":90000,"maxBytes":32});
    assert_eq!(c.call(json!({"op":"connectBearer","httpBaseUrl":base,"token":"synthetic-token","options":options}))["ok"], true);
    assert_eq!(c.call(json!({"op":"request","method":"GET","path":"/api/orchestration/shell"}))["error"], "too_large");
    assert_eq!(c.call(json!({"op":"request","method":"GET","path":"/api/orchestration/shell","maxBytes":1024}))["ok"], true);
    assert_eq!(c.call(json!({"op":"request","method":"GET","path":"/api/x","maxBytes":-1}))["error"], "invalid_max_bytes");
    assert_eq!(
        c.call(json!({"op":"requestOk","method":"POST","path":"/api/auth/clients/revoke","body":{"sessionId":"s"},"protocol":false})),
        json!({"ok":true,"data":{"succeeded":true}})
    );
    server.join().unwrap();
}
#[test]
fn descriptor_and_pairing_work_without_a_connection() {
    let c = Client::new();
    let descriptor = json!({"environmentId":ENV,"label":"Studio"}).to_string();
    let token = json!({"access_token":"paired-bearer-token","token_type":"Bearer","expires_in":60,"scope":"orchestration:read"}).to_string();
    let (base, server) = serve(vec![
        json(&descriptor),
        json(&descriptor),
        json(&descriptor),
        json(&token),
        json("{\"authenticated\":true}"),
    ]);
    assert_eq!(c.call(json!({"op":"fetchDescriptor","httpBaseUrl":base}))["data"]["label"], "Studio");
    let mismatch = c.call(json!({"op":"fetchDescriptor","httpBaseUrl":base,"environmentId":"11111111-1111-4111-8111-111111111111"}));
    assert_eq!(mismatch["error"], "t3_environment_mismatch");
    let paired = c.call(json!({"op":"pairBearer","httpBaseUrl":base,"pairingCode":"pairing-secret-1","environmentId":ENV,
        "client":{"label":"Omi","deviceType":"mobile","os":"android"},"connect":true,"options":{"requestTimeoutMs":8000}}));
    assert_eq!(paired["data"]["accessToken"], "paired-bearer-token");
    assert_eq!(paired["data"]["environmentId"], ENV);
    assert_eq!(paired["data"]["descriptor"]["label"], "Studio");
    assert_eq!(paired["data"]["connected"], true);
    assert_eq!(c.call(json!({"op":"request","method":"GET","path":"/api/auth/session","protocol":false}))["data"]["authenticated"], true);
    let seen = server.join().unwrap();
    assert!(seen[3].body.contains("client_label=Omi") && seen[3].body.contains("client_device_type=mobile") && seen[3].body.contains("client_os=android"));
    assert_eq!(seen[4].header("authorization").unwrap(), "Bearer paired-bearer-token");
    for (request, error) in [
        (json!({"op":"pairBearer","httpBaseUrl":"http://127.0.0.1:9"}), "invalid_pairing_code"),
        (json!({"op":"pairBearer","httpBaseUrl":"http://127.0.0.1:9","pairingCode":"pairing-secret-1","client":{"deviceType":"phone"}}), "invalid_pairing_client"),
        (json!({"op":"pairBearer","httpBaseUrl":"http://127.0.0.1:9","pairingCode":"pairing-secret-1","client":{"model":"x"}}), "invalid_pairing_client"),
        (json!({"op":"fetchDescriptor","httpBaseUrl":"https://host/prefix"}), "invalid_endpoint"),
    ] {
        assert_eq!(c.call(request.clone())["error"], error, "{request}");
    }
}
#[test]
fn bounded_thread_restores_through_the_abi() {
    let c = Client::new();
    assert_eq!(c.call(json!({"op":"boundedThread","threadId":"t"}))["error"], "not_connected");
    let snapshot = json!({"turnItemsOmitLocalVisible":true,"projection":{"thread":{"id":"t"},"turnItems":[{"id":"tail"}],"visibleTurnItems":[{"visibility":"local","item":{"id":"local"}}]}}).to_string();
    let (base, server) = serve(vec![json(&snapshot)]);
    c.call(json!({"op":"connectBearer","httpBaseUrl":base,"token":"synthetic-token"}));
    let r = c.call(json!({"op":"boundedThread","threadId":"t"}));
    assert_eq!(r["data"]["projection"]["turnItems"], json!([{"id":"local"},{"id":"tail"}]));
    assert!(server.join().unwrap()[0].head.starts_with("GET /api/orchestration/threads/t/bounded?compactTurnItems=1&dedupeBodies=1&compactIds=1 HTTP"));
    assert_eq!(c.call(json!({"op":"boundedThread","threadId":"../x"}))["error"], "invalid_id");
}
/// A grant over RPC, then one signed GET answered with `answer` (if any).
fn asset_server(relative: &'static str, answer: Option<Vec<u8>>) -> (String, thread::JoinHandle<Value>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        let (s, _) = listener.accept().unwrap();
        let mut ws = tungstenite::accept(s).unwrap();
        let r: Value = serde_json::from_str(ws.read().unwrap().to_text().unwrap()).unwrap();
        ws.send(tungstenite::Message::Text(json!({"_tag":"Exit","requestId":"1","exit":{"_tag":"Success","value":{"relativeUrl":relative}}}).to_string().into())).unwrap();
        if let Some(answer) = answer {
            let (mut s, _) = listener.accept().unwrap();
            let seen = read_request(&mut s);
            assert!(seen.header("authorization").is_none());
            s.write_all(&answer).unwrap();
        }
        r["payload"]["resource"].clone()
    });
    (base, server)
}
#[test]
fn asset_bytes_and_favicons_return_bytes_through_the_image_callback() {
    let c = Client::new();
    assert_eq!(c.call(json!({"op":"assetBytes"}))["error"], "binary_operation");
    let (status, bytes) = c.binary(json!({"op":"assetBytes","resource":{"_tag":"attachment"},"maxBytes":10}));
    assert_eq!((status["error"].as_str(), bytes), (Some("not_connected"), None));

    let resource = json!({"_tag":"attachment","attachmentId":"a","fileName":"clip.mp4","mimeType":"video/mp4","disposition":"inline"});
    let (base, server) = asset_server("/api/assets/tok/clip.mp4", Some(response("200 OK", &[("Content-Type", "video/mp4")], b"\x00\x01binary\xff")));
    c.call(json!({"op":"connectBearer","httpBaseUrl":base,"token":"synthetic-token"}));
    let (status, bytes) = c.binary(json!({"op":"assetBytes","resource":resource,"maxBytes":50 * 1024 * 1024,"timeoutMs":120000}));
    assert_eq!(status, json!({"ok":true,"data":{"sizeBytes":9,"mimeType":"video/mp4"}}));
    assert_eq!(bytes.unwrap(), b"\x00\x01binary\xff");
    assert_eq!(server.join().unwrap(), resource);

    for (request, error) in [
        (json!({"op":"assetBytes","resource":resource}), "invalid_asset_limit"),
        (json!({"op":"assetBytes","resource":resource,"maxBytes":10,"timeoutMs":0}), "invalid_asset_timeout"),
        (json!({"op":"assetBytes","maxBytes":10}), "invalid_asset_resource"),
        (json!({"op":"projectFavicon","cwd":"/repo","path":7}), "invalid_asset_resource"),
        (json!({"op":"thumbnail"}), "unknown_operation"),
    ] {
        let (status, bytes) = c.binary(request.clone());
        assert_eq!((status["error"].as_str(), bytes), (Some(error), None), "{request}");
    }

    let (base, server) = asset_server("/api/assets/tok/project-favicon-missing", None);
    c.call(json!({"op":"connectBearer","httpBaseUrl":base,"token":"synthetic-token"}));
    let (status, bytes) = c.binary(json!({"op":"projectFavicon","cwd":"/repo"}));
    assert_eq!((status, bytes), (json!({"ok":true,"data":{"found":false,"sizeBytes":0}}), None));
    assert_eq!(server.join().unwrap(), json!({"_tag":"project-favicon","cwd":"/repo"}));

    let (base, server) = asset_server("/api/assets/tok/favicon.svg", Some(response("200 OK", &[("Content-Type", "image/svg+xml")], b"<svg/>")));
    c.call(json!({"op":"connectBearer","httpBaseUrl":base,"token":"synthetic-token"}));
    let (status, bytes) = c.binary(json!({"op":"projectFavicon","cwd":"/repo","path":"favicon.svg"}));
    assert_eq!(status["data"], json!({"found":true,"sizeBytes":6,"mimeType":"image/svg+xml"}));
    assert_eq!(bytes.unwrap(), b"<svg/>");
    server.join().unwrap();
}

#[test]
fn parity_native_http_reads_authenticate_and_keep_opaque_cursors() {
 let c=Client::new();
 let (base,server)=serve(vec![json(r#"{"items":[],"nextCursor":null,"hasMoreHistory":false}"#),json(r#"{"files":[],"nextCursor":"opaque"}"#)]);
 assert_eq!(c.call(json!({"op":"connectBearer","httpBaseUrl":base,"token":"synthetic-token"}))["ok"],true);
 let history=c.call(json!({"op":"historyPage","threadId":"thread","cursor":"a+/=?"}));assert_eq!(history["ok"],true);
 let diff=c.call(json!({"op":"pullRequestDiff","reference":{"projectId":"p","host":"github.com","repository":"o/r","number":1},"cursor":"opaque+/="}));assert_eq!(diff["data"]["nextCursor"],"opaque");
 let seen=server.join().unwrap();assert!(seen[0].head.starts_with("GET /api/orchestration/threads/thread/history?cursor=a%2B%2F%3D%3F&compactIds=1 "));
 assert!(seen[1].head.starts_with("POST /api/pull-requests/diff "));for r in &seen {assert_eq!(r.header("authorization").as_deref(),Some("Bearer synthetic-token"));}
 assert_eq!(serde_json::from_str::<Value>(&seen[1].body).unwrap()["cursor"],"opaque+/=");
}
#[test]
fn registry_binary_rejects_foreign_url_without_connection_or_callback(){let c=Client::new();let(status,bytes)=c.binary(json!({"op":"registryIconBytes","url":"https://evil.test/icon.svg","requestId":"one"}));assert_eq!(status["error"],"invalid_registry_icon");assert!(bytes.is_none());}
