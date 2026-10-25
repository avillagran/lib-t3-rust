//! Review screens: the diff, the tool inspector, plans and checkpoint rewind.
use super::{domain_fn, DomainFn, NULL};
use crate::diff_review;
use serde_json::Value;

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
];
