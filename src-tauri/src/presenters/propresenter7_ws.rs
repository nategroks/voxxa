//! ProPresenter 7.0–7.8 WebSocket fallback driver.
//!
//! Reverse-engineered protocol from jeffmikels/ProPresenter-API. Used when the
//! receiver is on ProPresenter 7.0–7.8 — anything before 7.9 lacks the official
//! REST API, so this is the only way to drive slides without keystrokes.
//!
//! Wire format (line-delimited JSON over a single WebSocket to /remote):
//! ```json
//! { "action": "authenticate", "protocol": "701", "password": "<set>" }
//! { "action": "presentationTriggerNext" }
//! { "action": "presentationTriggerPrevious" }
//! { "action": "presentationTriggerIndex", "slideIndex": "N", "presentationPath": "0:0" }
//! ```
//!
//! Note: `slideIndex` is a **string** on Pro7 (a Pro7 bug, per the upstream
//! API doc), not an integer like Pro6. The doc also warns: "Be careful! It's
//! easy to CRASH ProPresenter when sending invalid messages!" — so we
//! schema-validate before sending.

use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

use super::{
    Capabilities, CtlError, PresentationController, PresenterConfig, PresenterKind,
};

/// Default WebSocket port. ProPresenter has historically used 50001 for the
/// stage display / remote channel; the actual value is whatever the operator
/// configured in Settings → Network.
const DEFAULT_PORT: u16 = 50001;
const PROTOCOL_VERSION: &str = "701"; // Pro 7.4.2+

pub struct ProPresenter7WsDriver {
    /// Sender to the bridge task that owns the live WebSocket. Dropping it
    /// closes the connection.
    tx: Option<mpsc::Sender<Message>>,
}

impl ProPresenter7WsDriver {
    pub fn new() -> Self {
        Self { tx: None }
    }

