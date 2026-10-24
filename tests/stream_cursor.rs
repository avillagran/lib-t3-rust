//! `stream::cursor_of`, `stream::advance` and `stream::start_of`: pure, so a host
//! never reimplements ot3's resume rule (live.rs, shell_watch.rs).
use serde_json::json;
use t3_protocol::stream::{advance, cursor_of, start_of, Start};

const THREAD: &str = "orchestration.subscribeThread";
const SHELL: &str = "orchestration.subscribeShell";

#[test]
fn cursor_of_reads_events_snapshots_and_unknown_events() {
    let event = json!({"kind":"event","sequence":42,"event":{"type":"message.updated","payload":{}}});
    assert_eq!(cursor_of(THREAD, &event), Some(42));
    let snapshot = json!({"kind":"snapshot","snapshotSequence":40,"projection":{},"hasMoreHistory":true});
    assert_eq!(cursor_of(THREAD, &snapshot), Some(40));
    // On the wire an event this build cannot decode is still `event` with its sequence…
    let unknown_on_wire = json!({"kind":"event","sequence":43,"event":{"type":"workflow.future-thing"}});
    assert_eq!(cursor_of(THREAD, &unknown_on_wire), Some(43));
    // …and a host that already decoded it as T3's client does keeps moving the cursor too.
    assert_eq!(cursor_of(THREAD, &json!({"kind":"unknown-event","sequence":44,"eventType":"x"})), Some(44));
    assert_eq!(cursor_of(THREAD, &json!({"kind":"some-later-kind","sequence":45})), Some(45));
    assert_eq!(cursor_of(THREAD, &json!({"kind":"synchronized"})), None);
    // Past 2^53, as the item arrived: no float in between.
    assert_eq!(cursor_of(THREAD, &json!({"kind":"event","sequence":9_007_199_254_740_993u64})), Some(9_007_199_254_740_993));

    // The shell nests its snapshot.
    assert_eq!(cursor_of(SHELL, &json!({"kind":"snapshot","snapshot":{"snapshotSequence":7,"threads":[]}})), Some(7));
    for kind in ["project.updated", "project.removed", "thread.updated", "thread.removed"] {
        assert_eq!(cursor_of(SHELL, &json!({"kind":kind,"sequence":8})), Some(8), "{kind}");
    }
    assert_eq!(cursor_of(SHELL, &json!({"kind":"synchronized"})), None);

    // Ordering only: terminal output and worktree setup states.
    let terminal = json!({"type":"snapshot","snapshot":{"sequence":5,"history":"$ "}});
    assert_eq!(cursor_of("terminal.attach", &terminal), Some(5));
    assert_eq!(cursor_of("terminal.attach", &json!({"type":"output","sequence":6,"data":"x"})), Some(6));
    assert_eq!(cursor_of("subscribeWorktreeSetup", &json!({"threadId":"t","sequence":3})), Some(3));
    assert_eq!(cursor_of("subscribeWorktreeSetup", &json!(null)), None);
    // Streams without a cursor, and tags the typed API does not know.
    assert_eq!(cursor_of("subscribeDeviceState", &json!({"sequence":9})), None);
    assert_eq!(cursor_of("subscribeSomethingNew", &json!({"sequence":9})), None);
}

#[test]
fn advance_lets_a_snapshot_set_the_cursor_and_events_only_move_it_on() {
    let mut cursor = Some(41);
    for (item, expected) in [
        (json!({"kind":"event","sequence":42}), Some(42)),
        (json!({"kind":"synchronized"}), Some(42)),
        // A replayed older event never moves the cursor back.
        (json!({"kind":"event","sequence":39}), Some(42)),
        // A snapshot replaces the state, so it sets the cursor, even lower (ot3's live.rs).
        (json!({"kind":"snapshot","snapshotSequence":30,"projection":{}}), Some(30)),
        (json!({"kind":"event","sequence":31,"event":{"type":"unknown.kind"}}), Some(31)),
    ] {
        cursor = advance(THREAD, cursor, &item);
        assert_eq!(cursor, expected, "{item}");
    }
    assert_eq!(advance(THREAD, None, &json!({"kind":"synchronized"})), None);
    assert_eq!(advance(SHELL, None, &json!({"kind":"snapshot","snapshot":{"snapshotSequence":5}})), Some(5));
    // A snapshot without a sequence keeps what the host had.
    assert_eq!(advance(SHELL, Some(9), &json!({"kind":"snapshot","snapshot":{}})), Some(9));
    assert_eq!(advance("subscribePreviewEvents", Some(3), &json!({"type":"opened"})), Some(3));
}

#[test]
fn start_of_tells_a_snapshot_from_a_resume() {
    assert_eq!(start_of(THREAD, &json!({"kind":"snapshot","snapshotSequence":1})), Some(Start::Snapshot));
    assert_eq!(start_of(THREAD, &json!({"kind":"event","sequence":2})), Some(Start::Resume));
    // Resumed with nothing new: the marker alone.
    assert_eq!(start_of(THREAD, &json!({"kind":"synchronized"})), Some(Start::Resume));
    assert_eq!(start_of(SHELL, &json!({"kind":"thread.updated","sequence":2})), Some(Start::Resume));
    assert_eq!(start_of("orchestration.subscribeCommandOutput", &json!({"kind":"replace","text":""})), Some(Start::Snapshot));
    assert_eq!(start_of("orchestration.subscribeCommandOutput", &json!({"kind":"append","text":""})), Some(Start::Resume));
    assert_eq!(start_of("terminal.attach", &json!({"type":"snapshot","snapshot":{}})), Some(Start::Snapshot));
    assert_eq!(start_of("subscribeServerConfig", &json!({"version":1,"type":"snapshot"})), Some(Start::Snapshot));
    for tag in ["subscribeDeviceState", "subscribeDiscoveredLocalServers", "subscribeWorktreeSetup", "server.getStorageCleanupReport", "provider.install.subscribe", "provider.auth.subscribe"] {
        assert_eq!(start_of(tag, &json!(null)), Some(Start::Snapshot), "{tag}");
    }
    assert_eq!(start_of("git.runStackedAction", &json!({"kind":"action_started"})), Some(Start::Events));
    assert_eq!(start_of("subscribePreviewEvents", &json!({"type":"opened"})), Some(Start::Events));
    assert_eq!(start_of("subscribeSomethingNew", &json!({})), None);
    assert_eq!([Start::Snapshot, Start::Resume, Start::Events].map(Start::as_str), ["snapshot", "resume", "events"]);
}
