//! Typed streams against the fake T3 server: one row per stream (args → exact
//! tag and payload in the `Request`), local rejections without a socket, the
//! first-item reads and the `describe` shapes.
#[path = "support/fake_t3.rs"]
mod fake_t3;
use fake_t3::{wire, FakeT3, Reply};
use serde_json::{json, Value};
use std::{
    cell::{Cell, RefCell},
    time::{Duration, Instant},
};
use t3_protocol::{
    api::{self, streams::*},
    Connection,
};

/// Largest `afterSequence` T3 decodes: past it a JS number loses precision.
const SAFE_MAX: u64 = (1 << 53) - 1;

/// (function, args, payload on the wire).
fn rows() -> Vec<(&'static str, Value, Value)> {
    let terminal = json!({"threadId":"t-1","terminalId":"term 1","cwd":"/r","worktreePath":"/r/wt","cols":80,"rows":24,
        "env":{"TERM":"xterm-256color","_X1":""},"providerInstanceId":"codex","restartIfNotRunning":true,"resume":true});
    vec![
        ("orchestration.subscribeShell", json!({"afterSequence":41,"requestCompletionMarker":true}), json!({"afterSequence":41,"requestCompletionMarker":true})),
        (
            "orchestration.subscribeThread",
            json!({"threadId":"t-1","afterSequence":SAFE_MAX,"requestCompletionMarker":true,"acceptBoundedSnapshot":true,"acceptCompactTurnItems":true}),
            json!({"threadId":"t-1","afterSequence":SAFE_MAX,"requestCompletionMarker":true,"acceptBoundedSnapshot":true,"acceptCompactTurnItems":true}),
        ),
        ("orchestration.subscribeCommandOutput", json!({"threadId":"t-1","itemId":"item:1"}), json!({"threadId":"t-1","itemId":"item:1"})),
        ("terminal.attach", terminal.clone(), terminal),
        ("device.subscribeDeviceState", json!({}), json!({})),
        (
            "preview.subscribeDiscoveredLocalServers",
            json!({"configuredUrls":[" http://localhost:3000 ","http://127.0.0.1:8080/app"]}),
            json!({"configuredUrls":["http://localhost:3000","http://127.0.0.1:8080/app"]}),
        ),
        ("preview.subscribePreviewEvents", json!({}), json!({})),
        (
            "server.subscribeServerConfig",
            json!({"environmentThemes":true,"usageLimitSources":false,"usageLimitsCommand":true}),
            json!({"environmentThemes":true,"usageLimitSources":false,"usageLimitsCommand":true}),
        ),
        ("worktreeSetup.subscribeWorktreeSetup", json!({"threadId":"t-1"}), json!({"threadId":"t-1"})),
        ("server.getStorageCleanupReport", json!({}), json!({})),
        ("provider.install.subscribe", json!({"instanceId":"codex"}), json!({"instanceId":"codex"})),
        ("provider.auth.subscribe", json!({"instanceId":"claude_2"}), json!({"instanceId":"claude_2"})),
        (
            "git.runStackedAction",
            json!({"actionId":"act-1","cwd":"/r","action":"commit_push_pr","commitMessage":"  Fix it  ","featureBranch":true,
                "filePaths":["src/a.rs","docs/b c.md"],"threadId":"t-1","projectId":"p-1"}),
            json!({"actionId":"act-1","cwd":"/r","action":"commit_push_pr","commitMessage":"Fix it","featureBranch":true,
                "filePaths":["src/a.rs","docs/b c.md"],"threadId":"t-1","projectId":"p-1"}),
        ),
    ]
}

/// Runs `spec` until its first item and returns it.
fn first(conn: &Connection, spec: &StreamSpec) -> Value {
    let got = RefCell::new(None);
    let done = Cell::new(false);
    let result = spec.subscribe(conn, || done.get(), || false, |item| {
        if let Some(v) = item {
            *got.borrow_mut() = Some(v);
            done.set(true);
        }
        Ok(())
    });
    // The fake ends the stream after its chunks; a cancel first is just as fine.
    assert!(result.is_ok() || result.as_ref().unwrap_err() == "stream_closed", "{result:?}");
    got.into_inner().expect("no item")
}