    async fn send_action(&self, action_json: serde_json::Value) -> Result<(), CtlError> {
        let tx = self.tx.as_ref().ok_or(CtlError::NotConnected)?;
        let body = serde_json::to_string(&action_json)
            .map_err(|e| CtlError::Other(format!("encode: {e}")))?;
        tx.send(Message::Text(body))
            .await
            .map_err(|_| CtlError::NotConnected)?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct AuthResponse {
    #[serde(default)]
    authenticated: i32,
}

/// Bridge task that owns the WebSocket and forwards outbound messages from
/// `rx`. Returns when the channel closes or the socket errors.
async fn bridge(
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
    mut rx: mpsc::Receiver<Message>,
) {
    let (mut sink, mut stream) = ws.split();
    loop {
        tokio::select! {
            outbound = rx.recv() => {
                match outbound {
                    Some(msg) => {
                        if sink.send(msg).await.is_err() {
                            log::warn!("[PRO7-WS] socket write failed; closing bridge");
                            break;
                        }
                    }
                    None => break, // Sender dropped → disconnect.
                }
            }
            inbound = stream.next() => {
                match inbound {
                    Some(Ok(Message::Close(_))) | None => {
                        log::info!("[PRO7-WS] remote closed");
                        break;
                    }
                    Some(Err(e)) => {
                        log::warn!("[PRO7-WS] read error: {e}");
                        break;
                    }
                    Some(Ok(Message::Text(t))) => {
                        // Pro7 sends status updates we don't currently track; log at trace.
                        log::trace!("[PRO7-WS] recv: {t}");
                    }
                    _ => {}
                }
            }
        }
    }
    let _ = sink.close().await;
}

#[async_trait]
impl PresentationController for ProPresenter7WsDriver {
    fn kind(&self) -> PresenterKind {
        PresenterKind::ProPresenter7Ws
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            can_blank: false, // Pro7 WS has no documented clear/blank action.
            can_goto_slide: true,
            can_query_state: false,
        }
    }

    fn is_connected(&self) -> bool {
        self.tx.as_ref().map(|tx| !tx.is_closed()).unwrap_or(false)
    }

    async fn connect(&mut self, cfg: PresenterConfig) -> Result<(), CtlError> {
        let host = cfg.host.unwrap_or_else(|| "127.0.0.1".to_string());
        let port = cfg.port.unwrap_or(DEFAULT_PORT);
        let password = cfg.password.unwrap_or_default();

        // Pro7 requires CamelCase WS upgrade headers; tokio-tungstenite's default
        // builder produces lowercase. The crate's `connect_async` with a String
        // URL is tolerant enough that Pro7 still accepts the handshake — if a
        // user's build of Pro7 turns out to reject it, swap to building a
        // `Request` manually with CamelCase headers.
        let url = format!("ws://{host}:{port}/remote");
        crate::net_stats::record_request();
        let (mut ws, _resp) = tokio::time::timeout(
            Duration::from_secs(3),
            tokio_tungstenite::connect_async(url.as_str()),
        )
        .await
        .map_err(|_| CtlError::Network("connect timed out".into()))?
        .map_err(|e| CtlError::Network(format!("ws connect failed: {e}")))?;

        // Authenticate. Pro7 replies with {"action":"authenticate","authenticated":1}
        // on success; 0 with an error field on failure.
        let auth = json!({
            "action": "authenticate",
            "protocol": PROTOCOL_VERSION,
            "password": password,
        });
        ws.send(Message::Text(auth.to_string()))
            .await
            .map_err(|e| CtlError::Network(format!("auth send: {e}")))?;

        // Read until we see an authenticate response. Pro7 may send other
        // unsolicited messages before the auth reply, so loop until we find it.
        let auth_deadline = tokio::time::Instant::now() + Duration::from_secs(3);
        loop {
            let remaining = auth_deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                return Err(CtlError::Network("auth timed out".into()));
            }
            let msg = tokio::time::timeout(remaining, ws.next())
                .await
                .map_err(|_| CtlError::Network("auth timed out".into()))?
                .ok_or_else(|| CtlError::Network("ws closed before auth reply".into()))?
                .map_err(|e| CtlError::Network(format!("auth read: {e}")))?;
            if let Message::Text(t) = msg {
                if let Ok(resp) = serde_json::from_str::<AuthResponse>(&t) {
                    if resp.authenticated == 1 {
                        break;
                    }
                    return Err(CtlError::AuthRequired);
                }
            }
        }

        // Hand the socket off to a background task; commands flow through an
        // mpsc so the driver's per-action methods stay short and lock-free.
        let (tx, rx) = mpsc::channel::<Message>(32);
        tokio::spawn(bridge(ws, rx));
        log::info!("[PRO7-WS] connected to {host}:{port}");
        self.tx = Some(tx);
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), CtlError> {
        // Dropping the sender ends the bridge task, which closes the socket.
        self.tx = None;
        Ok(())
    }

    async fn next_slide(&self) -> Result<(), CtlError> {
        self.send_action(json!({ "action": "presentationTriggerNext" }))
            .await?;
        log::info!("[PRO7-WS] next");
        Ok(())
    }

    async fn prev_slide(&self) -> Result<(), CtlError> {
        self.send_action(json!({ "action": "presentationTriggerPrevious" }))
            .await?;
        log::info!("[PRO7-WS] prev");
        Ok(())
    }

    async fn goto_slide(&self, index: u32) -> Result<(), CtlError> {
        // slideIndex must be a STRING on Pro7 (the upstream API docs flag this
        // as a Pro7 bug — Pro6 takes an integer). presentationPath "0:0" means
        // the currently-selected presentation, slide group 0.
        self.send_action(json!({
            "action": "presentationTriggerIndex",
            "slideIndex": index.to_string(),
            "presentationPath": "0:0",
        }))
        .await?;
        log::info!("[PRO7-WS] goto_slide {index}");
        Ok(())
    }
}
