use crate::aligner::{AlignConfig, LyricsAligner, Setlist, Slide, Song};
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

/// Load a setlist JSON file and prepare the aligner.
#[tauri::command]
pub async fn load_setlist(
    state: State<'_, AppState>,
    setlist_json: String,
) -> Result<Vec<Song>, String> {
    let setlist: Setlist = serde_json::from_str(&setlist_json).map_err(|e| e.to_string())?;

    // Flatten all slides across all songs
    let mut all_slides: Vec<Slide> = Vec::new();
    for song in &setlist.setlist {
        for slide in &song.slides {
            all_slides.push(slide.clone());
        }
    }

    let config = AlignConfig::default();
    let aligner = LyricsAligner::new(all_slides, config);

    *state.aligner.lock().await = Some(aligner);

    log::info!(
        "Loaded setlist with {} songs",
        setlist.setlist.len()
    );

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

                        // Transcribe when we have enough audio (~5 seconds)
                        if accumulated.len() >= 16000 * 5 {
                            let transcription = transcription.blocking_lock();
                            match transcription.transcribe(&accumulated) {
                                Ok(result) if !result.text.is_empty() => {
                                    log::info!("Heard: {}", result.text);

                                    let mut aligner_lock = aligner.blocking_lock();
                                    if let Some(ref mut aligner) = *aligner_lock {
                                        let advanced = aligner.update(&result.text);

                                        let _ = app_handle.emit("transcription", TranscriptionEvent {
                                            text: result.text.clone(),
                                            curr_score: 0.0,
                                            next_score: 0.0,
                                        });

                                        if advanced {
                                            if let Err(e) = crate::text_insert::send_next_slide() {
                                                log::error!("Failed to advance slide: {}", e);
                                            }

                                            let slide_text = aligner
                                                .current_slide()
                                                .map(|s| s.text.clone())
                                                .unwrap_or_default();

                                            let _ = app_handle.emit("slide-advanced", SlideAdvanced {
                                                slide_index: aligner.current_index(),
                                                total_slides: aligner.total_slides(),
                                                slide_text,
                                            });
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

/// List available audio input devices.
#[tauri::command]
pub async fn list_audio_devices() -> Result<Vec<String>, String> {
    crate::AudioEngine::list_devices().map_err(|e| e.to_string())
}

/// Get download status for all models.
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

/// Download a Whisper model.
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

/// Get current settings.
#[tauri::command]
pub async fn get_settings() -> Result<Settings, String> {
    Ok(Settings::default())
}

/// Save settings.
#[tauri::command]
pub async fn save_settings(_settings: Settings) -> Result<(), String> {
    Ok(())
}

/// Manually advance to next slide.
#[tauri::command]
pub async fn next_slide_manual(state: State<'_, AppState>) -> Result<(), String> {
    crate::text_insert::send_next_slide().map_err(|e| e.to_string())?;
    let mut aligner = state.aligner.lock().await;
    if let Some(ref mut a) = *aligner {
        // Manually bump the index to keep in sync
        let _ = a.update("__manual_advance__");
    }
    Ok(())
}

/// Manually go back a slide.
#[tauri::command]
pub async fn prev_slide_manual() -> Result<(), String> {
    crate::text_insert::send_prev_slide().map_err(|e| e.to_string())
}