#[test]
fn every_stream_sends_its_exact_tag_and_payload_once() {
    let rows = rows();
    let fake = FakeT3::spawn((0..rows.len()).map(|i| Reply::Chunks(vec![vec![json!({"row":i})]])).collect());
    let conn = fake.connection();
    let mut tags = vec![];
    for (i, (name, args, _)) in rows.iter().enumerate() {
        let spec = spec(name, args).unwrap();
        assert_eq!(first(&conn, &spec), json!({"row":i}), "{name}");
        tags.push(spec.tag());
    }
    let recorded = fake.finish();
    let expected = rows.iter().zip(&tags).map(|((_, _, payload), tag)| json!({"tag":tag,"payload":payload})).collect::<Vec<_>>();
    assert_eq!(wire(&recorded), expected);
    assert_eq!(recorded.connections, rows.len(), "one socket per stream, no retries");
    // afterSequence crosses as the same integer, never through a float.
    assert_eq!(recorded.requests[1]["payload"]["afterSequence"].as_u64(), Some(SAFE_MAX));
    // Every registered stream has a row, in registry order.
    let streams = api::functions().filter(|f| f.stream).map(|f| f.name).collect::<Vec<_>>();
    assert_eq!(rows.iter().map(|r| r.0).collect::<Vec<_>>(), streams);
    assert_eq!(streams.len(), 13);
}

#[test]
fn rust_subscribe_functions_put_the_same_requests_on_the_wire() {
    let rows = rows();
    let fake = FakeT3::spawn((0..rows.len()).map(|_| Reply::Chunks(vec![vec![json!({"ok":true})]])).collect());
    let conn = fake.connection();
    let run = |result: t3_protocol::Result<()>| assert!(result.is_ok() || result.unwrap_err() == "stream_closed");
    // Stops after the first item, like `first` above.
    let once = || {
        let done = std::rc::Rc::new(Cell::new(false));
        let seen = done.clone();
        (move || done.get(), move |item: Option<Value>| {
            if item.is_some() {
                seen.set(true);
            }
            Ok(())
        })
    };
    macro_rules! sub {
        ($f:ident($($arg:expr),*)) => {{ let (cancel, receive) = once(); run($f(&conn, $($arg,)* cancel, || false, receive)); }};
    }
    let shell = ShellSub { after_sequence: Some(41), request_completion_marker: Some(true) };
    sub!(subscribe_shell(&shell));
    let thread = ThreadSub {
        after_sequence: Some(SAFE_MAX),
        request_completion_marker: Some(true),
        accept_bounded_snapshot: Some(true),
        accept_compact_turn_items: Some(true),
        ..ThreadSub::new("t-1")
    };
    sub!(subscribe_thread(&thread));
    sub!(subscribe_command_output("t-1", "item:1"));
    let mut env = serde_json::Map::new();
    env.insert("TERM".into(), json!("xterm-256color"));
    env.insert("_X1".into(), json!(""));
    let attach = TerminalAttach {
        cwd: Some("/r".into()),
        worktree_path: Some("/r/wt".into()),
        cols: Some(80),
        rows: Some(24),
        env: Some(env),
        provider_instance_id: Some("codex".into()),
        restart_if_not_running: Some(true),
        resume: Some(true),
        ..TerminalAttach::new("t-1", "term 1")
    };
    sub!(subscribe_terminal_attach(&attach));
    sub!(subscribe_device_state());
    let urls = [" http://localhost:3000 ".to_string(), "http://127.0.0.1:8080/app".to_string()];
    sub!(subscribe_local_servers(Some(&urls[..])));
    sub!(subscribe_preview_events());
    let config = ConfigSub { environment_themes: Some(true), usage_limit_sources: Some(false), usage_limits_command: Some(true) };
    sub!(subscribe_server_config(&config));
    sub!(subscribe_worktree_setup("t-1"));
    sub!(subscribe_storage_cleanup_report());
    sub!(subscribe_provider_install("codex"));
    sub!(subscribe_provider_auth("claude_2"));
    let action = StackedAction {
        action_id: "act-1".into(),
        cwd: "/r".into(),
        action: "commit_push_pr".into(),
        commit_message: Some("  Fix it  ".into()),
        feature_branch: Some(true),
        file_paths: Some(vec!["src/a.rs".into(), "docs/b c.md".into()]),
        thread_id: Some("t-1".into()),
        project_id: Some("p-1".into()),
    };
    sub!(subscribe_stacked_action(&action));
    let recorded = fake.finish();
    let expected = rows.iter().map(|(name, _, payload)| json!({"tag":api::function(name).unwrap().tag,"payload":payload})).collect::<Vec<_>>();
    assert_eq!(wire(&recorded), expected);
}

