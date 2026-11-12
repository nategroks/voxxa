use crate::aligner::{Action, Conductor, MachineState, Setlist, SmartConfig, Song};
use crate::discovery::{self, DiscoveredService};
use crate::dispatch::{self, Sent};
use crate::hymnary::{HymnaryClient, HymnaryResult};
use crate::importers::{self, ImportFormat};
use crate::net_stats::{self, NetworkStats};
use crate::planning_center::{PcoClient, PcoPlan, PcoServiceType};
use crate::presenters::{
    make_controller, Capabilities, KeystrokeProfile, PresenterConfig, PresenterKind, PresenterState,
};
use crate::transcription::WhisperModel;
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{Emitter, State};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatusInfo {
    pub is_running: bool,
    pub model_loaded: bool,
    /// `None` when no song has been detected yet — distinguishes "not started"
    /// from "started at slide 0". Frontend uses this to decide whether to show
    /// the slide counter at all.
    pub current_slide: Option<usize>,
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
    /// Text of the slide that comes after `slide_index`. `None` at the end
    /// of the current song. The Stage Display uses this for its Next panel
    /// so it doesn't need to ping the backend.
    pub next_slide_text: Option<String>,
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

#[derive(Debug, Serialize, Clone)]
pub struct MicLevel {
    /// Peak amplitude in this window, 0.0–1.0. Clipped to 1.0 if the mic is hot.
    pub peak: f32,
    /// RMS energy in the same window — what the VAD effectively sees.
    pub rms: f32,
}

/// One row of the conductor's per-song confidence table, denormalised with
/// the song title so the frontend can render without a second lookup.
#[derive(Debug, Serialize, Clone)]
pub struct DetectionRow {
    pub song_index: usize,
    pub song_title: String,
    pub probability: f64,
    pub raw_score: f64,
}

#[derive(Debug, Serialize, Clone)]
pub struct DetectionEvent {
    pub rows: Vec<DetectionRow>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub display_name: String,
    pub downloaded: bool,
    pub loaded: bool,
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
    crate::aligner::check_setlist(&songs)?;
    let cfg = state.smart_config.lock().await.clone();
    let conductor = Conductor::new(songs.clone(), cfg);
    *state.conductor.lock().await = Some(conductor);
    // A new setlist numbers slides afresh; the blank state is the presenter's.
    let mut sent = state.sent.lock().await;
    (sent.global, sent.song) = (-1, None);
    log::info!("Loaded setlist with {} songs", songs.len());
    Ok(songs)
}

/// The operator's inline slide edit: replace one slide's text in the running
/// conductor without reloading it, so position and blank state are kept.
#[tauri::command]
pub async fn update_slide_text(
    state: State<'_, AppState>,
    global_index: usize,
    text: String,
) -> Result<(), String> {
    let mut conductor = state.conductor.lock().await;
    let c = conductor.as_mut().ok_or("No setlist loaded")?;
    if !c.update_slide_text(global_index, text) {
        return Err(format!("No slide {global_index} in the setlist"));
    }
    Ok(())
}

#[tauri::command]
pub async fn get_smart_config(state: State<'_, AppState>) -> Result<SmartConfig, String> {
    Ok(state.smart_config.lock().await.clone())
}

/// Update smart-blanking thresholds. Hot-swaps into the active conductor so
/// the change takes effect immediately without losing position.
#[tauri::command]
pub async fn set_smart_config(state: State<'_, AppState>, cfg: SmartConfig) -> Result<(), String> {
    *state.smart_config.lock().await = cfg.clone();
    if let Some(c) = state.conductor.lock().await.as_mut() {
        c.set_config(cfg);
    }
    Ok(())
}

/// Start listening: drives VAD → Whisper → Conductor → presenter.
#[tauri::command]
pub async fn start_listening(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    start_listening_with_state(&state, &app).await
}

/// Same as `start_listening` but callable from any code path that has a
/// borrow of AppState — the HTTP API routes use this so they don't have to
/// re-implement the audio loop.
pub async fn start_listening_with_state(
    state: &AppState,
    app: &tauri::AppHandle,
) -> Result<(), String> {
    {
        let c = state.conductor.lock().await;
        if c.is_none() {
            return Err("No setlist loaded. Load a setlist first.".to_string());
        }
    }
    {
        let t = state.transcription.lock().await;
        if t.current_model().is_none() {
            return Err(
                "No Whisper model loaded. Download one from the Models tab first.".to_string(),
            );
        }
    }
    let (rx, run) = {
        let mut audio = state.audio.lock().await;
        state
            .sessions
            .begin(|| audio.start().map_err(|e| e.to_string()))?
    };

    let vad = state.vad.clone();
    let transcription = state.transcription.clone();
    let conductor = state.conductor.clone();
    let sessions = state.sessions.clone();
    let app_handle = app.clone();
    // Capture the runtime handle so spawn_blocking can call back into async-land
    // without building a fresh runtime each iteration.
    let rt_handle = tokio::runtime::Handle::current();

    let app_handle_for_cleanup = app.clone();
    tokio::task::spawn_blocking(move || {
        let mut accumulated: Vec<f32> = Vec::new();
        let mut last_state = MachineState::Listening;
        let mut last_blank = true;
        let mut last_tick = Instant::now();
        let mut last_mic_emit = Instant::now();
        let mut peak_window: f32 = 0.0;
        let mut sumsq_window: f64 = 0.0;
        let mut samples_window: usize = 0;
        const TICK_EVERY: Duration = Duration::from_millis(200);
        // ~33 Hz UI updates — fast enough to look live, cheap enough to ignore.
        const MIC_EMIT_EVERY: Duration = Duration::from_millis(33);
        // `disconnected` distinguishes a clean stop (operator hit stop) from
        // the audio device dropping out underneath us. The frontend listens
        // for `listening-stopped` so it can re-arm the start button either way.
        let mut disconnected = false;
        let send = |action: Action, total: Option<usize>| {
            if let Err(e) = rt_handle.block_on(dispatch(&app_handle, action, total, false)) {
                log::warn!("[DISPATCH] {e}");
                let _ = app_handle.emit("presenter-error", e);
            }
        };

        while run.load(Ordering::SeqCst) {
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
                send(action, total);

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
                    // Mic level: track peak + sum-of-squares across the
                    // emission window. Cheap to compute (single pass over the
                    // chunk) and worth surfacing so operators can verify mic
                    // signal at a glance.
                    for &s in &samples {
                        let a = s.abs();
                        if a > peak_window {
                            peak_window = a;
                        }
                        sumsq_window += (s as f64) * (s as f64);
                    }
                    samples_window += samples.len();
                    if now.duration_since(last_mic_emit) >= MIC_EMIT_EVERY {
                        let rms = if samples_window > 0 {
                            (sumsq_window / samples_window as f64).sqrt() as f32
                        } else {
                            0.0
                        };
                        let _ = app_handle.emit(
                            "mic-level",
                            MicLevel {
                                peak: peak_window.min(1.0),
                                rms,
                            },
                        );
                        peak_window = 0.0;
                        sumsq_window = 0.0;
                        samples_window = 0;
                        last_mic_emit = now;
                    }

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
                                    let (action, total, detection) = {
                                        let mut cl = conductor.blocking_lock();
                                        match cl.as_mut() {
                                            Some(c) => {
                                                let act = c.on_transcript(&r.text, now);
                                                let titles = c.song_titles();
                                                // Top 5 candidates is enough — beyond
                                                // that the UI bars get noisy.
                                                let rows: Vec<DetectionRow> = c
                                                    .last_scores()
                                                    .iter()
                                                    .take(5)
                                                    .map(|s| DetectionRow {
                                                        song_index: s.song_index,
                                                        song_title: titles
                                                            .get(s.song_index)
                                                            .cloned()
                                                            .unwrap_or_default(),
                                                        probability: s.probability,
                                                        raw_score: s.raw_score,
                                                    })
                                                    .collect();
                                                (act, Some(c.total_slides()), rows)
                                            }
                                            None => (Action::Noop, None, Vec::new()),
                                        }
                                    };
                                    if !detection.is_empty() {
                                        let _ = app_handle.emit(
                                            "song-detection",
                                            DetectionEvent { rows: detection },
                                        );
                                    }
                                    send(action, total);
                                }
                                Ok(_) => {}
                                Err(e) => log::error!("Transcription error: {}", e),
                            }
                        }
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    // Stop also closes the channel; only a loss while this
                    // session should still run is a device loss.
                    disconnected = run.load(Ordering::SeqCst);
                    log::warn!("[AUDIO] capture channel closed (device lost: {disconnected})");
                    break;
                }
            }
        }
        // Reset the running state on loop exit (also after a device dropout),
        // unless a newer session has started meanwhile.
        if !sessions.end(&run) {
            // A newer session started meanwhile; this one's end is not news.
            return;
        }
        let _ = app_handle_for_cleanup.emit(
            "listening-stopped",
            serde_json::json!({ "disconnected": disconnected }),
        );
    });

    Ok(())
}

