//! Q9 / ADR-018: Call for help. No recognised word reaches the diagnostics
//! ring buffer, and so none reaches `generate_diagnostic_report` (#102).
//! Placeholder words only.

use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use voxxa_lib::listen;
use voxxa_lib::transcription::{Hypothesis, Recognizer};
use voxxa_lib::{Conductor, Setlist, SmartConfig, VadEngine};

const HEARD: &str = "kilo lima mike november oscar papa";

/// Hears `HEARD` once, on the first chunk, whatever the gate says.
struct Once(bool);

impl Recognizer for Once {
    fn feed(&mut self, _: &[f32]) -> Vec<Hypothesis> {
        if std::mem::replace(&mut self.0, true) {
            return Vec::new();
        }
        let words = (HEARD.split_whitespace())
            .map(|w| (w.to_string(), 0.0, 0.9))
            .collect();
        vec![Hypothesis {
            words,
            end: 1.0,
            error: None,
        }]
    }
    fn bias(&mut self, _: &str) {}
}

#[test]
fn heard_words_never_reach_the_diagnostics_log() {
    voxxa_lib::init_logger();
    let setlist: Setlist = serde_json::from_value(serde_json::json!({ "setlist": [
        { "title": "Placeholder Hymn", "slides": [{ "id": 0, "text": HEARD }, { "id": 1, "text": "quebec romeo sierra tango" }] }
    ] }))
    .unwrap();
    let conductor = Mutex::new(Some(Conductor::new(
        setlist.setlist,
        SmartConfig::default(),
    )));
    let (tx, rx) = mpsc::channel();
    for _ in 0..20 {
        tx.send(vec![0.0f32; 1600]).unwrap();
    }
    drop(tx);
    let base = Instant::now();
    let clock = |samples: u64| base + Duration::from_millis(samples / 16);
    let mut heard = 0;
    let sink = |_, e| heard += matches!(e, listen::Event::Input(listen::Input::Heard(_))) as usize;
    let vad = Mutex::new(VadEngine::new());
    listen::run(
        rx,
        &AtomicBool::new(true),
        &vad,
        &conductor,
        &mut Once(false),
        clock,
        sink,
    );

    assert_eq!(heard, 1, "the loop must have heard the words");
    let log = voxxa_lib::recent_log();
    assert!(log.iter().any(|l| l.contains("Heard 6 words")), "{log:?}");
    for word in HEARD.split_whitespace() {
        assert!(!log.iter().any(|l| l.contains(word)), "{word:?} in the log");
    }
}
