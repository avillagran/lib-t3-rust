use serde_json::{json, Value};
use t3_protocol::{domain::call, parity};
fn run(name: &str, v: Value) -> Value {
    call(name, &v).unwrap()
}
#[test]
fn permission_fallback_and_advertised_order() {
    let v = run(
        "models.permissionView",
        json!({"catalog":{},"runtimeMode":"full-access"}),
    );
    assert_eq!(v["rows"].as_array().unwrap().len(), 4);
    assert_eq!(v["current"]["id"], "full-access");
    let v = run(
        "models.permissionView",
        json!({"catalog":{"providers":[{"instanceId":"p","supportedRuntimeModes":["full-access","auto"]}]},"selection":{"instanceId":"p"},"runtimeMode":"approval-required"}),
    );
    assert_eq!(v["rows"][0]["id"], "auto");
    assert_eq!(v["current"]["id"], "auto");
}
#[test]
fn history_overlap_recent_wins_and_stale_page_rejected() {
    let mut v = json!({"older":[{"id":"a","createdAt":1,"text":"old"}],"recent":[{"id":"a","createdAt":1,"text":"live"}],"page":[{"id":"a","createdAt":1,"text":"page"},{"id":"b","createdAt":0}],"owner":"host:thread:g2","pageOwner":"host:thread:g2","cursor":"c","pageCursor":"c","hasMoreHistory":false});
    let out = run("history.mergeView", v.clone());
    assert_eq!(out["timeline"][1]["text"], "live");
    assert_eq!(out["older"][1]["text"], "page");
    assert_eq!(out["cursor"], Value::Null);
    assert_eq!(out["hasMoreHistory"], false);
    v["pageOwner"] = json!("host:thread:g1");
    assert_eq!(run("history.mergeView", v.clone())["applied"], false);
    v["pageOwner"] = v["owner"].clone();
    v["pageCursor"] = json!("other");
    assert_eq!(run("history.mergeView", v)["applied"], false);
}
#[test]
fn pr_overlap_host_scope_invalid_dates_and_refresh() {
    let pages = json!({"pages":[{"items":[{"environmentId":"a","host":"GitHub.com","repository":"O/R","number":1,"updatedAt":"bad","title":"old"}]},{"items":[{"environmentId":"a","host":"github.com","repository":"o/r","number":1,"updatedAt":"2026-01-02T00:00:00Z","title":"new"},{"environmentId":"b","host":"github.com","repository":"o/r","number":1,"updatedAt":"bad"}]}]});
    let v = run("pullRequests.listView", pages);
    assert_eq!(v["items"].as_array().unwrap().len(), 2);
    assert_eq!(v["items"][0]["title"], "new");
    assert_eq!(
        run("pullRequests.listView", json!({"pages":[]}))["items"],
        json!([])
    );
}
#[test]
fn output_split_escape_cr_replace_and_owner() {
    let r = json!({"threadId":"t","itemId":"i","generation":1});
    let first = run(
        "commandOutput.apply",
        json!({"reference":r,"frames":[{"kind":"append","text":"10%\r\u{1b}[3"}]}),
    );
    let next = run(
        "commandOutput.apply",
        json!({"reference":r,"state":first["state"],"frames":[{"kind":"append","text":"2m50%\r\n\u{1b}]title\u{1b}"}]}),
    );
    let end = run(
        "commandOutput.apply",
        json!({"reference":r,"state":next["state"],"frames":[{"kind":"append","text":"\\done","running":false}]}),
    );
    assert_eq!(end["text"], "50%\ndone");
    assert_eq!(end["running"], false);
    let replace = run(
        "commandOutput.apply",
        json!({"reference":r,"state":end["state"],"frames":[{"kind":"replace","text":"fresh"}]}),
    );
    assert_eq!(replace["text"], "fresh");
    let changed = run(
        "commandOutput.apply",
        json!({"reference":{"threadId":"t","itemId":"new"},"state":replace["state"],"frames":[],"closed":true}),
    );
    assert_eq!(changed["text"], "");
    assert_eq!(changed["running"], false);
}
#[test]
fn output_multibyte_tail_and_error() {
    let v = run(
        "commandOutput.apply",
        json!({"reference":{"threadId":"t","itemId":"i"},"frames":[{"kind":"append","text":"🦀".repeat(20000)}],"error":"disconnected"}),
    );
    assert!(v["text"].as_str().unwrap().len() <= 65536);
    assert_eq!(v["truncated"], true);
    assert_eq!(v["running"], false);
}
#[test]
fn utf16_completion_replace_stale_and_combining() {
    let v = run(
        "composer.transform",
        json!({"input":{"action":"completion","text":"🦀 é @fi","cursor":9}}),
    );
    assert_eq!(v["start"], 6);
    assert_eq!(v["query"], "fi");
    let v = run(
        "composer.transform",
        json!({"input":{"action":"replace","text":"🦀 @fi","cursor":6,"start":3,"expectedRange":"@fi","insert":"file"}}),
    );
    assert_eq!(v["text"], "🦀 file");
    assert_eq!(v["cursor"], 7);
    assert!(call(
        "composer.transform",
        &json!({"input":{"action":"replace","text":"🦀","cursor":1,"start":0,"expectedRange":"x"}})
    )
    .is_err());
    assert_eq!(
        call(
            "composer.transform",
            &json!({"input":{"action":"replace","text":"new","expectedText":"old"}})
        )
        .unwrap_err(),
        "stale_editor"
    );
}
#[test]
fn list_continue_exit_and_recall() {
    assert_eq!(
        run(
            "composer.transform",
            json!({"input":{"action":"list","text":"9. [x] done","cursor":11}})
        )["text"],
        "9. [x] done\n10. [ ] "
    );
    assert_eq!(
        run(
            "composer.transform",
            json!({"input":{"action":"list","text":"- ","cursor":2}})
        )["text"],
        ""
    );
    assert_eq!(
        run(
            "composer.transform",
            json!({"input":{"action":"recall","messages":[{"role":"user","recallableText":"one"},{"role":"user","recallableText":"two","attribution":{}},{"role":"user","recallableText":"three"}]}})
        ),
        json!(["three", "one"])
    );
}
#[test]
fn pins_midpoint_and_rebalance() {
    let v = run(
        "sidebar.pinPlan",
        json!({"orderedIds":["a","b","c"],"keys":{"a":"b","c":"d"},"movedId":"b"}),
    );
    let key = v[0]["orderKey"].as_str().unwrap();
    assert!(key > "b" && key < "d");
    let v = run(
        "sidebar.pinPlan",
        json!({"orderedIds":["a","b","c"],"keys":{},"movedId":"b"}),
    );
    assert_eq!(v.as_array().unwrap().len(), 3);
    assert!(v[0]["orderKey"].as_str() < v[1]["orderKey"].as_str());
}
#[test]
fn project_builder_validation() {
    let v = run(
        "projects.mutation",
        json!({"input":{"action":"project-rename","projectId":"p","title":" hello "},"commandId":"stable"}),
    );
    assert_eq!(v["commandId"], "stable");
    assert_eq!(v["title"], "hello");
    assert!(call(
        "projects.mutation",
        &json!({"input":{"action":"project-remove","projectId":"p"},"commandId":"id"})
    )
    .is_err());
}
#[test]
fn provider_safe_projection_and_mutation() {
    let settings = json!({"providerInstances":{"codex":{"driver":"codex","config":{"secret":"DO_NOT_RETURN"}}}});
    let view = run(
        "providerInstances.projection",
        json!({"settings":settings,"config":{},"writable":true}),
    );
    assert!(!view.to_string().contains("DO_NOT_RETURN"));
    assert!(call(
        "providerInstances.mutation",
        &json!({"input":{"action":"remove","instanceId":"codex"},"settings":settings})
    )
    .is_err());
}
#[test]
fn reference_routing_decode_and_scope() {
    let base = json!({"environmentId":"b","threadId":"current","environments":[{"id":"a","connected":true,"threads":[{"id":"id:1"}]},{"id":"b","connected":true,"threads":[{"id":"id:1"}]}]});
    let mut v = base.clone();
    v["href"] = json!("t3-thread://v1/id%3A1");
    assert_eq!(run("reader.reference", v)["environmentId"], "b");
    let mut v = base.clone();
    v["href"] = json!("file:///remote/a%20🦀.md:12:3");
    let out = run("reader.reference", v);
    assert_eq!(out["resource"]["path"], "/remote/a 🦀.md");
    assert_eq!(out["resource"]["threadId"], "current");
    for href in ["HTTPS://example.com/x", "#section", "javascript:alert(1)"] {
        let mut v = base.clone();
        v["href"] = json!(href);
        let out = run("reader.reference", v);
        assert!(["external", "anchor", "unsupported"].contains(&out["kind"].as_str().unwrap()));
    }
    let mut v = base;
    v["href"] = json!("file:///bad%ZZ");
    assert!(call("reader.reference", &v).is_err());
}
#[test]
fn terminal_response_actions() {
    let mut rows = vec![
        json!({"role":"assistant","text":"thinking"}),
        json!({"role":"activity"}),
        json!({"role":"assistant","text":"answer"}),
        json!({"role":"user","text":"next"}),
        json!({"role":"assistant","text":"live","streaming":true}),
    ];
    parity::message_actions(&mut rows);
    assert_eq!(rows[0]["commentary"], true);
    assert_eq!(rows[2]["showMessageActions"], true);
    assert_eq!(rows[4]["showMessageActions"], false);
}
#[test]
fn registry_icon_official_source_content_and_cancel() {
    use t3_protocol::registry_icon::*;
    for url in [
        "https://evil.test/registry/v1/a.svg",
        "https://cdn.agentclientprotocol.com/registry/v1/a.svg?token=x",
        "http://cdn.agentclientprotocol.com/registry/v1/a.svg",
    ] {
        assert!(validate(url).is_err());
    }
    assert!(validate_content(b"<svg></svg>").is_ok());
    assert!(validate_content(b"<html>oops</html>").is_err());
    assert_eq!(
        bytes(
            "https://cdn.agentclientprotocol.com/registry/v1/a.svg",
            || true
        )
        .err()
        .unwrap(),
        "cancelled"
    );
}
#[test]
fn new_thread_defaults_and_host_route_block() {
    let mut input = json!({"projects":[{"id":"p","environmentId":"e","title":"Project","defaultThreadEnvMode":"local","defaultModelSelection":{"instanceId":"p","model":"project"}}],"projectId":"p","environmentId":"e","source":{"modelSelection":{"instanceId":"p","model":"source"}},"build":true,"commandId":"durable","threadId":"thread","title":"New"});
    let v = run("composer.newThread", json!({"input":input}));
    assert_eq!(v["modelSelection"]["model"], "source");
    assert_eq!(v["request"]["fn"], "orchestration.launchThread");
    input["automatic"] = json!(true);
    assert_eq!(
        run("composer.newThread", json!({"input":input}))["missing"],
        json!(["host_route"])
    );
}
#[test]
fn quota_fresh_pool_workspace_and_pace() {
    let fixture: Value = serde_json::from_str(include_str!("fixtures/android-quota.json")).unwrap();
    let v = run("quotaSources.present", fixture);
    assert_eq!(v["providers"][0]["accounts"].as_array().unwrap().len(), 1);
    assert_eq!(v["providers"][0]["windows"][0]["availablePercent"], 60.);
    assert_eq!(v["providers"][0]["windows"][0]["pace"], "under");
    assert_eq!(
        v["notices"][0]["key"],
        "Disconnected; readings may be outdated"
    );
}
#[test]
fn durable_pin_commands_and_reference_serializer() {
    let v = run(
        "sidebar.pinCommands",
        json!({"plan":[{"id":"thread","orderKey":"bn"}],"commandIds":{"thread":"durable-pin"}}),
    );
    assert_eq!(v[0]["type"], "thread.pin.reorder");
    assert_eq!(v[0]["commandId"], "durable-pin");
    assert!(call(
        "sidebar.pinCommands",
        &json!({"plan":[{"id":"thread","orderKey":"bn"}],"commandIds":{}})
    )
    .is_err());
    let href = run("reader.threadReference", json!({"threadId":"thread%id"}));
    assert_eq!(href, "t3-thread://v1/thread%25id");
}
#[test]
fn quota_ambiguous_workspaces_do_not_merge_and_missing_segments_align() {
    let mut fixture: Value =
        serde_json::from_str(include_str!("fixtures/android-quota.json")).unwrap();
    fixture["environments"][0]["state"]["providers"][0]["auth"]["workspaceId"] = json!("w1");
    fixture["environments"][1]["state"]["providers"][0]["auth"]["workspaceId"] = json!("w2");
    let mut third = fixture["environments"][0].clone();
    third["id"] = json!("c");
    third["state"]["providers"][0]["auth"]
        .as_object_mut()
        .unwrap()
        .remove("workspaceId");
    third["state"]["providers"][0]["usageLimits"]["windows"][0]["kind"] = json!("weekly");
    fixture["environments"].as_array_mut().unwrap().push(third);
    let out = run("quotaSources.present", fixture);
    assert_eq!(out["providers"][0]["accounts"].as_array().unwrap().len(), 3);
    let windows = out["providers"][0]["windows"].as_array().unwrap();
    assert!(windows
        .iter()
        .all(|w| w["segments"].as_array().unwrap().len() == 3));
    assert!(windows[0]["segments"]
        .as_array()
        .unwrap()
        .iter()
        .any(Value::is_null));
}
