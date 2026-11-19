# QA — church-autoadvance

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
- **How to rerun:** `cargo test --features cli --test replay_determinism`; `voxxa-eval --full 2>/dev/null`; see `docs/runbooks/eval-regression.md`.
