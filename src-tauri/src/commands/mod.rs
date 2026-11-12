use crate::aligner::{Action, Conductor, MachineState, Setlist, SmartConfig, Song};
use crate::discovery::{self, DiscoveredService};
use crate::hymnary::{HymnaryClient, HymnaryResult};
use crate::importers::{self, ImportFormat};
use crate::net_stats::{self, NetworkStats};
use crate::planning_center::{PcoClient, PcoPlan, PcoServiceType};
use crate::presenters::{
    make_controller, Capabilities, CtlError, KeystrokeProfile, PresentationController,
    PresenterConfig, PresenterKind, PresenterState,
};
use crate::transcription::WhisperModel;
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{Emitter, State};
use tokio::sync::Mutex;

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
    // Reject degenerate setlists at the door rather than half-load them. An
    // empty setlist leaves the conductor in Listening forever; a song with
    // zero slides commits internally but can't fire a Goto, so the operator
    // sees song-detection events with no slide motion.
    if songs.is_empty() {
        return Err("Setlist is empty — no songs to load.".into());
    }
    if let Some(bad) = songs.iter().find(|s| s.slides.is_empty()) {
        return Err(format!(
            "Song \"{}\" has no slides — every song needs at least one.",
            bad.title
        ));
    }
    let cfg = state.smart_config.lock().await.clone();
    let conductor = Conductor::new(songs.clone(), cfg);
    *state.conductor.lock().await = Some(conductor);
    state.dispatched.reset();
    log::info!("Loaded setlist with {} songs", songs.len());
    Ok(songs)
}

/// Apply an inline slide edit in place, so the conductor keeps its position.
#[tauri::command]
pub async fn update_slide_text(
    state: State<'_, AppState>,
    song: usize,
    slide: usize,
    text: String,
) -> Result<(), String> {
    let mut conductor = state.conductor.lock().await;
    match conductor
        .as_mut()
        .map(|c| c.update_slide_text(song, slide, text))
    {
        Some(true) => Ok(()),
        _ => Err("That slide is not in the loaded setlist.".into()),
    }
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
    let (session, rx) = {
        let mut audio = state.audio.lock().await;
        state
            .listening
            .start(|| audio.start().map_err(|e| e.to_string()))?
    };

    let vad = state.vad.clone();
    let transcription = state.transcription.clone();
    let conductor = state.conductor.clone();
    let presenter = state.presenter.clone();
    let dispatched = state.dispatched.clone();
    let listening = state.listening.clone();
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
        // Auto-advance has no caller to return an error to; show it as a toast.
        let report = |r: Result<(), String>| {
            if let Err(e) = r {
                let _ = app_handle.emit("dispatch-error", e);
            }
        };

        while session.load(Ordering::SeqCst) {
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
                report(dispatch_action_with_total(
                    &rt_handle,
                    action,
                    &presenter,
                    &dispatched,
                    &app_handle,
                    total,
                    false,
                ));

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
                                    report(dispatch_action_with_total(
                                        &rt_handle,
                                        action,
                                        &presenter,
                                        &dispatched,
                                        &app_handle,
                                        total,
                                        false,
                                    ));
                                }
                                Ok(_) => {}
                                Err(e) => log::error!("Transcription error: {}", e),
                            }
                        }
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    log::warn!("[AUDIO] capture channel disconnected");
                    disconnected = true;
                    break;
                }
            }
        }
        // Ends only this session: a newer one keeps running.
        listening.finished(&session);
        let _ = app_handle_for_cleanup.emit(
            "listening-stopped",
            serde_json::json!({ "disconnected": disconnected }),
        );
    });

    Ok(())
}

/// The current listening session's run flag. Each session has its own flag,
/// so an old loop still winding down can never stop a newer session.
#[derive(Default)]
pub struct Listening(std::sync::Mutex<Option<Arc<AtomicBool>>>);

