//! Q4 / ADR-014: the `Recognizer` trait and Whisper's rolling window. The
//! window, hop, gate, bias flag and stream timestamps are tested with a fake
//! decoder, so they run in CI. The test that runs Whisper Tiny on the 10 s
//! fixture skips when the model is not downloaded (CI has none) and is run
//! locally.

use anyhow::anyhow;
use std::time::Instant;
use voxxa_lib::transcription::recognizer::words_from_tokens;
use voxxa_lib::transcription::{Decode, Hypothesis, Recognizer, Rolling, Stitcher, Word};
use voxxa_lib::{TranscriptionEngine, WhisperModel};

/// Public-domain text (Amazing Grace, verse 2) read by the Windows SAPI
/// voice, 16 kHz mono 16-bit PCM, 10.3 s (the Q2 `make-wav.ps1` `ag-v2.wav`).
const TEN_S_WAV: &str = "tests/fixtures/voxxad/amazing-grace-v2.wav";

fn wav_16k_mono(path: &str) -> Vec<f32> {
    let bytes = std::fs::read(path).unwrap();
    let data = (12..bytes.len())
        .find(|&i| &bytes[i..i + 4] == b"data")
        .unwrap();
    // PCM, one channel, 16000 Hz, 16 bits.
    assert_eq!(&bytes[20..28], &[1, 0, 1, 0, 0x80, 0x3e, 0, 0]);
    (bytes[data + 8..].as_chunks::<2>().0.iter())
        .map(|&b| f32::from(i16::from_le_bytes(b)) / 32768.0)
        .collect()
}

/// Remembers every window and prompt it is given and hears one word per
/// second of window, at 0.25 s, 1.25 s, ... from the window's start.
#[derive(Default)]
struct Fake {
    windows: Vec<usize>,
    prompts: Vec<Option<String>>,
    fail: Vec<usize>,
}

impl Decode for Fake {
    fn decode(&mut self, window: &[f32], prompt: Option<&str>) -> anyhow::Result<Vec<Word>> {
        self.windows.push(window.len());
        self.prompts.push(prompt.map(String::from));
        if self.fail.contains(&(self.windows.len() - 1)) {
            return Err(anyhow!("whisper failed"));
        }
        let secs = window.len() / 16000;
        Ok((0..secs)
            .map(|i| (format!("w{i}"), i as f32 + 0.25, 0.9))
            .collect())
    }
}

/// Feed `secs` of silence in 100 ms chunks.
fn feed_secs(r: &mut Rolling<Fake>, secs: usize) -> Vec<Hypothesis> {
    (0..secs * 10).flat_map(|_| r.feed(&[0.0; 1600])).collect()
}

