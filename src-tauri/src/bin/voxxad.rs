//! voxxad: Voxxa without the window (ADR-001 as adjusted by ADR-010). It runs
//! the GUI's listening loop (`voxxa_lib::listen`) on a WAV file or a device
//! and prints one JSON object per line. Every line, plus the conductor's
//! ticks, also goes to `session-YYYYMMDD-HHMM.jsonl` in the data directory;
//! `--replay` feeds a session's ticks and hypotheses to a fresh conductor and
//! prints the same lines again, without audio or Whisper.

use anyhow::{anyhow, bail, ensure, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::Mutex;
use voxxa_lib::listen::{self, Heard};
use voxxa_lib::{
    check_setlist, drive, make_controller, Action, AudioEngine, Conductor, KeystrokeProfile,
    MachineState, PresentationController, PresenterConfig, PresenterKind, Sent, Setlist,
    SmartConfig, TranscriptionEngine, VadEngine,
};

const USAGE: &str = "usage: voxxad --setlist <json> (--stdin-audio <wav|-> [--realtime] | --device <name> | --replay <session.jsonl>)
              [--presenter keystroke|propresenter7_rest|freeshow|openlp_v2|opensong|none] [--host <h>] [--port <p>] [--shadow]
       voxxad --list-devices
The WAV must be 16-bit PCM, 16 kHz mono. --shadow computes and prints actions but sends nothing.";

/// Session logs older than this are deleted when a new session starts.
const KEEP_SESSIONS: Duration = Duration::from_secs(14 * 24 * 3600);

/// One line on stdout and in the session log. `t` is ms since the session start.
#[derive(Serialize, Deserialize)]
struct Line {
    v: u8,
    t: u64,
    #[serde(flatten)]
    ev: Ev,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "ev", rename_all = "snake_case")]
enum Ev {
    Status {
        state: MachineState,
        blank: bool,
    },
    Hypothesis {
        text: String,
    },
    Action {
        action: Action,
    },
    Error {
        message: String,
    },
    /// A conductor tick, after speech last heard at `speech` ms. Session log only.
    Tick {
        #[serde(skip_serializing_if = "Option::is_none")]
        speech: Option<u64>,
    },
}

#[derive(Default)]
struct Opts {
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
}

fn parse_args() -> Result<Opts> {
    let mut o = Opts::default();
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let mut value = || {
            args.next()
                .ok_or_else(|| anyhow!("{flag} needs a value\n{USAGE}"))
        };
        match flag.as_str() {
            "--setlist" => o.setlist = Some(value()?),
            "--stdin-audio" => o.wav = Some(value()?),
            "--realtime" => o.realtime = true,
            "--device" => o.device = Some(value()?),
            "--presenter" => o.presenter = Some(value()?),
            "--host" => o.host = Some(value()?),
            "--port" => o.port = Some(value()?.parse().context("--port")?),
            "--shadow" => o.shadow = true,
            "--replay" => o.replay = Some(value()?),
            "--list-devices" => o.list_devices = true,
            "-h" | "--help" => {
                eprintln!("{USAGE}");
                std::process::exit(0)
            }
            _ => bail!("unknown argument {flag}\n{USAGE}"),
        }
    }
    Ok(o)
}

fn presenter_kind(name: &str) -> Result<Option<PresenterKind>> {
    Ok(Some(match name {
        "keystroke" => PresenterKind::Keystroke,
        "propresenter7_rest" => PresenterKind::ProPresenter7Rest,
        "freeshow" => PresenterKind::FreeShow,
        "openlp_v2" => PresenterKind::OpenLpV2,
        "opensong" => PresenterKind::OpenSong,
        "none" => return Ok(None),
        _ => bail!("unknown presenter {name}\n{USAGE}"),
    }))
}

