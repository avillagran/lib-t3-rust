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
