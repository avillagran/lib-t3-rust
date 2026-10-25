//! Presentation ported from ot3's QML and JavaScript: the conversation minimap,
//! subagent cards, theme colours and the string catalog.
use super::{domain_fn, DomainFn};
use crate::timeline::minimap;
use serde_json::json;

pub const FUNCTIONS: &[DomainFn] = &[
    domain_fn! {
        name: "timeline.minimap", module: "timeline",
        args: [(rows, Array, required), (visibleRowIds, Array, optional), (atEnd, Bool, optional)],
        returns: "object {items [{id, rowIndex, userText, assistantText, userPreview, assistantPreview}], visible (two items or more), currentIndex, visibleIds}: one marker per user row of timeline.view's timeline, paired with the last assistant reply before the next user row; currentIndex and visibleIds follow the rows on screen",
        run: |a| Ok(minimap::view(a.req_array("rows")?, a.array("visibleRowIds")?.map(Vec::as_slice).unwrap_or(&[]), a.bool("atEnd")?.unwrap_or(false))),
    },
    domain_fn! {
        name: "timeline.minimapIndex", module: "timeline",
        args: [(y, Number, required), (height, Number, required), (count, Integer, required)],
        returns: "integer: the marker under y on a rail height tall with count markers, or -1",
        run: |a| Ok(json!(minimap::index_at(a.f64("y")?.unwrap_or(0.0), a.f64("height")?.unwrap_or(0.0), a.i64("count")?.unwrap_or(0)))),
    },
];
