//! Local HTTP API for hardware integrations.
//!
//! Off by default; the operator opts in via Settings → Integrations and picks
//! a port (default 7575). Bound to 127.0.0.1 only — there is no use case for
//! letting Voxxa be commanded from another machine over the LAN, and binding
//! to 0.0.0.0 would be a footgun. Optional bearer token gates writes when set.
//!
//! Stream Deck plugins, Loupedeck profiles, foot pedals, MIDI controllers,
//! and arbitrary shell scripts are the use cases.

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

/// Shared with route handlers. Holds a `tauri::AppHandle` so handlers can
/// invoke commands the same way the JS frontend would, plus the bearer token
/// for write-protection.
#[derive(Clone)]
struct ApiContext {
    app: tauri::AppHandle,
    token: Option<String>,
}

pub struct HttpApiServer {
    /// `None` when the server isn't running.
    handle: Option<JoinHandle<()>>,
    port: u16,
}

impl HttpApiServer {
    pub fn new() -> Self {
        Self { handle: None, port: 0 }
    }

    pub fn is_running(&self) -> bool {
        self.handle.is_some()
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    /// Bind and start the API. Errors if it's already running or the port is taken.
    pub async fn start(
        &mut self,
        app: tauri::AppHandle,
        port: u16,
        token: Option<String>,
    ) -> Result<(), String> {
        if self.handle.is_some() {
            return Err("HTTP API already running".into());
        }
        let ctx = ApiContext { app, token };
        let router = Router::new()
            .route("/api/v1/state", get(get_state))
            .route("/api/v1/next", post(post_next))
            .route("/api/v1/prev", post(post_prev))
            .route("/api/v1/blank", post(post_blank))
            .route("/api/v1/listen/start", post(post_listen_start))
            .route("/api/v1/listen/stop", post(post_listen_stop))
            .with_state(ctx);

        // 127.0.0.1 only — never bind to all interfaces.
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| format!("bind {port}: {e}"))?;
        log::info!("[HTTP-API] listening on http://{addr}");

        let server = axum::serve(listener, router);
        let handle = tokio::spawn(async move {
            if let Err(e) = server.await {
                log::error!("[HTTP-API] server error: {e}");
            }
        });
        self.handle = Some(handle);
        self.port = port;
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(h) = self.handle.take() {
            h.abort();
            log::info!("[HTTP-API] stopped");
        }
        self.port = 0;
    }
}

#[derive(Serialize)]
struct ApiState {
    is_running: bool,
    current_slide: usize,
    total_slides: usize,
    song_title: Option<String>,
    machine_state: Option<String>,
    is_blank: bool,
}

#[derive(Serialize)]
struct ApiError {
    error: String,
}

/// Check the bearer token if one is configured. Returns Err with 401 when the
/// token is wrong; Ok(()) when it's either absent (no auth required) or correct.
fn check_token(ctx: &ApiContext, headers: &HeaderMap) -> Result<(), (StatusCode, Json<ApiError>)> {
    let Some(required) = &ctx.token else {
        return Ok(());
    };
    let header = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .unwrap_or("");
    // Constant-time-ish equality on equal-length strings; fine for a local API.
    if header == required {
        Ok(())
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiError {
                error: "invalid or missing bearer token".into(),
            }),
        ))
    }
}

async fn get_state(
    State(ctx): State<ApiContext>,
    headers: HeaderMap,
) -> Result<Json<ApiState>, (StatusCode, Json<ApiError>)> {
    check_token(&ctx, &headers)?;
    use tauri::Manager;
    let app_state = ctx.app.state::<crate::AppState>();
    let conductor = app_state.conductor.lock().await;
    let (current_slide, total_slides, machine_state, is_blank, song_title) = match &*conductor {
        Some(c) => (
            c.current_index(),
            c.total_slides(),
            Some(format!("{:?}", c.state())),
            c.is_blank(),
            c.current_song_title().map(String::from),
        ),
        None => (0, 0, None, true, None),
    };
    Ok(Json(ApiState {
        is_running: app_state
            .is_running
            .load(std::sync::atomic::Ordering::SeqCst),
        current_slide,
        total_slides,
        song_title,
        machine_state,
        is_blank,
    }))
}

async fn post_next(
    State(ctx): State<ApiContext>,
    headers: HeaderMap,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    check_token(&ctx, &headers)?;
    use tauri::Manager;
    let app_state = ctx.app.state::<crate::AppState>();
    // Route through the conductor so its slide counter stays in sync with the
    // presenter — same path the Tauri next_slide_manual command takes. A
    // direct presenter.next_slide() here would desync the conductor (Stream
    // Deck advances → conductor still thinks we're on the old slide → next
    // lyric Goto sends the wrong delta).
    crate::commands::manual_step(&app_state, &ctx.app, 1)
        .await
        .map_err(api_err)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn post_prev(
    State(ctx): State<ApiContext>,
    headers: HeaderMap,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    check_token(&ctx, &headers)?;
    use tauri::Manager;
    let app_state = ctx.app.state::<crate::AppState>();
    crate::commands::manual_step(&app_state, &ctx.app, -1)
        .await
        .map_err(api_err)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn post_blank(
    State(ctx): State<ApiContext>,
    headers: HeaderMap,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    check_token(&ctx, &headers)?;
    use tauri::Manager;
    let app_state = ctx.app.state::<crate::AppState>();
    {
        let p = app_state.presenter.lock().await;
        p.blank().await.map_err(api_err)?;
    }
    // Keep the conductor's is_blank in sync (same reason as blank_manual).
    if let Some(c) = app_state.conductor.lock().await.as_mut() {
        c.notify_external_blank();
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn post_listen_start(
    State(ctx): State<ApiContext>,
    headers: HeaderMap,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    check_token(&ctx, &headers)?;
    use tauri::Manager;
    let app_state = ctx.app.state::<crate::AppState>();
    crate::commands::start_listening_with_state(&app_state, &ctx.app)
        .await
        .map_err(api_err)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn post_listen_stop(
    State(ctx): State<ApiContext>,
    headers: HeaderMap,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    check_token(&ctx, &headers)?;
    use tauri::Manager;
    let app_state = ctx.app.state::<crate::AppState>();
    app_state
        .is_running
        .store(false, std::sync::atomic::Ordering::SeqCst);
    let mut audio = app_state.audio.lock().await;
    audio.stop();
    let mut vad = app_state.vad.lock().await;
    vad.reset();
    Ok(StatusCode::NO_CONTENT)
}

fn api_err<E: std::fmt::Display>(e: E) -> (StatusCode, Json<ApiError>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiError {
            error: e.to_string(),
        }),
    )
}

/// AppState extension: shared handle to the running server. Wrapped in Mutex
/// because the start command toggles it.
pub type SharedHttpApi = Arc<Mutex<HttpApiServer>>;
