//! One row per typed function: JSON args → exact tag and payload on the wire,
//! canned response → output. Then the same calls through the Rust signatures,
//! and every local rejection without a single socket.
#[path = "support/fake_t3.rs"]
mod fake_t3;
use fake_t3::{wire, FakeT3, Reply};
use serde_json::{json, Value};
use t3_protocol::api::{self, assets::AssetResource, orchestration::*, review::*, server::ProviderInstanceMutation};

fn launch_args() -> Value {
    json!({"commandId":"omarchy-l1","projectId":"p-1","title":"  Fix the build  ","modelSelection":{"instanceId":"codex","model":"gpt-5"},
        "runtimeMode":"full-access","interactionMode":"default","workspaceStrategy":{"type":"worktree","baseRef":"main","startFromOrigin":true},
        "creationSource":"mobile","initialMessage":{"messageId":"omarchy-m1","text":"hello","attachments":[]}})
}
/// (function, args, payload on the wire). The tag is always the function name in tanda 1.
fn rows() -> Vec<(&'static str, Value, Value)> {
    let mut launch = launch_args();
    launch["title"] = json!("Fix the build");
    vec![
        ("server.getConfig", json!({}), json!({})),
        ("server.getSettings", json!({}), json!({})),
        ("server.updateSettings", json!({"patch":{"sidebarAutoSettleAfterDays":3}}), json!({"patch":{"sidebarAutoSettleAfterDays":3}})),
        (
            "server.updateSettings",
            json!({"patch":{},"providerInstanceMutation":{"operation":"remove","instanceId":"codex_2"}}),
            json!({"patch":{},"providerInstanceMutation":{"operation":"remove","instanceId":"codex_2"}}),
        ),
        (
            "orchestration.dispatchCommand",
            json!({"type":"thread.archive","commandId":"omarchy-1","threadId":"t-1"}),
            json!({"type":"thread.archive","commandId":"omarchy-1","threadId":"t-1"}),
        ),
        ("orchestration.launchThread", launch_args(), launch),
        ("preview.list", json!({"threadId":"t-1"}), json!({"threadId":"t-1"})),
        (
            "assets.createUrl",
            json!({"resource":{"_tag":"workspace-file","threadId":"t-1","path":"README.md"}}),
            json!({"resource":{"_tag":"workspace-file","threadId":"t-1","path":"README.md"}}),
        ),
        ("vcs.refreshStatus", json!({"cwd":"/r"}), json!({"cwd":"/r"})),
        (
            "review.getDiffPreview",
            json!({"cwd":"/r","baseRef":" main ","ignoreWhitespace":true,"file":{"path":"src/a.rs","previousPath":null,"sourceKind":"branch-range"}}),
            json!({"cwd":"/r","baseRef":"main","ignoreWhitespace":true,"file":{"path":"src/a.rs","previousPath":null,"sourceKind":"branch-range"}}),
        ),
        (
            "projects.readFile",
            json!({"cwd":"/r","relativePath":"docs/x.md","allowMissing":true}),
            json!({"cwd":"/r","relativePath":"docs/x.md","allowMissing":true}),
        ),
    ]
}

#[test]
fn every_function_sends_its_exact_tag_and_payload_once() {
    let rows = rows();
    let fake = FakeT3::spawn(rows.iter().enumerate().map(|(i, _)| Reply::Ok(json!({"row":i}))).collect());
    let conn = fake.connection();
    for (i, (name, args, _)) in rows.iter().enumerate() {
        assert_eq!(api::call(&conn, name, args, None).unwrap(), json!({"row":i}), "{name}");
    }
    let recorded = fake.finish();
    let expected = rows.iter().map(|(name, _, payload)| json!({"tag":name,"payload":payload})).collect::<Vec<_>>();
    assert_eq!(wire(&recorded), expected);
    assert_eq!(recorded.connections, rows.len(), "one socket per call, no retries");
    // Every registered function has a row.
    let mut covered = rows.iter().map(|(name, _, _)| *name).collect::<Vec<_>>();
    covered.dedup();
    assert_eq!(covered, api::functions().map(|f| f.name).collect::<Vec<_>>());
}

