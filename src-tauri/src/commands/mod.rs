use crate::aligner::{Action, Conductor, MachineState, Setlist, SmartConfig, Song};
use crate::importers::{self, ImportFormat};
use crate::presenters::{
    make_controller, Capabilities, KeystrokeProfile, PresenterConfig, PresenterKind,
    PresenterState,
};
use crate::transcription::WhisperModel;
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{Emitter, State};
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatusInfo {
    pub is_running: bool,
    pub model_loaded: bool,
    pub current_slide: usize,
    pub total_slides: usize,
    pub song_title: Option<String>,
    pub machine_state: Option<MachineState>,
    pub is_blank: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlideAdvanced {
    pub slide_index: usize,
    pub total_slides: usize,
    pub slide_text: String,
    pub song_title: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StateChanged {
    pub state: MachineState,
    pub is_blank: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TranscriptionEvent {
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub model: String,
    pub language: Option<String>,
    pub device: Option<String>,
    pub vad_enabled: bool,
    pub similarity_threshold: f64,
    pub margin: f64,
    pub max_buffer_words: usize,
    pub block_duration: f64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            model: "Base".to_string(),
            language: None,
            device: None,
            vad_enabled: true,
            similarity_threshold: 70.0,
            margin: 10.0,
            max_buffer_words: 40,
            block_duration: 5.0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub display_name: String,
    pub downloaded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub downloaded: u64,
    pub total: u64,
    pub percent: f32,
}

#[derive(Debug, Serialize)]
pub struct PresenterDescriptor {
    pub kind: PresenterKind,
    pub display_name: &'static str,
}

#[derive(Debug, Serialize)]
pub struct PresenterInfo {
    pub kind: PresenterKind,
    pub display_name: &'static str,
    pub connected: bool,
    pub capabilities: Capabilities,
}

/// Load a setlist JSON file and prepare the smart-blanking conductor.
#[tauri::command]
pub async fn load_setlist(
    state: State<'_, AppState>,
    setlist_json: String,
) -> Result<Vec<Song>, String> {
    let setlist: Setlist = serde_json::from_str(&setlist_json).map_err(|e| e.to_string())?;
    let songs = setlist.setlist.clone();
    let conductor = Conductor::new(songs.clone(), SmartConfig::default());
    *state.conductor.lock().await = Some(conductor);
    state.last_dispatched_global.store(-1, Ordering::SeqCst);
    log::info!("Loaded setlist with {} songs", songs.len());
    Ok(songs)
}

/// Start listening: drives VAD → Whisper → Conductor → presenter.
#[tauri::command]
pub async fn start_listening(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    if state.is_running.load(Ordering::SeqCst) {
        return Err("Already running".to_string());
    }
    {
        let c = state.conductor.lock().await;
        if c.is_none() {
            return Err("No setlist loaded. Load a setlist first.".to_string());
        }
    }
    state.is_running.store(true, Ordering::SeqCst);

    let rx = {
        let mut audio = state.audio.lock().await;
        audio.start().map_err(|e| e.to_string())?
    };

    let vad = state.vad.clone();
    let transcription = state.transcription.clone();
    let conductor = state.conductor.clone();
    let presenter = state.presenter.clone();
    let last_dispatched = state.last_dispatched_global.clone();
    let is_running = state.is_running.clone();
    let app_handle = app.clone();
    // Capture the runtime handle so spawn_blocking can call back into async-land
    // without building a fresh runtime each iteration.
    let rt_handle = tokio::runtime::Handle::current();

    tokio::task::spawn_blocking(move || {
        let mut accumulated: Vec<f32> = Vec::new();
        let mut last_state = MachineState::Listening;
        let mut last_blank = true;
        let mut last_tick = Instant::now();
        const TICK_EVERY: Duration = Duration::from_millis(200);

        while is_running.load(Ordering::SeqCst) {
            let now = Instant::now();

            // Periodic tick — drives time-based transitions (silence → blank).
            if now.duration_since(last_tick) >= TICK_EVERY {
                last_tick = now;
                let (action, total) = {
                    let mut cl = conductor.blocking_lock();
                    match cl.as_mut() {
                        Some(c) => (c.tick(now), Some(c.total_slides())),
                        None => (Action::Noop, None),
                    }
                };
                dispatch_action_with_total(
                    &rt_handle,
                    action,
                    &presenter,
                    &last_dispatched,
                    &app_handle,
                    total,
                );

                // Emit state changes for the UI.
                let snap = {
                    let cl = conductor.blocking_lock();
                    cl.as_ref().map(|c| (c.state(), c.is_blank()))
                };
                if let Some((s, b)) = snap {
                    if s != last_state || b != last_blank {
                        let _ = app_handle.emit(
                            "machine-state",
                            StateChanged {
                                state: s,
                                is_blank: b,
                            },
                        );
                        last_state = s;
                        last_blank = b;
                    }
                }
            }

            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(samples) => {
                    let speech_segment = {
                        let mut v = vad.blocking_lock();
                        let seg = v.process(&samples);
                        if v.is_in_speech() {
                            if let Some(c) = conductor.blocking_lock().as_mut() {
                                c.on_speech(now);
                            }
                        }
                        seg
                    };

                    if let Some(segment) = speech_segment {
                        accumulated.extend_from_slice(&segment);

                        if accumulated.len() >= 16000 * 5 {
                            let result = {
                                let t = transcription.blocking_lock();
                                t.transcribe(&accumulated)
                            };
                            accumulated.clear();

                            match result {
                                Ok(r) if !r.text.is_empty() => {
                                    log::info!("Heard: {}", r.text);
                                    let _ = app_handle.emit(
                                        "transcription",
                                        TranscriptionEvent {
                                            text: r.text.clone(),
                                        },
                                    );
                                    let (action, total) = {
                                        let mut cl = conductor.blocking_lock();
                                        match cl.as_mut() {
                                            Some(c) => (
                                                c.on_transcript(&r.text, now),
                                                Some(c.total_slides()),
                                            ),
                                            None => (Action::Noop, None),
                                        }
                                    };
                                    dispatch_action_with_total(
                                        &rt_handle,
                                        action,
                                        &presenter,
                                        &last_dispatched,
                                        &app_handle,
                                        total,
                                    );
                                }
                                Ok(_) => {}
                                Err(e) => log::error!("Transcription error: {}", e),
                            }
                        }
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });

    Ok(())
}

/// Translate a `Conductor` action into one or more presenter calls. `Goto` is
/// resolved against `last_dispatched_global` to choose between `next/prev` (delta of
/// ±1) and the driver's `goto_slide` capability, falling back to a run of next/prev
/// keys when the driver lacks goto support.
fn dispatch_action_with_total(
    rt: &tokio::runtime::Handle,
    action: Action,
    presenter: &Arc<Mutex<Box<dyn crate::PresentationController>>>,
    last_dispatched: &Arc<AtomicI64>,
    app_handle: &tauri::AppHandle,
    total_slides_hint: Option<usize>,
) {
    match action {
        Action::Noop => {}
        Action::Blank => {
            let p = presenter.clone();
            let res = rt.block_on(async move {
                let p = p.lock().await;
                p.blank().await
            });
            if let Err(e) = res {
                log::warn!("blank failed: {e}");
            }
        }
        Action::Unblank => {
            let p = presenter.clone();
            let res = rt.block_on(async move {
                let p = p.lock().await;
                p.unblank().await
            });
            if let Err(e) = res {
                log::warn!("unblank failed: {e}");
            }
        }
        Action::Goto {
            song_index: _,
            slide_in_song: _,
            global_index,
            slide_text,
            song_title,
        } => {
            let target = global_index as i64;
            let prev = last_dispatched.load(Ordering::SeqCst);
            let p = presenter.clone();
            let can_goto = rt.block_on(async {
                let p = p.lock().await;
                p.capabilities().can_goto_slide
            });

            let p = presenter.clone();
            let res: Result<(), crate::CtlError> = rt.block_on(async move {
                let p = p.lock().await;
                if prev < 0 {
                    // No prior dispatch — use goto_slide where available; otherwise we
                    // can't jump to an absolute slide via keystrokes from an unknown
                    // start, so we just unblank and hope the operator manually advanced
                    // to slide 0 first. (Keystroke users typically start at slide 0.)
                    if can_goto {
                        return p.goto_slide(global_index as u32).await;
                    }
                    return p.unblank().await;
                }
                let delta = target - prev;
                if delta == 0 {
                    return p.unblank().await;
                }
                if delta == 1 {
                    return p.next_slide().await;
                }
                if delta == -1 {
                    return p.prev_slide().await;
                }
                if can_goto {
                    return p.goto_slide(global_index as u32).await;
                }
                // Translate larger jumps to repeated next/prev keystrokes. Sleep a
                // tick between sends so the receiver isn't overwhelmed.
                let steps = delta.abs() as usize;
                for _ in 0..steps {
                    let r = if delta > 0 {
                        p.next_slide().await
                    } else {
                        p.prev_slide().await
                    };
                    if let Err(e) = r {
                        return Err(e);
                    }
                    tokio::time::sleep(Duration::from_millis(60)).await;
                }
                Ok(())
            });

            if let Err(e) = res {
                log::error!("Goto({global_index}) failed: {e}");
                return;
            }
            last_dispatched.store(target, Ordering::SeqCst);

            let _ = app_handle.emit(
                "slide-advanced",
                SlideAdvanced {
                    slide_index: global_index,
                    // Fall back to (global+1) only when the caller didn't pass the
                    // true total — it's a harmless underestimate for the manual
                    // command paths that don't have conductor access on hand.
                    total_slides: total_slides_hint.unwrap_or(global_index + 1),
                    slide_text,
                    song_title,
                },
            );
        }
    }
}

/// Stop listening.
#[tauri::command]
pub async fn stop_listening(state: State<'_, AppState>) -> Result<(), String> {
    state.is_running.store(false, Ordering::SeqCst);
    let mut audio = state.audio.lock().await;
    audio.stop();
    let mut vad = state.vad.lock().await;
    vad.reset();
    Ok(())
}

/// Get current status.
#[tauri::command]
pub async fn get_status(state: State<'_, AppState>) -> Result<StatusInfo, String> {
    let conductor = state.conductor.lock().await;
    let (current_slide, total_slides, machine_state, is_blank, song_title) = match &*conductor {
        Some(c) => (
            c.current_index(),
            c.total_slides(),
            Some(c.state()),
            Some(c.is_blank()),
            c.current_song_title().map(|s| s.to_string()),
        ),
        None => (0, 0, None, None, None),
    };
    Ok(StatusInfo {
        is_running: state.is_running.load(Ordering::SeqCst),
        model_loaded: true,
        current_slide,
        total_slides,
        song_title,
        machine_state,
        is_blank,
    })
}

#[tauri::command]
pub async fn list_audio_devices() -> Result<Vec<String>, String> {
    crate::AudioEngine::list_devices().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_model_status(state: State<'_, AppState>) -> Result<Vec<ModelInfo>, String> {
    let transcription = state.transcription.lock().await;
    Ok(transcription
        .model_status()
        .into_iter()
        .map(|(m, downloaded)| ModelInfo {
            name: format!("{:?}", m),
            display_name: m.display_name().to_string(),
            downloaded,
        })
        .collect())
}

#[tauri::command]
pub async fn download_model(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    model_name: String,
) -> Result<(), String> {
    let model = match model_name.as_str() {
        "Tiny" => WhisperModel::Tiny,
        "Base" => WhisperModel::Base,
        "Small" => WhisperModel::Small,
        "Medium" => WhisperModel::Medium,
        "LargeV3Turbo" => WhisperModel::LargeV3Turbo,
        "DistilLargeV3" => WhisperModel::DistilLargeV3,
        _ => return Err(format!("Unknown model: {}", model_name)),
    };
    let transcription = state.transcription.lock().await;
    let app_handle = app.clone();
    transcription
        .download_model(&model, move |downloaded, total| {
            let percent = if total > 0 {
                (downloaded as f32 / total as f32) * 100.0
            } else {
                0.0
            };
            let _ = app_handle.emit(
                "download-progress",
                DownloadProgress {
                    downloaded,
                    total,
                    percent,
                },
            );
        })
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_settings() -> Result<Settings, String> {
    Ok(Settings::default())
}

#[tauri::command]
pub async fn save_settings(_settings: Settings) -> Result<(), String> {
    Ok(())
}

/// Manually advance to next slide. Updates `last_dispatched_global` so the
/// conductor's next Goto computes the right delta.
#[tauri::command]
pub async fn next_slide_manual(state: State<'_, AppState>) -> Result<(), String> {
    {
        let p = state.presenter.lock().await;
        p.next_slide().await.map_err(|e| e.to_string())?;
    }
    state.last_dispatched_global.fetch_add(1, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub async fn prev_slide_manual(state: State<'_, AppState>) -> Result<(), String> {
    {
        let p = state.presenter.lock().await;
        p.prev_slide().await.map_err(|e| e.to_string())?;
    }
    state.last_dispatched_global.fetch_sub(1, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub async fn blank_manual(state: State<'_, AppState>) -> Result<(), String> {
    let p = state.presenter.lock().await;
    p.blank().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_presenters() -> Vec<PresenterDescriptor> {
    PresenterKind::all()
        .iter()
        .map(|&k| PresenterDescriptor {
            kind: k,
            display_name: k.display_name(),
        })
        .collect()
}

#[derive(Debug, Deserialize)]
pub struct ConnectPresenterArgs {
    pub kind: PresenterKind,
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub keystroke_profile: Option<KeystrokeProfile>,
}

#[tauri::command]
pub async fn connect_presenter(
    state: State<'_, AppState>,
    args: ConnectPresenterArgs,
) -> Result<PresenterInfo, String> {
    let mut new_driver = make_controller(args.kind);
    let cfg = PresenterConfig {
        host: args.host,
        port: args.port,
        username: args.username,
        password: args.password,
        keystroke_profile: args.keystroke_profile,
    };
    new_driver.connect(cfg).await.map_err(|e| e.to_string())?;
    let mut active = state.presenter.lock().await;
    let _ = active.disconnect().await;
    *active = new_driver;
    Ok(PresenterInfo {
        kind: active.kind(),
        display_name: active.kind().display_name(),
        connected: active.is_connected(),
        capabilities: active.capabilities(),
    })
}

#[tauri::command]
pub async fn disconnect_presenter(state: State<'_, AppState>) -> Result<(), String> {
    let mut p = state.presenter.lock().await;
    p.disconnect().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_presenter_info(state: State<'_, AppState>) -> Result<PresenterInfo, String> {
    let p = state.presenter.lock().await;
    Ok(PresenterInfo {
        kind: p.kind(),
        display_name: p.kind().display_name(),
        connected: p.is_connected(),
        capabilities: p.capabilities(),
    })
}

/// Parse a text-source song (.txt, OpenLyrics, OpenSong, ChordPro). Returned
/// `Song` is appended to the in-flight setlist by the frontend.
#[tauri::command]
pub fn parse_song_text(
    content: String,
    format: ImportFormat,
    fallback_title: Option<String>,
) -> Result<Song, String> {
    let fallback = fallback_title.unwrap_or_else(|| "Untitled".to_string());
    importers::parse_text(format, &content, &fallback).map_err(|e| e.to_string())
}

/// Parse a binary-source song (.pptx). `bytes_b64` is base64-encoded raw file bytes.
#[tauri::command]
pub fn parse_song_bytes(
    bytes_b64: String,
    format: ImportFormat,
    fallback_title: Option<String>,
) -> Result<Song, String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(bytes_b64.as_bytes())
        .map_err(|e| format!("invalid base64: {e}"))?;
    let fallback = fallback_title.unwrap_or_else(|| "Untitled".to_string());
    importers::parse_bytes(format, &bytes, &fallback).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_presenter_state(
    state: State<'_, AppState>,
) -> Result<Option<PresenterState>, String> {
    let p = state.presenter.lock().await;
    match p.current_state().await {
        Ok(s) => Ok(Some(s)),
        Err(crate::CtlError::Unsupported(_)) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}
