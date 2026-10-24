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

/// Rebuilds the server's chunks from poll batches using only `boundaries`.
fn chunks(batches: &[Value]) -> (Vec<Vec<Value>>, Vec<Value>) {
    let (mut done, mut open) = (vec![], vec![]);
    for batch in batches {
        let items = batch["items"].as_array().unwrap();
        let mut start = 0;
        for end in batch["boundaries"].as_array().unwrap() {
            let end = end.as_u64().unwrap() as usize;
            assert!(end >= start && end <= items.len(), "{batch}");
            open.extend_from_slice(&items[start..end]);
            done.push(std::mem::take(&mut open));
            start = end;
        }
        open.extend_from_slice(&items[start..]);
    }
    (done, open)
}

#[test]
fn poll_marks_chunk_ends_the_completion_marker_and_idle() {
    let (base, server) = serve(|request, ws| {
        assert_eq!(request["payload"], json!({"afterSequence":41,"requestCompletionMarker":true}));
        assert!(chunk(ws, json!([{"kind":"snapshot","snapshot":{"snapshotSequence":41}},{"kind":"synchronized"}])));
        assert_eq!(read(ws).unwrap()["_tag"], "Ack");
        assert!(chunk(ws, json!([
            {"kind":"thread.updated","sequence":42,"thread":{"id":"a"}},
            {"kind":"thread.removed","sequence":43,"threadId":"b"},
            {"kind":"project.updated","sequence":44,"project":{"id":"p"}},
        ])));
        assert_eq!(read(ws).unwrap()["_tag"], "Ack");
        drain_until_interrupt(ws)
    });
    let client = Client::connected(&base);
    let id = client.subscribe("orchestration.subscribeShell", json!({"afterSequence":41,"requestCompletionMarker":true}));
    // Poll until both chunks are in and the stream says it is caught up.
    let mut batches = vec![];
    while items(&batches).len() < 5 || batches.last().is_none_or(|b: &Value| b["idle"] != true) {
        let batch = client.poll(&id);
        assert_eq!(batch["closed"], false, "{batch}");
        batches.push(batch);
        assert!(batches.len() < 50, "{batches:?}");
    }
    let (done, open) = chunks(&batches);
    assert!(open.is_empty(), "items after the last boundary: {open:?}");
    assert_eq!(done.iter().map(Vec::len).collect::<Vec<_>>(), vec![2, 3]);
    assert_eq!(done[0][1], json!({"kind":"synchronized"}));
    assert_eq!(done[1][2]["sequence"], 44);
    let last = batches.last().unwrap();
    assert_eq!((last["completionMarker"].clone(), last["closed"].clone()), (json!(true), json!(false)));
    // The batch carrying the marker item is the first to report it, and it stays set.
    let first_marked = batches.iter().position(|b| b["completionMarker"] == true).unwrap();
    assert!(items(&batches[first_marked..=first_marked]).contains(&json!({"kind":"synchronized"})));
    assert!(batches[first_marked..].iter().all(|b| b["completionMarker"] == true));
    client.call(json!({"op":"cancelSubscription","subscriptionId":id}));
    server.join().unwrap();
}

/// Records when each client Ping arrived; answers it with a Pong only when `answer`.
fn pinged_server(answer: bool) -> (String, JoinHandle<Vec<Instant>>) {
    serve(move |_, ws| {
        let mut pings = vec![];
        while let Some(frame) = read(ws) {
            match frame["_tag"].as_str() {
                Some("Ping") => {
                    pings.push(Instant::now());
                    if answer {
                        send(ws, json!({"_tag":"Pong"}));
                    }
                }
                Some("Interrupt") => break,
                _ => {}
            }
        }
        pings
    })
}

