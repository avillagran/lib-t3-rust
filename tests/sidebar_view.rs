//! `sidebar::view`: ot3's sidebar rows over several environments and its
//! SidebarModel.js / ProjectModel.js pipeline. Vectors come from ot3's
//! `docs/protocol/fixtures/shell.json`; the checks from its `sidebar.qml`,
//! `sidebar-order.qml` and Rust tests.
use serde_json::{json, Value};
use t3_protocol::sidebar::view::{self, Grouping, ViewState};

const NULL: Value = Value::Null;

fn fixture() -> (Vec<Value>, i64) {
    let fx: Value = serde_json::from_str(include_str!("fixtures/sidebar-environments.json")).unwrap();
    (fx["environments"].as_array().unwrap().clone(), fx["nowMs"].as_i64().unwrap())
}
fn state<'a>(filter: &'a str, project: &'a str, query: &'a str, grouped: bool) -> ViewState<'a> {
    ViewState {
        filter,
        project,
        environment: None,
        query,
        grouped,
        subthreads_expanded: &NULL,
        selected: "",
        returns: &NULL,
        grouping: Grouping { mode: None, overrides: &NULL, native: &NULL },
    }
}
fn keys(rows: &Value) -> Vec<&str> {
    rows.as_array().unwrap().iter().map(|r| r["key"].as_str().unwrap()).collect()
}
fn ids(rows: &Value) -> Vec<&str> {
    rows.as_array().unwrap().iter().map(|r| r["id"].as_str().unwrap()).collect()
}
/// Rows already normalized, as ot3's QML tests feed them.
fn roster(threads: Value) -> Value {
    json!({"threads":threads,"projects":[],"environments":[],"stale":false})
}

#[test]
fn shared_contract_vectors() {
    let vectors: Value = serde_json::from_str(include_str!("fixtures/ot3-shell-vectors.json")).unwrap();
    for v in vectors.as_array().unwrap() {
        assert_eq!(json!(view::rows(&v["shell"], "fixture", v["nowMs"].as_i64().unwrap()).unwrap()), v["expected"], "{}", v["name"]);
    }
}

#[test]
fn delegated_task_ids_may_contain_dots_and_lineage_stays_apart() {
    let id = "thread:delegated-task:command%3Amcp%3Aabc%3Adelegate-task%3Aupload-1.2";
    assert_eq!(view::rows(&json!({"threads":[{"id":id,"title":"Upload"}]}), "fixture", 0).unwrap().len(), 1);
    let shell = json!({"threads":[
        {"id":"root","title":"Root"},
        {"id":"child","title":"Child","lineage":{"parentThreadId":"root","relationshipToParent":"subagent"}},
        {"id":"fork","title":"Fork","creationSource":"provider","lineage":{"parentThreadId":"root","relationshipToParent":"fork"}},
        {"id":"native","title":"Native","creationSource":"provider","lineage":{"parentThreadId":"root","relationshipToParent":"subagent"}},
        {"id":"mcp","title":"MCP","creationSource":"mcp","lineage":{"parentThreadId":"root","relationshipToParent":"subagent"}}
    ]});
    let rows = view::rows(&shell, "fixture", 0).unwrap();
    assert_eq!(rows.len(), 5);
    for row in &rows {
        assert_eq!(row["providerSubagent"] == true, row["id"] == "native");
    }
    assert_eq!(view::rows(&json!({"threads":[{"id":"a","title":"t"},{"id":"a","title":"t"}]}), "e", 0).unwrap_err(), "malformed");
}

