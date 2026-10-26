//! `awareness.view`: T3's agent phases, the ongoing card and deduplicated
//! alerts over the same environments `sidebar.view` takes. The shells are
//! `sidebar.view`'s fixtures (`sidebar-environments.json`, ot3's
//! `ot3-shell-vectors.json`) plus small ones built here for each transition.
use chrono::{DateTime, SecondsFormat};
use serde_json::{json, Value};
use t3_protocol::awareness::{self, FINISHED_TTL_MS, RUNNING_TTL_MS, WAITING_TTL_MS};
use t3_protocol::domain::call;

/// 2026-10-15T04:00:00Z, the shared fixtures' `nowMs`.
const NOW: i64 = 1_792_036_800_000;
const MIN: i64 = 60_000;

fn iso(ms: i64) -> String {
    DateTime::from_timestamp_millis(ms).unwrap().to_rfc3339_opts(SecondsFormat::Millis, true)
}
fn fixture() -> Vec<Value> {
    let fx: Value = serde_json::from_str(include_str!("fixtures/sidebar-environments.json")).unwrap();
    assert_eq!(fx["nowMs"], NOW);
    fx["environments"].as_array().unwrap().clone()
}
fn thread(id: &str, status: &str, updated: i64) -> Value {
    json!({"id":id,"title":format!("Thread {id}"),"projectId":"p","status":status,"updatedAt":iso(updated),"latestRunId":format!("run-{id}"),"lineage":{}})
}
fn running(id: &str, started: i64) -> Value {
    let mut t = thread(id, "running", started);
    t["activityRunStartedAt"] = json!(iso(started));
    t
}
fn approval(id: &str, at: i64) -> Value {
    let mut t = running(id, at - MIN);
    t["updatedAt"] = json!(iso(at));
    t["pendingRuntimeRequest"] = json!({"id":format!("req-{id}"),"kind":"command_approval","createdAt":iso(at)});
    t
}
fn question(id: &str, at: i64) -> Value {
    let mut t = approval(id, at);
    t["pendingRuntimeRequest"]["kind"] = json!("user_input");
    t
}
fn done(id: &str, at: i64) -> Value {
    let mut t = thread(id, "completed", at);
    t["latestRunCompletedAt"] = json!(iso(at));
    t
}
fn failed(id: &str, at: i64) -> Value {
    let mut t = done(id, at);
    t["status"] = json!("failed");
    t
}
fn env(id: &str, threads: Vec<Value>) -> Value {
    json!({"id":id,"name":format!("Host {id}"),"shell":{"projects":[{"id":"p","title":"app"}],"threads":threads}})
}
fn view(environments: Vec<Value>, now: i64, baseline: &Value, extra: Value) -> Value {
    let mut args = json!({"environments":environments,"now":now});
    if !baseline.is_null() {
        args["baseline"] = baseline.clone();
    }
    for (k, v) in extra.as_object().into_iter().flatten() {
        args[k] = v.clone();
    }
    call("awareness.view", &args).unwrap()
}
fn ids(list: &Value, field: &str) -> Vec<String> {
    list.as_array().unwrap().iter().map(|v| v["route"][field].as_str().unwrap().to_string()).collect()
}
fn alert_threads(v: &Value) -> Vec<String> {
    ids(&v["alerts"], "threadId")
}
/// The baseline of a first, silent observation.
fn seed(environments: Vec<Value>, now: i64) -> Value {
    let v = view(environments, now, &Value::Null, json!({}));
    assert_eq!(v["alerts"], json!([]));
    v["baseline"].clone()
}

