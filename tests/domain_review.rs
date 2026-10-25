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

fn plan_projection() -> Value {
    json!({"thread":{"id":"thread-1","interactionMode":"plan"},"updatedAt":"2026-10-17T10:00:00Z",
      "runs":[{"id":"run-1","ordinal":1,"status":"completed"}],
      "turnItems":[{"item":{"type":"proposed_plan","planId":"plan-1"},"updatedAt":"2026-10-17T10:01:00Z"}],
      "messages":[],
      "plans":[{"id":"plan-1","threadId":"thread-1","runId":"run-1","nodeId":"n","status":"active","kind":"proposed_plan","markdown":"# Native Plan\n\n## Summary\nShip **it**\n\n## Steps\n1. Two"}]})
}

#[test]
fn plan_workflow_view_follow_up_and_titles() {
    let p = plan_projection();
    let view = call("planWorkflow.view", &json!({"projection":p})).unwrap();
    assert_eq!(keys(&view), ["plans", "ready"]);
    let plan = &view["plans"][0];
    assert_eq!(keys(plan), ["blocks", "displayMarkdown", "id", "markdown", "runId", "status", "threadId", "title", "updatedAt"]);
    assert_eq!((plan["title"].as_str(), plan["displayMarkdown"].as_str()), (Some("Native Plan"), Some("Ship **it**\n\n## Steps\n1. Two")));
    assert_eq!(plan["blocks"], json!(t3_protocol::markdown::blocks("Ship **it**\n\n## Steps\n1. Two", false)));
    assert_eq!(view["ready"]["id"], "plan-1");
    // The same plan view timeline.view returns, blocks and all.
    let timeline = call("timeline.view", &json!({"projection":p})).unwrap();
    assert_eq!(view, timeline["planView"]);
    assert!(call("planWorkflow.view", &json!({"projection":{"thread":{"id":"t"}}})).unwrap()["ready"].is_null());

    let implement = call("planWorkflow.followUp", &json!({"plan":view["ready"],"threadId":"thread-1"})).unwrap();
    assert_eq!(implement["interactionMode"], "default");
    assert_eq!(implement["sourcePlanRef"], json!({"threadId":"thread-1","planId":"plan-1"}));
    assert!(implement["text"].as_str().unwrap().starts_with("PLEASE IMPLEMENT THIS PLAN:\n# Native Plan"));
    let feedback = call("planWorkflow.followUp", &json!({"plan":view["ready"],"threadId":"thread-1","draft":" Shorter "})).unwrap();
    assert_eq!(feedback, json!({"text":"Shorter","interactionMode":"plan"}));
    assert_eq!(call("planWorkflow.followUp", &json!({"plan":view["ready"],"threadId":"thread-2"})).unwrap_err(), "invalid_plan_scope");

    let command = json!({"type":"message.dispatch","threadId":"thread-9","text":implement["text"]});
    let attached = call("planWorkflow.attachSourceRef", &json!({"command":command,"followUp":implement})).unwrap();
    assert_eq!(attached["sourcePlanRef"], json!({"threadId":"thread-1","planId":"plan-1"}));
    assert_eq!(call("planWorkflow.attachSourceRef", &json!({"command":{"type":"thread.create"},"followUp":implement})).unwrap_err(), "invalid_plan_command");
    assert_eq!(call("planWorkflow.implementationTitle", &json!({"markdown":plan["markdown"]})).unwrap(), "Implement Native Plan");
    assert_eq!(call("planWorkflow.implementationTitle", &json!({"markdown":"no heading"})).unwrap(), "Implement plan");
    assert_eq!(call("planWorkflow.mode", &json!({"planModeEnabled":true,"provider":{"showInteractionModeToggle":true},"requested":"plan"})).unwrap(), json!({"enabled":true,"interactionMode":"plan"}));
    assert_eq!(call("planWorkflow.mode", &json!({"planModeEnabled":true,"requested":"plan"})).unwrap()["interactionMode"], "default");
}

fn revert_projection() -> Value {
    json!({
        "thread":{"id":"thread:one","activeProviderThreadId":"provider:one","modelSelection":{"instanceId":"codex"},"worktreePath":"/w"},
        "providerThreads":[{"id":"provider:one","providerInstanceId":"codex","providerSessionId":"session:one"}],
        "checkpointScopes":[{"id":"scope:one","threadId":"thread:one","cwd":"/w","providerThreadId":"provider:one"}],
        "checkpoints":[
            {"id":"checkpoint:initial","threadId":"thread:one","scopeId":"scope:one","runId":null,"ordinalWithinScope":0,"appRunOrdinal":null,"status":"ready"},
            {"id":"checkpoint:1","threadId":"thread:one","scopeId":"scope:one","runId":"run:one","ordinalWithinScope":1,"appRunOrdinal":1,"status":"ready"},
            {"id":"checkpoint:2","threadId":"thread:one","scopeId":"scope:one","runId":"run:two","ordinalWithinScope":2,"appRunOrdinal":2,"status":"ready"}],
        "runs":[{"id":"run:one","ordinal":1,"status":"completed"},{"id":"run:two","ordinal":2,"status":"completed"}],
        "turnItems":[{"item":{"type":"user_message","messageId":"message:one","runId":"run:one","inputIntent":"turn_start"}},{"item":{"type":"user_message","messageId":"message:two","runId":"run:two","inputIntent":"queued_turn"}}],
        "messages":[
            {"id":"message:one","role":"user","runId":"run:one","text":"first","attachments":[],"context":[]},
            {"id":"message:two","role":"user","runId":"run:two","text":"second","attachments":[],"context":[]}]
    })
}

