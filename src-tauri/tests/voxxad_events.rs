//! Q3 / ADR-013: `voxxad`'s event schema v1, the session recorder, replay
//! determinism and the shared listening loop's clock. None of these needs a
//! Whisper model or an audio device, so they run in CI.

#[allow(dead_code)]
#[path = "../src/bin/voxxad/events.rs"]
mod events;

use chrono::NaiveDate;
use events::{expired, fingerprint, replay, status, Act, Kind, Line, Recorder};
use serde_json::json;
use std::process::Command;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use voxxa_lib::listen::{self, Event, Input};
use voxxa_lib::{Action, Conductor, MachineState, Setlist, SmartConfig, Song, VadEngine};

const SETLIST: &str = "tests/fixtures/voxxad/pd-hymns-setlist.json";
/// A real `voxxad --stdin-audio … --shadow` session (Whisper Tiny on
/// synthetic speech of public-domain hymn text, then a spoken prayer).
const SESSION: &str = "tests/fixtures/voxxad/session-pd-hymns.jsonl";

fn songs() -> Vec<Song> {
    let text = std::fs::read_to_string(SETLIST).unwrap();
    serde_json::from_str::<Setlist>(&text).unwrap().setlist
}

fn action_lines(jsonl: &str) -> Vec<&str> {
    (jsonl.lines())
        .filter(|l| l.contains(r#""type":"action""#))
        .collect()
}

fn amazing_grace(slide: usize) -> Act {
    Act::Goto {
        song: 0,
        slide,
        global: slide,
        title: "Amazing Grace".into(),
    }
}

#[test]
fn every_event_type_serialises_as_documented_and_round_trips() {
    let start = json!({ "mode": "file", "config": SmartConfig::default() });
    let cases = [
        (
            Line::new(1200, Kind::Action(amazing_grace(1))),
            r#"{"v":1,"t":1200,"type":"action","action":"goto","song":0,"slide":1,"global":1,"title":"Amazing Grace"}"#,
        ),
        (
            Line::new(5000, Kind::Action(Act::Blank)),
            r#"{"v":1,"t":5000,"type":"action","action":"blank"}"#,
        ),
        (
            Line::new(5200, Kind::Action(Act::Unblank)),
            r#"{"v":1,"t":5200,"type":"action","action":"unblank"}"#,
        ),
        (
            Line::new(
                900,
                Kind::Hypothesis {
                    text: "Amazing grace, how \"sweet\" the sound".into(),
                },
            ),
            r#"{"v":1,"t":900,"type":"hypothesis","text":"Amazing grace, how \"sweet\" the sound"}"#,
        ),
        (
            Line::new(
                0,
                Kind::Error {
                    message: "presenter: network error".into(),
                },
            ),
            r#"{"v":1,"t":0,"type":"error","message":"presenter: network error"}"#,
        ),
        (
            Line::new(
                400,
                Kind::Status {
                    state: "singing".into(),
                    blank: Some(false),
                    info: None,
                },
            ),
            r#"{"v":1,"t":400,"type":"status","state":"singing","blank":false}"#,
        ),
        (
            Line::new(
                30000,
                status("stop", Some(json!({ "reason": "end_of_audio" }))),
            ),
            r#"{"v":1,"t":30000,"type":"status","state":"stop","info":{"reason":"end_of_audio"}}"#,
        ),
        (
            Line::new(200, Kind::Tick),
            r#"{"v":1,"t":200,"type":"tick"}"#,
        ),
        (
            Line::new(180, Kind::Speech),
            r#"{"v":1,"t":180,"type":"speech"}"#,
        ),
    ];
    for (line, wire) in cases {
        assert_eq!(line.to_json(), wire);
        assert_eq!(serde_json::from_str::<Line>(wire).unwrap(), line, "{wire}");
    }
    // The start line carries a full SmartConfig, and it survives the trip.
    let line = Line::new(0, status("start", Some(start)));
    let back: Line = serde_json::from_str(&line.to_json()).unwrap();
    assert_eq!(back, line);
}

#[test]
fn only_tick_and_speech_stay_out_of_stdout() {
    assert!(!Line::new(0, Kind::Tick).on_stdout());
    assert!(!Line::new(0, Kind::Speech).on_stdout());
    assert!(Line::new(0, Kind::Action(Act::Blank)).on_stdout());
    assert!(Line::new(0, status("start", None)).on_stdout());
    let text = "x".to_string();
    assert!(Line::new(0, Kind::Hypothesis { text }).on_stdout());
}

#[test]
fn a_conductor_action_maps_to_the_wire_without_slide_text_and_noop_is_dropped() {
    assert_eq!(Act::from_action(&Action::Noop), None);
    assert_eq!(Act::from_action(&Action::Blank), Some(Act::Blank));
    let goto = Action::Goto {
        song_index: 2,
        slide_in_song: 1,
        global_index: 9,
        slide_text: "It is well (it is well)".into(),
        next_slide_text: None,
        song_title: "It Is Well with My Soul".into(),
    };
    let act = Act::from_action(&goto).unwrap();
    let wire = Line::new(0, Kind::Action(act)).to_json();
    assert_eq!(
        wire,
        r#"{"v":1,"t":0,"type":"action","action":"goto","song":2,"slide":1,"global":9,"title":"It Is Well with My Soul"}"#
    );
}

#[test]
fn the_recorder_keeps_the_last_speech_before_each_tick_or_transcript() {
    let base = Instant::now();
    let at = |ms| base + Duration::from_millis(ms);
    let mut r = Recorder::new(base);
    let mut out = Vec::new();
    let events = [
        (100, Event::Input(Input::Speech)),
        (150, Event::Input(Input::Speech)),
        (200, Event::Input(Input::Tick)),
        (200, Event::Action(Action::Noop, Some(13))),
        (300, Event::Input(Input::Tick)),
        (350, Event::Input(Input::Speech)),
        (400, Event::Input(Input::Heard("amazing grace".into()))),
        (400, Event::Action(Action::Blank, Some(13))),
        (400, Event::State(MachineState::BlankHold, true)),
        (450, Event::Error("whisper failed".into())),
    ];
    for (ms, event) in events {
        out.extend(r.lines(at(ms), &event));
    }
    let wire: Vec<String> = out.iter().map(Line::to_json).collect();
    assert_eq!(
        wire,
        [
            r#"{"v":1,"t":150,"type":"speech"}"#,
            r#"{"v":1,"t":200,"type":"tick"}"#,
            r#"{"v":1,"t":300,"type":"tick"}"#,
            r#"{"v":1,"t":350,"type":"speech"}"#,
            r#"{"v":1,"t":400,"type":"hypothesis","text":"amazing grace"}"#,
            r#"{"v":1,"t":400,"type":"action","action":"blank"}"#,
            r#"{"v":1,"t":400,"type":"status","state":"blank_hold","blank":true}"#,
            r#"{"v":1,"t":450,"type":"error","message":"whisper failed"}"#,
        ]
    );
    assert_eq!(r.last_t, 450);
}

#[test]
fn replay_reproduces_the_recorded_actions_byte_for_byte() {
    let log = std::fs::read_to_string(SESSION).unwrap();
    let recorded = action_lines(&log);
    assert!(recorded.len() >= 2, "the fixture must contain actions");
    let replayed: Vec<String> = (replay(&log, songs()).unwrap().iter())
        .map(Line::to_json)
        .filter(|l| l.contains(r#""type":"action""#))
        .collect();
    assert_eq!(replayed, recorded);
}

/// The live run's outputs: action lines and conductor status lines.
fn outputs(line: &&str) -> bool {
    line.contains(r#""type":"action""#) || line.contains(r#""blank":"#)
}

#[test]
fn replay_recomputes_the_actions_instead_of_copying_them() {
    let log = std::fs::read_to_string(SESSION).unwrap();
    // Leave only the inputs: no action and no conductor status line.
    let inputs: String = (log.lines())
        .filter(|l| !outputs(l))
        .map(|l| format!("{l}\n"))
        .collect();
    let replayed: Vec<String> = (replay(&inputs, songs()).unwrap().iter())
        .map(Line::to_json)
        .collect();
    let again: Vec<&str> = replayed
        .iter()
        .map(String::as_str)
        .filter(outputs)
        .collect();
    let live: Vec<&str> = log.lines().filter(outputs).collect();
    assert_eq!(again, live);
}

#[test]
fn a_changed_transcript_changes_the_replayed_actions() {
    let log = std::fs::read_to_string(SESSION).unwrap();
    // Every transcript becomes a line of prayer: no slide may be shown.
    let prayed: String = (log.lines())
        .map(|l| {
            let mut line: Line = serde_json::from_str(l).unwrap();
            if let Kind::Hypothesis { text } = &mut line.kind {
                *text = "Father we thank you for this morning and for everyone here".into();
            }
            format!("{}\n", line.to_json())
        })
        .collect();
    let replayed = replay(&prayed, songs()).unwrap();
    let shows = |l: &Line| matches!(l.kind, Kind::Action(Act::Goto { .. }));
    assert!(!replayed.iter().any(shows));
}

#[test]
fn replay_refuses_a_session_recorded_with_another_setlist() {
    let log = std::fs::read_to_string(SESSION).unwrap();
    let mut other = songs();
    other[0].slides[0].text.push_str(" (edited)");
    let err = replay(&log, other).unwrap_err();
    assert!(err.contains("different setlist"), "{err}");
    assert!(replay("", songs()).is_err());
    let no_start = r#"{"v":1,"t":200,"type":"tick"}"#;
    assert!(replay(no_start, songs()).unwrap_err().contains("start"));
}

#[test]
fn replay_names_the_line_of_a_cut_off_or_wrong_version_line() {
    let log = std::fs::read_to_string(SESSION).unwrap();
    let lines: Vec<&str> = log.lines().collect();
    let n = lines.len();
    // A crash can leave a partial last line.
    let cut = log.trim_end();
    let err = replay(&cut[..cut.len() - 5], songs()).unwrap_err();
    assert!(err.starts_with(&format!("session line {n}: ")), "{err}");
    // A future v2 log is refused, not misread.
    let mut v2 = lines.clone();
    let third = lines[2].replacen(r#""v":1"#, r#""v":2"#, 1);
    v2[2] = &third;
    let err = replay(&v2.join("\n"), songs()).unwrap_err();
    assert_eq!(err, "session line 3: v 2 is not 1");
}

#[test]
fn the_fingerprint_names_the_setlist_and_changes_with_any_text() {
    let fp = fingerprint(&songs());
    assert_eq!(fp["songs"], 3);
    assert_eq!(fp["slides"], 13);
    let mut other = songs();
    other[2].title.push('!');
    assert_ne!(fingerprint(&other), fp);
}

#[test]
fn session_logs_expire_after_fourteen_days_and_nothing_else_is_touched() {
    let today = NaiveDate::from_ymd_opt(2026, 11, 30).unwrap();
    assert!(
        !expired("session-20261116-0930.jsonl", today),
        "14 days: kept"
    );
    assert!(
        expired("session-20261115-0930.jsonl", today),
        "15 days: deleted"
    );
    assert!(expired("session-20261101-1000-2.jsonl", today));
    assert!(!expired("session-20261130-1000.jsonl", today));
    for other in [
        "session-20261101-1000.txt",
        "voxxa.log",
        "session-notadate.jsonl",
        "Session-20261101-1000.jsonl",
        "my-session-20261101-1000.jsonl",
    ] {
        assert!(!expired(other, today), "{other}");
    }
}

/// The shared loop at file speed: time is audio time, whatever the speed.
#[test]
fn the_shared_loop_ticks_every_200_ms_of_audio_on_a_sample_clock() {
    let (tx, rx) = mpsc::channel();
    for _ in 0..10 {
        tx.send(vec![0.0f32; 1600]).unwrap(); // 1 s of silence in 100 ms chunks
    }
    drop(tx);
    let base = Instant::now();
    let clock = |samples: u64| base + Duration::from_millis(samples / 16);
    let conductor = Mutex::new(Some(Conductor::new(songs(), SmartConfig::default())));
    let vad = Mutex::new(VadEngine::new());
    let mut ticks = Vec::new();
    let mut other = 0;
    let sink = |now: Instant, event: Event| match event {
        Event::Input(Input::Tick) => ticks.push(now.duration_since(base).as_millis()),
        Event::Action(Action::Noop, Some(13)) | Event::Mic(_) => {}
        _ => other += 1,
    };
    let no_speech = |_: &[f32]| -> anyhow::Result<String> { panic!("silence was transcribed") };
    let run = AtomicBool::new(true);
    let ended = listen::run(rx, &run, &vad, &conductor, no_speech, clock, sink);
    assert!(
        ended,
        "the end of the audio is reported as the channel closing"
    );
    assert_eq!(ticks, [200, 400, 600, 800, 1000]);
    assert_eq!(other, 0, "silence starts nothing");

    // A stopped session returns at once and reports a stop, not a loss.
    let (_tx, rx) = mpsc::channel();
    let stopped = AtomicBool::new(false);
    let ended = listen::run(rx, &stopped, &vad, &conductor, no_speech, clock, |_, _| {});
    assert!(!ended);
}

fn voxxad(args: &[&str]) -> (String, bool) {
    let out = Command::new(env!("CARGO_BIN_EXE_voxxad"))
        .args(args)
        .output()
        .unwrap();
    (String::from_utf8(out.stdout).unwrap(), out.status.success())
}

#[test]
fn the_voxxad_binary_replays_the_session_with_identical_action_lines() {
    let (stdout, ok) = voxxad(&["--replay", SESSION, "--setlist", SETLIST]);
    assert!(ok, "{stdout}");
    let log = std::fs::read_to_string(SESSION).unwrap();
    assert_eq!(action_lines(&stdout), action_lines(&log));
    // Every stdout line is one JSON object with v and t.
    for line in stdout.lines() {
        let v: serde_json::Value = serde_json::from_str(line).unwrap();
        assert_eq!(v["v"], 1, "{line}");
        assert!(v["t"].is_u64(), "{line}");
    }
}

#[test]
fn voxxad_reports_bad_input_as_one_error_line() {
    let dir = tempfile::tempdir().unwrap();
    // A mono 16-bit WAV with two samples at `rate`.
    let wav = |rate: u32| {
        let mut bytes = b"RIFF\0\0\0\0WAVEfmt ".to_vec();
        bytes.extend(16u32.to_le_bytes());
        bytes.extend([1, 0, 1, 0]);
        bytes.extend(rate.to_le_bytes());
        bytes.extend((rate * 2).to_le_bytes());
        bytes.extend([2, 0, 16, 0]);
        bytes.extend(b"data");
        bytes.extend(4u32.to_le_bytes());
        bytes.extend([0, 0, 0, 0]);
        let path = dir.path().join(format!("{rate}.wav"));
        std::fs::write(&path, bytes).unwrap();
        path.to_str().unwrap().to_string()
    };
    let (k8, k16) = (wav(8000), wav(16000));
    // Every case fails before a model is loaded or audio is opened.
    let cases: [(&[&str], &str); 5] = [
        (&["--stdin-audio", &k8, "--setlist", SETLIST], "16 kHz mono"),
        (&["--stdin-audio", &k16], "--setlist is required"),
        (&["--setlist", SETLIST], "give one of"),
        (&["--frobnicate"], "unknown argument"),
        (
            &[
                "--stdin-audio",
                &k16,
                "--setlist",
                SETLIST,
                "--presenter",
                "x",
            ],
            "unknown presenter",
        ),
    ];
    for (args, want) in cases {
        let (stdout, ok) = voxxad(args);
        assert!(!ok, "{args:?}");
        let line: Line = serde_json::from_str(stdout.trim()).unwrap();
        match line.kind {
            Kind::Error { message } => assert!(message.contains(want), "{message}"),
            other => panic!("{other:?}"),
        }
    }
}
