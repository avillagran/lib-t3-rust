//! Review screens: the diff, the tool inspector, plans and checkpoint rewind.
use super::{domain_fn, DomainFn, NULL};
use crate::api::Args;
use crate::{diff_review, tool_inspector, Result};
use serde_json::{json, Value};

/// A timeline row's `inspector` reference scoped to the viewer's environment and thread.
fn inspector_input(a: &Args) -> Result<Value> {
    let mut input = a.req_object("reference")?.clone();
    input["environment"] = json!(a.req("environment")?);
    input["threadId"] = json!(a.req("threadId")?);
    Ok(input)
}

pub const FUNCTIONS: &[DomainFn] = &[
    domain_fn! {
        name: "diffReview.present", module: "diffReview",
        args: [(result, Object, required), (summary, Object, optional)],
        returns: "object {files [{path, oldPath, kind, binary, missing?, additions, deletions, summaryAdditions?, summaryDeletions?, rows [{kind: hunk, addition, deletion, context or meta, text, oldLine, newLine}], splitRows [{kind, left, right}]}], additions, deletions, fromTurnCount, toTurnCount} from a diff RPC result {diff}",
        run: |a| diff_review::present(a.req_object("result")?, a.get("summary").unwrap_or(&NULL)),
    },
    domain_fn! {
        name: "diffReview.scopedResult", module: "diffReview",
        args: [(expected, Object, required), (receipt, Object, required), (summary, Object, required)],
        returns: "diffReview.present's object for a receipt of the still-open request, null for a late or other request; the receipt's error otherwise",
        run: |a| diff_review::scoped_result(a.req_object("expected")?, a.req_object("receipt")?, a.req_object("summary")?).unwrap_or(Ok(Value::Null)),
    },
    domain_fn! {
        name: "toolInspector.compactReference", module: "toolInspector",
        args: [(row, Object, required)],
        returns: "object {sourceThreadId, sourceItemId, type, runId, providerTurnId, revision} of a turn item or projected row (revision live while it runs); timeline activity entries already carry it as inspector",
        run: |a| tool_inspector::compact_reference(a.req_object("row")?),
    },
    domain_fn! {
        name: "toolInspector.scope", module: "toolInspector",
        args: [(reference, Object, required), (environment, Text, required), (threadId, Id, required)],
        returns: "object {environment, threadId, sourceThreadId, sourceItemId, type, runId, providerTurnId, revision}: the inspector's local scope, to match the read's answer against",
        run: |a| tool_inspector::scope(&inspector_input(a)?),
    },
    domain_fn! {
        name: "toolInspector.request", module: "toolInspector",
        args: [(reference, Object, required), (environment, Text, required), (threadId, Id, required)],
        returns: "object {method: orchestration.getTurnItem, payload {threadId, itemId, revision}}: the typed call that reads the item",
        run: |a| {
            let (method, payload) = tool_inspector::request(&inspector_input(a)?)?;
            Ok(json!({"method":method,"payload":payload}))
        },
    },
    domain_fn! {
        name: "toolInspector.parse", module: "toolInspector",
        args: [(reference, Object, required), (environment, Text, required), (threadId, Id, required), (response, Object, required)],
        returns: "object {scope, available, title, status, failed, exitCode, sections [{key, label, text, truncated}], metadata, metadataTruncated, raw, rawTruncated, outputOmitted, emptyOutput} from orchestration.getTurnItem's {item}; {scope, available: false, sections: [], raw: \"\", rawTruncated: false} when the item is gone",
        run: |a| tool_inspector::parse(&inspector_input(a)?, a.req_object("response")?),
    },
];
