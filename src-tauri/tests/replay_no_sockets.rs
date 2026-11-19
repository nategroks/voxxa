//! Q7 security: `voxxad --replay` opens no socket. Linux only: voxxad runs
//! with networking disabled (`unshare -rn`: a new network namespace, even
//! loopback down) under `strace -f -e trace=network`, and the trace must
//! hold no network syscall. The same harness must see the socket of a voxxad
//! run that does connect to 127.0.0.1 (a presenter), so a detector that sees
//! nothing fails too. Not run on Windows or macOS (no strace there); replay
//! has no OS-specific code. In CI (`CI` set) missing tools fail the test.
#![cfg(target_os = "linux")]

use std::process::Command;

/// The binary; it is built only with `--features cli`.
fn voxxad_exe() -> &'static str {
    option_env!("CARGO_BIN_EXE_voxxad").unwrap_or_else(|| panic!("run: cargo test --features cli"))
}

const SETLIST: &str = "tests/fixtures/voxxad/pd-hymns-setlist.json";
const SESSION: &str = "tests/fixtures/voxxad/session-pd-hymns.jsonl";
const WAV: &str = "tests/fixtures/voxxad/amazing-grace-v1.wav";

fn works(cmd: &str, args: &[&str]) -> bool {
    let out = Command::new(cmd).args(args).output();
    out.is_ok_and(|o| o.status.success())
}

/// voxxad's exit status and every network syscall it made, or `None` when
/// this machine cannot run the harness.
fn network_syscalls(args: &[&str]) -> Option<(bool, Vec<String>)> {
    if !works("strace", &["-V"]) || !works("unshare", &["-rn", "true"]) {
        let why = "needs strace and unprivileged `unshare -rn`";
        assert!(std::env::var_os("CI").is_none(), "{why}");
        eprintln!("skipped: {why}");
        return None;
    }
    let dir = tempfile::tempdir().unwrap();
    let trace = dir.path().join("trace");
    let out = Command::new("unshare")
        .args(["-rn", "strace", "-f", "-qq", "-e", "trace=network", "-o"])
        .arg(&trace)
        .arg(voxxad_exe())
        .args(args)
        .output()
        .unwrap();
    let text = std::fs::read_to_string(&trace).unwrap();
    // Syscall lines have "name(args) = result"; skip exit and signal notes.
    let calls = text.lines().filter(|l| l.contains('(')).map(String::from);
    Some((out.status.success(), calls.collect()))
}

#[test]
fn replay_opens_no_socket_with_networking_disabled() {
    // Positive control: a presenter connect to 127.0.0.1 is seen.
    let mut connect = vec!["--stdin-audio", WAV, "--setlist", SETLIST];
    connect.extend(["--presenter", "propresenter7_rest", "--host", "127.0.0.1"]);
    connect.extend(["--port", "9", "--model", "nonexistent"]);
    let Some((_, calls)) = network_syscalls(&connect) else {
        return;
    };
    assert!(calls.iter().any(|c| c.contains("socket(")), "{calls:?}");

    let (ok, calls) = network_syscalls(&["--replay", SESSION, "--setlist", SETLIST]).unwrap();
    assert!(ok, "the replay failed");
    assert_eq!(
        calls,
        Vec::<String>::new(),
        "voxxad --replay used the network"
    );
}
