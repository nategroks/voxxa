//! What reaches the audience screen: `voxxa_lib::drive` with fake presenters
//! that record every call (Q2-19). Covers the toggle-blank inversion (Q2-03,
//! audit 2), keys typed into Voxxa's own window (Q2-04, audit 3) and the
//! setlist-global Goto index (Q2-05, audit 4).

use async_trait::async_trait;
use std::sync::Mutex;
use voxxa_lib::{
    drive, Action, Capabilities, CtlError, PresentationController, PresenterConfig, PresenterKind,
    PresenterState, Sent,
};

struct Fake {
    kind: PresenterKind,
    caps: Capabilities,
    /// What the app reports as its selected presentation.
    presentation: Mutex<Option<String>>,
    calls: Mutex<Vec<String>>,
}

impl Fake {
    fn new(kind: PresenterKind, caps: Capabilities) -> Self {
        Self {
            kind,
            caps,
            presentation: Mutex::new(None),
            calls: Mutex::new(Vec::new()),
        }
    }
    /// The default Keystroke driver: '.' toggles blank, no goto.
    fn toggle() -> Self {
        Self::new(PresenterKind::Keystroke, Capabilities::KEYSTROKE)
    }
    /// ProPresenter 7 REST: goto + state query.
    fn goto(selected: &str) -> Self {
        let f = Self::new(PresenterKind::ProPresenter7Rest, Capabilities::FULL);
        *f.presentation.lock().unwrap() = Some(selected.into());
        f
    }
    fn log(&self, call: String) -> Result<(), CtlError> {
        self.calls.lock().unwrap().push(call);
        Ok(())
    }
    /// Calls since the last take.
    fn take(&self) -> Vec<String> {
        std::mem::take(&mut *self.calls.lock().unwrap())
    }
}

#[async_trait]
impl PresentationController for Fake {
    fn kind(&self) -> PresenterKind {
        self.kind
    }
    fn capabilities(&self) -> Capabilities {
        self.caps
    }
    fn is_connected(&self) -> bool {
        true
    }
    async fn connect(&mut self, _: PresenterConfig) -> Result<(), CtlError> {
        Ok(())
    }
    async fn disconnect(&mut self) -> Result<(), CtlError> {
        Ok(())
    }
    async fn next_slide(&self) -> Result<(), CtlError> {
        self.log("next".into())
    }
    async fn prev_slide(&self) -> Result<(), CtlError> {
        self.log("prev".into())
    }
    async fn blank(&self) -> Result<(), CtlError> {
        self.log("blank".into())
    }
    async fn unblank(&self) -> Result<(), CtlError> {
        self.log("unblank".into())
    }
    async fn goto_slide(&self, index: u32) -> Result<(), CtlError> {
        self.log(format!("goto {index}"))
    }
    async fn current_state(&self) -> Result<PresenterState, CtlError> {
        Ok(PresenterState {
            presentation_name: self.presentation.lock().unwrap().clone(),
            ..Default::default()
        })
    }
}

/// Goto as the conductor emits it for setlist.example.json
/// (Amazing Grace: 4 slides, Be Thou My Vision: 4, It Is Well: 5).
fn goto(song: usize, slide: usize) -> Action {
    let (offset, title) = [
        (0, "Amazing Grace"),
        (4, "Be Thou My Vision"),
        (8, "It Is Well with My Soul"),
    ][song];
    Action::Goto {
        song_index: song,
        slide_in_song: slide,
        global_index: offset + slide,
        slide_text: String::new(),
        next_slide_text: None,
        song_title: title.into(),
    }
}

/// Conductor action: not focused, not an operator action.
async fn auto(p: &Fake, sent: &mut Sent, action: Action) -> Result<(), CtlError> {
    drive(p, sent, &action, false, false).await
}

