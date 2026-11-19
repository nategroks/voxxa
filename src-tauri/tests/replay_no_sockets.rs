//! Q7 security: `voxxad --replay` opens no socket.
//!
//! Replay reads a session log and a setlist and prints actions; it builds no
//! presenter, loads no model and needs no network (ADR-013). This test runs
//! the real `voxxad` binary on every committed session log under an OS-level
//! socket tracer and asserts that the tracer saw no socket activity from it.
//!
//! - **Linux:** `strace -f -e trace=%network` records every network system
//!   call (`socket`, `connect`, `bind`, `sendto`, ...) of the process and its
//!   threads. When `unshare -rn` works, the run is also inside a new network
//!   namespace: networking is disabled, only a down loopback exists.
//! - **Windows:** an ETW session on the `Microsoft-Windows-Winsock-AFD`
//!   provider records every socket the kernel's AFD driver creates, connects
//!   or closes, with the process id. Starting it needs an administrator
//!   (GitHub's Windows runners are). Networking stays enabled.
//! - **macOS and anything else:** no tracer that works without disabling SIP,
//!   so the test prints why and passes. The replay code is the same Rust on
//!   every OS (`src/cli/voxxad/events.rs`, no OS-specific code).
//!
//! Each run first traces a positive control, this test binary connecting to
//! 127.0.0.1 (`socket_probe`), and fails if the tracer did not see it: a
//! tracer that sees nothing cannot pass the test. Set
//! `VOXXA_REQUIRE_SOCKET_TRACE=1` (CI does, on Linux and Windows) to fail
//! instead of skipping when no tracer is available.

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

/// The binary is built only with `--features cli`. Cargo sets `CARGO_BIN_EXE_voxxad`
/// even when it skips the binary, and the path may hold a stale build,
/// so the feature is checked: a plain `cargo test` fails here, loudly.
fn voxxad_exe() -> &'static str {
    assert!(cfg!(feature = "cli"), "run: cargo test --features cli");
    env!("CARGO_BIN_EXE_voxxad")
}

const RECORDINGS: &str = "../fixtures/recordings";
const PROBE_ENV: &str = "VOXXA_SOCKET_PROBE";

struct Run {
    name: String,
    program: PathBuf,
    args: Vec<OsString>,
    probe: bool,
}

struct Traced {
    success: bool,
    stdout: String,
    /// Network events the tracer recorded for this run's process.
    events: usize,
    /// Up to five of them, for the failure message (no transcript text).
    sample: Vec<String>,
}

/// The positive control: this test binary, running only `socket_probe`.
fn probe() -> Run {
    let args = [
        "socket_probe",
        "--exact",
        "--ignored",
        "--test-threads",
        "1",
    ];
    Run {
        name: "probe (connects to 127.0.0.1)".into(),
        program: std::env::current_exe().unwrap(),
        args: args.iter().map(OsString::from).collect(),
        probe: true,
    }
}

/// `voxxad --replay` on every committed log.
fn replays() -> Vec<Run> {
    let mut logs: Vec<(String, PathBuf, PathBuf)> = (std::fs::read_dir(RECORDINGS).unwrap())
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .map(|d| {
            let name = d.file_name().unwrap().to_string_lossy().into_owned();
            (name, d.join("session.jsonl"), d.join("setlist.json"))
        })
        .collect();
    logs.sort();
    logs.push((
        "session-pd-hymns".into(),
        "tests/fixtures/voxxad/session-pd-hymns.jsonl".into(),
        "tests/fixtures/voxxad/pd-hymns-setlist.json".into(),
    ));
    (logs.into_iter())
        .map(|(name, log, setlist)| Run {
            name,
            program: voxxad_exe().into(),
            args: vec![
                "--replay".into(),
                log.into(),
                "--setlist".into(),
                setlist.into(),
            ],
            probe: false,
        })
        .collect()
}

#[test]
#[ignore = "helper: the positive control of replay_opens_no_socket runs it"]
fn socket_probe() {
    if std::env::var_os(PROBE_ENV).is_none() {
        return;
    }
    // Nothing listens on port 9 (discard); creating the socket is enough.
    let addr = "127.0.0.1:9".parse().unwrap();
    let _ = std::net::TcpStream::connect_timeout(&addr, Duration::from_millis(500));
}

