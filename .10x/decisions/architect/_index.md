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
- **Built (Q3, 0669328 + 2f588df):** as decided; budgets everything else 2,496/2,500, voxxad 504/600. Replay of the exit-demo session is byte-identical on actions and is a CI test. Open risk: keystroke from a terminal has no focus guard (ADR-013 Risks).
- **Final (q3/base @ f2bcea1):** the 10x lane with ponytail grafts. ADR-013 was amended for privacy (the roaming `%APPDATA%`) and release packaging (Q3-03, was Q3-T1). Budgets: everything else 2,496/2,500, voxxad 531/600. Merge on hold: verifier round 3 found no test for the `--shadow` guard (Q3-01) or for the GUI event sink (Q3-02). Q4 puts the ADR-004 recogniser trait and the streaming hop into `listen::run`, which triggers ADR-013's review clause ("a second recogniser lands").

## Q4 Phase 2 check (2026-11-12): recogniser trait and streaming hop → ADR-014
- **Trigger:** ADR-013's review clause ("a second recogniser lands"); the closure becomes ADR-004's trait. ADR-013 carries a dated amendment pointing to ADR-014.
- **Decided in ADR-014 (Accepted on the user's pre-approved Q4 decisions):** `Recognizer { feed, bias, gate (provided) }` and `Hypothesis { words (absolute stream seconds), end, error }` in `transcription/recognizer.rs`; Whisper behind it as `Rolling<D: Decode>` (5 s window, 1 s hop, `single_segment`, token timestamps, `initial_prompt` from `bias()` only with `bias_enabled`, off by default); the batch in `listen::run` becomes the hop loop; the gate opens only on VAD speech in the last second and when the loop is at most one hop behind (skip, never catch up; a sample clock is never behind, so file speed stays deterministic).
- **Unchanged:** conductor API and code, event schema v1, replay rules, one dispatch path. The `hypothesis` cadence goes from one per ~5 s of speech to about one per second; the PD session fixture is re-recorded.
- **GUI:** only the recognition cadence changes; the sink becomes a pure, tested mapping (`commands::gui_event`, Q3-02 / #46).
- **Budget:** new row "recognizer (transcription/ + recognizer code)" capped at 700, a recorded deviation from ADR-010 Decision 3; everything else stays at 2,500.
- **Built (Q4, cc59ffb + 77b0b6f on `q4/10x`):** as decided. Exit demo with Whisper Tiny: a hypothesis every second of speech, goto Amazing Grace/0 at 6.0 s (was 8.1 s), goto /1 at 22.0 s, blank 2.7 s into the prayer, nothing shown during it; replay byte-identical. Budgets: everything else 2,270/2,500, recognizer 374/700. Open: a hop costs more than 1 s on the dev PC (debug ~2 s; release slower, suspected `opt-level = "s"` reaching whisper.cpp), so real-time hypotheses come every 2-5 s via the skip rule; details in `decisions/sde/church-autoadvance.md` (Q4).
- **Final (q4/base @ a4bae0e, 2026-11-13):** the 10x lane with three ponytail grafts and the review fixes; verifier round 3 passed (two verifiers). Changes to the design above, all recorded in ADR-014's dated notes:
  - a pure `Stitcher` sends the conductor only the words a window adds, held until there are 4, so the stitched `hypothesis` line comes every 1-3 s of speech;
  - a session-log-only `window` line (one per decoded hop, every 1000 ms of speech at file speed) is the 7th of ADR-001's 8 event types, additive in v1 and skipped by replay;
  - three conductor behaviour fixes, API unchanged: blank after 5 s of unmatched transcripts, also on a tick; no goto to the slide still showing after inter-verse silence (closes Q3-08 / #52);
  - word text decoded per segment (UTF-8 safe).
  - Budgets: everything else 2,286/2,500, recognizer 432/700, conductor 1,188/1,200 (12 left), voxxad 548/600. Root cause of the slow release build found (whisper.cpp built without `/O2` on MSVC, ticket Q4-01). Process: `reviews/2026-11-12-ponytail-q4.md`.

## Q5 Phase 2 check (2026-11-13): `voxxa-eval` and the first fixtures → ADR-015
- **Covered by ADR-013/014:** file-speed `t` is audio time; schema v1; `--replay` re-runs the conductor without a model, byte-identical on actions; a recogniser change needs logs re-recorded.
- **Not covered, decided in ADR-015 (Accepted on the user's pre-approved Q5 decisions):**
  - a third binary, `src/bin/voxxa-eval/`, runs the `voxxad` beside it as a subprocess (replay and the WAV reader live in the `voxxad` binary, so in-process would mean copying them) and scores only its `action` lines;
  - two modes: replay of a committed `session.jsonl` (default, CI, no model) and `--full` (`--stdin-audio --shadow --model tiny --log-dir .` in a temporary folder; `--record` re-records the log);
  - fixtures: every folder in `fixtures/recordings/` (the kit's folder), with `setlist.json`, `truth.jsonl`, `session.jsonl`, and `audio.json` (a part list over the committed WAVs) or an uncommitted `audio.wav`;
  - `truth.jsonl`: `{t_ms, song (index or title), slide}` and `{t_ms, "blank": true}`; `event` lines skipped; rows outside the setlist rejected;
  - metrics: truth row *i* owns `[t_i − 1000, t_{i+1} − 1000)`; precision = hits ÷ all gotos; false and double advances; missed slides; worst blank/unblank latency per row (`miss` fails any threshold); nearest-rank median and p95;
  - `fixtures/baseline.json` holds per-row thresholds (`{"<fixture>|total": {"<metric>": <worst accepted>}}`); exit 1 names the failing metric, exit 2 on an error; CI runs it as a step of the `check` job on all three OSes;
  - budget: a new 600-line "voxxa-eval" row, a recorded deviation from ADR-010 Decision 3.
- **Boundaries unchanged:** no GUI change, no conductor change, no sockets, no new crates; the eval only ever passes `--shadow` or `--replay`.
- **Built (Q5, `q5/base`):** the ponytail lane's code (`q5/ponytail`, merged), with grafts and review fixes from the 10x lane and the comparator: `voxxa-eval` 458/600 (469/600 at the final head c10e963, after the verifier fixes; ADR-015 Decision 8 amended so a metric with a value and no threshold fails; merge on hold until verifier B's test gaps Q5-08 … Q5-10 are closed); two stand-in fixtures (`prayer-then-reprise`, `prayer-without-pause`); the CI step. First numbers (precision and recall 1.00, latency median 2.9 s and p95 8.9 s, blank 6.6 s into a prayer with no pause) and the Q5-04 finding from the 10x lane (a song after an opening prayer is not shown until its next verse): `decisions/sde/church-autoadvance.md` (Q5), tickets `reviews/2026-11-13-q5-tickets.md`.