#[test]
fn environments_merge_into_one_sidebar_and_keep_their_identity() {
    let (envs, now) = fixture();
    let snapshot = view::snapshot(&envs, now).unwrap();
    assert_eq!(snapshot["environments"], json!([{"id":"laptop","name":"Laptop","state":"ready"},{"id":"desktop","name":"Desktop","state":"ready"},{"id":"offline","name":"Offline","state":"unreachable"}]));
    assert_eq!(snapshot["stale"], true, "an unreachable environment marks the sidebar stale");
    let threads = snapshot["threads"].as_array().unwrap();
    assert!(threads.iter().all(|t| t["key"] == format!("{}/{}", t["environmentId"].as_str().unwrap(), t["id"].as_str().unwrap())));
    assert!(!threads.iter().any(|t| t["id"] == "l-archived"));
    let row = threads.iter().find(|t| t["id"] == "l-failed").unwrap();
    assert_eq!((row["environmentName"].as_str(), row["projectKey"].as_str(), row["project"].as_str()), (Some("Laptop"), Some("laptop/p-docs"), Some("docs site")));
    assert_eq!(row["runtimeError"], "Provider crashed");
    assert_eq!(row["projectMonogram"], "DS");
    assert_eq!(view::snapshot(&[json!({"id":"a"}), json!({"id":"a"})], now).unwrap_err(), "invalid_environments");
    assert_eq!(view::snapshot(&[json!({"id":"bad","shell":{"threads":7}})], now).unwrap()["environments"][0]["state"], "malformed");

    let v = view::view(&snapshot, &state("all", "", "", false), now);
    let groups = v["projectGroups"].as_array().unwrap();
    let repo = groups.iter().find(|g| g["key"] == "repository/github.com/example/app").unwrap();
    let members = repo["members"].as_array().unwrap().iter().map(|m| m["key"].as_str().unwrap()).collect::<Vec<_>>();
    assert_eq!(members, ["laptop/p-app", "desktop/p-app", "desktop/p-api"], "one repository across machines");
    let scoped = view::view(&snapshot, &state("all", "laptop/p-app", "", false), now);
    assert_eq!(scoped["projectScope"], json!(["laptop/p-app", "desktop/p-app", "desktop/p-api"]), "a member's key selects its group");
    assert!(keys(&scoped["sections"]["active"]).iter().all(|k| !k.contains("l-failed")));

    let mut desktop = state("all", "", "", false);
    desktop.environment = Some("desktop");
    let only = view::view(&snapshot, &desktop, now);
    assert!(keys(&only["sections"]["active"]).iter().all(|k| k.starts_with("desktop/")));
    assert_eq!(only["filterCounts"]["all"], 7, "desktop's roster, its subagent aside");
    assert_eq!(v["counts"], json!({"done":1,"failed":1,"waiting":3,"working":2}), "header pills count every environment");
}

#[test]
fn rows_carry_the_ten_details_with_label_role_and_time() {
    let (envs, now) = fixture();
    let snapshot = view::snapshot(&envs, now).unwrap();
    let v = view::view(&snapshot, &state("all", "", "", false), now);
    let row = |id: &str| v["threads"].as_array().unwrap().iter().find(|t| t["id"] == id).unwrap().clone();
    for (id, detail, label, role) in [
        ("l-request", "approval", "Approval", "warning"),
        ("l-question", "input", "Input", "input"),
        ("l-plan", "plan", "Plan ready", "waiting"),
        ("d-goal", "goal", "Goal", "working"),
        ("l-working", "working", "Working", "working"),
        ("l-failed", "failed", "Failed", "failed"),
        ("l-review", "done", "Done", "done"),
        ("d-background", "waiting", "Waiting", "muted"),
        ("d-seen", "", "", "muted"),
    ] {
        let r = row(id);
        assert_eq!((r["detail"].as_str(), r["statusLabel"].as_str(), r["statusRole"].as_str()), (Some(detail), Some(label), Some(role)), "{id}");
    }
    assert_eq!(row("l-working")["time"], json!({"kind":"elapsed","at":now-125_000,"label":"2m 5s"}));
    assert_eq!(row("l-snoozed")["time"]["kind"], "elapsed", "a working thread shows its run time even while snoozed");
    assert_eq!(row("l-fork")["time"]["label"], "1h");
    assert_eq!(view::time_label("countdown", now + 61_000, now), "2m");
    assert_eq!(view::time_label("countdown", now + 3 * 86_400_000, now), "3d");
    assert_eq!(view::time_label("age", 0, now), "—");
    assert_eq!(view::time_label("age", now - 30_000, now), "now");
    assert_eq!(view::time_label("elapsed", now - 59_999, now), "59s");
}