#[test]
fn replay_opens_no_socket() {
    let mut runs = vec![probe()];
    runs.extend(replays());
    assert!(runs.len() >= 8, "six fixtures and the test log");
    let Some((tracer, traced)) = trace(&runs) else {
        let why = "no socket tracer here (Linux: strace; Windows: an administrator for ETW)";
        if std::env::var("VOXXA_REQUIRE_SOCKET_TRACE").as_deref() == Ok("1") {
            panic!("VOXXA_REQUIRE_SOCKET_TRACE=1 but {why}");
        }
        eprintln!("replay_opens_no_socket: skipped: {why}");
        return;
    };
    eprintln!("replay_opens_no_socket: traced with {tracer}");
    let control = &traced[0];
    eprintln!(
        "replay_opens_no_socket: the probe made {} events",
        control.events
    );
    assert!(
        control.events > 0,
        "{tracer} saw no socket from the probe, so it cannot vouch for replay"
    );
    for (run, t) in runs.iter().zip(&traced).skip(1) {
        assert!(t.success, "{}: voxxad --replay failed", run.name);
        assert!(
            t.stdout.contains(r#""type":"action""#),
            "{}: replay printed no action",
            run.name
        );
        assert_eq!(
            t.events, 0,
            "{}: voxxad --replay touched the network under {tracer}: {:?}",
            run.name, t.sample
        );
    }
}

/// Runs every command under this OS's tracer; `None` when there is none.
fn trace(runs: &[Run]) -> Option<(String, Vec<Traced>)> {
    if cfg!(target_os = "linux") {
        strace(runs)
    } else if cfg!(windows) {
        etw(runs)
    } else {
        None
    }
}

fn works(program: &str, args: &[&str]) -> bool {
    (Command::new(program).args(args).output()).is_ok_and(|o| o.status.success())
}

fn command(run: &Run) -> Command {
    let mut cmd = Command::new(&run.program);
    cmd.args(&run.args);
    if run.probe {
        cmd.env(PROBE_ENV, "1");
    }
    cmd
}

fn strace(runs: &[Run]) -> Option<(String, Vec<Traced>)> {
    if !works("strace", &["-V"]) {
        return None;
    }
    let unshare = works("unshare", &["-rn", "true"]);
    let dir = tempfile::tempdir().unwrap();
    let traced = (runs.iter().enumerate())
        .map(|(i, run)| {
            let file = dir.path().join(format!("trace-{i}.txt"));
            let inner = command(run);
            let mut cmd = Command::new(if unshare { "unshare" } else { "strace" });
            if unshare {
                cmd.args(["-rn", "strace"]);
            }
            cmd.args([
                "-f",
                "-qq",
                "-e",
                "trace=%network",
                "-e",
                "signal=none",
                "-o",
            ]);
            cmd.arg(&file)
                .arg(inner.get_program())
                .args(inner.get_args());
            cmd.envs(inner.get_envs().filter_map(|(k, v)| Some((k, v?))));
            let out = cmd.output().unwrap();
            let log = std::fs::read_to_string(&file).unwrap_or_default();
            // Each line is "<pid> <syscall>(...) = ..."; strace's own notes
            // (+++ exit, --- signal) are not system calls.
            let calls: Vec<String> = (log.lines())
                .map(|l| l.trim_start_matches(|c: char| c.is_ascii_digit() || c == ' '))
                .filter(|l| l.contains('(') && !l.starts_with("+++") && !l.starts_with("---"))
                .map(|l| l.chars().take(120).collect())
                .collect();
            Traced {
                success: out.status.success(),
                stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
                events: calls.len(),
                sample: calls.into_iter().take(5).collect(),
            }
        })
        .collect();
    let how = match unshare {
        true => "strace inside unshare -rn (networking disabled)",
        false => "strace (unshare -rn unavailable: networking enabled)",
    };
    Some((how.into(), traced))
}

/// Stops the ETW session even when an assertion fails.
struct Session(String);

impl Drop for Session {
    fn drop(&mut self) {
        let _ = Command::new("logman")
            .args(["stop", &self.0, "-ets"])
            .output();
    }
}

fn etw(runs: &[Run]) -> Option<(String, Vec<Traced>)> {
    let dir = tempfile::tempdir().unwrap();
    let etl = dir.path().join("afd.etl");
    let nanos = (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH))
        .unwrap()
        .subsec_nanos();
    let name = format!("voxxa-no-sockets-{}-{nanos}", std::process::id());
    let start = Command::new("logman")
        .args(["start", &name, "-p", "Microsoft-Windows-Winsock-AFD"])
        .args(["0xFFFFFFFFFFFFFFFF", "0xFF", "-o"])
        .arg(&etl)
        .arg("-ets")
        .output()
        .ok()?;
    if !start.status.success() {
        return None;
    }
    let session = Session(name);
    // All at once: the session is system-wide, and events carry the pid.
    let children: Vec<_> = (runs.iter())
        .map(|run| {
            let child = (command(run).stdout(std::process::Stdio::piped()).spawn()).unwrap();
            (child.id(), child)
        })
        .collect();
    let outputs: Vec<_> = (children.into_iter())
        .map(|(pid, child)| (pid, child.wait_with_output().unwrap()))
        .collect();
    drop(session);
    // Event counts per process id, read back with PowerShell.
    let script = format!(
        "Get-WinEvent -Path '{}' -Oldest -ErrorAction SilentlyContinue | \
         Group-Object ProcessId | ForEach-Object {{ \"$($_.Name) $($_.Count)\" }}",
        etl.display()
    );
    let read = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output()
        .unwrap();
    let counts: Vec<(u32, usize)> = (String::from_utf8_lossy(&read.stdout).lines())
        .filter_map(|l| {
            let (pid, n) = l.trim().split_once(' ')?;
            Some((pid.parse().ok()?, n.parse().ok()?))
        })
        .collect();
    let traced = (outputs.into_iter())
        .map(|(pid, out)| {
            let events = (counts.iter())
                .filter(|(p, _)| *p == pid)
                .map(|(_, n)| n)
                .sum();
            Traced {
                success: out.status.success(),
                stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
                events,
                sample: vec![format!("{events} Winsock-AFD events for pid {pid}")],
            }
        })
        .collect();
    Some((
        "ETW Microsoft-Windows-Winsock-AFD (networking enabled)".into(),
        traced,
    ))
}
