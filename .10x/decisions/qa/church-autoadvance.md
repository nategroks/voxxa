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

## 2026-11-19 — Q8 verifier round 2 closed on `q8/base`
Three findings, one commit each, each with a check that fails without it.
- bb46628 (fix) The 20-second test decided "started" from one `get_status` 1 s after the Listen click, so a start slower than that read "Listening didn't start" and was never stopped (mic open after the guide closed; "Try again" then ran with `wasRunning` true and never stopped either). `run()` (`src/simple/firstrun.js:173-218`) now waits until the click settles (the Listen button turns `active`, or `main.js` shows the refusal as an error toast; up to 15 s), stops whatever it started, waits for a start still in flight (up to 15 s more) so it is stopped too, and shows "Try again" only once the Listen button is off. e2e: a start refused after 1.6 s says "didn't start" with nothing listening; a 1.6 s start runs the full test and leaves nothing listening 2.5 s after the verdict. Under the old `run()` the second case fails ("Listening didn't start", `__mock.st.running` true) and the rest of the run cascades.
- 9ef8937 (test) Mutant css-amber-red-swap survived because `light()` checked only the class. It now also requires the computed background of `#s-light`: grey `rgb(107, 114, 128)`, amber `rgb(245, 158, 11)`, green and red the resolved `--success` and `--danger`, four distinct. The mutant now fails rows 10, 7, 8, 5 and 6.
- 808f14b (test) Mutants advanced-offscreen (`position: fixed; top: -100px`) and advanced-transparent (`opacity: 0`) survived the plain `checkVisibility()`. `seen()` adds `opacityProperty`/`visibilityProperty` and requires the box inside the window; `click()`, `clickButton()` and the three-controls check use it. `click()` first scrolls a panel the user can scroll (`overflow` auto or scroll), so `#diag-btn` below the fold of Settings still counts; a fixed or `body`-clipped element does not move. Both mutants now fail "default view shows exactly three controls" and the click on `#s-advanced`.

**Checks.** `npm run test:gui`: 6/6. `npm run test:gui:e2e`: **106/106** (was 102; screenshots 01-22 re-taken). Real app, guarded (scratch config outside git, `com.voxxa.qafinish`, CDP 9343, `npm run tauri build -- --no-bundle --config <scratch>`, ProPresenter mock on 127.0.0.1:1025; startup re-read: `src-tauri/`, `main.js`, `index.html`, `stage.html` unchanged since 9e96282, `audio.start()` only from `start_listening`, `commands/mod.rs:229`): `simple-mode-real-app.mjs` **27/27** (r01-r11), plus a read-only probe of the new checks: three controls on screen by `seen()`, light grey `rgb(107, 114, 128)`, not listening (r13). Never Listen, never the 20-second test, never the keystroke presenter chosen, no stage window, no HTTP API. App and mock stopped by PID; `%LOCALAPPDATA%\com.voxxa.qafinish` deleted. The tauri build rewrote `src-tauri/Cargo.toml` and two `gen/schemas` files with LF endings only; restored with `git checkout`.

**Gates** (Windows 11, rustc 1.99.1, `CARGO_TARGET_DIR=C:/dev/q3-10x-target`): all pass. (1) `npm ci && npm run build`; (2) `cargo fmt --check`; (3) `cargo clippy --all-targets -- -D warnings`, with and without `--features cli`; (4) `cargo test --features cli`: 210 passed, 1 ignored, 0 failed (without `VOXXA_REQUIRE_SOCKET_TRACE`); (5) `cargo-deny check licenses advisories bans`: ok; (6) `python scripts/loc_budget.py`: all OK, GUI 2,780 / 3,000, Simple mode **352 / 600**; (7) the `dictat` grep prints nothing. CI eval step: 21/21, "OK: no metric is worse". Only binaries match the control-byte grep.

## 2026-11-19 — Q8 final (`q8/base` @ ee431ce): verifier round 3, not ready to merge
**QA position: not ready to merge.** In verifier round 3 at ee431ce, verifier B passed and **verifier A failed**. The set-up guide stops main.js's shortcut keys only when the focus is inside the guide. When the focus is on the page body (when the guide opens on a fresh profile, and during the 20-second test after its Start button is removed), Space starts or stops listening and `s` opens the stage window behind the guide (`src/simple/firstrun.js:31`; `src/main.js:1563`). This is ticket **Q8-10** (medium). **The PR must not be merged until Q8-10 is fixed with e2e checks that press keys with the focus on `document.body`, each check fails on ee431ce, and a verifier confirms the fix.** Everything else in the exit criteria is met, apart from the founder's run, which is not a merge gate (user decision 3). Full record: `reviews/2026-11-19-ponytail-q8.md` §§ 6-8; tickets: `reviews/2026-11-19-q8-tickets.md`.

