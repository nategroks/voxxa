//! `voxxad`: Voxxa without a window (ADR-001 as adjusted by ADR-010; ADR-013).
//! It runs the GUI's listening loop (`voxxa_lib::listen::run`) on a WAV file
//! or an input device, prints one JSON object per line on stdout and writes
//! every line to a session log that `--replay` re-runs without a recogniser.

mod events;

use chrono::Local;
use events::{fingerprint, status, Kind, Line, Recorder};
use serde_json::json;
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Read, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use voxxa_lib::listen::{self, check_setlist, Event};
use voxxa_lib::transcription::Rolling;
use voxxa_lib::{
    drive, make_controller, AudioEngine, Conductor, KeystrokeProfile, PresentationController,
    PresenterConfig, PresenterKind, Sent, Setlist, SmartConfig, Song, TranscriptionEngine,
    VadEngine, WhisperModel,
};

const USAGE: &str = "\
usage: voxxad --setlist <json> --stdin-audio <wav|-> [--realtime] [options]
       voxxad --setlist <json> --device <name|default> [options]
       voxxad --setlist <json> --replay <session.jsonl>
       voxxad --list-devices
options:
  --presenter keystroke|propresenter7_rest|freeshow|openlp_v2|opensong|none
              (default none; keystroke types into the focused window)
  --host <host> --port <port>   presenter address
  --shadow    compute and log actions, dispatch nothing
  --model tiny|base|small|medium|large-v3-turbo|distil-large-v3
              (default: the smallest downloaded model)
  --log-dir <dir>  session logs (default <data dir>/voxxa/sessions)
  --bias      bias Whisper towards the slides around the current one
The WAV must be 16 kHz mono, 16-bit PCM or 32-bit float. It is read at file
speed (t is audio time) unless --realtime is given.";

/// 100 ms of 16 kHz audio per chunk.
const CHUNK: usize = 1600;

#[derive(Default)]
struct Args {
    setlist: Option<String>,
    wav: Option<String>,
    realtime: bool,
    device: Option<String>,
    presenter: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    shadow: bool,
    replay: Option<String>,
    list_devices: bool,
    model: Option<String>,
    log_dir: Option<String>,
    bias: bool,
    help: bool,
}

fn parse_args(mut it: impl Iterator<Item = String>) -> Result<Args, String> {
    let mut a = Args::default();
    while let Some(flag) = it.next() {
        let mut value = || it.next().ok_or(format!("{flag} needs a value"));
        match flag.as_str() {
            "--setlist" => a.setlist = Some(value()?),
            "--stdin-audio" => a.wav = Some(value()?),
            "--device" => a.device = Some(value()?),
            "--presenter" => a.presenter = Some(value()?),
            "--host" => a.host = Some(value()?),
            "--port" => a.port = Some(value()?.parse().map_err(|e| format!("--port: {e}"))?),
            "--replay" => a.replay = Some(value()?),
            "--model" => a.model = Some(value()?),
            "--log-dir" => a.log_dir = Some(value()?),
            "--realtime" => a.realtime = true,
            "--shadow" => a.shadow = true,
            "--bias" => a.bias = true,
            "--list-devices" => a.list_devices = true,
            "--help" | "-h" => a.help = true,
            _ => return Err(format!("unknown argument {flag}; see voxxad --help")),
        }
    }
    Ok(a)
}

fn main() {
    if let Err(message) = run(parse_args(std::env::args().skip(1))) {
        say(&Line::new(0, Kind::Error { message }).to_json());
        std::process::exit(1);
    }
}

