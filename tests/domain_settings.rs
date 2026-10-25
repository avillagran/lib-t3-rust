//! Settings through `domain`: server preferences as fields and patches, and the
//! project scope over several environments (`scopedSettings.targetView`,
//! `writePatch` and `page` are the pure halves of ot3's settings executor).
use serde_json::{json, Value};
use t3_protocol::domain::call;

fn keys(v: &Value) -> Vec<&str> {
    let mut k = v.as_object().unwrap().keys().map(String::as_str).collect::<Vec<_>>();
    k.sort();
    k
}
fn settings() -> Value {
    json!({"defaultRuntimeMode":"full-access","defaultThreadEnvMode":null,"worktreeSubmodules":null,"enableAgentBrowserAccess":true,
        "defaultModelSelection":{"instanceId":"claude","model":"opus"},"idleCompaction":"onReturn","projectSettingsFolded":true,
        "projectSettingsOverrides":{"p":{"defaultRuntimeMode":"approval-required"}},"providerInstances":{},"apiKey":"PRIVATE"})
}
fn config(project_overrides: bool) -> Value {
    json!({"environment":{"capabilities":{"projectSettingsOverrides":project_overrides}},
        "providers":[{"instanceId":"claude","driver":"claudeAgent","enabled":true,"installed":true,"models":[{"slug":"opus","name":"Opus"}]}]})
}

#[test]
fn native_settings_fields_and_patches() {
    let snapshot = call("nativeSettings.snapshot", &json!({"settings":settings()})).unwrap();
    assert_eq!(keys(&snapshot), ["fields"]);
    let field = |key: &str| snapshot["fields"].as_array().unwrap().iter().find(|f| f["key"] == key).cloned().unwrap();
    assert_eq!(field("defaultRuntimeMode")["kind"], "choice");
    assert_eq!(field("defaultModelSelection")["value"], json!({"instanceId":"claude","model":"opus"}));
    assert!(field("defaultRuntimeMode")["description"].as_str().unwrap().starts_with("Default permissions"));
    assert!(!snapshot.to_string().contains("PRIVATE"));
    assert_eq!(snapshot, t3_protocol::native_settings::snapshot(&settings()));

    assert_eq!(call("nativeSettings.patch", &json!({"input":{"key":"defaultRuntimeMode","value":"auto"}})).unwrap(), json!({"patch":{"defaultRuntimeMode":"auto"}}));
    assert_eq!(call("nativeSettings.patch", &json!({"input":{"key":"defaultModelSelection","value":null}})).unwrap(), json!({"patch":{"defaultModelSelection":null}}));
    assert_eq!(call("nativeSettings.patch", &json!({"input":{"key":"defaultRuntimeMode","value":"yolo"}})).unwrap_err(), "invalid_settings");
    assert_eq!(call("nativeSettings.patch", &json!({"input":{"key":"defaultModelSelection","value":{"instanceId":"gone","model":"x"}},"config":config(true)})).unwrap_err(), "model_unavailable");
    assert!(call("nativeSettings.description", &json!({"key":"defaultThreadEnvMode"})).unwrap().as_str().unwrap().starts_with("Where new threads start"));
    assert_eq!(call("nativeSettings.description", &json!({"key":"nope"})).unwrap(), "");
    assert_eq!(call("nativeSettings.modelSelection", &json!({"selection":{"provider":"codex","model":"gpt","options":[{"id":"effort","value":"high"}]}})).unwrap(), json!({"instanceId":"codex","model":"gpt","options":[{"id":"effort","value":"high"}]}));
    assert_eq!(call("nativeSettings.modelSelection", &json!({"selection":{"model":"gpt"}})).unwrap_err(), "invalid_model");
    assert_eq!(call("nativeSettings.idleCompaction", &json!({"settings":settings()})).unwrap(), "onReturn");
    assert!(call("nativeSettings.idleCompaction", &json!({"settings":{}})).unwrap().is_null());
    assert_eq!(call("nativeSettings.idleCompaction", &json!({"settings":settings(),"instanceId":"claude"})).unwrap(), "onReturn");
    assert_eq!(call("nativeSettings.idleCompactionPatch", &json!({"settings":settings(),"patch":{"patch":{"idleCompaction":"off"}}})).unwrap(), json!({"patch":{"idleCompaction":"off"}}));
    let defaults = call("nativeSettings.threadDefaults", &json!({"settings":settings(),"projectId":"p"})).unwrap();
    assert_eq!(defaults, json!({"defaultModelSelection":{"instanceId":"claude","model":"opus"},"modelDefaultConfigured":true,"defaultRuntimeMode":"approval-required"}));
}