#[test]
fn stream_arguments_are_rejected_before_any_socket() {
    let fake = FakeT3::spawn(vec![]);
    let conn = fake.connection();
    let cases = [
        ("orchestration.subscribeThread", json!({}), "invalid_thread_id"),
        ("orchestration.subscribeThread", json!({"threadId":"../t"}), "invalid_thread_id"),
        ("orchestration.subscribeThread", json!({"threadId":"t","afterSequence":-1}), "invalid_after_sequence"),
        ("orchestration.subscribeThread", json!({"threadId":"t","afterSequence":"7"}), "invalid_after_sequence"),
        ("orchestration.subscribeThread", json!({"threadId":"t","afterSequence":SAFE_MAX + 1}), "invalid_after_sequence"),
        ("orchestration.subscribeThread", json!({"threadId":"t","acceptBoundedSnapshot":"true"}), "invalid_accept_bounded_snapshot"),
        ("orchestration.subscribeThread", json!({"threadId":"t","afterSeq":1}), "invalid_input"),
        ("orchestration.subscribeShell", json!({"requestCompletionMarker":1}), "invalid_request_completion_marker"),
        ("orchestration.subscribeShell", json!([]), "invalid_input"),
        ("orchestration.subscribeCommandOutput", json!({"threadId":"t"}), "invalid_item_id"),
        ("terminal.attach", json!({"threadId":"t","terminalId":"x".repeat(129)}), "invalid_terminal_id"),
        ("terminal.attach", json!({"threadId":"t","terminalId":"a","cols":0}), "invalid_cols"),
        ("terminal.attach", json!({"threadId":"t","terminalId":"a","rows":501}), "invalid_rows"),
        ("terminal.attach", json!({"threadId":"t","terminalId":"a","env":{"1X":"a"}}), "invalid_env"),
        ("terminal.attach", json!({"threadId":"t","terminalId":"a","env":{"A":1}}), "invalid_env"),
        ("terminal.attach", json!({"threadId":"t","terminalId":"a","env":[]}), "invalid_env"),
        ("terminal.attach", json!({"threadId":"t","terminalId":"a","providerInstanceId":"9x"}), "invalid_provider_instance_id"),
        ("terminal.attach", json!({"threadId":"t","terminalId":"a","cwd":"\u{7}"}), "invalid_cwd"),
        ("preview.subscribeDiscoveredLocalServers", json!({"configuredUrls":vec!["http://a"; 33]}), "invalid_configured_urls"),
        ("preview.subscribeDiscoveredLocalServers", json!({"configuredUrls":[" "]}), "invalid_configured_urls"),
        ("preview.subscribeDiscoveredLocalServers", json!({"configuredUrls":"http://a"}), "invalid_configured_urls"),
        ("server.subscribeServerConfig", json!({"environmentThemes":"yes"}), "invalid_environment_themes"),
        ("worktreeSetup.subscribeWorktreeSetup", json!({"threadId":""}), "invalid_thread_id"),
        ("provider.install.subscribe", json!({"instanceId":"bad id"}), "invalid_instance_id"),
        ("provider.auth.subscribe", json!({}), "invalid_instance_id"),
        ("device.subscribeDeviceState", json!({"x":1}), "invalid_input"),
        ("git.runStackedAction", json!({"cwd":"/r","action":"commit"}), "invalid_action_id"),
        ("git.runStackedAction", json!({"actionId":"a","cwd":"/r","action":"rebase"}), "invalid_action"),
        ("git.runStackedAction", json!({"actionId":"a","cwd":"/r","action":"commit","filePaths":[]}), "invalid_file_paths"),
        ("git.runStackedAction", json!({"actionId":"a","cwd":"/r","action":"commit","filePaths":[""]}), "invalid_file_paths"),
        ("git.runStackedAction", json!({"actionId":"a","cwd":"/r","action":"commit","commitMessage":"m".repeat(10_001)}), "invalid_commit_message"),
        ("git.runStackedAction", json!({"actionId":"a","cwd":"/r","action":"commit","threadId":"t t"}), "invalid_thread_id"),
        ("vcs.refreshStatus", json!({"cwd":"/r"}), "not_a_stream"),
        ("no.such.stream", json!({}), "unknown_function"),
    ];
    for (name, args, error) in cases {
        assert_eq!(spec(name, &args).unwrap_err(), error, "{name} {args}");
    }
    // The typed Rust entry points share the same checks.
    let never = |_: Option<Value>| -> t3_protocol::Result<()> { panic!("a rejected stream delivered data") };
    assert_eq!(subscribe_thread(&conn, &ThreadSub::new(""), || false, || false, never).unwrap_err(), "invalid_thread_id");
    assert_eq!(subscribe_provider_auth(&conn, "-x", || false, || false, never).unwrap_err(), "invalid_instance_id");
    // A stream is never a unary call.
    for name in ["orchestration.subscribeShell", "git.runStackedAction"] {
        let f = api::function(name).unwrap();
        assert!(f.stream);
        let args = if name == "git.runStackedAction" { json!({"actionId":"a","cwd":"/r","action":"commit"}) } else { json!({}) };
        assert_eq!(api::call(&conn, name, &args, None).unwrap_err(), "stream_function");
    }
    assert_eq!(fake.finish().connections, 0, "validation happens before any socket");
}

