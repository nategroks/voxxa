# Architect — Index

## Discovered structure
- Single Tauri process. Rust lib (`voxxa_lib`) owns audio/VAD/transcription/aligner; `commands/mod.rs` exposes 11 IPC commands; frontend is one `index.html` + `main.js` (310 LOC) plus a transparent always-on-top `overlay.html`.
- Data flow: cpal callback → mpsc channel → blocking worker thread → VAD → accumulate ≥5 s → whisper → aligner.update → enigo keypress + Tauri events (`transcription`, `slide-advanced`).
- State: `AppState` with tokio Mutexes around AudioEngine, TranscriptionEngine, VadEngine, Option<LyricsAligner>, AtomicBool is_running.
- Model files in `dirs::data_dir()/voxxa/models`. No config file; defaults hard-coded.

## Active features
- `church-autoadvance` — see `church-autoadvance.md` (to be written after research).

## [DISCOVERED 2026-11-11] Structure of `claude/affectionate-lamport-8al8B`
- `aligner/` (883 LOC) is a **Conductor**: smart-blanking state machine (Listening / Singing / InterVerseSilence / BlankHold), parallel song detection with softmax over per-song partial-ratio plus an absolute `min_song_match` gate, within-song current/next partial-ratio, `manual_step` and `notify_external_blank` sync, `jump_to_song`. Flat slides per song: no sections or repeat-aware transitions (ADR-009's gap). Still fed by 5 s transcript chunks.
- `presenters/` (1,470 LOC): `PresentationController` trait with `Capabilities { can_blank, can_goto_slide, can_query_state }`, drivers keystroke (per-app key profiles), ProPresenter 7.9+ REST (goto + query), ProPresenter 7.0–7.8 WS, FreeShow (goto), OpenLP v2 (query), OpenSong; `capabilities_match_implementations` contract test. No slide-text read-back or in-app playlist import yet.
- `importers/` (1,293 LOC): txt, OpenLyrics, OpenSong, ChordPro, pptx, PDF, .pro7, EasyWorship 6 SQLite. `planning_center/` (290): PCO Services v2 via PAT (Basic auth). `hymnary/`: user-initiated Hymnary.org lookup. `discovery/`: localhost probes + mDNS. `http_api/`: axum, six routes, bearer token. `diagnostics/`: ring-buffer logger + report. `net_stats/`: outbound counter. `vad/`: Silero via `voice_activity_detector`. Stage-display window, tray emergency controls, toasts, welcome overlay, keyboard shortcuts.
- Deviations from accepted ADRs: in-process only (ADR-001), `tauri-plugin-store` dependency still declared though the GUI uses localStorage (ADR-003), Flatpak as the Linux packaging (ADR-008 said AppImage first), 38 Tauri commands and a multi-panel GUI (one-screen spec). Reconciled in ADR-010.
