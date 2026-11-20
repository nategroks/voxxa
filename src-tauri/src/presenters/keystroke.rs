//! Keystroke-emulation driver. Sends synthetic key presses to whatever window has focus.
//!
//! This is the universal fallback for presentation apps without an API
//! (EasyWorship, MediaShout, SongShow Plus, WorshipTools Presenter, VideoPsalm)
//! and for users who don't want to enable the network APIs on supported apps.
//!
//! The receiving app's window must be in the foreground. Only the WorshipTools
//! Presenter profile checks that the foreground app is the target (ADR-019 § 2).

use async_trait::async_trait;
use enigo::{Enigo, Key, Keyboard, Settings};
use serde::{Deserialize, Serialize};

use super::{
    Capabilities, CtlError, PresentationController, PresenterConfig, PresenterKind, PresenterState,
};

/// The process id of the foreground window's owner, 0 when there is none.
#[cfg(windows)]
pub fn foreground_pid() -> u32 {
    #[link(name = "user32")]
    extern "system" {
        fn GetForegroundWindow() -> *mut std::ffi::c_void;
        fn GetWindowThreadProcessId(hwnd: *mut std::ffi::c_void, pid: *mut u32) -> u32;
    }
    let mut pid = 0;
    // SAFETY: both calls only read; a null foreground window leaves pid 0.
    unsafe { GetWindowThreadProcessId(GetForegroundWindow(), &mut pid) };
    pid
}

/// Off Windows the caller's webview focus check stands alone.
#[cfg(not(windows))]
pub fn foreground_pid() -> u32 {
    0
}

/// Running processes as (pid, exe name), from a Toolhelp snapshot: no process
/// is opened, so WorshipTools Presenter is only listed, never touched.
#[cfg(windows)]
pub fn processes() -> Vec<(u32, String)> {
    use std::ffi::c_void;
    #[repr(C)]
    struct Entry {
        size: u32,
        usage: u32,
        pid: u32,
        heap: usize,
        module: u32,
        threads: u32,
        parent: u32,
        priority: i32,
        flags: u32,
        exe: [u16; 260],
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn CreateToolhelp32Snapshot(flags: u32, pid: u32) -> *mut c_void;
        fn Process32FirstW(snap: *mut c_void, entry: *mut Entry) -> i32;
        fn Process32NextW(snap: *mut c_void, entry: *mut Entry) -> i32;
        fn CloseHandle(handle: *mut c_void) -> i32;
    }
    let mut out = Vec::new();
    // SAFETY: TH32CS_SNAPPROCESS (2) snapshots the process list; `size` is set
    // as the API requires, and the snapshot handle is closed.
    unsafe {
        let snap = CreateToolhelp32Snapshot(2, 0);
        if snap as isize == -1 {
            return out;
        }
        let mut e: Entry = std::mem::zeroed();
        e.size = std::mem::size_of::<Entry>() as u32;
        let mut ok = Process32FirstW(snap, &mut e);
        while ok != 0 {
            let len = e.exe.iter().position(|&c| c == 0).unwrap_or(e.exe.len());
            out.push((e.pid, String::from_utf16_lossy(&e.exe[..len])));
            ok = Process32NextW(snap, &mut e);
        }
        CloseHandle(snap);
    }
    out
}

/// Off Windows nothing is found by process (ADR-019 § 1).
#[cfg(not(windows))]
pub fn processes() -> Vec<(u32, String)> {
    Vec::new()
}

/// WorshipTools Presenter's process (T1: `Presenter`, from
/// `C:\Program Files\Presenter\Presenter.exe`): its name, exe name or path.
pub fn is_presenter(exe: &str) -> bool {
    let name = exe.rsplit(['\\', '/']).next().unwrap_or(exe);
    name.eq_ignore_ascii_case("Presenter.exe") || name.eq_ignore_ascii_case("Presenter")
}

/// The foreground guard's sentence, shown amber word for word (ADR-019 § 2).
pub const FRONT_MSG: &str = "Bring WorshipTools Presenter to the front.";

/// The foreground window belongs to a WorshipTools Presenter process.
pub fn presenter_in_front(foreground_pid: u32, processes: &[(u32, String)]) -> bool {
    processes
        .iter()
        .any(|(pid, exe)| *pid == foreground_pid && is_presenter(exe))
}

/// Synthetic keys would land in Voxxa: one of its webviews has focus, or the
/// foreground window belongs to Voxxa's own process. The second covers the
/// tray menu's hidden window, which takes the foreground while the menu is
/// open (PT2-02).
pub fn own_window_focused(foreground_pid: u32, webview_focused: bool) -> bool {
    webview_focused || foreground_pid == std::process::id()
}

