//! In-memory diagnostic report.
//!
//! Captures the last N log lines in a ring buffer and exposes a single
//! `generate_report` call that snapshots system info + recent activity for
//! attachment to a bug report. **No uploading** — the user copy-pastes the
//! generated text into a GitHub issue or email, matching Voxxa's zero-telemetry
//! promise (see §7.3 of the project plan).

use crate::AppState;
use chrono::Utc;
use log::{Log, Metadata, Record};
use serde::Serialize;
use std::collections::VecDeque;
use std::sync::atomic::Ordering;
use std::sync::{Mutex, OnceLock};

const RING_CAPACITY: usize = 500;

struct RingLogger {
    entries: Mutex<VecDeque<String>>,
    /// Threshold below which records are discarded — saves space for the
    /// records that actually matter when something goes wrong.
    level_filter: log::LevelFilter,
}

impl RingLogger {
    fn new(level: log::LevelFilter) -> Self {
        Self {
            entries: Mutex::new(VecDeque::with_capacity(RING_CAPACITY)),
            level_filter: level,
        }
    }

    fn snapshot(&self) -> Vec<String> {
        self.entries
            .lock()
            .map(|q| q.iter().cloned().collect())
            .unwrap_or_default()
    }
}

impl Log for RingLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level_filter
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let line = format!(
            "{} [{}] {} — {}",
            Utc::now().format("%H:%M:%S%.3f"),
            record.level(),
            record.target(),
            record.args()
        );
        // Mirror to stderr so `cargo run`/`tauri dev` consoles still see logs.
        eprintln!("{line}");
        if let Ok(mut q) = self.entries.lock() {
            if q.len() == RING_CAPACITY {
                q.pop_front();
            }
            q.push_back(line);
        }
    }

    fn flush(&self) {}
}

static LOGGER: OnceLock<RingLogger> = OnceLock::new();

/// Install the ring-buffer logger. Call once at startup in place of
/// `env_logger::init`. Subsequent calls are no-ops.
pub fn init_logger() {
    let level = std::env::var("RUST_LOG")
        .ok()
        .and_then(|v| v.parse::<log::LevelFilter>().ok())
        .unwrap_or(log::LevelFilter::Info);
    let logger = LOGGER.get_or_init(|| RingLogger::new(level));
    // set_logger can only be called once per process; ignore the duplicate
    // error if init_logger is somehow invoked twice (e.g. test harness).
    let _ = log::set_logger(logger);
    log::set_max_level(level);
}

#[derive(Debug, Serialize)]
pub struct DiagnosticReport {
    pub generated_at: String,
    pub voxxa_version: String,
    pub os: String,
    pub arch: String,
    pub audio_devices: Vec<String>,
    pub presenter_kind: String,
    pub presenter_connected: bool,
    pub current_song: Option<String>,
    pub current_slide: usize,
    pub total_slides: usize,
    pub machine_state: Option<String>,
    pub is_blank: bool,
    pub is_running: bool,
    pub outbound_requests: u64,
    pub recent_log: Vec<String>,
}

pub async fn generate_report(state: &AppState) -> DiagnosticReport {
    let audio_devices = crate::AudioEngine::list_devices().unwrap_or_default();
    let presenter = state.presenter.lock().await;
    let (presenter_kind, presenter_connected) =
        (presenter.kind().display_name().to_string(), presenter.is_connected());
    drop(presenter);

    let conductor = state.conductor.lock().await;
    let (current_song, current_slide, total_slides, machine_state, is_blank) =
        match conductor.as_ref() {
            Some(c) => (
                c.current_song_title().map(String::from),
                c.current_index(),
                c.total_slides(),
                Some(format!("{:?}", c.state())),
                c.is_blank(),
            ),
            None => (None, 0, 0, None, true),
        };
    drop(conductor);

    DiagnosticReport {
        generated_at: Utc::now().to_rfc3339(),
        voxxa_version: env!("CARGO_PKG_VERSION").to_string(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        audio_devices,
        presenter_kind,
        presenter_connected,
        current_song,
        current_slide,
        total_slides,
        machine_state,
        is_blank,
        is_running: state.is_running.load(Ordering::SeqCst),
        outbound_requests: crate::net_stats::snapshot().outbound_requests,
        recent_log: LOGGER
            .get()
            .map(|l| l.snapshot())
            .unwrap_or_default(),
    }
}
