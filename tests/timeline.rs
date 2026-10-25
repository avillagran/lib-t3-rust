//! `timeline`: ot3's thread view and conversation timeline, with ot3's own
//! Rust tests ported and the projections of its integration tests as fixtures.
use serde_json::{json, Value};
use t3_protocol::timeline::{self, Render};

fn fixtures() -> Vec<(String, Value)> {
    let fx: Value = serde_json::from_str(include_str!("fixtures/timeline-projections.json")).unwrap();
    fx.as_array().unwrap().iter().map(|f| (f["name"].as_str().unwrap().to_string(), f.clone())).collect()
}
fn fixture(name: &str) -> Value {
    fixtures().into_iter().find(|(n, _)| n == name).unwrap().1
}
fn plain(p: &Value) -> Value {
    timeline::view(p, &json!({}), &Render::neutral(false)).unwrap()
}
fn md(input: &str) -> String {
    input.to_string()
}

#[test]
fn subagents_get_their_own_cards_and_failures_their_own_words() {
    let rows = timeline::conversation(&fixture("subagents-and-errors")["projection"], &[], &md);
    let cards = rows.iter().find(|r| r["role"] == "subagents").unwrap();
    let agents = cards["subagents"].as_array().unwrap();
    assert_eq!(agents.len(), 2, "one card group for the run");
    assert_eq!(agents[0]["title"], "Review tests");
    assert_eq!(agents[0]["childThreadId"], "c1");
    assert_eq!(agents[0]["progress"], "Reading files");
    assert_eq!(agents[0]["prompt"], "Check the tests");
    assert_eq!(agents[1]["title"], "fix-lint");
    assert!(agents[1]["result"].is_null(), "T3's automatic end notice is not a result");
    let errors = rows.iter().filter(|r| r["role"] == "error").collect::<Vec<_>>();
    assert_eq!(errors.len(), 1, "blank failures add no row");
    assert_eq!(errors[0]["error"]["message"], "The model refused: context too long.");
    assert_eq!(errors[0]["error"]["code"], "context_length");
    assert!(rows.iter().all(|r| r["role"] != "activity"), "subagents are not folded into the tool log");
}

#[test]
fn live_state_keeps_run_clock_and_latest_tool_context() {
    let base = 1_700_000_000_000_i64;
    let items = (0..650).map(|n| json!({"id":format!("t{n}"),"runId":"r","type":"command_execution","input":"rg file","status":if n==649{"running"}else{"completed"},"startedAt":base+1000+n})).collect::<Vec<_>>();
    let p = json!({"thread":{"activeRunId":"r"},"runs":[{"id":"r","status":"running","createdAt":base+500,"startedAt":base+600}],"messages":[{"id":"m","role":"user","text":"hello","createdAt":base+500}],"turnItems":items});
    let view = plain(&p);
    assert_eq!(view["threadState"]["startedAt"], base + 600);
    assert_eq!(view["activity"][0]["command"], "rg");
    let last = view["timeline"].as_array().unwrap().last().unwrap().clone();
    assert_eq!(last["activity"]["active"], true);
    assert_eq!(last["activity"]["entries"].as_array().unwrap().len(), 600);
}

#[test]
fn live_completion_updates_the_sidebar_return_without_a_metadata_update() {
    let view = plain(&fixture("live-completion")["projection"]);
    let base = 1_700_000_000_000_i64;
    assert_eq!(view["threadState"]["inboxAt"], base + 9000);
    assert_eq!(view["threadState"]["working"], false);
    assert_eq!(view["threadState"]["activeAnchorAt"], base);
}

#[test]
fn live_activity_does_not_invent_work_or_export_inputs() {
    let p = fixture("settlement-live-activity")["projection"].clone();
    let activity = timeline::live_activity(&p);
    assert_eq!(activity.as_array().unwrap().len(), 1);
    assert_eq!(activity[0]["id"], "live");
    assert!(!activity.to_string().contains("private command"));
    let mut stopped = p.clone();
    stopped["thread"]["activeRunId"] = Value::Null;
    assert_eq!(timeline::live_activity(&stopped), json!([]));
}

#[test]
fn contextual_timeline_keeps_active_work_and_message_order() {
    let p = fixture("contextual-activity")["projection"].clone();
    let messages = timeline::messages(&p, &Render::neutral(false)).unwrap();
    let rows = timeline::conversation(&p, &messages, &md);
    assert_eq!(rows.len(), 3);
    assert_eq!((rows[0]["id"].as_str(), rows[2]["id"].as_str()), (Some("u"), Some("a")));
    let a = &rows[1]["activity"];
    assert_eq!((a["category"].as_str(), a["active"].as_bool(), a["count"].as_u64(), a["command"].as_str()), (Some("read"), Some(true), Some(2), Some("cat")));
    assert_eq!(a["entries"].as_array().unwrap().len(), 2);
    let all = json!(rows).to_string();
    assert!(!all.contains("secret.txt") && !all.contains("private output"));
}

