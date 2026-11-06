use serde_json::{json, Value};
use t3_protocol::quota_sources as quotas;
fn limits() -> Value {
    json!({"checkedAt":"2026-10-18T12:00:00Z","credentialFingerprint":"same-key","windows":[{"id":"monthly","kind":"monthly","label":"Monthly","usedPercent":30,"resetsAt":"2026-11-01T00:00:00Z"}],"secret":"PRIVATE"})
}
fn provider() -> Value {
    json!({"instanceId":"custom-go","driver":"opencode","enabled":true,"installed":true,"auth":{"status":"authenticated","email":"account@example.test","accessToken":"PRIVATE"},"usageLimits":limits(),"environment":{"OPENCODE_API_KEY":"PRIVATE"},"models":[{"id":"PRIVATE"}]})
}
#[test]
fn public_projection_is_forward_compatible_and_drops_credentials() {
    let out = quotas::providers(
        &json!([provider(),{"instanceId":"future","driver":"future-driver","enabled":true,"installed":true,"usageLimits":limits()}]),
    );
    assert_eq!(out.len(), 2);
    assert_eq!(out[1]["driver"], "future-driver");
    assert_eq!(out[0]["usageLimits"]["windows"][0]["kind"], "monthly");
    assert!(!json!(out).to_string().contains("PRIVATE"));
    let raw = json!([{"id":"hub","label":"Hub","managementKey":"PRIVATE","accounts":[{"id":"one","driver":"codex","email":"same@example.test","workspaceId":"workspace","usageLimits":limits(),"accessToken":"PRIVATE"}],"error":"PRIVATE"}]);
    let sources = quotas::sources(&raw);
    assert_eq!(sources[0]["accounts"][0]["workspaceId"], "workspace");
    assert!(!json!(sources).to_string().contains("PRIVATE"));
    let mut invalid = limits();
    invalid["windows"] = json!([{"id":"bad","kind":"weekly","label":"Bad","usedPercent":101},{"id":"ok","kind":"other","label":"Other","usedPercent":0,"resetsAt":"bad"},{"id":"future","kind":"future-kind","label":"Future","usedPercent":0}]);
    let safe = quotas::limits(&invalid).unwrap();
    assert_eq!(safe["windows"].as_array().unwrap().len(), 1);
    assert!(safe["windows"][0].get("resetsAt").is_none());
    invalid["externalUsage"] = json!({"label":"Usage","url":"https://secret@example.test/path"});
    assert!(quotas::limits(&invalid)
        .unwrap()
        .get("externalUsage")
        .is_none());
}
#[test]
fn snapshots_replace_sources_and_unrelated_events_do_not_mutate_state() {
    let mut state = json!({"usageLimitSources":[{"id":"old"}]});
    assert!(!quotas::apply(
        &mut state,
        &json!({"version":1,"type":"settingsUpdated","payload":{"secret":"PRIVATE"}})
    ));
    assert!(quotas::apply(
        &mut state,
        &json!({"version":1,"type":"snapshot","config":{"providers":[provider()]}})
    ));
    assert!(state["usageLimitSources"].as_array().unwrap().is_empty());
    assert!(quotas::apply(
        &mut state,
        &json!({"version":1,"type":"usageLimitSourcesUpdated","payload":{"sources":[{"id":"hub","accounts":[]}]}})
    ));
    assert_eq!(state["usageLimitSources"][0]["id"], "hub");
    assert!(quotas::apply(
        &mut state,
        &json!({"version":1,"type":"providerStatuses","payload":{"providers":[]}})
    ));
    assert!(state["providers"].as_array().unwrap().is_empty());
    assert_eq!(state["usageLimitSources"].as_array().unwrap().len(), 1);
}

#[test]
fn reset_credit_ids_follow_the_actual_native_optional_trimmed_string_schema() {
    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/reset-credit-values-4288.json")).unwrap();
    assert_eq!(
        fixture["revision"],
        "32a0531748c44b25072d7495ce0b9bd7e9450a87"
    );
    for case in fixture["cases"].as_array().unwrap() {
        let mut raw = limits();
        raw["resetCredits"] = case["input"].clone();
        let projected = quotas::limits(&raw);
        if case["ok"] == true {
            assert_eq!(projected.unwrap()["resetCredits"], case["value"]);
        } else {
            assert!(
                projected.is_none(),
                "Invalid native nextCreditId must not become a redeemable observation"
            );
        }
    }
}
#[test]
fn reset_credit_identity_survives_provider_and_source_observation_updates() {
    let mut raw_limits = limits();
    raw_limits["resetCredits"] = json!({"availableCount":2,"nextCreditId":"synthetic-credit-one"});
    let mut raw_provider = provider();
    raw_provider["usageLimits"] = raw_limits.clone();
    let source = json!({"id":"hub","label":"Hub","accounts":[{"id":"account","driver":"codex","usageLimits":raw_limits}]});
    let mut state = json!({});
    assert!(quotas::apply(
        &mut state,
        &json!({"version":1,"type":"snapshot","config":{"providers":[raw_provider],"usageLimitSources":[source]}})
    ));
    assert!(quotas::apply(
        &mut state,
        &json!({"version":1,"type":"usageLimitSourcesUpdated","payload":{"sources":[source.clone()]}})
    ));
    assert_eq!(
        state["providers"][0]["usageLimits"]["resetCredits"]["nextCreditId"],
        "synthetic-credit-one"
    );
    assert_eq!(
        state["usageLimitSources"][0]["accounts"][0]["usageLimits"]["resetCredits"]["nextCreditId"],
        "synthetic-credit-one"
    );
    let mut source = source;
    source["accounts"][0]["usageLimits"]["resetCredits"]["nextCreditId"] =
        json!("synthetic-credit-two");
    assert!(quotas::apply(
        &mut state,
        &json!({"version":1,"type":"usageLimitSourcesUpdated","payload":{"sources":[source]}})
    ));
    assert_eq!(
        state["usageLimitSources"][0]["accounts"][0]["usageLimits"]["resetCredits"]["nextCreditId"],
        "synthetic-credit-two"
    );
    assert!(!state.to_string().contains("PRIVATE"));
}
