//! Q3-02 (#46), owned by Q4: what the GUI does with each event of the shared
//! listening loop. The Tauri event names and payloads are what `src/` listens
//! for; the loop's own dispatches are never operator dispatches.

use serde_json::json;
use voxxa_lib::listen::{DetectionRow, Event, Input, MicLevel};
use voxxa_lib::{to_gui, Action, MachineState, ToGui, PRESENTER_ERROR};

fn emitted(event: Event) -> (&'static str, serde_json::Value) {
    match to_gui(event) {
        Some(ToGui::Emit(name, payload)) => (name, payload),
        other => panic!("{other:?}"),
    }
}

#[test]
fn every_loop_event_maps_to_its_tauri_event_and_payload() {
    let heard = Event::Input(Input::Heard("amazing grace".into()));
    assert_eq!(
        emitted(heard),
        ("transcription", json!({ "text": "amazing grace" }))
    );
    let state = Event::State(MachineState::InterVerseSilence, true);
    assert_eq!(
        emitted(state),
        (
            "machine-state",
            json!({ "state": "inter_verse_silence", "is_blank": true })
        )
    );
    let mic = Event::Mic(MicLevel {
        peak: 0.5,
        rms: 0.25,
    });
    assert_eq!(
        emitted(mic),
        ("mic-level", json!({ "peak": 0.5, "rms": 0.25 }))
    );
    let row = DetectionRow {
        song_index: 2,
        song_title: "It Is Well with My Soul".into(),
        probability: 0.75,
        raw_score: 88.0,
    };
    let rows = json!({ "rows": [{
        "song_index": 2, "song_title": "It Is Well with My Soul",
        "probability": 0.75, "raw_score": 88.0,
    }] });
    assert_eq!(
        emitted(Event::Detection(vec![row])),
        ("song-detection", rows)
    );
    // A failed dispatch is reported under this name (the sink uses it).
    assert_eq!(PRESENTER_ERROR, "presenter-error");
}

#[test]
fn the_loop_dispatches_its_actions_as_not_the_operator() {
    for (action, total) in [(Action::Blank, Some(13)), (Action::Unblank, None)] {
        match to_gui(Event::Action(action.clone(), total)) {
            Some(ToGui::Dispatch(got, got_total, operator)) => {
                assert_eq!(format!("{got:?}"), format!("{action:?}"));
                assert_eq!(got_total, total);
                assert!(!operator, "the loop dispatched as the operator");
            }
            other => panic!("{other:?}"),
        }
    }
}

#[test]
fn inputs_other_than_transcripts_and_errors_reach_no_gui_event() {
    for event in [
        Event::Input(Input::Tick),
        Event::Input(Input::Speech),
        Event::Error("whisper failed".into()),
    ] {
        assert!(to_gui(event).is_none());
    }
}
