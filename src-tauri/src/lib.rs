mod aligner;
mod audio;
mod commands;
mod diagnostics;
mod discovery;
mod http_api;
mod hymnary;
mod importers;
mod net_stats;
mod planning_center;
mod presenters;
mod transcription;
mod tray;
mod vad;

use std::sync::Arc;
use tokio::sync::Mutex;

pub use aligner::{Action, Conductor, MachineState, Setlist, Slide, SmartConfig, Song};
pub use audio::AudioEngine;
pub use presenters::{
    make_controller, Capabilities, CtlError, KeystrokeProfile, PresentationController,
    PresenterConfig, PresenterKind, PresenterState,
};
pub use transcription::TranscriptionEngine;
pub use vad::VadEngine;

// Test-friendly importer re-exports. The integration tests in tests/
// exercise these against the example files in examples/; production code
// should keep going through commands::parse_song_text / parse_song_bytes.
pub use importers::chordpro::parse as chordpro_parse;
pub use importers::openlyrics::parse as openlyrics_parse;
pub use importers::opensong::parse as opensong_parse;
pub use importers::txt::parse as txt_parse;

/// Shared application state.
pub struct AppState {
    pub audio: Arc<Mutex<AudioEngine>>,
    pub transcription: Arc<Mutex<TranscriptionEngine>>,
    pub vad: Arc<Mutex<VadEngine>>,
    pub conductor: Arc<Mutex<Option<Conductor>>>,
    pub presenter: Arc<Mutex<Box<dyn PresentationController>>>,
    /// User-tunable smart-blanking thresholds. Applied to every new conductor
    /// constructed by load_setlist; can also be hot-swapped into the current
    /// conductor via set_smart_config.
    pub smart_config: Arc<Mutex<SmartConfig>>,
    /// Last global slide index the dispatcher actually sent to the presenter.
    /// Used to translate a `Goto` action into next/prev keypresses or a single
    /// `goto_slide` API call, depending on driver capabilities.
    pub last_dispatched_global: Arc<std::sync::atomic::AtomicI64>,
    pub is_running: Arc<std::sync::atomic::AtomicBool>,
    /// Optional local HTTP API for hardware integrations. None until the user
    /// turns it on in Settings.
    pub http_api: http_api::SharedHttpApi,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // The ring-buffer logger replaces env_logger so generate_diagnostic_report
    // has actual log lines to embed.
    diagnostics::init_logger();

    let audio = Arc::new(Mutex::new(AudioEngine::new()));
    let transcription = Arc::new(Mutex::new(TranscriptionEngine::new()));
    let vad = Arc::new(Mutex::new(VadEngine::new()));
    let conductor = Arc::new(Mutex::new(None::<Conductor>));
    let smart_config = Arc::new(Mutex::new(SmartConfig::default()));
    let last_dispatched_global = Arc::new(std::sync::atomic::AtomicI64::new(-1));
    let http_api = Arc::new(Mutex::new(http_api::HttpApiServer::new()));
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
        conductor: conductor.clone(),
        presenter: presenter.clone(),
        smart_config: smart_config.clone(),
        last_dispatched_global: last_dispatched_global.clone(),
        is_running: is_running.clone(),
        http_api: http_api.clone(),
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
            commands::select_audio_device,
            commands::get_model_status,
            commands::download_model,
            commands::next_slide_manual,
            commands::prev_slide_manual,
            commands::blank_manual,
            commands::jump_to_song,
            commands::list_presenters,
            commands::connect_presenter,
            commands::disconnect_presenter,
            commands::get_presenter_info,
            commands::get_presenter_state,
            commands::parse_song_text,
            commands::parse_song_bytes,
            commands::import_easyworship_db,
            commands::hymnary_search,
            commands::hymnary_fetch,
            commands::discover_presenters,
            commands::pco_verify,
            commands::pco_list_service_types,
            commands::pco_list_plans,
            commands::pco_import_plan,
            commands::get_network_stats,
            commands::generate_diagnostic_report,
            commands::get_smart_config,
            commands::set_smart_config,
            commands::load_model,
            commands::set_language,
            commands::get_language,
            commands::toggle_stage_display,
            commands::get_http_api_status,
            commands::start_http_api,
            commands::stop_http_api,
        ])
        .setup(|app| {
            tray::create_tray(app)?;
            log::info!("Voxxa started successfully");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Voxxa");
}