#[test]
fn phases_follow_t3_on_the_shared_shells() {
    let envs = fixture();
    let phase = |env: &str, id: &str| {
        let e = envs.iter().find(|e| e["id"] == env).unwrap();
        awareness::phase(e["shell"]["threads"].as_array().unwrap().iter().find(|t| t["id"] == id).unwrap())
    };
    for (env, id, expected) in [
        ("laptop", "l-working", Some("running")),
        ("laptop", "l-request", Some("waiting_for_approval")),
        ("laptop", "l-failed", Some("failed")),
        ("laptop", "l-review", Some("completed")),
        // T3 has no phase for a ready plan: the run completed.
        ("laptop", "l-plan", Some("completed")),
        // Only an explicit settled override hides a finished thread in T3, not settledAt.
        ("laptop", "l-settled", Some("failed")),
        ("laptop", "l-snoozed", Some("running")),
        ("laptop", "l-child", None),
        ("laptop", "l-grandchild", None),
        ("desktop", "d-waiting", Some("running")),
        // A background task without a kind holds nothing.
        ("desktop", "d-background", Some("completed")),
        ("desktop", "d-settled", None),
        ("desktop", "d-active", Some("completed")),
        ("desktop", "d-goal", Some("running")),
        ("desktop", "d-child", None),
    ] {
        assert_eq!(phase(env, id), expected, "{env}/{id}");
    }
    let vectors: Value = serde_json::from_str(include_str!("fixtures/ot3-shell-vectors.json")).unwrap();
    let mut seen = 0;
    for v in vectors.as_array().unwrap() {
        for t in v["shell"]["threads"].as_array().unwrap() {
            let p = awareness::phase(t);
            assert!(p.is_none() || awareness::priority(p.unwrap()) <= 3, "{}", v["name"]);
            seen += 1;
        }
    }
    assert!(seen >= 13);

    let base = thread("t", "running", NOW);
    let with = |patch: Value| {
        let mut t = base.clone();
        for (k, v) in patch.as_object().unwrap() {
            t[k] = v.clone();
        }
        awareness::phase(&t)
    };
    assert_eq!(with(json!({"pendingRuntimeRequest":{"kind":"user_input","blocking":false}})), Some("running"), "an async question does not stop a run");
    assert_eq!(with(json!({"status":"completed","pendingRuntimeRequest":{"kind":"user_input","blocking":false}})), Some("waiting_for_input"));
    assert_eq!(with(json!({"pendingRuntimeRequest":{"kind":"auth_refresh"}})), Some("running"));
    assert_eq!(with(json!({"status":"preparing"})), Some("starting"));
    assert_eq!(with(json!({"status":"idle","activityRunStatus":"waiting"})), Some("running"));
    assert_eq!(with(json!({"status":"completed","latestRunStopRequested":true})), None);
    assert_eq!(with(json!({"status":"completed","pendingBackgroundTasks":[{"kind":"monitor"}]})), Some("running"));
    assert_eq!(with(json!({"status":"completed","pendingBackgroundTasks":[{"kind":"command"}]})), Some("completed"));
    assert_eq!(with(json!({"status":"failed","settledOverride":"settled","pendingBackgroundTasks":[{"kind":"subagent"}]})), Some("failed"));
    assert_eq!(with(json!({"status":"failed","settledOverride":"settled"})), None);
    for status in ["idle", "queued", "interrupted", "cancelled", "rolled_back", "nonsense"] {
        assert_eq!(with(json!({"status":status})), None, "{status}");
    }
}