impl Listening {
    fn current(&self) -> std::sync::MutexGuard<'_, Option<Arc<AtomicBool>>> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn is_running(&self) -> bool {
        self.current()
            .as_ref()
            .is_some_and(|f| f.load(Ordering::SeqCst))
    }

    /// Claim a new session, then run `open` (the audio start). Nothing stays
    /// claimed when `open` fails, so a failed start never blocks the next one.
    pub fn start<T>(
        &self,
        open: impl FnOnce() -> Result<T, String>,
    ) -> Result<(Arc<AtomicBool>, T), String> {
        let mut current = self.current();
        if current.as_ref().is_some_and(|f| f.load(Ordering::SeqCst)) {
            return Err("Already running".into());
        }
        let opened = open()?;
        let flag = Arc::new(AtomicBool::new(true));
        *current = Some(flag.clone());
        Ok((flag, opened))
    }

    pub fn stop(&self) {
        if let Some(f) = self.current().take() {
            f.store(false, Ordering::SeqCst);
        }
    }

    /// A session's loop ended; it is cleared only if it is still the current one.
    pub fn finished(&self, flag: &Arc<AtomicBool>) {
        flag.store(false, Ordering::SeqCst);
        let mut current = self.current();
        if current.as_ref().is_some_and(|c| Arc::ptr_eq(c, flag)) {
            *current = None;
        }
    }
}

/// What Voxxa last sent to the presenter. Reset when the setlist or the
/// presenter changes.
pub struct Dispatched {
    /// Global slide index last shown; -1 when unknown.
    pub global: AtomicI64,
    /// Song index last addressed; -1 when none.
    pub song: AtomicI64,
    /// Voxxa's last blank command (true = blanked). False at start: nothing sent yet.
    pub blank: AtomicBool,
}

impl Default for Dispatched {
    fn default() -> Self {
        Self {
            global: AtomicI64::new(-1),
            song: AtomicI64::new(-1),
            blank: AtomicBool::new(false),
        }
    }
}

impl Dispatched {
    pub fn reset(&self) {
        self.global.store(-1, Ordering::SeqCst);
        self.song.store(-1, Ordering::SeqCst);
    }
}

/// The keystroke driver types into whichever window has focus; never into Voxxa's own.
fn check_focus(p: &dyn PresentationController, voxxa_focused: bool) -> Result<(), CtlError> {
    if voxxa_focused && p.kind() == PresenterKind::Keystroke {
        return Err(CtlError::Other(
            "Click your presentation app first: Voxxa's key presses would land in its own window."
                .into(),
        ));
    }
    Ok(())
}

/// Send blank/unblank only when it changes what Voxxa last sent, so a toggle
/// key never inverts the screen. A non-toggle blank is always re-sent.
async fn set_blank(
    p: &dyn PresentationController,
    d: &Dispatched,
    blank: bool,
) -> Result<(), CtlError> {
    if d.blank.load(Ordering::SeqCst) == blank && (p.blank_toggles() || !blank) {
        return Ok(());
    }
    if blank {
        p.blank().await?;
    } else {
        p.unblank().await?;
    }
    d.blank.store(blank, Ordering::SeqCst);
    Ok(())
}

async fn presentation_is(p: &dyn PresentationController, title: &str) -> bool {
    p.current_state()
        .await
        .ok()
        .and_then(|s| s.presentation_name)
        .is_some_and(|n| n.trim().eq_ignore_ascii_case(title.trim()))
}