/// Samples of a 16-bit PCM, 16 kHz mono WAV, as the capture loop delivers them.
fn read_wav(b: &[u8]) -> Result<Vec<f32>> {
    ensure!(
        b.len() >= 12 && &b[..4] == b"RIFF" && &b[8..12] == b"WAVE",
        "not a WAV file"
    );
    let (mut at, mut pcm16k) = (12, false);
    while at + 8 <= b.len() {
        let len = u32::from_le_bytes(b[at + 4..at + 8].try_into()?) as usize;
        // A streamed WAV may announce more data than it has.
        let body = &b[at + 8..(at + 8).saturating_add(len).min(b.len())];
        let u16_at = |i: usize| u16::from_le_bytes([body[i], body[i + 1]]);
        match &b[at..at + 4] {
            b"fmt " if body.len() >= 16 => {
                let rate = u32::from_le_bytes(body[4..8].try_into()?);
                pcm16k = u16_at(0) == 1 && u16_at(2) == 1 && rate == 16_000 && u16_at(14) == 16;
            }
            b"data" => {
                ensure!(pcm16k, "the WAV must be 16-bit PCM, 16 kHz mono");
                let to_f32 = |s: &[u8; 2]| i16::from_le_bytes(*s) as f32 / i16::MAX as f32;
                return Ok(body.as_chunks().0.iter().map(to_f32).collect());
            }
            _ => {}
        }
        at = at.saturating_add(8 + len + (len & 1));
    }
    bail!("the WAV has no data chunk")
}

/// A new session log in the data directory; logs past `KEEP_SESSIONS` go.
fn session_log() -> Result<(File, PathBuf)> {
    let dir = dirs::data_dir().context("no data directory")?.join("voxxa");
    fs::create_dir_all(&dir)?;
    let cutoff = SystemTime::now() - KEEP_SESSIONS;
    for entry in fs::read_dir(&dir)?.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let modified = entry.metadata().and_then(|m| m.modified());
        if name.starts_with("session-")
            && name.ends_with(".jsonl")
            && modified.is_ok_and(|m| m < cutoff)
        {
            let _ = fs::remove_file(entry.path());
        }
    }
    let stamp = chrono::Local::now()
        .format("session-%Y%m%d-%H%M")
        .to_string();
    for n in 1.. {
        let name = match n {
            1 => format!("{stamp}.jsonl"),
            _ => format!("{stamp}-{n}.jsonl"),
        };
        let path = dir.join(name);
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            r => return Ok((r?, path)),
        }
    }
    unreachable!()
}

/// Where lines go, and the presenter actions are sent to (none in shadow mode).
struct Out {
    t0: Instant,
    log: Option<File>,
    presenter: Option<(
        tokio::runtime::Runtime,
        Box<dyn PresentationController>,
        Sent,
    )>,
}

impl Out {
    fn write(&mut self, t: u64, ev: Ev) {
        let tick = matches!(ev, Ev::Tick { .. });
        let line = serde_json::to_string(&Line { v: 1, t, ev }).expect("events serialise");
        if let Some(log) = &mut self.log {
            if let Err(e) = writeln!(log, "{line}") {
                eprintln!("voxxad: session log: {e}");
            }
        }
        if !tick {
            println!("{line}");
        }
    }

    fn heard(&mut self, now: Instant, heard: Heard) {
        let t0 = self.t0;
        let ms = move |at: Instant| at.saturating_duration_since(t0).as_millis() as u64;
        let t = ms(now);
        let ev = match heard {
            Heard::Tick(speech) => Ev::Tick {
                speech: speech.map(ms),
            },
            Heard::Text(text) => Ev::Hypothesis { text: text.into() },
            Heard::State(state, blank) => Ev::Status { state, blank },
            Heard::Action(Action::Noop, _) | Heard::Samples(_) | Heard::Scores(_) => return,
            Heard::Action(action, _) => {
                self.write(
                    t,
                    Ev::Action {
                        action: action.clone(),
                    },
                );
                if let Some((rt, p, sent)) = &mut self.presenter {
                    if let Err(e) = rt.block_on(drive(p.as_ref(), sent, &action, false, false)) {
                        self.write(
                            t,
                            Ev::Error {
                                message: e.to_string(),
                            },
                        );
                    }
                }
                return;
            }
        };
        self.write(t, ev);
    }
}

fn replay(path: &str, conductor: &Mutex<Option<Conductor>>, out: &mut Out) -> Result<()> {
    let t0 = out.t0;
    let at = |ms: u64| t0 + Duration::from_millis(ms);
    let mut shown = (MachineState::Listening, true);
    let file = File::open(path).with_context(|| format!("open {path}"))?;
    for (i, line) in BufReader::new(file).lines().enumerate() {
        let line: Line =
            serde_json::from_str(&line?).with_context(|| format!("{path}:{}", i + 1))?;
        ensure!(line.v == 1, "{path}:{}: unknown version {}", i + 1, line.v);
        // Every other line is output, recomputed here.
        match line.ev {
            Ev::Tick { speech } => {
                let speech = speech.map(at);
                listen::tick(conductor, speech, at(line.t), &mut shown, &mut |n, h| {
                    out.heard(n, h)
                })
            }
            Ev::Hypothesis { text } => {
                listen::transcript(conductor, &text, at(line.t), &mut |n, h| out.heard(n, h))
            }
            _ => {}
        }
    }
    Ok(())
}

