//! ProPresenter 7.9+ official REST driver.
//!
//! Spec: <https://openapi.propresenter.com>
//!
//! The port is user-configurable in ProPresenter → Settings → Network. Common values
//! are 1025 (the OpenAPI doc example), 50001, and 8080. No authentication on localhost;
//! the receiving machine just has to have "Enable Network" turned on, otherwise every
//! call returns 404 with the body "The receiving machine does not have Network enabled
//! or the requested path was not found."

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

use super::{
    Capabilities, CtlError, PresentationController, PresenterConfig, PresenterKind,
    PresenterState,
};

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 1025;
/// The exact body ProPresenter returns when Network is off. We use this to give the
/// user a useful error instead of a generic 404.
const NETWORK_DISABLED_MARKER: &str = "does not have Network enabled";

pub struct ProPresenter7RestDriver {
    base_url: Option<String>,
    client: Client,
}

impl ProPresenter7RestDriver {
    pub fn new() -> Self {
        Self {
            base_url: None,
            client: Client::builder()
                // ProPresenter normally responds in < 50 ms on loopback; a hard ceiling
                // keeps a slide advance from blocking the audio thread on a stalled host.
                .timeout(Duration::from_secs(3))
                .build()
                .expect("reqwest client build"),
        }
    }

    fn base(&self) -> Result<&str, CtlError> {
        self.base_url.as_deref().ok_or(CtlError::NotConnected)
    }

    async fn get(&self, path: &str) -> Result<reqwest::Response, CtlError> {
        let url = format!("{}{}", self.base()?, path);
        let res = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| CtlError::Network(e.to_string()))?;

        let status = res.status();
        if status.is_success() {
            return Ok(res);
        }

        // 404 has a special meaning when the body matches the marker — Network is off.
        if status == reqwest::StatusCode::NOT_FOUND {
            let body = res.text().await.unwrap_or_default();
            if body.contains(NETWORK_DISABLED_MARKER) {
                return Err(CtlError::RemoteDisabled);
            }
            return Err(CtlError::Other(format!("404 from {}: {}", path, body)));
        }

        Err(CtlError::Other(format!("{} → {}", path, status)))
    }
}

#[derive(Debug, Deserialize)]
struct VersionResponse {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    version: Option<String>,
}

/// `GET /v1/presentation/slide_index` returns:
///   `{ "presentation_index": { ..., "index": N }, "slide_index": M }`
/// Fields can be absent depending on Pro7 build, so everything is optional.
#[derive(Debug, Deserialize)]
struct SlideIndexResponse {
    slide_index: Option<u32>,
    presentation_index: Option<PresentationIndexFragment>,
}

#[derive(Debug, Deserialize)]
struct PresentationIndexFragment {
    presentation_id: Option<PresentationIdFragment>,
}

#[derive(Debug, Deserialize)]
struct PresentationIdFragment {
    name: Option<String>,
}

#[async_trait]
impl PresentationController for ProPresenter7RestDriver {
    fn kind(&self) -> PresenterKind {
        PresenterKind::ProPresenter7Rest
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

        // Use a short-timeout probe rather than the shared client so a wrong port fails fast.
        let probe = Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .map_err(|e| CtlError::Other(e.to_string()))?;

        let url = format!("{base}/v1/version");
        let res = probe
            .get(&url)
            .send()
            .await
            .map_err(|e| CtlError::Network(format!("could not reach {url}: {e}")))?;

        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        if !status.is_success() {
            if body.contains(NETWORK_DISABLED_MARKER) {
                return Err(CtlError::RemoteDisabled);
            }
            return Err(CtlError::Other(format!(
                "ProPresenter at {base} returned {status}"
            )));
        }

        // Parse the version to confirm we're really talking to ProPresenter, not some
        // random service squatting on the port.
        let version: VersionResponse = serde_json::from_str(&body)
            .map_err(|e| CtlError::Other(format!("bad /v1/version response: {e}")))?;
        log::info!(
            "[PRO7-REST] connected to {} {} at {}",
            version.name.as_deref().unwrap_or("ProPresenter"),
            version.version.as_deref().unwrap_or("?"),
            base
        );

        self.base_url = Some(base);
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), CtlError> {
        self.base_url = None;
        Ok(())
    }

    async fn next_slide(&self) -> Result<(), CtlError> {
        self.get("/v1/trigger/next").await?;
        log::info!("[PRO7-REST] next");
        Ok(())
    }

    async fn prev_slide(&self) -> Result<(), CtlError> {
        self.get("/v1/trigger/previous").await?;
        log::info!("[PRO7-REST] prev");
        Ok(())
    }

    async fn blank(&self) -> Result<(), CtlError> {
        // Clears only the slide layer, leaving stage display / props alone.
        self.get("/v1/clear/layer/slide").await?;
        log::info!("[PRO7-REST] blank");
        Ok(())
    }

    async fn unblank(&self) -> Result<(), CtlError> {
        // REST API has no direct "unblank" — retriggering the current slide brings the
        // slide layer back at its original index.
        let state = self.current_state().await?;
        let idx = state
            .slide_index
            .ok_or_else(|| CtlError::Other("no active presentation to unblank".into()))?;
        self.goto_slide(idx).await?;
        log::info!("[PRO7-REST] unblank (re-triggered slide {idx})");
        Ok(())
    }

    async fn goto_slide(&self, index: u32) -> Result<(), CtlError> {
        let path = format!("/v1/presentation/active/{index}/trigger");
        self.get(&path).await?;
        log::info!("[PRO7-REST] goto_slide {index}");
        Ok(())
    }

    async fn current_state(&self) -> Result<PresenterState, CtlError> {
        let res = self.get("/v1/presentation/slide_index").await?;
        let body = res
            .text()
            .await
            .map_err(|e| CtlError::Network(e.to_string()))?;
        let parsed: SlideIndexResponse = serde_json::from_str(&body)
            .map_err(|e| CtlError::Other(format!("bad slide_index response: {e}")))?;

        Ok(PresenterState {
            slide_index: parsed.slide_index,
            presentation_name: parsed
                .presentation_index
                .and_then(|p| p.presentation_id)
                .and_then(|p| p.name),
            is_blank: None,
        })
    }
}
