//! Every typed `Command` variant: contract JSON → `Command` → the exact payload,
//! the wrappers around the existing builders, and the checks that run before
//! a command can be dispatched.
#[path = "support/fake_t3.rs"]
mod fake_t3;
use fake_t3::{wire, FakeT3, Reply};
use serde_json::{json, Value};
use t3_protocol::api::orchestration::*;

const C: &str = "omarchy-c1";
fn ids(kind: &str) -> Value {
    json!({"type":kind,"commandId":C,"threadId":"t-1"})
}
fn with(mut base: Value, extra: Value) -> Value {
    for (k, v) in extra.as_object().unwrap() {
        base[k] = v.clone();
    }
    base
}
/// Contract-shaped commands whose payload must come back unchanged.
fn identical() -> Vec<Value> {
    let pr = json!({"host":"github.com","repository":"o/r","number":7});
    vec![
        with(ids("thread.create"), json!({"projectId":"p-1","title":"New","modelSelection":{"instanceId":"codex","model":"m"},"runtimeMode":"auto","interactionMode":"plan","branch":null,"worktreePath":"/w","createdBy":"user","creationSource":"web"})),
        with(ids("message.dispatch"), json!({"messageId":"omarchy-m1","text":"hi","attachments":[],"createdBy":"user","creationSource":"web","dispatchMode":{"type":"queue_after_active"},"sourcePlanRef":{"threadId":"t-0","planId":"plan-1"},"deliveryIntent":"steer"})),
        with(ids("message.dispatch"), json!({"messageId":"omarchy-m2","text":"go","attachments":[{"type":"image","id":"f_1","name":"a.png","mimeType":"image/png","sizeBytes":3}],"createdBy":"user","creationSource":"mobile","dispatchMode":{"type":"defer_start","workspaceStrategy":{"type":"root","branch":"main"}}})),
        with(ids("message.dispatch"), json!({"messageId":"omarchy-m3","text":"now","attachments":[],"createdBy":"user","creationSource":"web","dispatchMode":{"type":"steer_active","targetRunId":"r-1"}})),
        with(ids("run.interrupt"), json!({"runId":"r-1","holdQueue":true})),
        with(ids("subagent.stop"), json!({"runId":"r-1","subagentId":"n-1"})),
        with(ids("thread.metadata.update"), json!({"title":"Renamed","branch":null,"worktreePath":"/w","expectedEmpty":true,"linkedPullRequest":null})),
        with(ids("thread.runtime-mode.set"), json!({"runtimeMode":"approval-required"})),
        with(ids("thread.interaction-mode.set"), json!({"interactionMode":"plan"})),
        json!({"type":"thread.fork","commandId":C,"sourceThreadId":"t-1","targetThreadId":"t-2","sourcePoint":{"type":"run","runId":"r-1"},"title":"Fork","createdBy":"user","creationSource":"web"}),
        json!({"type":"thread.merge_back","commandId":C,"sourceThreadId":"t-2","targetThreadId":"t-1","sourcePoint":{"type":"latest_stable"},"createdAt":"2026-10-24T10:00:00Z","createdBy":"user","creationSource":"web"}),
        ids("thread.archive"),
        ids("thread.unarchive"),
        ids("thread.delete"),
        ids("thread.settle"),
        with(ids("thread.settle"), json!({"settledAt":"2026-10-24T10:00:00.000Z"})),
        with(ids("thread.unsettle"), json!({"reason":"user"})),
        with(ids("thread.unsnooze"), json!({"reason":"user"})),
        with(ids("thread.auto-settle"), json!({"snapshotAt":"2026-10-24T10:00:00Z"})),
        with(ids("thread.auto-settle.set"), json!({"enabled":false})),
        with(ids("thread.pin"), json!({"orderKey":"a0"})),
        ids("thread.unpin"),
        with(ids("thread.pin.reorder"), json!({"orderKey":"a1"})),
        ids("thread.mark-unread"),
        with(ids("thread.visit"), json!({"visitedAt":"2026-10-24T10:00:00Z"})),
        with(ids("queued-run.cancel"), json!({"runId":"r-2"})),
        with(ids("queued-run.reorder"), json!({"runId":"r-2","beforeRunId":null})),
        with(ids("queued-run.reorder"), json!({"runId":"r-2","beforeRunId":"r-1"})),
        with(ids("queued-run.edit"), json!({"runId":"r-2","text":"edited","attachments":[]})),
        with(ids("queued-message.promote-to-steer"), json!({"queuedRunId":"r-2","targetRunId":"r-1"})),
        ids("queue.resume"),
        with(ids("runtime-request.respond"), json!({"requestId":"call_1%253Aa","decision":"acceptForSession"})),
        with(ids("runtime-request.respond"), json!({"requestId":"q-1","answers":{"color":"blue","tags":["a","b"]}})),
        with(ids("thread.user-input.dismiss"), json!({"requestId":"q-1"})),
        with(ids("checkpoint.rollback"), json!({"scopeId":"s-1","checkpointId":"cp-1","restoreFiles":true})),
        with(ids("thread.pull-request.link"), with(pr.clone(), json!({"url":"https://github.com/o/r/pull/7","source":"manual"}))),
        with(ids("thread.pull-request.unlink"), pr.clone()),
        with(ids("thread.pull-request.watch"), with(pr, json!({"watching":true,"link":{"url":"https://github.com/o/r/pull/7","source":"manual"}}))),
    ]
}

