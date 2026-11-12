use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, SupportedStreamConfig, SupportedStreamConfigRange};
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
    /// Opens the device at its default config (on Windows the shared-mode mix
    /// format, usually 44.1 or 48 kHz) and converts in the callback: shared
    /// mode refuses 16 kHz on most devices.
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
        let chosen = choose_config(default, &supported).ok_or_else(|| {
            anyhow::anyhow!(
                "device {} offers neither 32-bit float nor 16-bit capture",
                device.name().unwrap_or_else(|_| "?".into())
            )
        })?;
        let config = chosen.config();
        let (chans, rate) = (config.channels as usize, config.sample_rate.0);

        let (tx, rx) = mpsc::channel::<Vec<f32>>();
        let mut pos = 0.0;
        let stream = match chosen.sample_format() {
            SampleFormat::F32 => device.build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let mono = downmix_f32(data, chans);
                    let _ = tx.send(resample_to_16k(&mono, rate, &mut pos));
                },
                |err| log::error!("Audio stream error: {}", err),
                None,
            )?,
            SampleFormat::I16 => device.build_input_stream(
                &config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    let f: Vec<f32> = data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                    let mono = downmix_f32(&f, chans);
                    let _ = tx.send(resample_to_16k(&mono, rate, &mut pos));
                },
                |err| log::error!("Audio stream error: {}", err),
                None,
            )?,
            format => anyhow::bail!("Unsupported sample format: {:?}", format),
        };

        stream.play().context("Failed to start audio stream")?;
        self.stream = Some(SendStream(stream));

        log::info!(
            "Audio capture started at {rate} Hz x{chans}, converted to {SAMPLE_RATE} Hz mono"
        );
        Ok(rx)
    }

    /// Stop capturing audio.
    pub fn stop(&mut self) {
        self.stream = None;
        log::info!("Audio capture stopped");
    }
}

/// The device's default config if it is F32 or I16, else an F32 (then I16)
/// config at the default rate. 16 kHz is never required.
pub fn choose_config(
    default: SupportedStreamConfig,
    supported: &[SupportedStreamConfigRange],
) -> Option<SupportedStreamConfig> {
    if matches!(
        default.sample_format(),
        SampleFormat::F32 | SampleFormat::I16
    ) {
        return Some(default);
    }
    [SampleFormat::F32, SampleFormat::I16]
        .into_iter()
        .find_map(|format| {
            let range = *supported.iter().find(|c| c.sample_format() == format)?;
            Some(
                range
                    .try_with_sample_rate(default.sample_rate())
                    .unwrap_or_else(|| range.with_max_sample_rate()),
            )
        })
}

/// Linear resample of mono `input` at `from` Hz to 16 kHz. `pos` carries the
/// fractional read position from one callback to the next.
pub fn resample_to_16k(input: &[f32], from: u32, pos: &mut f64) -> Vec<f32> {
    let step = from as f64 / SAMPLE_RATE as f64;
    let mut out = Vec::with_capacity((input.len() as f64 / step) as usize + 1);
    while *pos < input.len() as f64 {
        let i = *pos as usize;
        let frac = (*pos - i as f64) as f32;
        let next = input[(i + 1).min(input.len() - 1)];
        out.push(input[i] * (1.0 - frac) + next * frac);
        *pos += step;
    }
    *pos -= input.len() as f64;
    out
}

/// Average a multi-channel interleaved buffer down to mono. For mono input
/// (the common case) this is a near-zero-cost passthrough.
pub fn downmix_f32(data: &[f32], chans: usize) -> Vec<f32> {
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