#[test]
fn cold_start_is_silent_and_counts_match_the_sidebar() {
    let envs = fixture();
    let v = view(envs.clone(), NOW, &Value::Null, json!({}));
    let sidebar = call("sidebar.view", &json!({"environments":envs,"now":NOW})).unwrap();
    assert_eq!(v["counts"], sidebar["counts"]);
    assert_eq!((v["alerts"].clone(), v["summaries"].clone()), (json!([]), json!([])), "no baseline: first observation");
    // Only pending, top-level threads of ready environments; the offline one has no entry.
    let baseline = &v["baseline"];
    assert_eq!(baseline["version"], 1);
    let mut laptop = baseline["environments"]["laptop"].as_object().unwrap().keys().cloned().collect::<Vec<_>>();
    laptop.sort();
    assert_eq!(laptop, ["l-failed", "l-plan", "l-question", "l-request", "l-review"], "settled and snoozed are never pending");
    assert!(baseline["environments"].get("offline").is_none());
    // Priority order: approval, failed, running, then results; the broken date has expired.
    let rows = ids(&v["rows"], "threadId");
    assert_eq!(rows.first().map(String::as_str), Some("l-request"));
    assert!(!rows.contains(&"d-undated".into()) && !rows.contains(&"l-child".into()) && !rows.contains(&"d-settled".into()));
    let phases = v["rows"].as_array().unwrap().iter().map(|r| awareness::priority(r["phase"].as_str().unwrap())).collect::<Vec<_>>();
    assert!(phases.windows(2).all(|w| w[0] <= w[1]), "{phases:?}");
    let row = v["rows"].as_array().unwrap().iter().find(|r| r["route"]["threadId"] == "l-plan").unwrap();
    assert_eq!((row["phase"].as_str(), row["status"].as_str(), row["host"].as_str(), row["project"].as_str()), (Some("completed"), Some("Plan ready"), Some("Laptop"), Some("app")));
    let card = &v["card"];
    assert_eq!((card["active"].as_bool(), card["phase"].as_str(), card["chip"]["kind"].as_str()), (Some(true), Some("waiting_for_approval"), Some("approve")));
    assert_eq!(card["rows"].as_array().unwrap().len(), 5);
    assert_eq!(card["title"], "2 need you", "l-request and l-question");
    assert_eq!(card["route"], json!({"environmentId":"laptop","threadId":"l-request"}));

    // `cold` ignores a stored baseline: following starts over without alerts.
    let mut changed = fixture();
    changed[0]["shell"]["threads"][0] = approval("l-working", NOW);
    let cold = view(changed.clone(), NOW, baseline, json!({"lifecycle":"cold"}));
    assert_eq!(cold["alerts"], json!([]));
    assert!(cold["baseline"]["environments"]["laptop"].get("l-working").is_some());
    assert_eq!(alert_threads(&view(changed, NOW, baseline, json!({"lifecycle":"resumed"}))), ["l-working"]);
    // Another version, or a baseline that is not one, resets silently.
    let old = json!({"version":0,"environments":baseline["environments"]});
    let mut changed = fixture();
    changed[0]["shell"]["threads"][0] = approval("l-working", NOW);
    assert_eq!(view(changed.clone(), NOW, &old, json!({}))["alerts"], json!([]));
    assert_eq!(view(changed, NOW, &json!({"anything":1}), json!({}))["alerts"], json!([]));
}

#[test]
fn transitions_alert_once_and_retries_repeat_the_same_ids() {
    let b = seed(vec![env("laptop", vec![running("a", NOW - 5 * MIN), running("b", NOW - 5 * MIN)])], NOW - MIN);
    let next = vec![env("laptop", vec![approval("a", NOW), done("b", NOW - 30_000)])];
    let v = view(next.clone(), NOW, &b, json!({}));
    assert_eq!(alert_threads(&v), ["a", "b"], "attention first, then finished");
    let a = &v["alerts"][0];
    let revision = v["baseline"]["environments"]["laptop"]["a"].as_str().unwrap();
    assert_eq!(a["id"], serde_json::to_string(&json!(["laptop", "a", "waiting_for_approval", revision])).unwrap());
    assert_eq!((a["group"].as_str(), a["title"].as_str(), a["body"].as_str(), a["status"].as_str()), (Some("attention"), Some("Thread a"), Some("Approval: app"), Some("Approval")));
    assert!(a.get("expiresAt").is_none(), "an attention alert stays until it is handled");
    let f = &v["alerts"][1];
    assert_eq!((f["group"].as_str(), f["phase"].as_str(), f["body"].as_str()), (Some("finished"), Some("completed"), Some("Done: app")));
    assert_eq!(f["expiresAt"], NOW - 30_000 + FINISHED_TTL_MS);
    assert_eq!(v["summaries"], json!([]), "one alert per group: no summary");

    // Retry with the same input: the same alerts and ids, so a host posting by id never doubles.
    assert_eq!(view(next.clone(), NOW, &b, json!({})), v);
    // With the returned baseline the same shells are seen.
    let again = view(next.clone(), NOW + 1_000, &v["baseline"], json!({}));
    assert_eq!(again["alerts"], json!([]));
    assert_eq!(again["baseline"], v["baseline"]);
    // A second request on a thread already waiting is a new revision: it alerts again.
    let mut second = next;
    second[0]["shell"]["threads"][0]["pendingRuntimeRequest"]["id"] = json!("req-a-2");
    let v2 = view(second, NOW + 2_000, &again["baseline"], json!({}));
    assert_eq!(alert_threads(&v2), ["a"]);
    assert_ne!(v2["alerts"][0]["id"], v["alerts"][0]["id"]);
}

