//! `activity.describe` and `changes.summaries` through `domain`: the shapes
//! the activity blocks and the changed-files card read.
use serde_json::{json, Value};
use t3_protocol::domain::call;

fn keys(v: &Value) -> Vec<&str> {
    let mut k = v.as_object().unwrap().keys().map(String::as_str).collect::<Vec<_>>();
    k.sort();
    k
}

#[test]
fn activity_describe_has_one_shape_and_hides_payloads() {
    let row = json!({"sourceThreadId":"t","sourceItemId":"i","item":{"id":"i","type":"command_execution","runId":"r","status":"running","input":"bash -lc 'sed -n 1p secret.txt'","output":"private output"}});
    let a = call("activity.describe", &json!({"row":row})).unwrap();
    assert_eq!(keys(&a), ["active", "category", "command", "failed", "id", "runId", "status", "title", "type"]);
    assert_eq!((a["category"].as_str(), a["command"].as_str(), a["active"].as_bool(), a["failed"].as_bool()), (Some("read"), Some("sed"), Some(true), Some(false)));
    assert!(!a.to_string().contains("secret") && !a.to_string().contains("private"));
    for (item, category) in [
        (json!({"type":"reasoning","status":"completed"}), "think"),
        (json!({"type":"compaction","status":"running"}), "compact"),
        (json!({"type":"file_change","fileName":"src/a.rs","status":"completed"}), "edit"),
        (json!({"type":"web_search","status":"completed"}), "web"),
        (json!({"type":"dynamic_tool","toolName":"mcp__x__lookup","input":{"title":"Lookup"},"status":"failed"}), "tool"),
        (json!({"type":"subagent","status":"running"}), "agent"),
    ] {
        let a = call("activity.describe", &json!({"row":item})).unwrap();
        assert_eq!(a["category"], category, "{item}");
        assert_eq!(keys(&a).len(), 9);
    }
    assert_eq!(call("activity.describe", &json!({"row":{"type":"dynamic_tool","toolName":"t","input":{"title":"Lookup"},"status":"failed"}})).unwrap()["title"], "Lookup");
    assert_eq!(call("activity.describe", &json!({"row":{"type":"file_change","fileName":"src/a.rs"}})).unwrap()["title"], "src/a.rs");
    for skipped in ["user_message", "assistant_message", "checkpoint", "todo_list", "system_notice", "error"] {
        assert!(call("activity.describe", &json!({"row":{"type":skipped}})).unwrap().is_null(), "{skipped}");
    }
    assert_eq!(call("activity.describe", &json!({"row":"x"})).unwrap_err(), "invalid_row");
}

#[test]
fn changes_summaries_has_one_shape_per_run() {
    let p = json!({"checkpoints":[
        {"id":"c1","status":"ready","runId":"r1","appRunOrdinal":2,"capturedAt":"2026-10-20T09:05:00Z","files":[{"path":"a.rs","kind":"modified","additions":4,"deletions":2},{"path":"b.md","kind":"added","additions":10}]},
        {"id":"c1b","status":"ready","runId":"r1","appRunOrdinal":2,"capturedAt":"2026-10-20T09:06:00Z","files":[{"path":"a.rs","kind":"modified","additions":5,"deletions":2}]},
        {"id":"nested","status":"ready","runId":"r1","appRunOrdinal":null,"files":[{"path":"child.rs"}]},
        {"id":"pending","status":"capturing","runId":"r2","appRunOrdinal":3,"files":[{"path":"x.rs"}]},
        {"id":"empty","status":"ready","runId":"r3","appRunOrdinal":4,"files":[]}
    ]});
    let rows = call("changes.summaries", &json!({"projection":p})).unwrap();
    let rows = rows.as_array().unwrap();
    assert_eq!(rows.len(), 1, "nested scopes, unready and empty checkpoints add no card");
    let r = &rows[0];
    assert_eq!(keys(r), ["additions", "capturedAt", "deletions", "files", "fromTurnCount", "id", "runId", "toTurnCount"]);
    assert_eq!((r["id"].as_str(), r["fromTurnCount"].as_u64(), r["toTurnCount"].as_u64()), (Some("c1b"), Some(1), Some(2)), "a run's latest checkpoint wins");
    assert_eq!((r["additions"].as_u64(), r["deletions"].as_u64()), (Some(5), Some(2)));
    assert_eq!(keys(&r["files"][0]), ["additions", "deletions", "kind", "path"]);
    let first = call("changes.summaries", &json!({"projection":{"checkpoints":[p["checkpoints"][0]]}})).unwrap();
    assert_eq!(first[0]["files"][1], json!({"path":"b.md","kind":"added","additions":10,"deletions":null}));
    assert_eq!(call("changes.summaries", &json!({"projection":{}})).unwrap(), json!([]));
}
