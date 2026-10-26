//! `{"op":"domain"}` through the C entry point: offline, listed in
//! `capabilities` and `describe`, validated like `call`, and every
//! registered function reachable by name.
use serde_json::{json, Value};
use std::ffi::{c_void, CStr};
use t3_protocol::ffi::*;

struct Client(*mut c_void);
impl Client {
    fn new() -> Self {
        Self(t3_client_new())
    }
    fn call(&self, request: Value) -> Value {
        let bytes = serde_json::to_vec(&request).unwrap();
        unsafe {
            let raw = t3_client_call(self.0, bytes.as_ptr(), bytes.len());
            let value = serde_json::from_slice(CStr::from_ptr(raw).to_bytes()).unwrap();
            t3_string_free(raw);
            value
        }
    }
    fn domain(&self, name: &str, args: Value) -> Value {
        self.call(json!({"op":"domain","fn":name,"args":args}))
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        unsafe { t3_client_free(self.0) }
    }
}

#[test]
fn capabilities_and_describe_list_the_domain_functions() {
    let c = Client::new();
    let caps = &c.call(json!({"op":"capabilities"}))["data"];
    assert!(caps["operations"].as_array().unwrap().iter().any(|o| o == "domain"));
    assert_eq!(caps["domainFunctions"], t3_protocol::domain::functions().count());
    let described = &c.call(json!({"op":"describe"}))["data"]["domain"];
    let names = described.as_array().unwrap().iter().map(|f| f["fn"].as_str().unwrap()).collect::<Vec<_>>();
    assert_eq!(names, t3_protocol::domain::functions().map(|f| f.name).collect::<Vec<_>>());
    let derive = described.as_array().unwrap().iter().find(|f| f["fn"] == "queue.derive").unwrap();
    assert_eq!(derive["args"], json!([{"name":"projection","type":"object","required":true}]));
    assert!(derive["returns"].as_str().unwrap().starts_with("object"));
}

#[test]
fn domain_answers_without_a_connection_and_keeps_the_error_envelope() {
    let c = Client::new();
    let r = c.domain("codeBlocks.parse", json!({"markdown":"```bash\necho hi\n```"}));
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["data"][0]["command"], "echo hi");
    assert_eq!(c.call(json!({"op":"domain","fn":"i18n.normalize","args":{"locale":"fr_CA.UTF-8"}}))["data"], "fr-ca");
    assert_eq!(c.call(json!({"op":"domain","fn":"queue.nope"})), json!({"ok":false,"error":"unknown_function"}));
    assert_eq!(c.call(json!({"op":"domain"})), json!({"ok":false,"error":"invalid_input"}));
    assert_eq!(c.domain("queue.derive", json!({})), json!({"ok":false,"error":"invalid_projection"}));
    assert_eq!(c.domain("queue.derive", json!({"projection":{},"extra":1})), json!({"ok":false,"error":"invalid_input"}));
    assert_eq!(c.domain("models.runtimeModeCommand", json!({"threadId":"t","commandId":"c","runtimeMode":"yolo"}))["error"], "invalid_runtime_mode");
    // A function's own rule still applies after the type check.
    assert_eq!(c.domain("history.path", json!({"threadId":"t","cursor":"bad\ncursor"}))["error"], "invalid_history_cursor");
}