#[test]
fn shelves_filters_and_search_follow_ot3() {
    // ot3 tests/qml/sidebar.qml, on rows without a sidebarStatus (legacy buckets).
    let threads = json!([
        {"id":"a","key":"fixture/a","projectKey":"fixture/p1","title":"Active","status":"running","working":true},
        {"id":"b","key":"fixture/b","projectKey":"fixture/p2","title":"Done","status":"completed","pending":true,"reason":"review"},
        {"id":"c","key":"fixture/c","projectKey":"fixture/p2","title":"Error","status":"failed"},
        {"id":"d","key":"fixture/d","projectKey":"fixture/p3","title":"Parked","status":"completed","settled":true},
        {"id":"e","key":"fixture/e","projectKey":"fixture/p1","title":"Approval","status":"running","working":true,"pending":true,"reason":"request"},
        {"id":"sleep","key":"fixture/sleep","environmentId":"fixture","title":"Snoozed","snoozed":true}
    ]);
    assert_eq!(view::state(&threads[4]), "waiting", "attention takes priority");
    let snapshot = roster(threads);
    let v = view::view(&snapshot, &state("all", "", "", false), 0);
    assert_eq!(v["filterCounts"]["done"], 2, "done and settled remain distinct");
    assert_eq!(v["sectionCounts"], json!({"active":4,"working":0,"snoozed":1,"settled":1}), "snoozed shelf is separate from active and settled");
    assert_eq!(ids(&view::view(&snapshot, &state("settled", "", "", false), 0)["sections"]["settled"]), ["d"]);
    assert_eq!(ids(&view::view(&snapshot, &state("all", "", "parked", false), 0)["sections"]["settled"]), ["d"], "search reveals settled match");
    assert_eq!(view::view(&snapshot, &state("failed", "fixture/p1", "", false), 0)["sectionCounts"]["active"], 0, "project and state intersection");
    assert_eq!(ids(&view::view(&snapshot, &state("all", "", "error", false), 0)["sections"]["active"]), ["c"]);
    assert_eq!(ids(&view::view(&snapshot, &state("failed", "", "", false), 0)["sections"]["active"]), ["c"]);

    let family = roster(json!([
        {"id":"parent","title":"Parent"},
        {"id":"child","title":"Child","parentThreadId":"parent","relationshipToParent":"subagent","working":true},
        {"id":"fork","title":"Fork","parentThreadId":"parent","relationshipToParent":"fork"}
    ]));
    let v = view::view(&family, &state("all", "", "", false), 0);
    assert_eq!(v["sectionCounts"]["active"], 2, "subagents stay outside the official sidebar roster; forks remain");
    assert_eq!(v["filterCounts"]["working"], 0, "filter counts follow the visible roster");
}

#[test]
fn subthreads_nest_under_their_parent() {
    let tree = roster(json!([
        {"id":"parent","key":"e/parent","environmentId":"e","title":"Parent"},
        {"id":"child","key":"e/child","environmentId":"e","title":"Child","parentThreadId":"parent","relationshipToParent":"subagent","working":true,"updatedAt":2},
        {"id":"older","key":"e/older","environmentId":"e","title":"Older","parentThreadId":"parent","relationshipToParent":"subagent","status":"completed","updatedAt":1},
        {"id":"grand","key":"e/grand","environmentId":"e","title":"Grand","parentThreadId":"older","relationshipToParent":"subagent","status":"completed"},
        {"id":"orphan","key":"e/orphan","environmentId":"e","title":"Orphan","parentThreadId":"gone","relationshipToParent":"subagent"}
    ]));
    let nested = view::view(&tree, &state("all", "", "", false), 0)["sections"]["active"].clone();
    assert_eq!(ids(&nested), ["parent", "child", "older"], "children follow the parent, newest first; orphans stay out");
    assert_eq!(nested[0]["subthreads"], json!({"count":2,"working":1,"waiting":0,"failed":0,"done":1}));
    assert_eq!(nested[0]["subthreadsExpanded"], true, "a working child opens its parent");
    assert_eq!((nested[1]["depth"].as_u64(), nested[2]["subthreadsExpanded"].as_bool()), (Some(1), Some(false)), "grandchildren folded");
    let closed = json!({"e/parent":false});
    let mut s = state("all", "", "", false);
    s.subthreads_expanded = &closed;
    assert_eq!(view::view(&tree, &s, 0)["sectionCounts"]["active"], 1, "a collapsed parent hides its children");
    let mut s = state("all", "", "", false);
    s.selected = "e/grand";
    let deep = view::view(&tree, &s, 0)["sections"]["active"].clone();
    assert_eq!((deep[3]["id"].as_str(), deep[3]["depth"].as_u64()), (Some("grand"), Some(2)), "the selected grandchild stays visible");
}

