use serde_json::{json, Value};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};
use t3_protocol::{
    speech::{Acceleration, ControlFailure, LocalModelAction, PostProcessRequest, UnloadTimeout},
    Connection, Credential,
};
fn isolated() {
    assert!(std::env::var_os("ANDROID_SANDBOX").is_some());
}
fn read_request(s: &mut TcpStream) -> (String, Value) {
    s.set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .unwrap();
    let mut bytes = Vec::new();
    let mut one = [0];
    while !bytes.ends_with(b"\r\n\r\n") {
        s.read_exact(&mut one).unwrap();
        bytes.push(one[0]);
        assert!(bytes.len() < 65536);
    }
    let headers = String::from_utf8(bytes).unwrap();
    let size = headers
        .lines()
        .filter_map(|l| l.split_once(':'))
        .find(|(k, _)| k.eq_ignore_ascii_case("content-length"))
        .map(|(_, v)| v.trim().parse::<usize>().unwrap())
        .unwrap_or(0);
    assert!(size <= 2 * 1024 * 1024);
    let mut body = vec![0; size];
    s.read_exact(&mut body).unwrap();
    (
        headers,
        if body.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&body).unwrap()
        },
    )
}
fn server(
    answers: Vec<(&'static str, String)>,
) -> (Connection, thread::JoinHandle<Vec<(String, Value)>>) {
    isolated();
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", l.local_addr().unwrap());
    let task = thread::spawn(move || {
        let mut requests = vec![];
        for (status, body) in answers {
            let (mut s, _) = l.accept().unwrap();
            requests.push(read_request(&mut s));
            write!(s,"HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
        }
        l.set_nonblocking(true).unwrap();
        assert!(l.accept().is_err(), "Mutations must never retry");
        requests
    });
    (
        Connection::new(
            &url,
            None,
            Credential::bearer("synthetic-local-voice-token").unwrap(),
        )
        .unwrap(),
        task,
    )
}
fn postprocess() -> PostProcessRequest {
    PostProcessRequest::decode(&json!({"options":{"speechCustomWords":[{"term":"SyntheticProject","aliases":[]}],"speechPostProcessingPrompt":{"mode":"custom","customInstructions":"Synthetic instructions"},"speechCorrectionWord":"sorry"},"transcript":"Synthetic transcript","draft":{"text":"Synthetic draft","selection":{"start":-2.5,"end":4}}})).unwrap()
}
#[test]
fn every_local_model_control_and_postprocess_uses_the_native_authenticated_http_route() {
    let mut answers = vec![
        (
            "200 OK",
            json!({"supported":false,"reason":"Synthetic unsupported platform"}).to_string(),
        ),
        ("200 OK", json!({"models":[]}).to_string()),
        ("200 OK", json!({"models":[]}).to_string()),
        ("204 No Content", String::new()),
    ];
    for _ in 0..6 {
        answers.push(("200 OK", json!({"synthetic":true}).to_string()));
    }
    answers.push((
        "200 OK",
        json!({"text":"Synthetic cleaned transcript"}).to_string(),
    ));
    let (c, task) = server(answers);
    assert_eq!(c.speech_status().unwrap()["supported"], false);
    c.speech_models(false).unwrap();
    c.speech_models(true).unwrap();
    c.speech_prepare_model().unwrap();
    for action in [
        LocalModelAction::Download,
        LocalModelAction::Select,
        LocalModelAction::CancelDownload,
        LocalModelAction::Remove,
    ] {
        c.speech_model_action(action, "local/model").unwrap();
    }
    c.speech_set_acceleration(&Acceleration::parse("gpu:synthetic-0").unwrap())
        .unwrap();
    c.speech_set_unload_timeout(UnloadTimeout::Minutes5)
        .unwrap();
    let request = postprocess();
    assert!(!format!("{request:?}").contains("Synthetic"));
    assert_eq!(
        c.speech_postprocess(&request).unwrap(),
        "Synthetic cleaned transcript"
    );
    let requests = task.join().unwrap();
    let expected = [
        "GET /api/voice/status ",
        "GET /api/voice/models ",
        "GET /api/voice/models?benchmarks=1 ",
        "POST /api/voice/models/prepare ",
        "POST /api/voice/models/download ",
        "POST /api/voice/models/select ",
        "POST /api/voice/models/cancel ",
        "POST /api/voice/models/remove ",
        "POST /api/voice/acceleration ",
        "POST /api/voice/model-unload-timeout ",
        "POST /api/voice/post-process ",
    ];
    for ((head, _), expected) in requests.iter().zip(expected) {
        assert!(head.starts_with(expected));
        assert!(head
            .to_lowercase()
            .contains("authorization: bearer synthetic-local-voice-token"));
    }
    assert_eq!(requests[3].1, Value::Null);
    for (_, body) in &requests[4..8] {
        assert_eq!(body, &json!({"modelId":"local/model"}));
    }
    assert_eq!(requests[8].1, json!({"acceleration":"gpu:synthetic-0"}));
    assert_eq!(requests[9].1, json!({"timeout":"min_5"}));
    assert_eq!(requests[10].1, *request.as_json());
}
#[test]
fn model_controls_preserve_native_busy_reason_and_refuse_replay_or_invalid_input() {
    let body=json!({"_tag":"EnvironmentRequestInvalidError","code":"invalid_request","reason":"speech_busy","traceId":" synthetic-trace "}).to_string();
    let (c, t) = server(vec![("400 Bad Request", body)]);
    assert_eq!(
        c.speech_model_action(LocalModelAction::Remove, "a")
            .unwrap_err(),
        ControlFailure::InvalidRequest {
            reason: "speech_busy".into(),
            trace_id: "synthetic-trace".into()
        }
    );
    assert_eq!(t.join().unwrap().len(), 1);
    for (status, code) in [
        ("401 Unauthorized", "unauthorized"),
        ("403 Forbidden", "unauthorized"),
        ("503 Service Unavailable", "http_503"),
    ] {
        let (c, t) = server(vec![(status, "{}".into())]);
        assert_eq!(
            c.speech_prepare_model().unwrap_err(),
            ControlFailure::Transport(code.into())
        );
        assert_eq!(t.join().unwrap().len(), 1);
    }
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let c = Connection::new(
        &format!("http://{}", listener.local_addr().unwrap()),
        None,
        Credential::bearer("synthetic").unwrap(),
    )
    .unwrap();
    assert_eq!(
        c.speech_model_action(LocalModelAction::Download, "")
            .unwrap_err(),
        ControlFailure::Transport("invalid_speech_model".into())
    );
    assert!(listener.accept().is_err());
}
#[test]
fn transcript_and_model_controls_reject_redirects_and_bound_native_error_responses() {
    isolated();
    let target = TcpListener::bind("127.0.0.1:0").unwrap();
    target.set_nonblocking(true).unwrap();
    let location = format!(
        "http://{}/must-not-receive-transcript",
        target.local_addr().unwrap()
    );
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", l.local_addr().unwrap());
    let task = thread::spawn(move || {
        let (mut s, _) = l.accept().unwrap();
        let request = read_request(&mut s);
        write!(s,"HTTP/1.1 307 Temporary Redirect\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
        request
    });
    let c = Connection::new(&base, None, Credential::bearer("synthetic").unwrap()).unwrap();
    assert_eq!(
        c.speech_postprocess(&postprocess()).unwrap_err(),
        ControlFailure::Transport("http_307".into())
    );
    task.join().unwrap();
    assert!(target.accept().is_err());
    let (c, t) = server(vec![("400 Bad Request", "x".repeat(65537))]);
    assert_eq!(
        c.speech_status().unwrap_err(),
        ControlFailure::Transport("too_large".into())
    );
    t.join().unwrap();
    let(c,t)=server(vec![("400 Bad Request",json!({"_tag":"EnvironmentRequestInvalidError","code":"invalid_request","reason":"future-reason","traceId":"synthetic"}).to_string())]);
    assert_eq!(
        c.speech_status().unwrap_err(),
        ControlFailure::Transport("http_400".into())
    );
    t.join().unwrap();
}
