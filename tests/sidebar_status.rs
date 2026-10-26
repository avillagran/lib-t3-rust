use serde_json::{json, Value};
use t3_protocol::sidebar::{metadata, sidebar_status};
fn iso(ms: i64) -> String {
    chrono::DateTime::from_timestamp_millis(ms).unwrap().to_rfc3339()
}
#[test]
fn sidebar_status_follows_t3_0_1_2_runtime_rules() {
    let v2 = |extra: Value| {
        let mut t = json!({"latestRunId":"r","status":"completed"});
        t.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        sidebar_status(&t)
    };
    assert_eq!(v2(json!({"status":"queued"})), "working");
    assert_eq!(v2(json!({"status":"idle"})), "waiting");
    assert_eq!(v2(json!({"status":"error"})), "failed");
    assert_eq!(v2(json!({"pendingRuntimeRequest":{"kind":"auth_refresh"}})), "ready", "auth refresh is not an approval");
    assert_eq!(v2(json!({"pendingSubagentRequest":"approval"})), "approval");
    assert_eq!(v2(json!({"pendingSubagentRequest":{"kind":"user_input"}})), "input");
    assert_eq!(v2(json!({"status":"completed","pendingRuntimeRequest":{"kind":"user_input","blocking":false}})), "input", "an async question still asks once the run stopped");
    assert_eq!(v2(json!({"status":"failed","pendingBackgroundTasks":[{"kind":"subagent"}]})), "failed", "a failure is never parked");
    assert_eq!(v2(json!({"activeProviderThreadId":null,"latestRunId":null})), "ready");
    // Legacy shells: runtime (or session) carries status and error class.
    assert_eq!(sidebar_status(&json!({"runtime":{"status":"running"}})), "working");
    assert_eq!(sidebar_status(&json!({"session":{"status":"failed","lastErrorClass":"usage_limit"}})), "limited");
    assert_eq!(metadata(&json!({"latestRunId":"r","status":"running"}), 0)["sidebarStatus"], "working");
}
#[test]
fn v2_thread_errors_and_async_questions_follow_the_thread_fields() {
    let now = 1_000_000;
    let m = metadata(&json!({"status":"failed","lastError":"quota","lastErrorClass":"usage_limit"}), now);
    assert_eq!((m["hasRuntimeError"].as_bool(), m["runtimeErrorClass"].as_str()), (Some(true), Some("usage_limit")));
    assert_eq!(metadata(&json!({"lastError":"  "}), now)["hasRuntimeError"], false);
    assert_eq!(metadata(&json!({"pendingRuntimeRequest":{"kind":"user_input","blocking":false}}), now)["canSnooze"], true);
    assert_eq!(metadata(&json!({"pendingRuntimeRequest":{"kind":"user_input"}}), now)["canSnooze"], false);
    // A failed V2 run does not count as queued just because a message is recent.
    assert_eq!(metadata(&json!({"status":"failed","latestUserMessageAt":iso(now - 100)}), now)["canSnooze"], true);
    assert_eq!(metadata(&json!({"latestUserMessageAt":iso(now - 100),"latestRunStartedAt":iso(now)}), now)["canSnooze"], true);
}
#[test]
fn only_a_request_or_a_fresh_failure_wakes_a_snooze_early() {
    let now = 10_000_000;
    let snoozed = json!({"latestRunId":"r","status":"completed","snoozedAt":iso(now - 5000),"snoozedUntil":iso(now + 60_000),"latestRunCompletedAt":iso(now - 1000)});
    let m = metadata(&snoozed, now);
    assert_eq!((m["snoozed"].as_bool(), m["woke"].as_bool()), (Some(true), Some(false)), "a completed run never wakes it");
    assert!(m["wokeAt"].is_null());
    let mut asked = snoozed.clone();
    asked["hasPendingApprovals"] = json!(true);
    asked["updatedAt"] = json!(iso(now - 300));
    let m = metadata(&asked, now);
    assert_eq!((m["snoozed"].as_bool(), m["woke"].as_bool(), m["wokeAt"].as_i64()), (Some(false), Some(true), Some(now - 300)));
    let mut failed = snoozed;
    failed["status"] = json!("failed");
    let m = metadata(&failed, now);
    assert_eq!((m["snoozed"].as_bool(), m["wokeAt"].as_i64()), (Some(false), Some(now - 1000)));
    failed["lastVisitedAt"] = json!(iso(now - 500));
    assert_eq!(metadata(&failed, now)["woke"], false, "visited after it woke");
}

#[test]
fn background_monitors_and_commands_hold_only_when_awaited() {
 use t3_protocol::sidebar::background_work_holds as holds;
 assert!(!holds(&json!([{"kind":"monitor"}])));assert!(!holds(&json!([{"kind":"command"}])));
 assert!(holds(&json!([{"kind":"monitor","awaited":true}])));assert!(holds(&json!([{"kind":"command","awaited":true}])));
 assert!(holds(&json!([{"kind":"subagent"}])));assert!(holds(&json!([{"kind":"background_task"}])));
}
