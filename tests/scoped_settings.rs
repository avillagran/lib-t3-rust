use t3_protocol::scoped_settings as s;
use serde_json::{json,Value};
#[test]
fn native_project_override_precedence_sources_and_folded_reset(){
 let raw=json!({"projectSettingsFolded":true,"defaultThreadEnvMode":null,"worktreeSubmodules":null,"defaultRuntimeMode":"full-access","defaultModelSelection":{"instanceId":"global","model":"one"},"providerInstances":{"disabled":{"enabled":false}},"projectSettingsOverrides":{"p":{"defaultModelSelection":{"instanceId":"disabled","model":"other"},"defaultRuntimeMode":"approval-required"}}});
 let project=json!({"id":"p","defaultThreadEnvMode":"worktree"});let file=json!({"defaultThreadEnvMode":"local","worktreeSubmodules":"none"});
 let(e,sources,_)=s::effective(&raw,Some(&project),Some(&file));assert_eq!(e["defaultModelSelection"]["instanceId"],"global");assert_eq!(e["defaultRuntimeMode"],"approval-required");assert_eq!(sources["defaultRuntimeMode"],"project");assert_eq!(e["defaultThreadEnvMode"],"local");assert_eq!(sources["defaultThreadEnvMode"],"t3.json");
 let mut legacy=raw.clone();legacy["projectSettingsFolded"]=json!(false);assert_eq!(s::effective(&legacy,Some(&project),None).0["defaultThreadEnvMode"],"worktree");
 let mut automatic=raw.clone();automatic["projectSettingsOverrides"]["p"]["defaultModelSelection"]=Value::Null;assert!(s::effective(&automatic,Some(&project),None).0["defaultModelSelection"].is_null());
}
#[test]
fn project_patch_keeps_other_overrides_and_completes_native_nested_values(){
 let raw=json!({"projectSettingsFolded":true,"sourceControlWritingStyle":{"mode":"repo_conventions","customInstructions":"keep","followChangeRequestTemplates":true},"projectSettingsOverrides":{"p":{"defaultModelSelection":null,"providerInstances":{"custom":{"enabled":true}}},"unrelated":{"defaultRuntimeMode":"full-access"}}});let project=json!({"id":"p"});
 let p=s::project_patch(&raw,&project,"sourceControlWritingStyle.mode",&json!({"patch":{"sourceControlWritingStyle":{"mode":"custom"}}}),false).unwrap();
 let e=&p["patch"]["projectSettingsOverrides"]["p"];assert_eq!(e["sourceControlWritingStyle"]["customInstructions"],"keep");assert_eq!(e["sourceControlWritingStyle"]["mode"],"custom");assert_eq!(e["providerInstances"]["custom"]["enabled"],true);assert!(e["defaultModelSelection"].is_null());assert!(p["patch"]["projectSettingsOverrides"].get("unrelated").is_none());
 let p=s::project_patch(&raw,&project,"defaultModelSelection",&json!({}),true).unwrap();assert!(p["patch"]["projectSettingsOverrides"]["p"].get("defaultModelSelection").is_none());
 assert!(s::project_patch(&raw,&project,"enableProviderUpdateChecks",&json!({"patch":{"enableProviderUpdateChecks":false}}),false).is_err());
}
#[test]
fn scopes_mixed_values_and_jsonc_keep_data_and_reject_invalid_files(){
 let fields=|v,source|json!([{"key":"defaultRuntimeMode","value":v,"source":source,"scopeWritable":true,"overridden":source=="project"}]);
 let a=s::aggregate(&[json!({"fields":fields("full-access","project")}),json!({"fields":fields("approval-required","environment")})]);assert_eq!(a["fields"][0]["mixed"],true);assert_eq!(a["fields"][0]["source"],"mixed");assert_eq!(a["fields"][0]["overridden"],true);
 let f=s::project_file("{/* note */\"defaultThreadEnvMode\":\"worktree\",// note\n\"worktreeSubmodules\":\"none\",\"scripts\":[{\"name\":\"dev\",\"command\":\"echo https://host/* literal */\",},],}").unwrap();assert_eq!(f["scripts"][0]["command"],"echo https://host/* literal */");assert_eq!(f["defaultThreadEnvMode"],"worktree");
 assert!(s::project_file("{\"defaultThreadEnvMode\":\"future\"}").is_none());assert!(s::project_file("{\"scripts\":[{\"command\":\"echo\"}],\"worktreeSubmodules\":\"none\"}").is_none());
 // An unknown environment ("gone") is the host's check; these fail on shape alone.
 for targets in [json!([]),json!([{"environment":"env","projectId":null},{"environment":"env","projectId":"p"}]),json!([{"environment":"env","projectId":null},{"environment":"env","projectId":null}])]{assert!(s::validated_targets(&json!({"action":"update","key":"defaultRuntimeMode","value":"full-access","scope":{"targets":targets}})).is_err())}
 assert_eq!(s::validated_targets(&json!({"scope":{"targets":[{"environment":"gone","projectId":null}]}})).unwrap().len(),1);
}
