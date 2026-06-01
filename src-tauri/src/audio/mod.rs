use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, StreamConfig};
use std::sync::mpsc;
use std::sync::Arc;

/// Audio sample rate for Whisper (16kHz mono).
pub const SAMPLE_RATE: u32 = 16000;

/// Wrapper to make cpal::Stream Send+Sync (it's only used from one thread at a time).
struct SendStream(#[allow(dead_code)] cpal::Stream);
unsafe impl Send for SendStream {}
unsafe impl Sync for SendStream {}

/// Manages audio capture from the system microphone.
pub struct AudioEngine {
    device_name: Option<String>,
    stream: Option<SendStream>,
}

impl AudioEngine {
    pub fn new() -> Self {
        Self {
            device_name: None,
            stream: None,
        }
    }

    /// List available audio input devices.
    pub fn list_devices() -> Result<Vec<String>> {
        let host = cpal::default_host();
        let devices: Vec<String> = host
            .input_devices()
            .context("Failed to enumerate input devices")?
            .filter_map(|d| d.name().ok())
            .collect();
        Ok(devices)
    }

    /// Select an audio input device by name.
    pub fn select_device(&mut self, name: Option<String>) {
        self.device_name = name;
    }

    /// Start capturing audio. Returns a receiver for audio chunks.
    ///
    /// Whisper expects 16 kHz mono. Most laptop / USB mics support this
    /// directly; pro audio interfaces locked to 48 kHz and stereo-only
    /// devices won't, and surface a clear error here so the operator can
    /// pick a different device.
    pub fn start(&mut self) -> Result<mpsc::Receiver<Vec<f32>>> {
        let host = cpal::default_host();
        let device = match &self.device_name {
            Some(name) => {
                let name = name.clone();
                host.input_devices()
                    .context("Failed to enumerate input devices")?
                    .find(|d| d.name().map(|n| n == name).unwrap_or(false))
                    .context(format!("Device '{}' not found", name))?
            }
            None => host
                .default_input_device()
                .context("No default audio input device")?,
        };

        // Pick the cheapest supported config that satisfies (channels >= 1,
        // sample-rate-range includes 16k). If nothing matches, surface a
        // helpful error rather than letting cpal fail with a cryptic one.
        let supported: Vec<_> = device
            .supported_input_configs()
            .context("Failed to get supported configs")?
            .collect();
        let chosen = supported
            .iter()
            .find(|c| {
                c.channels() >= 1
                    && c.min_sample_rate().0 <= SAMPLE_RATE
                    && c.max_sample_rate().0 >= SAMPLE_RATE
            })
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "device {} does not support 16 kHz capture (configs: {})",
                    device.name().unwrap_or_else(|_| "?".into()),
                    supported
                        .iter()
                        .map(|c| format!(
                            "{}-{}Hz×{}ch",
                            c.min_sample_rate().0,
                            c.max_sample_rate().0,
                            c.channels()
                        ))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })?;
        // Open mono if available, otherwise take whatever the device offers
        // and average down to mono in the callback.
        let channels = if supported.iter().any(|c| c.channels() == 1) {
            1
        } else {
            chosen.channels()
        };
        let config = StreamConfig {
            channels,
            sample_rate: cpal::SampleRate(SAMPLE_RATE),
            buffer_size: cpal::BufferSize::Default,
        };

        let (tx, rx) = mpsc::channel::<Vec<f32>>();
        let tx = Arc::new(std::sync::Mutex::new(tx));

        let sample_format = chosen.sample_format();

        let chans = channels as usize;
        let stream = match sample_format {
            SampleFormat::F32 => {
                let tx = tx.clone();
                device.build_input_stream(
                    &config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        let samples = downmix_f32(data, chans);
                        let _ = tx.lock().unwrap().send(samples);
                    },
                    |err| log::error!("Audio stream error: {}", err),
                    None,
                )?
            }
            SampleFormat::I16 => {
                let tx = tx.clone();
                device.build_input_stream(
                    &config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        let f: Vec<f32> =
                            data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                        let samples = downmix_f32(&f, chans);
                        let _ = tx.lock().unwrap().send(samples);
                    },
                    |err| log::error!("Audio stream error: {}", err),
                    None,
                )?
            }
            format => anyhow::bail!("Unsupported sample format: {:?}", format),
        };

        stream.play().context("Failed to start audio stream")?;
        self.stream = Some(SendStream(stream));

        log::info!("Audio capture started at {}Hz mono", SAMPLE_RATE);
        Ok(rx)
    }

    /// Stop capturing audio.
    pub fn stop(&mut self) {
        self.stream = None;
        log::info!("Audio capture stopped");
    }
}

/// Average a multi-channel interleaved buffer down to mono. For mono input
/// (the common case) this is a near-zero-cost passthrough.
fn downmix_f32(data: &[f32], chans: usize) -> Vec<f32> {
    if chans <= 1 {
        return data.to_vec();
    }
    let mut out = Vec::with_capacity(data.len() / chans);
    let scale = 1.0 / chans as f32;
    for frame in data.chunks_exact(chans) {
        let sum: f32 = frame.iter().sum();
        out.push(sum * scale);
    }
    out
}
