//! `projects::monogram` and `projects::identities`: ot3's `project_identity`
//! and its `T3Palette.projectColor` / `T3ProjectIcon` chip.
use serde_json::{json, Value};
use t3_protocol::projects::{color_slots, identities, monogram, palette};

#[test]
fn monograms_keep_t3_geometry() {
    for (name, expected) in [
        ("omarchy-t3-code", "OC"),
        ("T3", "T3"),
        ("app2", "A2"),
        ("docs site", "DS"),
        ("x", "XX"),
        ("", "PR"),
        ("   ", "PR"),
        ("über cool", "ÜC"),
        ("日本 語", "日語"),
        ("a-b-c", "AC"),
    ] {
        assert_eq!(monogram(name), expected, "{name}");
    }
}

#[test]
fn colour_slots_avoid_collisions_and_follow_the_theme() {
    // ot3 tests/qml/sidebar.qml: three projects, a partial theme, distinct colours.
    let projects = ["fixture/p1", "fixture/p2", "fixture/p3"].map(|k| json!({"key":k,"title":"One"}));
    let theme = json!({"blue":"#123456","green":"#459900","red":"#ff3344","magenta":"#7f4488"});
    let chips = identities(&projects, &theme, Some("#AbCdEf"));
    let colours = chips.iter().map(|c| c["color"].as_str().unwrap().to_string()).collect::<Vec<_>>();
    assert!(colours[0] != colours[1] && colours[1] != colours[2] && colours[0] != colours[2], "project collision avoidance: {colours:?}");
    let keys = palette(&theme, Some("#AbCdEf")).into_iter().map(|(k, _)| k).collect::<Vec<_>>();
    assert_eq!(keys, ["blue", "green", "magenta", "cyan", "red"], "keys the theme lacks fall back to one accent slot");
    assert_eq!(palette(&theme, Some("#AbCdEf"))[3].1.as_deref(), Some("#abcdef"));
    assert_eq!(palette(&Value::Null, None).len(), 9, "without a theme every key is a slot");
    // More projects than slots: every slot is used once, then hashes collide.
    let many = (0..12).map(|i| format!("e/p{i}")).collect::<Vec<_>>();
    let slots = color_slots(&many.iter().map(String::as_str).collect::<Vec<_>>(), 9);
    let mut used = slots.values().copied().collect::<Vec<_>>();
    used.sort();
    used.dedup();
    assert_eq!(used.len(), 9);
    assert_eq!(color_slots(&["a"], 9)["a"], 97 % 9);
}

#[test]
fn icon_overrides_pick_label_kind_and_colour() {
    let projects = [
        json!({"key":"e/1","title":"Site","projectIcon":{"kind":"emoji","emoji":"🚀","color":"red"}}),
        json!({"key":"e/2","title":"App","projectIcon":{"kind":"monogram","text":"Ω"}}),
        json!({"key":"e/3","title":"Lib","projectIcon":{"kind":"lucide","name":"rocket","color":"not-in-theme"}}),
        json!({"key":"e/4","title":"Docs"}),
    ];
    let theme = json!({"red":"#FF0000","blue":"#0000ff"});
    let chips = identities(&projects, &theme, Some("#00ff00"));
    assert_eq!((chips[0]["label"].as_str(), chips[0]["kind"].as_str(), chips[0]["colorKey"].as_str(), chips[0]["color"].as_str()), (Some("🚀"), Some("emoji"), Some("red"), Some("#ff0000")));
    assert_eq!(chips[0]["fill"], 0.0, "emoji and lucide chips have no tinted fill");
    assert_eq!((chips[1]["label"].as_str(), chips[1]["fill"].as_f64()), (Some("Ω"), Some(0.14)));
    assert_eq!((chips[2]["icon"].as_str(), chips[2]["label"].as_str()), (Some("rocket"), Some("LB")));
    assert_ne!(chips[2]["colorKey"], "not-in-theme", "a key the theme lacks keeps the slot colour");
    assert_eq!(chips[3]["label"], "DS");
    let unthemed = identities(&projects, &Value::Null, None);
    assert_eq!((unthemed[0]["colorKey"].as_str(), unthemed[0]["color"].as_str()), (Some("red"), None), "without a theme the host resolves the key");
}

#[test]
fn identities_through_domain_and_in_the_sidebar() {
    let chips = t3_protocol::domain::call("projects.identity", &json!({"projects":[{"key":"e/1","title":"omarchy-t3-code"}]})).unwrap();
    assert_eq!(chips[0]["monogram"], "OC");
    assert_eq!(t3_protocol::domain::call("projects.monogram", &json!({"name":"lib t3 rust"})).unwrap(), "LR");
    let fx: Value = serde_json::from_str(include_str!("fixtures/sidebar-environments.json")).unwrap();
    let v = t3_protocol::domain::call("sidebar.view", &json!({"environments":fx["environments"],"now":fx["nowMs"],"colors":{"blue":"#123456"},"accent":"#abcdef"})).unwrap();
    let ids = v["projectIdentities"].as_object().unwrap();
    assert_eq!(ids.len(), 4);
    assert_eq!(ids["laptop/p-docs"]["monogram"], "DS");
    assert!(ids.values().all(|i| i["color"].is_string()));
}