#[test]
fn reconnecting_after_an_outage_replays_nothing() {
    let shells = || vec![env("laptop", vec![approval("a", NOW - 10 * MIN), running("b", NOW - 10 * MIN), running("c", NOW - 10 * MIN)])];
    let first = seed(shells(), NOW - 10 * MIN);
    // Unreachable with its cached shell: rows stay (live ones as stale), no alert, baseline untouched.
    let mut down = shells();
    down[0]["state"] = json!("unreachable");
    down[0]["shell"]["threads"][2] = approval("c", NOW - 2 * MIN);
    let outage = view(down, NOW - 2 * MIN, &first, json!({}));
    assert_eq!(outage["alerts"], json!([]));
    assert_eq!(outage["baseline"], first, "neither withdrawn nor advanced");
    assert!(outage["rows"].as_array().unwrap().iter().all(|r| r["phase"] == "stale" && r["status"] == "Waiting" && r.get("since").is_none()));
    assert_eq!(outage["card"]["chip"]["kind"], "working", "Android's chip has no waiting kind");
    // Back with what it had: silent.
    assert_eq!(view(shells(), NOW, &outage["baseline"], json!({}))["alerts"], json!([]));
    // Back with what happened meanwhile: the old result is consumed silently,
    // the request still waiting for the user alerts.
    let mut meanwhile = shells();
    meanwhile[0]["shell"]["threads"][1] = done("b", NOW - 6 * MIN);
    meanwhile[0]["shell"]["threads"][2] = approval("c", NOW - 4 * MIN);
    let back = view(meanwhile.clone(), NOW, &outage["baseline"], json!({}));
    assert_eq!(alert_threads(&back), ["c"]);
    assert!(back["baseline"]["environments"]["laptop"].get("b").is_some(), "the stale result is consumed");
    assert_eq!(view(meanwhile, NOW + MIN, &back["baseline"], json!({}))["alerts"], json!([]));
}

#[test]
fn the_thread_on_screen_is_consumed() {
    let b = seed(vec![env("laptop", vec![running("a", NOW - MIN), running("b", NOW - MIN)])], NOW - MIN);
    let next = vec![env("laptop", vec![question("a", NOW), question("b", NOW)])];
    let v = view(next.clone(), NOW, &b, json!({"onScreen":{"environmentId":"laptop","threadId":"a"}}));
    assert_eq!(alert_threads(&v), ["b"]);
    assert_eq!(v["alerts"][0]["phase"], "waiting_for_input");
    assert!(v["baseline"]["environments"]["laptop"].get("a").is_some());
    // Leaving the thread does not bring its alert back.
    assert_eq!(view(next, NOW + MIN, &v["baseline"], json!({}))["alerts"], json!([]));
    // An on-screen thread of another environment suppresses nothing here.
    let b = seed(vec![env("laptop", vec![running("a", NOW - MIN)])], NOW - MIN);
    let v = view(vec![env("laptop", vec![question("a", NOW)])], NOW, &b, json!({"onScreen":{"environmentId":"desktop","threadId":"a"}}));
    assert_eq!(alert_threads(&v), ["a"]);
}

