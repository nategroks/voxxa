//! What reaches the audience screen: `voxxa_lib::drive` with fake presenters
//! that record every call (Q2-19). Covers the toggle-blank inversion (Q2-03,
//! audit 2), keys typed into Voxxa's own window (Q2-04, audit 3) and the
//! setlist-global Goto index (Q2-05, audit 4).

use async_trait::async_trait;
use std::sync::Mutex;
use voxxa_lib::{
    drive, make_controller, step, Action, Capabilities, CtlError, PresentationController,
    PresenterConfig, PresenterKind, PresenterState, Sent,
};

struct Fake {
    kind: PresenterKind,
    caps: Capabilities,
    /// What the app reports as its selected presentation.
    presentation: Mutex<Option<String>>,
    calls: Mutex<Vec<String>>,
    /// What the audience sees, as PowerPoint and Keynote behave: (slide,
    /// black). The toggle blacks out and back; while black, any key only
    /// ends the black.
    screen: Mutex<(i64, bool)>,
}

impl Fake {
    fn new(kind: PresenterKind, caps: Capabilities) -> Self {
        Self {
            kind,
            caps,
            presentation: Mutex::new(None),
            calls: Mutex::new(Vec::new()),
            screen: Mutex::new((0, false)),
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
        let mut screen = self.screen.lock().unwrap();
        match call.as_str() {
            "blank" | "unblank" => screen.1 = !screen.1,
            _ if screen.1 => screen.1 = false,
            "next" => screen.0 += 1,
            "prev" => screen.0 -= 1,
            _ => {}
        }
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
    assert_eq!(p.take(), ["blank", "unblank", "next"]);
    assert!(!sent.blank);
}

#[tokio::test]
async fn a_step_after_a_blank_shows_the_slide_not_black() {
    // PowerPoint and Keynote: any key ends the black screen. Stepping first
    // and toggling after would end the black and then black out the verse,
    // leaving the blank state inverted for the next Blank.
    let p = Fake::toggle();
    let mut sent = Sent::default();
    auto(&p, &mut sent, goto(0, 0)).await.unwrap();
    auto(&p, &mut sent, Action::Blank).await.unwrap();
    assert_eq!(*p.screen.lock().unwrap(), (0, true));
    auto(&p, &mut sent, goto(0, 1)).await.unwrap();
    assert_eq!(*p.screen.lock().unwrap(), (1, false), "slide 1, not black");
    auto(&p, &mut sent, Action::Blank).await.unwrap();
    assert_eq!(
        *p.screen.lock().unwrap(),
        (1, true),
        "the next Blank blanks"
    );
    // OpenSong: the other toggle driver also unblanks first.
    let kind = PresenterKind::OpenSong;
    let p = Fake::new(kind, make_controller(kind).capabilities());
    let mut sent = Sent::default();
    drive(&p, &mut sent, &goto(0, 0), false, true)
        .await
        .unwrap();
    auto(&p, &mut sent, Action::Blank).await.unwrap();
    auto(&p, &mut sent, goto(0, 2)).await.unwrap();
    assert_eq!(p.take(), ["blank", "unblank", "next", "next"]);
}

#[tokio::test]
async fn keystrokes_are_refused_while_voxxa_has_focus() {
    let p = Fake::toggle();
    let mut sent = Sent::default();
    auto(&p, &mut sent, goto(0, 0)).await.unwrap();
    for action in [goto(0, 1), Action::Blank] {
        for operator in [false, true] {
            let r = drive(&p, &mut sent, &action, true, operator).await;
            assert!(r
                .unwrap_err()
                .to_string()
                .contains("focus your presentation app"));
        }
    }
    // Unblank while not blanked presses nothing, so there is nothing to refuse.
    drive(&p, &mut sent, &Action::Unblank, true, false)
        .await
        .unwrap();
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
async fn a_first_goto_while_voxxa_has_focus_still_records_the_slide() {
    // The operator presses Start in Voxxa; the first detection sends no key
    // (the position is unknown), so focus must not stop it from recording
    // where the screen is. Otherwise the deck stays one slide behind.
    let p = Fake::toggle();
    let mut sent = Sent::default();
    drive(&p, &mut sent, &goto(0, 0), true, false)
        .await
        .unwrap();
    assert_eq!(p.take(), Vec::<String>::new());
    // The operator clicks PowerPoint; the next line advances one slide.
    auto(&p, &mut sent, goto(0, 1)).await.unwrap();
    assert_eq!(p.take(), ["next"]);
}

#[tokio::test]
async fn a_shorter_presentation_name_is_not_the_song() {
    // ProPresenter still shows "Grace" while the room sings Amazing Grace.
    let p = Fake::goto("Grace");
    let mut sent = Sent::default();
    assert!(auto(&p, &mut sent, goto(0, 1)).await.is_err());
    assert_eq!(p.take(), ["blank"], "no slide of the wrong presentation");
}

#[tokio::test]
async fn a_bracketed_subtitle_need_not_be_in_the_presentation_name() {
    // Planning Center: "Amazing Grace (My Chains Are Gone)"; ProPresenter:
    // "Amazing Grace". Without this the song could never be shown.
    let p = Fake::goto("Amazing Grace");
    let mut sent = Sent::default();
    let action = Action::Goto {
        song_index: 0,
        slide_in_song: 1,
        global_index: 1,
        slide_text: String::new(),
        next_slide_text: None,
        song_title: "Amazing Grace (My Chains Are Gone)".into(),
    };
    auto(&p, &mut sent, action).await.unwrap();
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
    // FreeShow: the only goto driver without a state query.
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

#[tokio::test]
async fn a_raw_step_after_a_blank_keeps_the_toggle_in_step() {
    // Next/Prev with no song active (button, HTTP /next, Stream Deck) press
    // one raw key. In PowerPoint any key ends the black, so the flag must
    // follow, or the first detection presses the toggle and blacks out.
    let p = Fake::toggle();
    let mut sent = Sent::default();
    auto(&p, &mut sent, goto(0, 0)).await.unwrap();
    drive(&p, &mut sent, &Action::Blank, false, true)
        .await
        .unwrap();
    step(&p, &mut sent, 1, false).await.unwrap();
    assert_eq!(*p.screen.lock().unwrap(), (1, false), "slide 1 shows");
    assert!(!sent.blank);
    auto(&p, &mut sent, goto(0, 0)).await.unwrap();
    assert_eq!(*p.screen.lock().unwrap(), (0, false), "verse 1 visible");
    assert!(!sent.blank);
    auto(&p, &mut sent, Action::Blank).await.unwrap();
    assert_eq!(
        *p.screen.lock().unwrap(),
        (0, true),
        "the next Blank blanks"
    );
    assert_eq!(p.take(), ["blank", "unblank", "next", "prev", "blank"]);
    // Voxxa focused: refused, nothing pressed, nothing recorded.
    let before = sent;
    assert!(step(&p, &mut sent, 1, true).await.is_err());
    assert_eq!((p.take(), sent), (vec![], before));
    // Network drivers: one key, no toggle bookkeeping.
    let p = Fake::goto("Amazing Grace");
    let mut sent = Sent::default();
    step(&p, &mut sent, -1, true).await.unwrap();
    assert_eq!(p.take(), ["prev"]);
}

#[tokio::test]
async fn a_raw_step_before_any_detection_keeps_the_position_unknown() {
    // Stream Deck Next through announcement slides before any song is
    // detected: Voxxa does not know where the deck is, so the first
    // detection must press nothing rather than step from a made-up position.
    let p = Fake::toggle();
    let mut sent = Sent::default();
    step(&p, &mut sent, 1, false).await.unwrap();
    auto(&p, &mut sent, goto(1, 1)).await.unwrap();
    assert_eq!(p.take(), ["next"], "only the operator's own key");
    assert_eq!(sent.global, 5);
}

#[tokio::test]
async fn drivers_without_a_per_song_slide_index_step_across_songs() {
    // PT2-05: ProPresenter WS "0:0" is the first playlist item and OpenSong
    // numbers slides across the set, so a goto would show the wrong song.
    use PresenterKind::*;
    for kind in [ProPresenter7Ws, ProPresenter6Ws, OpenSong] {
        let p = Fake::new(kind, make_controller(kind).capabilities());
        let mut sent = Sent::default();
        auto(&p, &mut sent, goto(0, 0)).await.unwrap();
        auto(&p, &mut sent, goto(1, 0)).await.unwrap();
        auto(&p, &mut sent, goto(0, 3)).await.unwrap();
        assert_eq!(
            p.take(),
            ["next", "next", "next", "next", "prev"],
            "{kind:?}"
        );
    }
}
