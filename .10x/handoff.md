# Current Handoff

## User decisions for Q8 (2026-11-18, before the unattended run)
- **Line budget:** Simple mode and the first-run flow go in ONE new recorded row in `scripts/loc_budget.py`, "Simple mode (src/simple/*)", capped at **600**, for a dedicated directory `src/simple/`, subtracted from the GUI row like the other rows. The GUI row (`src/`, `index.html`, `stage.html`) stays capped at 3,000, and the existing panels must not grow: the "Advanced" move should cost few lines (delete before you add). Record the row as a deviation in `loc_budget.py` and the ADR. No other budget is raised, and the GUI stays free of npm runtime dependencies. This supersedes the "49 lines left" warning below.
- **GUI testing, two ways:**
  - (a) Browser with a mocked backend: a test-only `invoke()` and event shim, kept out of the shipped bundle. Listen, the 20-second test and every audio path are exercised **only** here.
  - (b) The real app, guarded: a scratch build that is never committed. Each lane uses its own identifier (never `com.voxxa.app`) and WebView2 remote debugging through `additionalBrowserArgs`. Before the first launch, confirm from the code that startup opens no audio input; if anything does, do not launch. In the real app: never press Listen, never start the 20-second test, never pick the keystroke presenter, never open the stage window on a second display, never enable the HTTP API. Delete the scratch identifier's `%LOCALAPPDATA%` folder afterwards.
- **Usability:** agents walk through the research plan's usability script on the mocked GUI (and on the guarded real app where safe) and record each step. The founder's own run on their machine is a founder checklist item and a Q8 ticket; **the merge does not wait for it**.
- **Backend:** no change to backend commands (signatures or behaviour) and no new commands; delete no panel. The ADR is **ADR-017**.
- **Pre-approved:** push `q8/base`, open a draft PR against `main`, fix CI forward; merge once Q8's exit criteria are met (except the founder's run) and every CI job is green; file Q8 tickets as GitHub issues (label `q8`); agents may stop processes they started, by PID.
- **Content:** public-domain text only; agents never write or echo hymn text (mocks use placeholder words).
- The decisions for Q7 are in git history (c448139).

---

## From: Q8 → To: Q9 (Call for help, runbooks, quick-start)
**Date:** 2026-11-19
**Phase:** Q8 (Phase 4, Simple mode, ADR-017) is done on `q8/base`: head **ee431ce**, plus the state commit "state(implementation): Q8 Simple mode". In verifier round 3 at ee431ce, verifier B passed and verifier A failed (ticket Q8-10, below). The orchestrator fixed it afterwards (88aa63b, fce3e58), and a verifier confirmed the fix. Next: CI, PR and merge (pre-approved above); then Q9.

### What Q8 did
- **Process:** two lanes from f8500e3. `q8/10x` (6 commits, ADR first) was merged as the comparator's base (b4eacca). `q8/ponytail` (4 commits, bdc3348) stays local for reference. Then came the 10x review's fixes, four ponytail grafts and two verifier rounds of fixes, one commit each. Record: `reviews/2026-11-19-ponytail-q8.md`.
- **Built** in `src/simple/`:
  - `logic.js`, pure: `statusLight`, `songPosition`, `testVerdict`.
  - `simple.js`: the view, the mode switch (one toggle at a time) and Call for help.
  - `firstrun.js`: the set-up guide and the start-up reconnect.
  - `simple.css`, including the global `[hidden]` fix.

  Every old panel is unchanged behind Advanced (`#app` hidden), and `main.js` gained one line. Nothing under `src-tauri/` changed.
- **Records:**
  - ADR-017;
  - `decisions/{sde,qa}/church-autoadvance.md` ("Q8 final"; the QA record has the usability walk-through table) and `decisions/architect/_index.md`;
  - `reviews/2026-11-19-q8-tickets.md`: 16 open, 2 medium and 14 low. Q8 closes no earlier issue outright; #7 is fixed in part.

### Q8-10, the merge blocker: fixed 2026-11-19
Q8-10 was fixed by the orchestrator in 88aa63b: a capture-phase `keydown` listener on `window` blocks every key while the guide is shown. Four new e2e checks press Space and `s` with the focus on the page body, on step 1 and during the 20-second test. All four fail on the old code (41 passed, 6 failed) and pass with the fix. An independent verifier confirmed it at 88aa63b: 13 shortcut keys were blocked with the focus on the body, the buttons and the select, and the guide's own keyboard controls still work. The old-code mutant and the bubble-phase mutant both fail. The verifier's one survivor, keys blocked even after the guide closes, is killed by fce3e58 ("s reaches main.js again" after Skip; 117 e2e checks). The original description follows.
The guide's key block sits on the overlay box (`src/simple/firstrun.js:31`). With the focus on the page body, `main.js`'s document shortcuts (`src/main.js:1563`) still act. The focus is on the body when the guide opens on a fresh profile, and during the 20-second test after `#fr-extra` is cleared (`firstrun.js:174`). Space then starts listening behind the guide before a microphone is chosen, or stops the test and gives a false "I couldn't hear anything" verdict, and `s` opens the stage window.