#[test]
fn a_bounded_thread_spec_gets_the_large_item_limit_and_nothing_else_does() {
    let bounded = spec("orchestration.subscribeThread", &json!({"threadId":"t","acceptBoundedSnapshot":true})).unwrap();
    assert_eq!(bounded.item_limit(), 64 << 20);
    for (name, args) in [
        ("orchestration.subscribeThread", json!({"threadId":"t","acceptBoundedSnapshot":false})),
        ("orchestration.subscribeThread", json!({"threadId":"t"})),
        ("orchestration.subscribeShell", json!({})),
    ] {
        assert_eq!(spec(name, &args).unwrap().item_limit(), 8 << 20, "{name} {args}");
    }
}

#[test]
fn first_item_reads_return_the_current_state_and_stop() {
    let servers = json!({"servers":[{"url":"http://localhost:3000","port":3000}],"scannedAt":2});
    let fake = FakeT3::spawn(vec![
        Reply::Chunks(vec![vec![json!({"status":"installed","version":"1.2"})]]),
        Reply::Chunks(vec![vec![Value::Null]]),
        Reply::Chunks(vec![vec![json!({"servers":[],"scannedAt":1})], vec![servers.clone()]]),
        Reply::Chunks(vec![vec![json!({"servers":[],"scannedAt":1})]]),
        Reply::Chunks(vec![vec![json!({"status":"signed-in"})]]),
    ]);
    let conn = fake.connection();
    assert_eq!(api::call(&conn, "provider.installState", &json!({"instanceId":"codex"}), None).unwrap(), json!({"status":"installed","version":"1.2"}));
    // No report yet is a JSON null, not a timeout.
    assert_eq!(storage_cleanup_report(&conn, None).unwrap(), Value::Null);
    // The first list may precede the scan: wait for one with servers.
    assert_eq!(discovered_servers(&conn, None, None).unwrap(), servers);
    // A stream that ends first gives the last list it sent.
    let start = Instant::now();
    assert_eq!(api::call(&conn, "preview.discoveredServers", &json!({}), None).unwrap(), json!({"servers":[],"scannedAt":1}));
    assert!(start.elapsed() < Duration::from_secs(2), "{:?}", start.elapsed());
    assert_eq!(auth_state(&conn, "claude", Some(Duration::from_secs(5))).unwrap(), json!({"status":"signed-in"}));
    let recorded = fake.finish();
    assert_eq!(
        wire(&recorded),
        vec![
            json!({"tag":"provider.install.subscribe","payload":{"instanceId":"codex"}}),
            json!({"tag":"server.getStorageCleanupReport","payload":{}}),
            json!({"tag":"subscribeDiscoveredLocalServers","payload":{}}),
            json!({"tag":"subscribeDiscoveredLocalServers","payload":{}}),
            json!({"tag":"provider.auth.subscribe","payload":{"instanceId":"claude"}}),
        ]
    );
    assert_eq!(recorded.connections, 5);
}