#[test]
fn scoped_settings_effective_patches_and_files() {
    let project = json!({"id":"p","workspaceRoot":"/r"});
    let e = call("scopedSettings.effective", &json!({"settings":settings(),"project":project,"file":{"defaultThreadEnvMode":"worktree"}})).unwrap();
    assert_eq!(keys(&e), ["overrides", "settings", "sources"]);
    assert_eq!((e["settings"]["defaultRuntimeMode"].as_str(), e["sources"]["defaultRuntimeMode"].as_str()), (Some("approval-required"), Some("project")));
    assert_eq!((e["settings"]["defaultThreadEnvMode"].as_str(), e["sources"]["defaultThreadEnvMode"].as_str()), (Some("worktree"), Some("t3.json")));
    assert_eq!(call("scopedSettings.effective", &json!({"settings":settings()})).unwrap()["sources"]["defaultRuntimeMode"], "environment");
    let patch = call("scopedSettings.projectPatch", &json!({"settings":settings(),"project":project,"key":"defaultRuntimeMode","patch":{"patch":{"defaultRuntimeMode":"auto"}}})).unwrap();
    assert_eq!(patch, json!({"patch":{"projectSettingsOverrides":{"p":{"defaultRuntimeMode":"auto"}}}}));
    let cleared = call("scopedSettings.projectPatch", &json!({"settings":settings(),"project":project,"key":"defaultRuntimeMode","patch":{},"clear":true})).unwrap();
    assert_eq!(cleared, json!({"patch":{"projectSettingsOverrides":{"p":null}}}));
    assert_eq!(call("scopedSettings.projectPatch", &json!({"settings":settings(),"project":project,"key":"enableProviderUpdateChecks","patch":{}})).unwrap_err(), "setting_environment_only");
    assert_eq!(call("scopedSettings.projectFile", &json!({"contents":"{// note\n\"worktreeSubmodules\":\"none\",}"})).unwrap(), json!({"worktreeSubmodules":"none"}));
    assert!(call("scopedSettings.projectFile", &json!({"contents":"{\"defaultThreadEnvMode\":\"future\"}"})).unwrap().is_null());
    let targets = call("scopedSettings.validatedTargets", &json!({"scope":{"targets":[{"environment":"a","projectId":"p"},{"environment":"b","projectId":"p"}]}})).unwrap();
    assert_eq!(targets.as_array().unwrap().len(), 2);
    assert_eq!(call("scopedSettings.validatedTargets", &json!({"scope":{"targets":[{"environment":"a","projectId":null},{"environment":"b","projectId":"p"}]}})).unwrap_err(), "settings_scope_unavailable");
    assert_eq!(call("scopedSettings.writeWasDropped", &json!({"next":{"a":true},"before":{"a":false},"saved":{"a":false}})).unwrap(), true);
    assert_eq!(call("scopedSettings.writeWasDropped", &json!({"next":{"a":true},"before":{"a":false},"saved":{"a":true}})).unwrap(), false);
    let mut disabled = settings();
    disabled["projectSettingsOverrides"]["p"]["providerInstances"] = json!({"claude":{"enabled":false}});
    let adjusted = call("scopedSettings.projectCatalogConfig", &json!({"config":config(true),"settings":disabled,"project":project})).unwrap();
    assert_eq!(adjusted["providers"][0]["enabled"], false);
}