/// True when a Voxxa window has the keyboard focus, so synthetic keys would
/// land in Voxxa itself.
fn voxxa_focused(app: &tauri::AppHandle) -> bool {
    use tauri::Manager;
    crate::presenters::keystroke::foreground_is_voxxa()
        || app
            .webview_windows()
            .values()
            .any(|w| w.is_focused().unwrap_or(false))
}

/// Send one action to the presenter (see `dispatch::drive`) and tell the GUI.
/// Every path that changes the audience screen comes through here: the
/// listening loop, the buttons, the tray and the HTTP API. `operator` marks
/// the last three.
pub async fn dispatch(
    app: &tauri::AppHandle,
    action: Action,
    total_slides: Option<usize>,
    operator: bool,
) -> Result<(), String> {
    use tauri::Manager;
    if matches!(action, Action::Noop) {
        return Ok(());
    }
    let (presenter, sent) = {
        let state = app.state::<AppState>();
        (state.presenter.clone(), state.sent.clone())
    };
    let focused = voxxa_focused(app);
    let p = presenter.lock().await;
    let mut sent = sent.lock().await;
    dispatch::drive(p.as_ref(), &mut sent, &action, focused, operator)
        .await
        .map_err(|e| e.to_string())?;
    if let Action::Goto {
        global_index,
        slide_text,
        next_slide_text,
        song_title,
        ..
    } = action
    {
        let _ = app.emit(
            "slide-advanced",
            SlideAdvanced {
                slide_index: global_index,
                total_slides: total_slides.unwrap_or(global_index + 1),
                slide_text,
                next_slide_text,
                song_title,
            },
        );
    }
    Ok(())
}

