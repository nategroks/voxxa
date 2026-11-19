# QA — church-autoadvance

## 2026-11-19 — Q7 final (`q7/base` @ 6386f6b; both verifiers passed)
Details: `reviews/2026-11-19-qa-report.md` §11; the lanes and rulings: `reviews/2026-11-19-ponytail-q7.md`.
- **Both verifiers passed at 6386f6b.** Gates 1-7 and the CI eval step pass (210 passed, 1 ignored; the table unchanged from Q6). The socket mutants (TCP connect, and a UDP socket that never connects) fail the zero-sockets test; a 1 ms change to one logged action fails the determinism test at that fixture only; the NSIS script names neither tool. No test is hidden from CI: all 201 base tests still run with `--features cli`, plus 10 new.
- **Decision: the QA position is final.** Ready to merge once the PR's first CI run shows the evidence in QA report §9-§10 (Linux and Windows socket tests green, the Linux mutant red then green, full-eval green or warning on three OSes with times and the Node 20 check). Then the orchestrator runs full-eval once by hand on `main` (`workflow_dispatch`) and records its tables.
- **Release readiness of Increment 1′:** complete as a developer preview of the CLI; not a church release (p95 4.87 s against 2.5 s, #84; synthetic speech only, #73; founder checklist open).
- **Tickets:** 12 open (1 medium, 11 low), including Q7-08 to Q7-13 added at the close; Q7 closes #47 and #75.

## 2026-11-19 — Q7 verifier fixes (aa16e90)
- Gate 4 was flaky on Windows: `replay_opens_no_socket` counted Winsock-AFD events by pid alone. Fixed: only socket and connect events inside each child's start-to-exit window count (security review §10), with unit tests that fail if the window or the event filter is removed.
- Re-proved at aa16e90: 4 of 4 full `cargo test --features cli` runs pass (210 passed, 1 ignored); the CI eval step exits 0 with the table unchanged; the socket mutant fails; the determinism mutant fails; the NSIS installer names neither tool (control names both). QA report §10.

## 2026-10-24 — Windows verification of Increment 0 (local Windows session, handoff task 1)

Machine: Windows 11 Pro 10.0.26300, AMD Ryzen 3 1200 (4 cores, no SMT), fresh toolchain
installed via winget: Node 24.22.1, rustc/cargo 1.99.1 (stable-x86_64-pc-windows-msvc),
CMake 4.4.4, VS 2022 Build Tools (MSVC 14.44.35207, VCTools workload), LLVM (see below).
Branch: `claude/trusting-newton-v71ebu` @ d976698.

### Finding 1 — whisper-rs-sys 0.11.1 + LLVM ≥ 23 produces an empty `whisper_full_params`
- Symptom: `cargo clippy` / `cargo test` fail with ~60 × `E0609: no field … on type
  whisper_full_params` inside `whisper-rs-0.13.2/src/whisper_params.rs`; the generated
  `bindings.rs` has `pub struct whisper_full_params { pub _address: u8 }` with a layout test
  that expects 264 bytes.
- Not the header: clang 23.1.4 parses `wrapper.h` with zero diagnostics and its AST has
  every field of the struct.
- Isolated repro (scratch crate, same `wrapper.h` and clang args as whisper-rs-sys's
  build.rs, `allowlist_type("whisper_full_params")`):

  | bindgen | libclang | Result |
  |---|---|---|
  | 0.69.5 (pinned by whisper-rs-sys 0.11.x as `^0.69`) | 23.1.4 | empty struct (bug) |
  | 0.72.1 | 23.1.4 | all fields |
  | 0.69.5 | 20.1.8 | all fields |

- `WHISPER_DONT_GENERATE_BINDINGS=1` (bundled bindings) is **not** a workaround on Windows:
  those bindings were generated for a 64-bit LP64 target and fail their own layout
  asserts on MSVC (`attempt to compute 12_usize - 16_usize, which would overflow`, ×3).
- Gotcha: changing that env var does not rerun whisper-rs-sys's build script
  (it only watches `wrapper.h`); `cargo clean -p whisper-rs-sys` is required between tries.
- Unexplained: CI's `windows-latest` (image windows-2025-vs2026 20261021, "LLVM version:
  23.1.1") passes. Either 23.1.1 vs 23.1.4 differs in a way bindgen 0.69 depends on, or CI
  got its bindings another way. Not chased further.
- Local workaround used: LLVM 20.1.8 (`winget install LLVM.LLVM --version 20.1.8`).
- **Recommended fix (cloud owns `src-tauri/`):** move to whisper-rs ≥ 0.14.2
  (whisper-rs-sys ≥ 0.12.1 requires bindgen `^0.71`; whisper-rs 0.16.0 / -sys 0.15.0
  requires `^0.72`), ideally as part of P1.2 since the recognizer moves behind a trait
  there anyway. Until then, `docs/` or CONTRIBUTING should say "LLVM ≤ 22 on Windows".

### Finding 2 — `.github/workflows/build.yml` is invalid YAML (cloud owns workflows)
- Line 105: `releaseName: 'Voxxa ${{ github.ref_name || inputs.version || 'dev' }}'` nests
  single quotes inside a single-quoted scalar. GitHub cannot parse the file, so it reports
  a 0-second "workflow file issue" failure on every push (runs 40149187244, 40148590077)
  and the tag-triggered release build has never run.
- Same file: upload glob `bundle/**/*.nsis` matches nothing (NSIS output is
  `bundle/nsis/*-setup.exe`); `macos-13` runners are retired.

### Results (LLVM 20.1.8, clean whisper-rs-sys build)

| Step | Result | Time on Ryzen 3 1200 |
|---|---|---|
| `npm ci && npm run build` | ok | 0.1 min |
| `cargo fmt --check` | ok | <0.1 min |
| `cargo clippy --all-targets -- -D warnings` | ok | 1.1 min (whisper.cpp already built; cold ≈ 5.5 min) |
| `cargo test` | ok, 0 tests in 3 targets (as expected before P1.1) | 1.7 min |
| `npx tauri build` | ok | 4.8 min |

Artefacts: `bundle/nsis/Voxxa_0.1.0_x64-setup.exe` (3.3 MB) and
`bundle/msi/Voxxa_0.1.0_x64_en-US.msi` (4.6 MB), both unsigned.
Tauri warns that identifier `com.voxxa.app` ends in `.app` (conflicts with macOS bundle
extension); worth changing before the first public release, since the identifier also
names the settings folder and is costly to change later.

### Install test (non-admin, WebView2, tray, Models tab, SmartScreen)
(pending — needs a standard user account and the founder at the screen)

## 2026-11-11 — Q2 verification of the merged base (Phase 5)

Full report: `reviews/2026-11-11-qa-report.md`; tickets: `reviews/2026-11-11-q2-tickets.md`.

### Decisions
- **QA mode = simulate plus one real model download** (user decision). No microphone, no live keystrokes; the presenter always pointed at a local mock and `get_presenter_info` was checked before each dispatch; outbound traffic went through an allow-listing proxy. The founder runs the manual checklist (M1–M14) on the booth PC before any pilot.
- **Fix-now bar applied:** wrong content on the audience screen, crash or freeze, data loss or corruption, listening impossible on ordinary hardware, ADR-007 fixtures, ADR-011 violation. 18 triaged items plus Q2-44 (found by the verifier) are fixed with tests on `q2/10x` (1568810..b09373d); 34 items are tickets (14 medium, 20 low).
- **Every fix has a test that fails without it** (stash or patch-back, then re-mutated by the adversarial verifier). Wrong-slide fixes are tested at the dispatcher (`tests/dispatch.rs`, 10 tests with recording fake presenters that model the screen) or the conductor (`aligner` tests). The GUI-only fix (Q2-18) is covered by the committed CDP smoke `scripts/qa/cdp-smoke.mjs`.
- **ADR-011:** the stage window cannot default to an audience-facing output (created hidden, no position or monitor, opens on the primary display on an operator action). Not a violation. Copy and placement are ticket Q2-30.
- **Wrong-slide items kept as tickets** (adjudicated, not on the audience screen by default): Q2-22 pptx order (Low, synthetic trigger only), Q2-23 Pro7 WS cannot blank, Q2-24 Keystroke after restart, Q2-41 Next at a song's end, Q2-45 .pro7 blank slides, Q2-46 refused blank not retried, Q2-48 shorter Pro7 name.
- **Cut-list evidence gathered, no decision** (user decision): QA report §9 feeds ADR-012.

### How to rerun the GUI checks
1. Build a scratch copy with `additionalBrowserArgs: --remote-debugging-port=9222` on both windows (the WebView2 env var is ignored by wry) and `--features tauri/custom-protocol`. Never commit the switch (ticket Q2-53).
2. Start `python scripts/qa/mock_pro7.py`, launch the scratch exe, run `node scripts/qa/cdp-smoke.mjs`. Last result: 24/24.
3. Stop only the PIDs you started.

### Gates at the Q2 head (b09373d)
All seven pass on Windows: 45 unit + 33 integration tests; budgets GUI 2951/3000, everything else 2489/2500 (11 lines of headroom, the next tight spot).

## 2026-11-13 — Q5: `voxxa-eval` is the QA instrument for auto-advance (ADR-015)
- **What it measures:** each fixture's `voxxad` actions against the operator's `truth.jsonl`: precision (hits ÷ all gotos), recall, median/p95 latency, false advances per set (a wrong slide, a goto outside its row's span, a blank during singing), double advances (a goto to the slide already shown with no blank since, or a blank on a blank screen: the Q3-08 regression check), missed slides, blank and unblank latency (worst per row, `miss` when Voxxa never blanked or never came back). Definitions: `fixtures/README.md` "Metrics" and ADR-015 Decision 6.
- **Quality gate:** CI replays the committed session logs on Ubuntu, Windows and macOS and fails when any metric is worse than `fixtures/baseline.json` (exit 1, `FAIL <row>: <metric> is <v>, baseline needs <=|>= <limit>`). It needs no model and no audio, so it measures the conductor; a recogniser change, or a change to a `SmartConfig` default (Q5-07), needs the logs re-recorded locally (`voxxa-eval --full --record`).
- **Proved locally:** replay passes the committed baseline (exit 0); a scratch baseline with `total.p95_ms` 1000 exits 1 with `FAIL total: p95_ms is 8900, baseline needs <= 1000`. `tests/voxxa_eval.rs` holds the same proof, so `cargo test` also guards the gate.
- **Limits:** the two fixtures are stand-ins (synthetic speech, not singing; Q5-01). Use them to catch regressions, not to claim accuracy in a church. Latency is measured from action lines, not words (Q4-13). The 1 s early tolerance assumes a human operator's reaction time. Explicit `unblank` actions are never scored as false or double advances (known gap, pinned by a test). Neither fixture has an opening prayer before the first song; the 10x lane's fixture of that shape found Q5-04.
- **First numbers** (Whisper Tiny; replay gives the same table): precision 1.00, recall 1.00, latency median 2,900 ms and p95 8,900 ms (a late advance into verse 2, Q4-04), blank 200 ms after a pause but 6,600 ms into a prayer with no pause (Q4-03), unblank 1,500 ms. Q6 tunes against these and tightens the baseline.
- **Q5 final (c10e963, 2026-11-13):** the scorer and gate were mutation-tested in three verifier rounds. Rounds 1 and 2 left survivors that are now killed by tests, and closed two gate gaps (a dropped threshold switched its gate off; a replay that ran Whisper passed locally). In round 3 verifier A passed (full mode reproduced on a fresh clone, replay identical on LF and CRLF checkouts, more than 80 adversarial scoring checks). Verifier B failed: 63 of 70 mutants killed, six survive the tests. The two that matter are an action before the first row's window scored as a hit (Q5-08) and a total row that drops fixtures (Q5-09). **QA position: do not merge Q5 until Q5-08 and Q5-09 (and Q5-10's M52 and M55) have tests that fail under their mutants and a verifier confirms it.** `voxxa_eval` has 26 tests; 168 tests in all.

## 2026-11-13 — Q6 final: the conductor against the budget (ADR-016), `q6/base` @ 4edd7f6
- **Evidence added:** three synthetic text-to-speech fixtures of public-domain hymns (a repeated refrain, a jump, a prayer between songs), made offline with System.Speech straight to WAV files, plus Q5-04's opening-prayer reproduction. Six fixtures in all, all synthetic spoken audio, no singing (Q5-01). A fixture audit found them fit to decide on (truth rows re-derived three ways; generator byte-reproducible on the reference PC).
- **Decision measured twice, independently:** 1.67 false advances per 10 sung songs and p95 11.23 s at 733c7e1 (stream time at file speed, no inference time; live is worse, Q4-02). So the ADR-009 minimum was built.
- **Before → after (replay; full mode equal):** precision 0.93 → 1.00, recall 0.68 → 1.00, false advances 1 → 0, median 4.12 → 2.31 s, p95 11.23 → 4.87 s, unblank 8.49 → 2.49 s; blank and double unchanged. Table per fixture: `reviews/2026-11-13-q6-eval-before-after.md`.
- **Scripted tests** (`tests/conductor_evidence.rs` 19, `tests/conductor_unchanged.rs` 2): repeated chorus (labelled, unlabelled, two-slide chorus), bridge jump and verse jump, dropped and misheard words, an unrelated transcript, operator override (early step, step back, restart, dwell after a step), schema, and two traces of `MachineState`, blanking and `manual_step` taken from the pre-Q6 conductor.
- **Verifiers:** three rounds. Rounds 1 and 2 left conductor lines no test observed; each now has a test that fails under its mutant. Round 3 at 4edd7f6: both verifiers failed, on one item. Verifier B's mutation pass killed about 30 mutants by tests and 3 more only by the CI eval step; 2 low survivors remain (Q6-04, Q6-05).
- **Open, and it blocks the merge:** `jump-be-thou-my-vision` is worse on `median_ms` (900 → 1,200) and `p95_ms` (900 → 5,850) because its recall rose from 0.25 to 1.00, and its 1,000 ms cells cannot be met by any conductor that follows the jump. Gate 4 fails two `voxxa_eval` tests and the CI eval step exits 1 on those two cells. **QA position: do not merge Q6 until the user decides** (a one-time exception, 1,500 / 6,000, or keep `q6/base` out of `main`), and until the CI eval step and `cargo test` pass on that decision. Also for the user: the `total` row's blank and unblank cells ended Q6 500 ms looser than before it, because of the new prayer fixture.
- **Lesson for the gate:** a per-fixture latency threshold reads a recall gain as a regression when a fixture's old hits were few. Any later threshold rule should say how a row is judged when its recall rises.

## 2026-11-18 — Q6 close-out: QA position changed to "ready to merge" (`q6/base` @ 97cc21b)
- **The blocking item is closed by the user's decision** (2026-11-18): the jump row's cells are 1,500 / 6,000, and the `total` row's blank 7,500 and unblank 2,500 are accepted. The CI eval step and both `voxxa_eval` tests pass.
- **Final numbers** (replay; full mode with Whisper Tiny equal, on both 733c7e1 and the head): total 21/21 hits, precision and recall 1.00, 0 false and 0 double advances, median 2,310 ms, p95 4,870 ms, blank 7,280 ms, unblank 2,490 ms. No fixture worse than at 733c7e1 except the jump row's approved cells. Table: `reviews/2026-11-13-q6-eval-before-after.md` §7.
- **More scripted tests:** `tests/conductor_scripted.rs` (5, from the ponytail re-run: repeated chorus, bridge jump, lossy words, operator step, and a 27-step trace recorded at 733c7e1) and `tests/conductor_evidence.rs` now 25. The four scenarios fail on the 733c7e1 conductor, so they test the new behaviour; the traces pass there unchanged, so `MachineState`, blanking and `manual_step` are as before.
- **Verifiers:** after two more rounds with findings (c5ae273, d57f060), both verifiers passed at 97cc21b. Verifier A checked every goto against the audio segments, no goto during a prayer, and replay byte-identical across CRLF and LF checkouts. Verifier B ran 51 mutants; four non-blocking survivors are tickets Q6-13 and Q6-14.
- **QA position: ready to merge** once CI is green on all jobs. The latency limb of the budget is still missed (Q6-03), and all six fixtures are still synthetic speech (Q5-01).
- **Note for Q7:** `voxxa-eval --full` writes recognised words to stderr. Redirect it (`2>/dev/null`) so no recognised text reaches an agent's output.

## 2026-11-19 — Q7: close of Increment 1′ (`q7/10x`)
Report: `reviews/2026-11-19-qa-report.md`. Tickets: `reviews/2026-11-19-q7-tickets.md`.
- **Full fixture eval:**
  - On the Windows reference PC, Whisper Tiny at the head gives exactly the replay table on all six fixtures: 21/21 hits, 0 false, 0 double, median 2,310 ms, p95 4,870 ms, blank 7,280 ms, unblank 2,490 ms, exit 0.
  - On three OSes in CI, it runs as `.github/workflows/full-eval.yml`. That job is report-only (user decision), uses a SHA-256-pinned model, and has not run on GitHub yet; its first run is on the Q7 PR.
- **Replay determinism is a PR-gate test on three OSes:** `tests/replay_determinism.rs`.
  - Every committed log must replay to its own action lines byte for byte, and two replays must agree byte for byte.
  - Four logs held the 733c7e1 conductor's actions. They were re-recorded (20e3301). Their recogniser lines were identical (576/926/482/343 lines), and the table was unchanged, so `baseline.json` is unchanged.
  - Mutation: replay printing the wall time fails it on all seven logs.
- **Decision: the determinism contract is "a log's actions are today's conductor's".** A conductor change that moves actions must re-record the affected logs in the same PR, with the reason in the commit; the runbook says how. Since Q7, all six logs store their whole `SmartConfig`.
- **Gates at the head:** all pass, with 207 tests under `--features cli` (56 unit + 151 integration). Plain `cargo test` passes 147 and skips the four CLI test files (Q7-04).
- **QA position:** ready to merge once CI is green on all jobs, the new full-eval job included (it may warn, but must not fail). Increment 1′ is complete as a developer preview. It is not ready for churches: p95 is 4.87 s against a 2.5 s budget (#84), the fixtures are synthetic speech only (#73), and the founder checklist is open.
- **How to rerun:** `cargo test --features cli --test replay_determinism`; `voxxa-eval --full` with stderr filtered to `voxxa-eval:` lines (see the finish entry below); see `docs/runbooks/eval-regression.md`.

## 2026-11-19 — Q7 finish on `q7/base` (merge of `q7/10x`, grafts, review fixes)
Details: `reviews/2026-11-19-qa-report.md` section 9.
- **Decision: a plain `cargo test` fails the CLI tests, on purpose (closes Q7-04).** The `[[test]] required-features` blocks are gone. Each test file that runs `voxxad` or `voxxa-eval` checks `cfg!(feature = "cli")` and panics with "run: cargo test --features cli". Plain run: 197 passed, 10 failed with the hint; `--features cli` (CI): 207 passed.
- **Not grafted as written:** ponytail's `option_env!("CARGO_BIN_EXE_voxxad")` guard. Cargo 1.99.1 sets that variable even for a bin that `required-features` skips. So the guard never fires, and a stale binary can pass the tests (seen here: `replay_determinism` 2/2 on an old `voxxad.exe`).
- **One log list:** `tests/common/mod.rs` `committed_logs()` feeds both `replay_determinism` and `replay_no_sockets`.
- **The determinism rule is in `fixtures/README.md` now:** every committed log replays to its own action lines; an action-changing conductor PR re-records the affected logs with `voxxa-eval --full --record` and says why; all six logs store the whole `SmartConfig`, so a default change needs a re-record (#78).
- **Re-proved at the finish:** gates 1-7 and the CI eval step pass (table unchanged, exit 0); determinism passes, and a 1 ms change to one logged action fails it at that fixture only; the socket mutant fails the socket test (11 AFD events) and passes after the revert; the NSIS script names neither tool (control with `--features cli` names both).
- **QA position: ready to merge once the first CI run shows** the evidence listed in the QA report section 9: the Linux and Windows socket-test lines, the Linux mutant red then green, and full-eval green or warning on three OSes, with times and the Node 20 check. Increment 1′ stays a developer preview, not a church release.
- **How to rerun (replaces the line above):** `cargo test --features cli --test replay_determinism`; full mode with `2> >(grep '^voxxa-eval' >&2)` in bash, not `2>/dev/null`, which also hid voxxa-eval's own errors.

## 2026-11-19 — Q8: Simple mode and the first-run flow (`q8/10x`, ADR-017)
**Method (user decisions 2 and 3).**
- (a) The built GUI in headless Edge with a scripted backend: `npm run build && npm run test:gui:e2e -- <dir>`. Listen, the 20-second test and every audio path ran only here, with placeholder words.
- (b) The real app as a scratch build (`com.voxxa.qa10x`, CDP on port 9341), driven by `scripts/qa/simple-mode-real-app.mjs` with an allow-list of clicks: never Listen, the test, Next/Prev/Blank/Stage, key events, the keystroke presenter, the HTTP API or a model load. A ProPresenter mock (`scripts/qa/mock_pro7.py 1025`) stood in for the presentation app.
- (c) The pure light table in CI: `npm run test:gui`.

**Usability walk-through.** The research plan's usability test (`specs/2026-10-24-user-research-plan.md` § 2; objective O7: "go from download to green with the quick-start card and no call"; the § 4 reaction prompts), run as tasks. Screenshots: mock `01`-`21`, real `r01`-`r11`.

| # | Task | Mocked GUI | Real app (guarded) |
|---|---|---|---|
| U1 | First launch shows the set-up guide | pass (01) | pass (r01) |
| U2 | Choose the microphone | pass: list, choice saved, Settings shows it (20) | pass: 1 real device listed (names not recorded), Next with Default; no stream opened |
| U3 | Find the presentation app | pass both ways: none found says "arrow keys, keep your presentation app in front" (04); found, one click connects (05) | pass: auto-detect found the mock on 127.0.0.1:1025; one click connected; the presenter changed from keystroke to ProPresenter REST (r02, r03) |
| U4 | Get the voice model | pass: Tiny download with a progress bar, then "ready" (06) | partial: "Tiny (75MB) is on this computer." shown (r04); "Use it" not pressed (a failed load renames the user's model file, Q2-50) |
| U5 | Load this Sunday's songs | pass: the test step asks first and opens the existing file picker (07) | partial: the request is shown (r05); no songs loaded |
| U6 | 20-second test reaches green | pass: silence fails with "I couldn't hear anything…" (09); sound and words pass with "You're ready" (10); no heard text shown | not run (forbidden) |
| U7 | Start following, see song and slide | pass: Songs, then amber "Waiting for a song" (12), then green "Following <title>, slide 2 of 3" and "Song 2 of 3" (13) | not run (forbidden) |
| U8 | "What does amber mean?" | the sentence answers it: waiting for a song, talking not a song (14), or the screen is blank | grey sentence shown (r06) |
| U9 | "What would you do when it turns red?" | each red sentence names the action: check the cable, check the app is open (15, 16) | no red state reachable safely |
| U10 | Stop | pass: Off stops listening, grey (11) | not run (never started) |
| U11 | Find a setting | pass: Advanced, one click, every old panel (18-21) | pass (r08-r10); HTTP API left off |
| U12 | "No call" | Call for help says the bundle comes in the next update and to call the tech lead; no command runs (17) | same (r07) |
| U13 | Three controls only | pass: mode switch, Call for help, Advanced; Songs + Scripture disabled | pass (r06) |

**Verdict.** Every task completes on the mocked GUI. On the real app, every task that is safe without a microphone passes. Not proven: the device step on real audio, the 20-second test, Listen to green, Off (U6, U7, U10) and the model load (U4). They belong to the founder's run, which is not a merge gate (user decision 3): a founder checklist item and ticket Q8-01 below.

**Defect found:** the `hidden` attribute did not hide flex boxes, so the diagnostic modal and the welcome overlay covered the window at every start, and every presenter form showed at once. Fixed in Q8 (SDE record). The founder should still look through the old panels once.

**Tickets for the orchestrator (label `q8`):**
- Q8-01 (medium) The founder's usability run: the research plan's script on the founder's machine with a real microphone and presentation app (U2-U10 above); then five volunteers (O7).
- Q8-02 (low) Red "Lyrics don't match what I hear" is not mapped: `blank_hold` is amber because a prayer looks the same. Decide a time-based rule from pilot data.
- Q8-03 (low) Songs are loaded only in Advanced or the first-run test step; Simple mode has no song step for later Sundays. Read the presentation app's playlist (strategy: priority 1).
- Q8-04 (low) The browser test runs in CI on Windows only.
- Q8-05 (low) `main.js`'s own listening flag can drift when the HTTP API starts or stops listening. The Simple switch follows the backend, but its click goes through the old Listen button, which then needs two clicks (pre-existing).
- Q8-06 (low) The first-run model download checks no hash (same as Models, #9).

**Gates:** all pass. (1) `npm ci && npm run build`; (2) `cargo fmt --check`; (3) `cargo clippy --all-targets -- -D warnings`, with and without `--features cli`; (4) `cargo test --features cli` with `VOXXA_REQUIRE_SOCKET_TRACE=1`: 210 passed (56 unit + 154 integration), 1 ignored, 0 failed, unchanged from Q7; (5) `cargo-deny check licenses advisories bans`: ok; (6) `python scripts/loc_budget.py`: all OK; (7) the `dictat` grep prints nothing. The CI eval step: "OK: no metric is worse". `npm run test:gui`: 6/6. `npm run test:gui:e2e`: 77/77. Only binaries match the control-byte grep.

## 2026-11-19 — Q8 finish on `q8/base` (merge of `q8/10x`, review fixes, ponytail grafts; ADR-017 updated)
Commits after the merge (b4eacca), one each: 3ab4b6c the switch sends one toggle at a time (must-fix 1); 70e3519 row 5 carries the `presenter-error` reason (must-fix 2); 68df936 Space/Enter on Simple mode's and the guide's buttons no longer reach the Listen shortcut (both reviews); fa87345 the saved presenter is reconnected at start, never keystroke, never a form that needs a secret, and row 6 is red when the connected kind is not the one chosen (graft, should-fix 4); 91c3fc2 the status text is written only when it changes (graft); 9653f48 the 20-second test is green only on a `slide-advanced` inside the window, with "refused" and "not recognised" fails (graft); 42f7121 keystroke users are told to click into their app during the test (graft); 21b5b77 `still(0)` in the device step (nit 5). No graft was skipped.

**New checks.** `npm run test:gui`: 6/6 (`node:test` top-level tests; new cases inside them: row 5 with reason, `true` and blank payloads; row 6 for a kind other than the chosen one, and the matching kind; the `testVerdict` rows followed, refused and not recognised, a pass after one refusal, and order against "no words"). `npm run test:gui:e2e`: **91/91** (was 77): two same-tick clicks on Songs then Off end stopped with one `start_listening`; Off 150 ms after Songs ends stopped (the mock's `start_listening` now takes 300 ms); a real Space key (CDP `Input.dispatchKeyEvent`) on a focused `#fr-next` moves to step 2 and on `#s-help` shows the help message, neither calling `start_listening`; row 5 shows `dispatch.rs`'s focus message; a reload reconnects the saved ProPresenter form; an unchanged status gets no DOM write in 2.5 s; the keystroke sentence in the test step; the "Try again" test passes on a `slide-advanced` event. The switch, Space and text-write checks were run against the code before their fix and failed there (7, 6 and 1 failures).

**Usability walk-through, changed rows.** U3: a restart now keeps the presentation app chosen (real app, below). U6: green now needs a slide change the presenter took, with the sentence "Voxxa followed the song and changed the slide. You're ready." (mock 10). U9: the red row 5 names the presenter's reason (mock 15). Every other row as above.

**Real app, guarded (way b).** Scratch config outside git: identifier `com.voxxa.qafinish`, `additionalBrowserArgs` with `--remote-debugging-port=9343` on both windows; built with `npm run tauri build -- --no-bundle --config <scratch>`. Startup re-read before the launch: `run()` builds `AudioEngine::new()` (no stream; `audio.start()` is called only from `start_listening`, `commands/mod.rs:229`), the VAD, the keystroke controller (no I/O) and the tray; the page adds `get_status`, `get_presenter_info`, `list_audio_devices`, `select_audio_device` (stores a name) and, with a saved non-keystroke form, `connect_presenter`. Nothing opens an input stream. `scripts/qa/simple-mode-real-app.mjs` (now with `VOXXA_CDP_PORT`) with the ProPresenter mock on 127.0.0.1:1025: **27/27** (screenshots `r01`-`r11`). Then a real restart (stop by PID, start again): the backend started on keystroke and Simple mode reconnected the mock (`get_presenter_info` = `pro_presenter7_rest`, connected; one more `/v1/version` probe in the mock's log; not listening; screenshot `r12`). Never Listen, never the 20-second test, never the keystroke presenter chosen, no stage window, no HTTP API. App and mock stopped by PID; `%LOCALAPPDATA%\com.voxxa.qafinish` deleted.

**Gates** (Windows 11, rustc 1.99.1, `CARGO_TARGET_DIR=C:/dev/q3-10x-target`): all pass. (1) `npm ci && npm run build`; (2) `cargo fmt --check`; (3) `cargo clippy --all-targets -- -D warnings`, with and without `--features cli`; (4) `cargo test --features cli`: 210 passed, 1 ignored, 0 failed (run without `VOXXA_REQUIRE_SOCKET_TRACE`); (5) `cargo-deny check licenses advisories bans`: ok; (6) `python scripts/loc_budget.py`: all OK, GUI 2,780 / 3,000, Simple mode **341 / 600**; (7) the `dictat` grep prints nothing. The CI eval step: 21/21 hits, "OK: no metric is worse". Only binaries match the control-byte grep. `src-tauri/` is unchanged.

**Tickets changed.** Q8-05 stands (the old Listen button can still drift with the HTTP API; the switch's `pending` then waits 3 polls). New, low: Q8-07 a refused start shows `Songs` for up to 3 s before the switch falls back to `Off`; Q8-08 a church on ProPresenter's WebSocket API or on OpenLP with a login sees row 6 red after each start until it reconnects in Advanced (passwords are never saved); Q8-09 the keystroke driver's profile chosen in Settings is not restored at start (pre-existing; the reconnect never touches keystroke).

## 2026-11-19 — Q8 verifier findings closed on `q8/base`
Three findings from the round-1 verifiers, one commit each, each with a check that fails without it.
- ef84473 (verifier A) Row 9 after a webview reload: the page loses its song list, the backend keeps following, and the light read "Following <title>, slide <global index + 1> of 0." Row 9 now needs a position (`slides` > 0) and `songPosition`'s fallback carries no slide number, so the light says "Following the song." (ADR-017 row 9) and the song line keeps the title. `simple_logic.test.mjs` passes the fallback through `statusLight`; e2e step 8 reloads mid-song with the mock's setlist set (screenshot 22). Under the old line 22 the e2e check fails with "slide 0 of 0" and the unit test fails.
- 3f75c09 (verifier B) A panel leaked into the default view passed. `onlySimple()` in e2e (first view, after a reload, following a song, back from Advanced): no visible control outside `#simple`, nothing else on screen (toasts allowed). Mutants now caught by e2e: tab bar above `#app`, `#privacy-card` above `#app`, tab bar below `#simple`, `#section-settings` below `#simple` and unhidden.
- b191945 (verifier B) A hidden Skip passed because `element.click()` clicks hidden elements. e2e's `click()` and `clickButton()` throw on an element that fails `checkVisibility()`; Skip is checked visible on all four steps. Mutant `firstrun_skip_hidden` now fails ("Skip is visible on step 1", then "not visible, a user could not click it: #fr-skip").

**Checks.** `npm run test:gui`: 6/6 (new assertions inside the existing tests). `npm run test:gui:e2e`: **102/102** (was 91). Real app, guarded (`com.voxxa.qafinish`, CDP 9343, ProPresenter mock on 127.0.0.1:1025, scratch config outside git, `--no-bundle`; startup re-read: `audio.start()` only from `start_listening`, `commands/mod.rs:229`, and `src-tauri/`, `main.js`, `index.html` unchanged since 9e96282): `simple-mode-real-app.mjs` 27/27, plus a read-only probe of the same "only Simple mode" check: no leaks, first run closed (screenshot r13). Never Listen, never the 20-second test, never the keystroke presenter chosen, no stage window, no HTTP API; app and mock stopped by PID, `%LOCALAPPDATA%\com.voxxa.qafinish` deleted.

**Gates** (Windows 11, rustc 1.99.1, `CARGO_TARGET_DIR=C:/dev/q3-10x-target`): all pass. (1) `npm ci && npm run build`; (2) `cargo fmt --check`; (3) `cargo clippy --all-targets -- -D warnings`, with and without `--features cli`; (4) `cargo test --features cli`: 210 passed, 1 ignored, 0 failed (without `VOXXA_REQUIRE_SOCKET_TRACE`); (5) `cargo-deny check licenses advisories bans`: ok; (6) `python scripts/loc_budget.py`: all OK, GUI 2,780 / 3,000, Simple mode 341 / 600 (unchanged; the fix is two edited lines); (7) the `dictat` grep prints nothing. CI eval step: 21/21, "OK: no metric is worse". Only binaries match the control-byte grep.
