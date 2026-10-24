//! Subscription streams through the JSON C ABI against a fake WebSocket server.
//! Never a real T3 server: each fixture accepts one socket and scripts its frames.
use serde_json::{json, Value};
use std::{
    ffi::{c_void, CStr},
    net::{TcpListener, TcpStream},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use t3_protocol::ffi::*;
use tungstenite::{Message, WebSocket};

struct Client(usize);
impl Client {
    fn connected(base: &str) -> Self {
        let client = Self(t3_client_new() as usize);
        assert_eq!(client.call(json!({"op":"connectBearer","httpBaseUrl":base,"token":"fixture-token"}))["ok"], true);
        client
    }
    fn call(&self, value: Value) -> Value {
        let bytes = serde_json::to_vec(&value).unwrap();
        unsafe {
            let raw = t3_client_call(self.0 as *mut c_void, bytes.as_ptr(), bytes.len());
            let value = serde_json::from_slice(CStr::from_ptr(raw).to_bytes()).unwrap();
            t3_string_free(raw);
            value
        }
    }
    fn subscribe(&self, tag: &str, payload: Value) -> String {
        let started = self.call(json!({"op":"subscribe","tag":tag,"payload":payload}));
        started["data"]["subscriptionId"].as_str().unwrap_or_else(|| panic!("{started}")).to_owned()
    }
    fn poll(&self, id: &str) -> Value {
        let polled = self.call(json!({"op":"pollSubscription","subscriptionId":id,"timeoutMs":1000}));
        assert_eq!(polled["ok"], true, "{polled}");
        polled["data"].clone()
    }
    /// Polls until `done` holds for a batch (or the stream closes), returning every batch.
    fn poll_until(&self, id: &str, done: impl Fn(&Value) -> bool) -> Vec<Value> {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut batches = vec![];
        while Instant::now() < deadline {
            let batch = self.poll(id);
            let finished = done(&batch) || batch["closed"] == true;
            batches.push(batch);
            if finished {
                return batches;
            }
        }
        panic!("stream never finished: {batches:?}")
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        unsafe { t3_client_free(self.0 as *mut c_void) }
    }
}

type Socket = WebSocket<TcpStream>;
/// One fake subscription socket: hands the client's Request and the socket to `script`.
fn serve<T: Send + 'static>(script: impl FnOnce(Value, &mut Socket) -> T + Send + 'static) -> (String, JoinHandle<T>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        let (socket, _) = listener.accept().unwrap();
        socket.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
        let mut ws = tungstenite::accept(socket).unwrap();
        ws.set_config(|c| {
            c.max_message_size = None;
            c.max_frame_size = None;
        });
        let request = read(&mut ws).expect("no Request frame");
        assert_eq!(request["_tag"], "Request");
        script(request, &mut ws)
    });
    (base, server)
}
fn read(ws: &mut Socket) -> Option<Value> {
    loop {
        match ws.read().ok()? {
            Message::Text(t) => return serde_json::from_str(&t).ok(),
            Message::Close(_) => return None,
            _ => {}
        }
    }
}
fn send(ws: &mut Socket, frame: Value) -> bool {
    ws.send(Message::Text(frame.to_string().into())).is_ok()
}
fn chunk(ws: &mut Socket, values: Value) -> bool {
    send(ws, json!({"_tag":"Chunk","requestId":"1","values":values}))
}
/// Reads client frames (answering nothing) until Interrupt or the socket ends; returns the tags.
fn drain_until_interrupt(ws: &mut Socket) -> Vec<String> {
    let mut tags = vec![];
    while let Some(frame) = read(ws) {
        let tag = frame["_tag"].as_str().unwrap_or("").to_owned();
        tags.push(tag.clone());
        if tag == "Interrupt" {
            break;
        }
    }
    tags
}
fn items(batches: &[Value]) -> Vec<Value> {
    batches.iter().flat_map(|b| b["items"].as_array().unwrap().clone()).collect()
}

const TWENTY_MIB: usize = 20 << 20;
fn large_snapshot() -> Value {
    json!({"kind":"snapshot","snapshotSequence":4,"projection":{"blob":"x".repeat(TWENTY_MIB)}})
}

#[test]
fn a_twenty_mib_bounded_snapshot_reaches_the_host_whole() {
    let (base, server) = serve(|request, ws| {
        assert_eq!(request["payload"]["acceptBoundedSnapshot"], true);
        assert!(chunk(ws, json!([large_snapshot()])));
        assert_eq!(read(ws).unwrap()["_tag"], "Ack");
        drain_until_interrupt(ws)
    });
    let client = Client::connected(&base);
    let id = client.subscribe("orchestration.subscribeThread", json!({"threadId":"t","acceptBoundedSnapshot":true}));
    let batches = client.poll_until(&id, |b| !b["items"].as_array().unwrap().is_empty());
    let last = batches.last().unwrap();
    assert_eq!(last["closed"], false, "{}", last["error"]);
    let received = items(&batches);
    assert_eq!(received.len(), 1);
    assert_eq!(received[0]["snapshotSequence"], 4);
    assert_eq!(received[0]["projection"]["blob"].as_str().unwrap().len(), TWENTY_MIB);
    assert_eq!(client.call(json!({"op":"cancelSubscription","subscriptionId":id}))["data"]["cancelled"], true);
    assert_eq!(server.join().unwrap().last().map(String::as_str), Some("Interrupt"));
}

#[test]
fn without_bounded_snapshots_a_twenty_mib_item_is_rejected_not_truncated() {
    let (base, server) = serve(|_, ws| {
        chunk(ws, json!([large_snapshot()]));
        drain_until_interrupt(ws)
    });
    let client = Client::connected(&base);
    let id = client.subscribe("orchestration.subscribeThread", json!({"threadId":"t"}));
    let batches = client.poll_until(&id, |_| false);
    let last = batches.last().unwrap();
    assert_eq!(last["closed"], true);
    // The 8 MiB socket limit refuses the frame, exactly as before this mode existed.
    assert_eq!(last["error"], "disconnected");
    assert!(items(&batches).is_empty(), "a partial item leaked");
    server.join().unwrap();
}