#[test]
fn several_alerts_in_a_group_get_one_summary() {
    let b = seed(vec![env("laptop", vec![running("a", NOW - MIN), running("b", NOW - MIN)]), env("desktop", vec![running("c", NOW - MIN), running("d", NOW - MIN)])], NOW - MIN);
    let mut plan = done("b", NOW - 20 * MIN);
    plan["hasActionableProposedPlan"] = json!(true);
    let next = vec![env("laptop", vec![approval("a", NOW), plan]), env("desktop", vec![done("c", NOW - 10_000), failed("d", NOW - 20_000)])];
    let v = view(next.clone(), NOW, &b, json!({}));
    assert_eq!(alert_threads(&v), ["a", "b", "d", "c"], "attention, then failed before done");
    // A ready plan is attention (ot3's waiting), answered by the user.
    assert_eq!((v["alerts"][1]["group"].as_str(), v["alerts"][1]["phase"].as_str(), v["alerts"][1]["body"].as_str()), (Some("attention"), Some("waiting_for_input"), Some("Plan ready: app")));
    assert_eq!(
        v["summaries"],
        json!([
            {"group":"attention","title":"2 agents need attention","body":"Thread a, Thread b","lines":["Thread a","Thread b"],"route":{"environmentId":"laptop","threadId":"a"}},
            {"group":"finished","title":"2 agents finished","body":"Thread d, Thread c","lines":["Thread d","Thread c"],"route":{"environmentId":"desktop","threadId":"d"}}
        ])
    );
    let es = view(next, NOW, &b, json!({"locale":"es_CL.UTF-8"}));
    assert_eq!(es["summaries"][0]["title"], "2 agentes necesitan atención");
    assert_eq!(es["summaries"][1]["title"], "2 agentes terminaron");
    assert_eq!(es["alerts"][0]["body"], "Aprobación: app");
}

#[test]
fn a_result_older_than_two_minutes_does_not_alert() {
    let b = seed(vec![env("laptop", vec![running("a", NOW - 10 * MIN), running("b", NOW - 10 * MIN)])], NOW - 10 * MIN);
    let next = vec![env("laptop", vec![done("a", NOW - 2 * MIN - 1), failed("b", NOW - 2 * MIN)])];
    let v = view(next, NOW, &b, json!({}));
    assert_eq!(alert_threads(&v), ["b"], "fresh up to exactly two minutes");
    assert!(v["baseline"]["environments"]["laptop"].get("a").is_some(), "consumed, never alerted later");
    // Both still show on the finished card for 15 minutes.
    assert_eq!((v["card"]["active"].as_bool(), v["card"]["title"].as_str()), (Some(false), Some("Finished, 1 failed")));
    assert!(v["card"].get("chip").is_none());
    // Preferences: a finished alert turned off is consumed too.
    let b = seed(vec![env("laptop", vec![running("a", NOW - MIN), running("q", NOW - MIN)])], NOW - MIN);
    let next = vec![env("laptop", vec![done("a", NOW), question("q", NOW)])];
    let off = view(next.clone(), NOW, &b, json!({"preferences":{"completion":false,"input":false}}));
    assert_eq!(off["alerts"], json!([]));
    assert_eq!(view(next, NOW, &off["baseline"], json!({}))["alerts"], json!([]));
}

#[test]
fn settled_snoozed_and_subagent_threads_never_alert() {
    let b = seed(vec![env("laptop", vec![running("s", NOW - MIN), running("z", NOW - MIN), running("k", NOW - MIN), running("w", NOW - MIN)])], NOW - MIN);
    let mut settled = failed("s", NOW);
    settled["settledAt"] = json!(iso(NOW));
    let mut snoozed = done("z", NOW);
    snoozed["hasActionableProposedPlan"] = json!(true);
    snoozed["snoozedAt"] = json!(iso(NOW - MIN));
    snoozed["snoozedUntil"] = json!(iso(NOW + 5 * MIN));
    let mut child = approval("k", NOW);
    child["lineage"] = json!({"parentThreadId":"w","relationshipToParent":"subagent"});
    let next = || vec![env("laptop", vec![settled.clone(), snoozed.clone(), child.clone(), running("w", NOW)])];
    let v = view(next(), NOW, &b, json!({}));
    assert_eq!(v["alerts"], json!([]));
    assert_eq!(v["baseline"]["environments"]["laptop"], json!({}));
    assert!(!ids(&v["rows"], "threadId").contains(&"k".to_string()), "subagents have no row");
    // The snooze ends at a deadline (before the results' 15 minutes): the view
    // is due again then, and the ready plan alerts.
    assert_eq!(v["nextDeadline"], NOW + 5 * MIN);
    let woke = view(next(), NOW + 5 * MIN, &v["baseline"], json!({}));
    assert_eq!(alert_threads(&woke), ["z"]);
    // A request raises its hand through a snooze (T3 threadRaisedHandWhileSnoozed).
    let mut raised = approval("z", NOW);
    raised["snoozedAt"] = json!(iso(NOW - MIN));
    raised["snoozedUntil"] = json!(iso(NOW + 30 * MIN));
    assert_eq!(alert_threads(&view(vec![env("laptop", vec![raised])], NOW, &b, json!({}))), ["z"]);
}

