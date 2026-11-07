//! Exact native reset union and synthetic one-shot transport; no live redemption.
#[path = "support/fake_t3.rs"]
mod fake_t3;
use fake_t3::{fail, wire, FakeT3, Reply};
use serde_json::{json,Value};
use t3_protocol::api::{self,provider::{consume_reset_credit,ResetCreditInput,CONSUME_RESET_CREDIT}};

#[test]
fn input_matches_all_actual_native_union_transform_cases() {
    let fixture:Value=serde_json::from_str(include_str!("fixtures/reset-credit-input-4288.json")).unwrap();
    assert_eq!(fixture["revision"],"32a0531748c44b25072d7495ce0b9bd7e9450a87");
    assert_eq!(fixture["cases"].as_array().unwrap().len(),67);
    for case in fixture["cases"].as_array().unwrap(){
        let parsed=ResetCreditInput::from_json(&case["input"]).and_then(|v|v.payload());
        let registry=CONSUME_RESET_CREDIT.payload(&case["input"]);
        if case["ok"]==true {
            assert_eq!(parsed.unwrap(),case["value"]);
            assert_eq!(registry.unwrap(),case["value"]);
        }else{assert!(parsed.is_err());assert!(registry.is_err());}
    }
}
#[test]
fn typed_provider_and_source_calls_send_once_and_keep_scope_and_replay_policy() {
    let fake=FakeT3::spawn(vec![Reply::Ok(json!({"outcome":"reset"})),Reply::Ok(json!({"outcome":"alreadyRedeemed","warning":"Synthetic warning"}))]);
    let c=fake.connection();
    assert_eq!(consume_reset_credit(&c,&ResetCreditInput::Provider{instance_id:" Codex ".into()}).unwrap()["outcome"],"reset");
    let input=ResetCreditInput::Source{source_id:" raw:source\n ".into(),account_id:" account ".into(),credit_id:" credit ".into()};
    assert_eq!(consume_reset_credit(&c,&input).unwrap()["outcome"],"alreadyRedeemed");
    assert_eq!(wire(&fake.finish()),vec![json!({"tag":"provider.consumeResetCredit","payload":{"instanceId":"Codex"}}),json!({"tag":"provider.consumeResetCredit","payload":{"sourceId":" raw:source\n ","accountId":"account","creditId":"credit"}})]);
    let descriptor=CONSUME_RESET_CREDIT.describe();
    assert_eq!(descriptor["scope"],"providers:manage");
    assert_eq!(descriptor["replay"],"never");
    assert_eq!(api::function("provider.consumeResetCredit").unwrap().name,CONSUME_RESET_CREDIT.name);
}
#[test]
fn rejected_redemption_is_not_replayed() {
    let fake=FakeT3::spawn(vec![Reply::Fail(fail(json!({"_tag":"UsageLimitSourceError","detail":"Synthetic unavailable"})))]);
    assert!(consume_reset_credit(&fake.connection(),&ResetCreditInput::Provider{instance_id:"codex".into()}).is_err());
    assert_eq!(fake.finish().connections,1);
}
