//! Q7: replay determinism, on every OS CI runs (ubuntu, windows, macos).
//!
//! For every committed session log (each folder of `fixtures/recordings`,
//! plus the `voxxad_events` test log) the `voxxad` binary's `--replay`
//! must
//! 1. reproduce the log's own action lines byte for byte: today's conductor
//!    on the recorded recogniser lines does what the recorded run did, and
//! 2. give the same stdout, byte for byte, when run twice.
//!
//! (1) fails when a log holds actions from an older conductor. Then the log
//! is re-recorded (`voxxa-eval --full --record fixtures/recordings/<name>`)
//! and the commit says why; its recogniser lines stay the same.

mod common;

use common::committed_logs as logs;
use std::path::Path;
use std::process::Command;

/// The binary is built only with `--features cli`. Cargo sets `CARGO_BIN_EXE_voxxad`
/// even when it skips the binary, and the path may hold a stale build,
/// so the feature is checked: a plain `cargo test` fails here, loudly.
fn voxxad_exe() -> &'static str {
    if !cfg!(feature = "cli") {
        panic!("run: cargo test --features cli");
    }
    env!("CARGO_BIN_EXE_voxxad")
}

fn replay(log: &Path, setlist: &Path) -> Vec<u8> {
    let out = Command::new(voxxad_exe())
        .arg("--replay")
        .arg(log)
        .arg("--setlist")
        .arg(setlist)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", log.display());
    out.stdout
}

fn action_lines(text: &str) -> Vec<&str> {
    (text.lines())
        .filter(|l| l.contains(r#""type":"action""#))
        .collect()
}

#[test]
fn every_committed_log_replays_to_its_own_actions_and_twice_the_same() {
    let logs = logs();
    // The six Q6 fixtures and the test log; a new fixture is checked too.
    assert!(logs.len() >= 7, "{logs:?}");
    // Two replays of every log, all at once.
    let runs: Vec<_> = (logs.iter())
        .map(|(_, log, setlist)| {
            let (log, setlist) = (log.clone(), setlist.clone());
            let first = {
                let (log, setlist) = (log.clone(), setlist.clone());
                std::thread::spawn(move || replay(&log, &setlist))
            };
            let second = std::thread::spawn(move || replay(&log, &setlist));
            (first, second)
        })
        .collect();
    let mut failures = Vec::new();
    for ((name, log, _), (first, second)) in logs.iter().zip(runs) {
        let (first, second) = (first.join().unwrap(), second.join().unwrap());
        if first != second {
            failures.push(format!("{name}: two replays differ"));
        }
        let replayed = String::from_utf8(first).unwrap();
        let recorded = std::fs::read_to_string(log).unwrap();
        let (again, live) = (action_lines(&replayed), action_lines(&recorded));
        if live.is_empty() {
            failures.push(format!("{name}: the log has no action line"));
        }
        if again != live {
            // Counts and the first differing index only: never the text.
            let at = (again.iter().zip(&live)).position(|(a, b)| a != b);
            failures.push(format!(
                "{name}: replay printed {} action lines, the log holds {}; first difference at action {:?}",
                again.len(),
                live.len(),
                at.unwrap_or(again.len().min(live.len()))
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn replay_prints_only_lines_computed_from_the_log() {
    // Nothing in replay's stdout may depend on the clock or the machine:
    // no start line (it names the log path and version) and no wall time.
    let (_, log, setlist) = &logs()[0];
    let out = String::from_utf8(replay(log, setlist)).unwrap();
    assert!(!out.is_empty());
    for line in out.lines() {
        assert!(!line.contains(r#""state":"start""#), "a start line");
        let value: serde_json::Value = serde_json::from_str(line).unwrap();
        assert_eq!(value["v"], 1);
    }
}
