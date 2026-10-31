use serde_json::json;
use t3_protocol::{api::streams::spec as stream_spec, domain::call};
#[test]
fn context_and_reconnect_use_server_paths() {
    let c=call("terminal.context",&json!({"shell":{"projects":[{"id":"p","workspaceRoot":"/remote"}],"threads":[{"id":"t","projectId":"p"}]},"threadId":"t","fallback":"/fallback"})).unwrap();
    assert_eq!(c["cwd"], "/remote");
    for resume in [false, true] {
        let p=call("terminal.attach",&json!({"threadId":"t","terminalId":"term-1","context":c,"cols":80,"rows":24,"resume":resume})).unwrap();
        assert_eq!(p["restartIfNotRunning"], !resume);
        assert_eq!(p["resume"].as_bool().unwrap_or(false), resume);
    }
}
#[test]
fn output_preserves_split_ansi_unicode_and_replaces_snapshot() {
    let mut state = json!({});
    let mut text = String::new();
    for (i, chunk) in ["\u{1b}[", "31m", "🦀é", "\u{1b}]0;title", "\u{7}"]
        .iter()
        .enumerate()
    {
        let v = call(
            "terminal.output",
            &json!({"state":state,"event":{"type":"output","sequence":i,"data":chunk}}),
        )
        .unwrap();
        text.push_str(v["text"].as_str().unwrap());
        state = v["state"].clone();
    }
    assert_eq!(text, "\u{1b}[31m🦀é\u{1b}]0;title\u{7}");
    let v=call("terminal.output",&json!({"state":state,"event":{"type":"snapshot","snapshot":{"sequence":9,"history":"new","status":"exited","exitCode":0}}})).unwrap();
    assert_eq!(v["replace"], true);
    assert_eq!(v["finished"], true);
    assert_eq!(v["exitCode"], 0);
    let duplicate = call(
        "terminal.output",
        &json!({"state":v["state"],"event":{"type":"output","sequence":9,"data":"duplicate"}}),
    )
    .unwrap();
    assert!(duplicate["text"].is_null());
}
#[test]
fn exit_restart_error_and_clear() {
    let mut state = json!({});
    for event in [
        json!({"type":"exited","exitCode":1,"exitSignal":15}),
        json!({"type":"restarted","snapshot":{"history":"","status":"running","sequence":1}}),
        json!({"type":"cleared","sequence":2}),
        json!({"type":"error","message":"remote"}),
    ] {
        let v = call("terminal.output", &json!({"state":state,"event":event})).unwrap();
        if event["type"] == "restarted" {
            assert_eq!(v["finished"], false)
        }
        if event["type"] == "error" {
            assert_eq!(v["error"], "terminal_remote_error");
            assert_eq!(v["finished"], true)
        }
        state = v["state"].clone();
    }
}
#[test]
fn malformed_state_and_dimensions_fail_locally() {
    assert!(call(
        "terminal.output",
        &json!({"state":{"sequence":-1},"event":{}})
    )
    .is_err());
    assert!(call(
        "terminal.attach",
        &json!({"threadId":"t","terminalId":"x","context":{"cwd":"/r"},"cols":65536,"rows":24})
    )
    .is_err());
}
#[test]
fn observation_is_a_separate_read_only_wire_contract() {
    let spec = stream_spec(
        "terminal.observe",
        &json!({"threadId":"t","terminalId":"x"}),
    )
    .unwrap();
    assert_eq!(spec.tag(), "terminal.observe");
    assert!(stream_spec(
        "terminal.observe",
        &json!({"threadId":"t","terminalId":"x","resume":true})
    )
    .is_err());
}

#[test]
fn reconnect_fixture_and_authorization_are_exact() {
    let events: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("fixtures/terminal-output.json")).unwrap();
    let mut state = json!({});
    for (i, event) in events.iter().enumerate() {
        let v = call("terminal.output", &json!({"state":state,"event":event})).unwrap();
        if i == 1 {
            assert!(v["text"].is_null());
        }
        if i == 5 {
            assert_eq!(v["replace"], true);
            assert_eq!(v["finished"], true);
        }
        state = v["state"].clone();
    }
    assert_eq!(state["finished"], false);
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/t3-contract.json")).unwrap();
    // The saved server contract remains the authorization source of truth.
    assert_eq!(
        fixture["methods"]["terminal.observe"]["scope"],
        "terminal:read"
    );
    assert_eq!(
        fixture["methods"]["terminal.attach"]["scope"],
        "terminal:operate"
    );
    let attach = t3_protocol::api::streams::TERMINAL_ATTACH.describe();
    let observe = t3_protocol::api::streams::TERMINAL_OBSERVE.describe();
    assert_eq!(attach["scope"], "terminal:operate");
    assert_eq!(observe["scope"], "terminal:read");
}