/// Send one conductor action to the presenter. `voxxa_focused`: a Voxxa window
/// has keyboard focus. `operator_song`: the operator picked this song, which
/// confirms it for jump drivers that cannot report their open presentation.
pub async fn apply_action(
    p: &dyn PresentationController,
    d: &Dispatched,
    action: &Action,
    voxxa_focused: bool,
    operator_song: bool,
) -> Result<(), CtlError> {
    let Action::Goto {
        song_index,
        slide_in_song,
        global_index,
        song_title,
        ..
    } = action
    else {
        if matches!(action, Action::Noop) {
            return Ok(());
        }
        check_focus(p, voxxa_focused)?;
        return set_blank(p, d, matches!(action, Action::Blank)).await;
    };
    check_focus(p, voxxa_focused)?;
    let (song, target) = (*song_index as i64, *global_index as i64);
    let caps = p.capabilities();
    if caps.can_goto_slide {
        // Jump drivers address a slide in the app's open presentation: send the
        // slide within the song, and only once that song is confirmed open.
        let same_song = d.song.load(Ordering::SeqCst) == song;
        if !same_song
            && !presentation_is(p, song_title).await
            && !(operator_song && !caps.can_query_state)
        {
            let _ = set_blank(p, d, true).await;
            return Err(CtlError::Other(format!(
                "Open \"{song_title}\" in your presentation app, then pick it in Voxxa."
            )));
        }
        if !same_song || d.global.load(Ordering::SeqCst) != target {
            p.goto_slide(*slide_in_song as u32).await?;
        }
        set_blank(p, d, false).await?;
    } else {
        // Show first: on a blacked-out screen a slide key only un-blacks it.
        set_blank(p, d, false).await?;
        let prev = d.global.load(Ordering::SeqCst);
        // From an unknown position the operator is assumed to be on the slide.
        let delta = if prev < 0 { 0 } else { target - prev };
        for i in 0..delta.unsigned_abs() {
            if i > 0 {
                tokio::time::sleep(Duration::from_millis(60)).await;
            }
            if delta > 0 {
                p.next_slide().await?;
            } else {
                p.prev_slide().await?;
            }
        }
    }
    d.global.store(target, Ordering::SeqCst);
    d.song.store(song, Ordering::SeqCst);
    Ok(())
}

/// Manual next/prev when no song is active: a raw step in the presentation app.
pub async fn step_raw(
    p: &dyn PresentationController,
    d: &Dispatched,
    delta: i64,
    voxxa_focused: bool,
) -> Result<(), CtlError> {
    check_focus(p, voxxa_focused)?;
    if delta > 0 {
        p.next_slide().await?;
    } else {
        p.prev_slide().await?;
    }
    // Keep the count so a later lyric Goto computes a sane delta.
    d.global.fetch_add(delta, Ordering::SeqCst);
    Ok(())
}

fn voxxa_focused(app: &tauri::AppHandle) -> bool {
    use tauri::Manager;
    app.webview_windows()
        .values()
        .any(|w| w.is_focused().unwrap_or(false))
}

/// Run `action` from a blocking thread and tell the UI about a slide change.
fn dispatch_action_with_total(
    rt: &tokio::runtime::Handle,
    action: Action,
    presenter: &Arc<Mutex<Box<dyn crate::PresentationController>>>,
    dispatched: &Dispatched,
    app_handle: &tauri::AppHandle,
    total_slides_hint: Option<usize>,
    operator_song: bool,
) -> Result<(), String> {
    if matches!(action, Action::Noop) {
        return Ok(());
    }
    let focused = voxxa_focused(app_handle);
    let res = rt.block_on(async {
        let p = presenter.lock().await;
        apply_action(&**p, dispatched, &action, focused, operator_song).await
    });
    if let Err(e) = res {
        log::warn!("presenter action failed: {e}");
        return Err(e.to_string());
    }
    if let Action::Goto {
        global_index,
        slide_text,
        next_slide_text,
        song_title,
        ..
    } = action
    {
        let _ = app_handle.emit(
            "slide-advanced",
            SlideAdvanced {
                slide_index: global_index,
                // Fall back to (global+1) only when the caller didn't pass the
                // true total — it's a harmless underestimate for the manual
                // command paths that don't have conductor access on hand.
                total_slides: total_slides_hint.unwrap_or(global_index + 1),
                slide_text,
                next_slide_text,
                song_title,
            },
        );
    }
    Ok(())
}

