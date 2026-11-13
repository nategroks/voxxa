//! Q3-02 (#46): the GUI's sink for the shared listening loop. Each loop
//! event maps to one Tauri event, with the payload the frontend reads, or to
//! a dispatch of a loop action with `operator = false`. The mapping is pure,
//! so these run without a window, a model or an audio device.

use voxxa_lib::listen::{DetectionRow, Event, Input, MicLevel};
use voxxa_lib::{gui_event, gui_sink, presenter_error, Action, GuiEvent, MachineState};

fn emitted(event: Event) -> (&'static str, String) {
    match gui_event(event) {
        GuiEvent::Emit(name, payload) => (name, payload),
        other => panic!("not emitted: {other:?}"),
    }
}

#[test]
fn a_hypothesis_is_shown_as_a_transcription_event() {
    let heard = Event::Input(Input::Heard("Amazing grace, how sweet".into()));
    assert_eq!(
        emitted(heard),
        (
            "transcription",
            r#"{"text":"Amazing grace, how sweet"}"#.into()
        )
    );
}

#[test]
fn a_conductor_state_is_a_machine_state_event() {
    let cases = [
        (
            MachineState::Singing,
            false,
            r#"{"state":"singing","is_blank":false}"#,
        ),
        (
            MachineState::InterVerseSilence,
            false,
            r#"{"state":"inter_verse_silence","is_blank":false}"#,
        ),
        (
            MachineState::BlankHold,
            true,
            r#"{"state":"blank_hold","is_blank":true}"#,
        ),
        (
            MachineState::Listening,
            true,
            r#"{"state":"listening","is_blank":true}"#,
        ),
    ];
    for (state, blank, json) in cases {
        assert_eq!(
            emitted(Event::State(state, blank)),
            ("machine-state", json.into())
        );
    }
}

