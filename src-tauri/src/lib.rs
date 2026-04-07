mod audio;
mod commands;
mod text_insert;
mod transcription;
mod tray;
mod vad;

#[cfg(feature = "mcp")]
mod mcp;

use std::sync::Arc;
use tokio::sync::Mutex;

pub use audio::AudioEngine;
pub use transcription::TranscriptionEngine;
pub use vad::VadEngine;

/// Shared application state accessible from Tauri commands.
pub struct AppState {
    pub audio: Arc<Mutex<AudioEngine>>,
    pub transcription: Arc<Mutex<TranscriptionEngine>>,
    pub vad: Arc<Mutex<VadEngine>>,
    pub is_recording: Arc<std::sync::atomic::AtomicBool>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    let audio = Arc::new(Mutex::new(AudioEngine::new()));
    let transcription = Arc::new(Mutex::new(TranscriptionEngine::new()));
    let vad = Arc::new(Mutex::new(VadEngine::new()));
    let is_recording = Arc::new(std::sync::atomic::AtomicBool::new(false));

    let state = AppState {
        audio: audio.clone(),
        transcription: transcription.clone(),
        vad: vad.clone(),
        is_recording: is_recording.clone(),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::start_recording,
            commands::stop_recording,
            commands::get_status,
            commands::list_audio_devices,
            commands::get_model_status,
            commands::download_model,
            commands::set_hotkey,
            commands::get_settings,
            commands::save_settings,
            commands::get_transcription_history,
        ])
        .setup(|app| {
            tray::create_tray(app)?;
            log::info!("Voxxa started successfully");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Voxxa");
}
