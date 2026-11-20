//! Keystroke-emulation driver. Sends synthetic key presses to whatever window has focus.
//!
//! This is the universal fallback for presentation apps without an API
//! (EasyWorship, MediaShout, SongShow Plus, WorshipTools Presenter, VideoPsalm)
//! and for users who don't want to enable the network APIs on supported apps.
//!
//! The receiving app's window must be in the foreground. Only the WorshipTools
//! Presenter profile checks that the front window is the app's (ADR-019 § 2);
//! for every other profile the caller can only warn.

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

/// Synthetic keys would land in Voxxa: one of its webviews has focus, or the
/// foreground window belongs to Voxxa's own process. The second covers the
/// tray menu's hidden window, which takes the foreground while the menu is
/// open (PT2-02).
pub fn own_window_focused(foreground_pid: u32, webview_focused: bool) -> bool {
    webview_focused || foreground_pid == std::process::id()
}

/// What the WorshipTools guard returns when Presenter is not in front (ADR-019 § 2).
/// The status light shows it amber, word for word.
pub const FRONT_MSG: &str = "Bring WorshipTools Presenter to the front.";

/// Is this executable WorshipTools Presenter? Test card T1: the process is
/// `Presenter` (`C:\Program Files\Presenter\Presenter.exe`). Accepts the bare
/// process name or a full path, in any case.
pub fn is_presenter_exe(path: &str) -> bool {
    let name = path.rsplit(['\\', '/']).next().unwrap_or(path);
    name.eq_ignore_ascii_case("Presenter.exe") || name.eq_ignore_ascii_case("Presenter")
}

/// May a key be sent to the front window? Always, except with the WorshipTools
/// profile, whose keys go only to a window of Presenter's process (ADR-019 § 2, D2).
pub fn keys_allowed(profile: KeystrokeProfile, front_exe: Option<&str>) -> bool {
    profile != KeystrokeProfile::WorshipToolsPresenter || front_exe.is_some_and(is_presenter_exe)
}

/// The executable path of a running process, None when it cannot be read.
#[cfg(windows)]
pub fn process_path(pid: u32) -> Option<String> {
    use std::ffi::c_void;
    #[link(name = "kernel32")]
    extern "system" {
        fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut c_void;
        fn QueryFullProcessImageNameW(
            h: *mut c_void,
            flags: u32,
            buf: *mut u16,
            len: *mut u32,
        ) -> i32;
        fn CloseHandle(h: *mut c_void) -> i32;
    }
    const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
    let mut buf = [0u16; 1024];
    let mut len = buf.len() as u32;
    // SAFETY: the handle is checked and closed; the call writes at most `len`
    // u16s into `buf` and sets `len` to the count written. Read-only access:
    // nothing is sent to the process.
    let ok = unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if h.is_null() {
            return None;
        }
        let ok = QueryFullProcessImageNameW(h, 0, buf.as_mut_ptr(), &mut len);
        CloseHandle(h);
        ok
    };
    (ok != 0).then(|| String::from_utf16_lossy(&buf[..len as usize]))
}

/// The executable paths of every running process this user may query.
#[cfg(windows)]
pub fn process_paths() -> Vec<String> {
    #[link(name = "kernel32")]
    extern "system" {
        fn K32EnumProcesses(pids: *mut u32, bytes: u32, needed: *mut u32) -> i32;
    }
    let mut pids = vec![0u32; 4096];
    let mut needed = 0u32;
    let bytes = (pids.len() * 4) as u32;
    // SAFETY: the call fills at most `bytes` of `pids` and reports how many it wrote.
    if unsafe { K32EnumProcesses(pids.as_mut_ptr(), bytes, &mut needed) } == 0 {
        return Vec::new();
    }
    pids.truncate(needed as usize / 4);
    pids.into_iter().filter_map(process_path).collect()
}

/// Off Windows no process is looked up: discovery finds nothing by process.
#[cfg(not(windows))]
pub fn process_path(_pid: u32) -> Option<String> {
    None
}

