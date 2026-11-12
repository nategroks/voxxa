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
- `presenters/` (1,470 LOC): `PresentationController` trait with `Capabilities { can_blank, can_goto_slide, can_query_state }`. Drivers (capabilities corrected by Q0):

  | Driver | Capabilities |
  |---|---|
  | keystroke, per-app key profiles | blank only, as a toggle |
  | ProPresenter 7.9+ REST | blank, goto, query |
  | ProPresenter 7.0–7.8 WS, plus a **ProPresenter 6 WS** variant | goto |
  | FreeShow | blank, goto |
  | OpenLP v2 | blank, query |
  | OpenSong | blank as a toggle, goto |

  `capabilities_match_implementations` contract test (one-way only). No slide-text read-back or in-app playlist import yet.
- `importers/` (1,293 LOC): eight file formats: txt, OpenLyrics, OpenSong, ChordPro, pptx, PDF, .pro7, EasyWorship 6 SQLite. `planning_center/` (290): PCO Services v2 via PAT (Basic auth). `hymnary/`: user-initiated Hymnary.org lookup. `discovery/`: localhost probes, plus an mDNS browse for ProPresenter only (hits get a guessed port 1025). `http_api/`: axum, six routes, 127.0.0.1, **optional** bearer token (blank by default = no auth). `diagnostics/`: ring-buffer logger + report. `net_stats/`: outbound counter. `vad/`: Silero via `voice_activity_detector`. Stage-display window, tray emergency controls, toasts, welcome overlay, keyboard shortcuts.
- Deviations from accepted ADRs: in-process only (ADR-001), `tauri-plugin-store` dependency still declared though the GUI uses localStorage (ADR-003), Flatpak as the Linux packaging (ADR-008 said AppImage first), 38 Tauri commands and a multi-panel GUI (one-screen spec). Reconciled in ADR-010.

### Q0 verification (2026-11-11)
The statements above were checked against the code at `f7b56e6`; the full log is in `reviews/2026-11-11-q0-claim-verification.md`. Corrections are made inline above. New [DISCOVERED] items:
- [DISCOVERED] **The conductor does not yet enforce "a bad blank beats a wrong slide".**
  - After a blank, any unmatched speech re-shows the current song's slide (`aligner/mod.rs:331-340`).
  - BlankHold is reached only by silence. `unrecognized_speech_to_blank_secs` is really a second silence timer.
  - `min_song_match = 40` sits below the score random English text gets, and a one-song setlist always passes the softmax gate.
  - `is_blank = true` at start is only a flag; no Blank is ever sent while Listening.
  - Nothing returns to Listening once a song is committed.
- [DISCOVERED] **Dispatch is open-loop and uses the setlist-global slide index.** `goto_slide(global_index)` is read by ProPresenter, FreeShow and OpenSong as an index into the *active presentation*, so it is correct only if one presentation holds the whole setlist in order. Only Goto is gated by capability; Blank and Unblank ignore `can_blank`. The keystroke and OpenSong unblank re-sends the blank toggle. The GUI never reads `Capabilities`.
- [DISCOVERED] **The audio loop runs Whisper synchronously.** While inference runs, `tick()`, the silence and blank timers and VAD all stall. `download_model` holds the transcription lock for the whole download. Capture opens only at 16 kHz; there is no resampler.
- [DISCOVERED] **There are no Tauri capability files** (`gen/schemas/capabilities.json` is `{}`). App commands pass the ACL, but every `plugin:event|listen` is rejected. This is static analysis; one app launch confirms it. CSP is null and `withGlobalTauri` is true.
- [DISCOVERED] **The shell, store, global-shortcut and updater plugins are registered but unused.** Nothing calls the updater.
- [DISCOVERED] **The ring-buffer logger records every transcript** ("Heard: …"), and the diagnostics report embeds it.
- The 12 confirmed must-fix defects are listed in `reviews/2026-11-11-ponytail-q0.md` §4–5. They go to Q2.

## Q3 Phase 2 check (2026-11-12): `voxxad` → ADR-013
- **Covered by ADR-001/003 (as adjusted by ADR-010):** a `voxxad` CLI over `voxxa_lib`, GUI in-process, JSON lines with `"v":1`, no sockets, a rotated 14-day session log, `--replay` for support.
- **Not covered, decided in ADR-013 (Accepted on the user's pre-approved Q3 decisions):**
  - the listening loop moves from `commands::start_listening_with_state` to `voxxa_lib::listen::run`, parameterised by a clock (`samples → Instant`), a recogniser closure (the ADR-004 seam, smallest form) and an event sink; the GUI's sink emits exactly the old Tauri events;
  - every conductor input is a `voxxa_lib::Input` (`Tick`, `Speech`, `Heard`) applied through `Conductor::apply`, by the loop and by replay; `voxxad` at file speed uses a sample clock, so `t` is audio time;
  - event schema v1: `status`, `hypothesis`, `action`, `error` on stdout, plus `tick` and `speech` in the session log (6 of ADR-001's 8);
  - budget: a new 600-line `voxxad` row, a recorded deviation from ADR-010 Decision 3; the loop extraction fits the existing rows.
- **Boundaries unchanged:** one dispatch path (`dispatch::drive`); `--shadow` and `--presenter none` never build a presenter; replay never loads a model or opens audio.
