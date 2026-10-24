//! A fake T3 WebSocket server for the typed API tests. Each step serves one
//! connection: it reads the first `Request`, records it, and answers with the
//! step's reply. After the steps it keeps listening briefly and counts any
//! further connection, so a test can assert "one request, zero retries" and
//! "no socket at all" for locally rejected input.
#![allow(dead_code)]
use serde_json::{json, Value};
use std::{
    io::ErrorKind,
    net::TcpListener,
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use t3_protocol::{Connection, Credential};

pub enum Reply {
    /// `Exit` with `Success` and this value.
    Ok(Value),
    /// `Exit` with `Failure` and this cause.
    Fail(Value),
    /// Read the request, then close the socket without answering.
    DropAfterRequest,
    /// One `Chunk` per inner list, then a `Success` exit with `null`.
    Chunks(Vec<Vec<Value>>),
    /// Read the request and wait (up to 30 s) until the client goes away.
    Hang,
}
pub struct Recorded {
    /// Every `Request` received, in order (`{"tag","payload"}` and the rest of the frame).
    pub requests: Vec<Value>,
    /// Every accepted TCP connection, including ones that sent no request.
    pub connections: usize,
}
pub struct FakeT3 {
    pub base: String,
    server: JoinHandle<Recorded>,
}
impl FakeT3 {
    pub fn spawn(steps: Vec<Reply>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let server = thread::spawn(move || {
            let mut recorded = Recorded { requests: vec![], connections: 0 };
            for reply in steps {
                let (socket, _) = listener.accept().unwrap();
                recorded.connections += 1;
                socket.set_read_timeout(Some(Duration::from_secs(30))).unwrap();
                let mut ws = tungstenite::accept(socket).unwrap();
                let request: Value = serde_json::from_str(ws.read().unwrap().to_text().unwrap()).unwrap();
                assert_eq!(request["_tag"], "Request");
                recorded.requests.push(request);
                let exit = |exit: Value| tungstenite::Message::Text(json!({"_tag":"Exit","requestId":"1","exit":exit}).to_string().into());
                match reply {
                    Reply::Ok(value) => ws.send(exit(json!({"_tag":"Success","value":value}))).unwrap(),
                    Reply::Fail(cause) => ws.send(exit(json!({"_tag":"Failure","cause":cause}))).unwrap(),
                    Reply::DropAfterRequest => {
                        drop(ws);
                        continue;
                    }
                    Reply::Chunks(chunks) => {
                        for values in chunks {
                            ws.send(tungstenite::Message::Text(json!({"_tag":"Chunk","requestId":"1","values":values}).to_string().into())).unwrap();
                        }
                        ws.send(exit(json!({"_tag":"Success","value":null}))).unwrap();
                    }
                    Reply::Hang => {}
                }
                while ws.read().is_ok() {}
            }
            // Count retries or unexpected sockets for a short grace period.
            listener.set_nonblocking(true).unwrap();
            let until = Instant::now() + Duration::from_millis(300);
            while Instant::now() < until {
                match listener.accept() {
                    Ok(_) => recorded.connections += 1,
                    Err(e) if e.kind() == ErrorKind::WouldBlock => thread::sleep(Duration::from_millis(10)),
                    Err(e) => panic!("{e}"),
                }
            }
            recorded
        });
        Self { base, server }
    }
    pub fn connection(&self) -> Connection {
        Connection::new(&self.base, None, Credential::bearer("synthetic-token").unwrap()).unwrap()
    }
    pub fn finish(self) -> Recorded {
        self.server.join().unwrap()
    }
}
/// `[{"tag","payload"}]` of the recorded requests, for exact comparisons.
pub fn wire(recorded: &Recorded) -> Vec<Value> {
    recorded.requests.iter().map(|r| json!({"tag":r["tag"],"payload":r["payload"]})).collect()
}
/// An Effect cause with one typed failure, as T3 encodes it.
pub fn fail(error: Value) -> Value {
    json!([{"_tag":"Fail","error":error}])
}