#[test]
fn timeline_rows_carry_a_content_revision() {
    let row = |text: &str| timeline::conversation(&json!({}), &[json!({"id":"m","role":"assistant","text":text,"createdAt":1})], &md);
    let (a, b, c) = (row("one"), row("one"), row("two"));
    assert_eq!(a[0]["rev"], b[0]["rev"]);
    assert_ne!(a[0]["rev"], c[0]["rev"]);
    assert_eq!(a[0]["rev"].as_str().unwrap().len(), 16);
}

#[test]
fn grouped_activity_retains_each_native_source_without_output() {
    let rows = timeline::conversation(&fixture("grouped-inherited-activity")["projection"], &[], &md);
    let entries = rows[0]["activity"]["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 2);
    for (n, entry) in entries.iter().enumerate() {
        assert_eq!(entry["inspector"]["sourceThreadId"], "ancestor");
        assert_eq!(entry["inspector"]["sourceItemId"], format!("item{n}"));
        assert_eq!(entry["inspector"]["providerTurnId"], "p");
    }
    assert!(!json!(rows).to_string().contains("private-"));
}

#[test]
fn context_window_prefers_turn_usage_then_provider_thread_then_compaction() {
    let w = timeline::context_window(&fixture("context-window-turns")["projection"]);
    assert_eq!((w["usedTokens"].as_f64(), w["usedPercentage"].as_f64(), w["remainingTokens"].as_f64()), (Some(10.0), Some(10.0), Some(90.0)));
    let p = json!({"thread":{"activeProviderThreadId":"pt"},"providerTurns":[],"providerThreads":[{"id":"pt","contextUsage":{"usedTokens":7,"maxTokens":70,"cost":0.5}}]});
    assert_eq!((timeline::context_window(&p)["usedPercentage"].as_f64(), timeline::context_window(&p)["cost"].as_f64()), (Some(10.0), Some(0.5)));
    let c = timeline::context_window(&fixture("context-window-compaction")["projection"]);
    assert_eq!(c["usedTokens"], 1200.0);
    assert!(c["maxTokens"].is_null());
    assert!(timeline::context_window(&json!({"thread":{}})).is_null());
}

#[test]
fn every_row_kind_appears_in_time_order_and_rolled_back_runs_vanish() {
    let d = fixture("kitchen-sink");
    let view = timeline::view(&d["projection"], &d, &Render::neutral(false)).unwrap();
    let rows = view["timeline"].as_array().unwrap();
    let roles = rows.iter().map(|r| r["role"].as_str().unwrap()).collect::<std::collections::BTreeSet<_>>();
    assert_eq!(roles.into_iter().collect::<Vec<_>>(), ["activity", "assistant", "changes", "context", "error", "plan", "secret", "subagents", "user"]);
    let times = rows.iter().map(|r| r["createdAt"].as_i64().unwrap()).collect::<Vec<_>>();
    assert!(times.windows(2).all(|w| w[0] <= w[1]));
    let all = json!(rows).to_string();
    assert!(!all.contains("m-x") && !all.contains("ti-x") && !all.contains("cp-x"), "a rollback removes its messages, tools and changes");
    assert!(!rows.iter().any(|r| r["id"] == "m-queued"), "queued messages wait in the queue, not the timeline");
    assert!(!rows.iter().any(|r| r["id"] == "work/ti-early"), "nothing before the first message");
    let user = rows.iter().find(|r| r["id"] == "m-1").unwrap();
    assert_eq!((user["attachments"][0]["id"].as_str(), user["revertReference"].is_null()), (Some("img-1"), true));
    assert_eq!(rows.iter().find(|r| r["id"] == "m-auto").unwrap()["attribution"], "automation");
    assert_eq!(rows.iter().find(|r| r["id"] == "m-3").unwrap()["forkRunId"], "run-1", "one fork action per run, on its last reply");
    assert!(rows.iter().find(|r| r["id"] == "m-2").unwrap()["forkRunId"].is_null());
    assert_eq!(rows.iter().find(|r| r["id"] == "m-3").unwrap()["codeBlocks"][0]["command"], "cargo test");
    let changes = rows.iter().find(|r| r["role"] == "changes").unwrap();
    assert_eq!((changes["changes"]["additions"].as_u64(), changes["changes"]["deletions"].as_u64()), (Some(14), Some(2)));
    let html = rows.iter().find(|r| r["id"] == "html/ti-html").unwrap();
    assert_eq!(html["attachments"][0]["id"], "att-html");
    let app = rows.iter().find(|r| r["id"] == "app/ti-app").unwrap();
    assert_eq!(app["attachments"][0]["mcpApp"]["threadId"], "thread-k");
    let error = rows.iter().find(|r| r["role"] == "error").unwrap();
    assert_eq!((error["error"]["class"].as_str(), error["error"]["providerInstanceId"].as_str()), (Some("usage_limit"), Some("agent-b")));
    assert!(rows.iter().all(|r| r["rev"].as_str().is_some_and(|rev| rev.len() == 16)));

    let state = &view["threadState"];
    assert_eq!((state["working"].as_bool(), state["activeRunId"].as_str()), (Some(true), Some("run-3")));
    assert_eq!((state["mergeBackTargetThreadId"].as_str(), state["mergeBackRunId"].as_str()), (Some("thread-src"), None), "a later running run holds the merge back");
    assert_eq!(state["contextWindow"]["usedTokens"], 4000.0);
    assert_eq!(state["subagents"][0]["title"], "/agents/reviewer".rsplit('/').next().unwrap());
    assert_eq!(view["tasks"]["current"], "Test");
    assert_eq!(view["queue"]["items"][0]["text"], "later please");
    assert_eq!(view["approvals"][0]["requestId"], "rq");
    assert_eq!(view["activity"].as_array().unwrap().len(), 3, "the two running tools and the running subagent");
    assert_eq!((view["historyCursor"].as_str(), view["hasMoreHistory"].as_bool()), (Some("cursor-1"), Some(true)));
    assert!(view["messages"][0].get("rendered").is_none(), "no host renderer, no rendered text");
}

#[test]
fn a_host_renderer_fills_rendered_and_plans() {
    let d = fixture("plans");
    let render = |text: &str, blocks: &[Value], math: bool| format!("{math}:{text}:{}", blocks.len());
    let view = timeline::view(&d["projection"], &d, &Render { math: true, message: Some(&render), plan: &|s: &str| format!("<p>{s}</p>"), blocks: false }).unwrap();
    assert_eq!(view["planView"]["ready"]["id"], "plan-2");
    let plan = view["timeline"].as_array().unwrap().iter().find(|r| r["role"] == "plan" && r["plan"]["id"] == "plan-2").unwrap();
    assert_eq!(plan["plan"]["rendered"], "<p>One\n\n## Steps\nTwo</p>", "the plan's display Markdown, without its title and Summary heading");
    let msgs = fixture("code-actions-and-images");
    let view = timeline::view(&msgs["projection"], &msgs, &Render { math: false, message: Some(&render), plan: &md, blocks: false }).unwrap();
    let m = &view["messages"];
    assert_eq!(m[0]["rendered"], format!("false:{}:2", m[0]["text"].as_str().unwrap()));
    assert_eq!(m[0]["codeBlocks"][0]["command"], "bash experiments/tooltips/run-native");
    assert_eq!(m[1]["markdownImages"].as_array().unwrap().len(), 2);
    assert_eq!(m[1]["tables"][0]["csv"], "a,b\n1,2");
    assert_eq!(m[2]["text"], "Hello & world", "a Qt rich-text document is shown as its text");
    assert_eq!(timeline::messages(&json!({"messages":[{"id":"x","role":"user"}]}), &Render::neutral(false)).unwrap_err(), "malformed");
}

#[test]
fn timeline_through_domain() {
    let d = fixture("kitchen-sink");
    let mut args = json!({"projection":d["projection"],"historyCursor":"cursor-1","hasMoreHistory":true});
    let view = t3_protocol::domain::call("timeline.view", &args).unwrap();
    assert!(view.get("messages").is_none(), "message rows travel once, inside the timeline");
    assert_eq!(view["timeline"], plain_with_history(&d)["timeline"]);
    assert_eq!(view["hasMoreHistory"], true);
    args["mathRenderingEnabled"] = json!("yes");
    assert_eq!(t3_protocol::domain::call("timeline.view", &args).unwrap_err(), "invalid_math_rendering_enabled");
    assert_eq!(t3_protocol::domain::call("timeline.liveActivity", &json!({"projection":d["projection"]})).unwrap(), timeline::live_activity(&d["projection"]));
    assert_eq!(t3_protocol::domain::call("timeline.view", &json!({"projection":{"messages":7}})).unwrap_err(), "malformed");
}
fn plain_with_history(d: &Value) -> Value {
    timeline::view(&d["projection"], d, &Render::neutral(false)).unwrap()
}