#[cfg(not(windows))]
pub fn process_paths() -> Vec<String> {
    Vec::new()
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
            // ADR-019 § 4: "." does nothing in Presenter (T4). F10 clears the lyrics
            // (T3), but how to bring them back is untested and unblank() presses
            // the blank key again, so F10 is only the candidate: no blank key yet.
            // Up, Down, numpad +/- change the service item and F11/F12 clear media:
            // never sent.
            Self::WorshipToolsPresenter => KeyMap {
                next: Key::RightArrow,
                prev: Key::LeftArrow,
                blank: None,
            },
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

/// The WorshipTools guard as the driver applies it: Ok when a key may go to
/// the front window, else the amber sentence (ADR-019 § 2).
pub fn guard(profile: KeystrokeProfile, front_exe: Option<&str>) -> Result<(), CtlError> {
    keys_allowed(profile, front_exe)
        .then_some(())
        .ok_or_else(|| CtlError::Other(FRONT_MSG.into()))
}

/// The executable of the front window's process.
fn front_exe() -> Option<String> {
    process_path(foreground_pid())
}

/// Presses one key in the front window.
fn enigo_send(key: Key) -> Result<(), CtlError> {
    // A fresh Enigo per call sidesteps Send/Sync issues with the cached handle on
    // platforms where `Enigo` is not Send.
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| CtlError::Other(format!("enigo init: {e:?}")))?;
    enigo
        .key(key, enigo::Direction::Click)
        .map_err(|e| CtlError::Other(format!("enigo key: {e:?}")))
}

pub struct KeystrokeDriver {
    profile: KeystrokeProfile,
    /// Keystroke driver is "connected" from the moment a profile is chosen — there's no
    /// real connection to maintain.
    connected: bool,
    /// The front window's executable and the key sender. Tests swap both, so
    /// a test of tap() never reaches enigo or a real window.
    front: fn() -> Option<String>,
    send: fn(Key) -> Result<(), CtlError>,
}

impl KeystrokeDriver {
    pub fn new() -> Self {
        Self {
            profile: KeystrokeProfile::default(),
            connected: false,
            front: front_exe,
            send: enigo_send,
        }
    }

    fn tap(&self, key: Key) -> Result<(), CtlError> {
        // shortcut: off Windows the front process is unknown and the guard passes
        // (ADR-019 amendment); the own-window guard in dispatch still applies.
        if cfg!(windows) {
            guard(self.profile, (self.front)().as_deref())?;
        }
        (self.send)(key)
    }
}

#[async_trait]
impl PresentationController for KeystrokeDriver {
    fn kind(&self) -> PresenterKind {
        PresenterKind::Keystroke
    }

    fn capabilities(&self) -> Capabilities {
        // A profile without a blank key cannot blank (ADR-019 § 4).
        Capabilities {
            can_blank: self.profile.keymap().blank.is_some(),
            ..Capabilities::KEYSTROKE
        }
    }

