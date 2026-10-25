//! The review screens through `domain`: the diff, the tool inspector, plans and
//! checkpoint rewind. Each function's output shape, and that `domain` answers
//! exactly what the module does.
use serde_json::{json, Value};
use t3_protocol::domain::call;

fn keys(v: &Value) -> Vec<&str> {
    let mut k = v.as_object().unwrap().keys().map(String::as_str).collect::<Vec<_>>();
    k.sort();
    k
}

const PATCH: &str = "diff --git a/src/a.rs b/src/a.rs\n--- a/src/a.rs\n+++ b/src/a.rs\n@@ -1,2 +1,2 @@\n keep\n-old\n+new\n";

#[test]
fn diff_review_present_has_files_rows_and_split_rows() {
    let result = json!({"diff":PATCH,"fromTurnCount":1,"toTurnCount":2});
    let summary = json!({"files":[{"path":"src/a.rs","kind":"modified","additions":1,"deletions":1},{"path":"gone.txt","kind":"deleted","additions":0,"deletions":3}]});
    let view = call("diffReview.present", &json!({"result":result,"summary":summary})).unwrap();
    assert_eq!(view, t3_protocol::diff_review::present(&result, &summary).unwrap());
    assert_eq!(keys(&view), ["additions", "deletions", "files", "fromTurnCount", "toTurnCount"]);
    let file = &view["files"][0];
    assert_eq!(keys(file), ["additions", "binary", "deletions", "kind", "oldPath", "path", "rows", "splitRows", "summaryAdditions", "summaryDeletions"]);
    let kinds = file["rows"].as_array().unwrap().iter().map(|r| r["kind"].as_str().unwrap()).collect::<Vec<_>>();
    // The ---/+++ headers are meta rows before the first hunk.
    assert_eq!(kinds, ["meta", "meta", "hunk", "context", "deletion", "addition"]);
    assert_eq!(keys(&file["rows"][3]), ["kind", "newLine", "oldLine", "text"]);
    assert_eq!((file["rows"][3]["oldLine"].as_u64(), file["rows"][3]["newLine"].as_u64()), (Some(1), Some(1)));
    assert_eq!((file["rows"][4]["oldLine"].as_u64(), file["rows"][5]["newLine"].as_u64()), (Some(2), Some(2)));
    assert_eq!(file["splitRows"][4], json!({"kind":"change","left":file["rows"][4],"right":file["rows"][5]}));
    // A summary file without a patch stays listed, as unavailable.
    assert_eq!((view["files"][1]["missing"].as_bool(), view["files"][1]["rows"].as_array().map(Vec::len)), (Some(true), Some(0)));
    assert_eq!((view["additions"].as_u64(), view["deletions"].as_u64()), (Some(1), Some(4)));
    // The summary is optional; the module's limits come back as errors.
    assert_eq!(call("diffReview.present", &json!({"result":result})).unwrap()["files"].as_array().unwrap().len(), 1);
    assert_eq!(call("diffReview.present", &json!({"result":{}})).unwrap_err(), "invalid_diff_result");
    assert_eq!(call("diffReview.present", &json!({"result":{"diff":"diff --git nonsense\n"}})).unwrap_err(), "invalid_diff_header");
}

#[test]
fn diff_review_scoped_result_drops_late_receipts_and_keeps_errors() {
    let expected = json!({"requestId":"q1","environment":"e","thread":"t","fromTurnCount":1,"toTurnCount":2});
    let summary = json!({"files":[],"fromTurnCount":1,"toTurnCount":2});
    let receipt = json!({"requestId":"q1","environment":"e","thread":"t","result":{"threadId":"t","diff":PATCH,"fromTurnCount":1,"toTurnCount":2}});
    let view = call("diffReview.scopedResult", &json!({"expected":expected,"receipt":receipt,"summary":summary})).unwrap();
    assert_eq!(view["files"][0]["path"], "src/a.rs");
    let mut late = receipt.clone();
    late["requestId"] = json!("q0");
    assert!(call("diffReview.scopedResult", &json!({"expected":expected,"receipt":late,"summary":summary})).unwrap().is_null());
    let failed = json!({"requestId":"q1","environment":"e","thread":"t","error":"rpc_rejected: gone"});
    assert_eq!(call("diffReview.scopedResult", &json!({"expected":expected,"receipt":failed,"summary":summary})).unwrap_err(), "rpc_rejected: gone");
    let mut other = receipt;
    other["result"]["threadId"] = json!("u");
    assert_eq!(call("diffReview.scopedResult", &json!({"expected":expected,"receipt":other,"summary":summary})).unwrap_err(), "invalid_diff_thread");
}

