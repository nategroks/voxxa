//! Shared by the test files that replay every committed session log
//! (`replay_determinism`, `replay_no_sockets`). One list, so a log added
//! for one of them cannot go uncovered by the other.

use std::path::PathBuf;

const RECORDINGS: &str = "../fixtures/recordings";

/// (name, session log, setlist) for every committed log: each folder of
/// `fixtures/recordings` in name order, then the `voxxad_events` test log.
pub fn committed_logs() -> Vec<(String, PathBuf, PathBuf)> {
    let mut logs: Vec<_> = (std::fs::read_dir(RECORDINGS).unwrap().flatten())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .map(|dir| {
            let name = dir.file_name().unwrap().to_string_lossy().into_owned();
            (name, dir.join("session.jsonl"), dir.join("setlist.json"))
        })
        .collect();
    logs.sort();
    logs.push((
        "session-pd-hymns".into(),
        "tests/fixtures/voxxad/session-pd-hymns.jsonl".into(),
        "tests/fixtures/voxxad/pd-hymns-setlist.json".into(),
    ));
    logs
}
