//! Real WebSocket and exported ABI boundaries, not replacement-character tests.
use serde_json::{json, value::RawValue};
use std::{collections::BTreeMap, ffi::CStr, net::TcpListener, sync::mpsc, thread, time::Duration};
use t3_protocol::{api::orchestration, wtf16::Wtf16String, Connection, Credential, RawRpcExit};
use tungstenite::Message;
struct SyntheticWtf16Upgrade;
impl tungstenite::handshake::server::Callback for SyntheticWtf16Upgrade {
    fn on_request(
        self,
        req: &tungstenite::handshake::server::Request,
        response: tungstenite::handshake::server::Response,
    ) -> std::result::Result<
        tungstenite::handshake::server::Response,
        tungstenite::handshake::server::ErrorResponse,
    > {
        assert_eq!(req.headers()["authorization"], "Bearer synthetic-wtf16");
        Ok(response)
    }
}
fn object(raw: &str) -> BTreeMap<String, Box<RawValue>> {
    serde_json::from_str(raw).unwrap()
}
fn raw_string(text: &str) -> Wtf16String {
    Wtf16String::from_json(&RawValue::from_string(text.to_owned()).unwrap()).unwrap()
}
fn connection(base: &str) -> Connection {
    Connection::new(base, None, Credential::bearer("synthetic-wtf16").unwrap()).unwrap()
}
#[test]
fn isolated_units_and_all_json_escapes_round_trip_without_repair() {
    for (input, units) in [
        (r#""\ud800""#, vec![0xd800]),
        (r#""\udc00""#, vec![0xdc00]),
        (r#""\udc00\ud800""#, vec![0xdc00, 0xd800]),
        (r#""\ud83d\ude00""#, vec![0xd83d, 0xde00]),
        ("\"😀\"", vec![0xd83d, 0xde00]),
        (
            r#""\u0000\b\f\n\r\t\\\"\/""#,
            vec![0, 8, 12, 10, 13, 9, 92, 34, 47],
        ),
    ] {
        let value = raw_string(input);
        assert_eq!(value.units(), units);
        assert_eq!(Wtf16String::from_json(&value.to_json()).unwrap(), value);
    }
    assert!(raw_string(r#""\ud800""#).to_utf8().is_err());
    assert_eq!(raw_string(r#""\ud83d\ude00""#).to_utf8().unwrap(), "😀");
    for input in [
        r#""\ud80""#,
        r#""\ux800""#,
        r#""\q""#,
        r#""\ud800"junk"#,
        r#"{"query":"ab",}"#,
    ] {
        assert!(RawValue::from_string(input.to_owned()).is_err(), "{input}");
    }
    assert!(serde_json::from_slice::<Box<RawValue>>(b"\"\xff\"").is_err());
    assert!(Wtf16String::from_json(&RawValue::from_string("null".into()).unwrap()).is_err());
}
#[test]
fn native_unit_limits_and_ecmascript_whitespace_are_not_rust_scalar_rules() {
    let options = json!({"limit":1,"titlesOnly":true});
    let clipped = Wtf16String::from_units([vec![b'a' as u16; 199], vec![0xd83d]].concat());
    let payload = orchestration::search_threads_utf16_payload(&clipped, &options).unwrap();
    assert_eq!(
        Wtf16String::from_json(&object(payload.get())["query"]).unwrap(),
        clipped
    );
    assert!(orchestration::search_threads_utf16_payload(
        &Wtf16String::from("😀".repeat(101).as_str()),
        &options
    )
    .is_err());
    assert!(orchestration::search_threads_utf16_payload(
        &Wtf16String::from("😀".repeat(100).as_str()),
        &options
    )
    .is_ok());
    assert_eq!(
        Wtf16String::from("\u{feff}ab\u{feff}")
            .trim_ecmascript()
            .to_utf8()
            .unwrap(),
        "ab"
    );
    assert_eq!(
        Wtf16String::from("\u{0085}a")
            .trim_ecmascript()
            .to_utf8()
            .unwrap(),
        "\u{0085}a"
    );
    assert!(orchestration::SEARCH_THREADS
        .payload(&json!({"query":"😀".repeat(101)}))
        .is_err());
    assert!(orchestration::SEARCH_THREADS
        .payload(&json!({"query":"\u{0085}a"}))
        .is_ok());
    assert_eq!(
        orchestration::SEARCH_THREADS
            .payload(&json!({"query":"\u{feff}ab\u{feff}"}))
            .unwrap()["query"],
        "ab"
    );
    assert!(
        orchestration::search_threads_utf16_payload(&clipped, &json!({"query":"other"})).is_err()
    );
}
#[test]
fn actual_websocket_preserves_clipped_high_low_and_failure_cause() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let cases = [
        vec![0x61, 0xd800],
        vec![0x61, 0xdc00],
        vec![0xdc00, 0xd800],
        vec![0xd83d, 0xde00],
        [vec![0x61; 199], vec![0xd83d]].concat(),
    ];
    let expected = cases.clone();
    let (sent, received) = mpsc::channel();
    let server = thread::spawn(move || {
        for (index, units) in expected.iter().enumerate() {
            let (socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut ws = tungstenite::accept_hdr(socket, SyntheticWtf16Upgrade).unwrap();
            let frame = ws.read().unwrap();
            let frame = object(frame.to_text().unwrap());
            assert_eq!(frame["tag"].get(), "\"orchestration.searchThreads\"");
            let payload = object(frame["payload"].get());
            let query = Wtf16String::from_json(&payload["query"]).unwrap();
            assert_eq!(query.units(), units);
            ws.send(Message::Ping(vec![1].into())).unwrap();
            assert!(matches!(ws.read().unwrap(), Message::Pong(_)));
            ws.send(Message::Text(r#"{"_tag":"Ping"}"#.into())).unwrap();
            assert_eq!(
                object(ws.read().unwrap().to_text().unwrap())["_tag"].get(),
                "\"Pong\""
            );
            ws.send(Message::Text(
                r#"{"_tag":"Exit","requestId":"other","exit":{"_tag":"Success","value":"\udc00"}}"#
                    .into(),
            ))
            .unwrap();
            let reply = if index == expected.len() - 1 {
                format!(
                    r#"{{"_tag":"Exit","requestId":"1","exit":{{"_tag":"Failure","cause":{{"reason":{}}}}}}}"#,
                    query.to_json().get()
                )
            } else {
                format!(
                    r#"{{"_tag":"Exit","requestId":"1","exit":{{"_tag":"Success","value":{{"query":{}}}}}}}"#,
                    query.to_json().get()
                )
            };
            ws.send(Message::Text(reply.into())).unwrap();
            let _ = ws.read();
            sent.send(()).unwrap();
        }
    });
    for (index, units) in cases.iter().enumerate() {
        let result = orchestration::search_threads_utf16(
            &connection(&base),
            &Wtf16String::from_units(units.clone()),
            &json!({}),
        )
        .unwrap();
        let value = match result {
            RawRpcExit::Success(Some(value)) => {
                assert_ne!(index, cases.len() - 1);
                object(value.get()).remove("query").unwrap()
            }
            RawRpcExit::Success(None) => panic!("Search success needs a value"),
            RawRpcExit::Failure(cause) => {
                assert_eq!(index, cases.len() - 1);
                object(cause.get()).remove("reason").unwrap()
            }
        };
        assert_eq!(Wtf16String::from_json(&value).unwrap().units(), units);
        received.recv_timeout(Duration::from_secs(3)).unwrap();
    }
    server.join().unwrap();
}
#[test]
fn exported_raw_abi_preserves_json_string_and_refuses_malformed_before_socket() {
    unsafe {
        let handle = t3_protocol::ffi::t3_client_new();
        for input in [
            b"{".as_slice(),
            br#"{"op":"rpc","tag":"x","payload":"\uxxxx"}"#,
        ] {
            let out = t3_protocol::ffi::t3_client_call_raw(handle, input.as_ptr(), input.len());
            let text = CStr::from_ptr(out).to_str().unwrap();
            assert!(text.contains("malformed"));
            t3_protocol::ffi::t3_string_free(out);
        }
        let input = br#"{"op":"searchThreadsUtf16","query":"a\ud800","options":{}}"#;
        let out = t3_protocol::ffi::t3_client_call_raw(handle, input.as_ptr(), input.len());
        assert!(CStr::from_ptr(out)
            .to_str()
            .unwrap()
            .contains("not_connected"));
        t3_protocol::ffi::t3_string_free(out);
        t3_protocol::ffi::t3_client_free(handle);
    }
}
#[test]
fn malformed_server_json_is_an_outcome_error_without_replay() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        let (socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut ws = tungstenite::accept(socket).unwrap();
        let _ = ws.read().unwrap();
        ws.send(Message::Text("{".into())).unwrap();
    });
    let payload = RawValue::from_string(r#"{"query":"a\ud800"}"#.into()).unwrap();
    assert_eq!(
        connection(&base)
            .rpc_raw("orchestration.searchThreads", &payload)
            .unwrap_err(),
        "rpc_outcome_unknown"
    );
    server.join().unwrap();
}
#[test]
fn native_void_success_and_explicit_null_remain_distinct() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        for suffix in ["", ",\"value\":null"] {
            let (socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut ws = tungstenite::accept(socket).unwrap();
            let _ = ws.read().unwrap();
            ws.send(Message::Text(format!("{{\"_tag\":\"Exit\",\"requestId\":\"1\",\"exit\":{{\"_tag\":\"Success\"{suffix}}}}}").into())).unwrap();
            let _ = ws.read();
        }
    });
    let payload = RawValue::from_string("{}".into()).unwrap();
    assert!(matches!(
        connection(&base).rpc_raw("fixture.void", &payload).unwrap(),
        RawRpcExit::Success(None)
    ));
    assert!(
        matches!(connection(&base).rpc_raw("fixture.null",&payload).unwrap(),RawRpcExit::Success(Some(value)) if value.get()=="null")
    );
    server.join().unwrap();
}
#[test]
fn typed_visit_strings_reach_the_actual_wire_without_client_date_parsing() {
    let cases = [
        "2026-11-08",
        "2026-11-08T12:34:56",
        "+275760-09-13T00:00:00Z",
        " 2026-11-08 ",
        "not a date",
        "",
    ];
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        for expected in cases {
            let (socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut ws = tungstenite::accept(socket).unwrap();
            let frame = ws.read().unwrap();
            let value: serde_json::Value = serde_json::from_str(frame.to_text().unwrap()).unwrap();
            assert_eq!(value["tag"], "orchestration.dispatchCommand");
            assert_eq!(value["payload"]["visitedAt"], expected);
            ws.send(Message::Text(r#"{"_tag":"Exit","requestId":"1","exit":{"_tag":"Success","value":{"observed":true}}}"#.into())).unwrap();
            let _ = ws.read();
        }
    });
    for (index, raw) in cases.iter().enumerate() {
        let command = orchestration::Command::ThreadVisit {
            command_id: format!("visit-{index}"),
            thread_id: "t1".into(),
            visited_at: (*raw).into(),
            opened: None,
            search_query: None,
        };
        let payload = command.payload().unwrap();
        assert_eq!(payload["visitedAt"], *raw);
        assert_eq!(
            connection(&base)
                .rpc("orchestration.dispatchCommand", payload)
                .unwrap()["observed"],
            true
        );
    }
    server.join().unwrap();
}
