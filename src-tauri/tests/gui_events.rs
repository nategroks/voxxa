//! Q3-02 (#46): the GUI's sink for the shared listening loop. Each loop
//! event maps to one Tauri event, with the payload the frontend reads, or to
//! a dispatch of a loop action with `operator = false`. The mapping is pure,
//! so these run without a window, a model or an audio device.

use voxxa_lib::listen::{DetectionRow, Event, Input, MicLevel};
use voxxa_lib::{gui_event, presenter_error, Action, GuiEvent, MachineState};

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
