//! OpenSong driver (REST API on port 8082 by default).
//!
//! Spec: <http://www.opensong.org/d/manual/web_api>
//!
//! OpenSong's Automation API speaks XML responses to GET/POST endpoints. We
//! ignore the WebSocket subscription channel — we only need to write commands,
//! not stream state.
//!
//! Authentication: Optional `?api_key=` URL parameter (or HTTP Basic) when the
//! user has set one in OpenSong → Settings → General → System → Automation API.

use async_trait::async_trait;
use reqwest::{Client, RequestBuilder};
use std::time::Duration;

use super::{
    Capabilities, CtlError, PresentationController, PresenterConfig, PresenterKind,
};

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 8082;

pub struct OpenSongDriver {
    base_url: Option<String>,
    api_key: Option<String>,
    client: Client,
}

impl OpenSongDriver {
    pub fn new() -> Self {
        Self {
            base_url: None,
            api_key: None,
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
        // OpenSong accepts the API key as either a URL query string or a Basic
        // Auth header. We use Basic Auth with username "user" so the secret
        // never sits in URL access logs.
        match &self.api_key {
            Some(k) if !k.is_empty() => req.basic_auth("user", Some(k)),
            _ => req,
        }
    }

    async fn post(&self, path: &str) -> Result<(), CtlError> {
        let url = format!("{}{}", self.base()?, path);
        crate::net_stats::record_request();
        let res = self
            .auth(self.client.post(&url))
            .send()
            .await
            .map_err(|e| CtlError::Network(e.to_string()))?;
        let status = res.status();
        if status.is_success() {
            return Ok(());
        }
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(CtlError::AuthRequired);
        }
        Err(CtlError::Other(format!("{path} → {status}")))
    }
}

#[async_trait]
impl PresentationController for OpenSongDriver {
    fn kind(&self) -> PresenterKind {
        PresenterKind::OpenSong
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            can_blank: true,
            can_goto_slide: true,
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
        self.api_key = cfg.password.clone();

        // Probe `/presentation/status` — OpenSong returns XML on this path. 401
        // means we reached the API but credentials are wrong.
        let probe = Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .map_err(|e| CtlError::Other(e.to_string()))?;
        let mut req = probe.get(format!("{base}/presentation/status"));
        if let Some(k) = &self.api_key {
            if !k.is_empty() {
                req = req.basic_auth("user", Some(k));
            }
        }
        crate::net_stats::record_request();
        let res = req
            .send()
            .await
            .map_err(|e| CtlError::Network(format!("could not reach OpenSong at {base}: {e}")))?;
        let status = res.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(CtlError::AuthRequired);
        }
        // OpenSong returns 200 with an XML body when the API is enabled. Some
        // builds return 404 on /status — accept either as "service present" if
        // we got a TCP reply at all.
        if !status.is_success() && status != reqwest::StatusCode::NOT_FOUND {
            return Err(CtlError::Other(format!(
                "OpenSong at {base} returned {status}"
            )));
        }
        log::info!("[OPENSONG] connected to {base}");
        self.base_url = Some(base);
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), CtlError> {
        self.base_url = None;
        self.api_key = None;
        Ok(())
    }

    async fn next_slide(&self) -> Result<(), CtlError> {
        self.post("/presentation/next").await?;
        log::info!("[OPENSONG] next");
        Ok(())
    }

    async fn prev_slide(&self) -> Result<(), CtlError> {
        self.post("/presentation/previous").await?;
        log::info!("[OPENSONG] prev");
        Ok(())
    }

    async fn blank(&self) -> Result<(), CtlError> {
        self.post("/presentation/black").await?;
        log::info!("[OPENSONG] blank");
        Ok(())
    }

    async fn unblank(&self) -> Result<(), CtlError> {
        // OpenSong's "black" toggles; a second hit unblanks.
        self.post("/presentation/black").await?;
        log::info!("[OPENSONG] unblank");
        Ok(())
    }

    async fn goto_slide(&self, index: u32) -> Result<(), CtlError> {
        // 1-based slide indices on OpenSong's API.
        let one_based = index.saturating_add(1);
        let path = format!("/presentation/slide/{one_based}");
        self.post(&path).await?;
        log::info!("[OPENSONG] goto_slide {index} (1-based={one_based})");
        Ok(())
    }
}
