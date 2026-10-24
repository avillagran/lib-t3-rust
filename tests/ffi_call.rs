//! ABI 3 through the C entry point: capabilities, `describe`, and `call`
//! (happy path, unknown function, invalid input, local validation, scope).
#[path = "support/fake_t3.rs"]
mod fake_t3;
use fake_t3::{fail, wire, FakeT3, Reply};
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
    fn connect(&self, base: &str) {
        let r = self.call(json!({"op":"connectBearer","httpBaseUrl":base,"token":"synthetic-token"}));
        assert_eq!(r["ok"], true, "{r}");
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        unsafe { t3_client_free(self.0) }
    }
}

#[test]
fn capabilities_announce_abi_3_and_keep_every_earlier_operation() {
    let c = Client::new();
    let caps = &c.call(json!({"op":"capabilities"}))["data"];
    assert_eq!(caps["abiVersion"], 3);
    let ops = caps["operations"].as_array().unwrap();
    for op in ["connectBearer", "connectRelay", "request", "rpc", "rpcWithTimeout", "disconnect", "subscribe", "pollSubscription", "cancelSubscription", "call", "describe"] {
        assert!(ops.iter().any(|o| o == op), "{op}");
    }
    assert_eq!(caps["functions"], t3_protocol::api::functions().count());
    assert_eq!(caps["contract"], t3_protocol::api::CONTRACT);
    assert_eq!(caps["maxSubscriptions"], 8);
}

#[test]
fn describe_needs_no_connection_and_lists_every_function() {
    let c = Client::new();
    let d = c.call(json!({"op":"describe"}));
    assert_eq!(d["ok"], true);
    let functions = d["data"]["functions"].as_array().unwrap();
    assert_eq!(functions.len(), 10);
    let vcs = functions.iter().find(|f| f["fn"] == "vcs.refreshStatus").unwrap();
    assert_eq!(
        vcs,
        &json!({"fn":"vcs.refreshStatus","module":"vcs","tag":"vcs.refreshStatus","kind":"read","replay":"free","scope":"orchestration:read",
            "timeoutMs":20000,"stream":false,"openArgs":false,"args":[{"name":"cwd","type":"path","required":true}]})
    );
    let dispatch = functions.iter().find(|f| f["fn"] == "orchestration.dispatchCommand").unwrap();
    assert_eq!((dispatch["kind"].as_str(), dispatch["replay"].as_str()), (Some("mutation"), Some("id")));
    assert_eq!(dispatch["variants"].as_array().unwrap().len(), 34);
    let settings = functions.iter().find(|f| f["fn"] == "server.updateSettings").unwrap();
    assert_eq!((&settings["scope"], &settings["scopes"]), (&json!("dynamic"), &json!(["settings:write", "providers:manage"])));
}

#[test]
fn call_rejects_locally_before_asking_for_a_connection() {
    let c = Client::new();
    let cases = [
        (json!({"op":"call","fn":"vcs.nope","args":{}}), json!({"ok":false,"error":"unknown_function"})),
        (json!({"op":"call","args":{}}), json!({"ok":false,"error":"invalid_input"})),
        (json!({"op":"call","fn":"vcs.refreshStatus","args":[]}), json!({"ok":false,"error":"invalid_input"})),
        (json!({"op":"call","fn":"vcs.refreshStatus","args":{"cwd":"/r","x":1}}), json!({"ok":false,"error":"invalid_input"})),
        (json!({"op":"call","fn":"vcs.refreshStatus","args":{"cwd":""}}), json!({"ok":false,"error":"invalid_cwd"})),
        (json!({"op":"call","fn":"vcs.refreshStatus","args":{"cwd":"/r"},"timeoutMs":0}), json!({"ok":false,"error":"invalid_rpc_timeout"})),
        // Valid input on a handle with no connection gets that far, and no further.
        (json!({"op":"call","fn":"vcs.refreshStatus","args":{"cwd":"/r"}}), json!({"ok":false,"error":"not_connected"})),
        (json!({"op":"call","fn":"server.getConfig"}), json!({"ok":false,"error":"not_connected"})),
    ];
    for (request, expected) in cases {
        assert_eq!(c.call(request.clone()), expected, "{request}");
    }
}

#[test]
fn call_sends_once_and_carries_scope_and_reason_apart() {
    let fake = FakeT3::spawn(vec![
        Reply::Ok(json!({"isRepo":true,"refName":"main"})),
        Reply::Ok(json!({"sequence":7})),
        Reply::Fail(fail(json!({"_tag":"EnvironmentAuthorizationError","message":"The authenticated token is missing required scope: settings:write.","requiredScope":"settings:write"}))),
        Reply::Fail(fail(json!({"_tag":"GitCommandError","message":"Author identity unknown"}))),
    ]);
    let c = Client::new();
    c.connect(&fake.base);
    assert_eq!(
        c.call(json!({"op":"call","fn":"vcs.refreshStatus","args":{"cwd":"/r"}})),
        json!({"ok":true,"data":{"isRepo":true,"refName":"main"}})
    );
    assert_eq!(
        c.call(json!({"op":"call","fn":"orchestration.dispatchCommand","timeoutMs":20000,"args":{"type":"thread.archive","commandId":"omarchy-1","threadId":"t-1"}})),
        json!({"ok":true,"data":{"sequence":7}})
    );
    assert_eq!(
        c.call(json!({"op":"call","fn":"server.updateSettings","args":{"patch":{"enableProviderUpdateChecks":true}}})),
        json!({"ok":false,"error":"write_not_authorized","message":"The authenticated token is missing required scope: settings:write.","requiredScope":"settings:write"})
    );
    assert_eq!(
        c.call(json!({"op":"call","fn":"vcs.refreshStatus","args":{"cwd":"/r"}})),
        json!({"ok":false,"error":"rpc_rejected","message":"Author identity unknown"})
    );
    let recorded = fake.finish();
    assert_eq!(recorded.connections, 4);
    assert_eq!(wire(&recorded)[1], json!({"tag":"orchestration.dispatchCommand","payload":{"type":"thread.archive","commandId":"omarchy-1","threadId":"t-1"}}));
}
