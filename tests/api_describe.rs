//! The saved `describe` (`bindings/android/describe.json`) and the Kotlin
//! wrappers generated from it (`bindings/android/tests/.../T3Api.kt`) must match
//! the registry. Regenerate both with `T3_UPDATE_FIXTURES=1 cargo test --test api_describe`.
use serde_json::Value;
use std::{fmt::Write, fs, path::Path};

const DESCRIBE: &str = "bindings/android/describe.json";
const KOTLIN: &str = "bindings/android/tests/dev/avillagran/t3/T3Api.kt";

fn check(path: &str, expected: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    if std::env::var_os("T3_UPDATE_FIXTURES").is_some() {
        fs::write(&path, expected).unwrap();
    }
    let saved = fs::read_to_string(&path).unwrap_or_default();
    assert!(saved == expected, "{} is stale: run T3_UPDATE_FIXTURES=1 cargo test --test api_describe", path.display());
}
fn kotlin_type(arg: &Value) -> &'static str {
    match arg["type"].as_str().unwrap() {
        "bool" => "Boolean",
        "integer" => "Long",
        "object" | "command" => "JSONObject",
        "array" => "JSONArray",
        _ => "String",
    }
}
fn capitalized(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().chain(c).collect()).unwrap_or_default()
}
/// `provider.install.subscribe` → `installSubscribe`: the name after the module, camelCased.
fn method_name(name: &str) -> String {
    let mut parts = name.split('.').skip(1);
    let first = parts.next().unwrap().to_string();
    parts.fold(first, |acc, part| acc + &capitalized(part))
}
/// Kotlin wrappers: one class per module, one method per function (streams open a subscription).
fn kotlin(describe: &Value) -> String {
    let functions = describe["functions"].as_array().unwrap();
    let mut modules = functions.iter().map(|f| f["module"].as_str().unwrap()).collect::<Vec<_>>();
    modules.sort();
    modules.dedup();
    let mut out = String::new();
    writeln!(out, "// Generated from bindings/android/describe.json ({}) by tests/api_describe.rs.", describe["contract"].as_str().unwrap()).unwrap();
    writeln!(out, "// Do not edit: run `T3_UPDATE_FIXTURES=1 cargo test --test api_describe`.").unwrap();
    let arrays = functions.iter().flat_map(|f| f["args"].as_array().unwrap()).any(|a| a["type"] == "array");
    writeln!(out, "package dev.avillagran.t3\n").unwrap();
    if arrays {
        writeln!(out, "import org.json.JSONArray").unwrap();
    }
    writeln!(out, "import org.json.JSONObject\n").unwrap();
    writeln!(out, "/** Typed wrappers over [T3Client.call] and [T3Client.subscribeFn], one class per `describe` module. */").unwrap();
    writeln!(out, "@Suppress(\"unused\")\nclass T3Api(client: T3Client) {{").unwrap();
    for m in &modules {
        writeln!(out, "    val {m} = {}(client)", capitalized(m)).unwrap();
    }
    for m in &modules {
        writeln!(out, "\n    class {} internal constructor(private val client: T3Client) {{", capitalized(m)).unwrap();
        for f in functions.iter().filter(|f| f["module"] == *m) {
            let name = f["fn"].as_str().unwrap();
            let method = method_name(name);
            let scope = match f["scope"].as_str().unwrap() {
                "dynamic" => format!("one of {}", f["scopes"].as_array().unwrap().iter().map(|s| s.as_str().unwrap()).collect::<Vec<_>>().join(", ")),
                s => s.to_string(),
            };
            let stream = f["stream"] == true;
            if stream {
                writeln!(out, "        /** `{name}` · stream · {} · replay {} · scope {scope} */", f["kind"].as_str().unwrap(), f["replay"].as_str().unwrap()).unwrap();
            } else {
                writeln!(out, "        /** `{name}` · {} · replay {} · scope {scope} · {} ms */", f["kind"].as_str().unwrap(), f["replay"].as_str().unwrap(), f["timeoutMs"]).unwrap();
            }
            let (open, returns) = if stream { ("subscribeFn", "T3Subscription") } else { ("call", "Any?") };
            if f["openArgs"] == true {
                writeln!(out, "        fun {method}(args: JSONObject): {returns} = client.{open}(\"{name}\", args)").unwrap();
                continue;
            }
            let args = f["args"].as_array().unwrap();
            let ordered = args.iter().filter(|a| a["required"] == true).chain(args.iter().filter(|a| a["required"] != true));
            let params = ordered
                .map(|a| {
                    let arg = a["name"].as_str().unwrap();
                    if a["required"] == true {
                        format!("{arg}: {}", kotlin_type(a))
                    } else {
                        format!("{arg}: {}? = null", kotlin_type(a))
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            let puts = args.iter().map(|a| format!(".put(\"{0}\", {0})", a["name"].as_str().unwrap())).collect::<String>();
            writeln!(out, "        fun {method}({params}): {returns} =\n            client.{open}(\"{name}\", JSONObject(){puts})").unwrap();
        }
        writeln!(out, "    }}").unwrap();
    }
    writeln!(out, "}}").unwrap();
    out
}

#[test]
fn saved_describe_and_kotlin_wrappers_match_the_registry() {
    let describe = t3_protocol::api::describe();
    check(DESCRIBE, &(serde_json::to_string_pretty(&describe).unwrap() + "\n"));
    let saved: Value = serde_json::from_str(&fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(DESCRIBE)).unwrap()).unwrap();
    check(KOTLIN, &kotlin(&saved));
}
