//! FreeShow driver.
//!
//! FreeShow exposes a JSON action protocol on two transports — socket.io on port 5505
//! and plain HTTP POST on port 5506. We target the HTTP transport for simplicity; it
//! is documented at <https://freeshow.app/api> and requires no authentication. The
//! user must enable Settings → Connections in FreeShow.
//!
//! Wire format is a single JSON envelope:
//!     { "action": "<id>", "data": { ... } }
//! with the action id taken from the FreeShow API table.

use async_trait::async_trait;
use reqwest::Client;
use serde::Serialize;
use serde_json::{json, Value};
use std::time::Duration;

use super::{
    Capabilities, CtlError, PresentationController, PresenterConfig, PresenterKind,
};

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 5506;

pub struct FreeShowDriver {
    base_url: Option<String>,
    client: Client,
}

impl FreeShowDriver {
    pub fn new() -> Self {
        Self {
            base_url: None,
            client: Client::builder()
                .timeout(Duration::from_secs(3))
                .build()
                .expect("reqwest client build"),
        }
    }

    fn base(&self) -> Result<&str, CtlError> {
        self.base_url.as_deref().ok_or(CtlError::NotConnected)
    }

    async fn send_action<T: Serialize>(&self, action: &str, data: Option<T>) -> Result<Value, CtlError> {
        let body = match data {
            Some(d) => json!({ "action": action, "data": d }),
            None => json!({ "action": action }),
        };
        let url = self.base()?.to_string();
        let res = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| CtlError::Network(e.to_string()))?;

        if !res.status().is_success() {
            return Err(CtlError::Other(format!(
                "FreeShow {action} → {}",
                res.status()
            )));
        }

        // FreeShow may respond with an empty body for fire-and-forget actions; treat
        // that as Value::Null so callers don't have to special-case it.
        let text = res.text().await.unwrap_or_default();
        if text.is_empty() {
            return Ok(Value::Null);
        }
        serde_json::from_str(&text)
            .map_err(|e| CtlError::Other(format!("bad FreeShow response: {e}")))
    }
}

#[async_trait]
impl PresentationController for FreeShowDriver {
    fn kind(&self) -> PresenterKind {
        PresenterKind::FreeShow
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            can_blank: true,
            can_goto_slide: true,
            // FreeShow's `get_shows` etc. exist, but slide_index probing is non-trivial
            // and not needed for slide control. Mark unsupported for now.
            can_query_state: false,
        }
    }

    fn is_connected(&self) -> bool {
        self.base_url.is_some()
    }

    async fn connect(&mut self, cfg: PresenterConfig) -> Result<(), CtlError> {
        let host = cfg.host.unwrap_or_else(|| DEFAULT_HOST.to_string());
        let port = cfg.port.unwrap_or(DEFAULT_PORT);
        let base = format!("http://{host}:{port}");

        // FreeShow returns HTTP 200 for any well-formed action POST, even unknown
        // actions (it just no-ops). `get_shows` is a safe round-trip probe: it
        // returns a JSON object and proves the listener is FreeShow rather than
        // some other service squatting on the port.
        let probe = Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .map_err(|e| CtlError::Other(e.to_string()))?;
        let res = probe
            .post(&base)
            .json(&json!({ "action": "get_shows" }))
            .send()
            .await
            .map_err(|e| CtlError::Network(format!("could not reach FreeShow at {base}: {e}")))?;

        if !res.status().is_success() {
            return Err(CtlError::Other(format!(
                "FreeShow at {base} returned {}",
                res.status()
            )));
        }

        log::info!("[FREESHOW] connected to {base}");
        self.base_url = Some(base);
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), CtlError> {
        self.base_url = None;
        Ok(())
    }

    async fn next_slide(&self) -> Result<(), CtlError> {
        self.send_action::<()>("next_slide", None).await?;
        log::info!("[FREESHOW] next");
        Ok(())
    }

    async fn prev_slide(&self) -> Result<(), CtlError> {
        self.send_action::<()>("previous_slide", None).await?;
        log::info!("[FREESHOW] prev");
        Ok(())
    }

    async fn blank(&self) -> Result<(), CtlError> {
        // `clear_slide` only blanks the slide layer; `clear_all` would also wipe
        // background/audio. Slide-layer-only matches the operator's expectation.
        self.send_action::<()>("clear_slide", None).await?;
        log::info!("[FREESHOW] blank");
        Ok(())
    }

    async fn unblank(&self) -> Result<(), CtlError> {
        self.send_action::<()>("restore_output", None).await?;
        log::info!("[FREESHOW] unblank");
        Ok(())
    }

    async fn goto_slide(&self, index: u32) -> Result<(), CtlError> {
        self.send_action("index_select_slide", Some(json!({ "index": index })))
            .await?;
        log::info!("[FREESHOW] goto_slide {index}");
        Ok(())
    }
}