#[test]
fn a_first_item_read_without_an_item_times_out_or_reports_the_end() {
    let fake = FakeT3::spawn(vec![Reply::Hang, Reply::Fail(json!([{"_tag":"Fail","error":{"_tag":"ProviderSetupError"}}]))]);
    let conn = fake.connection();
    let start = Instant::now();
    let silent = api::call(&conn, "provider.authState", &json!({"instanceId":"codex"}), Some(Duration::from_millis(300)));
    assert_eq!(silent.unwrap_err(), "stream_timeout");
    assert!(start.elapsed() >= Duration::from_millis(300) && start.elapsed() < Duration::from_secs(2), "{:?}", start.elapsed());
    assert_eq!(install_state(&conn, "codex", None).unwrap_err(), "stream_closed");
    assert_eq!(fake.finish().connections, 2);
    // The bare helper: `Ok(None)` when nothing arrived in time.
    let fake = FakeT3::spawn(vec![Reply::Hang]);
    let conn = fake.connection();
    let spec = provider_install_spec("codex").unwrap();
    assert_eq!(t3_protocol::stream::first_item(&conn, &spec, Duration::from_millis(200), |_| true).unwrap(), None);
    fake.finish();
}

#[test]
fn describe_lists_every_stream_with_its_shape() {
    let d = api::describe();
    let functions = d["functions"].as_array().unwrap();
    let streams = d["streams"].as_array().unwrap();
    let flagged = functions.iter().filter(|f| f["stream"] == true).map(|f| f["fn"].clone()).collect::<Vec<_>>();
    assert_eq!(streams.iter().map(|s| s["fn"].clone()).collect::<Vec<_>>(), flagged);
    assert_eq!(SHAPES.len(), 13);
    let shape = |name: &str| streams.iter().find(|s| s["fn"] == name).unwrap().clone();
    assert_eq!(
        shape("orchestration.subscribeThread"),
        json!({"fn":"orchestration.subscribeThread","tag":"orchestration.subscribeThread","itemKey":"kind",
            "items":["synchronized","snapshot","event"],"starts":["snapshot","resume"],"cursor":"afterSequence","ends":[],"firstItem":null})
    );
    assert_eq!(shape("git.runStackedAction")["ends"], json!(["action_finished", "action_failed"]));
    assert_eq!(shape("provider.install.subscribe")["firstItem"], "provider.installState");
    assert_eq!(shape("device.subscribeDeviceState")["itemKey"], Value::Null);
    // Every first-item read is a registered, non-stream function with no wire tag.
    for s in streams.iter().filter(|s| !s["firstItem"].is_null()) {
        let read = api::function(s["firstItem"].as_str().unwrap()).unwrap();
        assert!(!read.stream && read.tag.is_none(), "{}", read.name);
    }
    let stacked = functions.iter().find(|f| f["fn"] == "git.runStackedAction").unwrap();
    assert_eq!((&stacked["replay"], &stacked["scopes"]), (&json!("never"), &json!(["source-control:write", "orchestration:read"])));
}