fn command_item(status: &str) -> Value {
    json!({"id":"item:1","threadId":"thread:source","runId":"run:1","nodeId":null,"providerThreadId":"p","providerTurnId":"turn:1",
        "nativeItemRef":null,"parentItemId":null,"ordinal":3,"status":status,"title":null,"startedAt":"2026-10-17T12:00:00.000Z",
        "completedAt":"2026-10-17T12:00:01.000Z","updatedAt":"2026-10-17T12:00:01.000Z","type":"command_execution",
        "input":"printf fixture","output":"fixture\n","exitCode":1})
}

#[test]
fn tool_inspector_reference_request_and_parse() {
    let item = command_item("failed");
    let reference = call("toolInspector.compactReference", &json!({"row":{"sourceThreadId":"thread:source","sourceItemId":"item:1","item":item}})).unwrap();
    assert_eq!(keys(&reference), ["providerTurnId", "revision", "runId", "sourceItemId", "sourceThreadId", "type"]);
    assert!(!reference.to_string().contains("printf"));
    assert_eq!(call("toolInspector.compactReference", &json!({"row":command_item("running")})).unwrap()["revision"], "live");
    let viewer = json!({"reference":reference,"environment":"laptop","threadId":"thread:viewer"});
    let scope = call("toolInspector.scope", &viewer).unwrap();
    assert_eq!((scope["environment"].as_str(), scope["threadId"].as_str(), scope["sourceThreadId"].as_str()), (Some("laptop"), Some("thread:viewer"), Some("thread:source")));
    let request = call("toolInspector.request", &viewer).unwrap();
    assert_eq!(request, json!({"method":"orchestration.getTurnItem","payload":{"threadId":"thread:source","itemId":"item:1","revision":"2026-10-17T12:00:01.000Z"}}));
    let typed = t3_protocol::api::function("orchestration.getTurnItem").unwrap();
    assert_eq!(typed.payload(&request["payload"]).unwrap(), request["payload"]);

    let mut args = viewer.clone();
    args["response"] = json!({"item":item});
    let view = call("toolInspector.parse", &args).unwrap();
    assert_eq!(keys(&view), ["available", "emptyOutput", "exitCode", "failed", "metadata", "metadataTruncated", "outputOmitted", "raw", "rawTruncated", "scope", "sections", "status", "title"]);
    assert_eq!((view["available"].as_bool(), view["failed"].as_bool(), view["exitCode"].as_i64()), (Some(true), Some(true), Some(1)));
    let sections = view["sections"].as_array().unwrap().iter().map(|s| s["key"].as_str().unwrap()).collect::<Vec<_>>();
    assert_eq!(sections, ["input", "output"]);
    assert_eq!(keys(&view["sections"][0]), ["key", "label", "text", "truncated"]);
    args["response"] = json!({"item":null});
    assert_eq!(call("toolInspector.parse", &args).unwrap()["available"], false);
    args["response"] = json!({"item":{"id":"other"}});
    assert_eq!(call("toolInspector.parse", &args).unwrap_err(), "tool_scope_mismatch");
    let mut bad = viewer;
    bad["reference"]["revision"] = json!("");
    assert_eq!(call("toolInspector.request", &bad).unwrap_err(), "invalid_tool_reference");
}
