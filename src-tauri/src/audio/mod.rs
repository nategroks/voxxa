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

        let config = StreamConfig {
            channels: 1,
            sample_rate: cpal::SampleRate(SAMPLE_RATE),
            buffer_size: cpal::BufferSize::Default,
        };

        let (tx, rx) = mpsc::channel::<Vec<f32>>();
        let tx = Arc::new(std::sync::Mutex::new(tx));

        let supported = device
            .supported_input_configs()
            .context("Failed to get supported configs")?;

        let sample_format = supported
            .into_iter()
            .next()
            .map(|c| c.sample_format())
            .unwrap_or(SampleFormat::F32);

        let stream = match sample_format {
            SampleFormat::F32 => {
                let tx = tx.clone();
                device.build_input_stream(
                    &config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        let _ = tx.lock().unwrap().send(data.to_vec());
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
                        let samples: Vec<f32> =
                            data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
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