#[test]
fn wake_cuts_a_silent_socket_within_the_probe_window_and_keeps_a_live_one() {
    let (silent_base, silent) = pinged_server(false);
    let (live_base, live) = pinged_server(true);
    // One handle per environment, as the host does; the op is per handle.
    let (a, b) = (Client::connected(&silent_base), Client::connected(&live_base));
    let silent_id = a.subscribe("orchestration.subscribeShell", json!({}));
    let live_id = b.subscribe("orchestration.subscribeShell", json!({}));
    // Let both workers open their sockets and reach the read loop.
    for (client, id) in [(&a, &silent_id), (&b, &live_id)] {
        client.poll_until(id, |batch| batch["idle"] == true);
    }
    assert_eq!(a.call(json!({"op":"wakeSubscriptions","subscriptionId":"missing"}))["error"], "unknown_subscription");
    assert_eq!(a.call(json!({"op":"wakeSubscriptions","subscriptionId":7}))["error"], "invalid_input");
    let woke = Instant::now();
    assert_eq!(a.call(json!({"op":"wakeSubscriptions"})), json!({"ok":true,"data":{"probed":1}}));
    assert_eq!(b.call(json!({"op":"wakeSubscriptions","subscriptionId":live_id})), json!({"ok":true,"data":{"probed":1}}));
    let batches = a.poll_until(&silent_id, |_| false);
    let cut = woke.elapsed();
    let last = batches.last().unwrap();
    assert_eq!(last["error"], "stream_timeout", "{last}");
    // STREAM_PROBE_TIMEOUT (3 s) counted from the call, observed on the next 100 ms read tick.
    assert!(cut >= Duration::from_millis(2900) && cut < Duration::from_millis(3400), "{cut:?}");
    // A stream that already ended is not probed again.
    assert_eq!(a.call(json!({"op":"wakeSubscriptions"}))["data"]["probed"], 0);
    let pings = silent.join().unwrap();
    assert!(pings[0].duration_since(woke) < Duration::from_millis(300), "no immediate ping");
    // The answering socket outlives the probe window.
    thread::sleep(Duration::from_millis(3600).saturating_sub(woke.elapsed()));
    let batch = b.poll(&live_id);
    assert_eq!(batch["closed"], false, "{batch}");
    b.call(json!({"op":"cancelSubscription","subscriptionId":live_id}));
    let pings = live.join().unwrap();
    assert!(pings[0].duration_since(woke) < Duration::from_millis(300), "no immediate ping");
}

/// Accepts `sockets` subscription sockets, each held open until its Interrupt; returns the Requests.
fn many_server(sockets: usize) -> (String, JoinHandle<Vec<Value>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        let workers: Vec<_> = (0..sockets)
            .map(|_| {
                let (socket, _) = listener.accept().unwrap();
                thread::spawn(move || {
                    socket.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
                    let mut ws = tungstenite::accept(socket).unwrap();
                    let request = read(&mut ws).unwrap();
                    drain_until_interrupt(&mut ws);
                    request
                })
            })
            .collect();
        workers.into_iter().map(|w| w.join().unwrap()).collect()
    });
    (base, server)
}

#[test]
fn eight_streams_per_handle_and_the_ninth_fails_with_a_stable_code() {
    let (base, server) = many_server(9);
    let client = Client::connected(&base);
    let bounded = json!({"threadId":"t","acceptBoundedSnapshot":true});
    let mut ids = vec![
        client.subscribe("orchestration.subscribeThread", bounded.clone()),
        client.subscribe("orchestration.subscribeThread", bounded.clone()),
    ];
    // At most two streams in the 64 MiB mode, whatever the free slots.
    let third = client.call(json!({"op":"subscribe","tag":"orchestration.subscribeThread","payload":bounded}));
    assert_eq!(third, json!({"ok":false,"error":"subscription_limit"}));
    for tag in ["orchestration.subscribeShell", "subscribeServerConfig", "subscribeDeviceState", "subscribePreviewEvents", "terminal.attach", "orchestration.subscribeThread"] {
        ids.push(client.subscribe(tag, json!({"threadId":"t"})));
    }
    let ninth = client.call(json!({"op":"subscribe","tag":"orchestration.subscribeShell","payload":{}}));
    assert_eq!(ninth, json!({"ok":false,"error":"subscription_limit"}));
    // All eight are live at once, each on its own socket.
    for id in &ids {
        let batch = client.poll_until(id, |b| b["idle"] == true).pop().unwrap();
        assert_eq!(batch["closed"], false, "{batch}");
    }
    // Cancelling frees the slot.
    assert_eq!(client.call(json!({"op":"cancelSubscription","subscriptionId":ids.remove(2)}))["data"]["cancelled"], true);
    ids.push(client.subscribe("orchestration.subscribeShell", json!({})));
    client.poll_until(ids.last().unwrap(), |b| b["idle"] == true);
    for id in &ids {
        client.call(json!({"op":"cancelSubscription","subscriptionId":id}));
    }
    let requests = server.join().unwrap();
    assert_eq!(requests.len(), 9);
    assert_eq!(requests.iter().filter(|r| r["payload"]["acceptBoundedSnapshot"] == true).count(), 2);
}

