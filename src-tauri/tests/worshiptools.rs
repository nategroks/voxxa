//! Q9W, ADR-019: WorshipTools Presenter. Discovery by process (a fake
//! process list), the never-step-across-songs rule and the missing blank, all
//! through `voxxa_lib::drive` with a fake presenter that records every call.
//! The fake takes its capabilities and its song-end flag from the real
//! keystroke driver connected with the WorshipTools profile, so the profile
//! sets them; no test presses a key. Titles are placeholders.

use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use voxxa_lib::discovery::{presenter_process, DiscoverySource};
use voxxa_lib::listen::Event;
use voxxa_lib::{
    drive, gui_sink, make_controller, step, Action, Capabilities, CtlError, KeystrokeProfile,
    PresentationController, PresenterConfig, PresenterKind, Sent,
};

struct Fake {
    caps: Capabilities,
    stops: bool,
    calls: Mutex<Vec<&'static str>>,
    /// Presenter is not in front: next and prev refuse with the guard's
    /// sentence, as the real driver does, and press nothing.
    behind: AtomicBool,
}

impl Fake {
    /// Capabilities and song-end flag of the real keystroke driver with `profile`.
    async fn like(profile: KeystrokeProfile) -> Self {
        let mut real = make_controller(PresenterKind::Keystroke);
        let cfg = PresenterConfig {
            keystroke_profile: Some(profile),
            ..Default::default()
        };
        real.connect(cfg).await.unwrap();
        Self {
            caps: real.capabilities(),
            stops: real.stops_at_song_end(),
            calls: Mutex::new(Vec::new()),
            behind: AtomicBool::new(false),
        }
    }
    fn log(&self, call: &'static str) -> Result<(), CtlError> {
        if self.behind.load(Ordering::SeqCst) && matches!(call, "next" | "prev") {
            return Err(CtlError::Other(FRONT.into()));
        }
        self.calls.lock().unwrap().push(call);
        Ok(())
    }
    fn take(&self) -> Vec<&'static str> {
        std::mem::take(&mut *self.calls.lock().unwrap())
    }
}

#[async_trait]
impl PresentationController for Fake {
    fn kind(&self) -> PresenterKind {
        PresenterKind::Keystroke
    }
    fn capabilities(&self) -> Capabilities {
        self.caps
    }
    fn stops_at_song_end(&self) -> bool {
        self.stops
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
        self.log("next")
    }
    async fn prev_slide(&self) -> Result<(), CtlError> {
        self.log("prev")
    }
    async fn blank(&self) -> Result<(), CtlError> {
        self.log("blank")
    }
    async fn unblank(&self) -> Result<(), CtlError> {
        self.log("unblank")
    }
}

async fn worship() -> Fake {
    Fake::like(KeystrokeProfile::WorshipToolsPresenter).await
}

