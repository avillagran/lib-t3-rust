//! `theme.tokens` on Omarchy's 22 themes (`fixtures/omarchy-theme-tokens.json`):
//! the golden output, and every colour ot3 drew for that theme (read back from
//! Qt) equal to the lib's. Regenerate the goldens with
//! `T3_UPDATE_FIXTURES=1 cargo test --test theme_tokens`; the `ot3` blocks stay.
use serde_json::{json, Value};
use std::path::Path;
use t3_protocol::domain::call;

const FIXTURE: &str = "tests/fixtures/omarchy-theme-tokens.json";

fn fixture() -> Value {
    serde_json::from_str(include_str!("fixtures/omarchy-theme-tokens.json")).unwrap()
}

#[test]
fn goldens_for_the_22_omarchy_themes() {
    let mut fx = fixture();
    let update = std::env::var_os("T3_UPDATE_FIXTURES").is_some();
    for theme in fx["themes"].as_array_mut().unwrap() {
        let tokens = call("theme.tokens", &json!({"colors":theme["colors"],"id":theme["id"]})).unwrap();
        if update {
            theme["tokens"] = tokens;
        } else {
            assert_eq!(tokens, theme["tokens"], "{} is stale: run T3_UPDATE_FIXTURES=1 cargo test --test theme_tokens", theme["id"]);
        }
    }
    if update {
        std::fs::write(Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE), serde_json::to_string_pretty(&fx).unwrap() + "\n").unwrap();
    }
    assert_eq!(fx["themes"].as_array().unwrap().len(), 22);
}

#[test]
fn every_role_is_what_ot3_drew() {
    let fx = fixture();
    let mut compared = 0;
    for theme in fx["themes"].as_array().unwrap() {
        let id = theme["id"].as_str().unwrap();
        let (ot3, t) = (&theme["ot3"], call("theme.tokens", &json!({"colors":theme["colors"],"id":id})).unwrap());
        let mut eq = |what: &str, a: &Value, b: &Value| {
            assert_eq!(a, b, "{id} {what}");
            compared += 1;
        };
        for (role, drawn) in ot3["rendered"].as_object().unwrap() {
            eq(role, drawn, &t["roles"][if role == "dim60" { "statusMuted" } else { role.as_str() }]);
        }
        for (role, colour) in ot3["palette"].as_object().unwrap() {
            eq(role, colour, &t["roles"][role]);
        }
        for (detail, colour) in ot3["detailColors"].as_object().unwrap() {
            eq(detail, colour, &t["statusColors"][detail]);
        }
        eq("projectColors", &ot3["projectColors"], &t["projectPalette"]);
        eq("projectFill", &ot3["projectFill"], &t["projectFills"]);
        eq("skyHeaderBase", &ot3["skyHeaderBase"], &t["sky"]["header"]["base"]);
        eq("lightSky", &ot3["lightSky"], &t["sky"]["lightSky"]);
        eq("document", &ot3["document"], &t["document"]);
        eq("mcpTheme", &ot3["mcpTheme"], &t["mcpTheme"]);
        for k in ["background", "foreground", "accent", "muted"] {
            eq(k, &ot3["commons"][k], &t["base"][k]);
        }
        eq("urgent", &ot3["commons"]["urgent"], &t["roles"]["failed"]);
    }
    assert_eq!(compared, 22 * 44);
}

#[test]
fn one_light_rule_agrees_with_every_theme() {
    let fx = fixture();
    let mut light = vec![];
    for theme in fx["themes"].as_array().unwrap() {
        let t = &theme["tokens"];
        // The sky's Rec.709 rule, the document and MCP rules and the theme's own mode agree.
        for other in [&t["document"]["appearance"], &t["mcpTheme"], &t["mode"]] {
            assert_eq!(&t["appearance"], other, "{}", theme["id"]);
        }
        if t["appearance"] == "light" {
            light.push(theme["id"].as_str().unwrap());
        }
    }
    assert_eq!(light, ["catppuccin-latte", "flexoki-light", "lupine", "rose-pine", "white"]);
}

#[test]
fn tokyo_night_and_catppuccin_latte() {
    let fx = fixture();
    let t = |id: &str| fx["themes"].as_array().unwrap().iter().find(|t| t["id"] == id).unwrap()["tokens"].clone();
    let tokyo = t("tokyo-night");
    assert_eq!((tokyo["name"].as_str(), tokyo["appearance"].as_str()), (Some("Tokyo Night"), Some("dark")));
    assert_eq!(tokyo["roles"]["dim"], "#6b718a");
    assert_eq!(tokyo["roles"]["divider"], "#272936");
    assert_eq!(tokyo["roles"]["surface"], "#21222e");
    assert_eq!(tokyo["roles"]["working"], "#7aa2f7");
    assert_eq!(tokyo["roles"]["warning"], "#eb927b");
    assert_eq!(tokyo["roles"]["onAccent"], "#1a1b26");
    assert_eq!(tokyo["layers"]["dim"], json!({"color":"foreground","alpha":0.57}));
    assert_eq!(tokyo["sky"]["header"], json!({"brightness":1.25,"nebula":0.55,"meteorRate":0.85,"base":"#0b0b10"}));
    assert_eq!((tokyo["sky"]["meteor"].as_str(), tokyo["sky"]["ink"].as_f64()), (Some("#ffffff"), Some(1.0)));
    let latte = t("catppuccin-latte");
    assert_eq!((latte["appearance"].as_str(), latte["sky"]["ink"].as_f64(), latte["sky"]["meteor"].as_str()), (Some("light"), Some(1.7), Some("#4c4f69")));
    assert_eq!(latte["roles"]["dim"], "#9295a6");
    assert_eq!(latte["roles"]["failed"], "#d20f39");
    // A theme without orange: warning falls back to yellow (T3Palette.qml:15).
    let white = t("white");
    assert!(white["base"]["orange"].is_null());
    assert_eq!(white["roles"]["warning"], white["roles"]["waiting"]);
}