#[test]
fn every_typed_variant_round_trips_to_the_contract_shape() {
    let mut types = vec![];
    for command in identical() {
        let typed = Command::from_json(&command).unwrap_or_else(|e| panic!("{e}: {command}"));
        assert!(!matches!(typed, Command::Raw(_)), "{command}");
        assert_eq!(typed.payload().unwrap(), command);
        assert_eq!(typed.command_id(), Some(C));
        types.push(command["type"].as_str().unwrap().to_string());
    }
    // Snooze normalizes its time; it is the only typed variant that rewrites a value.
    let snooze = with(ids("thread.snooze"), json!({"snoozedUntil":"2099-01-01T00:00:00Z"}));
    assert_eq!(Command::from_json(&snooze).unwrap().payload().unwrap()["snoozedUntil"], "2099-01-01T00:00:00+00:00");
    types.push("thread.snooze".into());
    types.sort();
    types.dedup();
    let mut expected = COMMAND_TYPES.iter().map(|t| t.to_string()).collect::<Vec<_>>();
    expected.sort();
    assert_eq!(types, expected, "the table covers every typed variant");
}

#[test]
fn wrappers_reuse_the_existing_builders() {
    let q = Command::queue(C, &json!({"action":"steer","thread":"t-1","runId":"r-2","targetRunId":"r-1"})).unwrap();
    assert_eq!(q.payload().unwrap(), with(ids("queued-message.promote-to-steer"), json!({"queuedRunId":"r-2","targetRunId":"r-1"})));
    let e = Command::queue(C, &json!({"action":"edit","thread":"t-1","runId":"r-2","text":"see ![a.png](t3-context://v1/image/f_1)","attachments":[{"type":"image","id":"f_1","name":"a.png"}]})).unwrap();
    assert_eq!(e.payload().unwrap()["text"], "see a.png");
    let a = Command::answer(C, &json!({"action":"respond","thread":"t-1","runtimeRequestId":"q-1","answers":{"x":"y"}})).unwrap();
    assert_eq!(a.payload().unwrap(), with(ids("runtime-request.respond"), json!({"requestId":"q-1","answers":{"x":"y"}})));
    let d = Command::answer(C, &json!({"action":"dismiss","thread":"t-1","runtimeRequestId":"q-1"})).unwrap();
    assert_eq!(d.payload().unwrap(), with(ids("thread.user-input.dismiss"), json!({"requestId":"q-1"})));
    let projection = json!({"runtimeRequests":[{"id":"req-1","status":"pending","kind":"command","createdAt":"1","responseCapability":{"type":"live"}}],
        "turnItems":[{"type":"approval_request","requestId":"req-1","options":[{"decision":"accept"},{"decision":"decline"}]}]});
    let ok = Command::approve(C, &json!({"thread":"t-1","runtimeRequestId":"req-1","decision":"accept"}), &projection);
    assert_eq!(ok.unwrap().payload().unwrap(), with(ids("runtime-request.respond"), json!({"requestId":"req-1","decision":"accept"})));
    assert_eq!(Command::approve(C, &json!({"thread":"t-1","runtimeRequestId":"gone","decision":"accept"}), &projection).unwrap_err(), "approval_no_longer_pending");
    let m = Command::runtime_mode(C, "t-1", "auto").unwrap();
    assert_eq!(m.payload().unwrap(), with(ids("thread.runtime-mode.set"), json!({"runtimeMode":"auto"})));
    assert_eq!(Command::runtime_mode(C, "t-1", "yolo").unwrap_err(), "invalid_runtime_mode");
    let s = Command::sidebar(C, "unsettle", "t-1", None).unwrap();
    assert_eq!(s.payload().unwrap(), with(ids("thread.unsettle"), json!({"reason":"user"})));
    assert!(Command::sidebar(C, "snooze", "t-1", Some("2000-01-01T00:00:00Z")).is_err());
    let r = Command::thread_action(C, &json!({"action":"thread-rename","thread":"t-1","title":" Better "})).unwrap();
    assert_eq!(r.payload().unwrap(), with(ids("thread.metadata.update"), json!({"title":"Better"})));
    assert_eq!(Command::thread_action(C, &json!({"action":"delete-thread","thread":"t-1"})).unwrap_err(), "confirmation_required");
}

