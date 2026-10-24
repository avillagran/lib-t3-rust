use serde_json::json;
use t3_protocol::activity::describe;
#[test]
fn failed_tool_rows_are_marked_without_exporting_their_output() {
    let failed = describe(&json!({"id":"a","type":"command_execution","input":"cargo test","status":"failed","output":"PRIVATE failure"})).unwrap();
    assert_eq!((failed["failed"].as_bool(), failed["active"].as_bool()), (Some(true), Some(false)));
    assert!(!failed.to_string().contains("PRIVATE"));
    // A completed command whose output reports failure (non-zero exit) is failed too.
    let exit = describe(&json!({"type":"command_execution","input":"make","status":"completed","outputIndicatesFailure":true})).unwrap();
    assert_eq!(exit["failed"], true);
    for status in ["completed", "running"] {
        let row = describe(&json!({"item":{"type":"command_execution","input":"ls","status":status,"outputIndicatesFailure":false}})).unwrap();
        assert_eq!(row["failed"], false, "{status}");
    }
}
