//! `voxxad`'s event schema v1 (ADR-013), the session recorder and replay.
//! Uses only `voxxa_lib`, so `tests/voxxad_events.rs` includes this file and
//! tests it without the binary, a microphone or a Whisper model.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use voxxa_lib::listen::{Event, Input};
use voxxa_lib::{Action, Conductor, MachineState, SmartConfig, Song};

/// Session logs older than this many days are deleted at start.
pub const KEEP_DAYS: i64 = 14;

/// One line of stdout or of the session log: `{"v":1,"t":<ms>,"type":…}`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Line {
    pub v: u8,
    /// Milliseconds since the session started, by the session's clock
    /// (audio time at file speed).
    pub t: u64,
    #[serde(flatten)]
    pub kind: Kind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Kind {
    /// `start`, a conductor state (`listening`, `singing`, …) or `stop`.
    Status {
        state: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        blank: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        info: Option<Value>,
    },
    /// A recogniser transcript; also the conductor input `Heard`.
    Hypothesis {
        text: String,
    },
    /// What the conductor asked the presenter to do.
    Action(Act),
    Error {
        message: String,
    },
    /// Session log only: the conductor tick.
    Tick,
    /// Session log only: VAD heard speech (the last one before the next
    /// tick or hypothesis; see `Conductor::on_speech`).
    Speech,
}

/// The conductor's [`Action`] on the wire. No slide text: the setlist has it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Act {
    Goto {
        song: usize,
        slide: usize,
        global: usize,
        title: String,
    },
    Blank,
    Unblank,
}

impl Act {
    /// `None` for `Noop`, which is never printed.
    pub fn from_action(action: &Action) -> Option<Act> {
        Some(match action {
            Action::Noop => return None,
            Action::Blank => Act::Blank,
            Action::Unblank => Act::Unblank,
            Action::Goto {
                song_index,
                slide_in_song,
                global_index,
                song_title,
                ..
            } => Act::Goto {
                song: *song_index,
                slide: *slide_in_song,
                global: *global_index,
                title: song_title.clone(),
            },
        })
    }
}

impl Line {
    pub fn new(t: u64, kind: Kind) -> Line {
        Line { v: 1, t, kind }
    }

    /// `status`, `hypothesis`, `action` and `error` go to stdout too.
    pub fn on_stdout(&self) -> bool {
        !matches!(self.kind, Kind::Tick | Kind::Speech)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("a Line always serialises")
    }
}

pub fn status(state: &str, info: Option<Value>) -> Kind {
    let state = state.to_string();
    Kind::Status {
        state,
        blank: None,
        info,
    }
}

fn machine(state: MachineState, blank: bool) -> Kind {
    let state = json!(state).as_str().unwrap_or_default().to_string();
    let blank = Some(blank);
    Kind::Status {
        state,
        blank,
        info: None,
    }
}

/// Identifies a setlist in the `start` line so replay can refuse another one
/// (FNV-1a over its JSON).
pub fn fingerprint(songs: &[Song]) -> Value {
    let bytes = serde_json::to_vec(songs).unwrap_or_default();
    let hash = (bytes.iter()).fold(0xcbf2_9ce4_8422_2325_u64, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    });
    let slides: usize = songs.iter().map(|s| s.slides.len()).sum();
    json!({ "songs": songs.len(), "slides": slides, "fnv1a": format!("{hash:016x}") })
}

/// Turns the listening loop's events into lines.
pub struct Recorder {
    base: Instant,
    pending_speech: Option<u64>,
    /// The latest `t` seen, for the `stop` line.
    pub last_t: u64,
}

impl Recorder {
    pub fn new(base: Instant) -> Recorder {
        Recorder {
            base,
            pending_speech: None,
            last_t: 0,
        }
    }

    pub fn lines(&mut self, now: Instant, event: &Event) -> Vec<Line> {
        let t = now.saturating_duration_since(self.base).as_millis() as u64;
        self.last_t = self.last_t.max(t);
        let kind = match event {
            Event::Input(Input::Speech) => {
                self.pending_speech = Some(t);
                return Vec::new();
            }
            Event::Input(Input::Tick) => Kind::Tick,
            Event::Input(Input::Heard(text)) => Kind::Hypothesis { text: text.clone() },
            Event::Action(action, _) => match Act::from_action(action) {
                Some(act) => Kind::Action(act),
                None => return Vec::new(),
            },
            Event::State(state, blank) => machine(*state, *blank),
            Event::Error(message) => Kind::Error {
                message: message.clone(),
            },
            Event::Mic(_) | Event::Detection(_) => return Vec::new(),
        };
        // Only the last speech before a tick or a transcript matters.
        let speech = matches!(kind, Kind::Tick | Kind::Hypothesis { .. })
            .then(|| self.pending_speech.take())
            .flatten();
        let speech = speech.map(|ts| Line::new(ts, Kind::Speech));
        speech.into_iter().chain([Line::new(t, kind)]).collect()
    }
}

/// Re-run a session log's conductor inputs (`tick`, `speech`, `hypothesis`)
/// with the `SmartConfig` from its `start` line, without a recogniser. Returns
/// the `hypothesis`, `action` and conductor `status` lines a live run prints.
pub fn replay(log: &str, songs: Vec<Song>) -> Result<Vec<Line>, String> {
    let mut lines = (log.lines().enumerate())
        .filter(|(_, l)| !l.trim().is_empty())
        .map(|(n, l)| match serde_json::from_str::<Line>(l) {
            Ok(line) if line.v == 1 => Ok(line),
            Ok(line) => Err(format!("session line {}: v {} is not 1", n + 1, line.v)),
            Err(e) => Err(format!("session line {}: {e}", n + 1)),
        });
    let start = lines.next().ok_or("the session log is empty")??;
    let info = match start.kind {
        Kind::Status {
            state,
            info: Some(info),
            ..
        } if state == "start" => info,
        _ => return Err("the session log does not begin with a start status".into()),
    };
    if info.get("setlist") != Some(&fingerprint(&songs)) {
        return Err("this session was recorded with a different setlist".into());
    }
    let config: SmartConfig =
        serde_json::from_value(info["config"].clone()).map_err(|e| format!("config: {e}"))?;
    let mut conductor = Conductor::new(songs, config);
    let base = Instant::now();
    let mut last = (MachineState::Listening, true);
    let mut out = Vec::new();
    for line in lines {
        let line = line?;
        let input = match &line.kind {
            Kind::Tick => Input::Tick,
            Kind::Speech => Input::Speech,
            Kind::Hypothesis { text } => {
                out.push(line.clone());
                Input::Heard(text.clone())
            }
            _ => continue,
        };
        let action = conductor.apply(&input, base + Duration::from_millis(line.t));
        if let Some(act) = Act::from_action(&action) {
            out.push(Line::new(line.t, Kind::Action(act)));
        }
        // The loop reports state changes after ticks only.
        let now = (conductor.state(), conductor.is_blank());
        if input == Input::Tick && now != last {
            out.push(Line::new(line.t, machine(now.0, now.1)));
            last = now;
        }
    }
    Ok(out)
}

/// A `session-YYYYMMDD-HHMM[-N].jsonl` name dated more than [`KEEP_DAYS`]
/// before `today`. Any other name is never expired.
pub fn expired(name: &str, today: NaiveDate) -> bool {
    let date = (name.strip_prefix("session-"))
        .filter(|_| name.ends_with(".jsonl"))
        .and_then(|rest| rest.get(..8))
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y%m%d").ok());
    date.is_some_and(|d| (today - d).num_days() > KEEP_DAYS)
}
