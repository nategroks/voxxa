//! Q4 / ADR-004, ADR-014: Whisper behind the `Recognizer` trait, over a 5 s
//! window fed in 1 s hops. The window arithmetic is tested with a fake
//! Whisper (CI has no model); one test runs the real Whisper Tiny when it is
//! downloaded and skips otherwise.

use std::cell::RefCell;
use voxxa_lib::{Hypothesis, Recognizer, Rolling, TranscriptionEngine, WhisperModel, Word};

const HOP: usize = 16_000;

/// What the fake Whisper was asked: the window length and the prompt.
type Calls = RefCell<Vec<(usize, Option<String>)>>;

/// A fake Whisper that hears two words, half a second into its window and
/// half a second before its end.
fn fake(calls: &Calls) -> impl FnMut(&[f32], Option<&str>) -> anyhow::Result<Vec<Word>> + '_ {
    |pcm, prompt| {
        calls
            .borrow_mut()
            .push((pcm.len(), prompt.map(String::from)));
        let secs = pcm.len() as f32 / 16_000.0;
        Ok(vec![
            ("amazing".into(), 0.5, 0.9),
            ("grace".into(), secs - 0.5, 0.8),
        ])
    }
}

fn starts(h: &[Hypothesis]) -> Vec<f32> {
    h.iter().flat_map(|h| h.words.iter().map(|w| w.1)).collect()
}

#[test]
fn whisper_reads_the_last_five_seconds_and_reports_stream_time() {
    let calls = Calls::default();
    let mut whisper = Rolling::new(fake(&calls));
    for k in 1..=7u32 {
        let hypotheses = whisper.feed(&[0.1; HOP]).unwrap();
        assert_eq!(hypotheses.len(), 1);
        assert_eq!(hypotheses[0].text(), "amazing grace");
        // Window times are moved to stream time: the window is the last
        // min(k, 5) seconds of the stream.
        let start = k.saturating_sub(5) as f32;
        assert_eq!(
            starts(&hypotheses),
            [start + 0.5, k as f32 - 0.5],
            "hop {k}"
        );
    }
    let windows: Vec<usize> = calls.borrow().iter().map(|c| c.0 / HOP).collect();
    assert_eq!(windows, [1, 2, 3, 4, 5, 5, 5]);
}

#[test]
fn a_muted_hop_is_not_transcribed_but_its_time_still_passes() {
    let calls = Calls::default();
    let mut whisper = Rolling::new(fake(&calls));
    assert!(whisper.feed(&[0.0; HOP]).unwrap().is_empty());
    assert!(whisper.feed(&[0.0; HOP]).unwrap().is_empty());
    assert!(calls.borrow().is_empty(), "silence was transcribed");
    // The third second is heard in a 3 s window that starts at 0.
    let h = whisper.feed(&[0.1; HOP]).unwrap();
    assert_eq!(starts(&h), [0.5, 2.5]);
}

#[test]
fn a_window_without_words_gives_no_hypothesis_and_errors_pass_through() {
    let mut nothing = Rolling::new(|_: &[f32], _: Option<&str>| Ok(Vec::new()));
    assert!(nothing.feed(&[0.1; HOP]).unwrap().is_empty());
    let mut failing =
        Rolling::new(|_: &[f32], _: Option<&str>| Err(anyhow::anyhow!("whisper failed")));
    let e = failing.feed(&[0.1; HOP]).unwrap_err();
    assert_eq!(e.to_string(), "whisper failed");
}

#[test]
fn bias_reaches_whisper_as_initial_prompt_only_when_enabled() {
    let calls = Calls::default();
    let mut whisper = Rolling::new(fake(&calls));
    whisper.bias("through many dangers toils and snares");
    whisper.feed(&[0.1; HOP]).unwrap();
    whisper.bias_enabled = true;
    whisper.feed(&[0.1; HOP]).unwrap();
    let prompts: Vec<_> = calls.borrow().iter().map(|c| c.1.clone()).collect();
    let bias = Some("through many dangers toils and snares".to_string());
    assert_eq!(prompts, [None, bias]);
}

/// Public-domain text (Amazing Grace, verse 1) read by the Windows SAPI
/// voice, 16 kHz mono 16-bit PCM; the first 10 s are used.
const SPEECH_WAV: &str = "tests/fixtures/voxxad/amazing-grace-v1.wav";

fn ten_seconds_of_speech() -> Vec<f32> {
    let bytes = std::fs::read(SPEECH_WAV).unwrap();
    // The fixture is 16 kHz mono 16-bit PCM; find its data chunk.
    let mut at = 12;
    while &bytes[at..at + 4] != b"data" {
        at += 8 + u32::from_le_bytes(bytes[at + 4..at + 8].try_into().unwrap()) as usize;
    }
    (bytes[at + 8..].as_chunks::<2>().0.iter())
        .map(|&b| f32::from(i16::from_le_bytes(b)) / 32768.0)
        .take(10 * HOP)
        .collect()
}

/// The real Whisper Tiny, fed 10 s of speech in 1 s hops: every hypothesis
/// carries stream times, in order, inside the window it was heard in.
/// Skips (passes) when the model is not downloaded, as in CI.
#[test]
fn whisper_tiny_on_ten_seconds_of_speech_gives_increasing_stream_times() {
    let mut engine = TranscriptionEngine::new();
    if !engine.is_model_downloaded(&WhisperModel::Tiny) {
        eprintln!("skipped: ggml-tiny.bin is not downloaded");
        return;
    }
    engine.load_model(&WhisperModel::Tiny).unwrap();
    let mut whisper = Rolling::new(|pcm: &[f32], prompt: Option<&str>| engine.words(pcm, prompt));
    let audio = ten_seconds_of_speech();
    assert_eq!(audio.len(), 10 * HOP);
    let mut heard = Vec::new();
    for (k, hop) in audio.chunks(HOP).enumerate() {
        let started = std::time::Instant::now();
        for h in whisper.feed(hop).unwrap() {
            let end = (k + 1) as f32;
            let window = (end - 5.0).max(0.0)..=end;
            let times: Vec<f32> = h.words.iter().map(|w| w.1).collect();
            eprintln!(
                "hop {} ({} ms): {times:?} {}",
                k + 1,
                started.elapsed().as_millis(),
                h.text()
            );
            assert!(times.windows(2).all(|w| w[0] <= w[1]), "{times:?}");
            assert!(times.iter().all(|t| window.contains(t)), "{times:?}");
            heard.push((end, times, h.text().to_lowercase()));
        }
    }
    // Speech fills the clip, so nearly every hop is heard ...
    assert!(heard.len() >= 8, "{heard:?}");
    // ... and the stream times of the words move forward with the hops.
    let firsts: Vec<f32> = heard.iter().map(|h| h.1[0]).collect();
    assert!(firsts.windows(2).all(|w| w[0] <= w[1] + 0.5), "{firsts:?}");
    assert!(heard.iter().any(|h| h.2.contains("grace")), "{heard:?}");
}