**What verifier A checked at ee431ce (round 3).**
- Gates 1-7, with `cargo test --features cli` run with `VOXXA_REQUIRE_SOCKET_TRACE=1`: 210 passed, 1 ignored. The CI eval step: 21/21. `test:gui` 6/6. e2e 106/106.
- Its own mocked probe: 53/53, plus 7 key-press notes. Every ADR-017 row was checked with its class, computed colour and exact sentence. Every Advanced panel works against the mock.
- The guarded real app (`com.voxxa.qaverifya`, CDP 9344): `simple-mode-real-app.mjs` 27/27 and a read-only probe 30/30. The ProPresenter mock's log holds only `/v1/version`. The app never listened, and Windows' microphone-usage registry has no Voxxa entry afterwards. App and mock were stopped by PID, and the scratch `%LOCALAPPDATA%` folder was deleted.

**What verifier B checked (passed).**
- `src-tauri/` has no diff against f8500e3. No panel was deleted: every old element id is still there except the five welcome-overlay ids.
- The budget has one new row (Simple mode 352 / 600), and the GUI row is 2,780 / 3,000. There is no runtime dependency, `dist/` holds no mock, and no scratch config is committed.
- 26 of 28 mutants were killed. The two survivors are minor: Q8-11 (M6b) and Q8-12 (M3c).

**Usability walk-through, final** (research plan § 2 usability test and objective O7, with the § 4 reaction prompts; this replaces the tables above):

| Step | Task | Mocked GUI | Real app (guarded) |
|---|---|---|---|
| U1 | First launch shows the set-up guide | pass | pass |
| U2 | Choose the microphone | pass (choice saved, re-applied at start, shown in Settings) | pass (enumeration only; names not recorded) |
| U3 | Find the presentation app | pass (found: one click connects; none found: "arrow keys" sentence) | pass (the mock found and connected; reconnected after a real restart, r12) |
| U4 | Get the voice model | pass (Tiny download with progress) | partial: state shown, "Use it" not pressed (Q2-50) |
| U5 | Load this Sunday's songs | pass (the test step asks and opens the file picker) | partial: request shown, no songs loaded |
| U6 | 20-second test reaches green | pass by clicks (green only on a slide change the presenter took; slow and refused starts handled); **fails with the keyboard** (Q8-10) | not run (forbidden) |
| U7 | Start following, see song and slide | pass (amber "Listening…", then green "Following <title>, slide n of m") | not run (forbidden) |
| U8 | "What does amber mean?" | the sentence answers it | grey sentence shown |
| U9 | "What would you do when it turns red?" | each red sentence names an action; row 5 gives the presenter's reason | no red state reachable safely |
| U10 | Stop | pass (Off stops listening, grey) | not run (never started) |
| U11 | Find a setting | pass (Advanced, one click) | pass (HTTP API left off) |
| U12 | "No call" | placeholder message, no command | placeholder message |
| U13 | Three controls only | pass (by `seen()`: on screen, opaque, visible) | pass |

**Verdict:** the script passes on the mocked GUI by mouse. U6 fails by keyboard until Q8-10 is fixed. On the real app, every step that is safe without a microphone passes. The founder's run (Q8-01: U2-U10 with a real microphone and presentation app, plus a look through the old panels) is a founder checklist item, not a merge gate.

**Tickets:** 16 open (2 medium, 14 low), `reviews/2026-11-19-q8-tickets.md`. The six listed in the Q8 section above are Q8-01 to Q8-06 there, and the three from the finish are Q8-07 to Q8-09. New: Q8-10 (the merge blocker), Q8-11 and Q8-12 from verifier B, and Q8-13 to Q8-16 from the comparator and the lane reports. Q8 closes no earlier issue outright; #7 is fixed in part (fa87345).

**Q8-10 closed (2026-11-19, orchestrator).** Q8-10 was fixed by the orchestrator in 88aa63b: a capture-phase `keydown` listener on `window` blocks every key while the guide is shown. Four new e2e checks press Space and `s` with the focus on the page body, on step 1 and during the 20-second test. All four fail on the old code (41 passed, 6 failed) and pass with the fix. An independent verifier confirmed it at 88aa63b: 13 shortcut keys were blocked with the focus on the body, the buttons and the select, and the guide's own keyboard controls still work. The old-code mutant and the bubble-phase mutant both fail. The verifier's one survivor, keys blocked even after the guide closes, is killed by fce3e58 ("s reaches main.js again" after Skip; 117 e2e checks). QA position: ready to merge once CI is green. U6 (the 20-second test) now also passes by keyboard.