#[test]
fn scoped_settings_target_views_writes_and_page() {
    let project = json!({"id":"p","workspaceRoot":"/r"});
    let target = json!({"environment":"laptop","projectId":"p"});
    let view = call("scopedSettings.targetView", &json!({"target":target,"config":config(true),"settings":settings(),"project":project,"writeAllowed":true})).unwrap();
    assert_eq!(keys(&view), ["catalog", "fields", "storageCapabilities", "target", "writeAllowed"]);
    let mode = view["fields"].as_array().unwrap().iter().find(|f| f["key"] == "defaultRuntimeMode").unwrap();
    assert_eq!((mode["source"].as_str(), mode["overridden"].as_bool(), mode["scopeWritable"].as_bool(), mode["projectScoped"].as_bool()), (Some("project"), Some(true), Some(true), Some(true)));
    // An environment target: every field is the environment's, device access is not writable here.
    let env = call("scopedSettings.targetView", &json!({"target":{"environment":"desk","projectId":null},"config":config(false),"settings":settings(),"writeAllowed":true})).unwrap();
    let mode = env["fields"].as_array().unwrap().iter().find(|f| f["key"] == "defaultRuntimeMode").unwrap();
    assert_eq!((mode["source"].as_str(), mode["overridden"].as_bool(), mode["projectScoped"].as_bool()), (Some("environment"), Some(false), Some(false)));

    let input = json!({"key":"defaultRuntimeMode","action":"update","value":"auto"});
    let write = |config: Value, project: Option<&Value>, allowed: bool| {
        let mut args = json!({"input":input,"config":config,"settings":settings(),"writeAllowed":allowed});
        if let Some(p) = project {
            args["project"] = p.clone();
        }
        call("scopedSettings.writePatch", &args)
    };
    assert_eq!(write(config(true), Some(&project), true).unwrap(), json!({"patch":{"projectSettingsOverrides":{"p":{"defaultRuntimeMode":"auto"}}}}));
    assert_eq!(write(config(true), None, true).unwrap(), json!({"patch":{"defaultRuntimeMode":"auto"}}));
    assert_eq!(write(config(false), Some(&project), true).unwrap_err(), "project_settings_unsupported");
    assert_eq!(write(config(true), Some(&project), false).unwrap_err(), "settings_write_forbidden");
    let clear = call("scopedSettings.writePatch", &json!({"input":{"key":"defaultRuntimeMode","action":"clear"},"config":config(true),"settings":settings(),"writeAllowed":true})).unwrap_err();
    assert_eq!(clear, "invalid_settings");

    let other = call("scopedSettings.targetView", &json!({"target":{"environment":"desk","projectId":"p"},"config":config(true),"settings":{"defaultRuntimeMode":"full-access"},"project":project,"writeAllowed":false})).unwrap();
    let aggregate = call("scopedSettings.aggregate", &json!({"views":[view, other]})).unwrap();
    let mode = aggregate["fields"].as_array().unwrap().iter().find(|f| f["key"] == "defaultRuntimeMode").unwrap();
    assert_eq!((mode["mixed"].as_bool(), mode["source"].as_str(), mode["scopeWritable"].as_bool()), (Some(true), Some("mixed"), Some(false)));
    let scope = json!({"targets":[target, {"environment":"desk","projectId":"p"}, {"environment":"gone","projectId":"p"}]});
    let page = call("scopedSettings.page", &json!({"scope":scope,"views":[view, other],"errors":[{"target":{"environment":"gone","projectId":"p"},"error":"settings_scope_unavailable"}],"requested":3})).unwrap();
    assert_eq!(keys(&page), ["complete", "errors", "fields", "modelCatalog", "savedTargets", "scope", "storageScopes", "targets", "writable"]);
    assert_eq!((page["complete"].as_bool(), page["writable"].as_bool(), page["targets"].as_array().map(Vec::len)), (Some(false), Some(false), Some(2)));
    assert_eq!(page["fields"], aggregate["fields"]);
    let alone = call("scopedSettings.page", &json!({"scope":{"targets":[target]},"views":[view],"requested":1})).unwrap();
    assert_eq!((alone["complete"].as_bool(), alone["writable"].as_bool()), (Some(true), Some(true)));
}