#[test]
fn a_refused_stream_reports_a_stable_code_and_the_reason_apart() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        use std::io::{Read, Write};
        let (mut socket, _) = listener.accept().unwrap();
        socket.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        let mut head = vec![];
        let mut byte = [0];
        while !head.ends_with(b"\r\n\r\n") && socket.read(&mut byte).unwrap() == 1 {
            head.push(byte[0]);
        }
        assert!(String::from_utf8(head).unwrap().starts_with("GET /ws"));
        socket.write_all(b"HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
    });
    let client = Client::connected(&base);
    let id = client.subscribe("orchestration.subscribeShell", json!({}));
    let end = client.poll_until(&id, |_| false).pop().unwrap();
    assert_eq!(end["error"], "rpc_connection_failed", "{end}");
    assert!(end["errorMessage"].as_str().unwrap().contains("401"), "{end}");
    server.join().unwrap();
    // A stream the server ends normally has a code and no reason.
    let (base, server) = serve(|_, ws| send(ws, json!({"_tag":"Exit","requestId":"1","exit":{"_tag":"Success"}})));
    let client = Client::connected(&base);
    let id = client.subscribe("orchestration.subscribeShell", json!({}));
    let end = client.poll_until(&id, |_| false).pop().unwrap();
    assert_eq!((end["error"].clone(), end["errorMessage"].clone()), (json!("stream_closed"), Value::Null));
    server.join().unwrap();
}

#[test]
fn capabilities_list_the_stream_ops_and_limits_without_a_new_abi_version() {
    let client = Client(t3_client_new() as usize);
    let c = client.call(json!({"op":"capabilities"}))["data"].clone();
    assert_eq!(c["abiVersion"], 3);
    for op in ["subscribe", "pollSubscription", "cancelSubscription", "wakeSubscriptions"] {
        assert!(c["operations"].as_array().unwrap().contains(&json!(op)), "{op}");
    }
    assert_eq!(c["maxSubscriptions"], 8);
    assert_eq!(c["maxLargeSubscriptions"], 2);
    assert_eq!((c["queueMaxItems"].clone(), c["queueMaxBytes"].clone(), c["itemMaxBytes"].clone()), (json!(64), json!(8 << 20), json!(8 << 20)));
    assert_eq!((c["largeItemMaxBytes"].clone(), c["largeQueueMaxBytes"].clone()), (json!(64 << 20), json!(72 << 20)));
    assert_eq!((c["maxPollMs"].clone(), c["probeTimeoutMs"].clone(), c["idleTimeoutMs"].clone()), (json!(1000), json!(3000), json!(20000)));
    assert_eq!(c["pollFields"], json!(["items", "boundaries", "idle", "completionMarker", "closed", "error", "errorMessage"]));
    // Wake needs no connection and probes nothing on an empty handle.
    assert_eq!(client.call(json!({"op":"wakeSubscriptions"})), json!({"ok":true,"data":{"probed":0}}));
}