#[test]
fn existing_pure_modules_answer_through_domain() {
    let c = Client::new();
    let event = json!({"type":"run.updated","payload":{"id":"r","status":"running"}});
    let applied = &c.domain("projection.apply", json!({"projection":{"thread":{"id":"t"}},"events":[event,{"type":"future.event","payload":{}}]}))["data"];
    assert_eq!(applied["changed"], true);
    assert_eq!(applied["applied"], 1);
    assert_eq!(applied["projection"]["thread"]["activeRunId"], "r");
    let thread = json!({"latestRunId":"r","status":"failed","lastErrorClass":"usage_limit"});
    assert_eq!(c.domain("sidebar.status", json!({"thread":thread}))["data"], "limited");
    assert_eq!(c.domain("sidebar.metadata", json!({"thread":thread,"now":0}))["data"]["sidebarStatus"], "limited");
    assert_eq!(c.domain("sidebar.sortTimestamps", json!({"thread":{"createdAt":10,"latestRunCompletedAt":1000}}))["data"]["inboxAt"], 1000);
    assert_eq!(c.domain("sidebar.command", json!({"action":"unsnooze","threadId":"t"}))["data"], json!({"type":"thread.unsnooze","threadId":"t","reason":"user"}));
    assert_eq!(c.domain("sidebar.threadAction", json!({"thread":"t","action":"delete-thread"}))["error"], "confirmation_required");
    let projection = json!({"thread":{"id":"t"},"runtimeRequests":[{"id":"q","kind":"user_input","status":"pending","responseCapability":{"type":"live"}},{"id":"a","kind":"command","status":"pending","responseCapability":{"type":"live"}}],"turnItems":[{"type":"user_input_request","requestId":"q","questions":[{"id":"x"}]}]});
    assert_eq!(c.domain("questions.pending", json!({"projection":projection}))["data"][0]["requestId"], "q");
    assert_eq!(c.domain("approvals.pending", json!({"projection":projection}))["data"][0]["requestId"], "a");
    assert_eq!(c.domain("approvals.command", json!({"thread":"t","runtimeRequestId":"a","decision":"accept","projection":projection}))["data"]["type"], "runtime-request.respond");
    assert_eq!(c.domain("questions.command", json!({"thread":"t","runtimeRequestId":"q","action":"dismiss"}))["data"]["type"], "thread.user-input.dismiss");
    assert_eq!(c.domain("queue.command", json!({"thread":"t","action":"cancel","runId":"r"}))["data"], json!({"type":"queued-run.cancel","runId":"r","threadId":"t"}));
    assert_eq!(c.domain("models.preferences", json!({"settings":{"favorites":[]}}))["data"]["favoritesReady"], true);
    assert_eq!(c.domain("models.favoritePayload", json!({"provider":"p","model":"m","favorite":true}))["data"]["patch"]["setModelFavorites"][0]["model"], "m");
    assert_eq!(c.domain("models.catalog", json!({"config":{"providers":[]}}))["data"]["models"], json!([]));
    assert_eq!(c.domain("composer.fileLink", json!({"path":"docs/a b.md"}))["data"], "[a b.md](docs/a%20b.md)");
    assert_eq!(c.domain("composer.paths", json!({"result":{"entries":[{"path":"src","kind":"directory"}]}}))["data"][0]["icon"], "folder-plus");
    assert_eq!(c.domain("composer.catalog", json!({"provider":{"slashCommands":[{"name":"compact"}]},"cwd":"/r","kind":"command","query":"comp"}))["data"][0]["insert"], "/compact ");
    assert_eq!(c.domain("composer.attachmentReference", json!({"attachment":{"type":"file","id":"f","name":"a.txt"}}))["data"], "[a.txt](t3-context://v1/file/f)");
    assert_eq!(c.domain("history.path", json!({"threadId":"t","cursor":"c+1"}))["data"], "/api/orchestration/threads/t/history?cursor=c%2B1&compactIds=1");
    assert_eq!(c.domain("history.envelope", json!({"page":{"items":[],"nextCursor":null,"hasMoreHistory":false},"threadId":"t"}))["data"]["hasMoreHistory"], false);
    assert_eq!(c.domain("i18n.translate", json!({"catalog":{"fr":{"Close":"Fermer"}},"locale":"fr_CA","source":"Close"}))["data"], "Fermer");
}

#[test]
fn domain_alone_accepts_thread_sized_input() {
    let c = Client::new();
    let big = "x".repeat(9 * 1024 * 1024);
    let r = c.domain("codeBlocks.parse", json!({"markdown":big}));
    assert_eq!((r["ok"].as_bool(), r["data"].as_array().map(Vec::len)), (Some(true), Some(0)), "a 9 MiB projection-sized input is accepted");
    assert_eq!(c.call(json!({"op":"call","fn":"vcs.refreshStatus","args":{"cwd":big}}))["error"], "invalid_input", "other operations keep 8 MiB");
    assert_eq!(t3_protocol::ffi::DOMAIN_MAX_BYTES, 72 * 1024 * 1024);
}
