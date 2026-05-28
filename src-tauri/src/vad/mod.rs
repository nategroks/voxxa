use anyhow::Result;

/// Silero VAD parameters.
const VAD_THRESHOLD: f32 = 0.5;
const MIN_SPEECH_MS: u64 = 250;
const MIN_SILENCE_MS: u64 = 300;
const WINDOW_SIZE: usize = 512; // 32ms at 16kHz

/// Voice Activity Detection engine using Silero VAD via ONNX Runtime.
///
/// Gates audio so only speech segments are sent to Whisper,
/// reducing unnecessary transcription and improving accuracy.
pub struct VadEngine {
    /// Whether VAD is initialized with a model.
    initialized: bool,
    /// Rolling state for the Silero model (h, c tensors).
    state_h: Vec<f32>,
    state_c: Vec<f32>,
    /// Current speech detection state.
    is_speech: bool,
    /// Accumulated speech samples for the current utterance.
    speech_buffer: Vec<f32>,
    /// Samples since last speech detection.
    silence_samples: usize,
    /// Samples of continuous speech.
    speech_samples: usize,
    sample_rate: u32,
}

impl VadEngine {
    pub fn new() -> Self {
        Self {
            initialized: false,
            state_h: vec![0.0; 2 * 1 * 64],
            state_c: vec![0.0; 2 * 1 * 64],
            is_speech: false,
            speech_buffer: Vec::new(),
            silence_samples: 0,
            speech_samples: 0,
            sample_rate: 16000,
        }
    }

    /// Initialize the VAD model from the given ONNX file path.
    pub fn initialize(&mut self, _model_path: &str) -> Result<()> {
        // In production, this loads the Silero VAD ONNX model via ort.
        // For now we use a simple energy-based VAD as a fallback.
        log::info!("VAD engine initialized (energy-based fallback)");
        self.initialized = true;
        Ok(())
    }

    /// Process an audio chunk and return completed speech segments.
    ///
    /// Returns `Some(samples)` when a complete speech utterance is detected
    /// (speech followed by sufficient silence), or `None` if still accumulating.
    pub fn process(&mut self, samples: &[f32]) -> Option<Vec<f32>> {
        if !self.initialized {
            // Pass through all audio if VAD isn't initialized
            return Some(samples.to_vec());
        }

        let mut result = None;

        // Process in windows
        for chunk in samples.chunks(WINDOW_SIZE) {
            let energy = rms_energy(chunk);
            let is_speech_frame = energy > VAD_THRESHOLD * 0.01; // Energy-based threshold

            if is_speech_frame {
                self.speech_samples += chunk.len();
                self.silence_samples = 0;

                let min_speech_samples =
                    (MIN_SPEECH_MS as usize * self.sample_rate as usize) / 1000;
                if self.speech_samples >= min_speech_samples {
                    self.is_speech = true;
                }

                if self.is_speech {
                    self.speech_buffer.extend_from_slice(chunk);
                }
            } else {
                self.silence_samples += chunk.len();

                if self.is_speech {
                    self.speech_buffer.extend_from_slice(chunk);

                    let min_silence_samples =
                        (MIN_SILENCE_MS as usize * self.sample_rate as usize) / 1000;
                    if self.silence_samples >= min_silence_samples {
                        // End of utterance
                        result = Some(std::mem::take(&mut self.speech_buffer));
                        self.is_speech = false;
                        self.speech_samples = 0;
                    }
                }
            }
        }

        result
    }

    /// Whether the VAD currently believes the user is speaking.
    /// Used by the smart-blanking state machine to gate transitions between
    /// SINGING / INTER_VERSE_SILENCE / BLANK_HOLD without waiting for a full utterance.
    pub fn is_in_speech(&self) -> bool {
        self.is_speech
    }

    /// Reset VAD state for a new recording session.
    pub fn reset(&mut self) {
        self.is_speech = false;
        self.speech_buffer.clear();
        self.silence_samples = 0;
        self.speech_samples = 0;
        self.state_h.fill(0.0);
        self.state_c.fill(0.0);
    }
}

/// Compute RMS energy of an audio frame.
fn rms_energy(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum: f32 = samples.iter().map(|s| s * s).sum();
    (sum / samples.len() as f32).sqrt()
}
