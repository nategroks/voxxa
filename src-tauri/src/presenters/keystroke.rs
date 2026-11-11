//! Keystroke-emulation driver. Sends synthetic key presses to whatever window has focus.
//!
//! This is the universal fallback for presentation apps without an API
//! (EasyWorship, MediaShout, SongShow Plus, WorshipTools Presenter, VideoPsalm)
//! and for users who don't want to enable the network APIs on supported apps.
//!
//! The receiving app's window must be in the foreground; we can't detect that
//! from here, but the caller can warn.

use async_trait::async_trait;
use enigo::{Enigo, Key, Keyboard, Settings};
use serde::{Deserialize, Serialize};

use super::{
    Capabilities, CtlError, PresentationController, PresenterConfig, PresenterKind,
    PresenterState,
};

/// Per-app key profile from the §2 table of the project plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeystrokeProfile {
    /// Right / Left / Period — works for ProPresenter, FreeShow, OpenLP, Proclaim, and most apps.
    Universal,
    ProPresenter,
    EasyWorship,
    OpenLp,
    MediaShout,
    SongShowPlus,
    PowerPoint,
    Keynote,
    VideoPsalm,
    WorshipToolsPresenter,
    Proclaim,
    FreeShow,
}

impl Default for KeystrokeProfile {
    fn default() -> Self {
        Self::Universal
    }
}

struct KeyMap {
    next: Key,
    prev: Key,
    blank: Option<Key>,
}

impl KeystrokeProfile {
    fn keymap(self) -> KeyMap {
        // Defaults to right/left arrow + "." for blank. Per-app overrides match the §2 table;
        // when an app accepts multiple keys for the same action we pick the most universally
        // reliable one (arrow keys over letter keys, since letter keys lose to typed-text focus).
        match self {
            Self::Universal
            | Self::ProPresenter
            | Self::OpenLp
            | Self::MediaShout
            | Self::SongShowPlus
            | Self::WorshipToolsPresenter
            | Self::Proclaim
            | Self::FreeShow => KeyMap {
                next: Key::RightArrow,
                prev: Key::LeftArrow,
                blank: Some(Key::Unicode('.')),
            },
            Self::EasyWorship => KeyMap {
                next: Key::RightArrow,
                prev: Key::LeftArrow,
                blank: Some(Key::F1),
            },
            Self::PowerPoint => KeyMap {
                next: Key::RightArrow,
                prev: Key::LeftArrow,
                blank: Some(Key::Unicode('b')),
            },
            Self::Keynote => KeyMap {
                next: Key::RightArrow,
                prev: Key::LeftArrow,
                blank: Some(Key::Unicode('b')),
            },
            Self::VideoPsalm => KeyMap {
                next: Key::RightArrow,
                prev: Key::LeftArrow,
                blank: Some(Key::F5),
            },
        }
    }
}

pub struct KeystrokeDriver {
    profile: KeystrokeProfile,
    /// Keystroke driver is "connected" from the moment a profile is chosen — there's no
    /// real connection to maintain.
    connected: bool,
}

impl KeystrokeDriver {
    pub fn new() -> Self {
        Self {
            profile: KeystrokeProfile::default(),
            connected: false,
        }
    }

    fn tap(&self, key: Key) -> Result<(), CtlError> {
        // A fresh Enigo per call sidesteps Send/Sync issues with the cached handle on
        // platforms where `Enigo` is not Send.
        let mut enigo = Enigo::new(&Settings::default())
            .map_err(|e| CtlError::Other(format!("enigo init: {e:?}")))?;
        enigo
            .key(key, enigo::Direction::Click)
            .map_err(|e| CtlError::Other(format!("enigo key: {e:?}")))?;
        Ok(())
    }
}

#[async_trait]
impl PresentationController for KeystrokeDriver {
    fn kind(&self) -> PresenterKind {
        PresenterKind::Keystroke
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities::KEYSTROKE
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn connect(&mut self, cfg: PresenterConfig) -> Result<(), CtlError> {
        self.profile = cfg.keystroke_profile.unwrap_or_default();
        self.connected = true;
        log::info!(
            "[KEYSTROKE] active, profile={:?}",
            self.profile
        );
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), CtlError> {
        self.connected = false;
        Ok(())
    }

    async fn next_slide(&self) -> Result<(), CtlError> {
        if !self.connected {
            return Err(CtlError::NotConnected);
        }
        let km = self.profile.keymap();
        self.tap(km.next)?;
        log::info!("[KEYSTROKE] next ({:?})", km.next);
        Ok(())
    }

    async fn prev_slide(&self) -> Result<(), CtlError> {
        if !self.connected {
            return Err(CtlError::NotConnected);
        }
        let km = self.profile.keymap();
        self.tap(km.prev)?;
        log::info!("[KEYSTROKE] prev ({:?})", km.prev);
        Ok(())
    }

    async fn blank(&self) -> Result<(), CtlError> {
        if !self.connected {
            return Err(CtlError::NotConnected);
        }
        let km = self.profile.keymap();
        match km.blank {
            Some(k) => {
                self.tap(k)?;
                log::info!("[KEYSTROKE] blank ({:?})", k);
                Ok(())
            }
            None => Err(CtlError::Unsupported("blank")),
        }
    }

    async fn unblank(&self) -> Result<(), CtlError> {
        // Most worship apps toggle blank with the same key.
        self.blank().await
    }

    async fn current_state(&self) -> Result<PresenterState, CtlError> {
        // We cannot read state from a target we only write keystrokes to.
        Err(CtlError::Unsupported("current_state"))
    }
}