#[tokio::test]
async fn toggle_blank_is_pressed_only_when_the_state_changes() {
    let p = Fake::toggle();
    let mut sent = Sent::default();
    // First detection after launch: Voxxa has blanked nothing, so no toggle.
    auto(&p, &mut sent, goto(0, 0)).await.unwrap();
    assert_eq!(p.take(), Vec::<String>::new());
    // Song -> Blank -> the same slide again: exactly one blank, one unblank.
    auto(&p, &mut sent, Action::Blank).await.unwrap();
    auto(&p, &mut sent, goto(0, 0)).await.unwrap();
    assert_eq!(p.take(), ["blank", "unblank"]);
    // A second Blank while blanked must not press the toggle again.
    auto(&p, &mut sent, Action::Blank).await.unwrap();
    auto(&p, &mut sent, Action::Blank).await.unwrap();
    assert_eq!(p.take(), ["blank"]);
    // The operator's Blank (tray, button, HTTP) shares the flag.
    let mut sent = Sent::default();
    auto(&p, &mut sent, goto(0, 0)).await.unwrap();
    drive(&p, &mut sent, &Action::Blank, false, true)
        .await
        .unwrap();
    auto(&p, &mut sent, goto(0, 1)).await.unwrap();
    assert_eq!(p.take(), ["blank", "next", "unblank"]);
    assert!(!sent.blank);
}

#[tokio::test]
async fn keystrokes_are_refused_while_voxxa_has_focus() {
    let p = Fake::toggle();
    let mut sent = Sent::default();
    auto(&p, &mut sent, goto(0, 0)).await.unwrap();
    for action in [goto(0, 1), Action::Blank, Action::Unblank] {
        for operator in [false, true] {
            let r = drive(&p, &mut sent, &action, true, operator).await;
            assert!(r
                .unwrap_err()
                .to_string()
                .contains("focus your presentation app"));
        }
    }
    assert_eq!(p.take(), Vec::<String>::new(), "no key may reach Voxxa");
    assert_eq!(
        sent,
        Sent {
            global: 0,
            song: None,
            blank: false
        }
    );
    // Not focused: keys go out as normal.
    drive(&p, &mut sent, &goto(0, 1), false, true)
        .await
        .unwrap();
    assert_eq!(p.take(), ["next"]);
    // Network drivers do not type, so focus does not matter to them.
    let p = Fake::goto("Amazing Grace");
    drive(&p, &mut Sent::default(), &goto(0, 1), true, false)
        .await
        .unwrap();
    assert_eq!(p.take(), ["goto 1"]);
}

#[tokio::test]
async fn goto_drivers_get_the_slide_within_the_selected_song() {
    let p = Fake::goto("Amazing Grace");
    let mut sent = Sent::default();
    auto(&p, &mut sent, goto(0, 2)).await.unwrap();
    assert_eq!(p.take(), ["goto 2"], "slide_in_song, not the setlist index");
    // Next song detected, but the app still has Amazing Grace selected:
    // blank (the safe failure) and ask the operator.
    let err = auto(&p, &mut sent, goto(1, 0)).await.unwrap_err();
    assert!(
        err.to_string().contains("select \"Be Thou My Vision\""),
        "{err}"
    );
    assert_eq!(p.take(), ["blank"]);
    // The operator selects it; the next Goto jumps and shows the slide.
    *p.presentation.lock().unwrap() = Some("Be Thou My Vision (Irish)".into());
    auto(&p, &mut sent, goto(1, 0)).await.unwrap();
    assert_eq!(p.take(), ["goto 0", "unblank"]);
    auto(&p, &mut sent, goto(1, 1)).await.unwrap();
    assert_eq!(p.take(), ["goto 1"]);
}

#[tokio::test]
async fn goto_drivers_that_cannot_report_the_presentation_trust_only_the_operator() {
    // FreeShow, OpenSong, ProPresenter WS: goto, no state query.
    let caps = Capabilities {
        can_query_state: false,
        ..Capabilities::FULL
    };
    let p = Fake::new(PresenterKind::FreeShow, caps);
    let mut sent = Sent::default();
    assert!(auto(&p, &mut sent, goto(2, 0)).await.is_err());
    assert_eq!(p.take(), ["blank"]);
    // The operator clicks the song chip (jump_to_song): trusted.
    drive(&p, &mut sent, &goto(2, 0), false, true)
        .await
        .unwrap();
    assert_eq!(p.take(), ["goto 0", "unblank"]);
    auto(&p, &mut sent, goto(2, 3)).await.unwrap();
    assert_eq!(p.take(), ["goto 3"]);
}
