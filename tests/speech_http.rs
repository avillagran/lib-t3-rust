use serde_json::json;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};
use t3_protocol::{
    speech::{decode_pcm_request, CloudProvider, SpeechFailure, TranscriptionOptions},
    Connection, Credential,
};
fn request(s: &mut TcpStream) -> (String, Vec<u8>) {
    s.set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .unwrap();
    let mut head = vec![];
    let mut one = [0];
    while !head.ends_with(b"\r\n\r\n") {
        s.read_exact(&mut one).unwrap();
        head.push(one[0]);
    }
    let head = String::from_utf8(head).unwrap();
    let n = head
        .lines()
        .find_map(|l| {
            l.split_once(':')
                .filter(|(k, _)| k.eq_ignore_ascii_case("content-length"))
                .map(|(_, v)| v.trim().parse::<usize>().unwrap())
        })
        .unwrap_or(0);
    let mut body = vec![0; n];
    s.read_exact(&mut body).unwrap();
    (head, body)
}
type Requests = Vec<(String, Vec<u8>)>;
fn serve(answers: Vec<(&'static str, &'static str)>) -> (Connection, thread::JoinHandle<Requests>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let handle = thread::spawn(move || {
        let mut seen = vec![];
        for (status, body) in answers {
            let (mut s, _) = listener.accept().unwrap();
            seen.push(request(&mut s));
            write!(s,"HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
        }
        listener.set_nonblocking(true).unwrap();
        assert!(listener.accept().is_err());
        seen
    });
    (
        Connection::new(
            &base,
            None,
            Credential::bearer("private-synthetic-token").unwrap(),
        )
        .unwrap(),
        handle,
    )
}
#[test]
fn actual_authenticated_cloud_and_pcm_endpoints() {
    assert!(std::env::var_os("ANDROID_SANDBOX").is_some());
    let (c, h) = serve(vec![
        ("200 OK", r#"{"openai":true,"xai":false}"#),
        (
            "200 OK",
            r#"{"models":[{"id":"author/model","name":"Synthetic"}]}"#,
        ),
        ("204 No Content", ""),
        ("200 OK", r#"{"text":"synthetic transcript"}"#),
    ]);
    assert_eq!(c.speech_cloud_keys().unwrap().openrouter, None);
    assert_eq!(c.speech_cloud_models().unwrap()[0].id, "author/model");
    c.speech_set_cloud_key(
        CloudProvider::Xai,
        &json!({"provider":"1Password","reference":"op://private/example"}),
    )
    .unwrap();
    assert_eq!(
        c.speech_transcribe(
            &0.25f32.to_le_bytes(),
            &TranscriptionOptions::default(),
            Some((CloudProvider::OpenRouter, Some("author/model")))
        )
        .unwrap(),
        "synthetic transcript"
    );
    let seen = h.join().unwrap();
    for (head, _) in &seen {
        assert!(head
            .to_lowercase()
            .contains("authorization: bearer private-synthetic-token"));
    }
    assert!(seen[0].0.starts_with("GET /api/voice/cloud/keys "));
    assert!(seen[2].0.starts_with("POST /api/voice/cloud/xai/key "));
    assert!(seen[3]
        .0
        .starts_with("POST /api/voice/cloud/openrouter/transcribe?model=author%2Fmodel "));
    assert!(seen[3]
        .0
        .to_lowercase()
        .contains("content-type: application/octet-stream"));
    let (_, pcm) = decode_pcm_request(&seen[3].1).unwrap();
    assert_eq!(pcm, 0.25f32.to_le_bytes());
}
#[test]
fn redirects_auth_failures_and_invalid_input_do_not_retry() {
    assert!(std::env::var_os("ANDROID_SANDBOX").is_some());
    for (status, expected) in [
        ("302 Found", "http_302"),
        ("401 Unauthorized", "unauthorized"),
        ("403 Forbidden", "unauthorized"),
    ] {
        let (c, h) = serve(vec![(status, "{}")]);
        assert_eq!(
            c.speech_cloud_keys().unwrap_err(),
            SpeechFailure::Transport(expected.into())
        );
        assert_eq!(h.join().unwrap().len(), 1);
    }
    let c = Connection::new(
        "http://127.0.0.1:1",
        None,
        Credential::bearer("private-synthetic-token").unwrap(),
    )
    .unwrap();
    assert_eq!(
        c.speech_transcribe(&[], &TranscriptionOptions::default(), None)
            .unwrap_err(),
        SpeechFailure::Transport("invalid_speech_audio".into())
    );
    assert_eq!(
        c.speech_set_cloud_key(CloudProvider::OpenAi, &json!("https://no"))
            .unwrap_err(),
        "invalid_speech_key"
    );
}
#[test]
fn credentialed_redirect_cannot_reach_a_different_origin() {
    assert!(std::env::var_os("ANDROID_SANDBOX").is_some());
    let destination = TcpListener::bind("127.0.0.1:0").unwrap();
    let location = format!("http://{}/collect", destination.local_addr().unwrap());
    let source = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", source.local_addr().unwrap());
    let h = thread::spawn(move || {
        let (mut s, _) = source.accept().unwrap();
        let seen = request(&mut s);
        write!(s,"HTTP/1.1 307 Temporary Redirect\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
        seen
    });
    let c = Connection::new(
        &base,
        None,
        Credential::bearer("private-synthetic-token").unwrap(),
    )
    .unwrap();
    assert_eq!(
        c.speech_transcribe(&[0; 4], &TranscriptionOptions::default(), None)
            .unwrap_err(),
        SpeechFailure::Transport("http_307".into())
    );
    assert!(h
        .join()
        .unwrap()
        .0
        .to_lowercase()
        .contains("authorization: bearer private-synthetic-token"));
    destination.set_nonblocking(true).unwrap();
    assert!(destination.accept().is_err());
}
#[test]
fn cloud_error_is_typed_and_response_is_bounded() {
    assert!(std::env::var_os("ANDROID_SANDBOX").is_some());
    let (c, h) = serve(vec![(
        "422 Unprocessable Entity",
        r#"{"_tag":"CloudSpeechError","provider":"openai","reason":"unreadable-key"}"#,
    )]);
    assert_eq!(
        c.speech_transcribe(
            &[0; 4],
            &TranscriptionOptions::default(),
            Some((CloudProvider::OpenAi, None))
        )
        .unwrap_err(),
        SpeechFailure::Cloud {
            provider: CloudProvider::OpenAi,
            reason: "unreadable-key".into()
        }
    );
    h.join().unwrap();
    let source = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", source.local_addr().unwrap());
    let h = thread::spawn(move || {
        let (mut s, _) = source.accept().unwrap();
        request(&mut s);
        write!(s,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 8388609\r\nConnection: close\r\n\r\n").unwrap();
        let _ = s.write_all(&vec![b' '; 8388609]);
    });
    let c = Connection::new(
        &base,
        None,
        Credential::bearer("private-synthetic-token").unwrap(),
    )
    .unwrap();
    assert_eq!(
        c.speech_cloud_keys().unwrap_err(),
        SpeechFailure::Transport("too_large".into())
    );
    h.join().unwrap();
}
