use crate::aligner::{AlignConfig, LyricsAligner, Setlist, Slide, Song};
use crate::presenters::{
    make_controller, Capabilities, KeystrokeProfile, PresenterConfig, PresenterKind,
    PresenterState,
};
use crate::transcription::WhisperModel;
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering;
use tauri::{Emitter, State};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatusInfo {
    pub is_running: bool,
    pub model_loaded: bool,
    pub current_slide: usize,
    pub total_slides: usize,
    pub song_title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlideAdvanced {
    pub slide_index: usize,
    pub total_slides: usize,
    pub slide_text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TranscriptionEvent {
    pub text: String,
    pub curr_score: f64,
    pub next_score: f64,
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

/// Front-end-facing descriptor for a presenter driver.
#[derive(Debug, Serialize)]
pub struct PresenterDescriptor {
    pub kind: PresenterKind,
    pub display_name: &'static str,
}

/// Snapshot of the active presenter for the Connection panel.
#[derive(Debug, Serialize)]
pub struct PresenterInfo {
    pub kind: PresenterKind,
    pub display_name: &'static str,
    pub connected: bool,
    pub capabilities: Capabilities,
}

/// Load a setlist JSON file and prepare the aligner.
#[tauri::command]
pub async fn load_setlist(
    state: State<'_, AppState>,
    setlist_json: String,
) -> Result<Vec<Song>, String> {
    let setlist: Setlist = serde_json::from_str(&setlist_json).map_err(|e| e.to_string())?;

    let mut all_slides: Vec<Slide> = Vec::new();
    for song in &setlist.setlist {
        for slide in &song.slides {
            all_slides.push(slide.clone());
        }
    }

    let config = AlignConfig::default();
    let aligner = LyricsAligner::new(all_slides, config);

    *state.aligner.lock().await = Some(aligner);

    log::info!("Loaded setlist with {} songs", setlist.setlist.len());

    Ok(setlist.setlist)
}

/// Start listening and auto-advancing slides.
#[tauri::command]
pub async fn start_listening(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    if state.is_running.load(Ordering::SeqCst) {
        return Err("Already running".to_string());
    }

    {
        let aligner = state.aligner.lock().await;
        if aligner.is_none() {
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
    let aligner = state.aligner.clone();
    let presenter = state.presenter.clone();
    let is_running = state.is_running.clone();
    let app_handle = app.clone();

    tokio::task::spawn_blocking(move || {
        let mut accumulated = Vec::new();

        while is_running.load(Ordering::SeqCst) {
            match rx.recv_timeout(std::time::Duration::from_millis(100)) {
                Ok(samples) => {
                    let speech_segment = {
                        let mut vad = vad.blocking_lock();
                        vad.process(&samples)
                    };

                    if let Some(segment) = speech_segment {
                        accumulated.extend_from_slice(&segment);

                        if accumulated.len() >= 16000 * 5 {
                            let transcription = transcription.blocking_lock();
                            match transcription.transcribe(&accumulated) {
                                Ok(result) if !result.text.is_empty() => {
                                    log::info!("Heard: {}", result.text);

                                    let mut aligner_lock = aligner.blocking_lock();
                                    if let Some(ref mut aligner) = *aligner_lock {
                                        let advanced = aligner.update(&result.text);

                                        let _ = app_handle.emit(
                                            "transcription",
                                            TranscriptionEvent {
                                                text: result.text.clone(),
                                                curr_score: 0.0,
                                                next_score: 0.0,
                                            },
                                        );

                                        if advanced {
                                            // Delegate to whichever presenter is active. The
                                            // tokio runtime created by spawn_blocking's host
                                            // isn't a runtime — Handle::current() panics here —
                                            // so we use a private current-thread runtime.
                                            let presenter = presenter.clone();
                                            let rt = tokio::runtime::Builder::new_current_thread()
                                                .enable_all()
                                                .build();
                                            if let Ok(rt) = rt {
                                                let res = rt.block_on(async {
                                                    let p = presenter.lock().await;
                                                    p.next_slide().await
                                                });
                                                if let Err(e) = res {
                                                    log::error!("next_slide failed: {e}");
                                                }
                                            }

                                            let slide_text = aligner
                                                .current_slide()
                                                .map(|s| s.text.clone())
                                                .unwrap_or_default();

                                            let _ = app_handle.emit(
                                                "slide-advanced",
                                                SlideAdvanced {
                                                    slide_index: aligner.current_index(),
                                                    total_slides: aligner.total_slides(),
                                                    slide_text,
                                                },
                                            );
                                        }
                                    }
                                }
                                Ok(_) => {}
                                Err(e) => log::error!("Transcription error: {}", e),
                            }
                            accumulated.clear();
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
    let aligner = state.aligner.lock().await;
    let (current_slide, total_slides) = match &*aligner {
        Some(a) => (a.current_index(), a.total_slides()),
        None => (0, 0),
    };

    Ok(StatusInfo {
        is_running: state.is_running.load(Ordering::SeqCst),
        model_loaded: true,
        current_slide,
        total_slides,
        song_title: None,
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

/// Manually advance to next slide.
#[tauri::command]
pub async fn next_slide_manual(state: State<'_, AppState>) -> Result<(), String> {
    {
        let p = state.presenter.lock().await;
        p.next_slide().await.map_err(|e| e.to_string())?;
    }
    let mut aligner = state.aligner.lock().await;
    if let Some(ref mut a) = *aligner {
        let _ = a.update("__manual_advance__");
    }
    Ok(())
}

/// Manually go back a slide.
#[tauri::command]
pub async fn prev_slide_manual(state: State<'_, AppState>) -> Result<(), String> {
    let p = state.presenter.lock().await;
    p.prev_slide().await.map_err(|e| e.to_string())
}

/// Manually blank the output. Falls back to the keystroke "blank" key on the keystroke
/// driver; ProPresenter REST clears the slide layer.
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
    pub password: Option<String>,
    #[serde(default)]
    pub keystroke_profile: Option<KeystrokeProfile>,
}

/// Swap to a different presenter driver. Disconnects the previous one first.
#[tauri::command]
pub async fn connect_presenter(
    state: State<'_, AppState>,
    args: ConnectPresenterArgs,
) -> Result<PresenterInfo, String> {
    let mut new_driver = make_controller(args.kind);
    let cfg = PresenterConfig {
        host: args.host,
        port: args.port,
        password: args.password,
        keystroke_profile: args.keystroke_profile,
    };
    new_driver
        .connect(cfg)
        .await
        .map_err(|e| e.to_string())?;

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

/// Read the active presenter's current state (slide index, presentation name).
/// Returns `None` when the driver doesn't support state queries.
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
