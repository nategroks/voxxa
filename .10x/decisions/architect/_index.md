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

## 2026-11-13 — Q6: ADR-016, ADR-009 minimum implemented
- **Decision:** the conductor at 733c7e1 missed the budget (1.67 false advances per 10 sung songs, p95 11.23 s), measured by two independent lanes; so ADR-009's minimum was built.
- **Built (`q6/base`, 4edd7f6; merge on hold):** a position tracker in `aligner/evidence/` (section labels, repeat-aware priors, a 12-word score, an accumulator with decay and `min_dwell_ms`) behind the unchanged conductor API; optional `sections[].kind` in setlist v1; Q5-04 fixed. False advances 1 → 0, p95 11.23 → 4.87 s (budget 2.5 s still missed). One fixture's latency cells got worse because its recall rose; the user decides the baseline (ADR-016, second amendment, item 5). Records: `church-autoadvance.md` (Q6), `reviews/2026-11-13-q6-eval-before-after.md`, tickets `reviews/2026-11-13-q6-tickets.md`.
- **Close-out (2026-11-18, final head 97cc21b):** the user approved the jump row's 1,500 / 6,000 cells and the `total` row's blank 7,500 / unblank 2,500. A re-run of the ponytail lane built a smaller tracker without the Q5-04 change; the comparator kept `q6/base` and grafted tests and docs only. zip 9.0.5 and `time` (replacing chrono) cleared two new advisories. Both verifiers passed; next is CI, PR and merge. Its scoring idea for slides being entered is ticket Q6-15.