#[test]
fn invalid_commands_are_rejected_before_dispatch() {
    let cases = [
        (with(ids("thread.archive"), json!({"commandId":""})), "invalid_command_id"),
        (with(ids("thread.archive"), json!({"threadId":"a/b"})), "invalid_thread_id"),
        (with(ids("thread.archive"), json!({"title":"x"})), "invalid_input"),
        (with(ids("thread.create"), json!({"projectId":"p","title":"t","modelSelection":{},"runtimeMode":"auto","interactionMode":"default","worktreePath":null,"createdBy":"user","creationSource":"web"})), "invalid_branch"),
        (with(ids("message.dispatch"), json!({"messageId":"m","text":"  ","attachments":[],"createdBy":"user","creationSource":"web","dispatchMode":{"type":"start_immediately"}})), "invalid_text"),
        (with(ids("message.dispatch"), json!({"messageId":"m","text":"x","attachments":[],"createdBy":"robot","creationSource":"web","dispatchMode":{"type":"start_immediately"}})), "invalid_created_by"),
        (with(ids("message.dispatch"), json!({"messageId":"m","text":"x","attachments":[],"createdBy":"user","creationSource":"web","dispatchMode":{"type":"later"}})), "invalid_dispatch_mode"),
        (with(ids("thread.visit"), json!({"visitedAt":"yesterday"})), "invalid_visited_at"),
        (with(ids("thread.unsettle"), json!({"reason":"bot"})), "invalid_reason"),
        (with(ids("runtime-request.respond"), json!({"requestId":" q"})), "invalid_input"),
        (with(ids("runtime-request.respond"), json!({"requestId":" q","decision":"accept"})), "invalid_request_id"),
        (with(ids("runtime-request.respond"), json!({"requestId":"q","decision":"maybe"})), "invalid_approval_decision"),
        (with(ids("thread.pull-request.unlink"), json!({"host":"h","repository":"o/r","number":0})), "invalid_number"),
        (with(ids("thread.pull-request.link"), json!({"host":"h","repository":"o/r","number":1,"url":"u","source":"bot"})), "invalid_source"),
        (with(ids("thread.fork"), json!({})), "invalid_source_thread_id"),
        (json!({"type":"thread.fork","commandId":C,"sourceThreadId":"a","targetThreadId":"b","sourcePoint":{"type":"later"},"createdBy":"user","creationSource":"web"}), "invalid_source_point"),
        (json!({"type":"no.such.command","commandId":C}), "invalid_type"),
        (json!({"commandId":C}), "invalid_type"),
        (json!([]), "invalid_input"),
    ];
    for (command, error) in cases {
        assert_eq!(Command::from_json(&command).unwrap_err(), error, "{command}");
    }
    let built = Command::ThreadPinReorder { command_id: C.into(), thread_id: "t-1".into(), order_key: "  ".into() };
    assert_eq!(built.payload().unwrap_err(), "invalid_order_key");
}

#[test]
fn raw_carries_untyped_contract_commands_and_dispatch_sends_once() {
    for kind in RAW_COMMAND_TYPES {
        let raw = json!({"type":kind,"commandId":C,"threadId":"t-1","anything":{"x":1}});
        let command = Command::from_json(&raw).unwrap();
        assert!(matches!(command, Command::Raw(_)));
        assert_eq!(command.payload().unwrap(), raw);
    }
    assert_eq!(Command::from_json(&json!({"type":"thread.stop","commandId":"bad id"})).unwrap_err(), "invalid_command_id");
    assert!(COMMAND_TYPES.iter().all(|t| !RAW_COMMAND_TYPES.contains(t)));

    let fake = FakeT3::spawn(vec![Reply::Ok(json!({"sequence":42}))]);
    let mut message = MessageDispatch::new(C, "t-1", "omarchy-m1", "hello", "mobile");
    message.dispatch_mode = DispatchMode::QueueAfterActive;
    let sent = dispatch(&fake.connection(), &Command::MessageDispatch(message)).unwrap();
    assert_eq!(sent, json!({"sequence":42}));
    let recorded = fake.finish();
    assert_eq!(
        wire(&recorded),
        vec![json!({"tag":"orchestration.dispatchCommand","payload":{"type":"message.dispatch","commandId":C,"threadId":"t-1","messageId":"omarchy-m1",
            "text":"hello","attachments":[],"createdBy":"user","creationSource":"mobile","dispatchMode":{"type":"queue_after_active"}}})]
    );
    assert_eq!(recorded.connections, 1);
}
