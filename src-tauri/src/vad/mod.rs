//! Voice Activity Detection.
//!
//! Primary backend: Silero VAD via the `voice_activity_detector` crate, which
//! bundles the Silero ONNX model and links ONNX Runtime at build time. We feed
//! it fixed 512-sample windows at 16 kHz (the documented v5 numerics from §4.1
//! of the plan) and apply hysteresis (`vad_speech_enter_prob: 0.60`,
//! `vad_speech_exit_prob: 0.35`) to convert per-frame probabilities into a
//! stable speech / silence signal.
//!
//! Fallback backend: simple RMS-energy threshold. Used only when Silero fails
//! to initialise (rare — typically only on platforms ORT doesn't support).

use anyhow::Result;
use voice_activity_detector::VoiceActivityDetector;

/// Plan §4.4 defaults.
const ENTER_THRESHOLD: f32 = 0.60;
const EXIT_THRESHOLD: f32 = 0.35;
const ENERGY_THRESHOLD: f32 = 0.005; // RMS — used only by the fallback path

const WINDOW_SIZE: usize = 512; // 32 ms at 16 kHz (Silero v5 expectation)
const SAMPLE_RATE: u32 = 16_000;

/// Minimum continuous speech in milliseconds before declaring an utterance has
/// started. Suppresses single-frame false positives.
const MIN_SPEECH_MS: u64 = 250;
/// Trailing silence in milliseconds that ends an utterance.
const MIN_SILENCE_MS: u64 = 300;

enum Backend {
    Silero(Box<VoiceActivityDetector>),
    Energy,
}

pub struct VadEngine {
    backend: Backend,
    /// Hysteresis state — `true` once a frame crosses the enter threshold and we
    /// haven't crossed the exit threshold since.
    is_speech: bool,
    /// Audio leftover from the previous `process()` call that didn't fill a
    /// full window. Carried forward so we never lose samples on chunk seams.
    pending: Vec<f32>,
    /// Samples of contiguous speech accumulated for the current utterance.
    speech_buffer: Vec<f32>,
    /// Samples of contiguous silence since the last speech frame.
    silence_samples: usize,
    /// Samples of contiguous speech (independent of `is_speech` latch).
    speech_samples: usize,
}

impl VadEngine {
    pub fn new() -> Self {
        let backend = match VoiceActivityDetector::builder()
            .sample_rate(SAMPLE_RATE)
            .chunk_size(WINDOW_SIZE)
            .build()
        {
            Ok(v) => {
                log::info!("[VAD] Silero backend ready");
                Backend::Silero(Box::new(v))
            }
            Err(e) => {
                log::warn!(
                    "[VAD] Silero init failed ({e}); falling back to RMS energy"
                );
                Backend::Energy
            }
        };
        Self {
            backend,
            is_speech: false,
            pending: Vec::with_capacity(WINDOW_SIZE * 2),
            speech_buffer: Vec::new(),
            silence_samples: 0,
            speech_samples: 0,
        }
    }

    /// Compatibility shim — older code paths called this before any signal
    /// flowed through. Silero needs no separate init step.
    pub fn initialize(&mut self, _model_path: &str) -> Result<()> {
        Ok(())
    }

    /// Whether the VAD currently believes the user is speaking.
    pub fn is_in_speech(&self) -> bool {
        self.is_speech
    }

    /// Feed a chunk of f32 mono 16 kHz audio. Returns `Some(samples)` whenever
    /// a complete utterance (speech followed by sufficient silence) finishes,
    /// or `None` while we're still accumulating.
    pub fn process(&mut self, samples: &[f32]) -> Option<Vec<f32>> {
        self.pending.extend_from_slice(samples);

        let mut completed: Option<Vec<f32>> = None;

        while self.pending.len() >= WINDOW_SIZE {
            // Drain a single window. Using drain avoids the per-chunk allocation
            // that `clone_from_slice` would force.
            let window: Vec<f32> = self.pending.drain(..WINDOW_SIZE).collect();
            let frame_is_speech = self.classify_frame(&window);

            if frame_is_speech {
                self.speech_samples += window.len();
                self.silence_samples = 0;
                self.speech_buffer.extend_from_slice(&window);
            } else {
                self.silence_samples += window.len();
                if self.is_speech {
                    // Keep the trailing silence in the buffer so Whisper hears
                    // a natural pause after the last word.
                    self.speech_buffer.extend_from_slice(&window);

                    let min_silence =
                        (MIN_SILENCE_MS as usize * SAMPLE_RATE as usize) / 1000;
                    if self.silence_samples >= min_silence {
                        completed = Some(std::mem::take(&mut self.speech_buffer));
                        self.is_speech = false;
                        self.speech_samples = 0;
                    }
                }
            }
        }

        completed
    }

    /// Run the backend on a 512-sample window and update the hysteresis state.
    /// Returns whether the new latched `is_speech` is true.
    fn classify_frame(&mut self, window: &[f32]) -> bool {
        debug_assert_eq!(window.len(), WINDOW_SIZE);

        let raw_above = match &mut self.backend {
            Backend::Silero(v) => {
                let prob: f32 = v.predict(window.iter().copied());
                // Schmitt-trigger hysteresis: once latched into speech, we stay
                // until we cross the lower threshold, so isolated dips between
                // syllables don't pop us out.
                if self.is_speech {
                    prob >= EXIT_THRESHOLD
                } else {
                    prob > ENTER_THRESHOLD
                }
            }
            Backend::Energy => {
                let energy = rms_energy(window);
                energy > ENERGY_THRESHOLD
            }
        };

        // Promote to is_speech only once we've held a positive read for the
        // minimum-speech duration. Prevents single-frame noise blips from
        // tripping the conductor.
        if raw_above {
            let min_speech =
                (MIN_SPEECH_MS as usize * SAMPLE_RATE as usize) / 1000;
            if !self.is_speech && self.speech_samples + WINDOW_SIZE >= min_speech {
                self.is_speech = true;
            }
        } else if !raw_above && self.is_speech {
            // Don't flip is_speech off here — that's `process()`'s job once
            // `silence_samples` crosses `MIN_SILENCE_MS`. This lets us keep
            // accumulating trailing silence into the utterance buffer.
        }
        raw_above
    }

    pub fn reset(&mut self) {
        self.is_speech = false;
        self.pending.clear();
        self.speech_buffer.clear();
        self.silence_samples = 0;
        self.speech_samples = 0;
        if let Backend::Silero(v) = &mut self.backend {
            v.reset();
        }
    }
}

fn rms_energy(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum: f32 = samples.iter().map(|s| s * s).sum();
    (sum / samples.len() as f32).sqrt()
}