#[test]
fn deadlines_are_absolute_and_never_extended() {
    let mut r = running("r", NOW - 30 * MIN);
    r["updatedAt"] = json!(iso(NOW - 10 * MIN));
    let envs = vec![env("laptop", vec![r, approval("a", NOW - 20 * MIN), done("d", NOW - 5 * MIN)])];
    let v = view(envs.clone(), NOW, &Value::Null, json!({}));
    let row = |v: &Value, id: &str| v["rows"].as_array().unwrap().iter().find(|r| r["route"]["threadId"] == id).cloned();
    assert_eq!(row(&v, "r").unwrap()["expiresAt"], NOW - 10 * MIN + RUNNING_TTL_MS);
    assert_eq!(row(&v, "r").unwrap()["since"], NOW - 30 * MIN, "the run's start, not updatedAt");
    assert_eq!(row(&v, "a").unwrap()["expiresAt"], NOW - 20 * MIN + WAITING_TTL_MS);
    assert_eq!(row(&v, "a").unwrap()["since"], NOW - 20 * MIN, "the request's creation");
    assert_eq!(row(&v, "d").unwrap()["expiresAt"], NOW - 5 * MIN + FINISHED_TTL_MS);
    assert_eq!(v["nextDeadline"], NOW + 10 * MIN, "the earliest: the result's 15 minutes");
    assert_eq!(v["card"]["expiresAt"], NOW - 20 * MIN + WAITING_TTL_MS, "the card lasts as long as its longest row");
    assert_eq!(v["card"]["since"], NOW - 20 * MIN);
    // Replaying the same shells later moves nothing; at the deadline the result is gone.
    let later = view(envs.clone(), NOW + 10 * MIN - 1, &v["baseline"], json!({}));
    assert_eq!(row(&later, "d").unwrap()["expiresAt"], row(&v, "d").unwrap()["expiresAt"]);
    let due = view(envs.clone(), NOW + 10 * MIN, &v["baseline"], json!({}));
    assert!(row(&due, "d").is_none());
    assert_eq!(due["nextDeadline"], NOW - 10 * MIN + RUNNING_TTL_MS);
    let gone = view(envs.clone(), NOW - 20 * MIN + WAITING_TTL_MS, &v["baseline"], json!({}));
    assert_eq!((gone["rows"].clone(), gone["card"].clone(), gone["nextDeadline"].clone()), (json!([]), Value::Null, Value::Null));
}

#[test]
fn a_revoked_environment_leaves_without_a_finished_alert() {
    let b = seed(vec![env("laptop", vec![running("a", NOW - MIN)]), env("phone", vec![approval("p", NOW - MIN)])], NOW - MIN);
    assert!(b["environments"]["phone"].get("p").is_some());
    let v = view(vec![env("laptop", vec![running("a", NOW)])], NOW, &b, json!({}));
    assert_eq!(v["alerts"], json!([]));
    assert_eq!(ids(&v["rows"], "environmentId"), ["laptop"]);
    assert!(v["baseline"]["environments"].get("phone").is_none(), "dropped with the environment");
    // Unauthorized but still listed: no alert either, and its entry is kept.
    let mut denied = env("phone", vec![done("p", NOW)]);
    denied["state"] = json!("unauthorized");
    let v = view(vec![env("laptop", vec![running("a", NOW)]), denied], NOW, &b, json!({}));
    assert_eq!(v["alerts"], json!([]));
    assert_eq!(v["baseline"]["environments"]["phone"], b["environments"]["phone"]);
}

