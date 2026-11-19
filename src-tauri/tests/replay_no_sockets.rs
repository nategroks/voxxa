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
//!   namespace: networking is disabled, only a down loopback exists. With
//!   `VOXXA_REQUIRE_NETNS=1` (CI's Linux job) the test fails when `unshare -rn`
//!   does not work, instead of tracing with networking enabled.
//! - **Windows:** an ETW session on the `Microsoft-Windows-Winsock-AFD`
//!   provider records every socket the kernel's AFD driver creates or
//!   connects, with the process id; `Microsoft-Windows-Kernel-Process` in the
//!   same session records when each child started and exited. A replay is
//!   charged only with socket-create and connect events of its own pid that
//!   were logged between its own start and exit, so a pid that Windows reuses
//!   before or after it does not count. Starting the session needs an
//!   administrator (GitHub's Windows runners are). Networking stays enabled.
//! - **macOS and anything else:** no tracer that works without disabling SIP,
//!   so the test prints why and passes. The replay code is the same Rust on
//!   every OS (`src/cli/voxxad/events.rs`, no OS-specific code).
//!
//! Each run first traces a positive control, this test binary connecting to
//! 127.0.0.1 (`socket_probe`), and fails if the tracer did not see it: a
//! tracer that sees nothing cannot pass the test. Set
//! `VOXXA_REQUIRE_SOCKET_TRACE=1` (CI does, on Linux and Windows) to fail
//! instead of skipping when no tracer is available.

mod common;

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

/// The binary is built only with `--features cli`. Cargo sets `CARGO_BIN_EXE_voxxad`
/// even when it skips the binary, and the path may hold a stale build,
/// so the feature is checked: a plain `cargo test` fails here, loudly.
fn voxxad_exe() -> &'static str {
    if !cfg!(feature = "cli") {
        panic!("run: cargo test --features cli");
    }
    env!("CARGO_BIN_EXE_voxxad")
}

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
    /// Network events the tracer charged to this run's process.
    events: usize,
    /// A few of them, for the failure message (no transcript text).
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
    (common::committed_logs().into_iter())
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
    let required = std::env::var("VOXXA_REQUIRE_NETNS").as_deref() == Ok("1");
    let how = strace_mode(unshare, required).unwrap_or_else(|why| panic!("{why}"));
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
    Some((how.into(), traced))
}

/// The Linux tracer's label, or why it must not run: `VOXXA_REQUIRE_NETNS=1`
/// turns a missing `unshare -rn` into a failure. Without it, a runner bump
/// (Ubuntu 24.04's AppArmor refuses unprivileged user namespaces) would
/// quietly trace with networking enabled, and a green log would not say so.
fn strace_mode(unshare: bool, require_netns: bool) -> Result<&'static str, String> {
    match (unshare, require_netns) {
        (true, _) => Ok("strace inside unshare -rn (networking disabled)"),
        (false, false) => Ok("strace (unshare -rn unavailable: networking enabled)"),
        (false, true) => Err("VOXXA_REQUIRE_NETNS=1 but `unshare -rn true` failed, \
             so networking cannot be disabled for the replays"
            .into()),
    }
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
    let session = Session(name.clone());
    // Process start and exit (keyword 0x10), for each child's lifetime.
    let process = Command::new("logman")
        .args([
            "update",
            "trace",
            &name,
            "-p",
            "Microsoft-Windows-Kernel-Process",
        ])
        .args(["0x10", "0xFF", "-ets"])
        .output()
        .unwrap();
    assert!(
        process.status.success(),
        "logman could not add Microsoft-Windows-Kernel-Process: {}",
        String::from_utf8_lossy(&process.stdout).trim()
    );
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
    let script = ETW_READ.replace("ETL_PATH", &etl.display().to_string());
    let read = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output()
        .unwrap();
    let events = parse_etw(&String::from_utf8_lossy(&read.stdout));
    let me = std::process::id();
    let traced = (runs.iter().zip(outputs))
        .map(|(run, (pid, out))| {
            let life = lifetime(&events, pid, me).unwrap_or_else(|| {
                panic!("{}: ETW recorded no start and exit for pid {pid}", run.name)
            });
            let ids = socket_events(&events, pid, life);
            Traced {
                success: out.status.success(),
                stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
                events: ids.len(),
                sample: vec![format!(
                    "{} socket-create events for pid {pid} during its lifetime, ids {ids:?}",
                    ids.len()
                )],
            }
        })
        .collect();
    Some((
        "ETW Microsoft-Windows-Winsock-AFD (networking enabled)".into(),
        traced,
    ))
}

/// Reads the ETL back, one line per event that matters (FILETIME ticks):
/// `S pid parent at` (process start), `E pid at` (process exit) and
/// `A pid event-id at` (any Winsock-AFD event; `socket_events` filters them).
const ETW_READ: &str = r#"
Get-WinEvent -Path 'ETL_PATH' -Oldest -ErrorAction SilentlyContinue | ForEach-Object {
  $at = $_.TimeCreated.ToFileTimeUtc()
  if ($_.ProviderName -eq 'Microsoft-Windows-Kernel-Process') {
    $d = @{}
    ([xml]$_.ToXml()).Event.EventData.Data | ForEach-Object { $d[$_.Name] = $_.'#text' }
    if ($_.Id -eq 1) { "S $($d.ProcessID) $($d.ParentProcessID) $at" }
    elseif ($_.Id -eq 2) { "E $($d.ProcessID) $at" }
  } elseif ($_.ProviderName -eq 'Microsoft-Windows-Winsock-AFD') {
    "A $($_.ProcessId) $($_.Id) $at"
  }
}
"#;

