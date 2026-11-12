//! Q2-10 (audit 8): listening must work on devices that capture only at their
//! mix rate (44.1 / 48 kHz on Windows shared mode). No audio device is opened.

use cpal::SupportedStreamConfigRange as Range;
use cpal::{SampleFormat, SampleRate, SupportedBufferSize, SupportedStreamConfig};
use voxxa_lib::{choose_config, Resampler};

/// 1 s of a 440 Hz sine at `rate`, as interleaved stereo, then downmixed the
/// way the capture callback does (average of the two channels).
fn sine_mono(rate: u32) -> Vec<f32> {
    let stereo: Vec<f32> = (0..rate)
        .flat_map(|i| {
            let v = (2.0 * std::f32::consts::PI * 440.0 * i as f32 / rate as f32).sin();
            [v, v]
        })
        .collect();
    let (frames, _) = stereo.as_chunks::<2>();
    frames.iter().map(|[l, r]| (l + r) / 2.0).collect()
}

/// Frequency from zero crossings over 1 s.
fn frequency(samples: &[f32]) -> f32 {
    let crossings = samples
        .windows(2)
        .filter(|w| (w[0] < 0.0) != (w[1] < 0.0))
        .count();
    crossings as f32 / 2.0
}

/// Resample in device-sized chunks, as the callback does.
fn resample(rate: u32, chunk: usize) -> Vec<f32> {
    let mut r = Resampler::new(rate);
    sine_mono(rate)
        .chunks(chunk)
        .flat_map(|c| r.process(c))
        .collect()
}

#[test]
fn device_rates_resample_to_16k_and_keep_the_pitch() {
    let out = resample(48_000, 480);
    assert_eq!(out.len(), 16_000);
    assert!(
        (frequency(&out) - 440.0).abs() <= 8.8,
        "{}",
        frequency(&out)
    );

    let out = resample(44_100, 441);
    assert!((15_999..=16_001).contains(&out.len()), "{}", out.len());
    assert!(
        (frequency(&out) - 440.0).abs() <= 8.8,
        "{}",
        frequency(&out)
    );

    assert_eq!(resample(16_000, 160).len(), 16_000);
}

#[test]
fn config_chooser_prefers_float_then_16_bit_and_never_needs_16k() {
    let def =
        |fmt| SupportedStreamConfig::new(2, SampleRate(48_000), SupportedBufferSize::Unknown, fmt);
    let range = |fmt| {
        Range::new(
            2,
            SampleRate(48_000),
            SampleRate(48_000),
            SupportedBufferSize::Unknown,
            fmt,
        )
    };
    // The Windows mix format: float at 48 kHz.
    let all = [
        range(SampleFormat::U8),
        range(SampleFormat::I16),
        range(SampleFormat::F32),
    ];
    assert_eq!(
        choose_config(&def(SampleFormat::F32), &all),
        Some((2, 48_000, SampleFormat::F32))
    );
    // A U8 default is never used; float wins over 16-bit.
    assert_eq!(
        choose_config(&def(SampleFormat::U8), &all),
        Some((2, 48_000, SampleFormat::F32))
    );
    assert_eq!(
        choose_config(&def(SampleFormat::U8), &all[..2]),
        Some((2, 48_000, SampleFormat::I16))
    );
    assert_eq!(choose_config(&def(SampleFormat::U8), &all[..1]), None);
}
