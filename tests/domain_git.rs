//! Git and pull requests through `domain`: the row badge, the pull request
//! page's reads and writes, the branch picker and the Git & changes sheet.
use serde_json::{json, Value};
use t3_protocol::domain::call;

fn keys(v: &Value) -> Vec<&str> {
    let mut k = v.as_object().unwrap().keys().map(String::as_str).collect::<Vec<_>>();
    k.sort();
    k
}
/// The typed API accepts `payload` as `method`'s args unchanged.
fn typed(r: &Value) {
    let method = r["method"].as_str().unwrap();
    let f = t3_protocol::api::function(method).unwrap_or_else(|| panic!("{method} is not a typed call"));
    assert_eq!(f.payload(&r["payload"]).unwrap(), r["payload"], "{method}");
}
const REFERENCE: &str = r#"{"projectId":"p","repository":"o/r","host":"github.com","number":7}"#;

#[test]
fn pull_request_badge_and_page_reads() {
    let link = |n: u64, state: &str| json!({"number":n,"url":format!("https://github.com/o/r/pull/{n}"),"repository":"o/r","snapshot":{"state":state}});
    let badge = call("pullRequests.sidebarBadge", &json!({"thread":{"projectId":"p","pullRequests":[link(7, "open")]}})).unwrap();
    assert_eq!(keys(&badge), ["current", "kind", "links", "state", "text"]);
    assert_eq!((badge["kind"].as_str(), badge["text"].as_str(), badge["state"].as_str(), badge["current"]["host"].as_str()), (Some("single"), Some("7"), Some("open"), Some("github.com")));
    let two = call("pullRequests.sidebarBadge", &json!({"thread":{"pullRequests":[link(7, "merged"), link(8, "merged")]}})).unwrap();
    assert_eq!((two["kind"].as_str(), two["text"].as_str(), two["state"].as_str()), (Some("multiple"), Some("+2"), Some("merged")));
    assert!(call("pullRequests.sidebarBadge", &json!({"thread":{}})).unwrap().is_null());
    assert_eq!(badge, t3_protocol::pull_requests::sidebar_badge(&json!({"projectId":"p","pullRequests":[link(7, "open")]})));

    let list = call("pullRequests.request", &json!({"action":"list","state":"merged","involvement":"authored","query":" rust "})).unwrap();
    assert_eq!(list["method"], "pullRequests.list");
    assert_eq!((list["payload"]["limit"].as_u64(), list["payload"]["query"].as_str(), list["payload"]["state"].as_str()), (Some(50), Some("rust"), Some("merged")));
    typed(&list);
    let reference: Value = serde_json::from_str(REFERENCE).unwrap();
    for (action, method) in [("detail", "pullRequests.detail"), ("checks", "pullRequests.checks"), ("activity", "pullRequests.activity")] {
        let r = call("pullRequests.request", &json!({"action":action,"reference":reference})).unwrap();
        assert_eq!(keys(&r), ["method", "payload"]);
        assert_eq!(r["method"], method);
        typed(&r);
    }
    let diff = call("pullRequests.request", &json!({"action":"diff","reference":reference,"cursor":"c2"})).unwrap();
    assert_eq!((diff["method"].as_str(), diff["payload"]["cursor"].as_str(), diff["payload"]["number"].as_u64()), (Some("diff"), Some("c2"), Some(7)));
    assert_eq!(call("pullRequests.request", &json!({"action":"detail","reference":{"projectId":"p","repository":"o/r","number":7}})).unwrap_err(), "invalid_pull_request");
    assert_eq!(call("pullRequests.request", &json!({"action":"merge"})).unwrap_err(), "invalid_action");
}

#[test]
fn pull_request_page_writes_and_thread_links() {
    let reference: Value = serde_json::from_str(REFERENCE).unwrap();
    for (args, method) in [
        (json!({"kind":"comment","body":"Looks good"}), "pullRequests.comment"),
        (json!({"kind":"review","verdict":"approve"}), "pullRequests.submitReview"),
        (json!({"kind":"review","verdict":"request-changes","body":"Please split"}), "pullRequests.submitReview"),
        (json!({"kind":"action","action":"merge","mergeMethod":"squash"}), "pullRequests.runAction"),
        (json!({"kind":"reviewer-candidates"}), "pullRequests.reviewerCandidates"),
        (json!({"kind":"label-candidates"}), "pullRequests.labelCandidates"),
        (json!({"kind":"reviewers","reviewers":[{"id":"octocat","kind":"user"}],"requested":true}), "pullRequests.requestReviewers"),
        (json!({"kind":"labels","labels":["bug"],"applied":true}), "pullRequests.setLabels"),
    ] {
        let mut args = args;
        args["reference"] = reference.clone();
        let r = call("pullRequests.payload", &args).unwrap();
        assert_eq!(r["method"], method, "{args}");
        typed(&r);
    }
    for (args, error) in [
        (json!({"kind":"comment","body":"  "}), "invalid_comment"),
        (json!({"kind":"review","verdict":"comment"}), "invalid_comment"),
        (json!({"kind":"review","verdict":"lgtm"}), "invalid_review"),
        (json!({"kind":"action","action":"revert"}), "invalid_command"),
    ] {
        let mut args = args;
        args["reference"] = reference.clone();
        assert_eq!(call("pullRequests.payload", &args).unwrap_err(), error, "{args}");
    }

    let pr = json!({"host":"github.com","repository":"o/r","number":5,"url":"https://github.com/o/r/pull/5"});
    let link = call("pullRequests.threadCommand", &json!({"thread":"t","link":"link","reference":pr})).unwrap();
    assert_eq!(link, json!({"type":"thread.pull-request.link","threadId":"t","host":"github.com","repository":"o/r","number":5,"url":"https://github.com/o/r/pull/5","source":"manual"}));
    let watch = call("pullRequests.threadCommand", &json!({"thread":"t","link":"watch","reference":pr,"commandId":"c-1"})).unwrap();
    assert_eq!((watch["type"].as_str(), watch["watching"].as_bool(), watch["commandId"].as_str()), (Some("thread.pull-request.watch"), Some(true), Some("c-1")));
    assert_eq!(call("pullRequests.threadCommand", &json!({"thread":"t","link":"unwatch","reference":pr})).unwrap()["watching"], false);
    assert_eq!(call("pullRequests.threadCommand", &json!({"thread":"t","link":"link","reference":{"host":"github.com","repository":"o/r","number":5,"url":"javascript:x"}})).unwrap_err(), "invalid_pull_request");
}
