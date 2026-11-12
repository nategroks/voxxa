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

/// The GUI's Stop clears the session's run flag and then stops capture,
/// which closes the channel. A loop that is waiting for audio when that
/// happens sees the channel close with the flag clear: a stop, reported as
/// listening-stopped {disconnected: false}, not a device loss.
#[test]
fn a_stop_that_closes_the_channel_mid_loop_is_not_a_device_loss() {
    let (tx, rx) = mpsc::channel();
    for _ in 0..10 {
        tx.send(vec![0.0f32; 1600]).unwrap();
    }
    drop(tx);
    let run = AtomicBool::new(true);
    let base = Instant::now();
    let mut heard = 0;
    // The operator presses Stop once the last chunk has been taken in.
    let clock = |samples: u64| {
        heard = samples;
        if samples == 16000 {
            run.store(false, std::sync::atomic::Ordering::SeqCst);
        }
        base + Duration::from_millis(samples / 16)
    };
    let conductor = Mutex::new(Some(Conductor::new(songs(), SmartConfig::default())));
    let vad = Mutex::new(VadEngine::new());
    let no_speech = |_: &[f32]| -> anyhow::Result<String> { panic!("silence was transcribed") };
    let lost = listen::run(rx, &run, &vad, &conductor, no_speech, clock, |_, _| {});
    assert_eq!(heard, 16000, "the stop came after the audio, mid-loop");
    assert!(!lost, "a stop was reported as a device loss");
}

/// Public-domain text (Amazing Grace, verse 1) read by the Windows SAPI
/// voice, 16 kHz mono 16-bit PCM, 12 s. Real speech for the real VAD; the
/// recogniser is faked, so no Whisper model is needed.
const SPEECH_WAV: &str = "tests/fixtures/voxxad/amazing-grace-v1.wav";

fn wav_16k_mono(path: &str) -> Vec<f32> {
    let bytes = std::fs::read(path).unwrap();
    assert_eq!((&bytes[..4], &bytes[8..12]), (&b"RIFF"[..], &b"WAVE"[..]));
    let (mut at, mut fmt, mut data) = (12, None, None);
    while at + 8 <= bytes.len() {
        let len = u32::from_le_bytes(bytes[at + 4..at + 8].try_into().unwrap()) as usize;
        let body = &bytes[at + 8..(at + 8 + len).min(bytes.len())];
        match &bytes[at..at + 4] {
            b"fmt " => fmt = Some(body),
            b"data" => data = Some(body),
            _ => {}
        }
        at += 8 + len + (len & 1);
    }
    let fmt = fmt.unwrap();
    // PCM, one channel, 16000 Hz, 16 bits.
    assert_eq!(&fmt[..8], &[1, 0, 1, 0, 0x80, 0x3e, 0, 0]);
    assert_eq!(fmt[14], 16);
    (data.unwrap().as_chunks::<2>().0.iter())
        .map(|&b| f32::from(i16::from_le_bytes(b)) / 32768.0)
        .collect()
}

/// 1 s of silence, the verse, 2 s of silence, the verse again, 12 s of
/// silence: long enough for inter-verse silence and the blank after it.
fn service() -> Vec<f32> {
    let verse = wav_16k_mono(SPEECH_WAV);
    let quiet = |secs: usize| vec![0.0f32; 16000 * secs];
    [quiet(1), verse.clone(), quiet(2), verse, quiet(12)].concat()
}

/// What one `listen::run` over some audio produced.
struct Heard {
    /// Every event but mic levels, in order, with its audio time in ms.
    events: Vec<(u64, Event)>,
    /// The session log `voxxad` would write for this run.
    log: String,
    /// The length of each slice handed to the recogniser.
    slices: Vec<usize>,
    /// Every slice handed to the recogniser, one after the other.
    fed: Vec<f32>,
    lost: bool,
}

/// Run the shared loop at file speed over `audio`; the fake recogniser
/// answers with `texts` in turn, then with nothing. `operator` sees each
/// event before it is recorded and may act on the conductor, as the GUI's
/// Blank button does.
fn listen_to(
    audio: &[f32],
    texts: &[String],
    operator: impl FnMut(&Mutex<Option<Conductor>>, &Event),
) -> Heard {
    let mut texts = texts.iter();
    let recognise = |_| Ok(texts.next().cloned().unwrap_or_default());
    listen_with(songs(), audio, recognise, operator)
}

