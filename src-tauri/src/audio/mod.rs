use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, StreamConfig};
use cpal::{SupportedStreamConfig, SupportedStreamConfigRange};
use std::sync::mpsc;

/// Audio sample rate for Whisper (16kHz mono).
pub const SAMPLE_RATE: u32 = 16000;

/// Wrapper to make cpal::Stream Send+Sync (it's only used from one thread at a time).
struct SendStream(#[allow(dead_code)] cpal::Stream);
unsafe impl Send for SendStream {}
unsafe impl Sync for SendStream {}

/// Manages audio capture from the system microphone.
#[derive(Default)]
pub struct AudioEngine {
    device_name: Option<String>,
    stream: Option<SendStream>,
}

impl AudioEngine {
    pub fn new() -> Self {
        Self::default()
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

    /// Start capturing audio. Returns a receiver of 16 kHz mono chunks.
    ///
    /// Whisper expects 16 kHz mono, but Windows shared-mode devices only
    /// capture at their mix rate (usually 44.1 or 48 kHz). So the device is
    /// opened at its default config and downmixed and resampled here.
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

        let default = device
            .default_input_config()
            .context("Failed to get the default input config")?;
        let supported: Vec<_> = device
            .supported_input_configs()
            .context("Failed to get supported configs")?
            .collect();
        let (channels, rate, sample_format) =
            choose_config(&default, &supported).ok_or_else(|| {
                anyhow::anyhow!(
                    "device {} offers no 32-bit float or 16-bit capture format",
                    device.name().unwrap_or_else(|_| "?".into())
                )
            })?;
        let config = StreamConfig {
            channels,
            sample_rate: cpal::SampleRate(rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let (tx, rx) = mpsc::channel::<Vec<f32>>();
        let chans = channels as usize;
        let mut resampler = Resampler::new(rate);
        let on_error = |err| log::error!("Audio stream error: {}", err);
        let stream = match sample_format {
            SampleFormat::F32 => device.build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let _ = tx.send(resampler.process(&downmix_f32(data, chans)));
                },
                on_error,
                None,
            )?,
            _ => device.build_input_stream(
                &config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    let f: Vec<f32> = data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                    let _ = tx.send(resampler.process(&downmix_f32(&f, chans)));
                },
                on_error,
                None,
            )?,
        };

        stream.play().context("Failed to start audio stream")?;
        self.stream = Some(SendStream(stream));

        log::info!(
            "Audio capture started at {rate} Hz x{channels}, resampled to {SAMPLE_RATE} Hz mono"
        );
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

/// The capture format: the device default when it is 32-bit float or 16-bit,
/// else a float, then a 16-bit config near the default rate. Never U8 (the
/// callbacks handle only F32 and I16) and never a required 16 kHz.
pub fn choose_config(
    default: &SupportedStreamConfig,
    supported: &[SupportedStreamConfigRange],
) -> Option<(u16, u32, SampleFormat)> {
    let rate = default.sample_rate().0;
    if matches!(
        default.sample_format(),
        SampleFormat::F32 | SampleFormat::I16
    ) {
        return Some((default.channels(), rate, default.sample_format()));
    }
    let span = |c: &SupportedStreamConfigRange| (c.min_sample_rate().0, c.max_sample_rate().0);
    [SampleFormat::F32, SampleFormat::I16]
        .iter()
        .find_map(|&fmt| {
            supported
                .iter()
                .filter(|c| c.sample_format() == fmt)
                .min_by_key(|c| !(span(c).0..=span(c).1).contains(&rate))
                .map(|c| (c.channels(), rate.clamp(span(c).0, span(c).1), fmt))
        })
}

/// Linear resampler from the device rate to 16 kHz mono. It keeps its
/// position across callbacks, so chunk edges do not jitter the pitch.
pub struct Resampler {
    /// Input samples per output sample.
    step: f64,
    /// Next read position in the current chunk; -1 is the previous chunk's last sample.
    pos: f64,
    last: f32,
}

impl Resampler {
    pub fn new(from_rate: u32) -> Self {
        Self {
            step: from_rate as f64 / SAMPLE_RATE as f64,
            pos: 0.0,
            last: 0.0,
        }
    }

    pub fn process(&mut self, input: &[f32]) -> Vec<f32> {
        if self.step == 1.0 || input.is_empty() {
            return input.to_vec();
        }
        let at = |i: isize| if i < 0 { self.last } else { input[i as usize] };
        let mut out = Vec::with_capacity((input.len() as f64 / self.step) as usize + 1);
        while self.pos < (input.len() - 1) as f64 {
            let i = self.pos.floor();
            let frac = (self.pos - i) as f32;
            out.push(at(i as isize) * (1.0 - frac) + at(i as isize + 1) * frac);
            self.pos += self.step;
        }
        self.pos -= input.len() as f64;
        self.last = input[input.len() - 1];
        out
    }
}
