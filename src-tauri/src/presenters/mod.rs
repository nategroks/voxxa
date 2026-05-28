//! Presentation software drivers.
//!
//! Every concrete driver (keystroke fallback, ProPresenter REST, etc.) implements
//! the [`PresentationController`] trait. The rest of the app — VAD, alignment,
//! Tauri commands — only ever sees the trait, so swapping presenters at runtime
//! is a single `connect_presenter` call.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

pub mod freeshow;
pub mod keystroke;
pub mod openlp_v2;
pub mod propresenter7_rest;

pub use freeshow::FreeShowDriver;
pub use keystroke::{KeystrokeDriver, KeystrokeProfile};
pub use openlp_v2::OpenLpV2Driver;
pub use propresenter7_rest::ProPresenter7RestDriver;

/// Identifier for a driver implementation. Stable across releases — settings persist this.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresenterKind {
    /// Universal arrow-key sender, optionally tuned per app.
    Keystroke,
    /// ProPresenter 7.9+ official HTTP REST API.
    ProPresenter7Rest,
    /// FreeShow JSON action protocol over HTTP (port 5506) or socket.io (port 5505).
    FreeShow,
    /// OpenLP v2 web API (REST on 4316, optional Basic Auth).
    OpenLpV2,
}

impl PresenterKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Keystroke => "Keystroke (any app)",
            Self::ProPresenter7Rest => "ProPresenter 7.9+ (REST API)",
            Self::FreeShow => "FreeShow",
            Self::OpenLpV2 => "OpenLP (Web Remote v2)",
        }
    }

    pub fn all() -> &'static [PresenterKind] {
        &[
            Self::Keystroke,
            Self::ProPresenter7Rest,
            Self::FreeShow,
            Self::OpenLpV2,
        ]
    }
}

/// Configuration envelope passed to `connect`. Each driver picks the field(s) it needs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PresenterConfig {
    /// Keystroke driver: which per-app key profile to use.
    pub keystroke_profile: Option<KeystrokeProfile>,
    /// HTTP drivers: hostname or IP. Defaults to `127.0.0.1`.
    pub host: Option<String>,
    /// HTTP drivers: TCP port. ProPresenter default `1025`.
    pub port: Option<u16>,
    /// Drivers that need a username (OpenLP Basic Auth).
    pub username: Option<String>,
    /// Drivers that need a password/token.
    pub password: Option<String>,
}

/// Static capabilities a driver advertises so the UI can hide buttons it cannot fulfill.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Capabilities {
    pub can_blank: bool,
    pub can_goto_slide: bool,
    pub can_query_state: bool,
}

impl Capabilities {
    pub const KEYSTROKE: Capabilities = Capabilities {
        can_blank: true,
        can_goto_slide: false,
        can_query_state: false,
    };

    pub const FULL: Capabilities = Capabilities {
        can_blank: true,
        can_goto_slide: true,
        can_query_state: true,
    };
}

/// Snapshot of what the receiving app is doing right now. Drivers without query support
/// return `None` for fields they can't observe.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PresenterState {
    pub slide_index: Option<u32>,
    pub presentation_name: Option<String>,
    pub is_blank: Option<bool>,
}

#[derive(Debug)]
pub enum CtlError {
    NotConnected,
    Unsupported(&'static str),
    Network(String),
    /// The receiving machine has the app open but Network/Remote is disabled.
    RemoteDisabled,
    AuthRequired,
    Other(String),
}

impl fmt::Display for CtlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotConnected => write!(f, "presenter not connected"),
            Self::Unsupported(op) => write!(f, "operation not supported by this driver: {op}"),
            Self::Network(e) => write!(f, "network error: {e}"),
            Self::RemoteDisabled => write!(
                f,
                "remote/network control is disabled in the presentation app"
            ),
            Self::AuthRequired => write!(f, "authentication required"),
            Self::Other(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for CtlError {}

#[async_trait]
pub trait PresentationController: Send + Sync {
    fn kind(&self) -> PresenterKind;
    fn capabilities(&self) -> Capabilities;
    fn is_connected(&self) -> bool;

    async fn connect(&mut self, cfg: PresenterConfig) -> Result<(), CtlError>;
    async fn disconnect(&mut self) -> Result<(), CtlError>;

    async fn next_slide(&self) -> Result<(), CtlError>;
    async fn prev_slide(&self) -> Result<(), CtlError>;

    async fn blank(&self) -> Result<(), CtlError> {
        Err(CtlError::Unsupported("blank"))
    }
    async fn unblank(&self) -> Result<(), CtlError> {
        Err(CtlError::Unsupported("unblank"))
    }
    async fn goto_slide(&self, _index: u32) -> Result<(), CtlError> {
        Err(CtlError::Unsupported("goto_slide"))
    }
    async fn current_state(&self) -> Result<PresenterState, CtlError> {
        Err(CtlError::Unsupported("current_state"))
    }
}

/// Build a fresh, disconnected driver of the requested kind.
pub fn make_controller(kind: PresenterKind) -> Box<dyn PresentationController> {
    match kind {
        PresenterKind::Keystroke => Box::new(KeystrokeDriver::new()),
        PresenterKind::ProPresenter7Rest => Box::new(ProPresenter7RestDriver::new()),
        PresenterKind::FreeShow => Box::new(FreeShowDriver::new()),
        PresenterKind::OpenLpV2 => Box::new(OpenLpV2Driver::new()),
    }
}
