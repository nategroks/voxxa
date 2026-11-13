//! `voxxa-eval`: scores `voxxad` against an operator's truth (ADR-015).
//! It runs the `voxxad` binary beside it, so it scores exactly what
//! `voxxad` prints and reimplements nothing of the conductor.

mod audio;
mod score;

use score::Metrics;
use serde_json::Value;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use voxxa_lib::Setlist;

const USAGE: &str = "\
usage: voxxa-eval [--full [--record]] [--baseline <json>] [<fixture dir>...]
Scores voxxad's actions against each fixture's truth.jsonl (default: every
folder in fixtures/recordings) and prints precision, recall, median/p95
latency, false and double advances, blank and unblank timing.
  (default)   replay <fixture>/session.jsonl: no model, no audio
  --full      voxxad --stdin-audio <fixture>/audio.wav --shadow --model tiny;
              audio.wav is rebuilt from audio.json first when there is one
  --record    with --full: save the session log as <fixture>/session.jsonl
  --baseline  thresholds (default fixtures/baseline.json); exit 1 when a
              metric is worse, 2 on an error";

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../fixtures");

fn main() {
    match run(std::env::args().skip(1)) {
        Ok(true) => {}
        Ok(false) => std::process::exit(1),
        Err(e) => {
            eprintln!("voxxa-eval: {e}");
            std::process::exit(2);
        }
    }
}

fn run(mut args: impl Iterator<Item = String>) -> Result<bool, String> {
    let (mut full, mut record) = (false, false);
    let (mut baseline, mut dirs) = (format!("{FIXTURES}/baseline.json"), Vec::new());
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--full" => full = true,
            "--record" => record = true,
            "--baseline" => baseline = args.next().ok_or("--baseline needs a value")?,
            "--help" | "-h" => {
                println!("{USAGE}");
                return Ok(true);
            }
            flag if flag.starts_with('-') => return Err(format!("unknown flag {flag}\n{USAGE}")),
            _ => dirs.push(PathBuf::from(arg)),
        }
    }
    if record && !full {
        return Err("--record needs --full".into());
    }
    if dirs.is_empty() {
        let all = fs::read_dir(format!("{FIXTURES}/recordings")).map_err(|e| e.to_string())?;
        dirs = all
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        dirs.sort();
    }
    let limits: Value = serde_json::from_str(&read(Path::new(&baseline))?)
        .map_err(|e| format!("{baseline}: {e}"))?;

    let mut rows = Vec::new();
    for dir in &dirs {
        let name = dir.file_name().unwrap_or_default().to_string_lossy();
        let setlist = dir.join("setlist.json");
        let songs = serde_json::from_str::<Setlist>(&read(&setlist)?)
            .map_err(|e| format!("{}: {e}", setlist.display()))?;
        let songs: Vec<_> = (songs.setlist.into_iter())
            .map(|s| (s.title, s.slides.len()))
            .collect();
        let truth = score::parse_truth(&read(&dir.join("truth.jsonl"))?, &songs)
            .map_err(|e| format!("{name}: {e}"))?;
        let stdout = if full {
            run_full(dir, &setlist, record)?
        } else {
            let session = dir.join("session.jsonl");
            let replay = "--replay".as_ref();
            let args = [
                replay,
                session.as_os_str(),
                "--setlist".as_ref(),
                setlist.as_os_str(),
            ];
            voxxad(&args, None)?
        };
        // Only a recognizer run prints a start line; replay never does.
        if stdout.contains(r#""state":"start""#) != full {
            return Err(format!("{name}: voxxad did not run in the mode asked for"));
        }
        for error in stdout.lines().filter(|l| l.contains(r#""type":"error""#)) {
            eprintln!("voxxa-eval: {name}: voxxad reported {error}");
        }
        let actions = score::parse_actions(&stdout).map_err(|e| format!("{name}: {e}"))?;
        rows.push((name.into_owned(), score::score(&truth, &actions)));
    }
    let mut total = Metrics::default();
    rows.iter().for_each(|(_, m)| total.add(m));
    rows.push(("total".into(), total));

    let mode = if full {
        "full (Whisper Tiny)"
    } else {
        "replay"
    };
    println!("mode: {mode}\n{}", score::table(&rows));
    let fails: Vec<String> = (rows.iter())
        .flat_map(|(name, m)| score::check(name, m, &limits))
        .collect();
    for fail in &fails {
        println!("FAIL {fail}");
    }
    if fails.is_empty() {
        println!("OK: no metric is worse than {baseline}");
    }
    Ok(fails.is_empty())
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

/// Runs the `voxxad` built beside this binary (in `cwd` if given) and
/// returns its stdout.
fn voxxad(args: &[&OsStr], cwd: Option<&Path>) -> Result<String, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe = exe.with_file_name(format!("voxxad{}", std::env::consts::EXE_SUFFIX));
    let mut cmd = Command::new(&exe);
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    let out = (cmd.args(args).stderr(Stdio::inherit()).output())
        .map_err(|e| format!("{}: {e} (build it: cargo build --bins)", exe.display()))?;
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    match out.status.success() {
        true => Ok(stdout),
        false => Err(format!("voxxad failed: {}", stdout.trim())),
    }
}

/// `voxxad` on the fixture's audio with Whisper Tiny, in shadow mode at
/// file speed. It runs in a temporary folder with `--log-dir .`, so the
/// session log's `log` path is relative and names no user folder. With
/// `record`, that log is copied to `session.jsonl`.
fn run_full(dir: &Path, setlist: &Path, record: bool) -> Result<String, String> {
    let abs = |p: &Path| std::path::absolute(p).map_err(|e| format!("{}: {e}", p.display()));
    let (dir, setlist) = (abs(dir)?, abs(setlist)?);
    let wav = dir.join("audio.wav");
    let recipe = dir.join("audio.json");
    if recipe.exists() {
        let bytes = audio::build(&dir, &read(&recipe)?)?;
        fs::write(&wav, bytes).map_err(|e| format!("{}: {e}", wav.display()))?;
    }
    let logs = tempfile::tempdir().map_err(|e| e.to_string())?;
    let mut args: Vec<&OsStr> = vec!["--stdin-audio".as_ref(), wav.as_ref()];
    args.extend(["--setlist".as_ref(), setlist.as_os_str()]);
    args.extend(["--shadow", "--model", "tiny", "--log-dir", "."].map(OsStr::new));
    let stdout = voxxad(&args, Some(logs.path()))?;
    if record {
        let log = (fs::read_dir(logs.path())
            .map_err(|e| e.to_string())?
            .flatten())
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|x| x == "jsonl"))
        .ok_or("voxxad wrote no session log")?;
        let out = dir.join("session.jsonl");
        fs::copy(&log, &out).map_err(|e| format!("{}: {e}", out.display()))?;
    }
    Ok(stdout)
}