/// Per-app key profile from the §2 table of the project plan.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeystrokeProfile {
    /// Right / Left / Period — works for ProPresenter, FreeShow, OpenLP, Proclaim, and most apps.
    #[default]
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
            | Self::Proclaim
            | Self::FreeShow => KeyMap {
                next: Key::RightArrow,
                prev: Key::LeftArrow,
                blank: Some(Key::Unicode('.')),
            },
            // ADR-019 § 4: `.` does nothing in Presenter (T4). F10 clears its lyrics
            // but the way back is untested, so it is only the candidate: no blank.
            Self::WorshipToolsPresenter => KeyMap {
                next: Key::RightArrow,
                prev: Key::LeftArrow,
                blank: None,
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
        // shortcut: off Windows the foreground app is unknown and keys go out as
        // before; add a macOS check if a Mac church pilots Presenter.
        if cfg!(windows)
            && self.profile == KeystrokeProfile::WorshipToolsPresenter
            && !presenter_in_front(foreground_pid(), &processes())
        {
            return Err(CtlError::Other(FRONT_MSG.into()));
        }
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
        Capabilities {
            can_blank: self.profile.keymap().blank.is_some(),
            ..Capabilities::KEYSTROKE
        }
    }

    fn song_change_ask(&self, title: &str) -> Option<String> {
        (self.profile == KeystrokeProfile::WorshipToolsPresenter)
            .then(|| format!("Put \"{title}\" live in Presenter."))
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn connect(&mut self, cfg: PresenterConfig) -> Result<(), CtlError> {
        self.profile = cfg.keystroke_profile.unwrap_or_default();
        self.connected = true;
        log::info!("[KEYSTROKE] active, profile={:?}", self.profile);
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

#[cfg(test)]
mod tests {
    #[test]
    fn a_foreground_window_of_our_own_process_counts_as_voxxa() {
        use super::own_window_focused;
        assert!(own_window_focused(std::process::id(), false));
        assert!(own_window_focused(0, true));
        assert!(!own_window_focused(0, false));
    }

    #[test]
    fn presenter_is_known_by_its_process_name_or_path() {
        use super::is_presenter;
        assert!(is_presenter(r"C:\Program Files\Presenter\Presenter.exe"));
        assert!(is_presenter("Presenter.exe"));
        assert!(is_presenter("PRESENTER.EXE"));
        assert!(is_presenter("Presenter"));
        for other in ["ProPresenter.exe", "Presenter.exe.bak", "explorer.exe", ""] {
            assert!(!is_presenter(other), "{other}");
        }
    }

    #[test]
    fn keys_go_to_presenter_only_when_it_is_in_front() {
        use super::presenter_in_front;
        let procs = [
            (10, "explorer.exe".to_string()),
            (20, "Presenter.exe".to_string()),
        ];
        assert!(presenter_in_front(20, &procs));
        assert!(!presenter_in_front(10, &procs), "another app in front");
        assert!(!presenter_in_front(0, &procs), "no foreground window");
        assert!(!presenter_in_front(20, &[]), "Presenter not running");
    }

    /// No key is pressed here: only the key maps, capabilities and a blank
    /// that fails before any key.
    #[tokio::test]
    async fn worshiptools_gets_right_and_left_and_no_blank() {
        use super::*;
        let km = KeystrokeProfile::WorshipToolsPresenter.keymap();
        assert_eq!(
            (km.next, km.prev, km.blank),
            (Key::RightArrow, Key::LeftArrow, None)
        );
        // F10-F12 clear Presenter's screen; Up, Down and numpad +/- change the service item.
        #[allow(unused_mut)]
        let mut never = vec![Key::F10, Key::F11, Key::F12, Key::UpArrow, Key::DownArrow];
        #[cfg(windows)]
        never.extend([Key::Add, Key::Subtract]);
        for k in [km.next, km.prev].into_iter().chain(km.blank) {
            assert!(!never.contains(&k), "{k:?}");
        }
        let cfg = |p| PresenterConfig {
            keystroke_profile: Some(p),
            ..Default::default()
        };
        let mut d = KeystrokeDriver::new();
        d.connect(cfg(KeystrokeProfile::WorshipToolsPresenter))
            .await
            .unwrap();
        assert!(!d.capabilities().can_blank);
        assert_eq!(
            d.song_change_ask("Song A").as_deref(),
            Some("Put \"Song A\" live in Presenter.")
        );
        let r = d.blank().await;
        assert!(matches!(r, Err(CtlError::Unsupported("blank"))), "{r:?}");
        // Every other profile keeps its keys, its blank and its song steps.
        use KeystrokeProfile::*;
        for p in [
            Universal,
            ProPresenter,
            EasyWorship,
            OpenLp,
            MediaShout,
            SongShowPlus,
            PowerPoint,
            Keynote,
            VideoPsalm,
            Proclaim,
            FreeShow,
        ] {
            let km = p.keymap();
            assert_eq!((km.next, km.prev), (Key::RightArrow, Key::LeftArrow));
            assert!(km.blank.is_some(), "{p:?}");
            d.connect(cfg(p)).await.unwrap();
            assert!(
                d.capabilities().can_blank && d.song_change_ask("Song A").is_none(),
                "{p:?}"
            );
        }
        assert_eq!(Universal.keymap().blank, Some(Key::Unicode('.')));
    }
}
