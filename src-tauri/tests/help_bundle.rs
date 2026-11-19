//! Q9 / ADR-018: Call for help. #102 (no recognised text reaches the
//! diagnostics buffer), the bundle's files, the words window, secret
//! redaction, the zip, the e-mail draft and the runbook links. No window,
//! microphone, network or Whisper model; placeholder words only.

use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use voxxa_lib::listen;
use voxxa_lib::transcription::{Hypothesis, Recognizer};
use voxxa_lib::{Conductor, SmartConfig, Song, VadEngine};

/// Hears four placeholder words on its first hop, then nothing.
struct Hears(Option<Vec<&'static str>>);

impl Recognizer for Hears {
    fn feed(&mut self, _: &[f32]) -> Vec<Hypothesis> {
        let Some(words) = self.0.take() else {
            return Vec::new();
        };
        let words = (words.iter().enumerate())
            .map(|(i, w)| (w.to_string(), i as f32 * 0.3, 0.9))
            .collect();
        vec![Hypothesis {
            words,
            end: 1.0,
            error: None,
        }]
    }
    fn bias(&mut self, _: &str) {}
}

/// #102 (Q7-08): the listening loop logs how many words it heard, never the
/// words, so neither the ring buffer nor `generate_diagnostic_report`'s
/// `recent_log` (the same buffer) carries a transcript.
#[test]
fn no_recognised_text_reaches_the_diagnostics_buffer() {
    voxxa_lib::init_logger();
    let words = vec!["zulu", "yankee", "xray", "whiskey"];
    let song: Song = serde_json::from_value(serde_json::json!({
        "title": "Alpha Hymn",
        "slides": [{ "id": 0, "text": "alpha bravo charlie delta" }]
    }))
    .unwrap();
    let conductor = Mutex::new(Some(Conductor::new(vec![song], SmartConfig::default())));
    let vad = Mutex::new(VadEngine::new());
    let (tx, rx) = mpsc::channel();
    tx.send(vec![0.0f32; 1600]).unwrap();
    drop(tx);
    let base = Instant::now();
    let clock = |n: u64| base + Duration::from_millis(n / 16);
    let mut heard = Vec::new();
    let sink = |_, e| {
        if let listen::Event::Input(listen::Input::Heard(t)) = e {
            heard.push(t)
        }
    };
    let run = AtomicBool::new(true);
    listen::run(
        rx,
        &run,
        &vad,
        &conductor,
        &mut Hears(Some(words.clone())),
        clock,
        sink,
    );
    assert_eq!(heard, ["zulu yankee xray whiskey"], "the loop heard them");
    let log = voxxa_lib::recent_log().join("\n");
    assert!(log.contains("Heard 4 words"), "{log}");
    for w in words {
        assert!(!log.contains(w), "the buffer holds a heard word");
    }
}
