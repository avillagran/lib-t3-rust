//! Subagent cards through `domain`: ot3's buckets, labels, tones, detail and
//! elapsed time, the live overlay and the group header.
use serde_json::{json, Value};
use t3_protocol::domain::call;

fn keys(v: &Value) -> Vec<&str> {
    let mut k = v.as_object().unwrap().keys().map(String::as_str).collect::<Vec<_>>();
    k.sort();
    k
}

#[test]
fn subagent_cards_from_a_timeline_row() {
    let cards = json!([
        {"id":"s1","title":"Explore","status":"pending","startedAt":1000,"progress":null,"result":null},
        {"id":"s2","title":"Review","status":"completed","startedAt":1000,"completedAt":250000,"progress":"reading","result":"2 issues"},
        {"id":"s3","title":"Tests","status":"failed","startedAt":0,"result":"exit 1"},
    ]);
    let live = json!([{"id":"s1","status":"running","progress":"step 2","startedAt":1000}]);
    let view = call("subagents.cards", &json!({"cards":cards,"threadSubagents":live,"now":3_726_000})).unwrap();
    assert_eq!(keys(&view), ["anyLive", "counts", "expanded", "group", "header", "subagents", "summary"]);
    assert_eq!((view["group"].as_bool(), view["anyLive"].as_bool(), view["expanded"].as_bool()), (Some(true), Some(true), Some(true)));
    assert_eq!(view["header"], json!({"source":"{n} subagents","args":{"n":3}}));
    assert_eq!(view["summary"], json!([{"source":"{n} working","args":{"n":1}},{"source":"{n} done","args":{"n":1}},{"source":"{n} failed","args":{"n":1}}]));
    let first = &view["subagents"][0];
    assert_eq!((first["subagent"]["status"].as_str(), first["subagent"]["title"].as_str()), (Some("running"), Some("Explore")));
    assert_eq!(first["view"], json!({"live":true,"settled":false,"label":"Running","tone":"working","detail":"step 2","elapsed":"1h 02m"}));
    assert_eq!(view["subagents"][1]["view"], json!({"live":false,"settled":true,"label":"Completed","tone":"done","detail":"2 issues","elapsed":"4m 09s"}));
    assert_eq!(view["subagents"][2]["view"]["elapsed"], "");
    assert_eq!(call("subagents.buckets", &json!({"subagents":[{"status":"waiting"},{"status":"interrupted"},{"status":"idle"},{}]})).unwrap(), json!({"working":1,"done":0,"failed":0,"stopped":2,"idle":1}));
    assert_eq!(call("subagents.describe", &json!({"subagent":{"status":"idle"},"now":0})).unwrap()["label"], "Idle · resumable");
    // Four settled cards start collapsed.
    let quiet = json!([{"status":"completed"},{"status":"completed"},{"status":"failed"},{"status":"cancelled"}]);
    assert_eq!(call("subagents.cards", &json!({"cards":quiet,"now":0})).unwrap()["expanded"], false);
    let sub = call("subagents.fromThread", &json!({"thread":{"key":"laptop/t9","id":"t9","title":"Fix","pending":true,"reason":"waiting","updatedAt":5}})).unwrap();
    assert_eq!(sub, json!({"id":"thread:laptop/t9","title":"Fix","status":"waiting","childThreadId":"t9","startedAt":0,"updatedAt":5,"fromThread":true}));
}

#[test]
fn timeline_subagent_rows_feed_the_cards() {
    let fixtures: Value = serde_json::from_str(include_str!("fixtures/timeline-projections.json")).unwrap();
    let mut rows = 0;
    for fx in fixtures.as_array().unwrap() {
        let view = call("timeline.view", &json!({"projection":fx["projection"]})).unwrap();
        for row in view["timeline"].as_array().unwrap().iter().filter(|r| r["role"] == "subagents") {
            let cards = call("subagents.cards", &json!({"cards":row["subagents"],"threadSubagents":view["threadState"]["subagents"],"now":1_792_036_800_000i64})).unwrap();
            assert_eq!(cards["subagents"].as_array().unwrap().len(), row["subagents"].as_array().unwrap().len());
            rows += 1;
        }
    }
    assert!(rows > 0, "the fixtures have subagent rows");
}
