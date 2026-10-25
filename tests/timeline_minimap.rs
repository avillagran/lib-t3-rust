//! `timeline.minimap` over `timeline.view`'s rows: one marker per user row
//! with its last reply, and the marker rules of ot3's minimap.
use serde_json::{json, Value};
use t3_protocol::domain::call;

#[test]
fn minimap_follows_the_timeline_rows() {
    let fixtures: Value = serde_json::from_str(include_str!("fixtures/timeline-projections.json")).unwrap();
    let mut seen = 0;
    for fx in fixtures.as_array().unwrap() {
        let rows = call("timeline.view", &json!({"projection":fx["projection"]})).unwrap()["timeline"].clone();
        let map = call("timeline.minimap", &json!({"rows":rows})).unwrap();
        let rows = rows.as_array().unwrap();
        let users = rows.iter().filter(|r| r["role"] == "user").count();
        assert_eq!(map["items"].as_array().unwrap().len(), users, "{}", fx["name"]);
        assert_eq!(map["visible"], users >= 2);
        for item in map["items"].as_array().unwrap() {
            let mut k = item.as_object().unwrap().keys().map(String::as_str).collect::<Vec<_>>();
            k.sort();
            assert_eq!(k, ["assistantPreview", "assistantText", "id", "rowIndex", "userPreview", "userText"]);
            let at = item["rowIndex"].as_u64().unwrap() as usize;
            assert_eq!((rows[at]["role"].as_str(), &rows[at]["id"]), (Some("user"), &item["id"]));
            seen += 1;
        }
        // With nothing on screen and the list at its end, the last marker is current.
        assert_eq!(call("timeline.minimap", &json!({"rows":rows,"atEnd":true})).unwrap()["currentIndex"], users as i64 - 1);
    }
    assert!(seen > 0);
}

#[test]
fn minimap_current_marker_and_rail_index() {
    let rows = json!([
        {"id":"u1","role":"user","text":"Fix\nthe   build"},{"id":"a1","role":"assistant","text":"Done."},
        {"id":"t1","role":"activity"},{"id":"u2","role":"user","text":"And tests?"},{"id":"a2","role":"assistant","text":"Added\ttwo."},
        {"id":"u3","role":"user","text":"Thanks"},
    ]);
    let map = call("timeline.minimap", &json!({"rows":rows,"visibleRowIds":["t1","u2","a2"]})).unwrap();
    assert_eq!((map["currentIndex"].as_i64(), map["visibleIds"].clone()), (Some(1), json!(["u2"])));
    assert_eq!((map["items"][0]["userPreview"].as_str(), map["items"][1]["assistantPreview"].as_str()), (Some("Fix the build"), Some("Added two.")));
    assert!(map["items"][2]["assistantText"].is_null());
    // Only an activity row on screen: the marker at or before it.
    assert_eq!(call("timeline.minimap", &json!({"rows":rows,"visibleRowIds":["t1"]})).unwrap()["currentIndex"], 0);
    assert_eq!(call("timeline.minimapIndex", &json!({"y":75.0,"height":100.0,"count":3})).unwrap(), 2);
    assert_eq!(call("timeline.minimapIndex", &json!({"y":10,"height":0,"count":3})).unwrap(), -1);
    assert_eq!(call("timeline.minimapIndex", &json!({"y":10,"height":100,"count":1.5})).unwrap_err(), "invalid_count");
}
