//! Error taxonomy of the typed API against a fake server: authorization by
//! `_tag` with its scope, auth-sounding git failures as plain rejections, and
//! `rpc_outcome_unknown` after one request, never followed by a retry.
#[path = "support/fake_t3.rs"]
mod fake_t3;
use fake_t3::{fail, FakeT3, Reply};
use serde_json::json;
use t3_protocol::{
    api::{self, orchestration::Command, Kind},
    error_code, errors,
};

#[test]
fn missing_scope_is_write_not_authorized_with_the_required_scope() {
    let fake = FakeT3::spawn(vec![Reply::Fail(fail(json!({"_tag":"EnvironmentAuthorizationError",
        "message":"The authenticated token is missing required scope: settings:write.","requiredScope":"settings:write"})))]);
    let error = api::server::update_settings(&fake.connection(), &json!({"enableProviderUpdateChecks":true}), None).unwrap_err();
    fake.finish();
    assert_eq!(error_code(&error), "write_not_authorized");
    assert_eq!(errors::required_scope(&error), Some("settings:write"));
    assert_eq!(errors::message(&error), Some("The authenticated token is missing required scope: settings:write."));
}

#[test]
fn auth_words_in_a_git_failure_stay_rpc_rejected() {
    // Regression: the old classifier matched "Auth" anywhere in the cause.
    let causes = [
        fail(json!({"_tag":"GitCommandError","message":"Author identity unknown\n\n*** Please tell me who you are."})),
        fail(json!({"_tag":"VcsError","message":"Authentication failed for 'https://github.com/o/r.git/'"})),
    ];
    let fake = FakeT3::spawn(causes.into_iter().map(Reply::Fail).collect());
    let conn = fake.connection();
    let first = api::vcs::refresh_status(&conn, "/r").unwrap_err();
    let second = api::call(&conn, "orchestration.dispatchCommand", &json!({"type":"thread.archive","commandId":"c-1","threadId":"t-1"}), None).unwrap_err();
    fake.finish();
    assert_eq!(first, "rpc_rejected: Author identity unknown");
    assert_eq!(second, "rpc_rejected: Authentication failed for 'https://github.com/o/r.git/'");
    assert_eq!(errors::required_scope(&first), None);
}

#[test]
fn outcome_unknown_after_sending_is_reported_once_for_each_kind() {
    // One representative per kind tanda 1 has: L, M·id, M·set.
    let cases = [
        ("server.getConfig", json!({}), Kind::Read),
        ("orchestration.dispatchCommand", json!({"type":"thread.archive","commandId":"c-1","threadId":"t-1"}), Kind::MutationWithCommandId),
        ("server.updateSettings", json!({"patch":{"enableProviderUpdateChecks":true}}), Kind::IdempotentMutation),
    ];
    for (name, args, kind) in cases {
        assert_eq!(api::function(name).unwrap().kind, kind);
        let fake = FakeT3::spawn(vec![Reply::DropAfterRequest]);
        assert_eq!(api::call(&fake.connection(), name, &args, None).unwrap_err(), "rpc_outcome_unknown", "{name}");
        let recorded = fake.finish();
        assert_eq!(recorded.requests.len(), 1, "{name}: exactly one request");
        assert_eq!(recorded.connections, 1, "{name}: no retry");
    }
    // The same through a typed signature and a caller deadline (rpc_with_timeout).
    let fake = FakeT3::spawn(vec![Reply::DropAfterRequest]);
    let command = Command::ThreadArchive { command_id: "c-1".into(), thread_id: "t-1".into() };
    assert_eq!(api::orchestration::dispatch(&fake.connection(), &command).unwrap_err(), "rpc_outcome_unknown");
    assert_eq!(fake.finish().connections, 1);
    let fake = FakeT3::spawn(vec![Reply::DropAfterRequest]);
    let f = api::function("vcs.refreshStatus").unwrap();
    assert_eq!(f.call(&fake.connection(), &json!({"cwd":"/r"}), Some(std::time::Duration::from_secs(2))).unwrap_err(), "rpc_outcome_unknown");
    assert_eq!(fake.finish().requests.len(), 1);
}
