//! OpenLP Web Remote API v2 driver.
//!
//! Spec: <https://openlp.org/manuals/3.1/web-remote/>
//! Source: <https://gitlab.com/openlp/openlp/-/blob/master/openlp/core/api/versions/v2/controller.py>
//!
//! REST API on port 4316 (state-push WebSocket on 4317 is not used by this driver —
//! we just call REST). Authentication is optional; when enabled in OpenLP's Remote
//! settings the requests need HTTP Basic Auth.
//!
//! Linux caveat from the OpenLP 3.0+ release notes:
//!   "OpenLP at present does not behave well under Wayland so the recommendation is
//!    to run under X11."
//! The driver itself doesn't care — it's pure HTTP — but the operator setup matters.

use async_trait::async_trait;
use base64::Engine;
use reqwest::{Client, RequestBuilder};
use serde::Deserialize;
use std::time::Duration;

use super::{
    Capabilities, CtlError, PresentationController, PresenterConfig, PresenterKind,
    PresenterState,
};

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 4316;

pub struct OpenLpV2Driver {
    base_url: Option<String>,
    auth_header: Option<String>,
    client: Client,
}

impl OpenLpV2Driver {
    pub fn new() -> Self {
        Self {
            base_url: None,
            auth_header: None,
            client: Client::builder()
                .timeout(Duration::from_secs(3))
                .build()
                .expect("reqwest client build"),
        }
    }

    fn base(&self) -> Result<&str, CtlError> {
        self.base_url.as_deref().ok_or(CtlError::NotConnected)
    }

    fn auth(&self, req: RequestBuilder) -> RequestBuilder {
        match &self.auth_header {
            Some(h) => req.header("Authorization", h),
            None => req,
        }
    }

    async fn post(&self, path: &str) -> Result<reqwest::Response, CtlError> {
        let url = format!("{}{}", self.base()?, path);
        crate::net_stats::record_request();
        let res = self
            .auth(self.client.post(&url))
            .send()
            .await
            .map_err(|e| CtlError::Network(e.to_string()))?;
        let status = res.status();
        if status.is_success() {
            return Ok(res);
        }
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(CtlError::AuthRequired);
        }
        Err(CtlError::Other(format!("{path} → {status}")))
    }

    async fn get(&self, path: &str) -> Result<reqwest::Response, CtlError> {
        let url = format!("{}{}", self.base()?, path);
        crate::net_stats::record_request();
        let res = self
            .auth(self.client.get(&url))
            .send()
            .await
            .map_err(|e| CtlError::Network(e.to_string()))?;
        let status = res.status();
        if status.is_success() {
            return Ok(res);
        }
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(CtlError::AuthRequired);
        }
        Err(CtlError::Other(format!("{path} → {status}")))
    }
}

/// `GET /api/v2/core/poll` returns the live controller state.
#[derive(Debug, Deserialize)]
struct PollEnvelope {
    results: PollResults,
}

#[derive(Debug, Deserialize, Default)]
struct PollResults {
    #[serde(default)]
    slide: Option<u32>,
    #[serde(default)]
    item: Option<String>,
    #[serde(default)]
    blank: Option<bool>,
}

fn basic_auth_header(user: &str, pass: &str) -> String {
    let token = base64::engine::general_purpose::STANDARD.encode(format!("{user}:{pass}"));
    format!("Basic {token}")
}

#[async_trait]
impl PresentationController for OpenLpV2Driver {
    fn kind(&self) -> PresenterKind {
        PresenterKind::OpenLpV2
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities::FULL
    }

    fn is_connected(&self) -> bool {
        self.base_url.is_some()
    }

    async fn connect(&mut self, cfg: PresenterConfig) -> Result<(), CtlError> {
        let host = cfg.host.unwrap_or_else(|| DEFAULT_HOST.to_string());
        let port = cfg.port.unwrap_or(DEFAULT_PORT);
        let base = format!("http://{host}:{port}");

        // If a password is supplied, default the username to "openlp" — that's the
        // documented default in OpenLP's Remote settings. The user can override via
        // PresenterConfig::username.
        let auth_header = match cfg.password.as_deref() {
            Some(p) if !p.is_empty() => Some(basic_auth_header(
                cfg.username.as_deref().unwrap_or("openlp"),
                p,
            )),
            _ => None,
        };

        // Probe with /api/v2/core/poll — it's a cheap GET that requires the same
        // auth as the controller endpoints, so a 401 here means the credentials
        // need fixing before we accept the connection.
        let probe = Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .map_err(|e| CtlError::Other(e.to_string()))?;
        let mut req = probe.get(format!("{base}/api/v2/core/poll"));
        if let Some(h) = &auth_header {
            req = req.header("Authorization", h);
        }
        crate::net_stats::record_request();
        let res = req
            .send()
            .await
            .map_err(|e| CtlError::Network(format!("could not reach OpenLP at {base}: {e}")))?;
        let status = res.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(CtlError::AuthRequired);
        }
        if !status.is_success() {
            return Err(CtlError::Other(format!(
                "OpenLP at {base} returned {status}"
            )));
        }

        log::info!("[OPENLP] connected to {base}");
        self.base_url = Some(base);
        self.auth_header = auth_header;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), CtlError> {
        self.base_url = None;
        self.auth_header = None;
        Ok(())
    }

    async fn next_slide(&self) -> Result<(), CtlError> {
        self.post("/api/v2/controller/live/next").await?;
        log::info!("[OPENLP] next");
        Ok(())
    }

    async fn prev_slide(&self) -> Result<(), CtlError> {
        self.post("/api/v2/controller/live/previous").await?;
        log::info!("[OPENLP] prev");
        Ok(())
    }

    async fn blank(&self) -> Result<(), CtlError> {
        self.post("/api/v2/controller/blank").await?;
        log::info!("[OPENLP] blank");
        Ok(())
    }

    async fn unblank(&self) -> Result<(), CtlError> {
        self.post("/api/v2/controller/show").await?;
        log::info!("[OPENLP] unblank");
        Ok(())
    }

    async fn current_state(&self) -> Result<PresenterState, CtlError> {
        let res = self.get("/api/v2/core/poll").await?;
        let body = res
            .text()
            .await
            .map_err(|e| CtlError::Network(e.to_string()))?;
        let env: PollEnvelope = serde_json::from_str(&body)
            .map_err(|e| CtlError::Other(format!("bad /core/poll body: {e}")))?;
        Ok(PresenterState {
            slide_index: env.results.slide,
            presentation_name: env.results.item,
            is_blank: env.results.blank,
        })
    }

    // goto_slide intentionally left at the trait default (Unsupported). OpenLP's exact
    // jump-to-slide endpoint path varies across point releases and the plan calls for
    // verifying it against the live OpenLP source before shipping. Until then, the
    // smart-blanking state machine should fall through to next/prev sequencing.
}