/// Blank the audience output now: the operator's Blank button, the tray and
/// the HTTP API. The conductor is told, so its next lyric match unblanks.
pub async fn blank_output(app: &tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;
    dispatch(app, Action::Blank, None, true).await?;
    let conductor = app.state::<AppState>().conductor.clone();
    if let Some(c) = conductor.lock().await.as_mut() {
        c.notify_external_blank();
    }
    Ok(())
}

/// The single listening session. Each session has its own run flag: stop
/// clears the current one, and a loop runs while its own flag is set, so a
/// slow old loop cannot end a new session.
#[derive(Default)]
pub struct Sessions {
    running: AtomicBool,
    current: std::sync::Mutex<Arc<AtomicBool>>,
}

impl Sessions {
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Claim the session atomically and run `start`. On error the claim is
    /// released, so a failed start never leaves "Already running" behind.
    pub fn begin<T>(
        &self,
        start: impl FnOnce() -> Result<T, String>,
    ) -> Result<(T, Arc<AtomicBool>), String> {
        if self.running.swap(true, Ordering::SeqCst) {
            return Err("Already running".into());
        }
        match start() {
            Ok(v) => {
                let run = Arc::new(AtomicBool::new(true));
                *self.current.lock().unwrap() = run.clone();
                Ok((v, run))
            }
            Err(e) => {
                self.running.store(false, Ordering::SeqCst);
                Err(e)
            }
        }
    }

