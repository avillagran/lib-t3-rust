use serde_json::{json, Value};
use t3_protocol::projection::apply;
fn event(typ: &str, payload: Value) -> Value {
    json!({"type":typ,"payload":payload})
}
#[test]
fn partial_payload_events_of_t3_0_1_touch_only_the_row_they_name() {
    let mut p = json!({"thread":{"id":"one","title":"Kept","pullRequests":[
        {"host":"github.com","repository":"a/b","number":3,"state":"open"},
        {"host":"github.com","repository":"a/b","number":4,"state":"open"}
    ]},"providerSessions":[{"id":"s1"},{"id":"s2"}],"runs":[{"id":"r","status":"running"},{"id":"other"}]});
    assert!(apply(&mut p, &event("thread.pull-request-link-updated", json!({"host":"github.com","repository":"a/b","number":4,"state":"merged"}))));
    assert_eq!(p["thread"]["title"], "Kept", "a link update is not a whole thread row");
    assert_eq!(p["thread"]["pullRequests"][0]["state"], "open");
    assert_eq!(p["thread"]["pullRequests"][1]["state"], "merged");
    assert!(apply(&mut p, &event("provider-session.detached", json!({"providerSessionId":"s2"}))));
    assert_eq!(p["providerSessions"], json!([{"id":"s1"}]));
    assert!(apply(&mut p, &event("run.background-work-cancelled", json!({"runId":"r","restartCancelledBackgroundWork":[{"taskId":"dev"}]}))));
    assert_eq!(p["runs"][0]["status"], "running");
    assert_eq!(p["runs"][0]["restartCancelledBackgroundWork"][0]["taskId"], "dev");
    assert!(p["runs"][1].get("restartCancelledBackgroundWork").is_none());
    let before = p.clone();
    assert!(apply(&mut p, &event("checkpoint.rollback-requested", json!({"checkpointId":"c"}))));
    assert_eq!(p, before, "a rollback request changes nothing until its result arrives");
    let mut bare = json!({"thread":{"id":"one"}});
    assert!(!apply(&mut bare, &event("thread.pull-request-link-updated", json!({"number":1}))), "no links to update");
}
#[test]
fn context_handoffs_and_transfers_are_mirrored() {
    let mut p = json!({"thread":{"id":"one"}});
    assert!(apply(&mut p, &event("context-handoff.updated", json!({"id":"h","status":"pending"}))));
    assert!(apply(&mut p, &event("context-handoff.updated", json!({"id":"h","status":"done"}))));
    assert_eq!(p["contextHandoffs"], json!([{"id":"h","status":"done"}]));
    assert!(apply(&mut p, &event("context-transfer.created", json!({"id":"x"}))));
    assert!(apply(&mut p, &event("context-transfer.updated", json!({"id":"x","status":"applied"}))));
    assert_eq!(p["contextTransfers"], json!([{"id":"x","status":"applied"}]));
}
#[test]
fn provider_turn_keeps_token_usage_only_when_an_update_omits_it() {
    let mut p = json!({"thread":{"id":"one"},"providerTurns":[{"id":"t","tokenUsage":{"usedTokens":5}}]});
    assert!(apply(&mut p, &event("provider-turn.updated", json!({"id":"t","status":"running"}))));
    assert_eq!(p["providerTurns"][0]["tokenUsage"]["usedTokens"], 5);
    assert!(apply(&mut p, &event("provider-turn.updated", json!({"id":"t","tokenUsage":{"usedTokens":9}}))));
    assert_eq!(p["providerTurns"][0]["tokenUsage"]["usedTokens"], 9);
    assert!(apply(&mut p, &event("provider-turn.updated", json!({"id":"new"}))));
    assert!(p["providerTurns"][1]["tokenUsage"].is_null());
}
#[test]
fn live_mcp_app_keeps_its_reference_when_outputs_are_dropped() {
    use t3_protocol::documents::mcp_app;
    let app = json!({"t3McpApp":{"attachmentId":"att","server":"weather","tool":"forecast","resourceUri":"ui://weather/forecast"}});
    // Text content carrying the app as JSON, as MCP tool results do.
    let item = json!({"id":"i1","threadId":"t","type":"mcp_tool_call","status":"completed","toolName":"weather.forecast","output":{"content":[{"type":"text","text":app.to_string()}],"secret":"dropped"}});
    let mut p = json!({"thread":{"id":"t"}});
    assert!(apply(&mut p, &event("turn-item.updated", item.clone())));
    let kept = &p["turnItems"][0];
    assert_eq!(kept["output"], app, "only the app reference survives compaction");
    assert_eq!(mcp_app(kept), mcp_app(&item));
    assert_eq!(mcp_app(kept).unwrap()["mcpApp"]["threadId"], "t");
    let mut running = item.clone();
    running["status"] = json!("inProgress");
    assert!(mcp_app(&running).is_none());
    let mut failed = item;
    failed["output"]["isError"] = json!(true);
    assert!(mcp_app(&failed).is_none());
    let mut p = json!({"thread":{"id":"t"}});
    assert!(apply(&mut p, &event("turn-item.updated", failed)));
    assert!(p["turnItems"][0]["output"].is_null());
}