## 2026-11-19 — Q9 (10x lane, `q9/10x`): Call for help, runbook links, the exit demo
**Method:** as in Q8, with both ways of GUI testing.

**(a) Mocked browser.** `tests/gui/e2e.mjs` (158 checks; Q8: 117) and `tests/gui/mock-tauri.js`. The mock gained the four Q9 commands, with configurable preview and save delays and a refused save. It writes and opens nothing; it records the calls.

**(b) The real app, guarded.**
- **Build:** a scratch build with identifier `com.voxxa.qa9x` and CDP on port 9351, from a config outside git, built with `npm run tauri build -- --no-bundle --config`.
- **Run:** started with `VOXXA_HELP_DIR=<scratch>`, and a placeholder `voxxad` session log in `<scratch>/sessions`. Driven by `scripts/qa/call-for-help-real-app.mjs`.
- **Startup check, before the first launch:** `audio.start()` is called only from `start_listening` (`src-tauri/src/commands/mod.rs:229`). `select_audio_device` only stores a name, and the preview's report only lists devices.
- **Never done in the real app:** Listen, the 20-second test, the keystroke presenter, the stage window, the HTTP API, a model button, Save (which would call `open_help_email`) or a real runbook link.
- **Cleanup:** the app was stopped by PID (8304). `%LOCALAPPDATA%\com.voxxa.qa9x` and the scratch bundles were deleted afterwards. No `voxxa-help-*.zip` appeared on the real Desktop. `src-tauri/Cargo.toml` and `gen/schemas` were restored after each build (#91).

**Rust tests (`src-tauri/tests/help_bundle.rs`, 8 tests, on all three OSes in CI):**
- **#102:** fails on the old `Heard: {text}` line.
- **Redaction:** found a real gap, `passphrase`, which was fixed before the commit.
- **The words window:** newest log by name, `-10` after `-2`, the 120 s cut, `window` lines left out, no log, and a log with no words.
- **The preview writes nothing.**
- **The zip:** with and without the words file (absent, not empty); never overwrites; a failed write leaves no file.
- **The mailto:** no recipient; only encoded characters; names the file and its path; the folder is the second target.
- **Links resolve:** the backend's three names and `logic.js`'s `RUNBOOKS` are files in `docs/runbooks/`, and every relative link in the four new docs exists.

**Logic test (`test:gui`, 7 tests):**
- the runbook column in the row table;
- every red row links a runbook;
- each runbook file exists.

**E2E Call for help (mocked):**
- Space on the button opens the review and starts nothing.
- Save waits for the files.
- `#simple` is inert, and 8 Tab presses never reach it.
- Every file is listed with its size.
- The words are shown, with the box on by default and the agreed label.
- The other files are one click away.
- `localStorage` reaches the preview.
- Nothing is written before Save.
- With the dialog open, Space and `s` on the page body do nothing.
- Cancel and Escape write nothing.
- Unchecked, the zip has no words file. A slow save pressed twice is sent once.
- After the write, `open_help_email` is called once, and the path and the next steps are shown.
- Checked, the zip has all three files.
- A refused save shows its reason, Save works again, and no e-mail is opened.
- With no log, there are two files, no box, and a sentence saying so.

**E2E runbook links:** rows 1, 3, 5 and 6 show "What to do" and call `open_runbook` with the right name; rows 4, 7 and 9 show no link.

**Mutation checks:**

| Mutant | Result |
|---|---|
| `inert` removed | 2 e2e checks fail (inert; Tab reached `#simple` after 1 press) |
| The old `Heard:` line | `no_recognised_text_reaches_the_diagnostics_buffer` fails |

**Usability script, only the Call-for-help step (user decision 4):**

| # | Task | Mocked GUI | Real app (guarded) |
|---|---|---|---|
| U12 | "No call": Call for help | pass: review → Save → path shown and the e-mail draft and folder opened once (screenshots 18-21) | pass up to the review screen (02, 03); the bundle was written through the real backend into the scratch folder; Save and the e-mail step were not pressed (rule) |
| U9 (part) | "What would you do when it turns red?" | each red sentence now has "What to do", which opens the matching runbook (15, 16) | no red state reachable safely |

**Exit demo, the real backend in the guarded app, placeholder words:**
- The review screen listed `diagnostic-report.json (0.7 KB)`, `operator-settings.json (0.4 KB)` and `words-last-2-minutes.txt (0.2 KB)`.
- Box checked, `voxxa-help-20261119-212707.zip` (1,143 bytes) holds:
  - report, 713 bytes;
  - settings, 370 bytes;
  - words, 168 bytes. Only the two `hypothesis` lines within 120 s of the log's end (130.0 s and 170.0 s) are in it. The earlier line and the `window` line are not.
- Box unchecked, `voxxa-help-20261119-212709.zip` (889 bytes) has the report and the settings only.
- In both zips:
  - the presenter password, `voxxa.pcoSecret` and the HTTP API `token` are `[redacted]`;
  - `recentSetlists` is left out;
  - the report's `recent_log` has no `Heard:` line.
- A second write without a new review was refused. `open_runbook("../../evil")` was refused.

**Not tested:** the real `open` of a mail program and Explorer (never triggered, by rule). Only the URL and the folder it would open are tested, in Rust. CI has not run Q9 yet.

## 2026-11-19 — Q9 finish on `q9/base`: exit demo re-taken
**(a) Mocked browser.**
- `npm run test:gui:e2e` passed 160/160, with screenshots 01-26.
- New since the 10x lane: when the e-mail step fails, the message says the folder is open. When the error starts with `folder: `, it says the folder did not open.
- `test:gui` passed 7/7.

**(b) The real app, guarded.**
- **Build:** a scratch build with identifier `com.voxxa.qa9finish` and CDP on port 9353, from a config outside git, built with `npm run tauri build -- --no-bundle --config`.
- **Startup check, before the first launch:** the input stream opens only in `audio.start()`. Its only caller is `start_listening` (`src-tauri/src/commands/mod.rs:229`), which runs on Listen (`src/main.js:700`), the tray toggle or the HTTP API. None of these was used.
- **Run:** started with `VOXXA_HELP_DIR=<scratch>` and a placeholder session log in `<scratch>/sessions`. Driven by `scripts/qa/call-for-help-real-app.mjs`: 10/10 passed.
- **The review screen listed:** `diagnostic-report.json (0.7 KB)`, `operator-settings.json (0.4 KB)` and `words-last-2-minutes.txt (0.1 KB)`, with the box on (screenshot `02-real-review`).
- **Box checked:** `voxxa-help-20261119-221649.zip` (1,125 bytes, deflate) holds three files: the report (713 bytes), the settings (370 bytes) and the words (142 bytes). The words file has only the lines at 130.0 s and 170.0 s. The earlier line and the `window` line are not in it.
- **Box unchecked:** `voxxa-help-20261119-221650.zip` (886 bytes) holds the report and the settings only.
- **In both zips:**
  - the 3 placeholder secrets are `[redacted]`;
  - the setlists are left out;
  - there is no `Heard:` line.
- **Refusals:** a second write without a new review, and `open_runbook("../../evil")`.
- **Never done:** Save, the e-mail step, the folder step, Listen, the 20-second test, a presenter, the stage window and the HTTP API.
- **Cleanup:**
  - The app was stopped by PID (11816).
  - `%LOCALAPPDATA%\com.voxxa.qa9finish` and both scratch zips were deleted.
  - The real Desktop had no `voxxa-help-*.zip` before or after.
  - `src-tauri/Cargo.toml` (line endings only) was restored after the build.

**Not tested:** the real `open` of a mail program and Explorer, by rule. CI has not run the finished `q9/base` yet.

## 2026-11-19 — Q9 final (`q9/base` @ 89f4ccb): verifier round 3, both passed
**QA position: ready to merge once CI is green.** Both exit criteria are met. **Bundle produced:** the real backend, in the guarded real app, wrote the zip into a scratch folder with the box checked (three files) and unchecked (two files, the words file absent), with every placeholder secret redacted and no heard text in the report. **Links resolve:** `every_runbook_link_resolves` (Rust) and "each runbook link resolves to a file in docs/runbooks" (Node) check the three runbooks, the four docs' relative links and, through `runbook_url`, the exact GitHub URL. Full record: `reviews/2026-11-19-ponytail-q9.md` §§ 6-7; tickets: `reviews/2026-11-19-q9-tickets.md`.

**Verifier rounds.**
- **Round 1 (8ade143): did not pass.** Five findings: redaction missed names with `key` in the middle and Planning Center's application and client IDs (3e3efd7); survivors M2b and M5b, because the Rust tests called only the pure helpers (5ba1c00, new `tests/help_commands.rs` running the real preview and Save steps); survivor M5c (708fa62); the quick-start printed on two pages (84ca89c, now 41 lines and kept to one page by a test).
- **Round 2 (c0fae68): did not pass.** The quick-start sent the volunteer to a releases page with no release (82ff710, with a test).
- **Round 3 (89f4ccb): both passed.**
  - **Verifier A** (exit criteria, privacy, real app): every gate, `cargo test --features cli` with `VOXXA_REQUIRE_SOCKET_TRACE=1` 221 passed; e2e 160/160; its own mocked harness 21/21, with a layer that really writes the zip; a scratch Rust crate 30/30 over 7 runs (18 placeholder secrets, the 129,999 / 130,000 ms cut, the newer `-2` log, both zips, a second Save refused, `runbook_url`). Every label and message in the quick-start and the runbooks matched the code. Guarded real app `com.voxxa.qa9va` (CDP 9354, PID 13408): 14/14, both zips in the scratch folder, Save never pressed.
  - **Verifier B** (scope, network, privacy, mutants): every gate; no manifest, lock file, capability, config, budget script or workflow changed; no network code; `dist/` holds no mock. 30 mutants, 29 killed. The survivor, a Debug-level heard-text line (M1c), reaches the buffer only under `RUST_LOG=debug` (Q9-11).

**Where each check ran.**
- **Mocked browser:** every Call for help path, including Save, the e-mail and folder step, both failure messages, both box states, no log, Cancel, Escape, the key block, `inert`, and every runbook link click. Listen, the 20-second test and the audio paths ran only here.
- **Rust tests with temp folders:** #102, redaction, the words window, the zip with and without the words file, no overwrite, a failed write, the preview and Save commands for real (`VOXXA_HELP_DIR`), the `mailto:` URL, `runbook_url`, links resolve, and the quick-start's length and install step.
- **Guarded real app** (four scratch identifiers across the run: `qa9x`, `qa9p`, `qa9finish`, `qa9va`): Simple mode, Call for help up to the review screen, and the bundle written through the real `write_help_bundle` into a scratch folder. Never Listen, the 20-second test, a presenter, the stage window, the HTTP API, a model button, Save, a runbook link or the e-mail and folder step. Each app was stopped by PID and its `%LOCALAPPDATA%` folder deleted. No `voxxa-help-*.zip` appeared on the real Desktop.

**Usability script, Call-for-help step only (user decision 4):** U12 passes in the mock end to end, and in the real app up to the review screen and the backend write. The "What to do" part of U9 passes in the mock. The founder's run should add both (Q9-07, Q9-12).

**Not tested:** the real launch of a mail program and Explorer (by rule; Q9-12); CI has not run Q9 yet; the browser test still runs only on Windows (Q8-04).

**Tickets:** 16 open (1 medium, 15 low). The test and tooling ones: Q9-07, Q9-11, Q9-12, Q9-15.

## 2026-11-20 — Q9W (10x lane, `q9w/10x`): exit demo
ADR-019 and the Q9W exit list. Placeholder titles only ("Song A", "Bravo Hymn"); no recognised words anywhere.

**Unit tests (Rust, fake presenters and pure helpers; no test creates enigo or presses a key):**
- Discovery by process with a fake process list: `src-tauri/tests/worshiptools.rs:108` (hit with kind, profile, source, empty host and port 0, and the JSON the GUI reads; misses on an empty list and on look-alikes). Name matching: `presenters/keystroke.rs` test `presenter_is_known_by_its_process_name_or_path`.
- The foreground guard: `keystroke.rs` test `worshiptools_keys_go_only_to_presenter_in_front` (`keys_allowed`; unreadable front window refused; other profiles unaffected; `FRONT_MSG` word for word).
- No step across songs: `worshiptools.rs:148` (inside a song: Rights; song change: no key, the ask, first slide recorded; a mid-song landing; a chip back to an earlier song); `:179` raw Next/Prev unchanged; `:193` other profiles still step across.
- `can_blank` false with no red error: `worshiptools.rs:203` (conductor blank skipped, operator blank "not supported", no key) and `:224` (through `gui_sink`: a prayer emits no `presenter-error`; the song change emits only the ask).
- The exact key map: `keystroke.rs` tests `the_worshiptools_key_map_is_right_left_and_no_blank` (never F10/F11/F12/Up/Down/numpad +/-/".") and `other_profiles_keep_their_keys`; `can_blank_and_the_song_end_stop_come_from_the_profile`.

**GUI, mocked backend:** `tests/gui/simple_logic.test.mjs:143,160` (both amber sentences word for word, look-alikes stay red, the can't-clear sentences); `tests/gui/e2e.mjs:529-579`, section 9 (step 2 names WorshipTools Presenter and connects with its profile, the pick next to another app, amber guard and song-change ask on the light, info toasts, the can't-clear row, the restart reconnect, and no reconnect for a universal form). Results: `test:gui` 9/9, e2e 176/176.

**GUI, the real app (guarded scratch build):** identifier `com.voxxa.qa9wx` (inlined in the binary; `com.voxxa.app` absent), CDP 9361, built with `npm run tauri build -- --no-bundle --config <scratch>`, never committed. Startup opens no audio input (checked in code first). With WorshipTools Presenter running on LHMEZPC01, step 2 ("Find your presentation app") showed "Found WorshipTools Presenter. It has no remote control, so Voxxa presses its keys for you. Keep WorshipTools Presenter in front during the service." and saved `{"kind":"keystroke","keystroke_profile":"worship_tools_presenter"}`; `get_presenter_info` then reported `can_blank: false`. After the app was stopped by PID and started again, `can_blank` was false at once (the profile was reconnected). Only "Next" on step 1, "Skip setup", "Advanced" and "Set-up guide" were clicked; Listen, the test, Next/Previous/Blank, the HTTP API and the stage window were never touched; Presenter's process was never touched. Both PIDs stopped; `%LOCALAPPDATA%\com.voxxa.qa9wx` deleted. (The first CDP pass read step 1 before step 2 rendered and skipped the guide; the second pass reopened the guide and is the one recorded.)

**Not tested here:** a real key reaching Presenter, the guard against a real foreground switch, Presenter as administrator, and the F10 unblank; these are founder checklist M15 and M16 (in `handoff.md`). CI has not run Q9W yet.

## 2026-11-20 — Q9W base (`q9w/base`): exit demo re-taken after the merge, fixes and grafts
**Gates** (Windows, `CARGO_TARGET_DIR=C:/dev/q3-10x-target`). npm ci and the build pass. fmt passes. clippy `-D warnings` passes with and without `--features cli`. `cargo test --features cli`: 235 passed, 0 failed, 1 ignored (2 tests are new: `tests/dispatch.rs` 13, `tests/worshiptools.rs` 8). cargo-deny: advisories, bans and licences pass. The budget passes: everything else 2316 / 2317, discovery 210 / 250, GUI 2784 / 3000, Simple mode 450 / 600. The `dictat` grep is empty. The CI eval step scores 21/21: "OK: no metric is worse". `test:gui`: 9/9. e2e: 185/185 (9 more checks than the lane).

**GUI, mocked backend** (e2e section 9). After the guard's sentence and after the song-change ask, there is no `.toast-error`, and a delivered slide turns the light green. The ask arrives as dispatch now sends it: `slide-advanced` for the new song's first slide, then the ask. The light shows the ask, and the song line shows the new song. In Advanced, the song chip's ask is a bare info toast, and the strip's active chip moves to the new song. A refused Next shows the guard's sentence as info, and no toast says "failed". Screenshots: `31-advanced-worshiptools-asks-as-info` and 27-30.

**GUI, the real app (guarded scratch build).** Identifier `com.voxxa.qa9wf`, inlined in the binary as two immediates; neither `com.voxxa.app` nor `xa.app` is in it. CDP on 9363. Built with `npm run tauri build -- --no-bundle --config <scratch>` and never committed. Startup opens no audio input: `lib.rs:73` builds `AudioEngine::new()` only, `select_audio_device` stores a name, and capture opens only in `start_listening_with_state` (`commands/mod.rs:229`). WorshipTools Presenter was running (8 `Presenter` processes, untouched).
- Run 1 (fresh profile): step 1 Next led to step 2. Step 2 said "Found WorshipTools Presenter. It has no remote control, so Voxxa presses its keys for you. Keep WorshipTools Presenter in front during the service." It saved `{"kind":"keystroke","keystroke_profile":"worship_tools_presenter"}`, and `get_presenter_info` changed from `can_blank: true` to `false`. Skip setup was pressed.
- Run 2 (restart): the first run stayed closed, and `can_blank` was false from the start (the profile was reconnected).
- Only Next on step 1, Skip setup and read-only invokes were used. Listen, the 20-second test, Next/Previous/Blank, the HTTP API and the stage window were never touched. Both PIDs (9912, 14300) were stopped. `%LOCALAPPDATA%\com.voxxa.qa9wf` was deleted. The installed Voxxa that was already running (not started here) was left alone.

**Not tested here:** the same as the lane (M15, M16). Also untested: what Left does on Presenter's first slide (added to M16). CI has not run `q9w/base` yet.

## 2026-11-20 — Q9W base (`q9w/base`): exit demo re-taken after the verifier fixes (4fb1346)
**Gates** (Windows, `CARGO_TARGET_DIR=C:/dev/q3-10x-target`). npm ci and the build pass. fmt passes. clippy `-D warnings` passes with and without `--features cli`. `cargo test --features cli`: 236 passed, 0 failed, 1 ignored (`tests/worshiptools.rs` 9; the keystroke unit tests are 6, and the call-site test runs on Windows only). cargo-deny: advisories, bans and licences pass. The budget passes: everything else 2315 / 2317, discovery 217 / 250, presenters 1374 / 2000, keystroke.rs 396 / 400, GUI 2784 / 3000, Simple mode 450 / 600. The `dictat` grep is empty. The CI eval step: "OK: no metric is worse". `test:gui`: 9/9. e2e: 185/185. No GUI file changed.

**Mutants re-run** (each must now fail a test): the guard call deleted, `cfg!(windows)` flipped, the guard result ignored, `ends_with("presenter.exe")`, the process hit dropped from the merge, and `tell_gui` reduced to `ok` or comparing the whole `Sent`. All six fail. Every keystroke test driver records keys, so none could press one.

**GUI, the real app (guarded scratch build).** Identifier `com.voxxa.qa9wf`. Neither `com.voxxa.app` nor `xa.app` is in the binary. CDP on 9363. Built with `npm run tauri build -- --no-bundle --config <scratch>`, never committed. Startup still opens no audio input: `lib.rs:73` builds `AudioEngine::new()` only, and capture opens only in `start_listening_with_state` (`commands/mod.rs:229`). WorshipTools Presenter was running (8 `Presenter` processes, untouched).
- Run 1 (fresh profile): step 2 said "Found WorshipTools Presenter. It has no remote control, so Voxxa presses its keys for you. Keep WorshipTools Presenter in front during the service." It saved `{"kind":"keystroke","keystroke_profile":"worship_tools_presenter"}`, and `can_blank` changed from true to false. The log shows `profile=WorshipToolsPresenter`. Skip setup was pressed.
- Run 2 (restart): the first run stayed closed, and `can_blank` was false from the start (the profile was reconnected).
- Only Next on step 1, Skip setup and read-only invokes were used. Listen, the 20-second test, Next/Previous/Blank, the HTTP API and the stage window were never touched. Both PIDs (7232, 12656) were stopped. `%LOCALAPPDATA%\com.voxxa.qa9wf` was deleted. The installed Voxxa that was already running (PID 10228, not started here) was left alone.

**Not tested here:** the same as before (M15, M16, and Left on Presenter's first slide). CI has not run `q9w/base` yet.

## 2026-11-20 — Q9W base (`q9w/base`): exit demo re-taken after verifier round 2 (7a8a457)
**Gates** (Windows, `CARGO_TARGET_DIR=C:/dev/q3-10x-target`). npm ci and the build pass. fmt passes. clippy `-D warnings` passes with and without `--features cli`. `cargo test --features cli`: 237 passed, 0 failed, 1 ignored (`tests/worshiptools.rs` now has 10 tests). cargo-deny: advisories, bans and licences pass. The budget passes: discovery 223 / 250; the other rows are unchanged (everything else 2315 / 2317, presenters 1374 / 2000, keystroke.rs 396 / 400, GUI 2784 / 3000, Simple mode 450 / 600). The `dictat` grep is empty. The CI eval step: 21/21, "OK: no metric is worse". `test:gui`: 9/9. e2e: 188/188 (3 new checks). The control-byte grep matches nothing.

**Mutants re-run:** `gui-advanced-profile-not-set` fails 2 e2e checks (186 passed, 2 failed). `disc-not-wired` fails `discovery_runs_the_process_strategy`.

**GUI, the real app (guarded scratch build).** Identifier `com.voxxa.qa9wf`; `com.voxxa.app` and `xa.app` are not in the binary. CDP on 9363. Built with `npm run tauri build -- --no-bundle --config <scratch>`, never committed. The build left line-ending-only changes in `src-tauri/Cargo.toml` and `src-tauri/gen/schemas/*.json`, which were restored with `git checkout`. Startup opens no audio input: `lib.rs:73` builds `AudioEngine::new()` only (`audio/mod.rs:23`), and capture opens only in `start_listening_with_state` (`commands/mod.rs:229`). Neither file changed since 006372c. WorshipTools Presenter was running (8 `Presenter` processes, untouched, still 8 afterwards).
- Run 1 (fresh profile): step 2 said "Found WorshipTools Presenter. It has no remote control, so Voxxa presses its keys for you. Keep WorshipTools Presenter in front during the service." It saved `{"kind":"keystroke","keystroke_profile":"worship_tools_presenter"}`, and `can_blank` changed from true to false (log: `profile=WorshipToolsPresenter`). Skip setup was pressed.
- Run 2 (restart): the first run stayed closed, and `can_blank` was false from the start (the log shows the profile reconnected). Then Advanced → Settings. Connect with keystroke and universal gave `can_blank` true. The selects were set to ProPresenter and universal, and Auto-detect was run. It listed one row, "WorshipTools Presenter" with meta "process" (no host:port). Clicking the row set the selects to keystroke and `worship_tools_presenter`. Connect then gave `can_blank` false, and the log shows `profile=WorshipToolsPresenter`. This is the live proof that `discover_all` passes the real process list.
- Used: Next on step 1, Skip setup, the Advanced and Settings tabs, the two selects, Auto-detect, the discovered row, Connect (it only sets the driver's profile; listening was off and nothing was dispatched), and read-only invokes. Never used: Listen, the 20-second test, Next/Previous/Blank, the HTTP API or the stage window. Both PIDs (14924, 13732) were stopped. `%LOCALAPPDATA%\com.voxxa.qa9wf` was deleted. The installed Voxxa (PID 10228, not started here) was left alone.

**Not tested here:** M15, M16, and Left on Presenter's first slide. CI has not run `q9w/base` yet.

## 2026-11-20 — Q9W final (`q9w/base` @ b0de71b): verifier round 3, both passed
**QA position: ready to merge once CI is green.** Every exit criterion is met:
- **Unit tests** (Rust, fake presenters, fake process lists, a recording sender; no test can reach enigo): discovery by process (`tests/worshiptools.rs:121,164,210`), the foreground guard (`presenters/keystroke.rs:397,422`), no step across songs (`tests/worshiptools.rs:244,281,330,344`), `can_blank` false with no red error at a blank (`:354,375`; ProPresenter WS `tests/dispatch.rs:376`), and the exact key map (`keystroke.rs:441,470,488`).
- **GUI tests:** the amber sentences word for word (`tests/gui/simple_logic.test.mjs`, e2e section 9) and the restart reconnect (e2e reload: WorshipTools reconnected, universal not).
- **All gates pass** at b0de71b (finisher, verifiers A and B): 237 passed, 1 ignored; `test:gui` 9/9; e2e 188/188; the CI eval step 21/21, "OK: no metric is worse".
- **M15 and M16** are in the founder checklist (`handoff.md`, "Still open from earlier steps").

Full record: `reviews/2026-11-20-ponytail-q9w.md` §§ 6-8; tickets: `reviews/2026-11-20-q9w-tickets.md`.

**Verifier rounds.**
- **Round 1 (1927856): did not pass.** INTEGRATIONS overstated Auto-detect (e47b2c0); the guard's call site, a ProPresenter look-alike, the process hit in the merge and the GUI tell had surviving mutants (8022bfd, 63b03d6, 97279a8, 4fb1346).
- **Round 2 (006372c): did not pass.** Two survivors: Advanced → Auto-detect not setting the profile (d538a4b, e2e) and the process strategy not wired into discovery (7a8a457, Rust).
- **Round 3 (b0de71b): both passed.**
  - **Verifier A** (exit criteria, real app): every gate; 8 tests of its own on a `git archive` copy (never committed) with the real driver, a fake front window and a recording sender, all pass; read-only, the real `process_paths()` found Presenter's 8 processes; guarded real app `com.voxxa.qa9wa` (CDP 9364, PIDs 284 and 5092): step 2 named Presenter, the restart kept the profile. Notes: INTEGRATIONS' "connects" wording (Q9W-12) and the M16 number (Q9V needs a new number for its eval).
  - **Verifier B** (scope, safety, mutants): every gate; 35 Rust and 18 GUI mutants, all killed; scope clean (no manifest, lock, config or capability change); guarded real app `com.voxxa.qa9wb` (CDP 9365, PID 6120): step 2 named Presenter.

**Where each check ran.**
- **Mocked browser** (headless Edge, `tests/gui/e2e.mjs` with `mock-tauri.js`; Node for `simple_logic`): step 2 and the pick, both amber sentences and their info toasts, no red toast after an ask, the light turning green on a delivered slide, the can't-clear row, the restart reconnect, Advanced → Auto-detect and Connect, the song strip following the ask. Listen, the 20-second test and every dispatch ran only here.
- **Rust tests:** everything in the unit-test list above, plus `tell_gui`, the merge, the whole discovery with a fake process list, and the 11 other profiles' maps.
- **Guarded real app** (identifiers `qa9wx`, `qa9wp`, `qa9wf`, `qa9wa`, `qa9wb`; WorshipTools Presenter running, untouched): **the real first-run step 2 named the running WorshipTools Presenter in every run**; the real discovery returned the process hit; `can_blank` false after step 2 and after a real restart; Advanced → Auto-detect listed the real row and Connect set the profile. Never Listen, the 20-second test, Next/Previous/Blank, the HTTP API or the stage window, and no key was sent. Each app was stopped by PID and its `%LOCALAPPDATA%` folder deleted.

**Not tested, by rule** (M15, M16): a real key reaching Presenter, a real foreground switch, Presenter as administrator, the F10 unblank, Left on Presenter's first slide. CI has not run Q9W yet.

**Tickets:** 12 open (2 medium, 10 low). The test ones: Q9W-01 (the halfway multi-key step) and Q9W-11 (`discover_all` with the real process list).
