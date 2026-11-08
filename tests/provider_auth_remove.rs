//! Exact native input and one-shot synthetic transport; no account store access.
#[path = "support/fake_t3.rs"]
mod fake_t3;
use fake_t3::{fail, wire, FakeT3, Reply};
use serde_json::{json, Value};
use t3_protocol::api::{
    self,
    provider::{
        auth_remove_method, auth_start, auth_start_method_id, AuthRemoveMethodInput,
        AUTH_REMOVE_METHOD, AUTH_START,
    },
};

#[test]
fn represented_inputs_match_original_schema_and_record_surrogate_ingress_gap() {
    let fixture: Value = serde_json::from_str(include_str!(
        "fixtures/provider-auth-remove-method-native-4340.json"
    ))
    .unwrap();
    assert_eq!(
        fixture["revision"],
        "a9e709f15f825cacac1b6d665fa1920c328d68b7"
    );
    assert_eq!(
        fixture["complete"], true,
        "Generate the original-schema oracle before accepting this proof"
    );
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 21);
    assert_eq!(fixture["nativeResultSha256"].as_str().unwrap().len(), 64);
    assert_eq!(fixture["exclusions"].as_array().unwrap().len(), 2);
    for case in fixture["exclusions"].as_array().unwrap() {
        // The separate whole native result retains these accepted/rejected
        // JavaScript cases unchanged. Generic Rust JSON ingress is the gap.
        assert!(serde_json::from_str::<Value>(case["rawInput"].as_str().unwrap()).is_err());
        assert_eq!(case["reason"], "generic_rust_json_isolated_surrogate");
    }
    for case in fixture["cases"].as_array().unwrap() {
        let typed =
            AuthRemoveMethodInput::from_json(&case["input"]).and_then(|input| input.payload());
        let raw = AUTH_REMOVE_METHOD.payload(&case["input"]);
        if case["ok"] == true {
            assert_eq!(typed.unwrap(), case["value"], "{}", case["name"]);
            assert_eq!(raw.unwrap(), case["value"], "{}", case["name"]);
        } else {
            assert!(typed.is_err(), "{}", case["name"]);
            assert!(raw.is_err(), "{}", case["name"]);
        }
    }
}

#[test]
fn reconnect_method_matches_native_absence_null_controls_and_utf16() {
    let fixture: Value = serde_json::from_str(include_str!(
        "fixtures/provider-auth-remove-method-native-4340.json"
    ))
    .unwrap();
    assert_eq!(
        fixture["complete"], true,
        "Generate the original-schema oracle first"
    );
    assert_eq!(fixture["authStartCases"].as_array().unwrap().len(), 7);
    for case in fixture["authStartCases"].as_array().unwrap() {
        let result = AUTH_START.payload(&case["input"]);
        if case["ok"] == true {
            assert_eq!(result.unwrap(), case["value"], "{}", case["name"]);
        } else {
            assert!(result.is_err(), "{}", case["name"]);
        }
    }
    let method = format!("chatgpt-profile:{}", "😀".repeat(50));
    let fake = FakeT3::spawn(vec![Reply::Ok(json!({"phase":"waiting"}))]);
    auth_start(
        &fake.connection(),
        "codex",
        Some(&format!("\u{feff}{method}\u{a0}")),
        None,
        None,
    )
    .unwrap();
    assert_eq!(
        wire(&fake.finish()),
        vec![
            json!({"tag":"provider.auth.start","payload":{"instanceId":"codex","methodId":method}})
        ]
    );
    assert_eq!(auth_start_method_id(&json!({})).unwrap(), None);
    assert!(auth_start_method_id(&json!({"methodId":null})).is_err());
}

#[test]
fn typed_and_registry_dispatch_preserve_native_controls_and_utf16() {
    for method in [
        "profile:a\0b".to_string(),
        "profile:a\nb".to_string(),
        "😀".repeat(64),
    ] {
        let input = json!({"instanceId":" \u{feff}codex\u{a0}","methodId":method,"extra":true});
        let expected = json!({"instanceId":"codex","methodId":method});
        assert_eq!(
            AuthRemoveMethodInput::from_json(&input)
                .unwrap()
                .payload()
                .unwrap(),
            expected
        );
        assert_eq!(AUTH_REMOVE_METHOD.payload(&input).unwrap(), expected);
        let fake = FakeT3::spawn(vec![Reply::Ok(json!({"phase":"idle","methods":[]}))]);
        let response = api::call(
            &fake.connection(),
            "provider.auth.removeMethod",
            &input,
            None,
        )
        .unwrap();
        assert_eq!(response["phase"], "idle");
        assert_eq!(
            wire(&fake.finish()),
            vec![json!({"tag":"provider.auth.removeMethod","payload":expected})]
        );
    }
    let invalid = json!({"instanceId":"codex","methodId":"😀".repeat(65)});
    assert!(AUTH_REMOVE_METHOD.payload(&invalid).is_err());
    assert!(AuthRemoveMethodInput {
        instance_id: "codex".into(),
        method_id: "😀".repeat(65)
    }
    .payload()
    .is_err());
}

#[test]
fn uncertain_removal_is_not_replayed() {
    let fake = FakeT3::spawn(vec![Reply::DropAfterRequest]);
    assert!(auth_remove_method(&fake.connection(), "codex", "profile:a").is_err());
    let recorded = fake.finish();
    assert_eq!(recorded.connections, 1);
    assert_eq!(recorded.requests.len(), 1);
}

#[test]
fn typed_helper_sends_once_and_does_not_replay_a_rejected_removal() {
    let detail = "Only incomplete ChatGPT connections can be removed.";
    let fake = FakeT3::spawn(vec![Reply::Fail(fail(
        json!({"_tag":"ProviderSetupError","instanceId":"codex","operation":"removeMethod","detail":detail,"message":detail}),
    ))]);
    assert!(auth_remove_method(&fake.connection(), " codex ", " profile:a\0b ").is_err());
    let recorded = fake.finish();
    assert_eq!(recorded.connections, 1);
    assert_eq!(
        wire(&recorded),
        vec![
            json!({"tag":"provider.auth.removeMethod","payload":{"instanceId":"codex","methodId":"profile:a\0b"}})
        ]
    );
    assert_eq!(AUTH_REMOVE_METHOD.describe()["scope"], "providers:manage");
    assert_eq!(AUTH_REMOVE_METHOD.describe()["replay"], "never");
}