    fn stops_at_song_end(&self) -> bool {
        self.profile == KeystrokeProfile::WorshipToolsPresenter
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

    // ADR-019. No test here presses a key: every driver below has a fake front
    // window and a recording sender, so tap() never reaches enigo.
    use super::*;
    use std::cell::RefCell;
    const PRESENTER: &str = r"C:\Program Files\Presenter\Presenter.exe";

    thread_local! {
        static KEYS: RefCell<Vec<Key>> = const { RefCell::new(Vec::new()) };
    }
    fn record(key: Key) -> Result<(), CtlError> {
        KEYS.with_borrow_mut(|k| k.push(key));
        Ok(())
    }
    fn recorded() -> Vec<Key> {
        KEYS.with_borrow_mut(std::mem::take)
    }
    fn presenter_in_front() -> Option<String> {
        Some(PRESENTER.into())
    }
    fn explorer_in_front() -> Option<String> {
        Some(r"C:\Windows\explorer.exe".into())
    }
    const ALL: [KeystrokeProfile; 12] = {
        use KeystrokeProfile::*;
        [
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
        ]
    };

    fn driver(profile: KeystrokeProfile) -> KeystrokeDriver {
        KeystrokeDriver {
            profile,
            connected: true,
            front: presenter_in_front,
            send: record,
        }
    }

    #[tokio::test]
    async fn the_driver_asks_the_guard_before_every_key() {
        use KeystrokeProfile::*;
        let behind = KeystrokeDriver {
            front: explorer_in_front,
            ..driver(WorshipToolsPresenter)
        };
        // Off Windows the front process is unknown and the guard passes.
        for r in [behind.next_slide().await, behind.prev_slide().await] {
            if cfg!(windows) {
                assert_eq!(r.unwrap_err().to_string(), FRONT_MSG);
            } else {
                r.unwrap();
            }
        }
        let none: &[Key] = if cfg!(windows) {
            &[]
        } else {
            &[Key::RightArrow, Key::LeftArrow]
        };
        assert_eq!(recorded(), none, "no key while Presenter is behind");
        // Presenter in front: Right, then Left.
        let d = driver(WorshipToolsPresenter);
        d.next_slide().await.unwrap();
        d.prev_slide().await.unwrap();
        assert_eq!(recorded(), [Key::RightArrow, Key::LeftArrow]);
        // Another profile is not checked.
        let other = KeystrokeDriver {
            front: explorer_in_front,
            ..driver(Universal)
        };
        other.next_slide().await.unwrap();
        assert_eq!(recorded(), [Key::RightArrow]);
        assert_eq!(guard(WorshipToolsPresenter, Some(PRESENTER)).ok(), Some(()));
    }

    #[test]
    fn presenter_is_known_by_its_process_name_or_path() {
        for hit in [
            PRESENTER,
            r"D:\apps\presenter\PRESENTER.EXE",
            "Presenter.exe",
            "Presenter",
        ] {
            assert!(is_presenter_exe(hit), "{hit}");
        }
        for miss in [
            r"C:\Program Files\Presenter\PresenterUpdate.exe",
            r"C:\Windows\explorer.exe",
            r"C:\Presenter\Voxxa.exe",
            r"C:\Program Files\Renewed Vision\ProPresenter\ProPresenter.exe",
            "ProPresenter",
            "",
        ] {
            assert!(!is_presenter_exe(miss), "{miss}");
        }
    }

    #[test]
    fn worshiptools_keys_go_only_to_presenter_in_front() {
        let wt = KeystrokeProfile::WorshipToolsPresenter;
        assert!(keys_allowed(wt, Some(PRESENTER)));
        assert!(!keys_allowed(wt, Some(r"C:\Program Files\Voxxa\voxxa.exe")));
        assert!(
            !keys_allowed(wt, None),
            "an unreadable front window is not Presenter"
        );
        assert_eq!(FRONT_MSG, "Bring WorshipTools Presenter to the front.");
        // The guard covers this profile only (ADR-019 § 2).
        for p in ALL.into_iter().filter(|&p| p != wt) {
            assert!(
                keys_allowed(p, None) && keys_allowed(p, Some(r"C:\x\other.exe")),
                "{p:?}"
            );
        }
    }

    #[test]
    fn the_worshiptools_key_map_is_right_left_and_no_blank() {
        let km = KeystrokeProfile::WorshipToolsPresenter.keymap();
        assert_eq!(
            (km.next, km.prev, km.blank),
            (Key::RightArrow, Key::LeftArrow, None)
        );
        // Up/Down and numpad +/- change the service item; F10-F12 clear lyrics or media.
        #[allow(unused_mut)]
        let mut never = vec![
            Key::F10,
            Key::F11,
            Key::F12,
            Key::UpArrow,
            Key::DownArrow,
            Key::Unicode('+'),
            Key::Unicode('-'),
            Key::Unicode('.'),
        ];
        #[cfg(windows)]
        never.extend([Key::Add, Key::Subtract]);
        for k in [Some(km.next), Some(km.prev), km.blank]
            .into_iter()
            .flatten()
        {
            assert!(!never.contains(&k), "{k:?}");
        }
    }

    #[test]
    fn other_profiles_keep_their_keys() {
        use KeystrokeProfile::*;
        for p in ALL.into_iter().filter(|&p| p != WorshipToolsPresenter) {
            let km = p.keymap();
            let blank = match p {
                EasyWorship => Key::F1,
                PowerPoint | Keynote => Key::Unicode('b'),
                VideoPsalm => Key::F5,
                _ => Key::Unicode('.'),
            };
            assert_eq!(
                (km.next, km.prev, km.blank),
                (Key::RightArrow, Key::LeftArrow, Some(blank))
            );
        }
    }

    #[tokio::test]
    async fn can_blank_and_the_song_end_stop_come_from_the_profile() {
        for p in ALL {
            let d = driver(p);
            let wt = p == KeystrokeProfile::WorshipToolsPresenter;
            assert_eq!(d.capabilities().can_blank, !wt, "{p:?}");
            assert_eq!(d.stops_at_song_end(), wt, "{p:?}");
            assert!(!d.capabilities().can_goto_slide && !d.capabilities().can_query_state);
        }
        // The operator's blank: "not supported", and no key is pressed (the
        // sender only records, so even a regression could press nothing real).
        let r = driver(KeystrokeProfile::WorshipToolsPresenter)
            .blank()
            .await;
        assert!(matches!(r, Err(CtlError::Unsupported("blank"))), "{r:?}");
        assert_eq!(recorded(), Vec::<Key>::new());
    }
}