/// Stop listening.
#[tauri::command]
pub async fn stop_listening(state: State<'_, AppState>) -> Result<(), String> {
    stop_listening_with_state(&state).await;
    Ok(())
}

/// Stop from any path (GUI, HTTP API).
pub async fn stop_listening_with_state(state: &AppState) {
    state.listening.stop();
    let mut audio = state.audio.lock().await;
    audio.stop();
    let mut vad = state.vad.lock().await;
    vad.reset();
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
        is_running: state.listening.is_running(),
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
    crate::transcription::download_and_load(
        &state.transcription,
        &model,
        &model.download_url(),
        move |downloaded, total| {
            let percent = if total > 0 {
                (downloaded as f32 / total as f32) * 100.0
            } else {
                0.0
            };
            let _ = app.emit(
                "download-progress",
                DownloadProgress {
                    downloaded,
                    total,
                    percent,
                },
            );
        },
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
        // No active song: a raw step in the presentation app.
        let focused = voxxa_focused(app);
        let p = state.presenter.lock().await;
        return step_raw(&**p, &state.dispatched, delta as i64, focused)
            .await
            .map_err(|e| e.to_string());
    }
    dispatch_blocking(state, app, action, total, false).await
}

async fn dispatch_blocking(
    state: &AppState,
    app: &tauri::AppHandle,
    action: Action,
    total: Option<usize>,
    operator_song: bool,
) -> Result<(), String> {
    let rt = tokio::runtime::Handle::current();
    let presenter = state.presenter.clone();
    let dispatched = state.dispatched.clone();
    let app_handle = app.clone();
    tokio::task::spawn_blocking(move || {
        dispatch_action_with_total(
            &rt,
            action,
            &presenter,
            &dispatched,
            &app_handle,
            total,
            operator_song,
        )
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn blank_manual(state: State<'_, AppState>, app: tauri::AppHandle) -> Result<(), String> {
    blank_now(&state, &app).await
}

/// Operator blank from any path (button, tray, HTTP API): one place, so the
/// toggle state and the focus guard cannot drift between copies.
pub async fn blank_now(state: &AppState, app: &tauri::AppHandle) -> Result<(), String> {
    let focused = voxxa_focused(app);
    {
        let p = state.presenter.lock().await;
        apply_action(&**p, &state.dispatched, &Action::Blank, focused, false)
            .await
            .map_err(|e| e.to_string())?;
    }
    // Tell the conductor we blanked, so the next lyric match fires an
    // unblank instead of seeing is_blank=false and assuming we're already
    // showing.
    if let Some(c) = state.conductor.lock().await.as_mut() {
        c.notify_external_blank();
    }
    Ok(())
}

/// Operator override: jump straight to a specific song's first slide. Used
/// when the service order diverges from the loaded setlist.
#[tauri::command]
pub async fn jump_to_song(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    song_index: usize,
) -> Result<(), String> {
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
    dispatch_blocking(&state, &app, action, total, true).await
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
    // Nothing has been sent to the new driver yet.
    state.dispatched.reset();
    state.dispatched.blank.store(false, Ordering::SeqCst);
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
/// <token>`; with no token given one is generated. Returns the token in use.
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
    if visible {
        win.hide().map_err(|e| e.to_string())?;
        Ok(false)
    } else {
        // No set_focus: keystrokes must keep going to the presentation app.
        win.show().map_err(|e| e.to_string())?;
        Ok(true)
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

/// Runs an importer on a blocking worker: a sync command would run on the UI
/// thread, where a parser panic aborts the app and a big file freezes it.
async fn off_ui_thread<T: Send + 'static>(
    parse: impl FnOnce() -> anyhow::Result<T> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(parse)
        .await
        .map_err(|_| "the importer crashed on this file".to_string())?
        .map_err(|e| e.to_string())
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