/// Song A: 4 slides (global 0-3), Song B: 4 (4-7), Song C: 5 (8-12).
fn goto(song: usize, slide: usize) -> Action {
    let (offset, title) = [(0, "Song A"), (4, "Song B"), (8, "Song C")][song];
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

fn put_live(title: &str) -> String {
    format!("Put \"{title}\" live in Presenter.")
}

/// The guard's sentence (`presenters/keystroke.rs`, FRONT_MSG).
const FRONT: &str = "Bring WorshipTools Presenter to the front.";

#[test]
fn presenter_is_found_by_its_process() {
    let running = |paths: &[&str]| -> Vec<String> { paths.iter().map(|p| p.to_string()).collect() };
    let hit = presenter_process(&running(&[
        r"C:\Windows\explorer.exe",
        r"C:\Program Files\Presenter\Presenter.exe",
    ]))
    .expect("Presenter is running");
    assert_eq!(hit.kind, PresenterKind::Keystroke);
    assert_eq!(
        hit.keystroke_profile,
        Some(KeystrokeProfile::WorshipToolsPresenter)
    );
    assert_eq!(hit.source, DiscoverySource::Process);
    assert_eq!(hit.name, "WorshipTools Presenter");
    assert_eq!((hit.host.as_str(), hit.port), ("", 0), "no host or port");
    // What the GUI reads (first-run step 2 and Advanced → Auto-detect).
    let json = serde_json::to_value(&hit).unwrap();
    assert_eq!(json["kind"], "keystroke");
    assert_eq!(json["keystroke_profile"], "worship_tools_presenter");
    assert_eq!(json["source"], "process");
    // Not running, or only a look-alike name: nothing found.
    assert!(presenter_process(&[]).is_none());
    assert!(presenter_process(&running(&[
        r"C:\Program Files\Presenter\PresenterUpdate.exe",
        r"C:\Program Files\Voxxa\voxxa.exe",
    ]))
    .is_none());
}

#[tokio::test]
async fn the_profile_sets_the_flags() {
    let wt = worship().await;
    assert!(wt.stops && !wt.caps.can_blank);
    for profile in [KeystrokeProfile::Universal, KeystrokeProfile::PowerPoint] {
        let other = Fake::like(profile).await;
        assert!(!other.stops && other.caps.can_blank, "{profile:?}");
    }
}

#[tokio::test]
async fn worshiptools_never_steps_across_a_song() {
    let p = worship().await;
    let mut sent = Sent::default();
    // Inside Song A: steps as before.
    auto(&p, &mut sent, goto(0, 0)).await.unwrap();
    auto(&p, &mut sent, goto(0, 1)).await.unwrap();
    auto(&p, &mut sent, goto(0, 3)).await.unwrap();
    assert_eq!(p.take(), ["next", "next", "next"]);
    // Song A's last slide -> Song B: no Right (T2), the amber ask, and the
    // screen is recorded at Song B's first slide.
    let r = auto(&p, &mut sent, goto(1, 0)).await;
    assert_eq!(r.unwrap_err().to_string(), put_live("Song B"));
    assert_eq!(p.take(), Vec::<&str>::new(), "no key across a song");
    assert_eq!((sent.global, sent.song), (Some(4), Some(1)));
    // The next step inside Song B starts from its first slide.
    auto(&p, &mut sent, goto(1, 2)).await.unwrap();
    assert_eq!(p.take(), ["next", "next"]);
    // A change that lands mid-song: still no key; the first slide is recorded.
    let r = auto(&p, &mut sent, goto(2, 3)).await;
    assert_eq!(r.unwrap_err().to_string(), put_live("Song C"));
    assert_eq!((sent.global, sent.song), (Some(8), Some(2)));
    auto(&p, &mut sent, goto(2, 3)).await.unwrap();
    assert_eq!(p.take(), ["next", "next", "next"]);
    // A jump back to an earlier song, by the operator's song chip too.
    let r = drive(&p, &mut sent, &goto(0, 2), false, true).await;
    assert_eq!(r.unwrap_err().to_string(), put_live("Song A"));
    assert_eq!(p.take(), Vec::<&str>::new());
    assert_eq!((sent.global, sent.song), (Some(0), Some(0)));
}

/// `commands::dispatch` tells the GUI (`slide-advanced`) when `drive` is Ok
/// or moved the song or slide in `sent`. The song-change ask moves it, so the
/// stage and the song strip follow the new song (the conductor's change and
/// the operator's song chip); the guard's refusal does not, so they stay with
/// the screen. tests/gui/e2e.mjs section 9 checks what the GUI then shows.
#[tokio::test]
async fn the_song_change_ask_moves_sent_and_the_guards_refusal_does_not() {
    let p = worship().await;
    let mut sent = Sent::default();
    auto(&p, &mut sent, goto(0, 0)).await.unwrap();
    auto(&p, &mut sent, goto(0, 1)).await.unwrap();
    p.take();
    // Presenter behind another window: refused, nothing pressed, nothing moved.
    p.behind.store(true, Ordering::SeqCst);
    let before = sent;
    let r = auto(&p, &mut sent, goto(0, 2)).await;
    assert_eq!(r.unwrap_err().to_string(), FRONT);
    assert_eq!(sent, before, "no slide-advanced for a refusal");
    // A song change presses no key, so it asks even then, and moves `sent`.
    let r = auto(&p, &mut sent, goto(1, 0)).await;
    assert_eq!(r.unwrap_err().to_string(), put_live("Song B"));
    assert_eq!(
        (sent.global, sent.song),
        (Some(4), Some(1)),
        "slide-advanced"
    );
    // The operator's song chip: the same.
    p.behind.store(false, Ordering::SeqCst);
    let r = drive(&p, &mut sent, &goto(2, 0), false, true).await;
    assert_eq!(r.unwrap_err().to_string(), put_live("Song C"));
    assert_eq!(
        (sent.global, sent.song),
        (Some(8), Some(2)),
        "slide-advanced"
    );
    assert_eq!(p.take(), Vec::<&str>::new());
}

#[tokio::test]
async fn the_operators_raw_next_and_prev_are_unchanged() {
    // Q9W changes no raw key: the operator's Next at a song's end is pressed.
    let p = worship().await;
    let mut sent = Sent::default();
    auto(&p, &mut sent, goto(0, 0)).await.unwrap();
    auto(&p, &mut sent, goto(0, 3)).await.unwrap();
    p.take();
    step(&p, &mut sent, 1, false).await.unwrap();
    step(&p, &mut sent, -1, false).await.unwrap();
    assert_eq!(p.take(), ["next", "prev"]);
    assert_eq!(sent.global, Some(3));
}

#[tokio::test]
async fn other_keystroke_profiles_still_step_across_songs() {
    let p = Fake::like(KeystrokeProfile::Universal).await;
    let mut sent = Sent::default();
    auto(&p, &mut sent, goto(0, 3)).await.unwrap();
    auto(&p, &mut sent, goto(1, 1)).await.unwrap();
    assert_eq!(p.take(), ["next", "next"]);
    assert_eq!((sent.global, sent.song), (Some(5), Some(1)));
}

#[tokio::test]
async fn a_blank_worshiptools_cannot_do_is_skipped_or_refused() {
    let p = worship().await;
    let mut sent = Sent::default();
    auto(&p, &mut sent, goto(0, 0)).await.unwrap();
    // The conductor's blank and unblank (a prayer): nothing pressed, no error.
    auto(&p, &mut sent, Action::Blank).await.unwrap();
    auto(&p, &mut sent, Action::Unblank).await.unwrap();
    assert!(!sent.blank);
    // The operator's blank (button, tray, HTTP): "not supported", even while
    // Voxxa has the focus, and nothing pressed.
    for focused in [false, true] {
        let r = drive(&p, &mut sent, &Action::Blank, focused, true).await;
        assert!(matches!(r, Err(CtlError::Unsupported("blank"))), "{r:?}");
        assert!(r.unwrap_err().to_string().contains("not supported"));
    }
    // The next slide is one Right, with no blank key before it.
    auto(&p, &mut sent, goto(0, 1)).await.unwrap();
    assert_eq!(p.take(), ["next"]);
}

#[test]
fn the_light_gets_no_red_error_at_a_prayer_and_the_amber_ask_at_a_song_change() {
    // The GUI's sink, as `commands::start_listening_with_state` wires it: a failed send is a
    // `presenter-error` event, which Simple mode shows red unless it is one of
    // ADR-019's amber sentences (tests/gui).
    let rt = tokio::runtime::Runtime::new().unwrap();
    let p = rt.block_on(worship());
    let mut sent = Sent::default();
    let mut shown = Vec::new();
    for action in [goto(0, 3), Action::Blank, Action::Unblank, goto(1, 0)] {
        gui_sink(
            Event::Action(action, Some(13)),
            &mut |name, payload| shown.push((name, payload)),
            &mut |a, _, operator| {
                rt.block_on(drive(&p, &mut sent, &a, false, operator))
                    .map_err(|e| e.to_string())
            },
        );
    }
    let ask = serde_json::to_string(&put_live("Song B")).unwrap();
    assert_eq!(shown, [("presenter-error", ask)], "only the song change");
    assert_eq!(p.take(), Vec::<&str>::new());
}