/// `listen_to` with any setlist and any recogniser, which is told the
/// number (0, 1, ...) of the slice it is given.
fn listen_with(
    setlist: Vec<Song>,
    audio: &[f32],
    mut recognise: impl FnMut(usize) -> anyhow::Result<String>,
    mut operator: impl FnMut(&Mutex<Option<Conductor>>, &Event),
) -> Heard {
    let (tx, rx) = mpsc::channel();
    for chunk in audio.chunks(1600) {
        tx.send(chunk.to_vec()).unwrap();
    }
    drop(tx);
    let base = Instant::now();
    let clock = |samples: u64| base + Duration::from_millis(samples / 16);
    let start = json!({ "config": SmartConfig::default(), "setlist": fingerprint(&setlist) });
    let conductor = Mutex::new(Some(Conductor::new(setlist, SmartConfig::default())));
    let vad = Mutex::new(VadEngine::new());
    let mut log = format!("{}\n", Line::new(0, status("start", Some(start))).to_json());
    let (mut recorder, mut events, mut slices) = (Recorder::new(base), Vec::new(), Vec::new());
    let mut fed = Vec::new();
    let transcribe = |pcm: &[f32]| -> anyhow::Result<String> {
        slices.push(pcm.len());
        fed.extend_from_slice(pcm);
        recognise(slices.len() - 1)
    };
    let sink = |now: Instant, event: Event| {
        if matches!(event, Event::Mic(_)) {
            return;
        }
        operator(&conductor, &event);
        for line in recorder.lines(now, &event) {
            log.push_str(&line.to_json());
            log.push('\n');
        }
        events.push((now.duration_since(base).as_millis() as u64, event));
    };
    let run = AtomicBool::new(true);
    let lost = listen::run(rx, &run, &vad, &conductor, transcribe, clock, sink);
    Heard {
        events,
        log,
        slices,
        fed,
        lost,
    }
}

/// The lines of Amazing Grace, slide by slide: what the fake recogniser hears.
fn verses() -> Vec<String> {
    songs()[0].slides.iter().map(|s| s.text.clone()).collect()
}

/// The audio time of the first transcript: the start of the first chunk
/// after which the VAD's finished utterances add up to 5 s (80000
/// samples), worked out from the VAD alone.
fn first_transcript_ms(audio: &[f32]) -> u64 {
    let mut vad = VadEngine::new();
    let mut heard = 0;
    for (i, chunk) in audio.chunks(1600).enumerate() {
        heard += vad.process(chunk).map_or(0, |segment| segment.len());
        if heard >= 80000 {
            return i as u64 * 100;
        }
    }
    panic!("the audio never holds 5 s of speech");
}