fn run(args: Result<Args, String>) -> Result<(), String> {
    let args = args?;
    if args.help {
        say(USAGE);
        return Ok(());
    }
    if args.list_devices {
        let devices = AudioEngine::list_devices().map_err(|e| e.to_string())?;
        let info = Some(json!({ "devices": devices }));
        say(&Line::new(0, status("devices", info)).to_json());
        return Ok(());
    }
    let live = [
        &args.wav,
        &args.device,
        &args.presenter,
        &args.host,
        &args.model,
        &args.log_dir,
    ];
    let flags = args.shadow || args.bias || args.port.is_some();
    let live = flags || live.iter().any(|o| o.is_some());
    if args.replay.is_some() && live {
        return Err("--replay takes only --setlist; see voxxad --help".into());
    }
    if args.realtime && args.wav.is_none() {
        return Err("--realtime needs --stdin-audio; see voxxad --help".into());
    }
    let setlist = args.setlist.as_deref().ok_or("--setlist is required")?;
    let songs = load_setlist(setlist)?;
    if let Some(path) = &args.replay {
        let log = fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        for line in events::replay(&log, songs).map_err(|e| format!("{path}: {e}"))? {
            say(&line.to_json());
        }
        return Ok(());
    }
    if args.presenter.as_deref() == Some("keystroke") && !args.shadow {
        let warning = "keystroke types into the focused window; focus the presentation app";
        let _ = writeln!(std::io::stderr(), "voxxad: {warning}");
    }
    let (rx, mode, _capture) = match (&args.wav, &args.device) {
        (Some(path), None) => {
            let mode = if args.realtime { "realtime" } else { "file" };
            (feed(read_wav(path)?, args.realtime), mode, None)
        }
        (None, Some(name)) => {
            let mut audio = AudioEngine::new();
            audio.select_device(Some(name.clone()).filter(|n| n != "default"));
            let rx = audio.start().map_err(|e| format!("audio: {e}"))?;
            (rx, "device", Some(audio))
        }
        _ => return Err("give one of --stdin-audio, --device or --replay".into()),
    };
    let rt = Runtime::new().map_err(|e| e.to_string())?;
    let presenter = connect_presenter(&args, &rt)?;
    let mut engine = TranscriptionEngine::new();
    let model = pick_model(&engine, args.model.as_deref())?;
    engine.load_model(&model).map_err(|e| e.to_string())?;
    let (path, mut log) = open_session(args.log_dir.as_deref())?;

    let config = SmartConfig::default();
    let driving = presenter.as_ref().and(args.presenter.as_deref());
    let info = json!({
        "mode": mode, "shadow": args.shadow, "model": model.filename(), "presenter": driving,
        "bias": args.bias,
        "setlist": fingerprint(&songs), "config": config,
        "log": path.display().to_string(), "version": env!("CARGO_PKG_VERSION"),
    });
    emit(&mut log, &Line::new(0, status("start", Some(info))));

    let base = Instant::now();
    let clock: Box<dyn FnMut(u64) -> Instant> = match mode {
        // Audio time: 16 samples per millisecond.
        "file" => Box::new(move |samples| base + Duration::from_millis(samples / 16)),
        // Wall time in whole milliseconds, so the log replays exactly.
        _ => Box::new(move |_| base + Duration::from_millis(base.elapsed().as_millis() as u64)),
    };
    let running = Arc::new(AtomicBool::new(true));
    let stop = running.clone();
    rt.spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            stop.store(false, Ordering::SeqCst);
        }
    });
    let conductor = Mutex::new(Some(Conductor::new(songs, config)));
    let vad = Mutex::new(VadEngine::new());
    let mut recorder = Recorder::new(base);
    let mut sent = Sent::default();
    let sink = |now: Instant, event: Event| {
        for line in recorder.lines(now, &event) {
            emit(&mut log, &line);
        }
        if let (Event::Action(action, _), Some(p)) = (&event, &presenter) {
            // The GUI's one dispatch path; voxxad has no window to guard.
            if let Err(e) = rt.block_on(drive(p.as_ref(), &mut sent, action, false, false)) {
                let message = format!("presenter: {e}");
                emit(
                    &mut log,
                    &Line::new(recorder.last_t, Kind::Error { message }),
                );
            }
        }
    };
    let mut recognizer = Rolling::new(engine);
    recognizer.bias_enabled = args.bias;
    let lost = listen::run(rx, &running, &vad, &conductor, &mut recognizer, clock, sink);
    let reason = match (mode, lost) {
        ("device", true) => "device_lost",
        (_, true) => "end_of_audio",
        _ => "stopped",
    };
    let info = Some(json!({ "reason": reason }));
    emit(&mut log, &Line::new(recorder.last_t, status("stop", info)));
    Ok(())
}

/// Write to the session log; `status`, `hypothesis`, `action` and `error`
/// also go to stdout, flushed per line so a pipe sees them at once.
fn emit(log: &mut File, line: &Line) {
    let json = line.to_json();
    let _ = writeln!(log, "{json}");
    if line.on_stdout() {
        say(&json);
    }
}

/// One line on stdout, flushed. A closed pipe (`| head -1`) is ignored
/// instead of panicking.
fn say(text: &str) {
    let mut out = std::io::stdout().lock();
    let _ = writeln!(out, "{text}").and_then(|_| out.flush());
}

fn load_setlist(path: &str) -> Result<Vec<Song>, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    let setlist: Setlist = serde_json::from_str(&text).map_err(|e| format!("{path}: {e}"))?;
    check_setlist(&setlist.setlist)?;
    Ok(setlist.setlist)
}

