//! Q7: replay is deterministic on every OS. For each fixture in
//! `fixtures/recordings`, `voxxad --replay` prints the action lines its
//! session log recorded, byte for byte, and a second replay prints exactly
//! the same stdout. A log whose actions came from an older conductor fails
//! here: re-record it (`voxxa-eval --full --record`), see fixtures/README.md.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The binary; it is built only with `--features cli`.
fn voxxad_exe() -> &'static str {
    option_env!("CARGO_BIN_EXE_voxxad").unwrap_or_else(|| panic!("run: cargo test --features cli"))
}

fn replay(dir: &Path) -> String {
    let out = Command::new(voxxad_exe())
        .arg("--replay")
        .arg(dir.join("session.jsonl"))
        .arg("--setlist")
        .arg(dir.join("setlist.json"))
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success(), "{}: {stdout}", dir.display());
    stdout
}

fn actions(jsonl: &str) -> Vec<&str> {
    (jsonl.lines())
        .filter(|l| l.contains(r#""type":"action""#))
        .collect()
}

#[test]
fn every_fixture_replays_its_logged_actions_byte_for_byte_and_twice_the_same() {
    let mut dirs: Vec<PathBuf> = std::fs::read_dir("../fixtures/recordings")
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    assert!(dirs.len() >= 6, "{dirs:?}");
    // Two replays per fixture, all at once: a debug replay takes up to ~40 s.
    let runs: Vec<_> = (dirs.iter().cloned())
        .flat_map(|d| [d.clone(), d])
        .map(|d| std::thread::spawn(move || replay(&d)))
        .collect();
    let outs: Vec<String> = runs.into_iter().map(|h| h.join().unwrap()).collect();
    for (dir, pair) in dirs.iter().zip(outs.chunks(2)) {
        let name = dir.file_name().unwrap().to_string_lossy();
        assert_eq!(pair[0], pair[1], "{name}: two replays differ");
        // str::lines drops the \r of a CRLF checkout, so this compares the JSON bytes.
        let log = std::fs::read_to_string(dir.join("session.jsonl")).unwrap();
        let (logged, replayed) = (actions(&log), actions(&pair[0]));
        assert!(!logged.is_empty(), "{name}: the log has no action line");
        assert_eq!(
            replayed, logged,
            "{name}: replay's actions differ from the log's; re-record it"
        );
    }
}