#[test]
fn inbox_order_pins_and_working_returns() {
    // ot3 tests/qml/sidebar-order.qml (grouped sidebar).
    let row = |id: &str, inbox: i64, updated: i64| json!({"id":id,"key":format!("fixture/{id}"),"environmentId":"fixture","project":"Repo","title":id,"status":"completed","inboxAt":inbox,"updatedAt":updated});
    let mut running = row("running", 500, 500);
    running["working"] = json!(true);
    running["status"] = json!("running");
    running["lastSentAt"] = json!(400);
    let mut pinned = row("pinned", 10, 10);
    pinned["pinned"] = json!(true);
    let mut settled_old = row("settled-old", 10, 9000);
    settled_old["settled"] = json!(true);
    settled_old["settledSortAt"] = json!(100);
    let mut settled_recent = row("settled-recent", 10, 50);
    settled_recent["settled"] = json!(true);
    settled_recent["settledSortAt"] = json!(2000);
    let threads = vec![row("old", 100, 9000), row("recent", 2000, 50), running.clone(), pinned, settled_old, settled_recent];
    let v = view::view(&roster(json!(threads)), &state("all", "", "", true), 1);
    assert_eq!(ids(&v["sections"]["active"]), ["pinned", "recent", "old"], "recent completion outranks old metadata update, with pins first");
    assert_eq!(ids(&v["sections"]["working"]), ["running"]);
    assert_eq!(ids(&v["sections"]["settled"]), ["settled-recent", "settled-old"], "history ordered by completion/settlement");
    assert_eq!(v["returns"]["returned"], json!({}), "the first view does not invent returns");

    let mut threads = threads;
    threads[2] = json!({"id":"running","key":"fixture/running","environmentId":"fixture","title":"running","status":"completed","inboxAt":2500,"updatedAt":500});
    let mut s = state("all", "", "", true);
    let previous = v["returns"].clone();
    s.returns = &previous;
    let next = view::view(&roster(json!(threads)), &s, 3000);
    assert_eq!(ids(&next["sections"]["active"])[1], "running", "leaving Working surfaces immediately");
    assert_eq!(next["returns"]["returned"]["fixture/running"], 3000);
    let again = next["returns"].clone();
    s.returns = &again;
    assert_eq!(view::view(&roster(json!(threads)), &s, 9000)["returns"]["returned"], json!({"fixture/running":3000}), "an ordinary refresh does not reshuffle");
    let question = json!({"id":"q","key":"fixture/q","working":true,"pending":true,"reason":"request"});
    let with_question = view::observe_returns(&json!({"working":{"fixture/q":true},"returned":{}}), &[question], 4000);
    assert_eq!(with_question["returned"]["fixture/q"], 4000, "a mid-run question returns to the inbox");
    assert_eq!(view::observe_returns(&with_question, &[], 5000)["returned"], json!({}), "deleted returns are pruned");

    // Ungrouped: T3's manual order keys first in key order, then creation anchors.
    let keyed = roster(json!([
        {"id":"x","key":"e/x","activeAnchorAt":5},{"id":"k2","key":"e/k2","activeOrderKey":"b"},
        {"id":"y","key":"e/y","activeAnchorAt":9},{"id":"k1","key":"e/k1","activeOrderKey":"ab"}
    ]));
    assert_eq!(ids(&view::view(&keyed, &state("all", "", "", false), 0)["sections"]["active"]), ["y", "x", "k1", "k2"]);
}

