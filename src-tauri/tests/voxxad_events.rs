//! Q3 / ADR-013: `voxxad`'s event schema v1, the session recorder, replay
//! determinism and the shared listening loop's clock; Q4 / ADR-014: the
//! loop's streaming hop. None of these needs a Whisper model or an audio
//! device (the recogniser's decoder is faked), so they run in CI.

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
use voxxa_lib::transcription::{Decode, Hypothesis, Recognizer, Rolling, Word};
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
        (
            Line::new(
                3000,
                Kind::Window {
                    text: "Amazing grace how sweet".into(),
                },
            ),
            r#"{"v":1,"t":3000,"type":"window","text":"Amazing grace how sweet"}"#,
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
fn only_tick_speech_and_window_stay_out_of_stdout() {
    assert!(!Line::new(0, Kind::Tick).on_stdout());
    assert!(!Line::new(0, Kind::Speech).on_stdout());
    let text = "amazing grace".to_string();
    assert!(!Line::new(0, Kind::Window { text }).on_stdout());
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

/// Decodes nothing: the VAD gate must keep it from ever being asked.
struct Silent;

impl Decode for Silent {
    fn decode(&mut self, _: &[f32], _: Option<&str>) -> anyhow::Result<Vec<Word>> {
        panic!("silence was decoded")
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
    let run = AtomicBool::new(true);
    let silent = &mut Rolling::new(Silent);
    let ended = listen::run(rx, &run, &vad, &conductor, silent, clock, sink);
    assert!(
        ended,
        "the end of the audio is reported as the channel closing"
    );
    assert_eq!(ticks, [200, 400, 600, 800, 1000]);
    assert_eq!(other, 0, "silence starts nothing");

    // A loop that was held up for a second (a slow transcript) ticks once
    // when it gets going again, not five times to catch up.
    let (tx, rx) = mpsc::channel();
    tx.send(vec![0.0f32; 16000]).unwrap();
    for _ in 0..5 {
        tx.send(vec![0.0f32; 16]).unwrap(); // 1 ms each
    }
    drop(tx);
    let mut ticks = Vec::new();
    let sink = |now: Instant, event: Event| {
        if let Event::Input(Input::Tick) = event {
            ticks.push(now.duration_since(base).as_millis());
        }
    };
    let conductor = Mutex::new(Some(Conductor::new(songs(), SmartConfig::default())));
    let silent = &mut Rolling::new(Silent);
    listen::run(rx, &run, &vad, &conductor, silent, clock, sink);
    assert_eq!(ticks, [1000]);

    // A stopped session returns at once and reports a stop, not a loss.
    let (_tx, rx) = mpsc::channel();
    let stopped = AtomicBool::new(false);
    let silent = &mut Rolling::new(Silent);
    let ended = listen::run(rx, &stopped, &vad, &conductor, silent, clock, |_, _| {});
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
    let silent = &mut Rolling::new(Silent);
    let lost = listen::run(rx, &run, &vad, &conductor, silent, clock, |_, _| {});
    assert_eq!(heard, 16000, "the stop came after the audio, mid-loop");
    assert!(!lost, "a stop was reported as a device loss");
}

/// Public-domain text (Amazing Grace, verse 1) read by the Windows SAPI
/// voice, 16 kHz mono 16-bit PCM, 12 s. Real speech for the real VAD; the
/// recogniser is faked, so no Whisper model is needed.
const SPEECH_WAV: &str = "tests/fixtures/voxxad/amazing-grace-v1.wav";
/// Verse 2 the same way (10.3 s).
const SPEECH_WAV_2: &str = "tests/fixtures/voxxad/amazing-grace-v2.wav";

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

fn quiet(secs: usize) -> Vec<f32> {
    vec![0.0f32; 16000 * secs]
}

/// 1 s of silence, the verse, 2 s of silence, the verse again, 12 s of
/// silence: long enough for inter-verse silence and the blank after it.
fn service() -> Vec<f32> {
    let verse = wav_16k_mono(SPEECH_WAV);
    [quiet(1), verse.clone(), quiet(2), verse, quiet(12)].concat()
}

/// What one `listen::run` over some audio produced.
struct Heard {
    /// Every event but mic levels, in order, with its audio time in ms.
    events: Vec<(u64, Event)>,
    /// The session log `voxxad` would write for this run.
    log: String,
    /// The length of each window the decoder was given.
    windows: Vec<usize>,
    /// Every sample the recogniser was fed, in order.
    fed: Vec<f32>,
    lost: bool,
}

/// A decoder told the number (0, 1, ...) of the window it is given; it
/// hears the answer's words 0.3 s apart from the window's start.
struct Numbered<F> {
    answer: F,
    windows: Vec<usize>,
}

impl<F: FnMut(usize) -> anyhow::Result<String>> Decode for Numbered<F> {
    fn decode(&mut self, window: &[f32], _: Option<&str>) -> anyhow::Result<Vec<Word>> {
        self.windows.push(window.len());
        let text = (self.answer)(self.windows.len() - 1)?;
        Ok((text.split_whitespace().enumerate())
            .map(|(i, w)| (w.to_string(), i as f32 * 0.3, 0.9))
            .collect())
    }
}

/// Wraps a recogniser and keeps everything it is fed.
struct Spy<'a, R> {
    inner: R,
    fed: &'a mut Vec<f32>,
}

impl<R: Recognizer> Recognizer for Spy<'_, R> {
    fn feed(&mut self, pcm16k: &[f32]) -> Vec<Hypothesis> {
        self.fed.extend_from_slice(pcm16k);
        self.inner.feed(pcm16k)
    }
    fn bias(&mut self, text: &str) {
        self.inner.bias(text)
    }
    fn gate(&mut self, open: bool) {
        self.inner.gate(open)
    }
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

/// `listen_to` with any setlist and any answer for window n.
fn listen_with(
    setlist: Vec<Song>,
    audio: &[f32],
    answer: impl FnMut(usize) -> anyhow::Result<String>,
    operator: impl FnMut(&Mutex<Option<Conductor>>, &Event),
) -> Heard {
    let numbered = Numbered {
        answer,
        windows: Vec::new(),
    };
    let mut fed = Vec::new();
    let mut spy = Spy {
        inner: Rolling::new(numbered),
        fed: &mut fed,
    };
    let (events, log, lost) = listen_with_recognizer(setlist, audio, &mut spy, operator);
    let windows = spy.inner.decoder().windows.clone();
    Heard {
        events,
        log,
        windows,
        fed,
        lost,
    }
}

type Run = (Vec<(u64, Event)>, String, bool);

fn listen_with_recognizer(
    setlist: Vec<Song>,
    audio: &[f32],
    recognizer: &mut impl Recognizer,
    mut operator: impl FnMut(&Mutex<Option<Conductor>>, &Event),
) -> Run {
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
    let (mut recorder, mut events) = (Recorder::new(base), Vec::new());
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
    let lost = listen::run(rx, &run, &vad, &conductor, recognizer, clock, sink);
    (events, log, lost)
}

/// The lines of Amazing Grace, slide by slide: what the fake recogniser hears.
fn verses() -> Vec<String> {
    songs()[0].slides.iter().map(|s| s.text.clone()).collect()
}

/// Audio times (ms) of the chunks the VAD calls speech, worked out from
/// the VAD alone: the end of each such 100 ms chunk.
fn vad_speech_ms(audio: &[f32]) -> Vec<u64> {
    let mut vad = VadEngine::new();
    (audio.chunks(1600).enumerate())
        .filter_map(|(i, chunk)| {
            vad.process(chunk);
            vad.is_in_speech().then_some((i as u64 + 1) * 100)
        })
        .collect()
}

/// The hop times the gate opens for: every whole second of audio with VAD
/// speech in the second before it.
fn hops_with_speech(audio: &[f32]) -> Vec<u64> {
    let speech = vad_speech_ms(audio);
    (1..=audio.len() as u64 / 16000)
        .map(|s| s * 1000)
        .filter(|&t| speech.iter().any(|&at| at <= t && t - at < 1000))
        .collect()
}

#[test]
fn the_shared_loop_on_real_speech_agrees_with_a_replay_of_its_inputs() {
    let audio = service();
    // The first hop hears verse 1, the next nothing, the third verse 2.
    let v = verses();
    let heard = listen_to(
        &audio,
        &[v[0].clone(), String::new(), v[1].clone()],
        |_, _| {},
    );
    assert!(heard.lost, "the end of the audio closes the channel");

    // VAD speech reaches the sink.
    let speech = (heard.events.iter())
        .filter(|(_, e)| matches!(e, Event::Input(Input::Speech)))
        .count();
    assert!(speech > 20, "{speech} speech inputs");

    // The recogniser is asked at every whole second of audio with speech in
    // the second before it, and only then (silence is never decoded), over
    // the last 5 s of audio or all of it if there is less.
    let hops = hops_with_speech(&audio);
    assert!(hops.len() >= 15, "{hops:?}");
    let windows: Vec<usize> = (hops.iter())
        .map(|&t| (t as usize * 16).min(80000))
        .collect();
    assert_eq!(heard.windows, windows);
    // Every sample is fed once, in order: overlap stays inside the window.
    assert!(
        heard.fed == audio,
        "{} of {} samples fed",
        heard.fed.len(),
        audio.len()
    );

    // A hypothesis is timed at the end of its hop's audio; a hop that hears
    // nothing gives none.
    let heard_at: Vec<u64> = (heard.events.iter())
        .filter(|(_, e)| matches!(e, Event::Input(Input::Heard(_))))
        .map(|(t, _)| *t)
        .collect();
    assert_eq!(heard_at, [hops[0], hops[2]]);

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
    assert_eq!(transcripts, 2);

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
    // Replay also returns the hypotheses, as recorded.
    let hypotheses = |lines: Vec<String>| -> Vec<String> {
        (lines.into_iter())
            .filter(|l| l.contains(r#""type":"hypothesis""#))
            .collect()
    };
    let replayed = replay(&inputs, songs()).unwrap();
    let recorded = heard.log.lines().map(String::from).collect();
    assert_eq!(
        hypotheses(replayed.iter().map(Line::to_json).collect()),
        hypotheses(recorded)
    );
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
    // ... and the unblank at the first tick after the slide that brought
    // the lyrics back (the tick at the hop's own time comes after it).
    let goto = (heard.events.iter())
        .position(|(_, e)| matches!(e, Event::Action(Action::Goto { .. }, _)))
        .and_then(|first| {
            (heard.events.iter().enumerate().skip(first + 1))
                .find(|(_, (_, e))| matches!(e, Event::Action(Action::Goto { .. }, _)))
        })
        .map(|(i, (t, _))| (i, *t))
        .unwrap();
    let after = (heard.events[goto.0..].iter())
        .find_map(|(t, e)| match e {
            Event::State(s, b) => Some((*t, *s, *b)),
            _ => None,
        })
        .unwrap();
    assert_eq!(after.1, MachineState::Singing, "{states:?}");
    assert!(
        !after.2 && after.0 - goto.1 < 200,
        "{after:?} after {goto:?}"
    );
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

/// Whisper fails on one window and hears nothing in another. The GUI gets
/// one error and nothing at all for the empty window (no transcription
/// event, no repeat of the old song detection); the loop goes on to the
/// next hop. The recogniser is still fed every sample once, in order: a
/// failed hop does not hold audio back or hand it over again.
#[test]
fn a_failed_or_empty_hop_tells_the_gui_one_error_or_nothing_and_the_loop_goes_on() {
    // The service with a third verse: enough speech for many hops.
    let audio = [service(), wav_16k_mono(SPEECH_WAV)].concat();
    let verses = verses();
    // Window 0 fails, window 2 is empty, the others hear the verses in turn.
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
    assert!(heard.windows.len() >= 6, "{:?}", heard.windows);
    assert!(heard.fed == audio, "{} of {}", heard.fed.len(), audio.len());

    // What the GUI is told after each hop, in order.
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
    for text in (1..heard.windows.len()).filter_map(text) {
        let words: Vec<&str> = text.split_whitespace().collect();
        want.extend([format!("heard {}", words.join(" ")), "detection 5".into()]);
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

/// Hears the words of a script that fall inside each window, as Whisper on
/// overlapping windows would: a word at stream time `t` is heard by every
/// window that holds `t`. The script's own clock is the samples fed.
struct Scripted {
    words: Vec<(String, f32)>,
    fed: std::rc::Rc<std::cell::Cell<u64>>,
}

impl Decode for Scripted {
    fn decode(&mut self, window: &[f32], _: Option<&str>) -> anyhow::Result<Vec<Word>> {
        let end = self.fed.get() as f32 / 16000.0;
        let start = end - window.len() as f32 / 16000.0;
        Ok((self.words.iter())
            .filter(|(_, t)| *t >= start && *t < end)
            .map(|(w, t)| (w.clone(), t - start, 0.9))
            .collect())
    }
}

/// Counts the samples fed before handing them on, for [`Scripted`].
struct Clocked<R> {
    inner: R,
    fed: std::rc::Rc<std::cell::Cell<u64>>,
}

impl<R: Recognizer> Recognizer for Clocked<R> {
    fn feed(&mut self, pcm16k: &[f32]) -> Vec<Hypothesis> {
        self.fed.set(self.fed.get() + pcm16k.len() as u64);
        self.inner.feed(pcm16k)
    }
    fn bias(&mut self, text: &str) {
        self.inner.bias(text)
    }
    fn gate(&mut self, open: bool) {
        self.inner.gate(open)
    }
}

/// `text`'s words spread evenly over `from..to` seconds.
fn spread(text: &str, from: f32, to: f32) -> Vec<(String, f32)> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let step = (to - from) / words.len() as f32;
    (words.iter().enumerate())
        .map(|(i, w)| (w.to_string(), from + i as f32 * step))
        .collect()
}

/// ADR-014: the conductor now hears the last 5 s again every second. On a
/// service of verse 1, verse 2 and a spoken prayer, each slide is shown
/// once, never twice in a row and never out of order, the prayer blanks,
/// and nothing is shown during the prayer.
#[test]
fn overlapping_windows_show_each_slide_once_and_the_prayer_nothing() {
    let verse = wav_16k_mono(SPEECH_WAV);
    let secs = verse.len() as f32 / 16000.0;
    // The VAD hears real speech in all three parts; the script says what
    // was said: verse 1, verse 2, then a prayer.
    let audio = [
        quiet(1),
        verse.clone(),
        quiet(1),
        verse.clone(),
        quiet(3),
        verse,
        quiet(8),
    ]
    .concat();
    let (v1, v2) = (1.0, 2.0 + secs);
    let prayer = 5.0 + 2.0 * secs;
    let pray = "Let us bow our heads and pray. Father, we thank you for this morning \
                and for everyone gathered here today. Be with us as we listen.";
    let verses = verses();
    let mut words = spread(&verses[0], v1 + 0.5, v1 + secs - 0.5);
    words.extend(spread(&verses[1], v2 + 0.5, v2 + secs - 0.5));
    words.extend(spread(pray, prayer + 0.5, prayer + secs - 0.5));
    let fed = std::rc::Rc::new(std::cell::Cell::new(0));
    let scripted = Scripted {
        words,
        fed: fed.clone(),
    };
    let mut recognizer = Clocked {
        inner: Rolling::new(scripted),
        fed,
    };
    let (events, log, _) = listen_with_recognizer(songs(), &audio, &mut recognizer, |_, _| {});
    let hypotheses = (events.iter())
        .filter(|(_, e)| matches!(e, Event::Input(Input::Heard(_))))
        .count();
    assert!(
        hypotheses >= 12,
        "{hypotheses} hypotheses: one every 1 to 2 s of speech"
    );

    let actions = actions_of(&events);
    let names: Vec<&str> = actions.iter().map(|(_, a)| a.as_str()).collect();
    assert_eq!(names, ["goto 0/0", "goto 0/1", "blank"], "{actions:?}");
    // Slide 1 comes while verse 2 is heard, the blank during the prayer,
    // and nothing after it.
    let (ms, at) = (|s: f32| (s * 1000.0) as u64, |i: usize| actions[i].0);
    assert!(at(0) < ms(v2), "{actions:?}");
    assert!(at(1) > ms(v2) && at(1) < ms(v2 + secs), "{actions:?}");
    assert!(
        at(2) > ms(prayer) && at(2) < ms(prayer + secs),
        "{actions:?}"
    );
    // Replay of this session agrees.
    replay_agrees(&log, songs());
}

/// The conductor's actions, with their audio times (ms), as "goto
/// song/slide", "blank" or "unblank".
fn actions_of(events: &[(u64, Event)]) -> Vec<(u64, String)> {
    (events.iter())
        .filter_map(|(t, e)| match e {
            Event::Action(
                Action::Goto {
                    slide_in_song,
                    song_index,
                    ..
                },
                _,
            ) => Some((*t, format!("goto {song_index}/{slide_in_song}"))),
            Event::Action(Action::Blank, _) => Some((*t, "blank".into())),
            Event::Action(Action::Unblank, _) => Some((*t, "unblank".into())),
            _ => None,
        })
        .collect()
}

/// Replay of a session log computes the same outputs from its inputs.
fn replay_agrees(log: &str, setlist: Vec<Song>) {
    let live: Vec<&str> = log.lines().filter(outputs).collect();
    let again: Vec<String> = (replay(log, setlist).unwrap().iter())
        .map(Line::to_json)
        .filter(|l| outputs(&l.as_str()))
        .collect();
    assert_eq!(again, live);
}

/// Real speech for the VAD, `secs` long: the two verse fixtures back to
/// back, repeated. What was said is scripted separately.
fn speech(secs: f32) -> Vec<f32> {
    let two = [wav_16k_mono(SPEECH_WAV), wav_16k_mono(SPEECH_WAV_2)].concat();
    let len = (secs * 16000.0) as usize;
    two.iter().cycle().take(len).copied().collect()
}

/// `slides` sung straight through at `rate` words a second over real
/// speech, heard by Whisper-like overlapping windows; then 8 s of silence.
/// Returns the conductor's actions, after checking that replay agrees.
fn sing(setlist: Vec<Song>, slides: &[String], rate: f32) -> Vec<(u64, String)> {
    let text = slides.join(" ");
    let secs = text.split_whitespace().count() as f32 / rate;
    let audio = [quiet(1), speech(secs + 1.0), quiet(8)].concat();
    let fed = std::rc::Rc::new(std::cell::Cell::new(0));
    let scripted = Scripted {
        words: spread(&text, 1.0, 1.0 + secs),
        fed: fed.clone(),
    };
    let mut recognizer = Clocked {
        inner: Rolling::new(scripted),
        fed,
    };
    let run = listen_with_recognizer(setlist.clone(), &audio, &mut recognizer, |_, _| {});
    replay_agrees(&run.1, setlist);
    actions_of(&run.0)
}

/// ADR-014 Decision 7 (the stitcher): each hop hands the conductor only
/// the words its window adds, padded to the conductor's 4-word floor. It
/// Is Well arranged V1 C V2 C V3 C (the chorus three times) and the four
/// verses of Amazing Grace, sung at 2, 2.5 and 3 words a second: every
/// slide is shown once, in order, with no skip, no repeat and no blank
/// before the last one.
#[test]
fn stitched_hops_show_every_slide_once_in_order_at_any_singing_speed() {
    let mut setlist = songs();
    let well = setlist[2].slides.clone();
    let arranged: Vec<_> = [0, 1, 2, 1, 3, 1]
        .iter()
        .enumerate()
        .map(|(id, &v)| json!({ "id": 20 + id, "text": well[v].text }))
        .collect();
    let song = json!({ "title": "It Is Well (V1 C V2 C V3 C)", "slides": arranged });
    setlist[2] = serde_json::from_value(song).unwrap();
    let text = |song: usize| -> Vec<String> {
        (setlist[song].slides.iter())
            .map(|s| s.text.clone())
            .collect()
    };
    let songs = [(0, text(0)), (2, text(2))];
    std::thread::scope(|scope| {
        let runs: Vec<_> = (songs.iter())
            .flat_map(|song| [2.0, 2.5, 3.0].map(|rate| (song, rate)))
            .map(|((song, slides), rate)| {
                let setlist = setlist.clone();
                let run = scope.spawn(move || sing(setlist, slides, rate));
                (*song, slides.len(), rate, run)
            })
            .collect();
        for (song, slides, rate, run) in runs {
            let actions = run.join().unwrap();
            let names: Vec<&str> = actions.iter().map(|(_, a)| a.as_str()).collect();
            let gotos: Vec<&str> = (names.iter().copied())
                .filter(|a| a.starts_with("goto"))
                .collect();
            let want: Vec<String> = (0..slides).map(|i| format!("goto {song}/{i}")).collect();
            assert_eq!(gotos, want, "song {song} at {rate} words/s: {actions:?}");
            let last = names.iter().rposition(|a| a.starts_with("goto")).unwrap();
            assert!(
                !names[..last].contains(&"blank"),
                "song {song} at {rate} words/s: {actions:?}"
            );
        }
    });
}

/// A decoder that keeps the loop busy for `cost` of wall time on
/// [`slow_run`]'s clock and hears the verses in turn. It records, for each
/// decode, the stream time (ms) and how far the loop was behind the audio.
struct Slow<'a> {
    cost: Duration,
    /// Wall time (from the session start) the loop is busy until.
    busy: &'a std::cell::Cell<Duration>,
    /// Samples received at the loop's last clock reading.
    seen: &'a std::cell::Cell<u64>,
    fed: std::rc::Rc<std::cell::Cell<u64>>,
    decodes: Vec<(u64, Duration)>,
}

impl Decode for Slow<'_> {
    fn decode(&mut self, _: &[f32], _: Option<&str>) -> anyhow::Result<Vec<Word>> {
        let audio = Duration::from_millis(self.seen.get() / 16);
        let wall = audio.max(self.busy.get());
        self.decodes.push((self.fed.get() / 16, wall - audio));
        self.busy.set(wall + self.cost);
        let verses = verses();
        let text = &verses[self.decodes.len() % verses.len()];
        Ok((text.split_whitespace().enumerate())
            .map(|(i, w)| (w.to_string(), i as f32 * 0.3, 0.9))
            .collect())
    }
}

struct SlowRun {
    /// Stream time (ms) of each decode, and how far behind the loop was.
    decodes: Vec<(u64, Duration)>,
    fed: Vec<f32>,
    /// How far behind the audio the loop was when the audio ended.
    behind_at_end: Duration,
    actions: usize,
}

/// The shared loop on a wall clock (the GUI, `--realtime`): audio arrives
/// in real time and a backlog drains at once, so wall time is the later of
/// the audio received and the time the loop is busy until. Each decode
/// keeps the loop busy for `cost`, each dispatched action for `dispatch`.
fn slow_run(audio: &[f32], cost: Duration, dispatch: Duration) -> SlowRun {
    let (tx, rx) = mpsc::channel();
    for chunk in audio.chunks(1600) {
        tx.send(chunk.to_vec()).unwrap();
    }
    drop(tx);
    let base = Instant::now();
    let (busy, seen) = (
        std::cell::Cell::new(Duration::ZERO),
        std::cell::Cell::new(0),
    );
    let clock = |samples: u64| {
        seen.set(samples);
        base + Duration::from_millis(samples / 16).max(busy.get())
    };
    let fed_count = std::rc::Rc::new(std::cell::Cell::new(0));
    let slow = Slow {
        cost,
        busy: &busy,
        seen: &seen,
        fed: fed_count.clone(),
        decodes: Vec::new(),
    };
    let mut fed = Vec::new();
    let mut spy = Spy {
        inner: Clocked {
            inner: Rolling::new(slow),
            fed: fed_count,
        },
        fed: &mut fed,
    };
    let mut actions = 0;
    let sink = |now: Instant, event: Event| {
        if let Event::Action(action, _) = event {
            if !matches!(action, Action::Noop) {
                actions += 1;
                busy.set(now.duration_since(base) + dispatch);
            }
        }
    };
    let conductor = Mutex::new(Some(Conductor::new(songs(), SmartConfig::default())));
    let vad = Mutex::new(VadEngine::new());
    let run = AtomicBool::new(true);
    listen::run(rx, &run, &vad, &conductor, &mut spy, clock, sink);
    let decodes = spy.inner.inner.decoder().decodes.clone();
    drop(spy);
    let end = Duration::from_millis(audio.len() as u64 / 16);
    SlowRun {
        decodes,
        fed,
        behind_at_end: busy.get().saturating_sub(end),
        actions,
    }
}

/// ADR-014 Decision 5 (Q3-06): a recogniser slower than real time must not
/// let audio queue up without bound on a wall clock. The loop decodes only
/// while it is at most one hop behind and skips the hops in between, so a
/// 2.5 s decode gives about one decode per 2.5 s of speech, never one a
/// second; every sample is still fed once, in order. Time spent in the
/// conductor and in dispatch counts too: it is wall time like any other.
/// At file speed every whole second with recent speech is decoded.
#[test]
fn a_slow_recogniser_skips_hops_and_the_lag_stays_bounded() {
    let audio = service();
    let hops = hops_with_speech(&audio);
    let (hop, decode) = (Duration::from_secs(1), Duration::from_millis(2500));

    // File speed: one decode at every whole second with recent speech, and
    // the loop is never behind.
    let fast = slow_run(&audio, Duration::ZERO, Duration::ZERO);
    let at: Vec<u64> = fast.decodes.iter().map(|d| d.0).collect();
    assert_eq!(at, hops);
    assert!(fast.decodes.iter().all(|d| d.1.is_zero()));

    // 2.5 s a decode, then also 700 ms per dispatched action.
    let slow = slow_run(&audio, decode, Duration::ZERO);
    let dispatching = slow_run(&audio, decode, Duration::from_millis(700));
    assert!(dispatching.actions >= 2, "{} actions", dispatching.actions);
    for run in [&slow, &dispatching] {
        assert!(run.fed == audio, "{} of {}", run.fed.len(), audio.len());
        // No decode starts while the loop is more than a hop behind ...
        assert!(run.decodes.iter().all(|d| d.1 <= hop), "{:?}", run.decodes);
        // ... so there are about half as many as at file speed, not one a
        // second of speech ...
        let n = run.decodes.len();
        assert!(n >= 4 && n * 2 <= hops.len() + 1, "{n} of {}", hops.len());
        // ... and when the audio ends, the loop is less than one decode and
        // one hop behind.
        assert!(run.behind_at_end < decode + hop, "{:?}", run.behind_at_end);
    }
    assert!(dispatching.decodes.len() <= slow.decodes.len());
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
    let run = AtomicBool::new(true);
    let silent = &mut Rolling::new(Silent);
    listen::run(rx, &run, &vad, &conductor, silent, clock, sink);
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
    let cases: [(&[&str], &str); 9] = [
        (&["--stdin-audio", &k8, "--setlist", SETLIST], "16 kHz mono"),
        (
            &["--replay", SESSION, "--setlist", SETLIST, "--shadow"],
            "--replay takes only",
        ),
        (
            &["--replay", SESSION, "--setlist", SETLIST, "--bias"],
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

/// Q3-01: `--shadow` must never reach the presenter. Deleting the shadow
/// guard in `connect_presenter` used to pass every gate. The presenter is
/// built before the model is picked, so an unknown model ends both runs
/// without needing Whisper, after the presenter step.
#[test]
fn shadow_never_connects_to_the_presenter() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port().to_string();
    let args = |shadow: bool| {
        let mut a = vec!["--stdin-audio", SPEECH_WAV, "--setlist", SETLIST];
        a.extend(["--presenter", "propresenter7_rest", "--host", "127.0.0.1"]);
        a.extend(["--port", &port, "--model", "nonexistent"]);
        a.extend(shadow.then_some("--shadow"));
        a.into_iter().map(String::from).collect::<Vec<_>>()
    };
    let error = |args: Vec<String>| {
        let out = Command::new(env!("CARGO_BIN_EXE_voxxad"))
            .args(args)
            .output()
            .unwrap();
        assert!(!out.status.success());
        let stdout = String::from_utf8(out.stdout).unwrap();
        match serde_json::from_str::<Line>(stdout.trim()).unwrap().kind {
            Kind::Error { message } => message,
            other => panic!("{other:?}"),
        }
    };
    // Shadow: nothing connects; the run fails later, on the model.
    listener.set_nonblocking(true).unwrap();
    let message = error(args(true));
    assert!(message.contains("unknown model"), "{message}");
    let pending = listener.accept().map(|_| ()).map_err(|e| e.kind());
    assert_eq!(pending, Err(std::io::ErrorKind::WouldBlock));
    // Without shadow the same command connects, and fails on the presenter
    // because this listener hangs up without answering.
    listener.set_nonblocking(false).unwrap();
    let hang_up = std::thread::spawn(move || listener.accept().is_ok());
    let message = error(args(false));
    assert!(message.starts_with("presenter:"), "{message}");
    assert!(hang_up.join().unwrap());
}

/// ADR-014 exit criterion: the session log has one `window` line per
/// decoded hop, at the hop's time, so the log shows what the recogniser
/// heard every second of speech even while the stitcher holds words back.
/// Window lines are not conductor inputs: replay gives the same outputs
/// with or without them.
#[test]
fn every_decoded_hop_is_one_window_line_in_the_session_log() {
    let audio = service();
    let hops = hops_with_speech(&audio);
    // Every window hears the same words, so the stitcher hands on only the
    // first hop after each pause (it is cleared when the gate closes).
    let same = "Amazing grace how sweet the sound";
    let heard = listen_with(songs(), &audio, |_| Ok(same.into()), |_, _| {});
    assert_eq!(heard.windows.len(), hops.len());
    let lines: Vec<Line> = (heard.log.lines())
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let window_at: Vec<u64> = (lines.iter())
        .filter(|l| matches!(&l.kind, Kind::Window { text } if text == same))
        .map(|l| l.t)
        .collect();
    // That is every whole second of audio with VAD speech in the second
    // before it: 1000 ms apart while the VAD hears speech.
    assert_eq!(window_at, hops);
    assert!(hops.len() >= 15, "{hops:?}");
    let hypotheses = (lines.iter())
        .filter(|l| matches!(l.kind, Kind::Hypothesis { .. }))
        .count();
    let pauses = window_at.windows(2).filter(|w| w[1] - w[0] > 1000).count();
    assert_eq!(hypotheses, 1 + pauses, "{window_at:?}");

    let without: String = (heard.log.lines())
        .filter(|l| !l.contains(r#""type":"window""#))
        .map(|l| format!("{l}\n"))
        .collect();
    let out = |log: &str| replay(log, songs()).unwrap();
    assert_eq!(out(&heard.log), out(&without));
    replay_agrees(&heard.log, songs());
}
