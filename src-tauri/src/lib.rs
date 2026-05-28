mod aligner;
mod audio;
mod commands;
mod presenters;
mod transcription;
mod tray;
mod vad;

#[cfg(feature = "mcp")]
mod mcp;

use std::sync::Arc;
use tokio::sync::Mutex;

pub use aligner::{AlignConfig, LyricsAligner, Setlist, Slide, Song};
pub use audio::AudioEngine;
pub use presenters::{
    make_controller, Capabilities, CtlError, KeystrokeProfile, PresentationController,
    PresenterConfig, PresenterKind, PresenterState,
};
pub use transcription::TranscriptionEngine;
pub use vad::VadEngine;

/// Shared application state.
pub struct AppState {
    pub audio: Arc<Mutex<AudioEngine>>,
    pub transcription: Arc<Mutex<TranscriptionEngine>>,
    pub vad: Arc<Mutex<VadEngine>>,
    pub aligner: Arc<Mutex<Option<LyricsAligner>>>,
    pub presenter: Arc<Mutex<Box<dyn PresentationController>>>,
    pub is_running: Arc<std::sync::atomic::AtomicBool>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    let audio = Arc::new(Mutex::new(AudioEngine::new()));
    let transcription = Arc::new(Mutex::new(TranscriptionEngine::new()));
    let vad = Arc::new(Mutex::new(VadEngine::new()));
    let aligner = Arc::new(Mutex::new(None::<LyricsAligner>));
    // Default driver: keystroke / universal — works the moment the user focuses any
    // presentation app, no configuration required.
    let mut keystroke = make_controller(PresenterKind::Keystroke);
    {
        // Pre-connect with default profile so manual prev/next work before the user
        // touches Settings.
        let cfg = PresenterConfig {
            keystroke_profile: Some(KeystrokeProfile::Universal),
            ..Default::default()
        };
        // Connect is async; we're in sync `run()`. The keystroke connect doesn't
        // actually do I/O, so block_on a tiny runtime.
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio rt");
        let _ = rt.block_on(keystroke.connect(cfg));
    }
    let presenter = Arc::new(Mutex::new(keystroke));
    let is_running = Arc::new(std::sync::atomic::AtomicBool::new(false));

    let state = AppState {
        audio: audio.clone(),
        transcription: transcription.clone(),
        vad: vad.clone(),
        aligner: aligner.clone(),
        presenter: presenter.clone(),
        is_running: is_running.clone(),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::load_setlist,
            commands::start_listening,
            commands::stop_listening,
            commands::get_status,
            commands::list_audio_devices,
            commands::get_model_status,
            commands::download_model,
            commands::get_settings,
            commands::save_settings,
            commands::next_slide_manual,
            commands::prev_slide_manual,
            commands::blank_manual,
            commands::list_presenters,
            commands::connect_presenter,
            commands::disconnect_presenter,
            commands::get_presenter_info,
            commands::get_presenter_state,
        ])
        .setup(|app| {
            tray::create_tray(app)?;
            log::info!("Voxxa started successfully");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Voxxa");
}