#[test]
fn the_card_words_its_chip_title_and_rows_as_t3_mobile() {
    let card = |threads: Vec<Value>| view(vec![env("laptop", threads)], NOW, &Value::Null, json!({}))["card"].clone();
    let one = card(vec![running("a", NOW)]);
    assert_eq!((one["title"].as_str(), one["subtitle"].as_str(), one["status"].as_str()), (Some("Thread a"), Some("app · Host laptop"), Some("Working")));
    assert_eq!(one["chip"], json!({"kind":"working","count":1}));
    let many = card(vec![running("a", NOW), running("b", NOW - MIN), done("c", NOW)]);
    assert_eq!((many["title"].as_str(), many["subtitle"].as_str()), (Some("2 working"), Some("app · 2 active")));
    assert_eq!(many["chip"], json!({"kind":"live","count":2}));
    assert_eq!(ids(&many["rows"], "threadId"), ["a", "b", "c"]);
    let ask = card(vec![running("a", NOW), question("q", NOW), approval("p", NOW - MIN)]);
    assert_eq!((ask["phase"].as_str(), ask["title"].as_str()), (Some("waiting_for_input"), Some("2 need you")));
    assert_eq!(ask["chip"], json!({"kind":"answer","count":2}));
    // A failed result leads the rows, the chip still follows the live work.
    let mixed = card(vec![running("a", NOW), failed("f", NOW)]);
    assert_eq!((mixed["phase"].as_str(), mixed["title"].as_str(), mixed["chip"]["kind"].as_str()), (Some("failed"), Some("1 failed"), Some("working")));
    // At most five rows: live work first, then the newest results.
    let six = card(vec![running("a", NOW), running("b", NOW), running("c", NOW), running("d", NOW), done("e", NOW - MIN), failed("f", NOW - 2 * MIN)]);
    assert_eq!(ids(&six["rows"], "threadId"), ["a", "b", "c", "d", "e"]);
    let finished = card(vec![done("a", NOW), done("b", NOW - MIN)]);
    assert_eq!((finished["active"].as_bool(), finished["title"].as_str(), finished["subtitle"].as_str()), (Some(false), Some("All finished"), Some("app · 2 threads")));
    assert!(finished.get("chip").is_none());
    assert_eq!(card(vec![]), Value::Null);
    // Every key NotificationContract.kt reads.
    for key in ["active", "phase", "title", "subtitle", "status", "chip", "since", "expiresAt", "route", "rows"] {
        assert!(ask.get(key).is_some(), "{key}");
    }
    let es = view(vec![env("laptop", vec![running("a", NOW), question("q", NOW), approval("p", NOW)])], NOW, &Value::Null, json!({"locale":"es"}))["card"].clone();
    assert_eq!((es["title"].as_str(), es["subtitle"].as_str(), es["status"].as_str()), (Some("2 te necesitan"), Some("app · 3 activos"), Some("Aprobación")));
}

#[test]
fn arguments_are_checked() {
    let envs = json!([env("laptop", vec![])]);
    let err = |extra: Value| {
        let mut args = json!({"environments":envs,"now":NOW});
        for (k, v) in extra.as_object().unwrap() {
            args[k] = v.clone();
        }
        call("awareness.view", &args).unwrap_err()
    };
    assert_eq!(err(json!({"onScreen":{"environmentId":"laptop"}})), "invalid_onScreen");
    assert_eq!(err(json!({"preferences":{"approval":"yes"}})), "invalid_preferences");
    assert_eq!(err(json!({"preferences":{"sound":true}})), "invalid_preferences");
    assert_eq!(err(json!({"lifecycle":"warm"})), "invalid_lifecycle");
    assert_eq!(err(json!({"baseline":[]})), "invalid_baseline");
    assert_eq!(call("awareness.view", &json!({"environments":envs})).unwrap_err(), "invalid_now");
    assert_eq!(call("awareness.view", &json!({"environments":[{"id":"a"},{"id":"a"}],"now":NOW})).unwrap_err(), "invalid_environments");
}