#[test]
fn project_grouping_modes_and_overrides() {
    let projects = [
        json!({"key":"a/1","environmentId":"a","title":"app","workspaceRoot":"/w/app","repositoryIdentity":{"canonicalKey":"gh/app","rootPath":"/w/app"}}),
        json!({"key":"a/2","environmentId":"a","title":"api","workspaceRoot":"/w/app/api/","repositoryIdentity":{"origin":{"canonicalKey":"gh/app"},"rootPath":"/w/app"}}),
        json!({"key":"b/1","environmentId":"b","title":"solo","workspaceRoot":"C:\\work\\solo"}),
    ];
    let group_keys = |g: Grouping| view::groups(&projects, &g).iter().map(|g| g["key"].as_str().unwrap().to_string()).collect::<Vec<_>>();
    assert_eq!(group_keys(Grouping { mode: None, overrides: &NULL, native: &NULL }), ["repository/gh/app", "b/1"]);
    assert_eq!(group_keys(Grouping { mode: Some("repository_path"), overrides: &NULL, native: &NULL }), ["repository/gh/app", "repository/gh/app::api", "b/1"]);
    assert_eq!(group_keys(Grouping { mode: Some("separate"), overrides: &NULL, native: &NULL }), ["a/1", "a/2", "b/1"]);
    let overrides = json!({"a:/w/app/api":"separate","a:/w/app":"inherit"});
    assert_eq!(group_keys(Grouping { mode: None, overrides: &overrides, native: &NULL }), ["repository/gh/app", "a/2", "b/1"]);
    let native = json!({"mode":"separate","overrides":{"a:/w/app":"repository"}});
    assert_eq!(group_keys(Grouping { mode: None, overrides: &NULL, native: &native }), ["repository/gh/app", "a/2", "b/1"], "the desktop setting is the fallback");
    let groups = view::groups(&projects, &Grouping { mode: None, overrides: &NULL, native: &NULL });
    assert_eq!(view::scope("", &groups), view::Scope::All);
    assert_eq!(view::scope("a/2", &groups), view::Scope::Many(vec!["a/1".into(), "a/2".into()]));
    assert_eq!(view::scope("gone/1", &groups), view::Scope::One("gone/1".into()));
}

#[test]
fn sidebar_view_through_domain() {
    let (envs, now) = fixture();
    let v = t3_protocol::domain::call("sidebar.view", &json!({"environments":envs,"now":now,"grouped":true,"filter":"all"})).unwrap();
    let working = v["threads"].as_array().unwrap().iter().filter(|t| t["state"] == "working" && t["pinned"] != true && t["relationshipToParent"] != "subagent").count();
    assert_eq!(v["sectionCounts"]["working"], working, "grouped: unpinned working rows fold into the Working shelf");
    assert!(v["sections"]["active"].as_array().unwrap().iter().all(|r| r["state"] != "working" || r["depth"].as_u64().unwrap_or(0) > 0));
    assert!(v["returns"]["working"].is_object(), "grouped views hand back the returns to keep");
    let detail = t3_protocol::domain::call("sidebar.detail", &json!({"thread":v["threads"][0],"now":now})).unwrap();
    assert_eq!(detail["detail"], v["threads"][0]["detail"]);
    assert_eq!(t3_protocol::domain::call("sidebar.view", &json!({"environments":[],"now":now,"filter":"nope"})).unwrap_err(), "invalid_filter");
    assert_eq!(t3_protocol::domain::call("sidebar.snapshot", &json!({"environments":envs,"now":now})).unwrap()["working"], 8);
}
