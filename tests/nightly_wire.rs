use serde_json::json;
use t3_protocol::{api, bounded_snapshot, projection, wire};
#[test]
fn id_table_expands_only_contract_record_fields_and_is_consumed_once() {
 let mut v=json!({"ids":["thread","m",17,"item"],"projection":{"thread":{"id":0,"activeRunId":2,"config":{"id":123}},"messages":[{"id":1,"threadId":0,"input":{"id":42}}],"visibleTurnItems":[{"sourceThreadId":0,"sourceItemId":3,"item":{"id":3,"messageId":1,"relatedIds":[0,2],"input":{"id":0}}}]}});
 wire::expand_ids(&mut v).unwrap();
 assert_eq!(v["projection"]["thread"]["activeRunId"],17);
 assert_eq!(v["projection"]["visibleTurnItems"][0]["item"]["messageId"],"m");
 assert_eq!(v["projection"]["visibleTurnItems"][0]["item"]["relatedIds"],json!(["thread",17]));
 assert_eq!(v["projection"]["thread"]["config"]["id"],123);
 let once=v.clone();wire::expand_ids(&mut v).unwrap();assert_eq!(v,once);
 assert!(wire::expand_ids(&mut json!({"ids":["one"],"projection":{"thread":{"id":4}}})).is_err());
}
#[test]
fn bounded_restores_ids_then_bodies_then_local_items_without_touching_inherited_rows() {
 let v=json!({"snapshotSequence":3,"ids":["t","m","i","p","cp","s"],"turnItemsOmitLocalVisible":true,"projection":{"thread":{"id":0},"messages":[{"id":1,"text":"hello"}],"checkpoints":[{"id":4,"files":[{"path":"a"}]}],"subagents":[{"id":5,"prompt":"review","result":"ok"}],"visibleTurnItems":[{"sourceThreadId":0,"sourceItemId":2,"visibility":"local","item":{"id":2,"type":"assistant_message","messageId":1,"text":"","textOmitted":true}},{"sourceThreadId":3,"sourceItemId":2,"visibility":"inherited","item":{"id":2,"type":"assistant_message","messageId":1,"text":"parent"}}],"turnItems":[{"id":"cp-item","type":"checkpoint","checkpointId":4,"files":[],"filesOmitted":true},{"id":"s-item","type":"subagent","subagentId":5,"prompt":"","result":null,"promptOmitted":true,"resultOmitted":true}]}});
 let got=bounded_snapshot::restore(v).unwrap();
 assert_eq!(got["projection"]["turnItems"][0]["text"],"hello");
 assert_eq!(got["projection"]["turnItems"][1]["files"][0]["path"],"a");
 assert_eq!(got["projection"]["turnItems"][2]["result"],"ok");
 assert_eq!(got["projection"]["visibleTurnItems"][1]["item"]["text"],"parent");
 assert_eq!(bounded_snapshot::restore(got.clone()).unwrap(),got);
}
#[test]
fn live_dedupe_uses_newest_state_and_missing_bodies_force_resnapshot() {
 let mut p=json!({"thread":{"id":"t"},"messages":[{"id":"m","text":"streamed"}],"subagents":[{"id":"s","prompt":"review","result":"ok"}]});
 projection::try_apply(&mut p,&json!({"type":"turn-item.updated","payload":{"id":"i","type":"assistant_message","messageId":"m","text":"","textOmitted":true}})).unwrap();
 assert_eq!(p["turnItems"][0]["text"],"streamed");
 projection::try_apply(&mut p,&json!({"type":"subagent.updated","payload":{"id":"s","prompt":"","result":null,"promptOmitted":true,"resultOmitted":true}})).unwrap();
 assert_eq!(p["subagents"][0]["prompt"],"review");
 let before=p.clone();assert!(projection::try_apply(&mut p,&json!({"type":"turn-item.updated","payload":{"id":"bad","type":"user_message","messageId":"missing","textOmitted":true}})).is_err());assert_eq!(p,before);
}
#[test]
fn new_registry_reads_validate_before_io_and_internal_guest_commands_remain_private() {
 assert!(api::orchestration::GET_OLDER_SETTLED_THREADS.payload(&json!({"before":{"settledAt":"2026-10-26T00:00:00Z"},"limit":50})).is_ok());
 assert!(api::orchestration::GET_OLDER_SETTLED_THREADS.payload(&json!({"before":{"settledAt":"bad"},"limit":51})).is_err());
 assert!(api::orchestration::Command::from_json(&json!({"type":"thread.guest-share.redeem","commandId":"c","threadId":"t"})).is_err());
 assert!(api::orchestration::Command::from_json(&json!({"type":"thread.guest-message.dismiss","commandId":"c","threadId":"t","guestMessageId":"g"})).is_ok());
}
#[test]
fn automatic_compaction_is_one_quiet_row_but_keeps_errors_and_manual_compaction() {
 let p=json!({"thread":{"id":"t"},"messages":[{"id":"compact-prompt","runId":"auto","role":"user","text":"/compact","idleCompaction":"beforeCacheExpires","createdAt":1000},{"id":"compact-answer","runId":"auto","role":"assistant","text":"Done"},{"id":"human","runId":"manual","role":"user","text":"/compact"}],"runs":[{"id":"auto","status":"running"},{"id":"manual","status":"completed"}]});
 let mut rows=vec![json!({"id":"compact-prompt","role":"user"}),json!({"id":"compact-answer","role":"assistant"}),json!({"id":"work","role":"activity","activity":{"runId":"auto"}}),json!({"id":"human","role":"user"}),json!({"id":"err","role":"error"})];
 t3_protocol::compaction::timeline(&p,&mut rows);
 assert_eq!(rows.len(),3);
 assert!(rows.iter().any(|r|r["id"]=="human"));assert!(rows.iter().any(|r|r["id"]=="err"));
 let row=rows.iter().find(|r|r["id"]=="compact/auto").unwrap();assert_eq!(row["activity"]["category"],"compact");assert_eq!(row["activity"]["active"],true);
 assert_eq!(t3_protocol::timeline::active_run_id(&p),"");
}
#[path="support/fake_t3.rs"]
mod fake_t3;
#[test]
fn compact_snapshot_decodes_through_the_actual_websocket_transport() {
 use std::cell::{Cell,RefCell};
 let snapshot=json!({"kind":"snapshot","snapshotSequence":1,"ids":["t","m","i"],"turnItemsOmitLocalVisible":true,"projection":{"thread":{"id":0},"messages":[{"id":1,"text":"hello"}],"turnItems":[],"visibleTurnItems":[{"sourceThreadId":0,"sourceItemId":2,"visibility":"local","item":{"id":2,"type":"assistant_message","messageId":1,"text":"","textOmitted":true}}]}});
 let server=fake_t3::FakeT3::spawn(vec![fake_t3::Reply::Chunks(vec![vec![snapshot]])]);
 let c=server.connection();let done=Cell::new(false);let got=RefCell::new(None);
 let payload=json!({"threadId":"t","acceptBoundedSnapshot":true,"acceptCompactIds":true,"acceptDedupedBodies":true});
 let result=c.subscribe("orchestration.subscribeThread",payload,||done.get(),|v|{if let Some(snapshot)=v{*got.borrow_mut()=Some(bounded_snapshot::restore(snapshot)?);done.set(true);}Ok(())});
 assert!(result.is_ok()||result.as_ref().err().map(String::as_str)==Some("stream_closed"));
 let got=got.into_inner().unwrap();assert_eq!(got["projection"]["thread"]["id"],"t");assert_eq!(got["projection"]["turnItems"][0]["text"],"hello");
 assert_eq!(server.finish().connections,1);
}

#[test]
fn owner_approval_of_stored_guest_body_round_trips_with_empty_local_text() {
 let command=json!({"type":"message.dispatch","commandId":"c","threadId":"t","messageId":"m","text":"","attachments":[],"createdBy":"user","creationSource":"web","dispatchMode":{"type":"start_immediately"},"deliveryIntent":"auto","guestMessageId":"g"});
 assert_eq!(api::Command::from_json(&command).unwrap().payload().unwrap(),command);
 let mut ordinary=command;ordinary.as_object_mut().unwrap().remove("guestMessageId");assert!(api::Command::from_json(&ordinary).is_err());
}