## 2026-11-19 — Q8: ADR-017, Simple mode in front of the existing panels
- **Decision (ADR-017, Accepted on the user's binding Q8 decisions):** Simple mode is the default view, built in `src/simple/` in front of the unchanged `#app`. It has a mode switch (Scripture disabled), a status light with one sentence, the song and slide, Call for help, and an Advanced link. It implements ADR-010 Decision 4.
  - The light is a pure function (`statusLight`, 10 rows, first match wins) of existing signals only: `get_status`, `get_presenter_info` and the existing events.
  - Off and Songs click the existing Listen button.
  - The welcome overlay is replaced by a first-run flow: device, presenter, model (Tiny), and a 20-second test that passes only on a slide change the presenter took.
  - The saved presenter is reconnected at start, except the keystroke driver and kinds that need a secret.
- **Boundaries kept:** `src-tauri/` has no diff (no command changed or added), no panel was deleted, and `main.js` gained one line. Call for help calls no command, because the diagnostics report carries transcripts (Q7-08, #102). The operator window only shows titles and slide numbers (ADR-011).
- **Budget:** a new row "Simple mode (src/simple/*)" capped at 600, a recorded deviation from ADR-010 Decision 3 (352 / 600 at ee431ce). The GUI row shrank from 2,951 to 2,780 / 3,000.
- **Found:** `main.css`'s `display: flex` beat the `hidden` attribute on every start, so the diagnostic modal covered the window. A global `[hidden]` rule in `src/simple/simple.css` fixes it.
- **ADR-017 is ahead of the code on one point:** § 1 (line 29) and Consequences (line 111) say the set-up guide stops every key. At ee431ce it stops only keys aimed inside the guide (`src/simple/firstrun.js:31`), so with the focus on the page body, `main.js`'s shortcuts still act. This is ticket Q8-10, the merge blocker. Keep the ADR's wording and fix the code. Also, § 2 (line 36) gives the `machine-state` event priority over the poll, which the code does not do (Q8-12, a wording fix).
- **Status:** implemented on `q8/base` @ ee431ce. Verifier round 3: B passed, A failed (Q8-10). Q8-10 fixed in 88aa63b and fce3e58 and verified, so ADR-017 § 1 now matches the code. Records: `reviews/2026-11-19-ponytail-q8.md`, `reviews/2026-11-19-q8-tickets.md`, and `decisions/{sde,qa}/church-autoadvance.md` ("Q8 final"). Review triggers: Q9 wires Call for help, Increment 4′ enables Scripture, the founder's run fails a task, or the pilots show that amber `blank_hold` hides a real failure.

## 2026-11-19 — Q9: ADR-018, Call for help, runbook links, quick-start
- **Decision (ADR-018, Accepted on the user's binding Q9 decisions):** Call for help builds a help bundle in the backend, shows every file on a review screen, and writes one zip to the Desktop only on Save. Then it opens the zip's folder and an e-mail draft with no recipient. The red rows of ADR-017's light (and the grey "model missing" row) link a runbook on GitHub. `docs/quick-start.md` is the one-page card for research objective O7.
  - **#102 first:** `listen.rs` logs a word count, never the text, so no recognised text reaches the diagnostics buffer or the report.
  - **The bundle:** `diagnostic-report.json`, `operator-settings.json` (only `voxxa.*` keys, secrets redacted by property name at any depth, whole setlists left out) and, when a `voxxad` session log exists, `words-last-2-minutes.txt`, controlled by a checkbox that is on by default. Unchecked, the words file is absent.
  - **The previewed files stay in the backend** as the pending bundle, and Save writes exactly those, with `create_new`. The webview cannot choose what is written or opened: `open_help_email` takes no argument, and `open_runbook` accepts three names.
  - **Opening** goes through `tauri-plugin-shell`'s `open`, called from Rust (deprecated, no capability change, no new crate; Q9-03). No network call.
  - **QA override** `VOXXA_HELP_DIR`: write there, read `<dir>/sessions`, open nothing.
- **ADR-017 amended** (§ 1: Call for help is the review screen; § 2: the runbook column on rows 1, 3, 5 and 6, no sentence changed).
- **Boundaries kept:** four new commands, no existing command signature changed, no capability change, no new crate. Budgets: everything else in `src-tauri/src` 2,494 / 2,500 (**6 left**), Simple mode 436 / 600, GUI 2,780 / 3,000 unchanged. No cap raised, no row added.
- **Architect notes for later steps:**
  - The backend row is nearly full. A later step that adds code to `src-tauri/src` outside its own rows must remove lines first or ask the user for a budget decision before it writes code.
  - The words file depends on `voxxad`'s logs; the GUI writes none (#53, Q9-02). A GUI session log is an architecture decision (where, how long, permissions; #49, #99), not a Q9 follow-up.
  - CSP is still null (#8). Q10 sets a strict CSP. Call for help's dialog renders file content with `textContent` and has one static `innerHTML` string (`src/simple/help.js:38`), so it needs no inline-script allowance.
- **Status:** implemented on `q9/base` @ 89f4ccb; both verifiers passed in round 3. Records: `reviews/2026-11-19-ponytail-q9.md`, `reviews/2026-11-19-q9-tickets.md` (16 open: 1 medium, 15 low), and `decisions/{sde,qa,security}/church-autoadvance.md` ("Q9 final"). Review triggers (ADR-018): the support address exists (Q9-01), the GUI writes session logs (#53, Q9-02), the shell plugin drops `open` (Q9-03), or a pilot's bundle misses what support needed.

## 2026-11-20 — R0: ADR-019, WorshipTools Presenter and licensed test audio
- [STATED 2026-11-20] **F1** The pilot (LHMEZPC01) runs WorshipTools Presenter; first-run step 2 says "No presentation app found…" (`src/simple/firstrun.js:107`). Confirmed.
- [STATED 2026-11-20] **F2** Discovery probes only ProPresenter 1025 plus mDNS, FreeShow 5506, OpenLP 4316 and OpenSong 8082 (`src-tauri/src/discovery/mod.rs:48-53`, `:159`). Presenter has no local API (ADR-006:6). Confirmed.
- [STATED 2026-11-20] **F3** Presenter's shortcuts (WorshipTools docs, cited from the spec): Right/Space/Page Down next, Left/Page Up previous, Down/Up and numpad +/− service item, F10/F11/F12 clear slide/background/all; no blank toggle.
- [STATED 2026-11-20] **F4** `WorshipToolsPresenter` shares the default map (Right, Left, `.`; `presenters/keystroke.rs:83-88`). The only focus check is `own_window_focused` (`keystroke.rs:42`, called at `commands/mod.rs:284`); ADR-006:11's "only when the target app is foreground" is not implemented. Confirmed.
- [STATED 2026-11-20] **F5/F6** "See a Victory" (Elevation Worship, CCLI Song # 7129060) is in Presenter. Presenter cannot export songs; lyrics come only from Planning Center or the church's own text file (ADR-007).
- **Keystroke dispatch walks the flat setlist** (`dispatch.rs:141-160`): crossing a song boundary presses Right, which in Presenter is "next service item" (T2). ADR-019 § 3 stops that for this profile.
- **Budget for Q9W:** presenters 1149/2000; everything else 2494/2500; one new recorded row allowed (D5).

## 2026-11-20 — Q9W: ADR-019 §§ 1-4 and 10 implemented
- **Decision (ADR-019, Accepted; four amendments in Q9W):** WorshipTools Presenter is found by its process (`discovery/mod.rs:43`, a `Process` source) and driven by the keystroke driver with its own profile. Keys go only to Presenter's window (a pure guard inside `KeystrokeDriver::tap`). A song change presses nothing and asks 'Put "<song>" live in Presenter.', with the new song's first slide recorded (`dispatch.rs:151-156`, a per-driver `stops_at_song_end` flag). `can_blank` comes from the key map (Right, Left, no blank), and dispatch skips a conductor blank that the driver cannot do. A saved non-universal keystroke form is reconnected at start. **Amended 2026-11-24 (#183):** every Presenter song opens on a blank slide, so the flag is now `songs_open_on_blank` and `presenters::step_from` counts the blanks: Voxxa presses Right for a song's first lyrics and steps into the next song in the setlist by itself; only a change back or past a song asks.
- **Amendments made in Q9W:** the Phase 2 check (discovery row capped at 250; "everything else" lowered 2500 → 2317 so its headroom stayed 6; the guard passes off Windows); the base merge (the GUI follows the song change through `tell_gui`; § 4 is shared, so ProPresenter 7/6 over WebSocket skip the conductor's blank too; the asks are info toasts); the two verifier rounds (the guard tested at its call site; `merge` and `discover_with` pure and tested).
- **Notes:**
  - "Everything else in src-tauri/src" is 2315 / 2317: Q10's Rust needs lines found or a budget decision first (Q9W-04).
  - The conductor never moves Presenter between service items, but a multi-key step stopped halfway by the guard can leave Voxxa behind the screen (Q9W-01), and a missed ask still lets the next Right reach the next item (ADR-019 § 3 limit, Q9W-03).
  - The guard covers this profile only; ADR-006's "only when the target app is foreground" still does not hold for the other keystroke profiles.
- **Status:** implemented on `q9w/base` @ b0de71b; both verifiers passed in round 3. Records: `reviews/2026-11-20-ponytail-q9w.md`, `reviews/2026-11-20-q9w-tickets.md` (12 open: 2 medium, 10 low), and `decisions/{sde,qa,security}/church-autoadvance.md` ("Q9W final"). Review triggers (ADR-019): the founder confirms the F10 unblank key (Q9W-02), keys fail at the pilot more than once a service (reopen MIDI), or WorshipTools ships a local API.

## 2026-11-23 — Q10: ADR-020, operator-initiated updates, a strict CSP and draft releases
- **Decision (ADR-020, Accepted on the founder's Q10 decisions of 2026-11-23; amends ADR-008 "Windows" and "Updates"):**
  - **Updates (option B):** only the operator's **Check for updates** in Advanced → Settings reaches the network for an update; never at startup, on a timer or in Simple mode, and never while listening (refused before any request, and again after the download). Voxxa's own commands `check_for_update` and `install_update` call `tauri-plugin-updater` from Rust; the webview gets **no** updater permission (`capabilities/default.json` stays `core:event:default`). A pure comparator (`updater::should_update`, semver precedence, strictly greater) is the plugin's default and the instance's; install takes only the version the operator confirmed and says Voxxa will restart. Endpoint `releases/latest`, so drafts and pre-releases are never offered.
  - **CSP:** a strict string (`default-src 'self'`, no `unsafe-eval`, no `unsafe-inline`; `connect-src ipc: http://ipc.localhost`), with a Vite-only `devCsp`. The three inline style attributes were removed.
  - **Release pipeline:** every tag build is a **draft** (pre-release for a `-` tag); the tag's base version must equal `tauri.conf.json`, so an rc installs as X.Y.Z. Windows: NSIS `currentUser` plus MSI, WebView2 `embedBootstrapper`; signing SignPath (all four values) → else Azure (all six) → else unsigned, then Authenticode checked, then the updater signatures over the final bytes. One job writes `latest.json`. winget only after the founder publishes a full release. Every action pinned by SHA.
  - **Budget (§ 5):** one new recorded row, "updater (src/updater/)", cap 150, following ADR-019 § 10's precedent (D5). "Everything else" keeps its cap of 2,317.
- **Amendments made in Q10** (all in the ADR text, d7ef67d and f424c74): SHA-pinned actions; Azure only with all six values; the `latest.json` job gated on a `yes`/`no` output and required to cover four platforms; `ReleaseNotFound` gets its own sentence, never "up to date"; the rc note; each `APPLE_*` exported only when set; the new source-text tests.
- **Notes:**
  - Budget at f424c74: updater 98 / 150; everything else 2316 / 2317 (one line left); GUI 2859 / 3000; Simple mode 450 / 600.
  - Residual: `latest.json`'s version is unsigned until `requireSignedVersion` can be turned on (Q10-03). The inner `voxxa.exe` of the NSIS installer is unsigned on the SignPath path (Q10-04). ADR-008's "stable" channel is not built (Q10-06).
  - `timestampUrl` stays plain http (DigiCert documents no https endpoint; RFC 3161 replies are signed), a recorded deviation.
  - The release jobs are unproven until the orchestrator's `v0.1.0-rc.1` tag run, and the signing paths until the founder's secrets exist.
- **Status:** implemented on `q10/base` @ f424c74; both verifiers passed in round 2. Records: `reviews/2026-11-23-ponytail-q10.md`, `reviews/2026-11-23-q10-tickets.md` (10 open: 2 medium, 8 low), `decisions/{sde,qa,security,devops}/church-autoadvance.md` ("Q10 final"), `docs/RELEASING.md`. Review triggers (ADR-020): the Tauri CLI signs the version (`requireSignedVersion`), SignPath accepts or declines the project, the first full release is published (prove an update from it to the next), or churches ask for an automatic check (an opt-in setting, off by default).

## 2026-11-25 — V0: ADR-021 § 1 Accepted, the #59 release flags
- **Decision (ADR-021 § 1, Accepted by the founder, FA-01, 2026-11-25; §§ 2-3 Proposed, § 4's status open):** on MSVC the `cmake` crate 0.1.58 writes whisper.cpp's `CMAKE_{C,CXX}_FLAGS_RELEASE` without `/O2`, and whisper-rs-sys always builds Release, so every Windows release build ran Whisper unoptimised (Tiny on the private song P1: 461 s of compute, 130 s fixed). `build.yml`'s Windows job sets both variables to `-MD -O2 -Ob2 -DNDEBUG` in its `env:` and nowhere else (`-Ob2` breaks gcc and clang). Its cache key is `release-portable-ggml-o2`. "Check that whisper.cpp was built with -O2" (`scripts/ci/check-msvc-release-flags.sh`) runs right after the installer build.
- **Guards:** the check's fixture test runs on every PR (`ci.yml`, the `budgets` job and the `check` job's Windows leg). `release_config.rs` pins the env lines, the key and the step, and refuses the variables in every other workflow and in a `.cargo/config(.toml)`. Debug builds (all of `ci.yml` and `full-eval.yml`) are already optimised and stay unchanged.
- **Notes:**
  - No Rust change and no budget row: nothing under `src-tauri/src` or the GUI changed.
  - Local release builds need `scripts/dev/msvc-release-flags.ps1` dot-sourced, and nothing warns without it (V0-02).
  - ADR-021's references to ADR-022, FA items and plan steps (V3, V4, V6) point to the 2026-11-24 strategy draft, not yet committed.
- **Status:** built on `fix/59-msvc-release-flags`, not merged. Records: `reviews/2026-11-25-ponytail-v0.md` (the lane record, the adjudication and the verifier round), `reviews/2026-11-25-v0-tickets.md` (3 low candidates), `decisions/{devops,qa}/church-autoadvance.md` ("V0"). Proof still open: the next founder-approved rc draft's Windows job must pass the check. Review triggers (ADR-021): Small ≤ 1.0 s per pass on CPU, the Vulkan proof fails its bar, or the check fails on a runner image update.