#[test]
fn checkpoint_revert_reference_safety_command_and_observation() {
    let p = revert_projection();
    assert_eq!(call("checkpointRevert.canPreview", &json!({"projection":p,"message":p["messages"][1]})).unwrap(), true);
    assert_eq!(call("checkpointRevert.canPreview", &json!({"projection":p,"message":{"role":"assistant","runId":"run:two"}})).unwrap(), false);
    let reference = call("checkpointRevert.reference", &json!({"projection":p,"messageId":"message:two"})).unwrap();
    assert_eq!(keys(&reference), ["checkpointId", "checkpointRunId", "messageCheckpointId", "messageId", "messageRunId", "priorRollbackRequestId", "providerInstanceId", "providerThreadId", "scopeId", "threadId", "turnCount"]);
    assert_eq!((reference["checkpointId"].as_str(), reference["turnCount"].as_u64()), (Some("checkpoint:1"), Some(1)));
    assert_eq!(call("checkpointRevert.reference", &json!({"projection":p,"messageId":"message:none"})).unwrap_err(), "revert_message_unavailable");

    let open = json!({"canOperate":true,"connected":true,"supportsConversationRollback":true});
    let gate = call("checkpointRevert.safety", &json!({"safety":open})).unwrap();
    assert_eq!(keys(&gate), ["allowed", "error", "safety"]);
    assert_eq!((gate["allowed"].as_bool(), gate["error"].is_null()), (Some(true), true));
    assert_eq!(gate["safety"], json!({"canOperate":true,"connected":true,"supportsConversationRollback":true,"working":false,"sendBusy":false,"connecting":false,"pendingAttachments":false,"rewinding":false}));
    let mut busy = open.clone();
    busy["working"] = json!(true);
    assert_eq!(call("checkpointRevert.safety", &json!({"safety":busy})).unwrap()["error"], "revert_interrupt_first");
    // Unknown is false: a missing or non-boolean capability keeps the gate shut.
    assert_eq!(call("checkpointRevert.safety", &json!({"safety":{"canOperate":true,"connected":"yes","supportsConversationRollback":true}})).unwrap()["error"], "revert_disconnected");

    let input = json!({"requestId":"b3e86a43-744c-46ec-b5f8-65ae79316907","environment":"local","thread":"thread:one","confirmed":true,"restoreFiles":true,"reference":reference});
    let command = call("checkpointRevert.command", &json!({"input":input,"projection":p,"safety":open})).unwrap();
    assert_eq!(command, json!({"type":"checkpoint.rollback","commandId":"omarchy-b3e86a43-744c-46ec-b5f8-65ae79316907","threadId":"thread:one","scopeId":"scope:one","checkpointId":"checkpoint:1","restoreFiles":true}));
    assert_eq!(call("checkpointRevert.confirmedCommand", &json!({"input":input})).unwrap(), command);
    assert_eq!(call("checkpointRevert.command", &json!({"input":input,"projection":p,"safety":busy})).unwrap_err(), "revert_interrupt_first");
    let mut no_worktree = p.clone();
    no_worktree["thread"]["worktreePath"] = Value::Null;
    assert_eq!(call("checkpointRevert.command", &json!({"input":input,"projection":no_worktree,"safety":open})).unwrap_err(), "revert_files_require_worktree");

    let recovered = call("checkpointRevert.recoveryMessage", &json!({"projection":p,"reference":reference})).unwrap();
    assert_eq!(recovered, json!({"id":"message:two","runId":"run:two","text":"second","attachments":[],"context":[]}));
    assert_eq!(call("checkpointRevert.observation", &json!({"input":input,"projection":p})).unwrap(), json!({"state":"pending"}));
    let mut done = p.clone();
    done["thread"]["rollbackRequestId"] = command["commandId"].clone();
    done["runs"][1]["status"] = json!("rolled_back");
    assert_eq!(call("checkpointRevert.observation", &json!({"input":input,"projection":done})).unwrap(), json!({"state":"complete"}));
    assert_eq!(call("checkpointRevert.appendPrompt", &json!({"current":"draft","restored":"second"})).unwrap(), "draft\n\nsecond");
}