#[test]
fn the_shared_loop_on_real_speech_agrees_with_a_replay_of_its_inputs() {
    let audio = service();
    let heard = listen_to(&audio, &verses(), |_, _| {});
    assert!(heard.lost, "the end of the audio closes the channel");

    // VAD speech reaches the sink.
    let speech = (heard.events.iter())
        .filter(|(_, e)| matches!(e, Event::Input(Input::Speech)))
        .count();
    assert!(speech > 20, "{speech} speech inputs");

    // The recogniser gets at least 5 s at a time, and the first transcript
    // comes when the VAD's utterances first add up to 5 s.
    assert!(heard.slices.len() >= 2, "{:?}", heard.slices);
    assert!(
        heard.slices.iter().all(|&n| n >= 80000),
        "{:?}",
        heard.slices
    );
    let first = (heard.events.iter())
        .find(|(_, e)| matches!(e, Event::Input(Input::Heard(_))))
        .map(|(t, _)| *t);
    assert_eq!(first, Some(first_transcript_ms(&audio)));

    // Each transcript is reported, then its song detection, then the
    // conductor's action: the GUI shows the detection before the slide.
    let mut transcripts = 0;
    for (i, (_, e)) in heard.events.iter().enumerate() {
        if let Event::Input(Input::Heard(_)) = e {
            transcripts += 1;
            let next: Vec<_> = heard.events[i + 1..i + 3].iter().map(|(_, e)| e).collect();
            assert!(
                matches!(next[..], [Event::Detection(_), Event::Action(..)]),
                "transcript {transcripts}"
            );
        }
    }
    assert_eq!(transcripts, heard.slices.len());

    // The live outputs (actions and conductor states) are what replay
    // computes from the recorded inputs alone ...
    let live: Vec<&str> = heard.log.lines().filter(outputs).collect();
    let inputs: String = (heard.log.lines())
        .filter(|l| !outputs(l))
        .map(|l| format!("{l}\n"))
        .collect();
    let again = |inputs: &str| -> Vec<String> {
        (replay(inputs, songs()).unwrap().iter())
            .map(Line::to_json)
            .filter(|l| outputs(&l.as_str()))
            .collect()
    };
    assert_eq!(again(&inputs), live);
    for want in [
        r#""action":"goto","song":0,"slide":0,"#,
        r#""action":"goto","song":0,"slide":1,"#,
        r#""action":"blank""#,
        r#""state":"singing","blank":false"#,
        r#""state":"inter_verse_silence","blank":false"#,
        r#""state":"blank_hold","blank":true"#,
    ] {
        assert!(live.iter().any(|l| l.contains(want)), "{want}: {live:#?}");
    }
    // ... and the speech inputs matter: without them the conductor would
    // fall silent between transcripts.
    let deaf: String = (inputs.lines())
        .filter(|l| !l.contains(r#""type":"speech""#))
        .map(|l| format!("{l}\n"))
        .collect();
    assert_ne!(again(&deaf), live);
}

/// The operator's Blank button (`commands::blank_output`) blanks the
/// conductor without changing its state; the GUI learns of it, and of the
/// next lyric match that unblanks, only from the loop's state events.
#[test]
fn a_blank_without_a_state_change_still_reaches_the_gui() {
    let mut blanked_at = None;
    let heard = listen_to(&service(), &verses(), |conductor, event| {
        if let Event::State(MachineState::Singing, false) = event {
            if blanked_at.is_none() {
                let mut c = conductor.blocking_lock();
                c.as_mut().unwrap().notify_external_blank();
                blanked_at = Some(());
            }
        }
    });
    assert!(blanked_at.is_some(), "the song never started");
    let states: Vec<(u64, MachineState, bool)> = (heard.events.iter())
        .filter_map(|(t, e)| match e {
            Event::State(s, b) => Some((*t, *s, *b)),
            _ => None,
        })
        .collect();
    // The blank is reported at the next tick, still singing ...
    let (t0, t1) = (states[0].0, states[1].0);
    assert_eq!(
        (states[0].1, states[0].2, states[1].1, states[1].2, t1),
        (
            MachineState::Singing,
            false,
            MachineState::Singing,
            true,
            t0 + 200
        ),
        "{states:?}"
    );
    // ... and the unblank at the tick after the slide that brought the
    // lyrics back.
    let goto = (heard.events.iter())
        .filter(|(_, e)| matches!(e, Event::Action(Action::Goto { .. }, _)))
        .map(|(t, _)| *t)
        .nth(1)
        .unwrap();
    let after = states.iter().find(|s| s.0 > goto);
    assert_eq!(
        after,
        Some(&(goto + 200, MachineState::Singing, false)),
        "{states:?}"
    );
}

/// Everything the VAD hands on from `audio` in 100 ms chunks, in order.
fn vad_speech(audio: &[f32]) -> Vec<f32> {
    let mut vad = VadEngine::new();
    (audio.chunks(1600))
        .filter_map(|chunk| vad.process(chunk))
        .flatten()
        .collect()
}

/// The PD hymns and three more songs that nobody sings: six candidates.
fn six_songs() -> Vec<Song> {
    let mut six = songs();
    for n in 4..=6 {
        let text = format!("filler number {n} is a slide that nobody in this test sings");
        let slides = json!([{ "id": 9 + n, "text": text }]);
        let song = json!({ "title": format!("Filler {n}"), "slides": slides });
        six.push(serde_json::from_value(song).unwrap());
    }
    six
}

/// Whisper fails on one slice and hears nothing in another. The GUI gets
/// one error and nothing at all for the empty transcript (no transcription
/// event, no repeat of the old song detection); the audio of both slices
/// is dropped, so the buffer cannot grow while errors repeat; and the loop
/// goes on to the next transcript.
#[test]
fn a_failed_or_empty_transcript_drops_its_audio_and_the_loop_goes_on() {
    // The service with a third verse: enough speech for four slices.
    let audio = [service(), wav_16k_mono(SPEECH_WAV)].concat();
    let verses = verses();
    // Slice 0 fails, slice 2 is empty, the others hear the verses in turn.
    let text = |n: usize| match n {
        0 | 2 => None,
        1 => verses.first(),
        n => verses.get(n - 2),
    };
    let recognise = |n: usize| match n {
        0 => Err(anyhow::anyhow!("whisper failed")),
        n => Ok(text(n).cloned().unwrap_or_default()),
    };
    let heard = listen_with(six_songs(), &audio, recognise, |_, _| {});
    assert!(heard.slices.len() >= 4, "{:?}", heard.slices);

    // The recogniser got the VAD's speech in order, none of it twice: the
    // audio of a failed or empty slice is not handed over again.
    let speech = vad_speech(&audio);
    assert!(heard.fed.len() <= speech.len(), "{:?}", heard.slices);
    assert!(heard.fed == speech[..heard.fed.len()], "{:?}", heard.slices);

    // What the GUI is told after each slice, in order.
    let mut told = Vec::new();
    for (_, e) in &heard.events {
        match e {
            Event::Error(m) => told.push(format!("error {m}")),
            Event::Input(Input::Heard(text)) => told.push(format!("heard {text}")),
            Event::Detection(rows) => told.push(format!("detection {}", rows.len())),
            _ => {}
        }
    }
    let mut want = vec!["error whisper failed".to_string()];
    for text in (1..heard.slices.len()).filter_map(text) {
        want.extend([format!("heard {text}"), "detection 5".to_string()]);
    }
    assert_eq!(told, want);
    let error = r#""type":"error","message":"whisper failed""#;
    assert!(heard.log.contains(error), "{}", heard.log);
    assert!(!heard.log.contains(r#""text":"""#), "{}", heard.log);

    // The loop went on: the slide follows the later transcripts.
    let shown: Vec<usize> = (heard.events.iter())
        .filter_map(|(_, e)| match e {
            Event::Action(Action::Goto { slide_in_song, .. }, _) => Some(*slide_in_song),
            _ => None,
        })
        .collect();
    assert!(shown.len() >= 2 && shown[0] == 0, "{shown:?}");
}

/// The GUI's only mic-signal meter: peak and RMS over each window of at
/// least 33 ms of audio, peak clipped at 1.0, both back to 0 once the
/// signal goes quiet.
#[test]
fn the_mic_meter_reports_peak_and_rms_per_33_ms_window_of_audio() {
    let (tx, rx) = mpsc::channel();
    // 10 ms chunks: one too loud (±1.5), then 200 ms of silence.
    let loud: Vec<f32> = (0..160)
        .map(|i| if i % 2 == 0 { 1.5 } else { -1.5 })
        .collect();
    tx.send(loud).unwrap();
    for _ in 0..20 {
        tx.send(vec![0.0f32; 160]).unwrap();
    }
    drop(tx);
    let base = Instant::now();
    let clock = |samples: u64| base + Duration::from_millis(samples / 16);
    let conductor = Mutex::new(Some(Conductor::new(songs(), SmartConfig::default())));
    let vad = Mutex::new(VadEngine::new());
    let mut mic = Vec::new();
    let sink = |now: Instant, event: Event| {
        if let Event::Mic(level) = event {
            mic.push((now.duration_since(base).as_millis(), level.peak, level.rms));
        }
    };
    let no_speech = |_: &[f32]| -> anyhow::Result<String> { panic!("a click was transcribed") };
    let run = AtomicBool::new(true);
    listen::run(rx, &run, &vad, &conductor, no_speech, clock, sink);
    // The first window holds the loud chunk and four quiet ones (800
    // samples): RMS sqrt(160 * 2.25 / 800). Every later window is silent.
    let first_rms = (160.0f32 * 2.25 / 800.0).sqrt();
    let mut want = vec![(40, 1.0, first_rms)];
    want.extend([80, 120, 160, 200].map(|t| (t, 0.0, 0.0)));
    assert_eq!(mic.len(), want.len(), "{mic:?}");
    for ((t, peak, rms), (want_t, want_peak, want_rms)) in mic.iter().zip(&want) {
        assert_eq!((*t, *peak), (*want_t, *want_peak), "{mic:?}");
        assert!((rms - want_rms).abs() < 1e-6, "{mic:?}");
    }
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
    let cases: [(&[&str], &str); 8] = [
        (&["--stdin-audio", &k8, "--setlist", SETLIST], "16 kHz mono"),
        (
            &["--replay", SESSION, "--setlist", SETLIST, "--shadow"],
            "--replay takes only",
        ),
        (
            &["--replay", SESSION, "--stdin-audio", &k16],
            "--replay takes only",
        ),
        (
            &["--setlist", SETLIST, "--realtime", "--device", "x"],
            "--realtime needs",
        ),
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
    // Keystroke warns on stderr before any audio is read; shadow does not.
    // The 8 kHz file stops both runs before a presenter is built.
    let stderr = |extra: &[&str]| {
        let args = ["--stdin-audio", &k8, "--setlist", SETLIST, "--presenter"];
        let out = Command::new(env!("CARGO_BIN_EXE_voxxad"))
            .args(args.iter().chain(["keystroke"].iter()).chain(extra))
            .output()
            .unwrap();
        assert!(!out.status.success());
        String::from_utf8(out.stderr).unwrap()
    };
    assert!(stderr(&[]).contains("focus the presentation app"));
    assert!(stderr(&["--shadow"]).is_empty());
}
