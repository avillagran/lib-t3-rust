use serde_json::{json};
#[test]
fn compact_sidebar_preserves_rows_without_duplicate_titles() {
 let args=json!({"now":0,"environments":[{"id":"e","shell":{"threads":[{"id":"t","title":"unique-long-title"}]}}],"compact":true});
 let result=t3_protocol::domain::call("sidebar.view", &args).unwrap();
 assert_eq!(result["format"], "indexed-v1");
 let encoded=result.to_string();
 assert_eq!(encoded.matches("unique-long-title").count(),1);
 assert!(result["sections"]["active"][0].is_array());
 assert!(result["threads"][0].is_object());
}
#[test]
fn compact_rows_decode_to_the_legacy_view() {
 let fixture: serde_json::Value=serde_json::from_str(include_str!("fixtures/sidebar-environments.json")).unwrap();
 for grouped in [false,true] {
 let mut args=json!({"environments":fixture["environments"],"now":fixture["nowMs"],"grouped":grouped});
 let full=t3_protocol::domain::call("sidebar.view", &args).unwrap();
 args["compact"]=json!(true);
 let mut compact=t3_protocol::domain::call("sidebar.view", &args).unwrap();
 let defaults=compact["rowDefaults"].as_object().unwrap().clone();
 for row in compact["threads"].as_array_mut().unwrap() {
  let absent=row["$absent"].as_array().cloned().unwrap_or_default();
  for (k,v) in &defaults { if row.get(k).is_none() && !absent.contains(&json!(k)) { row[k]=v.clone(); } }
  row.as_object_mut().unwrap().remove("$absent");
 }
 for name in ["active","working","snoozed","settled"] {
  let refs=compact["sections"][name].as_array().unwrap().clone();
  compact["sections"][name]=serde_json::Value::Array(refs.iter().map(|r| {
   let mut row=compact["threads"][r[0].as_u64().unwrap() as usize].clone();
   for (k,v) in r[1].as_object().unwrap() { row[k]=v.clone(); } row
  }).collect());
 }
 compact.as_object_mut().unwrap().remove("format"); compact.as_object_mut().unwrap().remove("rowDefaults");
 assert_eq!(full,compact);
 }
}
#[test]
fn compact_admission_bounds_aggregate_bytes_and_rows() {
 for threads in [vec![json!({"id":"t","title":"x".repeat(4*1024*1024)})], (0..1001).map(|n|json!({"id":format!("t{n}")})).collect()] {
  let args=json!({"now":0,"compact":true,"environments":[{"id":"e","shell":{"threads":threads}}]});
  assert!(matches!(t3_protocol::domain::call("sidebar.view",&args),Err(e) if e=="sidebar_presentation_limit"));
 }
}
