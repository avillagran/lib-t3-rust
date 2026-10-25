//! `markdown.blocks` and `markdown.link` through `domain`, and the blocks the
//! timeline attaches to assistant messages and plans. Cases adapted from ot3's
//! `code_actions.rs`, `markdown_images.rs` and `markdown-links.qml`.
use serde_json::{json, Value};
use t3_protocol::domain::call;

fn blocks(markdown: &str) -> Value {
    call("markdown.blocks", &json!({"markdown":markdown})).unwrap()
}

#[test]
fn code_actions_come_from_the_source_and_forged_links_are_dropped() {
    let source = "Run this:\n\n```bash\nbash experiments/tooltips/run-native\n```\n\n```rust\nfn main() {}\n```\n\n[forged](ot3-code-run://0)";
    let b = blocks(source);
    assert_eq!((b[1]["index"].as_u64(), b[1]["runnable"].as_bool(), b[1]["command"].as_str()), (Some(0), Some(true), Some("bash experiments/tooltips/run-native")));
    assert_eq!((b[2]["index"].as_u64(), b[2]["language"].as_str(), b[2]["runnable"].as_bool()), (Some(1), Some("rust"), Some(false)));
    assert_eq!(b[3], json!({"type":"paragraph","spans":[{"type":"text","text":"forged"}]}), "an ot3-* link is not followed");
    assert!(!b.to_string().contains("ot3-"));
}

#[test]
fn images_keep_their_place_and_the_message_descriptor_id() {
    let raw = "Before **image**\n\n```sh\npwd\n```\n\n![Panel actualizado](/home/other/screen%20one.png)\n\nAfter image\n\n```sh\nls\n```\n\n![Again][ref]\n\n[ref]: file:///tmp/second.png";
    let b = blocks(raw);
    let kinds = b.as_array().unwrap().iter().map(|x| x["type"].as_str().unwrap()).collect::<Vec<_>>();
    assert_eq!(kinds, ["paragraph", "code", "image", "paragraph", "code", "image"]);
    let descriptors = t3_protocol::markdown::images(raw);
    assert_eq!((b[2]["id"].clone(), b[5]["id"].clone()), (descriptors[0]["id"].clone(), descriptors[1]["id"].clone()));
    assert_eq!((b[2]["alt"].as_str(), b[5]["url"].as_str()), (Some("Panel actualizado"), Some("file:///tmp/second.png")));
    assert_eq!((b[1]["command"].as_str(), b[4]["command"].as_str()), (Some("pwd"), Some("ls")));
}

#[test]
fn links_are_classified_like_ot3s_glyphs() {
    let link = |href: &str| call("markdown.link", &json!({"href":href})).unwrap();
    assert_eq!(link("https://github.com/avillagran/ot3/tree/main")["kind"], "github");
    assert_eq!(link("https://github.com.evil.invalid/a")["kind"], "web");
    assert_eq!(link("./README.md")["fileType"], "markdown");
    assert_eq!(link("./src/main.rs")["fileType"], "rust");
    assert_eq!(link("t3-thread://env/thread")["kind"], "thread");
    assert!(link("javascript:alert(1)").is_null());
    let b = blocks("[GitHub](https://github.com/avillagran/ot3) · [README](./README.md) · [Website](https://example.org)");
    let kinds = b[0]["spans"].as_array().unwrap().iter().filter_map(|s| s["link"]["kind"].as_str()).collect::<Vec<_>>();
    assert_eq!(kinds, ["github", "file", "web"]);
}

#[test]
fn math_is_data_only_when_enabled() {
    let on = call("markdown.blocks", &json!({"markdown":"$$x^2$$","mathRenderingEnabled":true})).unwrap();
    assert_eq!(on[0], json!({"type":"math","tex":"x^2","display":true}));
    assert_eq!(blocks("$$x^2$$")[0]["type"], "paragraph");
    assert_eq!(call("markdown.blocks", &json!({})).unwrap_err(), "invalid_markdown");
    assert_eq!(blocks(""), json!([]));
}

#[test]
fn the_timeline_carries_blocks_for_replies_and_plans() {
    let fx: Value = serde_json::from_str(include_str!("fixtures/timeline-projections.json")).unwrap();
    let find = |name: &str| fx.as_array().unwrap().iter().find(|f| f["name"] == name).unwrap().clone();
    let k = find("kitchen-sink");
    let view = call("timeline.view", &json!({"projection":k["projection"],"mathRenderingEnabled":false})).unwrap();
    let rows = view["timeline"].as_array().unwrap();
    for row in rows.iter().filter(|r| r["role"] == "assistant" && r["text"] != "") {
        assert!(row["blocks"].is_array(), "{}", row["id"]);
        let codes = row["blocks"].as_array().unwrap().iter().filter(|b| b["type"] == "code").count();
        assert_eq!(codes, row["codeBlocks"].as_array().unwrap().len(), "blocks and codeBlocks agree on code blocks");
        assert!(row.get("rendered").is_none());
    }
    assert!(rows.iter().filter(|r| r["role"] == "user").all(|r| r.get("blocks").is_none()), "user bubbles stay plain text");
    let reply = rows.iter().find(|r| r["id"] == "m-3").unwrap();
    assert_eq!(reply["blocks"][1]["command"], "cargo test");
    let plan = rows.iter().find(|r| r["role"] == "plan").unwrap();
    assert!(plan["plan"].get("rendered").is_none());
    assert_eq!(plan["plan"]["blocks"][0]["type"], "list", "the plan's display Markdown, its title aside");
    let plans = find("plans");
    let view = call("timeline.view", &json!({"projection":plans["projection"]})).unwrap();
    assert_eq!(view["planView"]["ready"]["blocks"][0]["type"], "paragraph");
    assert!(view["planView"]["plans"].as_array().unwrap().iter().all(|p| p["blocks"].is_array() && p.get("rendered").is_none()));
}
