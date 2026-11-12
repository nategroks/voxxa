//! The decision of what reaches the audience screen: turns one conductor
//! [`Action`] into presenter calls. Pure over the [`PresentationController`]
//! trait plus [`Sent`], so tests drive it with fake presenters
//! (`tests/dispatch.rs`); `commands::dispatch` wires it to the app.

use crate::aligner::Action;
use crate::presenters::{CtlError, PresentationController, PresenterKind};
use std::time::Duration;

/// What Voxxa last sent to the presenter. Voxxa has sent nothing at launch,
/// so it starts "no slide, not blanked".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sent {
    /// Setlist-global index of the last slide sent; -1 when none yet.
    pub global: i64,
    /// Song of that slide. Goto drivers address slides inside one presentation
    /// (one song), so a song change needs the right presentation selected.
    pub song: Option<usize>,
    /// Voxxa last sent a blank. Toggle drivers depend on this being right.
    pub blank: bool,
}

impl Default for Sent {
    fn default() -> Self {
        Self {
            global: -1,
            song: None,
            blank: false,
        }
    }
}

pub const FOCUS_MSG: &str = "focus your presentation app: Voxxa's own window has the keyboard";

/// Drivers whose unblank presses the same toggle as blank.
fn is_toggle(kind: PresenterKind) -> bool {
    matches!(kind, PresenterKind::Keystroke | PresenterKind::OpenSong)
}

/// The keystroke driver types into the focused window. When that is Voxxa,
/// the keys would land in Voxxa's own shortcuts and loop, so refuse.
pub fn refuse_if_focused(
    p: &dyn PresentationController,
    voxxa_focused: bool,
) -> Result<(), CtlError> {
    if voxxa_focused && p.kind() == PresenterKind::Keystroke {
        return Err(CtlError::Other(FOCUS_MSG.into()));
    }
    Ok(())
}

/// Blank or unblank. A toggle driver is only pressed when the wanted state
/// differs from what Voxxa last sent; other drivers are idempotent.
async fn set_blank(
    p: &dyn PresentationController,
    sent: &mut Sent,
    want: bool,
    voxxa_focused: bool,
) -> Result<(), CtlError> {
    if sent.blank == want && is_toggle(p.kind()) {
        return Ok(());
    }
    refuse_if_focused(p, voxxa_focused)?;
    if want {
        p.blank().await?;
    } else {
        p.unblank().await?;
    }
    sent.blank = want;
    Ok(())
}

/// Is `title` the presentation the app has selected? Drivers that cannot
/// report it trust an operator action (the operator can see the app).
async fn presentation_is(p: &dyn PresentationController, title: &str, operator: bool) -> bool {
    if !p.capabilities().can_query_state {
        return operator;
    }
    let norm = |s: &str| -> String {
        s.chars()
            .filter(|c| c.is_alphanumeric())
            .flat_map(char::to_lowercase)
            .collect()
    };
    // A subtitle in brackets ("Amazing Grace (My Chains Are Gone)", as
    // Planning Center lists it) is often not in the app's name.
    let want = norm(title.split('(').next().unwrap_or(title));
    match p.current_state().await.map(|s| s.presentation_name) {
        Ok(Some(name)) => {
            let have = norm(&name);
            // The app's name may add to the title ("Be Thou My Vision (Irish)"),
            // never drop from it: "Grace" is not "Amazing Grace".
            !want.is_empty() && have.contains(&want)
        }
        _ => false,
    }
}

/// Send one conductor action. `operator` marks button, tray, HTTP and
/// song-chip actions; conductor actions pass false.
pub async fn drive(
    p: &dyn PresentationController,
    sent: &mut Sent,
    action: &Action,
    voxxa_focused: bool,
    operator: bool,
) -> Result<(), CtlError> {
    let (song, slide, target, title) = match action {
        Action::Noop => return Ok(()),
        Action::Blank | Action::Unblank => {
            let want = matches!(action, Action::Blank);
            return set_blank(p, sent, want, voxxa_focused).await;
        }
        Action::Goto {
            song_index,
            slide_in_song,
            global_index,
            song_title,
            ..
        } => (
            *song_index,
            *slide_in_song as u32,
            *global_index as i64,
            song_title,
        ),
    };
    // Focus is checked only where a key is really pressed: a Goto that sends
    // nothing must still record where the screen is.
    // A toggle driver unblanks before it steps: in PowerPoint and Keynote any
    // key ends the black, so a step first and then the toggle would black out
    // the verse. Idempotent drivers unblank after the goto (ProPresenter's
    // unblank re-triggers the current slide).
    let unblank_first = is_toggle(p.kind()) && sent.blank;
    if p.capabilities().can_goto_slide {
        // Goto drivers address the slide inside the selected presentation.
        let new_song = sent.song != Some(song);
        if new_song && !presentation_is(p, title, operator).await {
            let _ = set_blank(p, sent, true, voxxa_focused).await;
            return Err(CtlError::Other(format!(
                "select \"{title}\" in your presentation app"
            )));
        }
        if unblank_first {
            set_blank(p, sent, false, voxxa_focused).await?;
        }
        if new_song || target != sent.global {
            p.goto_slide(slide).await?;
        }
        sent.song = Some(song);
    } else if sent.global >= 0 {
        // Keystroke-style drivers step through the setlist with next/prev.
        // With nothing sent yet the position is unknown, so nothing is pressed.
        let delta = target - sent.global;
        if delta != 0 {
            refuse_if_focused(p, voxxa_focused)?;
        }
        if unblank_first {
            set_blank(p, sent, false, voxxa_focused).await?;
        }
        for i in 0..delta.unsigned_abs() {
            if i > 0 {
                tokio::time::sleep(Duration::from_millis(60)).await;
            }
            if delta > 0 {
                p.next_slide().await?;
            } else {
                p.prev_slide().await?;
            }
        }
    }
    sent.global = target;
    if sent.blank {
        set_blank(p, sent, false, voxxa_focused).await?;
    }
    Ok(())
}