/// A 16 kHz mono WAV (16-bit PCM or 32-bit float) as samples; `-` is stdin.
fn read_wav(path: &str) -> Result<Vec<f32>, String> {
    let mut bytes = Vec::new();
    let read = match path {
        "-" => std::io::stdin().read_to_end(&mut bytes),
        _ => File::open(path).and_then(|mut f| f.read_to_end(&mut bytes)),
    };
    read.map_err(|e| format!("{path}: {e}"))?;
    let bad = |why: &str| format!("{path}: {why}");
    if bytes.get(..4) != Some(b"RIFF") || bytes.get(8..12) != Some(b"WAVE") {
        return Err(bad("not a WAV file"));
    }
    let u16_at = |b: &[u8], i: usize| b.get(i..i + 2).map(|s| u16::from_le_bytes([s[0], s[1]]));
    let u32_at = |b: &[u8], i: usize| {
        (b.get(i..i + 4)).map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
    };
    let (mut pos, mut format) = (12, None);
    while let (Some(id), Some(len)) = (bytes.get(pos..pos + 4), u32_at(&bytes, pos + 4)) {
        let end = (pos + 8 + len as usize).min(bytes.len());
        let body = &bytes[(pos + 8).min(end)..end];
        if id == b"fmt " {
            // Format tag, channels, sample rate, bits per sample.
            format = Some((
                u16_at(body, 0),
                u16_at(body, 2),
                u32_at(body, 4),
                u16_at(body, 14),
            ));
        } else if id == b"data" {
            return match format.ok_or_else(|| bad("data before fmt"))? {
                (Some(1), Some(1), Some(16_000), Some(16)) => Ok((body.as_chunks().0.iter())
                    .map(|&s| i16::from_le_bytes(s) as f32 / i16::MAX as f32)
                    .collect()),
                (Some(3), Some(1), Some(16_000), Some(32)) => Ok((body.as_chunks().0.iter())
                    .map(|&s| f32::from_le_bytes(s))
                    .collect()),
                (_, ch, rate, bits) => Err(bad(&format!(
                    "need 16 kHz mono, 16-bit PCM or 32-bit float; got {} Hz, {} channel(s), {} bits",
                    rate.unwrap_or(0),
                    ch.unwrap_or(0),
                    bits.unwrap_or(0)
                ))),
            };
        }
        pos += 8 + len as usize + (len as usize & 1);
    }
    Err(bad("no data chunk"))
}

/// Send the samples in 100 ms chunks, as fast as the loop takes them or, with
/// `realtime`, one chunk every 100 ms. The channel closes at the end. At most
/// 64 chunks (6.4 s) wait in it, so a long file is not copied twice.
fn feed(pcm: Vec<f32>, realtime: bool) -> mpsc::Receiver<Vec<f32>> {
    let (tx, rx) = mpsc::sync_channel(64);
    std::thread::spawn(move || {
        for chunk in pcm.chunks(CHUNK) {
            if realtime {
                std::thread::sleep(Duration::from_millis(100));
            }
            if tx.send(chunk.to_vec()).is_err() {
                break;
            }
        }
    });
    rx
}

/// `None` in shadow mode and for `none`: nothing is built, nothing is sent.
fn connect_presenter(
    args: &Args,
    rt: &Runtime,
) -> Result<Option<Box<dyn PresentationController>>, String> {
    let kind = match args.presenter.as_deref().unwrap_or("none") {
        "none" => return Ok(None),
        "keystroke" => PresenterKind::Keystroke,
        "propresenter7_rest" => PresenterKind::ProPresenter7Rest,
        "freeshow" => PresenterKind::FreeShow,
        "openlp_v2" => PresenterKind::OpenLpV2,
        "opensong" => PresenterKind::OpenSong,
        other => return Err(format!("unknown presenter {other}; see voxxad --help")),
    };
    if args.shadow {
        return Ok(None);
    }
    let mut presenter = make_controller(kind);
    let config = PresenterConfig {
        keystroke_profile: Some(KeystrokeProfile::Universal),
        host: args.host.clone(),
        port: args.port,
        ..Default::default()
    };
    rt.block_on(presenter.connect(config))
        .map_err(|e| format!("presenter: {e}"))?;
    Ok(Some(presenter))
}

/// `--model <name>`, else the smallest downloaded model.
fn pick_model(engine: &TranscriptionEngine, name: Option<&str>) -> Result<WhisperModel, String> {
    let wanted = name.map(|n| format!("ggml-{n}.bin"));
    (engine.model_status().into_iter())
        .find(|(m, downloaded, _)| match &wanted {
            Some(file) => m.filename() == file,
            None => *downloaded,
        })
        .map(|(m, _, _)| m)
        .ok_or_else(|| match name {
            Some(n) => format!("unknown model {n}; see voxxad --help"),
            None => "no Whisper model downloaded; download one in the Voxxa app".into(),
        })
}

/// A new `session-YYYYMMDD-HHMM[-N].jsonl`, after deleting logs older than
/// 14 days.
fn open_session(dir: Option<&str>) -> Result<(PathBuf, File), String> {
    let dir = match dir {
        Some(d) => PathBuf::from(d),
        None => (dirs::data_dir().ok_or("no data directory")?)
            .join("voxxa")
            .join("sessions"),
    };
    fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let now = Local::now();
    for entry in fs::read_dir(&dir).into_iter().flatten().flatten() {
        if events::expired(&entry.file_name().to_string_lossy(), now.date_naive()) {
            let _ = fs::remove_file(entry.path());
        }
    }
    let stem = format!("session-{}", now.format("%Y%m%d-%H%M"));
    for n in 1..1000 {
        let name = match n {
            1 => format!("{stem}.jsonl"),
            _ => format!("{stem}-{n}.jsonl"),
        };
        let path = dir.join(name);
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(e) if e.kind() == ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(format!("{}: {e}", path.display())),
        }
    }
    Err(format!("{}: too many sessions this minute", dir.display()))
}
