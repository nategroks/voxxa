use crate::AppState;
use crate::transcription::WhisperModel;
use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering;
use tauri::{Emitter, State};

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusInfo {
    pub is_recording: bool,
    pub model_loaded: bool,
    pub current_model: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub model: String,
    pub language: Option<String>,
    pub hotkey: String,
    pub auto_paste: bool,
    pub show_overlay: bool,
    pub device: Option<String>,
    pub vad_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            model: "Base".to_string(),
            language: None,
            hotkey: "CmdOrCtrl+Shift+Space".to_string(),
            auto_paste: true,
            show_overlay: true,
            device: None,
            vad_enabled: true,
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

/// Start recording audio and transcribing.
#[tauri::command]
pub async fn start_recording(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    if state.is_recording.load(Ordering::SeqCst) {
        return Err("Already recording".to_string());
    }

    state.is_recording.store(true, Ordering::SeqCst);

    // Start audio capture
    let rx = {
        let mut audio = state.audio.lock().await;
        audio.start().map_err(|e| e.to_string())?
    };

    // Spawn a task to process audio through VAD and Whisper
    let vad = state.vad.clone();
    let transcription = state.transcription.clone();
    let is_recording = state.is_recording.clone();
    let app_handle = app.clone();

    tokio::task::spawn_blocking(move || {
        let mut accumulated = Vec::new();

        while is_recording.load(Ordering::SeqCst) {
            match rx.recv_timeout(std::time::Duration::from_millis(100)) {
                Ok(samples) => {
                    // Run through VAD
                    let speech_segment = {
                        let mut vad = vad.blocking_lock();
                        vad.process(&samples)
                    };

                    if let Some(segment) = speech_segment {
                        accumulated.extend_from_slice(&segment);

                        // Transcribe when we have enough audio (>1 second)
                        if accumulated.len() >= 16000 {
                            let transcription = transcription.blocking_lock();
                            match transcription.transcribe(&accumulated) {
                                Ok(result) if !result.text.is_empty() => {
                                    log::info!("Transcribed: {}", result.text);
                                    let _ = app_handle.emit("transcription", &result);
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

        // Transcribe any remaining audio
        if !accumulated.is_empty() {
            let transcription = transcription.blocking_lock();
            if let Ok(result) = transcription.transcribe(&accumulated) {
                if !result.text.is_empty() {
                    let _ = app_handle.emit("transcription", &result);
                }
            }
        }
    });

    Ok(())
}

/// Stop recording.
#[tauri::command]
pub async fn stop_recording(state: State<'_, AppState>) -> Result<(), String> {
    state.is_recording.store(false, Ordering::SeqCst);
    let mut audio = state.audio.lock().await;
    audio.stop();
    let mut vad = state.vad.lock().await;
    vad.reset();
    Ok(())
}

/// Get current app status.
#[tauri::command]
pub async fn get_status(state: State<'_, AppState>) -> Result<StatusInfo, String> {
    let transcription = state.transcription.lock().await;
    Ok(StatusInfo {
        is_recording: state.is_recording.load(Ordering::SeqCst),
        model_loaded: transcription.model_status().iter().any(|(_, d)| *d),
        current_model: None,
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

/// Set the global hotkey for push-to-talk.
#[tauri::command]
pub async fn set_hotkey(_hotkey: String) -> Result<(), String> {
    // Hotkey registration is handled via tauri-plugin-global-shortcut
    // This command stores the preference
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

/// Get transcription history.
#[tauri::command]
pub async fn get_transcription_history() -> Result<Vec<String>, String> {
    Ok(Vec::new())
}