#[test]
fn rust_signatures_put_the_same_requests_on_the_wire() {
    let rows = rows();
    let fake = FakeT3::spawn((0..rows.len()).map(|_| Reply::Ok(json!({"ok":true}))).collect());
    let conn = fake.connection();
    api::server::config(&conn).unwrap();
    api::server::settings(&conn).unwrap();
    api::server::update_settings(&conn, &json!({"sidebarAutoSettleAfterDays":3}), None).unwrap();
    api::server::update_settings(&conn, &json!({}), Some(&ProviderInstanceMutation::Remove { instance_id: "codex_2".into() })).unwrap();
    api::orchestration::dispatch(&conn, &Command::ThreadArchive { command_id: "omarchy-1".into(), thread_id: "t-1".into() }).unwrap();
    let launch = ThreadLaunch {
        command_id: "omarchy-l1".into(),
        project_id: "p-1".into(),
        title: "  Fix the build  ".into(),
        model_selection: json!({"instanceId":"codex","model":"gpt-5"}),
        runtime_mode: "full-access".into(),
        interaction_mode: "default".into(),
        workspace_strategy: WorkspaceStrategy::Worktree { base_ref: "main".into(), branch: None, start_from_origin: Some(true) },
        creation_source: Some("mobile".into()),
        thread_id: None,
        reuse_existing_thread: None,
        generate_title: None,
        initial_message: Some(InitialMessage { message_id: Some("omarchy-m1".into()), text: "hello".into(), context: None, attachments: vec![] }),
    };
    api::orchestration::launch_thread(&conn, &launch).unwrap();
    api::preview::list(&conn, "t-1").unwrap();
    api::assets::create_url(&conn, &AssetResource::WorkspaceFile { thread_id: "t-1".into(), path: "README.md".into() }).unwrap();
    api::vcs::refresh_status(&conn, "/r").unwrap();
    let file = DiffFile { path: "src/a.rs".into(), previous_path: None, source_kind: SourceKind::BranchRange };
    api::review::diff_preview(&conn, "/r", Some(" main "), Some(true), Some(&file)).unwrap();
    api::projects::read_file(&conn, "/r", "docs/x.md", Some(true)).unwrap();
    let recorded = fake.finish();
    let expected = rows.iter().map(|(name, _, payload)| json!({"tag":name,"payload":payload})).collect::<Vec<_>>();
    assert_eq!(wire(&recorded), expected);
}

#[test]
fn local_rejections_never_open_a_socket() {
    let fake = FakeT3::spawn(vec![]);
    let conn = fake.connection();
    let long = "x".repeat(513);
    let cases = [
        ("vcs.refreshStatus", json!({"cwd":""}), "invalid_cwd"),
        ("vcs.refreshStatus", json!({}), "invalid_cwd"),
        ("vcs.refreshStatus", json!({"cwd":"/r","extra":1}), "invalid_input"),
        ("vcs.refreshStatus", json!("not an object"), "invalid_input"),
        ("preview.list", json!({"threadId":"../t"}), "invalid_thread_id"),
        ("orchestration.dispatchCommand", json!({"type":"thread.archive","commandId":"bad id","threadId":"t"}), "invalid_command_id"),
        ("orchestration.dispatchCommand", json!({"type":"thread.archive","threadId":"t"}), "invalid_command_id"),
        ("orchestration.dispatchCommand", json!({"type":"thread.archive","commandId":"c","threadId":"t","x":1}), "invalid_input"),
        ("orchestration.dispatchCommand", json!({"type":"thread.explode","commandId":"c"}), "invalid_type"),
        ("orchestration.launchThread", { let mut a = launch_args(); a["title"] = json!("   "); a }, "invalid_title"),
        ("orchestration.launchThread", { let mut a = launch_args(); a["runtimeMode"] = json!("yolo"); a }, "invalid_runtime_mode"),
        ("orchestration.launchThread", { let mut a = launch_args(); a["workspaceStrategy"] = json!({"type":"cloud"}); a }, "invalid_workspace_strategy"),
        ("orchestration.launchThread", { let mut a = launch_args(); a["commandId"] = json!(null); a }, "invalid_command_id"),
        ("server.updateSettings", json!({"patch":[]}), "invalid_patch"),
        ("server.updateSettings", json!({"patch":{},"providerInstanceMutation":{"operation":"remove","instanceId":"9bad"}}), "invalid_provider_instance_mutation"),
        ("assets.createUrl", json!({"resource":{"_tag":"attachment","attachmentId":" "}}), "invalid_attachment_id"),
        ("assets.createUrl", json!({"resource":{"_tag":"workspace-file","threadId":"t","path":"a","x":1}}), "invalid_input"),
        ("assets.createUrl", json!({"resource":{"_tag":"anything"}}), "invalid_resource"),
        ("review.getDiffPreview", json!({"cwd":"/r","file":{"path":"a"}}), "invalid_file"),
        ("review.getDiffPreview", json!({"cwd":"/r","baseRef":"  "}), "invalid_base_ref"),
        ("projects.readFile", json!({"cwd":"/r","relativePath":long}), "invalid_relative_path"),
        ("projects.readFile", json!({"cwd":"/r","relativePath":"a","allowMissing":"yes"}), "invalid_allow_missing"),
        ("no.such.function", json!({}), "unknown_function"),
    ];
    for (name, args, error) in cases {
        assert_eq!(api::call(&conn, name, &args, None).unwrap_err(), error, "{name} {args}");
    }
    assert_eq!(api::vcs::refresh_status(&conn, "\u{7}").unwrap_err(), "invalid_cwd");
    assert_eq!(api::preview::list(&conn, "").unwrap_err(), "invalid_thread_id");
    assert_eq!(api::projects::read_file(&conn, "/r", "", None).unwrap_err(), "invalid_relative_path");
    let recorded = fake.finish();
    assert_eq!(recorded.connections, 0, "validation happens before any socket");
}