#[test]
fn the_mic_level_keeps_its_f32_digits() {
    let level = Event::Mic(MicLevel {
        peak: 0.1,
        rms: 0.05,
    });
    assert_eq!(
        emitted(level),
        ("mic-level", r#"{"peak":0.1,"rms":0.05}"#.into())
    );
}

#[test]
fn song_detection_rows_keep_their_field_order() {
    let rows = vec![DetectionRow {
        song_index: 2,
        song_title: "It Is Well with My Soul".into(),
        probability: 0.75,
        raw_score: 88.5,
    }];
    let want = r#"{"rows":[{"song_index":2,"song_title":"It Is Well with My Soul","probability":0.75,"raw_score":88.5}]}"#;
    assert_eq!(
        emitted(Event::Detection(rows)),
        ("song-detection", want.into())
    );
}

#[test]
fn every_loop_action_is_dispatched_as_not_from_the_operator() {
    let goto = Action::Goto {
        song_index: 0,
        slide_in_song: 1,
        global_index: 1,
        slide_text: "'Twas grace that taught my heart to fear".into(),
        next_slide_text: None,
        song_title: "Amazing Grace".into(),
    };
    for (action, total) in [
        (goto, Some(13)),
        (Action::Blank, Some(13)),
        (Action::Unblank, None),
        (Action::Noop, Some(13)),
    ] {
        let want = format!("{action:?}");
        match gui_event(Event::Action(action, total)) {
            GuiEvent::Dispatch(a, t, operator) => {
                assert_eq!((format!("{a:?}"), t), (want, total));
                assert!(!operator, "a loop action is not the operator's");
            }
            other => panic!("{other:?}"),
        }
    }
}

#[test]
fn inputs_other_than_hypotheses_and_errors_show_nothing() {
    for event in [
        Event::Input(Input::Tick),
        Event::Input(Input::Speech),
        Event::Error("whisper failed".into()),
        Event::Window("amazing grace how sweet the sound".into()),
    ] {
        assert!(matches!(gui_event(event), GuiEvent::Nothing));
    }
}

#[test]
fn a_failed_dispatch_is_a_presenter_error_event() {
    match presenter_error("ProPresenter refused".into()) {
        GuiEvent::Emit(name, payload) => {
            assert_eq!(
                (name, payload.as_str()),
                ("presenter-error", r#""ProPresenter refused""#)
            );
        }
        other => panic!("{other:?}"),
    }
}

/// What `gui_sink` did with some loop events: the Tauri events it emitted
/// and the actions it sent to `commands::dispatch`, with their operator flag.
struct Sunk {
    emitted: Vec<(&'static str, String)>,
    sent: Vec<(String, Option<usize>, bool)>,
}

/// Runs `gui_sink` over `events` with recording closures, as
/// `start_listening_with_state` runs it with the app's `emit_str` and
/// `commands::dispatch`. `fail` makes every dispatch fail with that error.
fn sink(events: Vec<Event>, fail: Option<&str>) -> Sunk {
    let mut sunk = Sunk {
        emitted: Vec::new(),
        sent: Vec::new(),
    };
    let mut emit = |name: &'static str, payload: String| sunk.emitted.push((name, payload));
    let mut sent = Vec::new();
    let mut send = |action: Action, total: Option<usize>, operator: bool| {
        sent.push((format!("{action:?}"), total, operator));
        fail.map_or(Ok(()), |e| Err(e.to_string()))
    };
    for event in events {
        gui_sink(event, &mut emit, &mut send);
    }
    sunk.sent = sent;
    sunk
}

fn loop_actions() -> Vec<(Action, Option<usize>)> {
    let goto = Action::Goto {
        song_index: 0,
        slide_in_song: 0,
        global_index: 0,
        slide_text: "Amazing grace, how sweet the sound".into(),
        next_slide_text: Some("'Twas grace that taught my heart to fear".into()),
        song_title: "Amazing Grace".into(),
    };
    vec![
        (goto, Some(13)),
        (Action::Blank, Some(13)),
        (Action::Unblank, None),
    ]
}

/// Q3-02 (#46): what the GUI's sink hands to `commands::dispatch`, not only
/// what `gui_event` maps. Every loop action is sent once, unchanged, with
/// `operator = false` (so `dispatch::drive` keeps its wrong-presentation
/// guard), and a successful send shows nothing.
#[test]
fn the_gui_sink_sends_every_loop_action_with_operator_false() {
    let events = (loop_actions().into_iter())
        .map(|(a, t)| Event::Action(a, t))
        .collect();
    let sunk = sink(events, None);
    let want: Vec<_> = (loop_actions().into_iter())
        .map(|(a, t)| (format!("{a:?}"), t, false))
        .collect();
    assert_eq!(sunk.sent, want);
    assert!(sunk.emitted.is_empty(), "{:?}", sunk.emitted);
}

/// A failed send is shown once as `presenter-error` with the error as a
/// JSON string, and the next action is still sent.
#[test]
fn a_failed_send_shows_exactly_one_presenter_error_per_action() {
    let events = (loop_actions().into_iter())
        .map(|(a, t)| Event::Action(a, t))
        .collect();
    let sunk = sink(events, Some("ProPresenter refused"));
    assert_eq!(sunk.sent.len(), 3);
    let error = ("presenter-error", r#""ProPresenter refused""#.to_string());
    assert_eq!(sunk.emitted, vec![error.clone(), error.clone(), error]);
}

/// Every event `gui_event` maps to a Tauri event reaches the emit closure
/// unchanged and in order; inputs, recogniser errors and window text reach
/// neither closure.
#[test]
fn the_gui_sink_emits_what_gui_event_maps_and_nothing_else() {
    let rows = || {
        vec![DetectionRow {
            song_index: 0,
            song_title: "Amazing Grace".into(),
            probability: 0.9,
            raw_score: 91.0,
        }]
    };
    let level = || MicLevel {
        peak: 0.5,
        rms: 0.25,
    };
    let shown = || {
        vec![
            Event::Input(Input::Heard("Amazing grace, how sweet".into())),
            Event::State(MachineState::Singing, false),
            Event::Mic(level()),
            Event::Detection(rows()),
        ]
    };
    let silent = [
        Event::Input(Input::Tick),
        Event::Input(Input::Speech),
        Event::Error("whisper failed".into()),
        Event::Window("amazing grace how sweet the sound".into()),
    ];
    let sunk = sink(shown().into_iter().chain(silent).collect(), None);
    let want: Vec<_> = shown().into_iter().map(emitted).collect();
    let names: Vec<&str> = want.iter().map(|w| w.0).collect();
    assert_eq!(
        names,
        [
            "transcription",
            "machine-state",
            "mic-level",
            "song-detection"
        ]
    );
    assert_eq!(sunk.emitted, want);
    assert!(sunk.sent.is_empty(), "{:?}", sunk.sent);
}