Fix: a capture-phase `window` keydown listener while the guide is shown, plus e2e checks with the focus on `document.body` that fail on ee431ce. Then a verifier confirms. Then push, draft PR, every CI job green (the browser test has never run on GitHub, Q8-04), merge, and file the tickets (label `q8`; Q8-10 only if it is still open). Add the comments to #7, #44, #34, #9, #102 and #37 (tickets file, last section).

### Q9's task (prompt pack, `specs/2026-11-11-prompts-10x-team.md`, Stage C)
**Q9 — Call for help, runbooks, quick-start (S), Phase 4.** Wire Call for help to `generate_diagnostic_report`, plus an export of the operator's `localStorage` state and the last 2 minutes of hypotheses from the session log. The bundle is zipped to the Desktop, shown to the user before it is written, then opened in the support e-mail draft. Write `docs/runbooks/{no-audio-device,presentation-app-not-reachable,model-missing}.md` and link each from the matching red status. Write `docs/quick-start.md` (one page, Windows). No telemetry. Exit: the bundle is produced, and the links resolve.

### What Q9 must know
**Where Call for help lives.**
- The button and its message are in `src/simple/simple.js:24-25`: the `#s-help` button and the hidden `#s-help-msg` paragraph. The handler is at `:35`, and only un-hides the message. In Q8 it calls no command.
- ADR-017 § 1 ("Call for help (Q8)") and its review trigger ("when Q9 wires Call for help") say an amendment is due.
- The e2e checks that Call for help calls no command, and that Space on it shows the message (`tests/gui/e2e.mjs:326`). Q9 changes both checks on purpose.
- Budget: Simple mode is 352 / 600 (248 left) and the GUI row 2,780 / 3,000 (220 left). Put the Call for help code in `src/simple/`.

