//! Bounded foreign-host event inbox. No host callbacks or unbounded worker queues.
use crate::{error_code, stream, Connection, Result, MAX_BYTES};
use serde_json::{json, Value};
use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Condvar, Mutex,
    },
    time::Duration,
};
pub(crate) const MAX_ITEMS: usize = 64;
/// Per-subscription bounds, fixed by its mode when it starts. Sizes are serialized JSON.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Limits {
    /// Largest single item: the same limit the WebSocket frame was read with.
    pub(crate) item: usize,
    /// Every queued, not yet polled item together.
    pub(crate) queue: usize,
}
impl Limits {
    pub(crate) const NORMAL: Self = Self { item: MAX_BYTES as usize, queue: MAX_BYTES as usize };
    /// 8 MiB everywhere except `orchestration.subscribeThread` with `acceptBoundedSnapshot`:
    /// that one item may be a 64 MiB snapshot, and the queue keeps a normal stream's
    /// 8 MiB of events behind it (72 MiB).
    pub(crate) fn of(tag: &str, payload: &Value) -> Self {
        let item = stream::item_limit(tag, payload) as usize;
        if item > Self::NORMAL.item {
            Self { item, queue: item + Self::NORMAL.queue }
        } else {
            Self::NORMAL
        }
    }
}
impl Default for Limits {
    fn default() -> Self {
        Self::NORMAL
    }
}
/// Counts serialized bytes without building a copy of a (possibly 64 MiB) item.
struct Counter(usize);
impl std::io::Write for Counter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 += bytes.len();
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
#[derive(Default)]
struct Inbox {
    items: VecDeque<(Value, usize)>,
    bytes: usize,
    closed: bool,
    error: Option<String>,
}
#[derive(Default)]
struct Shared {
    inbox: Mutex<Inbox>,
    ready: Condvar,
    limits: Limits,
}
impl Shared {
    fn new(limits: Limits) -> Self {
        Self { limits, ..Self::default() }
    }
    fn push(&self, item: Value) -> Result<()> {
        let mut size = Counter(0);
        serde_json::to_writer(&mut size, &item).map_err(|_| "malformed_stream")?;
        let size = size.0;
        let mut inbox = self.inbox.lock().map_err(|_| "internal_error")?;
        if inbox.closed {
            return Err("stream_cancelled".into());
        }
        if inbox.items.len() >= MAX_ITEMS
            || size > self.limits.item
            || size > self.limits.queue - inbox.bytes
        {
            return Err("stream_backpressure".into());
        }
        inbox.bytes += size;
        inbox.items.push_back((item, size));
        self.ready.notify_all();
        Ok(())
    }
    fn finish(&self, error: Option<String>) {
        if let Ok(mut inbox) = self.inbox.lock() {
            inbox.closed = true;
            inbox.error = error;
            self.ready.notify_all();
        }
    }
}
pub(crate) struct Subscription {
    shared: Arc<Shared>,
    cancel: Arc<AtomicBool>,
}
impl Subscription {
    pub(crate) fn start(connection: Connection, tag: String, payload: Value) -> Result<Self> {
        let shared = Arc::new(Shared::new(Limits::of(&tag, &payload)));
        let cancel = Arc::new(AtomicBool::new(false));
        let (queue, stop) = (shared.clone(), cancel.clone());
        std::thread::Builder::new()
            .name("t3-events".into())
            .spawn(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    connection.subscribe(
                        &tag,
                        payload,
                        || stop.load(Ordering::Acquire),
                        |item| {
                            if let Some(value) = item {
                                queue.push(value)?;
                            }
                            Ok(())
                        },
                    )
                }))
                .unwrap_or_else(|_| Err("internal_error".into()));
                queue.finish(if stop.load(Ordering::Acquire) {
                    None
                } else {
                    result.err()
                });
            })
            .map_err(|_| "local_error")?;
        Ok(Self { shared, cancel })
    }

    pub(crate) fn poll(&self, timeout_ms: u64) -> Result<Value> {
        let inbox = self.shared.inbox.lock().map_err(|_| "internal_error")?;
        let (mut inbox, _) = self
            .shared
            .ready
            .wait_timeout_while(inbox, Duration::from_millis(timeout_ms), |v| {
                v.items.is_empty() && !v.closed
            })
            .map_err(|_| "internal_error")?;
        let items: Vec<_> = inbox.items.drain(..).map(|(v, _)| v).collect();
        inbox.bytes = 0;
        // `error` stays a stable code; a reason after it goes to `errorMessage`.
        let code = inbox.error.as_deref().map(error_code);
        let message = inbox.error.as_deref().zip(code).and_then(|(e, c)| e.get(c.len() + 2..));
        Ok(json!({"items":items,"closed":inbox.closed,"error":code,"errorMessage":message}))
    }
    pub(crate) fn cancel(&self) {
        self.cancel.store(true, Ordering::Release);
        if let Ok(mut inbox) = self.shared.inbox.lock() {
            inbox.items.clear();
            inbox.bytes = 0;
            inbox.closed = true;
            inbox.error = None;
            self.shared.ready.notify_all();
        }
    }
}
impl Drop for Subscription {
    fn drop(&mut self) {
        self.cancel();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounds_never_silently_drop_events() {
        let shared = Shared::default();
        for n in 0..MAX_ITEMS {
            shared.push(json!({"sequence":n})).unwrap();
        }
        assert_eq!(
            shared.push(json!({"sequence":64})).unwrap_err(),
            "stream_backpressure"
        );
        assert_eq!(shared.inbox.lock().unwrap().items.len(), MAX_ITEMS);
        let shared = Shared::default();
        assert_eq!(
            shared
                .push(json!("a".repeat(MAX_BYTES as usize)))
                .unwrap_err(),
            "stream_backpressure"
        );
        assert!(shared.inbox.lock().unwrap().items.is_empty());
    }
    #[test]
    fn only_a_bounded_thread_subscription_gets_the_large_item_limit() {
        let bounded = json!({"threadId":"t","acceptBoundedSnapshot":true});
        let large = Limits::of("orchestration.subscribeThread", &bounded);
        assert_eq!(large, Limits { item: 64 << 20, queue: 72 << 20 });
        for (tag, payload) in [
            ("orchestration.subscribeThread", json!({"threadId":"t"})),
            ("orchestration.subscribeThread", json!({"acceptBoundedSnapshot":"true"})),
            ("orchestration.subscribeShell", json!({"acceptBoundedSnapshot":true})),
        ] {
            assert_eq!(Limits::of(tag, &payload), Limits::NORMAL, "{tag} {payload}");
        }
        // A large item fits once; the queue then still holds 8 MiB of events behind it.
        let shared = Shared::new(large);
        shared.push(json!("a".repeat(63 << 20))).unwrap();
        shared.push(json!("b".repeat(7 << 20))).unwrap();
        assert_eq!(shared.push(json!("c".repeat(3 << 20))).unwrap_err(), "stream_backpressure");
        assert_eq!(shared.inbox.lock().unwrap().items.len(), 2);
        let shared = Shared::new(large);
        assert_eq!(shared.push(json!("a".repeat(64 << 20))).unwrap_err(), "stream_backpressure");
    }
    #[test]
    fn cancel_wakes_a_waiter_without_a_network_tick() {
        let s = Arc::new(Subscription {
            shared: Arc::default(),
            cancel: Arc::default(),
        });
        let waiting = s.clone();
        let worker = std::thread::spawn(move || waiting.poll(1000).unwrap());
        s.cancel();
        let value = worker.join().unwrap();
        assert_eq!(value["closed"], true);
        assert!(value["items"].as_array().unwrap().is_empty());
    }
}