fn run() -> Result<()> {
    let o = parse_args()?;
    if o.list_devices {
        for name in AudioEngine::list_devices()? {
            println!("{name}");
        }
        return Ok(());
    }
    let kind = presenter_kind(o.presenter.as_deref().unwrap_or("none"))?;
    let path = o
        .setlist
        .as_deref()
        .ok_or_else(|| anyhow!("--setlist is required\n{USAGE}"))?;
    let text = fs::read_to_string(path).with_context(|| format!("read {path}"))?;
    let songs = serde_json::from_str::<Setlist>(&text)
        .with_context(|| path.to_string())?
        .setlist;
    check_setlist(&songs).map_err(|e| anyhow!(e))?;
    let conductor = Mutex::new(Some(Conductor::new(songs, SmartConfig::default())));
    let t0 = Instant::now();
    let mut out = Out {
        t0,
        log: None,
        presenter: None,
    };
    if let Some(log) = &o.replay {
        return replay(log, &conductor, &mut out);
    }

    let wav = match o.wav.as_deref() {
        Some("-") => {
            let mut bytes = Vec::new();
            std::io::stdin().read_to_end(&mut bytes)?;
            Some(read_wav(&bytes)?)
        }
        Some(p) => Some(read_wav(
            &fs::read(p).with_context(|| format!("read {p}"))?,
        )?),
        None if o.device.is_some() => None,
        None => bail!("give --stdin-audio, --device or --replay\n{USAGE}"),
    };
    let mut tr = TranscriptionEngine::new();
    let (model, ..) = tr.model_status().into_iter().find(|m| m.1).ok_or_else(|| {
        anyhow!(
            "no Whisper model in {}; download one in Voxxa",
            tr.model_dir().display()
        )
    })?;
    tr.load_model(&model)?;
    if let (Some(kind), false) = (kind, o.shadow) {
        let mut p = make_controller(kind);
        let rt = tokio::runtime::Runtime::new()?;
        let cfg = PresenterConfig {
            keystroke_profile: Some(KeystrokeProfile::Universal),
            host: o.host.clone(),
            port: o.port,
            ..Default::default()
        };
        rt.block_on(p.connect(cfg))?;
        out.presenter = Some((rt, p, Sent::default()));
    }

    let mut audio = AudioEngine::new();
    let (rx, realtime) = match wav {
        Some(samples) => {
            let (tx, rx) = mpsc::sync_channel(64);
            let realtime = o.realtime;
            std::thread::spawn(move || {
                // 100 ms chunks, so the file clock stays on whole milliseconds.
                for chunk in samples.chunks(1600) {
                    if tx.send(chunk.to_vec()).is_err() {
                        break;
                    }
                    if realtime {
                        std::thread::sleep(Duration::from_millis(100));
                    }
                }
            });
            (rx, realtime)
        }
        None => {
            audio.select_device(o.device.clone());
            (audio.start()?, true)
        }
    };
    let (log, log_path) = session_log()?;
    eprintln!("voxxad: session log {}", log_path.display());
    out.log = Some(log);

    // Whole milliseconds, so --replay feeds the conductor exactly these times.
    let clock = |fed: u64| {
        let ms = if realtime {
            t0.elapsed().as_millis() as u64
        } else {
            fed / 16
        };
        t0 + Duration::from_millis(ms)
    };
    let (vad, tr) = (Mutex::new(VadEngine::new()), Mutex::new(tr));
    let run = AtomicBool::new(true);
    let ended = listen::listen(rx, &run, clock, &vad, &tr, &conductor, |n, h| {
        out.heard(n, h)
    });
    if ended && o.device.is_some() {
        bail!("the audio device was lost");
    }
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        let line = Line {
            v: 1,
            t: 0,
            ev: Ev::Error {
                message: format!("{e:#}"),
            },
        };
        println!(
            "{}",
            serde_json::to_string(&line).expect("events serialise")
        );
        std::process::exit(1);
    }
}
