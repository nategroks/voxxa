# Current Handoff

## User decisions for Q3 (2026-11-12, before the unattended run)
- **Line budget:** add a separate budget row for the `voxxad` binary (`src-tauri/src/bin/voxxad*`) capped at **600** lines, recorded as a deviation in the ADR and in `scripts/loc_budget.py`. The audio-loop extraction into the library must fit the existing rows ("everything else in src-tauri/src" stays at 2500).
- **Pre-approved:** push `q3/base` and open a draft PR against `main`; fix CI forward; merge that PR into `main` once Q3's exit criteria are met and every CI job is green; file Q3 tickets as GitHub issues (label `q3`).
- **ADR-012 (cut list)** is not decided yet; Q3 does not depend on it. Q3's ADR is ADR-013.
- **QA mode** (as in Q2): no microphone (`voxxad --device` is never run), no keystroke dispatch; `--shadow`, `--presenter none` or a local mock presenter only; audio from public-domain WAV files; the Whisper model `%APPDATA%\voxxa\models\ggml-tiny.bin` is present.

---

## From: Q3 → To: Q4 (Recognizer trait and streaming hop)
**Date:** 2026-11-12
**Phase:** Phase 4 Implementation done on `q3/base`; merge on hold → Q4

### What Q3 did
- Added the headless `voxxad` binary over `voxxa_lib` and moved the GUI's listening loop into the library, so the GUI and the CLI share one function. ADR-013 (Accepted) records the design.
- Ran as two lanes (`q3/10x`, `q3/ponytail`) with cross-reviews, a comparator (base 10x, six ponytail grafts), a finisher and three verifier rounds: `reviews/2026-11-12-ponytail-q3.md`.
- Code head: `q3/base` @ **912b978**. All 7 gates pass on Windows; 109 tests (54 unit + 55 integration; `voxxad_events` 20, no Whisper model or audio device needed).
- Exit demo shown with real output: a shadow run printed the actions, replay was byte-identical on them, and the ProPresenter mock received nothing under `--shadow`.
- Record: `decisions/sde/church-autoadvance.md` ("Q3 final"). Tickets: `reviews/2026-11-12-q3-tickets.md` (12 filed as GitHub issues #46–#57, label `q3`: 6 medium, 6 low; Q3-01 closed before filing).

### Q3 close-out (orchestrator)
Verifier round 3 at f2bcea1: verifier A passed, verifier B failed on two items.
1. **Q3-01, closed (912b978):** `shadow_never_connects_to_the_presenter` runs the binary against a test-owned `TcpListener` with `--model nonexistent`; with the `--shadow` guard deleted it fails.
2. **Q3-02 (#46), deferred to Q4:** the GUI event sink has no test (mutations N8b–N8d). **Q4 owns it**: when the streaming hop rewrites `listen.rs` and the sink, make the event mapping pure and test names, payloads and `operator=false`.
3. All 7 gates pass on 912b978 (109 tests).
4. Pre-approved: push `q3/base`, draft PR against `main`, CI (first Linux and macOS run of `listen.rs` and `voxxad_events`; the loop test builds a real `VadEngine`, which falls back to the energy detector if Silero/ORT cannot start), merge when every job is green, file the 12 tickets (label `q3`).

### What Q4 must know
**Where the shared loop lives.**
- `src-tauri/src/listen.rs`: `voxxa_lib::listen::run(rx, run, vad, conductor, transcribe, clock, sink) -> bool`. It returns true when the channel closes while `run` is set (device lost or end of file).
- Callers: the GUI (`commands::start_listening_with_state`, clock `|_| Instant::now()`, sink emits the Tauri events and dispatches through `commands::dispatch`) and `voxxad` (`src-tauri/src/bin/voxxad/main.rs`, sample or wall clock, sink writes JSON lines and dispatches through `dispatch::drive`).
- The recogniser is a closure, `FnMut(&[f32]) -> Result<String>`. That is the ADR-004 seam in its smallest form. Q4's Recognizer trait replaces it; ADR-013's review trigger ("a second recogniser lands") applies, so amend ADR-013 or write a new ADR first.

**The hop loop replaces the 5 s batch there.**
- The batch is the `accumulated.len() < 16000 * 5` block at `listen.rs:140` and the recogniser call after it. Transcription is synchronous inside the loop, so ticks, timers and VAD stall while Whisper runs (Q3-06). In a debug build `--realtime` fell behind (47 s of audio ended at t = 92 s). Measure real-time factor and tick lag in a release build.
- Tests pin today's batch on purpose, so Q4 will change them deliberately, not by accident. In `src-tauri/tests/voxxad_events.rs`: `the_shared_loop_on_real_speech_agrees_with_a_replay_of_its_inputs` (every slice ≥ 80,000 samples; the first transcript comes when the VAD's speech adds up to 5 s) and `a_failed_or_empty_transcript_drops_its_audio_and_the_loop_goes_on` (the recogniser gets the VAD's speech in order and **none of it twice**). An overlapping hop sends audio twice by design, so that assertion has to be restated for the hop.
- The test harness `listen_with` drives the loop with any setlist and a fake recogniser told the slice number. Use it for hop tests; CI has no Whisper model.
- GUI behaviour stayed identical in Q3. A hop changes recognition timing in the GUI too; record that as a deliberate behaviour change in the ADR.

**Event schema v1 and replay rules (ADR-013).**
- One JSON object per line, keys in order `v`, `t`, `type`. `t` is ms since the session started: audio time at file speed (sample clock), wall time rounded to whole ms with `--realtime` and `--device`.
- `status` (`start`, `listening`, `singing`, `inter_verse_silence`, `blank_hold`, `stop`), `hypothesis` (`text`), `action` (`goto` with `song`, `slide`, `global`, `title`; `blank`; `unblank`), `error` (`message`) go to stdout and the session log. `tick` and `speech` go to the session log only. That is 6 of ADR-001's 8 types. A new type (for example a partial hypothesis) may be added within 8. Changing an existing type's meaning needs `"v":2`.
- Every conductor input is an `Input` (`Tick`, `Speech`, `Heard`) applied through `Conductor::apply(&Input, now)`, by the loop and by replay. Replay applies the logged `tick`, `speech` and `hypothesis` lines at `base + t` and must stay byte-identical on action lines without the model.
- Speech lines are coalesced to the last one before each tick or hypothesis. This is exact only while `Conductor::on_speech` just records `now` (noted on the method). If the hop makes speech matter between ticks, log every speech input instead.
- The start line carries `SmartConfig` and the setlist fingerprint; replay refuses another setlist, a bad line and a `v` other than 1.
- The committed fixture `tests/fixtures/voxxad/session-pd-hymns.jsonl` is a real session. If Q4 changes what the conductor does with the same inputs, re-record it deliberately and say why in the commit.
- Q4's `voxxa-eval` reads these lines. The conductor can repeat a goto to the slide already shown (Q3-08): dispatch drops it, but an eval would count it.

**Budget numbers (after Q3).**
| Row | Used / cap | Left |
|---|---|---|
| everything else in src-tauri/src (holds `listen.rs`) | 2,496 / 2,500 | 4 |
| conductor (`aligner/`) | 1,153 / 1,200 | 47 |
| voxxad binary (`src/bin/voxxad*`, new in Q3, user decision) | 531 / 600 | 69 |
| presenters | 1,149 / 2,000 | 851 |
| importers + planning_center | 1,362 / 1,600 | 238 |
| GUI | 2,951 / 3,000 | 49 |

The hop loop goes in `listen.rs`, which has 4 lines left. Q4 needs an explicit budget decision before code: move recogniser code into a module with its own recorded row, or ask the user. Never raise a budget silently. Tests in `src-tauri/tests/` are not counted.

**Other things to keep.**
- One dispatch path: `dispatch::drive`. `--shadow` and `--presenter none` never build a presenter.
- Q2 behaviour changes stay (QA report §6): blank on the second unmatched transcript, keystroke focus guard, per-song goto with a blank on an unconfirmed song change, 50-song cap, 16 kHz resampling of device audio.
- Safety rules for unattended runs are unchanged: no microphone, no keystroke dispatch, a local mock presenter only, stop every process by PID.
- Watch out: text written through a Bash heredoc on this PC can turn `\0`, `\n` and `\v` into control bytes (it put NULs into a test file and a vertical tab into this file in Q3). Write such text with the Edit or Write tools and check with `git grep -lIP '[\x00\x0b]'` (ticket Q3-12).

### Still open from earlier steps
- **Founder checklist** before any pilot leaves shadow mode: QA report §10 (M1–M14) and security review §10, plus Q3-07 (start and stop listening in the GUI, unplug, and one `voxxad --device default --shadow` run on the booth PC). First clear `%LOCALAPPDATA%\com.voxxa.app`; keep `%APPDATA%\voxxa\models\ggml-tiny.bin`.
- **User decision:** the cut-list ADR-012 (evidence in the QA report §9).
- **Q1 A15 follow-ups:** `fixtures/setlists/pd-hymns.json` and `fixtures/pco/plan-sample.json` don't fit the May parsers; re-run the Windows verification (a003992) on the merged head (`handoff-local-windows.md`); finish reconciling `docs/signing-policy.md` with `SIGNING.md`.

**Process rule (user, 2026-11-11):** every prompt (Q2…Q27) also runs independently through ponytail. Run it as two lanes plus cross-review for code steps, or `ponytail` + `ponytail-audit` for verification and close steps. Record the result in `reviews/YYYY-MM-DD-ponytail-<Q>.md`.

---

## Handoff History
- 2026-11-12 Q2 → Q3 (handed to the user for ADR-012, then Q3): Q2 verified the merged base in simulate mode and fixed every crash, data-loss, freeze and wrong-slide defect found, with tests (code head 713f40f after the ponytail-audit re-run, merged into `main` as 7bd84a1); 43 tickets filed as issues #2–#44; the full section, with the unattended-run needs and the cut-list evidence, is in this file's git history (f2bcea1).
- 2026-11-11 Q2: QA + security verification in simulate mode; 18 fix-now items + verifier finding Q2-44 fixed with tests; two fix lanes (10x, ponytail) → 10x base with ponytail grafts; 34 tickets; ADR-011 stage window compliant by default; cut-list evidence for ADR-012.
- 2026-11-11 Q1: May branch merged per ADR-010 + A1–A15; two lanes (10x, ponytail) → 10x base with ponytail grafts; all local gates green; build.yml YAML fixed.
- 2026-11-11 Q0 gate: the user accepted ADR-010 as amended; ADR-011; cuts deferred until after Q2; Q2 widened to cover wrong-slide defects.
- 2026-11-11 Q0: claims verified, ADR-010 path table + amendments A1–A15, ponytail lane; stopped at the user gate.
- 2026-11-11 re-planning: May branch discovered; ADR-010 Proposed; prompt pack regenerated for /10x-Team:10x-team.
- 2026-10-24 init → Discovery: `.10x/` created.
- 2026-10-24 Discovery → Strategy: codebase scanned; critical finding: model never loaded, VAD never initialized.
- 2026-10-24 Strategy → Design → Planning: specs + ADR-001…009 + prompt plan written in one pass after five-angle research.
