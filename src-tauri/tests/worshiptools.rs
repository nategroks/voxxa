//! Q9W, ADR-019: WorshipTools Presenter. Discovery by process (a fake
//! process list), the blank each song opens on (#183) and the missing blank key, all
//! through `voxxa_lib::drive` with a fake presenter that records every call.
//! The fake takes its capabilities and its song-blank flag from the real
//! keystroke driver connected with the WorshipTools profile, so the profile
//! sets them; no test presses a key. Titles are placeholders.

use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use voxxa_lib::discovery::{
    discover_with, merge, presenter_process, DiscoveredService, DiscoverySource,
};
use voxxa_lib::listen::Event;
use voxxa_lib::{
    drive, gui_sink, make_controller, step, tell_gui, Action, Capabilities, CtlError,
    KeystrokeProfile, PresentationController, PresenterConfig, PresenterKind, Sent,
};

struct Fake {
    caps: Capabilities,
    blanks: bool,
    calls: Mutex<Vec<&'static str>>,
    /// Presenter is not in front: next and prev refuse with the guard's
    /// sentence, as the real driver does, and press nothing.
    behind: AtomicBool,
}

impl Fake {
    /// Capabilities and song-blank flag of the real keystroke driver with `profile`.
    async fn like(profile: KeystrokeProfile) -> Self {
        let mut real = make_controller(PresenterKind::Keystroke);
        let cfg = PresenterConfig {
            keystroke_profile: Some(profile),
            ..Default::default()
        };
        real.connect(cfg).await.unwrap();
        Self {
            caps: real.capabilities(),
            blanks: real.songs_open_on_blank(),
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
    fn songs_open_on_blank(&self) -> bool {
        self.blanks
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
    // The bare process name or a full path, in any case.
    for other in [
        r"D:\apps\presenter\PRESENTER.EXE",
        "Presenter.exe",
        "Presenter",
    ] {
        assert!(presenter_process(&running(&[other])).is_some(), "{other}");
    }
    // Not running, or only a look-alike name: nothing found.
    assert!(presenter_process(&[]).is_none());
    assert!(presenter_process(&running(&[
        r"C:\Program Files\Presenter\PresenterUpdate.exe",
        r"C:\Program Files\Voxxa\voxxa.exe",
        r"C:\Presenter\Voxxa.exe",
        r"C:\Windows\explorer.exe",
        r"C:\Program Files\Renewed Vision\ProPresenter\ProPresenter.exe",
        "ProPresenter",
        "",
    ]))
    .is_none());
}

#[test]
fn discovery_keeps_the_process_hit_beside_the_network_hits() {
    let net = |kind, port, source| DiscoveredService {
        kind,
        keystroke_profile: None,
        host: "127.0.0.1".into(),
        port,
        name: "App on 127.0.0.1".into(),
        source,
    };
    let pro7 = |source| net(PresenterKind::ProPresenter7Rest, 1025, source);
    let process = presenter_process(&["Presenter".to_string()]);
    // ProPresenter seen twice (this computer and mDNS) is listed once.
    let all = merge(
        vec![
            pro7(DiscoverySource::Localhost),
            net(PresenterKind::OpenLpV2, 4316, DiscoverySource::Localhost),
        ],
        vec![pro7(DiscoverySource::Mdns)],
        process,
    );
    let found: Vec<_> = all.iter().map(|s| (s.kind, s.source)).collect();
    assert_eq!(found.len(), 3, "{found:?}");
    let wt: Vec<_> = all
        .iter()
        .filter(|s| s.keystroke_profile == Some(KeystrokeProfile::WorshipToolsPresenter))
        .collect();
    assert_eq!(wt.len(), 1, "{found:?}");
    assert_eq!(
        (wt[0].kind, wt[0].source),
        (PresenterKind::Keystroke, DiscoverySource::Process)
    );
    // Presenter alone (first-run step 2 then uses it at once).
    let alone = merge(
        Vec::new(),
        Vec::new(),
        presenter_process(&["Presenter".to_string()]),
    );
    assert_eq!(alone.len(), 1);
    assert_eq!(alone[0].source, DiscoverySource::Process);
}

/// The whole discovery run (what discover_all runs, with a fake process list):
/// the process strategy is wired in beside the network probes. Whatever the
/// network probes find on this computer, the process hit is there exactly
/// when Presenter is in the list.
#[tokio::test]
async fn discovery_runs_the_process_strategy() {
    let short = std::time::Duration::from_millis(50);
    let processes = |all: &[DiscoveredService]| -> Vec<_> {
        all.iter()
            .filter(|s| s.source == DiscoverySource::Process)
            .map(|s| (s.kind, s.keystroke_profile))
            .collect()
    };
    let with = discover_with(short, || {
        vec![r"C:\Program Files\Presenter\Presenter.exe".into()]
    })
    .await;
    assert_eq!(
        processes(&with),
        [(
            PresenterKind::Keystroke,
            Some(KeystrokeProfile::WorshipToolsPresenter)
        )]
    );
    let without = discover_with(short, || vec![r"C:\Windows\explorer.exe".into()]).await;
    assert!(processes(&without).is_empty());
}

#[tokio::test]
async fn the_profile_sets_the_flags() {
    let wt = worship().await;
    assert!(wt.blanks && !wt.caps.can_blank);
    for profile in [KeystrokeProfile::Universal, KeystrokeProfile::PowerPoint] {
        let other = Fake::like(profile).await;
        assert!(!other.blanks && other.caps.can_blank, "{profile:?}");
    }
}

/// #183: in Presenter every song opens on a blank slide (Presenter's slide 1),
/// a song is put live on it, and Right on a song's last slide shows the next
/// song's blank (founder, LHMEZPC01, 2026-11-24). Voxxa's setlist has no
/// blanks, so dispatch counts them.
#[tokio::test]
async fn worshiptools_counts_the_blank_each_song_opens_on() {
    let p = worship().await;
    let mut sent = Sent::default();
    // The first song is live on its blank: one Right for its first lyrics.
    auto(&p, &mut sent, goto(0, 0)).await.unwrap();
    assert_eq!(p.take(), ["next"]);
    // Inside Song A: one Right per slide, as before.
    auto(&p, &mut sent, goto(0, 1)).await.unwrap();
    auto(&p, &mut sent, goto(0, 3)).await.unwrap();
    assert_eq!(p.take(), ["next", "next", "next"]);
    // Song A's last slide -> Song B's first: Right to Song B's blank, Right to
    // its first lyrics. No ask: Voxxa runs the song change itself.
    auto(&p, &mut sent, goto(1, 0)).await.unwrap();
    assert_eq!(p.take(), ["next", "next"]);
    assert_eq!((sent.global, sent.song), (Some(4), Some(1)));
    auto(&p, &mut sent, goto(1, 2)).await.unwrap();
    assert_eq!(p.take(), ["next", "next"]);
    // Song B's slide 2 -> Song C's slide 3: B's last slide, C's blank, C 0-3.
    auto(&p, &mut sent, goto(2, 3)).await.unwrap();
    assert_eq!(p.take(), ["next"; 6]);
    assert_eq!((sent.global, sent.song), (Some(11), Some(2)));
    // Back to an earlier song (the operator's song chip too): keys can't get
    // there, so nothing is pressed, the light asks, and Song A's blank is
    // recorded: the operator puts it live on its blank.
    let r = drive(&p, &mut sent, &goto(0, 2), false, true).await;
    assert_eq!(r.unwrap_err().to_string(), put_live("Song A"));
    assert_eq!(p.take(), Vec::<&str>::new());
    assert_eq!((sent.global, sent.song), (Some(-1), Some(0)));
    // The next line sung in Song A: from its blank to its slide 2.
    auto(&p, &mut sent, goto(0, 2)).await.unwrap();
    assert_eq!(p.take(), ["next"; 3]);
    // Past a song (Song A -> Song C): the same ask, nothing pressed.
    let r = auto(&p, &mut sent, goto(2, 0)).await;
    assert_eq!(r.unwrap_err().to_string(), put_live("Song C"));
    assert_eq!(p.take(), Vec::<&str>::new());
    assert_eq!((sent.global, sent.song), (Some(7), Some(2)));
    // Voxxa started while a later song is live and already sung: from that
    // song's blank, so Song B's slide 2 is three Rights.
    let mut fresh = Sent::default();
    auto(&p, &mut fresh, goto(1, 2)).await.unwrap();
    assert_eq!(p.take(), ["next"; 3]);
    // The operator's raw Right on a song's last slide reaches the next song's
    // blank, and the count still holds: then one Right for its first lyrics.
    let mut sent = Sent::default();
    auto(&p, &mut sent, goto(0, 3)).await.unwrap();
    step(&p, &mut sent, 1, false).await.unwrap();
    p.take();
    auto(&p, &mut sent, goto(1, 0)).await.unwrap();
    assert_eq!(p.take(), ["next"]);
}

/// `commands::dispatch` tells the GUI (`slide-advanced`) when `tell_gui` says
/// so: `drive` was Ok, or it moved the song or slide in `sent`. The song-change
/// ask moves it, so the stage and the song strip follow the new song (the
/// conductor's change and the operator's song chip); the guard's refusal does
/// not, so they stay with the screen. tests/gui/e2e.mjs section 9 checks what
/// the GUI then shows.
#[tokio::test]
async fn the_song_change_ask_tells_the_gui_and_the_guards_refusal_does_not() {
    let p = worship().await;
    let mut sent = Sent::default();
    // Drive as commands::dispatch does; true when it would emit slide-advanced.
    async fn told(p: &Fake, sent: &mut Sent, action: Action, operator: bool) -> (String, bool) {
        let before = *sent;
        let r = drive(p, sent, &action, false, operator).await;
        let tell = tell_gui(r.is_ok(), before, *sent);
        (r.err().map(|e| e.to_string()).unwrap_or_default(), tell)
    }
    assert_eq!(
        told(&p, &mut sent, goto(0, 0), false).await,
        (String::new(), true)
    );
    assert_eq!(
        told(&p, &mut sent, goto(0, 1), false).await,
        (String::new(), true)
    );
    p.take();
    // Presenter behind another window: refused, nothing pressed, nothing moved,
    // inside a song and at the step into the next song.
    p.behind.store(true, Ordering::SeqCst);
    let before = sent;
    for action in [goto(0, 2), goto(1, 0)] {
        let r = told(&p, &mut sent, action, false).await;
        assert_eq!(
            r,
            (FRONT.to_string(), false),
            "no slide-advanced for a refusal"
        );
        assert_eq!(sent, before);
    }
    // A song change keys can't make (past Song B) presses no key, so it asks
    // even then, and moves `sent` to Song C's blank.
    let r = told(&p, &mut sent, goto(2, 0), false).await;
    assert_eq!(r, (put_live("Song C"), true), "slide-advanced");
    assert_eq!((sent.global, sent.song), (Some(7), Some(2)));
    // The operator's song chip back to Song A: the same.
    p.behind.store(false, Ordering::SeqCst);
    let r = told(&p, &mut sent, goto(0, 0), true).await;
    assert_eq!(r, (put_live("Song A"), true), "slide-advanced");
    assert_eq!((sent.global, sent.song), (Some(-1), Some(0)));
    assert_eq!(p.take(), Vec::<&str>::new());
    // A refusal that changed only the blank (a goto driver's "select …"
    // blanks first) tells nothing.
    let blanked = Sent {
        blank: true,
        ..sent
    };
    assert!(!tell_gui(false, sent, blanked));
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
    p.take();
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
fn the_light_gets_no_red_error_at_a_prayer_and_the_amber_ask_only_going_back() {
    // The GUI's sink, as `commands::start_listening_with_state` wires it: a failed send is a
    // `presenter-error` event, which Simple mode shows red unless it is one of
    // ADR-019's amber sentences (tests/gui).
    let rt = tokio::runtime::Runtime::new().unwrap();
    let p = rt.block_on(worship());
    let mut sent = Sent::default();
    let mut shown = Vec::new();
    for action in [
        goto(0, 3),
        Action::Blank,
        Action::Unblank,
        goto(1, 0),
        goto(0, 0),
    ] {
        gui_sink(
            Event::Action(action, Some(13)),
            &mut |name, payload| shown.push((name, payload)),
            &mut |a, _, operator| {
                rt.block_on(drive(&p, &mut sent, &a, false, operator))
                    .map_err(|e| e.to_string())
            },
        );
    }
    // Song A from its blank to slide 3, then into Song B (its blank, slide 0):
    // six Rights and no error. Only the jump back to Song A asks.
    let ask = serde_json::to_string(&put_live("Song A")).unwrap();
    assert_eq!(shown, [("presenter-error", ask)], "only going back");
    assert_eq!(p.take(), ["next"; 6]);
}
