//! `voxxa-eval` (ADR-015): runs `voxxad` on each fixture, scores its
//! `action` lines against the operator's `truth.jsonl` and prints one table.
//! With `--baseline` it exits 1 when a metric is worse than its threshold.
//! It never links the conductor: `voxxad` is a subprocess, replaying a
//! committed session log (the default, as in CI) or, with `--full`,
//! listening to the fixture's audio in shadow mode.

mod audio;
mod metrics;

use metrics::{check, parse_actions, parse_setlist, parse_truth, score, table, Report};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const USAGE: &str =
    "usage: voxxa-eval [--baseline <json>] [--fixtures <dir>] [--voxxad <path>] [name...]
       voxxa-eval --full [--model <name>] [--record] [--fixtures <dir>] [name...]
       voxxa-eval --write-audio [--fixtures <dir>] [name...]
A fixture is a folder with setlist.json and truth.jsonl (default: fixtures/recordings/*).
Default: replay <fixture>/session.jsonl (no model, no audio). --full: voxxad --shadow on
audio.wav or the audio.json recipe; --record saves its log as session.jsonl. --baseline:
exit 1 if a metric is worse than a threshold. Exit 2 on an error. See fixtures/README.md.";

#[derive(Default)]
struct Args {
    fixtures: Option<PathBuf>,
    baseline: Option<PathBuf>,
    voxxad: Option<PathBuf>,
    model: Option<String>,
    full: bool,
    record: bool,
    write_audio: bool,
    help: bool,
    names: Vec<String>,
}

fn parse_args(mut it: impl Iterator<Item = String>) -> Result<Args, String> {
    let mut a = Args::default();
    while let Some(flag) = it.next() {
        let mut value = || it.next().ok_or(format!("{flag} needs a value"));
        match flag.as_str() {
            "--fixtures" => a.fixtures = Some(value()?.into()),
            "--baseline" => a.baseline = Some(value()?.into()),
            "--voxxad" => a.voxxad = Some(value()?.into()),
            "--model" => a.model = Some(value()?),
            "--full" => a.full = true,
            "--record" => a.record = true,
            "--write-audio" => a.write_audio = true,
            "--help" | "-h" => a.help = true,
            f if f.starts_with('-') => return Err(format!("unknown argument {f}\n{USAGE}")),
            name => a.names.push(name.into()),
        }
    }
    if (a.record || a.model.is_some()) && !a.full {
        return Err("--record and --model need --full".into());
    }
    Ok(a)
}

fn main() -> ExitCode {
    match run() {
        Ok(0) => ExitCode::SUCCESS,
        Ok(_) => ExitCode::from(1),
        Err(message) => {
            eprintln!("voxxa-eval: {message}");
            ExitCode::from(2)
        }
    }
}

/// The number of gate failures.
fn run() -> Result<usize, String> {
    let args = parse_args(std::env::args().skip(1))?;
    if args.help {
        println!("{USAGE}");
        return Ok(0);
    }
    let dir = (args.fixtures.clone()).unwrap_or_else(|| "fixtures/recordings".into());
    let fixtures = find_fixtures(&dir, &args.names)?;
    if args.write_audio {
        for fx in &fixtures {
            let wav = fx.join("audio.wav");
            fs::write(&wav, audio::build(&fx.join("audio.json"))?).map_err(|e| e.to_string())?;
            println!("wrote {}", wav.display());
        }
        return Ok(0);
    }
    // Read the baseline first: a bad one should not cost a full run.
    let baseline = match &args.baseline {
        Some(path) => Some(read_baseline(path)?),
        None => None,
    };
    let voxxad = match &args.voxxad {
        Some(p) => p.clone(),
        None => (std::env::current_exe().map_err(|e| e.to_string())?)
            .with_file_name(format!("voxxad{}", std::env::consts::EXE_SUFFIX)),
    };
    if !voxxad.is_file() {
        let hint = "build it (cargo build --bin voxxad) or pass --voxxad";
        return Err(format!("{}: voxxad not found; {hint}", voxxad.display()));
    }
    let mode = if args.full {
        "full (voxxad --stdin-audio --shadow)"
    } else {
        "replay"
    };
    let (n, tolerance) = (fixtures.len(), metrics::TOLERANCE_MS);
    println!("voxxa-eval: {mode}, {n} fixture(s), tolerance {tolerance} ms");
    let mut rows = Vec::new();
    for fx in &fixtures {
        let name = fx
            .file_name()
            .map_or("?".into(), |n| n.to_string_lossy().into_owned());
        let at = |e: String| format!("{name}: {e}");
        let songs = parse_setlist(&read(&fx.join("setlist.json"))?).map_err(at)?;
        let truth = parse_truth(&read(&fx.join("truth.jsonl"))?, &songs).map_err(at)?;
        let stdout = run_voxxad(fx, &args, &voxxad).map_err(at)?;
        let report = score(&truth, &parse_actions(&stdout).map_err(at)?);
        rows.push((name, report));
    }
    let mut total = Report::default();
    rows.iter().for_each(|(_, r)| total.add(r));
    println!("{}", table(&rows, &total));
    let Some((path, per_set, totals)) = baseline else {
        return Ok(0);
    };
    let mut failures = Vec::new();
    for (name, report) in &rows {
        failures.extend(check(name, report, &per_set)?);
    }
    failures.extend(check("total", &total, &totals)?);
    for line in &failures {
        println!("{line}");
    }
    match failures.len() {
        0 => println!("gate: every metric meets {}", path.display()),
        n => println!("voxxa-eval: {n} metric(s) worse than {}", path.display()),
    }
    Ok(failures.len())
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

/// The fixture folders: the named ones, else every folder in `dir` holding
/// `truth.jsonl` and `setlist.json`, by name. None is an error.
fn find_fixtures(dir: &Path, names: &[String]) -> Result<Vec<PathBuf>, String> {
    let is_fixture = |p: &Path| p.join("truth.jsonl").is_file() && p.join("setlist.json").is_file();
    let mut found: Vec<PathBuf> = if names.is_empty() {
        let entries = fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        (entries.flatten().map(|e| e.path()))
            .filter(|p| is_fixture(p))
            .collect()
    } else {
        names.iter().map(|n| dir.join(n)).collect()
    };
    found.sort();
    if let Some(bad) = found.iter().find(|p| !is_fixture(p)) {
        return Err(format!(
            "{}: no truth.jsonl and setlist.json",
            bad.display()
        ));
    }
    if found.is_empty() {
        return Err(format!("{}: no fixtures", dir.display()));
    }
    Ok(found)
}

/// (path, per_set thresholds, total thresholds).
type Baseline = (
    PathBuf,
    serde_json::Map<String, Value>,
    serde_json::Map<String, Value>,
);

fn read_baseline(path: &Path) -> Result<Baseline, String> {
    let at = |e: &dyn std::fmt::Display| format!("{}: {e}", path.display());
    let v: Value = serde_json::from_str(&read(path)?).map_err(|e| at(&e))?;
    let object = v.as_object().ok_or_else(|| at(&"not a JSON object"))?;
    if let Some(key) = object
        .keys()
        .find(|k| !["v", "per_set", "total", "notes"].contains(&k.as_str()))
    {
        return Err(at(&format!("unknown key {key}")));
    }
    if v["v"] != 1 {
        return Err(at(&"need \"v\": 1"));
    }
    let section = |key: &str| match &v[key] {
        Value::Null => Ok(Default::default()),
        Value::Object(map) => Ok(map.clone()),
        _ => Err(at(&format!("{key} is not an object"))),
    };
    let (per_set, total) = (section("per_set")?, section("total")?);
    // Unknown metric names fail here, before any run.
    check("", &Report::default(), &per_set).map_err(|e| at(&e))?;
    check("", &Report::default(), &total).map_err(|e| at(&e))?;
    Ok((path.to_path_buf(), per_set, total))
}

/// `voxxad`'s stdout for one fixture: a replay of its `session.jsonl`, or
/// with `--full` a shadow run on its audio in a temporary folder (so the
/// session log lands there, not in `%APPDATA%`, and names no user folder).
fn run_voxxad(fx: &Path, args: &Args, voxxad: &Path) -> Result<String, String> {
    let fx = std::path::absolute(fx).map_err(|e| e.to_string())?;
    let setlist = fx.join("setlist.json");
    if !args.full {
        let log = fx.join("session.jsonl");
        if !log.is_file() {
            return Err("no session.jsonl; record one with --full --record".into());
        }
        let mut cmd = Command::new(voxxad);
        return output(cmd.arg("--replay").arg(log).arg("--setlist").arg(setlist));
    }
    let tmp = tempfile::tempdir().map_err(|e| e.to_string())?;
    let wav = match fx.join("audio.wav") {
        wav if wav.is_file() => wav,
        _ => {
            let wav = tmp.path().join("audio.wav");
            fs::write(&wav, audio::build(&fx.join("audio.json"))?).map_err(|e| e.to_string())?;
            wav
        }
    };
    let mut cmd = Command::new(voxxad);
    cmd.current_dir(tmp.path()).arg("--stdin-audio").arg(wav);
    cmd.arg("--setlist")
        .arg(setlist)
        .args(["--shadow", "--log-dir", "."]);
    if let Some(model) = &args.model {
        cmd.args(["--model", model]);
    }
    let stdout = output(&mut cmd)?;
    if args.record {
        let entries = fs::read_dir(tmp.path()).map_err(|e| e.to_string())?;
        let log = (entries.flatten().map(|e| e.path()))
            .find(|p| p.extension().is_some_and(|x| x == "jsonl"))
            .ok_or("voxxad wrote no session log")?;
        fs::copy(&log, fx.join("session.jsonl")).map_err(|e| e.to_string())?;
        eprintln!(
            "voxxa-eval: recorded {}",
            fx.join("session.jsonl").display()
        );
    }
    Ok(stdout)
}

/// Run `voxxad` to the end; its `error` lines go to stderr, and a failed
/// run is an error carrying its last line.
fn output(cmd: &mut Command) -> Result<String, String> {
    let out = cmd.output().map_err(|e| format!("voxxad: {e}"))?;
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    for line in stdout.lines().filter(|l| l.contains(r#""type":"error""#)) {
        eprintln!("voxxad: {line}");
    }
    if !out.status.success() {
        let last = stdout.lines().last().unwrap_or("").to_string();
        return Err(format!("voxxad failed ({}): {last}", out.status));
    }
    Ok(stdout)
}
