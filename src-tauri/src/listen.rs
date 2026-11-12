//! The listening loop: audio → VAD → Whisper → conductor. One loop for the
//! GUI (`commands::start_listening_with_state`) and `voxxad`. It reports
//! through `on`; the caller turns reports into Tauri events or JSON lines
//! and dispatches actions. `tick` and `transcript` are every conductor input
//! the loop makes, so `voxxad --replay` reproduces a session by calling them.

use crate::aligner::{Action, Conductor, MachineState};
use crate::transcription::TranscriptionEngine;
use crate::vad::VadEngine;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// One row of the conductor's per-song confidence table, denormalised with
/// the song title so the frontend can render without a second lookup.
#[derive(Debug, Serialize, Clone)]
pub struct DetectionRow {
    pub song_index: usize,
    pub song_title: String,
    pub probability: f64,
    pub raw_score: f64,
}

/// What the loop reports, each with the `now` the conductor saw.
pub enum Heard<'a> {
    /// A 16 kHz mono chunk as captured.
    Samples(&'a [f32]),
    /// The conductor ticked, first told of speech last heard at this time.
    Tick(Option<Instant>),
    /// Whisper's text, before the conductor sees it.
    Text(&'a str),
    /// Top song candidates after a transcript (never empty).
    Scores(Vec<DetectionRow>),
    /// Send this to the presenter; total slides when a setlist is loaded.
    Action(Action, Option<usize>),
    /// The machine state or blank changed.
    State(MachineState, bool),
}

const TICK_EVERY: Duration = Duration::from_millis(200);

/// Run until `run` clears (false) or the audio channel closes while `run` is
/// set (true: device lost, or the end of a file). `clock(samples fed)` is now.
pub fn listen(
    rx: Receiver<Vec<f32>>,
    run: &AtomicBool,
    clock: impl Fn(u64) -> Instant,
    vad: &Mutex<VadEngine>,
    transcription: &Mutex<TranscriptionEngine>,
    conductor: &Mutex<Option<Conductor>>,
    mut on: impl FnMut(Instant, Heard),
) -> bool {
    let mut accumulated: Vec<f32> = Vec::new();
    let mut shown = (MachineState::Listening, true);
    let mut fed = 0u64;
    let mut last_tick = clock(0);
    // Only `tick` reads the last speech time, so it is handed over there.
    let mut speech = None;
    while run.load(Ordering::SeqCst) {
        let now = clock(fed);
        // Periodic tick — drives time-based transitions (silence → blank).
        if now.duration_since(last_tick) >= TICK_EVERY {
            last_tick = now;
            tick(conductor, speech.take(), now, &mut shown, &mut on);
        }
        let samples = match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(samples) => samples,
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => {
                // Stop also closes the channel; only a loss while this
                // session should still run is a device loss.
                let lost = run.load(Ordering::SeqCst);
                log::warn!("[AUDIO] capture channel closed (device lost: {lost})");
                return lost;
            }
        };
        fed += samples.len() as u64;
        on(now, Heard::Samples(&samples));
        let segment = {
            let mut v = vad.blocking_lock();
            let seg = v.process(&samples);
            if v.is_in_speech() {
                speech = Some(now);
            }
            seg
        };
        let Some(segment) = segment else { continue };
        accumulated.extend_from_slice(&segment);
        if accumulated.len() < 16000 * 5 {
            continue;
        }
        let result = transcription.blocking_lock().transcribe(&accumulated);
        accumulated.clear();
        match result {
            Ok(r) if !r.text.is_empty() => {
                log::info!("Heard: {}", r.text);
                transcript(conductor, &r.text, now, &mut on);
            }
            Ok(_) => {}
            Err(e) => log::error!("Transcription error: {}", e),
        }
    }
    false
}

/// Tick the conductor; `shown` is the state last reported.
pub fn tick(
    conductor: &Mutex<Option<Conductor>>,
    speech: Option<Instant>,
    now: Instant,
    shown: &mut (MachineState, bool),
    on: &mut impl FnMut(Instant, Heard),
) {
    on(now, Heard::Tick(speech));
    let (action, total) = match conductor.blocking_lock().as_mut() {
        Some(c) => {
            if let Some(at) = speech {
                c.on_speech(at);
            }
            (c.tick(now), Some(c.total_slides()))
        }
        None => (Action::Noop, None),
    };
    on(now, Heard::Action(action, total));
    let snap = conductor
        .blocking_lock()
        .as_ref()
        .map(|c| (c.state(), c.is_blank()));
    if let Some(s) = snap.filter(|s| s != shown) {
        *shown = s;
        on(now, Heard::State(s.0, s.1));
    }
}

/// Give the conductor one transcript.
pub fn transcript(
    conductor: &Mutex<Option<Conductor>>,
    text: &str,
    now: Instant,
    on: &mut impl FnMut(Instant, Heard),
) {
    on(now, Heard::Text(text));
    let (action, total, rows) = match conductor.blocking_lock().as_mut() {
        Some(c) => {
            let act = c.on_transcript(text, now);
            let titles = c.song_titles();
            // Top 5 candidates is enough — beyond that the UI bars get noisy.
            let rows: Vec<DetectionRow> = c
                .last_scores()
                .iter()
                .take(5)
                .map(|s| DetectionRow {
                    song_index: s.song_index,
                    song_title: titles.get(s.song_index).cloned().unwrap_or_default(),
                    probability: s.probability,
                    raw_score: s.raw_score,
                })
                .collect();
            (act, Some(c.total_slides()), rows)
        }
        None => (Action::Noop, None, Vec::new()),
    };
    if !rows.is_empty() {
        on(now, Heard::Scores(rows));
    }
    on(now, Heard::Action(action, total));
}
