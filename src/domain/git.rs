//! Git and pull requests: the row badge and the pull request page, the branch
//! picker and the Git & changes sheet.
use super::{domain_fn, DomainFn};
use crate::pull_requests;
use serde_json::{json, Value};

const PR_READS: &[&str] = &["list", "detail", "checks", "activity", "diff"];
const PR_WRITES: &[&str] = &["comment", "review", "action", "reviewer-candidates", "label-candidates", "reviewers", "labels"];
const PR_LINKS: &[&str] = &["link", "unlink", "watch", "unwatch"];

/// A `(method, payload)` pair as `{method, payload}`.
fn method((method, payload): (impl Into<String>, Value)) -> Value {
    json!({"method":method.into(),"payload":payload})
}

pub const FUNCTIONS: &[DomainFn] = &[
    domain_fn! {
        name: "pullRequests.sidebarBadge", module: "pullRequests",
        args: [(thread, Object, required)],
        returns: "object {kind: single, multiple or stack, text, state: open, draft, merged, closed or unknown, current, links} of a thread's linked pull requests, or null",
        run: |a| Ok(pull_requests::sidebar_badge(a.req_object("thread")?)),
    },
    domain_fn! {
        name: "pullRequests.request", module: "pullRequests",
        args: [
            (action, Enum(PR_READS), required), (state, Text, optional), (involvement, Text, optional), (query, Text, optional),
            (cursors, Object, optional), (reference, Object, optional), (cursor, Text, optional),
        ],
        returns: "object {method, payload}: the typed pullRequests.list, detail, checks or activity call and its args (list: 50 per page, state open and involvement all by default), or method diff with the /api/pull-requests/diff query; reference is {projectId, repository, host, number}",
        run: |a| Ok(method(pull_requests::request(a.all())?)),
    },
    domain_fn! {
        name: "pullRequests.payload", module: "pullRequests",
        args: [
            (kind, Enum(PR_WRITES), required), (reference, Object, required), (body, Text, optional), (verdict, Text, optional),
            (action, Text, optional), (mergeMethod, Text, optional), (reviewers, Array, optional), (requested, Bool, optional),
            (labels, Array, optional), (applied, Bool, optional),
        ],
        returns: "object {method, payload}: the typed pullRequests.comment, submitReview, runAction, reviewerCandidates, labelCandidates, requestReviewers or setLabels call and its args, after the page's rules (a comment needs words; only the page's actions)",
        run: |a| Ok(method(pull_requests::payload(a.all())?)),
    },
    domain_fn! {
        name: "pullRequests.threadCommand", module: "pullRequests",
        args: [(thread, Id, required), (link, Enum(PR_LINKS), required), (reference, Object, required), (commandId, Id, optional)],
        returns: "command thread.pull-request.link, unlink or watch for a reference {host, repository, number, url}; commandId only when given",
        run: |a| {
            let mut command = pull_requests::thread_command(a.all())?;
            match a.str("commandId")? {
                Some(id) => command["commandId"] = json!(id),
                None => {
                    command.as_object_mut().unwrap().remove("commandId");
                }
            }
            Ok(command)
        },
    },
];