**Privacy comes first.**
- `generate_diagnostic_report` carries every transcript the GUI heard (Q7-08, **#102**, medium): `log::info!("Heard: …")` at `src-tauri/src/listen.rs:182` goes into the ring buffer, and `diagnostics/mod.rs:153` copies it into the report. That is why Q8 did not wire it. Fix #102 before or with Q9.
- "The last 2 minutes of hypotheses" is transcript text by definition. The GUI also writes no session log (Q3-09, #53); only `voxxad` does. So the task as written needs either a GUI session log or another source.
- The bundle must be shown before it is written (task text), and no heard text may appear anywhere else.
- Ask the user for Q9's decisions before the run, as for Q3–Q8. Those decisions include whether a backend change or a new command is allowed: Q8's "no backend change" was for Q8 only. They also include what goes into the `localStorage` export. The keys are `voxxa.firstLaunchDone`, `voxxa.audioDevice`, `voxxa.presenterForm` (host, port and username; passwords are never stored, `src/main.js:1131-1133`), `voxxa.lastModel`, `voxxa.smartConfig`, `voxxa.language` and `voxxa.httpApi` (port and on/off only). **`voxxa.recentSetlists` holds up to eight whole setlists, slide text included** (`src/main.js:448-477`, about 1 MB at most). A naive `localStorage` export would put song text, which may be copyrighted, into the bundle, so leave that key out or reduce it to titles.

**Linking runbooks from the light.** The status table is `statusLight` in `src/simple/logic.js:7-25`:
- red row 1, mic lost (`:9`) → `no-audio-device`;
- red rows 5 and 6, presenter refused or not found (`:13-19`) → `presentation-app-not-reachable`;
- **"model missing" is not red.** It is grey row 3, "Off. Download the voice model in Advanced first." (`:11`), shown only while Off. Decide where `model-missing` links from (that grey sentence, or the guide's model step), and record it in ADR-017 § 2.

Any sentence change has to be made in four places: `tests/gui/simple_logic.test.mjs`, the e2e's exact-sentence checks, the ADR-017 § 2 table and `docs/quick-start.md`.

**How to test the GUI (Q8's method; keep it).**
- **(a) Mocked browser.** `npm run build && npm run test:gui:e2e -- <screenshot dir>`. `tests/gui/e2e.mjs` serves `dist/` with `tests/gui/mock-tauri.js` injected before the app's scripts, so the shim is never bundled, and the test checks that. It drives headless Edge or Chrome over CDP.
  - Add every command Q9 calls to the mock, with a configurable delay, like `st.startDelay`. A slow command is where Q8's bugs were.
  - Use the e2e's `click()`, which refuses anything a user cannot see (`seen()`). Keep the `onlySimple()` and computed-colour checks.
  - Listen, the 20-second test and every audio path run only here.
  - The pure logic runs with `npm run test:gui` (`node:test`). In CI, the logic test runs on 3 OSes and the browser test on Windows only.
- **(b) The guarded real app**, a scratch build never committed:
  - Write a config outside git with your own identifier (`com.voxxa.qa<name>`, never `com.voxxa.app`) and `additionalBrowserArgs` with `--remote-debugging-port=<port>` on both windows.
  - Build with `npm run tauri build -- --no-bundle --config <scratch>`. Run `python scripts/qa/mock_pro7.py 1025` (only if port 1025 is free), then `VOXXA_CDP_PORT=<port> node scripts/qa/simple-mode-real-app.mjs <dir>`.
  - Before the first launch, re-read the startup path: `audio.start()` is called only from `start_listening` (`commands/mod.rs:229`).
  - Never press Listen, run the 20-second test, choose the keystroke presenter, open the stage window, enable the HTTP API, or press "Use" or "Download" on a model (a failed load renames the model aside, Q2-50).
  - **New for Q9:** a real Call for help may write to the user's real Desktop and open their mail client. Do not press it in the real app unless the user's Q9 decisions allow it. Test the bundle in the mock, or point it at a scratch folder.
  - Afterwards, stop the app and the mock by PID, delete `%LOCALAPPDATA%\com.voxxa.qa<name>`, and restore the LF-only rewrites of `src-tauri/Cargo.toml` and `gen/schemas/*.json` (#91).
  - The release `voxxa.exe` in `C:/dev/q3-10x-target/release/` is a scratch build with a QA identifier. Never ship it.

**Keys.** Q8-10 must be fixed first. Shortcut keys stay live in Simple mode outside its own buttons (Q8-14). A new dialog, such as the bundle preview, needs the capture-phase key block that Q8-10 adds, or `main.js`'s Space shortcut will reach it.

**The quick-start** (`docs/quick-start.md`) is the "quick-start card" of research objective O7 ("download to green with the quick-start card and no call"). Its steps should match the guide in order: microphone, presentation app, model, 20-second test. Its sentences should match the light. The founder's usability run (Q8-01) can then use it.

### Numbers at the finish (Q8)
Replay, exit 0: 21 of 21 hits, precision and recall 1.00, 0 false and 0 double advances, median 2,310 ms, p95 4,870 ms (#84), unchanged from Q6. `baseline.json` unchanged. Tests: 210 passed with `--features cli` (56 unit + 154 integration), 1 ignored. GUI: `test:gui` 6/6, e2e 106/106.

| Row | Used / cap | Left |
|---|---|---|
| everything else in src-tauri/src | 2,309 / 2,500 | 191 |
| conductor (`aligner/`, without `evidence/`) | 1,173 / 1,200 | 27 |
| conductor evidence (`aligner/evidence/`) | 215 / 500 | 285 |
| recognizer | 432 / 700 | 268 |
| voxxad (`src/cli/voxxad/`) | 552 / 600 | 48 |
| voxxa-eval (`src/cli/voxxa-eval/`) | 469 / 600 | 131 |
| presenters | 1,149 / 2,000 | 851 |
| importers + planning_center | 1,386 / 1,600 | 214 |
| GUI (without `src/simple/`) | 2,780 / 3,000 | 220 |
| Simple mode (`src/simple/*`) | 352 / 600 | 248 |

Tests in `src-tauri/tests/`, `tests/gui/` and `scripts/qa/` are not counted.

**Other things to keep.**
- Safety rules for unattended runs: no microphone, no keystroke dispatch, `--shadow`/`--replay`/`--presenter none` only for `voxxad`, and stop every process you start, by PID.
- Build and test the tools with `--features cli`. A plain `cargo test` fails 10 tests on purpose. Never move a tool back to `src-tauri/src/bin/`. The eval commands are in `fixtures/README.md` and `docs/runbooks/eval-regression.md`. Never print setlist or recognised text.
- The shared `CARGO_TARGET_DIR` (`C:/dev/q3-10x-target`): two `cargo test` runs in it at once can crash rustc. Use a separate target for parallel work.
- Write files with the Edit or Write tools, not Bash heredocs (#56). Check `git grep -lIP '[\x00\x0b]'` before every commit.
- CI: keep Linux on `ubuntu-22.04` (the `unshare -rn` comment in `ci.yml`), and keep the Windows socket test's per-child time window.

### Still open from earlier steps
- **Founder checklist** before any pilot leaves shadow mode: QA report Q2 §10 (M1–M14), the security checklist, Q3-07 (#51), and now **the usability script on the founder's machine (Q8-01)**: U1-U13 with a real microphone and presentation app, plus a look through the old panels. First clear `%LOCALAPPDATA%\com.voxxa.app`; keep `%APPDATA%\voxxa\models\ggml-tiny.bin`.
- **Founder:** record the two public-domain hymn fixtures (#73).
- **User decisions:** the cut-list ADR-012; Q6-01 (#83) and Q6-10 (#89).
- **Q1 A15 follow-ups:** Q7-12 (#106); re-run the Windows verification on the merged head (`handoff-local-windows.md`); reconcile `docs/signing-policy.md` with `SIGNING.md`.

**Process rule (user, 2026-11-11):** every prompt (Q2…Q27) also runs independently through ponytail. Run code steps (Q9 is one) as two lanes plus cross-review, and verification and close steps as `ponytail` + `ponytail-audit`. Record the result in `reviews/YYYY-MM-DD-ponytail-<Q>.md`.

---

## Handoff History
- 2026-11-19 Q7 → Q8: Q7 closed Increment 1′ (a determinism test on 3 OSes, a zero-sockets test for `--replay`, the report-only full eval, the tools out of the installers, the CLI release notes, the Companion guide and the runbook); merged into `main` as c448139 (PR #95); 12 tickets filed as #96–#107; #47 and #75 closed; the full section, with the eval commands, the budget table and the CI notes, is in this file's git history (ee431ce).
- 2026-11-19 Q6 → Q7: Q6 built ADR-009's minimum (the evidence tracker, ADR-016) on six synthetic fixtures (false advances 1 → 0, p95 11.23 → 4.87 s), with the user's one-time baseline exception for the jump row; merged into `main` as a60d1ce (PR #82); 12 tickets filed as #83–#94; #76 and #62 closed; the full section, with the eval commands, the determinism trap and the re-recording rules, is in this file's git history (6386f6b).
- 2026-11-13 Q5 → Q6: Q5 added `voxxa-eval` (replay in CI, `--full` with Whisper Tiny locally), two stand-in fixtures, `fixtures/baseline.json` and a CI eval step (ADR-015); merged into `main` as 69f7d27 (PR #72) after the orchestrator closed Q5-08, Q5-09 and Q5-10's M52/M55 with tests (5858872) and proved the gate in real CI (run 43469439549); 9 tickets filed as issues #73–#81; the full section, with the eval commands, the first numbers and the Q5-04 reproduction, is in this file's git history (4edd7f6).
- 2026-11-13 Q4 → Q5: Q4 put Whisper behind ADR-004's `Recognizer` trait as a 5 s window decoded every 1 s, with a stitcher and a session-log `window` line (ADR-014); code head a4bae0e, merged into `main` as 571cf0d (PR #58); 13 tickets filed as issues #59–#71; #46 and #52 closed; the full section, with the schema, cadence, replay and alignment notes for the eval, is in this file's git history (c10e963).
- 2026-11-12 Q3 → Q4: Q3 added the headless `voxxad` CLI and moved the listening loop into `voxxa_lib::listen::run` (ADR-013; schema v1, byte-identical replay); code head 912b978, merged into `main` as ec6d85e (PR #45); 12 tickets filed as issues #46–#57; Q3-02 (#46) handed to Q4; the full section is in this file's git history (a4bae0e).
- 2026-11-12 Q2 → Q3 (handed to the user for ADR-012, then Q3): Q2 verified the merged base in simulate mode and fixed every crash, data-loss, freeze and wrong-slide defect found, with tests (code head 713f40f after the ponytail-audit re-run, merged into `main` as 7bd84a1); 43 tickets filed as issues #2–#44; the full section, with the unattended-run needs and the cut-list evidence, is in this file's git history (f2bcea1).
- 2026-11-11 Q2: QA + security verification in simulate mode; 18 fix-now items + verifier finding Q2-44 fixed with tests; two fix lanes (10x, ponytail) → 10x base with ponytail grafts; 34 tickets; ADR-011 stage window compliant by default; cut-list evidence for ADR-012.
- 2026-11-11 Q1: May branch merged per ADR-010 + A1–A15; two lanes (10x, ponytail) → 10x base with ponytail grafts; all local gates green; build.yml YAML fixed.
- 2026-11-11 Q0 gate: the user accepted ADR-010 as amended; ADR-011; cuts deferred until after Q2; Q2 widened to cover wrong-slide defects.
- 2026-11-11 Q0: claims verified, ADR-010 path table + amendments A1–A15, ponytail lane; stopped at the user gate.
- 2026-11-11 re-planning: May branch discovered; ADR-010 Proposed; prompt pack regenerated for /10x-Team:10x-team.
- 2026-10-24 init → Discovery: `.10x/` created.
- 2026-10-24 Discovery → Strategy: codebase scanned; critical finding: model never loaded, VAD never initialized.
- 2026-10-24 Strategy → Design → Planning: specs + ADR-001…009 + prompt plan written in one pass after five-angle research.
