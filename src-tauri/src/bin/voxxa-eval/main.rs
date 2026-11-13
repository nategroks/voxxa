//! `voxxa-eval`: scores `voxxad` against an operator's truth (ADR-015).
//! It runs the `voxxad` binary beside it, so it scores exactly what
//! `voxxad` prints and reimplements nothing of the conductor.

mod audio;
mod score;

use score::Metrics;
use serde_json::Value;
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
            voxxad(&[
                replay,
                session.as_os_str(),
                "--setlist".as_ref(),
                setlist.as_os_str(),
            ])?
        };
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

/// Runs the `voxxad` built beside this binary and returns its stdout.
fn voxxad(args: &[&std::ffi::OsStr]) -> Result<String, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe = exe.with_file_name(format!("voxxad{}", std::env::consts::EXE_SUFFIX));
    let out = (Command::new(&exe)
        .args(args)
        .stderr(Stdio::inherit())
        .output())
    .map_err(|e| format!("{}: {e} (build it: cargo build --bins)", exe.display()))?;
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    match out.status.success() {
        true => Ok(stdout),
        false => Err(format!("voxxad failed: {}", stdout.trim())),
    }
}

/// `voxxad` on the fixture's audio with Whisper Tiny, in shadow mode at
/// file speed. With `record`, its session log becomes `session.jsonl`.
fn run_full(dir: &Path, setlist: &Path, record: bool) -> Result<String, String> {
    let wav = dir.join("audio.wav");
    let recipe = dir.join("audio.json");
    if recipe.exists() {
        let bytes = audio::build(dir, &read(&recipe)?)?;
        fs::write(&wav, bytes).map_err(|e| format!("{}: {e}", wav.display()))?;
    }
    let logs = tempfile::tempdir().map_err(|e| e.to_string())?;
    let stdout = voxxad(&[
        "--stdin-audio".as_ref(),
        wav.as_os_str(),
        "--setlist".as_ref(),
        setlist.as_os_str(),
        "--shadow".as_ref(),
        "--model".as_ref(),
        "tiny".as_ref(),
        "--log-dir".as_ref(),
        logs.path().as_os_str(),
    ])?;
    if record {
        let log = (fs::read_dir(logs.path())
            .map_err(|e| e.to_string())?
            .flatten())
        .map(|e| e.path())
        .next()
        .ok_or("voxxad wrote no session log")?;
        // The start line names the log's path; keep only a stable file name.
        let path = serde_json::to_string(&log.display().to_string()).unwrap_or_default();
        let text = read(&log)?.replacen(&path, r#""session.jsonl""#, 1);
        let out = dir.join("session.jsonl");
        fs::write(&out, text).map_err(|e| format!("{}: {e}", out.display()))?;
    }
    Ok(stdout)
}
