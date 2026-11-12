//! The listening loop: audio → VAD → recogniser → conductor (ADR-013,
//! ADR-014). One function for the GUI (`commands::start_listening_with_state`)
//! and for `voxxad`; each passes its own clock, recogniser and event sink.

pub use crate::aligner::{check_setlist, Input};
use crate::aligner::{Action, Conductor, MachineState};
use crate::transcription::{Hypothesis, Recognizer, HOP_MS};
use crate::vad::VadEngine;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Clone)]
pub struct MicLevel {
    /// Peak amplitude in this window, 0.0–1.0. Clipped to 1.0 if the mic is hot.
    pub peak: f32,
    /// RMS energy in the same window — what the VAD effectively sees.
    pub rms: f32,
}

/// One row of the conductor's per-song confidence table, denormalised with
/// the song title so the frontend can render without a second lookup.
#[derive(Debug, Serialize, Clone)]
pub struct DetectionRow {
    pub song_index: usize,
    pub song_title: String,
    pub probability: f64,
    pub raw_score: f64,
}

/// What the loop reports, in order, each with the clock reading it happened at.
pub enum Event {
    /// A conductor input, reported just before the conductor gets it.
    Input(Input),
    /// The conductor's answer to that input, with the setlist's slide count.
    Action(Action, Option<usize>),
    /// The conductor's state after a tick, when it changed.
    State(MachineState, bool),
    Mic(MicLevel),
    /// Top song candidates after a transcript.
    Detection(Vec<DetectionRow>),
    /// The recogniser failed on one hop; the loop goes on.
    Error(String),
}

const TICK_EVERY: Duration = Duration::from_millis(200);
// ~33 Hz UI updates — fast enough to look live, cheap enough to ignore.
const MIC_EMIT_EVERY: Duration = Duration::from_millis(33);
const HOP: usize = HOP_MS * 16;

/// Run until `run` clears (a stop: returns false) or the audio channel closes
/// while `run` is still set (device loss or end of file: returns true).
/// `clock` maps the number of 16 kHz samples received so far to the time the
/// conductor sees; `recognizer` is fed every hop of audio, muted when the
/// VAD heard no speech in it, and each hypothesis goes to the conductor.
pub fn run(
    rx: Receiver<Vec<f32>>,
    run: &AtomicBool,
    vad: &Mutex<VadEngine>,
    conductor: &Mutex<Option<Conductor>>,
    mut recognizer: impl Recognizer,
    mut clock: impl FnMut(u64) -> Instant,
    mut sink: impl FnMut(Instant, Event),
) -> bool {
    let (mut hop, mut heard, mut due) = (Vec::new(), false, HOP);
    let (mut last_state, mut last_blank) = (MachineState::Listening, true);
    let mut received: u64 = 0;
    let (mut last_tick, mut last_mic_emit) = (clock(0), clock(0));
    let (mut peak_window, mut sumsq_window, mut samples_window) = (0.0f32, 0.0f64, 0usize);
    // Apply one input under the conductor lock; `extra` reads more state
    // under the same lock (the detection rows).
    let apply = |input: Input, now, extra: &mut dyn FnMut(&Conductor)| {
        let mut cl = conductor.blocking_lock();
        cl.as_mut().map_or((Action::Noop, None), |c| {
            let action = c.apply(&input, now);
            extra(c);
            (action, Some(c.total_slides()))
        })
    };

    while run.load(Ordering::SeqCst) {
        let now = clock(received);

        // Periodic tick — drives time-based transitions (silence → blank).
        if now.duration_since(last_tick) >= TICK_EVERY {
            last_tick = now;
            sink(now, Event::Input(Input::Tick));
            let (action, total) = apply(Input::Tick, now, &mut |_| {});
            sink(now, Event::Action(action, total));
            let snap = conductor
                .blocking_lock()
                .as_ref()
                .map(|c| (c.state(), c.is_blank()));
            if let Some((s, b)) = snap.filter(|&snap| snap != (last_state, last_blank)) {
                sink(now, Event::State(s, b));
                (last_state, last_blank) = (s, b);
            }
        }

        let samples = match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(samples) => samples,
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => {
                // Stop also closes the channel; only a loss while this
                // session should still run is a device loss.
                let disconnected = run.load(Ordering::SeqCst);
                log::warn!("[AUDIO] capture channel closed (device lost: {disconnected})");
                return disconnected;
            }
        };
        received += samples.len() as u64;
        // Mic level: peak + sum-of-squares across the emission window, so
        // operators can verify mic signal at a glance.
        for &s in &samples {
            peak_window = peak_window.max(s.abs());
            sumsq_window += (s as f64) * (s as f64);
        }
        samples_window += samples.len();
        if now.duration_since(last_mic_emit) >= MIC_EMIT_EVERY {
            // An empty window has a zero sum, so its RMS is 0.
            let rms = (sumsq_window / samples_window.max(1) as f64).sqrt() as f32;
            let peak = peak_window.min(1.0);
            sink(now, Event::Mic(MicLevel { peak, rms }));
            (peak_window, sumsq_window, samples_window) = (0.0, 0.0, 0);
            last_mic_emit = now;
        }

        {
            let mut v = vad.blocking_lock();
            v.process(&samples);
            if v.is_in_speech() {
                heard = true;
                sink(now, Event::Input(Input::Speech));
                apply(Input::Speech, now, &mut |_| {});
            }
        }
        hop.extend_from_slice(&samples);
        if hop.len() < due {
            continue;
        }
        if !heard {
            hop.fill(0.0);
        }
        let now = clock(received);
        let result = recognizer.feed(&hop);
        // A recogniser slower than real time (GUI, --realtime) waits as long
        // for its next hop as this one took, so at most one hop's audio queues
        // up. At file speed the clock is audio time and every hop is HOP_MS.
        let took = clock(received).saturating_duration_since(now);
        due = HOP.max(took.as_millis() as usize * 16);
        (hop, heard) = (Vec::new(), false);
        let hypotheses = match result {
            Ok(hypotheses) => hypotheses,
            Err(e) => {
                log::error!("Transcription error: {}", e);
                sink(now, Event::Error(e.to_string()));
                continue;
            }
        };
        for text in hypotheses.iter().map(Hypothesis::text) {
            log::info!("Heard: {}", text);
            sink(now, Event::Input(Input::Heard(text.clone())));
            let mut rows = Vec::new();
            let (action, total) = apply(Input::Heard(text), now, &mut |c| {
                let titles = c.song_titles();
                // Top 5 candidates is enough — beyond that the UI bars get noisy.
                rows = c
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
            });
            if !rows.is_empty() {
                sink(now, Event::Detection(rows));
            }
            if let Action::Goto {
                slide_text,
                next_slide_text,
                ..
            } = &action
            {
                let next = next_slide_text.as_deref().unwrap_or_default();
                recognizer.bias(&format!("{slide_text} {next}"));
            }
            sink(now, Event::Action(action, total));
        }
    }
    false
}
