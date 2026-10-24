//! Drift between the typed API registry and T3's contract, pinned in
//! `tests/fixtures/t3-contract.json` (regenerate with `tools/extract-t3-contract`).
//! With `T3_CONTRACTS_DIR` pointing at a T3 checkout, tags are also re-read live.
use serde_json::Value;
use t3_protocol::api::{self, orchestration, Scope};

fn contract() -> Value {
    serde_json::from_str(include_str!("fixtures/t3-contract.json")).unwrap()
}

#[test]
fn registry_matches_the_pinned_contract_tag_by_tag() {
    let contract = contract();
    assert_eq!(contract["contract"], api::CONTRACT);
    for f in api::functions() {
        let Some(tag) = f.tag else { continue };
        let method = &contract["methods"][tag];
        assert!(method.is_object(), "{tag} is not a T3 RPC");
        let scopes = match f.scope {
            Scope::Fixed(scope) => vec![scope],
            Scope::Dynamic(scopes) => scopes.to_vec(),
        };
        for scope in scopes {
            assert!(contract["scopes"].as_array().unwrap().iter().any(|s| s == scope), "{tag}: {scope} is not a T3 scope");
        }
        assert_eq!(method["stream"], f.stream, "{tag}: stream");
        match f.scope {
            Scope::Fixed(scope) => {
                assert_eq!(method["scope"], scope, "{tag}: scope");
                assert_eq!(method["dynamic"], false, "{tag} is payload-dependent in T3");
            }
            Scope::Dynamic(scopes) => {
                assert_eq!(method["dynamic"], true, "{tag} has a fixed scope in T3");
                let base = method["scope"].as_str().unwrap();
                assert!(scopes.contains(&base), "{tag}: {base} not among {scopes:?}");
            }
        }
    }
}

#[test]
fn command_types_cover_the_contract_union_exactly() {
    let contract = contract();
    let mut ours = orchestration::COMMAND_TYPES.iter().chain(orchestration::RAW_COMMAND_TYPES).map(|t| t.to_string()).collect::<Vec<_>>();
    ours.sort();
    let theirs = contract["commandTypes"].as_array().unwrap().iter().map(|t| t.as_str().unwrap().to_string()).collect::<Vec<_>>();
    assert_eq!(ours, theirs);
}

#[test]
fn live_checkout_still_declares_every_tag() {
    let Ok(dir) = std::env::var("T3_CONTRACTS_DIR") else { return };
    let src = std::path::Path::new(&dir);
    let text = ["rpc.ts", "orchestrationV2.ts"].iter().map(|f| std::fs::read_to_string(src.join(f)).unwrap()).collect::<String>();
    for tag in api::functions().filter_map(|f| f.tag) {
        assert!(text.contains(&format!("\"{tag}\"")), "{tag} is gone from {dir}");
    }
    // The handler-level scope the fixture records as an exception (tools/extract-t3-contract).
    let ws = src.join("../../../apps/server/src/ws.ts");
    if let Ok(ws) = std::fs::read_to_string(ws) {
        assert!(ws.contains("requiredScopeForDeviceList("), "device.list no longer has a handler-level scope");
    }
}