/// One line of `ETW_READ`'s output.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Etw {
    Start { pid: u32, parent: u32, at: u64 },
    Stop { pid: u32, at: u64 },
    Afd { pid: u32, id: u16, at: u64 },
}

fn parse_etw(text: &str) -> Vec<Etw> {
    (text.lines())
        .filter_map(|line| {
            let f: Vec<&str> = line.split_whitespace().collect();
            Some(match f[..] {
                ["S", pid, parent, at] => Etw::Start {
                    pid: pid.parse().ok()?,
                    parent: parent.parse().ok()?,
                    at: at.parse().ok()?,
                },
                ["E", pid, at] => Etw::Stop {
                    pid: pid.parse().ok()?,
                    at: at.parse().ok()?,
                },
                ["A", pid, id, at] => Etw::Afd {
                    pid: pid.parse().ok()?,
                    id: id.parse().ok()?,
                    at: at.parse().ok()?,
                },
                _ => return None,
            })
        })
        .collect()
}

/// When the child `pid` of `parent` ran: from its start event to the first
/// exit event of that pid after it. A process that has the same pid before
/// or after it (Windows reuses pids) has its own start and exit events.
fn lifetime(events: &[Etw], pid: u32, parent: u32) -> Option<(u64, u64)> {
    let start = events.iter().find_map(|e| match *e {
        Etw::Start {
            pid: p,
            parent: q,
            at,
        } if p == pid && q == parent => Some(at),
        _ => None,
    })?;
    let stop = (events.iter())
        .filter_map(|e| match *e {
            Etw::Stop { pid: p, at } if p == pid && at >= start => Some(at),
            _ => None,
        })
        .min()?;
    Some((start, stop))
}

/// Winsock-AFD events that mean "this process made a socket": socket
/// creation (1000), which AFD logs in the creating process's context. A
/// connect, send, receive or close needs a socket first, so counting only
/// creation loses no socket. The later events can be logged in whatever
/// process is current when the I/O completes: on a busy windows-latest
/// runner, six ConnectEx (1020) events with no socket were charged to a
/// replay's pid (PR #95, first CI run), so they must not count.
const SOCKET_EVENTS: [u16; 1] = [1000];

/// The ids of `pid`'s socket events from inside `life` (start to exit).
fn socket_events(events: &[Etw], pid: u32, (from, to): (u64, u64)) -> Vec<u16> {
    (events.iter())
        .filter_map(|e| match *e {
            Etw::Afd { pid: p, id, at }
                if p == pid && (from..=to).contains(&at) && SOCKET_EVENTS.contains(&id) =>
            {
                Some(id)
            }
            _ => None,
        })
        .collect()
}

#[test]
fn etw_charges_a_replay_only_with_its_own_sockets() {
    let me = 7;
    let events = [
        // pid 40 before our child: another process, already exited.
        Etw::Start {
            pid: 40,
            parent: 3,
            at: 1,
        },
        Etw::Afd {
            pid: 40,
            id: 1000,
            at: 2,
        },
        Etw::Stop { pid: 40, at: 3 },
        Etw::Afd {
            pid: 40,
            id: 1018,
            at: 4,
        },
        // Our child.
        Etw::Start {
            pid: 40,
            parent: me,
            at: 10,
        },
        Etw::Afd {
            pid: 40,
            id: 1003,
            at: 11,
        }, // a send: not a socket creation
        Etw::Afd {
            pid: 40,
            id: 1020,
            at: 11,
        }, // a ConnectEx completion with no socket: logged in our context
        Etw::Afd {
            pid: 41,
            id: 1000,
            at: 12,
        }, // another pid
        Etw::Stop { pid: 40, at: 20 },
        // pid 40 reused after our child exited.
        Etw::Afd {
            pid: 40,
            id: 1000,
            at: 21,
        },
        Etw::Start {
            pid: 40,
            parent: 3,
            at: 30,
        },
        Etw::Afd {
            pid: 40,
            id: 1017,
            at: 31,
        },
        Etw::Stop { pid: 40, at: 40 },
    ];
    let life = lifetime(&events, 40, me);
    assert_eq!(life, Some((10, 20)));
    assert_eq!(socket_events(&events, 40, life.unwrap()), Vec::<u16>::new());

    // The same child making a socket (and connecting) is caught.
    let mut caught = events.to_vec();
    caught.push(Etw::Afd {
        pid: 40,
        id: 1000,
        at: 12,
    });
    caught.push(Etw::Afd {
        pid: 40,
        id: 1018,
        at: 13,
    });
    assert_eq!(socket_events(&caught, 40, (10, 20)), vec![1000]);

    // No start event from this parent, or no exit after it: no lifetime,
    // and `etw` fails the test instead of guessing.
    assert_eq!(lifetime(&events, 40, 99), None);
    assert_eq!(lifetime(&events[..7], 40, me), None);
}

#[test]
fn etw_read_back_lines_parse() {
    let text = "S 40 7 10\r\nA 40 1000 12\r\nE 40 20\r\nWARNING: x\r\n\r\nA 4 1003\r\n";
    assert_eq!(
        parse_etw(text),
        vec![
            Etw::Start {
                pid: 40,
                parent: 7,
                at: 10
            },
            Etw::Afd {
                pid: 40,
                id: 1000,
                at: 12
            },
            Etw::Stop { pid: 40, at: 20 },
        ]
    );
}

#[test]
fn a_required_network_namespace_fails_without_unshare() {
    assert!(strace_mode(true, true)
        .unwrap()
        .contains("networking disabled"));
    assert!(strace_mode(true, false)
        .unwrap()
        .contains("networking disabled"));
    assert!(strace_mode(false, false)
        .unwrap()
        .contains("networking enabled"));
    assert!(strace_mode(false, true).is_err());
}
