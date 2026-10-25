//! Subscription usage through `domain`: the usage page's request and totals and
//! the quota strip, with their shapes, and that no credential crosses.
use serde_json::{json, Value};
use t3_protocol::domain::call;

fn keys(v: &Value) -> Vec<&str> {
    let mut k = v.as_object().unwrap().keys().map(String::as_str).collect::<Vec<_>>();
    k.sort();
    k
}
fn limits() -> Value {
    json!({"checkedAt":"2026-10-18T12:00:00Z","windows":[{"id":"5h","kind":"session","label":"5 hours","usedPercent":42.5,"resetsAt":"2026-10-18T15:00:00Z","windowDurationMins":300}],"accessToken":"PRIVATE"})
}

#[test]
fn usage_summary_window_request_and_aggregate() {
    assert_eq!(call("usageSummary.window", &json!({"period":"week","today":"2026-10-20"})).unwrap(), json!({"sinceDay":"2026-10-14","untilDay":"2026-10-20"}));
    assert_eq!(call("usageSummary.window", &json!({"period":"year","today":"2026-10-20"})).unwrap()["sinceDay"], "2026-01-01");
    let request = call("usageSummary.request", &json!({"period":"month","today":"2026-10-20","timeZone":"America/Santiago"})).unwrap();
    assert_eq!(request, json!({"sinceDay":"2026-09-21","untilDay":"2026-10-20","payload":{"sinceDay":"2026-09-21","untilDay":"2026-10-20","timeZone":"America/Santiago","resolution":"day"}}));
    // The payload is exactly what the typed server.getUsageSummary accepts.
    let typed = t3_protocol::api::function("server.getUsageSummary").unwrap();
    assert_eq!(typed.payload(&request["payload"]).unwrap(), request["payload"]);
    assert_eq!(call("usageSummary.request", &json!({"period":"day","today":"2026-10-20","timeZone":" "})).unwrap_err(), "invalid_time_zone");
    assert_eq!(call("usageSummary.window", &json!({"period":"decade","today":"2026-10-20"})).unwrap_err(), "invalid_period");
    assert_eq!(call("usageSummary.window", &json!({"period":"day","today":"20/10/2026"})).unwrap_err(), "invalid_today");

    let totals = json!({"uncachedInputTokens":100,"cachedInputTokens":10,"cacheCreationTokens":0,"outputTokens":50,"reasoningTokens":5});
    let summary = json!({"buckets":[{"day":"2026-10-20","provider":"claude","model":"opus","totals":totals,"costUsd":1.5,"cacheSavingsUsd":0.25,"records":3,"unpricedRecords":1,"sessions":2}],
        "sources":[{"fingerprint":{"provider":"claude"},"status":"ok","message":null}],"pricing":{"status":"fresh"},"readAt":"2026-10-20T10:00:00Z"});
    let a = call("usageSummary.aggregate", &json!({"summary":summary,"sinceDay":"2026-10-14","untilDay":"2026-10-20"})).unwrap();
    assert_eq!(keys(&a), ["cacheSavings", "cost", "days", "kinds", "models", "pricing", "providers", "readAt", "sessions", "sinceDay", "sources", "tokens", "unpricedRecords", "untilDay"]);
    assert_eq!((a["cost"].as_f64(), a["tokens"].as_u64(), a["sessions"].as_u64(), a["days"].as_array().map(Vec::len)), (Some(1.5), Some(165), Some(2), Some(7)));
    assert_eq!(a["models"][0], json!({"provider":"claude","model":"opus","cost":1.5,"tokens":165,"records":3}));
    assert_eq!(a["kinds"], json!({"input":100,"cached":10,"cacheWrite":0,"output":50,"reasoning":5}));
    assert_eq!(call("usageSummary.aggregate", &json!({"summary":summary,"sinceDay":"x","untilDay":"2026-10-20"})).unwrap_err(), "invalid_since_day");
}

#[test]
fn quota_sources_limits_providers_sources_and_apply() {
    let l = call("quotaSources.limits", &json!({"usageLimits":limits()})).unwrap();
    assert_eq!(keys(&l), ["checkedAt", "windows"]);
    assert_eq!(keys(&l["windows"][0]), ["id", "kind", "label", "resetsAt", "usedPercent", "windowDurationMins"]);
    assert!(call("quotaSources.limits", &json!({"usageLimits":{"windows":[]}})).unwrap().is_null());

    let provider = json!({"instanceId":"claude","driver":"claudeAgent","enabled":true,"installed":true,"auth":{"status":"authenticated","email":"a@example.test","token":"PRIVATE"},"usageLimits":limits(),"config":{"apiKey":"PRIVATE"}});
    let p = call("quotaSources.providers", &json!({"providers":[provider]})).unwrap();
    assert_eq!(keys(&p[0]), ["auth", "availability", "driver", "enabled", "installed", "instanceId", "usageLimits"]);
    assert_eq!(keys(&p[0]["auth"]), ["email", "status"]);
    let s = call("quotaSources.sources", &json!({"sources":[{"id":"hub","accounts":[{"id":"one","driver":"codex","plan":"pro","usageLimits":limits(),"refreshToken":"PRIVATE"}],"error":"PRIVATE"}]})).unwrap();
    assert_eq!(keys(&s[0]), ["accounts", "error", "id", "label"]);
    assert_eq!((s[0]["label"].as_str(), s[0]["error"].as_str()), (Some("hub"), Some("Could not read usage source.")));
    assert!(!json!([p, s]).to_string().contains("PRIVATE"));

    let first = call("quotaSources.apply", &json!({"event":{"version":1,"type":"snapshot","config":{"providers":[provider]}}})).unwrap();
    assert_eq!(first["changed"], true);
    assert_eq!(keys(&first["state"]), ["connected", "providers", "usageLimitSources"]);
    let next = call("quotaSources.apply", &json!({"state":first["state"],"event":{"version":1,"type":"usageLimitSourcesUpdated","payload":{"sources":[{"id":"hub","accounts":[]}]}}})).unwrap();
    assert_eq!((next["changed"].as_bool(), next["state"]["usageLimitSources"][0]["id"].as_str()), (Some(true), Some("hub")));
    let same = call("quotaSources.apply", &json!({"state":next["state"],"event":{"version":1,"type":"settingsUpdated","payload":{}}})).unwrap();
    assert_eq!((same["changed"].as_bool(), &same["state"]), (Some(false), &next["state"]));
}