    /// A session's loop ended: clear `running` only if it is still current.
    /// False when a newer session has started, so the GUI is not told.
    pub fn end(&self, run: &Arc<AtomicBool>) -> bool {
        run.store(false, Ordering::SeqCst);
        let current = Arc::ptr_eq(&self.current.lock().unwrap(), run);
        if current {
            self.running.store(false, Ordering::SeqCst);
        }
        current
    }

    /// Stop the current session (its loop exits on its next check).
    pub fn stop(&self) {
        let run = self.current.lock().unwrap().clone();
        let _ = self.end(&run);
    }
}

/// Stop listening: the GUI button and the HTTP API.
pub async fn stop_session(state: &AppState) {
    // Take the audio lock first, so a Start in progress finishes before it
    // is stopped instead of having its new stream dropped underneath it.
    let mut audio = state.audio.lock().await;
    state.sessions.stop();
    audio.stop();
    drop(audio);
    state.vad.lock().await.reset();
}

#[tauri::command]
pub async fn stop_listening(state: State<'_, AppState>) -> Result<(), String> {
    stop_session(&state).await;
    Ok(())
}

/// Get current status.
#[tauri::command]
pub async fn get_status(state: State<'_, AppState>) -> Result<StatusInfo, String> {
    let conductor = state.conductor.lock().await;
    let (current_slide, total_slides, machine_state, is_blank, song_title) = match &*conductor {
        Some(c) => (
            // None when no song is yet detected — `current_index()` returns 0
            // in that case which would be misleading on a status badge.
            c.current_song().map(|_| c.current_index()),
            c.total_slides(),
            Some(c.state()),
            Some(c.is_blank()),
            c.current_song_title().map(|s| s.to_string()),
        ),
        None => (None, 0, None, None, None),
    };
    let model_loaded = state.transcription.lock().await.current_model().is_some();
    Ok(StatusInfo {
        is_running: state.sessions.is_running(),
        model_loaded,
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

/// Pick the input device to capture from. `None` (or empty string) means use
/// the OS default. Takes effect on the next `start_listening` call — we don't
/// hot-swap mid-capture because that would drop the audio buffer and confuse
/// the conductor.
#[tauri::command]
pub async fn select_audio_device(
    state: State<'_, AppState>,
    device: Option<String>,
) -> Result<(), String> {
    let normalised = device.filter(|s| !s.is_empty());
    let mut audio = state.audio.lock().await;
    audio.select_device(normalised);
    Ok(())
}

#[tauri::command]
pub async fn get_model_status(state: State<'_, AppState>) -> Result<Vec<ModelInfo>, String> {
    let transcription = state.transcription.lock().await;
    Ok(transcription
        .model_status()
        .into_iter()
        .map(|(m, downloaded, loaded)| ModelInfo {
            name: format!("{:?}", m),
            display_name: m.display_name().to_string(),
            downloaded,
            loaded,
        })
        .collect())
}

/// Load a downloaded Whisper model into memory so transcription can run.
/// Must be called before start_listening — otherwise the audio loop errors
/// on every Whisper call.
#[tauri::command]
pub async fn load_model(state: State<'_, AppState>, model_name: String) -> Result<(), String> {
    let model = parse_model(&model_name)?;
    let mut transcription = state.transcription.lock().await;
    transcription.load_model(&model).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_language(
    state: State<'_, AppState>,
    language: Option<String>,
) -> Result<(), String> {
    let mut transcription = state.transcription.lock().await;
    // Treat empty string the same as "auto-detect" so the UI's `<option value="">`
    // doesn't accidentally set the language to an empty Whisper language code.
    let normalised = language.filter(|s| !s.is_empty());
    transcription.set_language(normalised);
    Ok(())
}

#[tauri::command]
pub async fn get_language(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let transcription = state.transcription.lock().await;
    Ok(transcription.language().map(String::from))
}

fn parse_model(name: &str) -> Result<WhisperModel, String> {
    Ok(match name {
        "Tiny" => WhisperModel::Tiny,
        "Base" => WhisperModel::Base,
        "Small" => WhisperModel::Small,
        "Medium" => WhisperModel::Medium,
        "LargeV3Turbo" => WhisperModel::LargeV3Turbo,
        "DistilLargeV3" => WhisperModel::DistilLargeV3,
        other => return Err(format!("Unknown model: {other}")),
    })
}

#[tauri::command]
pub async fn download_model(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    model_name: String,
) -> Result<(), String> {
    let model = parse_model(&model_name)?;
    let app_handle = app.clone();
    let progress = move |downloaded, total| {
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
    };
    // Downloads without holding the transcription lock, then auto-loads the
    // model if none is loaded, so the operator need not press "Use".
    crate::transcription::download_and_load(
        &state.transcription,
        &model,
        &model.download_url(),
        progress,
    )
    .await
    .map_err(|e| e.to_string())
}

/// Manually advance one slide. Routes through the conductor when a song is
/// active so its internal slide counter stays in sync (otherwise the next
/// lyric-triggered Goto would compute the wrong delta). When no song has been
/// detected yet, falls back to a raw presenter keystroke.
#[tauri::command]
pub async fn next_slide_manual(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    manual_step(&state, &app, 1).await
}

#[tauri::command]
pub async fn prev_slide_manual(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    manual_step(&state, &app, -1).await
}

pub async fn manual_step(
    state: &AppState,
    app: &tauri::AppHandle,
    delta: i32,
) -> Result<(), String> {
    refuse_keys_from_voxxa(state, app).await?;
    let (action, total) = {
        let mut c = state.conductor.lock().await;
        match c.as_mut() {
            Some(cd) => (
                cd.manual_step(delta, Instant::now()),
                Some(cd.total_slides()),
            ),
            None => (Action::Noop, None),
        }
    };
    if matches!(action, Action::Noop) {
        // No active song: one raw key, recorded in `sent` (dispatch::step).
        let p = state.presenter.lock().await;
        let mut sent = state.sent.lock().await;
        return dispatch::step(p.as_ref(), &mut sent, delta, voxxa_focused(app))
            .await
            .map_err(|e| e.to_string());
    }
    // Next/Prev do not confirm that the app shows a new song; only the song
    // chip does (see dispatch::drive).
    dispatch(app, action, total, false).await
}

/// With the keystroke driver, refuse an operator step while Voxxa has the
/// focus before the conductor moves, so the conductor never runs ahead of
/// a screen that did not move.
async fn refuse_keys_from_voxxa(state: &AppState, app: &tauri::AppHandle) -> Result<(), String> {
    let p = state.presenter.lock().await;
    dispatch::refuse_if_focused(p.as_ref(), voxxa_focused(app)).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn blank_manual(app: tauri::AppHandle) -> Result<(), String> {
    blank_output(&app).await
}

/// Operator override: jump straight to a specific song's first slide. Used
/// when the service order diverges from the loaded setlist.
#[tauri::command]
pub async fn jump_to_song(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    song_index: usize,
) -> Result<(), String> {
    refuse_keys_from_voxxa(&state, &app).await?;
    let (action, total) = {
        let mut c = state.conductor.lock().await;
        match c.as_mut() {
            Some(cd) => (
                cd.jump_to_song(song_index, Instant::now()),
                Some(cd.total_slides()),
            ),
            None => return Err("No setlist loaded".into()),
        }
    };
    dispatch(&app, action, total, true).await
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
    // Nothing has been sent to the new presenter yet.
    *state.sent.lock().await = Sent::default();
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

/// Snapshot of outbound HTTP request count for the privacy UI.
#[tauri::command]
pub fn get_network_stats() -> NetworkStats {
    net_stats::snapshot()
}

#[derive(Debug, Serialize)]
pub struct HttpApiStatus {
    pub running: bool,
    pub port: u16,
}

#[tauri::command]
pub async fn get_http_api_status(state: State<'_, AppState>) -> Result<HttpApiStatus, String> {
    let api = state.http_api.lock().await;
    Ok(HttpApiStatus {
        running: api.is_running(),
        port: api.port(),
    })
}

/// Turn the local HTTP API on. Every request needs `Authorization: Bearer
/// <token>`; a blank `token` makes Voxxa generate one. Returns the token in
/// force so the GUI can show it.
#[tauri::command]
pub async fn start_http_api(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    port: u16,
    token: Option<String>,
) -> Result<String, String> {
    let mut api = state.http_api.lock().await;
    api.start(app, port, token).await
}

#[tauri::command]
pub async fn stop_http_api(state: State<'_, AppState>) -> Result<(), String> {
    let mut api = state.http_api.lock().await;
    api.stop();
    Ok(())
}

/// Toggle the always-on-top Stage Display companion window. Used by worship
/// leaders on a confidence monitor — second screen showing what Voxxa
/// thinks is happening.
#[tauri::command]
pub fn toggle_stage_display(app: tauri::AppHandle) -> Result<bool, String> {
    use tauri::Manager;
    let win = app
        .get_webview_window("stage")
        .ok_or("Stage Display window not configured")?;
    let visible = win.is_visible().map_err(|e| e.to_string())?;
    show_stage(&win, !visible).map_err(|e| e.to_string())?;
    Ok(!visible)
}

/// Show or hide the Stage Display without taking the keyboard from the
/// presentation app, where the keystroke driver's keys must land. On Windows
/// tao's show() is ShowWindow(SW_SHOW), which activates even a "focusable":
/// false window; hide goes the same way so tao's own flag is never used.
pub fn show_stage(win: &tauri::WebviewWindow, show: bool) -> tauri::Result<()> {
    #[cfg(windows)]
    if let Ok(hwnd) = win.hwnd() {
        #[link(name = "user32")]
        extern "system" {
            fn ShowWindow(hwnd: *mut std::ffi::c_void, cmd: i32) -> i32;
        }
        // SW_SHOWNOACTIVATE = 4, SW_HIDE = 0.
        // SAFETY: a live HWND of this window; ShowWindow only reads it.
        unsafe { ShowWindow(hwnd.0, if show { 4 } else { 0 }) };
        return Ok(());
    }
    if show {
        win.show()
    } else {
        win.hide()
    }
}

/// Build a copy-pasteable diagnostic report (system info + recent log lines).
/// Voxxa never uploads — the user owns where this text goes.
#[tauri::command]
pub async fn generate_diagnostic_report(
    state: State<'_, AppState>,
) -> Result<crate::diagnostics::DiagnosticReport, String> {
    Ok(crate::diagnostics::generate_report(&state).await)
}

/// Test Planning Center credentials. Returns the authenticated user's name.
#[tauri::command]
pub async fn pco_verify(app_id: String, secret: String) -> Result<String, String> {
    let client = PcoClient::new(&app_id, &secret).map_err(|e| e.to_string())?;
    client.verify().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pco_list_service_types(
    app_id: String,
    secret: String,
) -> Result<Vec<PcoServiceType>, String> {
    let client = PcoClient::new(&app_id, &secret).map_err(|e| e.to_string())?;
    client.list_service_types().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pco_list_plans(
    app_id: String,
    secret: String,
    service_type_id: String,
) -> Result<Vec<PcoPlan>, String> {
    let client = PcoClient::new(&app_id, &secret).map_err(|e| e.to_string())?;
    client
        .list_plans(&service_type_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pco_import_plan(
    app_id: String,
    secret: String,
    service_type_id: String,
    plan_id: String,
) -> Result<Vec<Song>, String> {
    let client = PcoClient::new(&app_id, &secret).map_err(|e| e.to_string())?;
    client
        .import_plan(&service_type_id, &plan_id)
        .await
        .map_err(|e| e.to_string())
}

/// Probe localhost ports + browse mDNS for ProPresenter / FreeShow / OpenLP.
/// `timeout_ms` bounds total wall-time; recommended 2000–3000.
#[tauri::command]
pub async fn discover_presenters(
    timeout_ms: Option<u64>,
) -> Result<Vec<DiscoveredService>, String> {
    let timeout = Duration::from_millis(timeout_ms.unwrap_or(2500));
    Ok(discovery::discover_all(timeout).await)
}

/// Run an importer off the UI thread. Sync Tauri commands run on the main
/// thread, so a large file froze the window and a parser panic aborted the
/// app; here a panic becomes an ordinary error.
async fn off_ui_thread<T: Send + 'static>(
    parse: impl FnOnce() -> anyhow::Result<T> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(parse)
        .await
        .map_err(|_| "the importer failed on this file".to_string())?
        .map_err(|e| e.to_string())
}

/// Parse a text-source song (.txt, OpenLyrics, OpenSong, ChordPro). Returned
/// `Song` is appended to the in-flight setlist by the frontend.
#[tauri::command]
pub async fn parse_song_text(
    content: String,
    format: ImportFormat,
    fallback_title: Option<String>,
) -> Result<Song, String> {
    let fallback = fallback_title.unwrap_or_else(|| "Untitled".to_string());
    off_ui_thread(move || importers::parse_text(format, &content, &fallback)).await
}

/// Parse a binary-source song (.pptx, .pdf, .pro7). `bytes_b64` is base64-encoded raw file bytes.
#[tauri::command]
pub async fn parse_song_bytes(
    bytes_b64: String,
    format: ImportFormat,
    fallback_title: Option<String>,
) -> Result<Song, String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(bytes_b64.as_bytes())
        .map_err(|e| format!("invalid base64: {e}"))?;
    let fallback = fallback_title.unwrap_or_else(|| "Untitled".to_string());
    off_ui_thread(move || importers::parse_bytes(format, &bytes, &fallback)).await
}

/// Search Hymnary.org for public-domain hymns. No auth required.
#[tauri::command]
pub async fn hymnary_search(query: String) -> Result<Vec<HymnaryResult>, String> {
    let client = HymnaryClient::new().map_err(|e| e.to_string())?;
    client.search(&query).await.map_err(|e| e.to_string())
}

/// Fetch a Hymnary hymn's full lyric text. Errors when the hymn is still in
/// copyright (Hymnary only returns text for public-domain hymns).
#[tauri::command]
pub async fn hymnary_fetch(slug: String) -> Result<Song, String> {
    let client = HymnaryClient::new().map_err(|e| e.to_string())?;
    client.fetch_song(&slug).await.map_err(|e| e.to_string())
}

/// Import an EasyWorship 6 SQLite library file. Returns every song in the
/// database — unlike the other importers, one .db file is a whole multi-song
/// library, not a single song.
#[tauri::command]
pub async fn import_easyworship_db(bytes_b64: String) -> Result<Vec<Song>, String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(bytes_b64.as_bytes())
        .map_err(|e| format!("invalid base64: {e}"))?;
    off_ui_thread(move || importers::easyworship::parse_database(&bytes)).await
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
