//! Opt-in proof against an immutable real server, never a user's profile.
use std::{
    fs,
    net::TcpListener,
    path::PathBuf,
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use t3_protocol::{Connection, Credential};
struct Server(Child);
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
#[test]
#[ignore = "requires an explicitly mounted immutable server inside android-sandbox"]
fn real_server_separates_generic_and_media_ticket_audiences() {
    assert_eq!(std::env::var("ANDROID_SANDBOX").as_deref(), Ok("1"));
    assert!(std::path::Path::new("/run/.containerenv").is_file());
    let executable = std::env::var_os("O3_MEDIA_TEST_SERVER").expect("explicit immutable server");
    let root: PathBuf = std::env::temp_dir().join(format!(
        "sdk-media-purpose-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    let issued = Command::new(&executable)
        .args(["auth", "session", "issue", "--base-dir"])
        .arg(&root)
        .args([
            "--ttl",
            "1h",
            "--subject",
            "private-media-purpose",
            "--token-only",
        ])
        .output()
        .unwrap();
    assert!(
        issued.status.success(),
        "server CLI failed: {}",
        String::from_utf8_lossy(&issued.stderr)
    );
    let token = String::from_utf8(issued.stdout).unwrap();
    let port = {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    };
    let port_text = port.to_string();
    let mut server = Server(
        Command::new(&executable)
            .args(["serve", "--base-dir"])
            .arg(&root)
            .args(["--host", "127.0.0.1", "--port", &port_text])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let base = format!("http://127.0.0.1:{port}");
    let connection =
        Connection::new(&base, None, Credential::bearer(token.trim()).unwrap()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(15);
    while connection
        .request_json("GET", "/api/auth/session", None, false)
        .is_err()
    {
        assert!(server.0.try_wait().unwrap().is_none(), "server exited");
        assert!(Instant::now() < deadline, "server startup timed out");
        thread::sleep(Duration::from_millis(50));
    }
    let http = reqwest::blocking::Client::new();
    for (route, expected) in [
        ("/api/auth/websocket-ticket", 403),
        ("/api/auth/websocket-ticket/rpc", 401),
        ("/api/auth/websocket-ticket/media", 403),
    ] {
        let ticket = connection.request_json("POST", route, None, false).unwrap();
        let response = http
            .get(format!(
                "{base}/api/device-hub/vendor/serve-emu/api/screenshot"
            ))
            .query(&[
                ("device", "synthetic"),
                ("hostId", "local"),
                ("wsTicket", ticket["ticket"].as_str().unwrap()),
            ])
            .send()
            .unwrap();
        assert_eq!(
            response.status().as_u16(),
            expected,
            "ticket audience route {route}"
        );
    }
    // Generic legacy tickets and media tickets reach disabled-device policy (403).
    // An explicitly RPC-bound ticket fails media admission (401).
    // No actual hub, device, helper build, account, or VM is started by this proof.
    connection.device_hub("/api/device-hub", "local").unwrap();
}
