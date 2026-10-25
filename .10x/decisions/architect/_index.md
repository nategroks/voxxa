# Architect — Index

## Discovered structure
- Single Tauri process. Rust lib (`voxxa_lib`) owns audio/VAD/transcription/aligner; `commands/mod.rs` exposes 11 IPC commands; frontend is one `index.html` + `main.js` (310 LOC) plus a transparent always-on-top `overlay.html`.
- Data flow: cpal callback → mpsc channel → blocking worker thread → VAD → accumulate ≥5 s → whisper → aligner.update → enigo keypress + Tauri events (`transcription`, `slide-advanced`).
- State: `AppState` with tokio Mutexes around AudioEngine, TranscriptionEngine, VadEngine, Option<LyricsAligner>, AtomicBool is_running.
- Model files in `dirs::data_dir()/voxxa/models`. No config file; defaults hard-coded.

## Active features
- `church-autoadvance` — see `church-autoadvance.md` (to be written after research).