#[test]
fn rolling_decodes_once_a_second_over_the_last_five_seconds() {
    let mut r = Rolling::new(Fake::default());
    let hyps = feed_secs(&mut r, 8);
    let ends: Vec<f32> = hyps.iter().map(|h| h.end).collect();
    assert_eq!(ends, [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
    // The window grows to 5 s, then slides.
    assert_eq!(r.fed(), 8 * 16000);
    let windows = &r.decoder().windows;
    assert_eq!(windows[..], [1, 2, 3, 4, 5, 5, 5, 5].map(|s| s * 16000));
}

#[test]
fn word_times_are_stream_time_not_window_time() {
    let mut r = Rolling::new(Fake::default());
    let hyps = feed_secs(&mut r, 8);
    // At 8 s the window is 3..8 s: the fake's word at 0.25 s is at 3.25 s.
    let last = hyps.last().unwrap();
    let starts: Vec<f32> = last.words.iter().map(|w| w.1).collect();
    assert_eq!(starts, [3.25, 4.25, 5.25, 6.25, 7.25]);
    assert_eq!(last.text(), "w0 w1 w2 w3 w4");
    for h in &hyps {
        assert!(h.words.windows(2).all(|w| w[0].1 <= w[1].1), "{h:?}");
        assert!(h.words.iter().all(|w| w.1 >= h.end - 5.0 && w.1 <= h.end));
    }
}

#[test]
fn a_closed_gate_skips_hops_but_keeps_stream_time() {
    let mut r = Rolling::new(Fake::default());
    assert_eq!(feed_secs(&mut r, 2).len(), 2);
    r.gate(false);
    assert!(
        feed_secs(&mut r, 3).is_empty(),
        "nothing decoded while closed"
    );
    r.gate(true);
    let hyps = feed_secs(&mut r, 1);
    // The next hop decodes the newest 5 s, timed in stream seconds.
    assert_eq!(hyps.len(), 1);
    assert_eq!(hyps[0].end, 6.0);
    assert_eq!(hyps[0].words[0].1, 1.25);
}

#[test]
fn several_hops_in_one_feed_decode_once_on_the_newest_window() {
    let mut r = Rolling::new(Fake::default());
    // 3.5 s at once (a loop that fell behind and drained its queue).
    let hyps = r.feed(&vec![0.0; 56000]);
    assert_eq!(hyps.len(), 1);
    assert_eq!(hyps[0].end, 3.5);
    // The next boundary is 4 s, not 1 s.
    assert!(r.feed(&[0.0; 4000]).is_empty());
    assert_eq!(r.feed(&[0.0; 4000])[0].end, 4.0);
}

#[test]
fn bias_reaches_the_engine_only_when_enabled() {
    let mut r = Rolling::new(Fake::default());
    r.bias("Amazing grace, how sweet the sound\nThat saved a wretch like me");
    feed_secs(&mut r, 1);
    r.bias_enabled = true;
    feed_secs(&mut r, 1);
    r.bias("   ");
    feed_secs(&mut r, 1);
    let biased = "Amazing grace, how sweet the sound That saved a wretch like me";
    let prompts = &r.decoder().prompts;
    assert_eq!(prompts[..], [None, Some(biased.to_string()), None]);
}

#[test]
fn a_failed_decode_is_one_hypothesis_with_an_error_and_an_empty_one_is_none() {
    let fake = Fake {
        fail: vec![1],
        ..Fake::default()
    };
    let mut r = Rolling::new(fake);
    let hyps = feed_secs(&mut r, 3);
    let errors: Vec<_> = hyps.iter().map(|h| h.error.as_deref()).collect();
    assert_eq!(errors, [None, Some("whisper failed"), None]);
    assert!(hyps[1].words.is_empty());
    // A window the fake hears nothing in gives no hypothesis at all.
    let mut r = Rolling::new(Fake::default());
    r.hop_ms = 500;
    let hyps = feed_secs(&mut r, 1);
    assert_eq!(r.decoder().windows, [8000, 16000]);
    assert_eq!(hyps.iter().map(|h| h.end).collect::<Vec<_>>(), [1.0]);
}

/// `text`'s words, one every `step` seconds from `from`.
fn timed(text: &str, from: f32, step: f32) -> Vec<Word> {
    (text.split_whitespace().enumerate())
        .map(|(i, w)| (w.to_string(), from + i as f32 * step, 0.9))
        .collect()
}

#[test]
fn the_stitcher_hands_on_only_the_words_a_window_adds() {
    let mut s = Stitcher::default();
    // The first window is all new.
    let first = timed("Amazing grace, how sweet the sound that", 0.5, 0.4);
    assert_eq!(
        s.next(&first).as_deref(),
        Some("Amazing grace, how sweet the sound that")
    );
    // One second later: the window lost two words at its start and gained
    // three at its end; Whisper heard the cut-off "grace" as "race" and the
    // case and commas differ. Three new words are under the conductor's 4,
    // so they are held ...
    let mut second = timed("race how sweet the Sound, that saved a wretch", 0.9, 0.4);
    second[0].1 = 0.95;
    assert_eq!(s.next(&second), None);
    // ... until the next hop brings a fourth.
    let third = timed("the sound that saved a wretch like", 2.1, 0.4);
    assert_eq!(s.next(&third).as_deref(), Some("saved a wretch like"));
    // A window with nothing new gives nothing.
    let fourth = timed("sound that saved a wretch like", 2.5, 0.4);
    assert_eq!(s.next(&fourth), None);
    // After silence the stitcher forgets: a repeat is new again.
    s.clear();
    assert_eq!(
        s.next(&fourth).as_deref(),
        Some("sound that saved a wretch like")
    );
    // Held words do not outlive a silence.
    s.clear();
    assert_eq!(s.next(&timed("me", 9.0, 0.4)), None);
    s.clear();
    assert_eq!(s.next(&timed("I once was", 10.0, 0.4)), None);
}

#[test]
fn a_repeated_phrase_lines_up_by_word_time() {
    // "it is well, it is well": the new window's start lines up with either
    // "it is well" equally well by the words alone; times decide.
    let mut s = Stitcher::default();
    let sent = timed("with my soul it is well it is well", 10.0, 0.5);
    s.next(&sent).unwrap();
    // Starts at the second "it" (13 s) and adds "with my soul" and more.
    let new = timed("it is well with my soul it", 13.0, 0.5);
    assert_eq!(s.next(&new).as_deref(), Some("with my soul it"));
}

#[test]
fn word_text_comes_from_the_segment_and_times_from_space_led_tokens() {
    let tok = |text: &str, t: f32, p: f32| (text.to_string(), t, p);
    // U+FFFD is what a token cut inside a UTF-8 letter decodes to on its
    // own; the segment's text has the whole letter.
    let tokens = [
        tok(" Thou", 0.0, 0.9),
        tok(" art", 0.5, 0.8),
        tok(",", 0.8, 0.6),
        tok(" na", 1.0, 0.6),
        tok("\u{fffd}", 1.2, 0.4),
        tok("\u{fffd}ve", 1.3, 0.5),
        tok(" ", 1.5, 0.1),
        tok(" Lord", 1.6, 0.7),
    ];
    let words = words_from_tokens(" Thou art, na\u{ef}ve Lord", &tokens);
    let want: Vec<Word> = vec![
        ("Thou".into(), 0.0, 0.9),
        ("art,".into(), 0.5, 0.7),
        ("na\u{ef}ve".into(), 1.0, 0.5),
        ("Lord".into(), 1.6, 0.7),
    ];
    assert_eq!(words.len(), want.len(), "{words:?}");
    for (got, want) in words.iter().zip(&want) {
        assert_eq!((&got.0, got.1), (&want.0, want.1), "{words:?}");
        assert!((got.2 - want.2).abs() < 1e-6, "{words:?}");
    }
    // More words than space-led tokens: the rest take the last known start.
    let words = words_from_tokens("my soul", &[tok(" my", 2.0, 0.8)]);
    assert_eq!(words, [("my".into(), 2.0, 0.8), ("soul".into(), 2.0, 0.8)]);
    assert!(words_from_tokens("", &[]).is_empty());
}

/// The Q4 exit test: Whisper Tiny on the 10 s fixture, fed in 100 ms chunks
/// with the gate open. Hypotheses come every second, their ends rise by one
/// hop, and every word's start is stream time: inside its window and in order.
#[test]
fn whisper_tiny_on_a_10_s_wav_gives_monotonically_increasing_timestamps() {
    let mut engine = TranscriptionEngine::new();
    if !engine.is_model_downloaded(&WhisperModel::Tiny) {
        eprintln!("skipped: ggml-tiny.bin is not in {:?}", engine.model_dir());
        return;
    }
    engine.load_model(&WhisperModel::Tiny).unwrap();
    let audio = wav_16k_mono(TEN_S_WAV);
    assert!((160_000..170_000).contains(&audio.len()), "{}", audio.len());
    let mut r = Rolling::new(engine);
    let mut hyps = Vec::new();
    for chunk in audio.chunks(1600) {
        let started = Instant::now();
        let heard = r.feed(chunk);
        if let Some(h) = heard.first() {
            let ms = started.elapsed().as_millis();
            eprintln!("hop end {:>4.1} s  {ms:>5} ms  {}", h.end, h.text());
        }
        hyps.extend(heard);
    }
    assert!(hyps.iter().all(|h| h.error.is_none()), "{hyps:?}");
    assert!(
        hyps.len() >= 8,
        "{} hypotheses in 10 s of speech",
        hyps.len()
    );
    assert!(hyps.windows(2).all(|p| p[1].end > p[0].end), "{hyps:?}");
    for h in &hyps {
        let starts: Vec<f32> = h.words.iter().map(|w| w.1).collect();
        assert!(starts.windows(2).all(|p| p[0] <= p[1]), "{h:?}");
        assert!(
            starts.iter().all(|&t| t >= h.end - 5.0 && t <= h.end),
            "{h:?}"
        );
    }
    // Stream time, not window time: as the window slides, its first word
    // starts later (in window time it would always be near 0). Whisper
    // times the same word a few 10 ms ticks apart in two windows, so a
    // step back of up to 50 ms is jitter, not a reset.
    let first: Vec<f32> = hyps.iter().map(|h| h.words[0].1).collect();
    eprintln!("first word starts: {first:?}");
    assert!(first.windows(2).all(|p| p[1] >= p[0] - 0.05), "{first:?}");
    assert!(first[first.len() - 1] - first[0] >= 3.0, "{first:?}");
    let late = hyps.iter().filter(|h| h.end > 6.0).collect::<Vec<_>>();
    assert!(late.iter().all(|h| h.words.iter().all(|w| w.1 >= 1.0)));
    let all: String = hyps
        .iter()
        .map(Hypothesis::text)
        .collect::<Vec<_>>()
        .join(" ");
    assert!(all.to_lowercase().contains("grace"), "{all}");
}
