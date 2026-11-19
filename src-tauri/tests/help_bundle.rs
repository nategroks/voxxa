//! Q9 / ADR-018: Call for help. No recognised word reaches the diagnostics
//! ring buffer, and so none reaches `generate_diagnostic_report` (#102); the
//! words file, the zip and the e-mail draft. Placeholder words only.

use std::fs;
use std::io::Read;
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

fn line(t: u64, kind: &str, text: &str) -> String {
    serde_json::json!({ "v": 1, "t": t, "type": kind, "text": text }).to_string() + "\n"
}

#[test]
fn words_file_is_the_newest_logs_last_two_minutes_of_hypotheses() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(voxxa_lib::help::recent_words(dir.path()), None, "no log");
    let old = dir.path().join("session-20261118-1000.jsonl");
    fs::write(&old, line(500_000, "hypothesis", "golf hotel")).unwrap();
    let past = std::time::SystemTime::now() - Duration::from_secs(3600);
    fs::File::options()
        .write(true)
        .open(&old)
        .unwrap()
        .set_modified(past)
        .unwrap();
    let log = [
        line(0, "status", "start"),
        line(10_000, "hypothesis", "alpha bravo"),
        line(130_000, "window", "india juliet"),
        line(130_000, "hypothesis", "charlie delta"),
        line(200_000, "hypothesis", "echo foxtrot"),
        "not json\n".into(),
        line(250_000, "tick", ""),
    ];
    let new = dir.path().join("session-20261119-1000.jsonl");
    fs::write(&new, log.concat()).unwrap();
    // The end is t = 250,000, so 130,000 is in and 10,000 is out; window lines never.
    let words = voxxa_lib::help::recent_words(dir.path());
    assert_eq!(words.as_deref(), Some("charlie delta\necho foxtrot"));
    fs::write(&new, line(0, "status", "start") + &line(9, "tick", "")).unwrap();
    assert_eq!(voxxa_lib::help::recent_words(dir.path()), None, "no words");
}

fn names(zip: &std::path::Path) -> Vec<(String, String)> {
    let mut zip = zip::ZipArchive::new(fs::File::open(zip).unwrap()).unwrap();
    (0..zip.len())
        .map(|i| {
            let mut f = zip.by_index(i).unwrap();
            let mut text = String::new();
            f.read_to_string(&mut text).unwrap();
            (f.name().unwrap().to_string(), text)
        })
        .collect()
}

#[test]
fn the_zip_holds_exactly_the_reviewed_files() {
    let dir = tempfile::tempdir().unwrap();
    let files = vec![
        ("report.json".to_string(), "{}".to_string()),
        (
            "settings.json".to_string(),
            r#"{"voxxa.language":"en"}"#.to_string(),
        ),
    ];
    let path = voxxa_lib::help::write_bundle(dir.path(), &files).unwrap();
    let name = path.file_name().unwrap().to_string_lossy();
    assert!(
        name.starts_with("voxxa-help-") && name.ends_with(".zip"),
        "{name}"
    );
    assert_eq!(
        names(&path),
        files,
        "the box unchecked: no words file at all"
    );
    let with = [
        files.clone(),
        vec![("words.txt".into(), "alpha bravo".into())],
    ]
    .concat();
    fs::remove_file(&path).unwrap();
    let path = voxxa_lib::help::write_bundle(dir.path(), &with).unwrap();
    assert_eq!(names(&path), with);
}

#[test]
fn bundle_names_from_the_webview_are_plain_file_names() {
    let dir = tempfile::tempdir().unwrap();
    for bad in [
        "../x.txt", "a/b.txt", "a\\b.txt", "c:x.txt", "..", ".hidden", "",
    ] {
        let files = [(bad.to_string(), "x".to_string())];
        assert!(
            voxxa_lib::help::write_bundle(dir.path(), &files).is_err(),
            "{bad:?}"
        );
    }
    assert_eq!(
        fs::read_dir(dir.path()).unwrap().count(),
        0,
        "nothing written"
    );
}

#[test]
fn the_draft_has_no_recipient_and_names_the_file_and_its_folder() {
    assert_eq!(
        voxxa_lib::help::SUPPORT_EMAIL,
        "",
        "Q9 ticket: no support address yet"
    );
    let zip = std::env::temp_dir().join("voxxa-help-20261119-120000.zip");
    let url = voxxa_lib::help::mailto(&zip);
    assert!(
        url.starts_with("mailto:?subject=Voxxa%3A%20call%20for%20help&body="),
        "{url}"
    );
    assert!(url.contains("voxxa-help-20261119-120000.zip"), "{url}");
    assert!(!url.contains(' ') && !url.contains('\n'), "{url}");
    let folder = zip.parent().unwrap().file_name().unwrap().to_string_lossy();
    assert!(
        url.contains(&format!("{folder}%20%28that%20folder")),
        "{url}"
    );
}
