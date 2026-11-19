# Current Handoff

## User decisions for Q7 (2026-11-18, before the unattended run)
- **Full Whisper eval:** a separate, report-only CI workflow (`workflow_dispatch`, plus `pull_request` with a paths filter on that workflow file only, so it can run before it reaches `main`). It downloads Whisper Tiny pinned by SHA-256, runs `voxxa-eval --full` and uploads the table; it never fails the PR. The PR gate stays replay plus the determinism test, on all three OSes.
- **Shipping the CLI:** `voxxad` and `voxxa-eval` do **not** ship in installers yet. If the Tauri bundle would include them, exclude them and prove it; CI and `cargo test` must still build and test both. Release notes for the CLI are a file in the repo; **no tag and no GitHub release**.
- **Pre-approved:** push `q7/base`, open a draft PR against `main`, fix CI forward; run the manual full-eval job; merge once Q7's exit criteria are met and every CI job is green; file Q7 tickets as GitHub issues (label `q7`); agents may stop processes they started, by PID.
- **QA mode** (as in Q2–Q6): no microphone, no keystroke dispatch; `--shadow`, `--replay` or `--presenter none` only; public-domain text only, and agents never echo hymn text; `ggml-tiny.bin` is present.
- The decisions for Q6 are in the "From: Q6" section below and in git history (a60d1ce).

---

## From: Q6 → To: Q7 (close Increment 1′)
**Date:** 2026-11-13; updated 2026-11-19 after the close-out
**Phase:** Phase 4 Implementation done and verified on `q6/base` (**final head 97cc21b**, plus the state commit "state: Q6 ponytail re-run, baseline decision, close-out"). The user decided the baseline item on 2026-11-18, and **both verifiers passed at 97cc21b**. Next: CI + PR + merge by the orchestrator (pre-approved), then Q7.

### What Q6 did
- **Fixtures first** (user decision 2, 733c7e1): three synthetic TTS fixtures of public-domain hymns, `refrain-it-is-well` (a repeated refrain), `jump-be-thou-my-vision` (verse 1, verse 3, verse 4, back to verse 1) and `prayer-between-songs`. Generator: `scripts/fixtures/make_tts_fixture.ps1` (offline System.Speech, WAV files only, byte-reproducible on the reference PC). 12 WAVs in `src-tauri/tests/fixtures/voxxad/tts-*.wav`. An independent audit found them fit to decide on.
- **Decision, measured twice independently at 733c7e1:** 1 false advance over 6 sung songs = 1.67 per 10 songs (limit 1); p95 11.23 s over 13 hits (limit 2.5 s). Both limbs fail, so ADR-009's minimum was built. Decision record: ADR-016 (`adrs/016-adr-009-minimum-implemented.md`, Accepted, five amendments).
- **Built:** a position tracker in `src-tauri/src/aligner/evidence/` (optional `sections[].kind` in setlist v1, repeat-aware priors in `SmartConfig.evidence`, word-token scoring on the last 12 words, an accumulator with decay and `min_dwell_ms`; the operator always wins). Q5-04 fixed: while blank, song detection reads only the newest transcript; its reproduction is the sixth fixture, `prayer-before-first-song`. `MachineState`, blanking, `manual_step`, the conductor's public API and event schema v1 are unchanged; no GUI, `voxxa-eval` or recogniser change; no sockets.
- **Dependencies (2026-11-18, for gate 5):** zip 9.0.0 → 9.0.5 (b7a5382) and chrono replaced by `time` (734b467), to clear RUSTSEC-2026-0548 and RUSTSEC-2026-0543, which appeared in the advisory database while `Cargo.lock` was unchanged. No advisory is ignored. `voxxad` now names session logs with `time` (local time, as before); the diagnostics report's `generated_at` ends in `Z`.
- **Process:** two lanes. The first ponytail build lane was stopped by an output content filter and committed nothing, so the first comparator took the 10x lane (merge d92800c) with two review fixes; three verifier rounds on 2026-11-13. On 2026-11-18 the ponytail lane was re-run (`q6/ponytail` @ 7d4b4b6), reviewed and compared: the comparator kept `q6/base` and grafted tests, docs, the baseline cells and a decay clamp. Two more verifier rounds found gaps, each closed with a test or the dependency fix; both verifiers passed at 97cc21b. Record: `reviews/2026-11-13-ponytail-q6.md` (§10 for the re-run).
- **Records:** `reviews/2026-11-13-q6-eval-before-after.md` (summary, the fixtures' limits, and §7 the user's decision and the final table), `reviews/2026-11-13-q6-before-after.md` (every hit, every round), `reviews/2026-11-13-q6-tickets.md` (12 open: 2 medium, 10 low), `decisions/{architect,sde,dba,qa,security}/church-autoadvance.md` ("Q6 final" and the 2026-11-18 close-out).

### Final numbers at 97cc21b (replay; full mode with Whisper Tiny gives the same table)
Before is 733c7e1. The numbers have not changed since 4edd7f6; every later commit was a test, docs, the baseline, the decay clamp (no effect at the defaults) or a dependency.

| fixture | before median / p95 ms | after median / p95 ms | recall before → after | false before → after |
|---|---|---|---|---|
| jump-be-thou-my-vision | 900 / 900 | **1,200 / 5,850** (baseline 1,500 / 6,000, user exception) | 0.25 → 1.00 | 0 → 0 |
| prayer-before-first-song | 6,710 / 6,710 (log replayed with the old conductor) | 1,710 / 2,800 | 0.50 → 1.00 | 0 → 0 |
| prayer-between-songs | 8,490 / 11,230 | 2,490 / 4,870 | 1.00 → 1.00 | 0 → 0 |
| prayer-then-reprise | 2,900 / 8,900 | 2,900 / 2,900 | 1.00 → 1.00 | 0 → 0 |
| prayer-without-pause | 2,900 / 8,900 | 1,900 / 2,900 | 1.00 → 1.00 | 0 → 0 |
| refrain-it-is-well | 4,120 / 10,310 | 2,120 / 3,900 | 0.50 → 1.00 | 1 → 0 |
| total | 4,120 / 11,230 | 2,310 / 4,870 | 0.68 → 1.00 | 1 → 0 |

- Rule after: **0 false advances per 10 songs (met); p95 4.87 s (not met, budget 2.5 s)**. Precision 1.00, double advances 0, unblank 2,490 ms. Blank timing is unchanged (6.6-7.3 s into a prayer with no pause, Q4-03, #61).
- All latency is **stream time at file speed**: no inference time, not wall clock. Live it would be higher (Q4-02, #60). No `--realtime` run was made.
- **Gates at 97cc21b, all pass:** 1 `npm ci && npm run build`, 2 fmt, 3 clippy `-D warnings`, 4 `cargo test` (201 tests in 21 binaries, 0 failed), 5 cargo-deny (advisories, bans, licences ok), 6 loc budget, 7 the `dictat` grep, and the CI eval step (exit 0, "OK: no metric is worse"). Only binaries match the control-byte grep.
- **The ponytail re-run, for comparison** (five old fixtures, replay): 19/19 hits, 0 false, median 2,310 ms, p95 8,490 ms, unblank 8,490 ms. It was faster on one cell (jump p95 3,850 ms, the log's floor) and slower wherever a song starts after a prayer. Ticket Q6-15 keeps its idea.

### Decided by the user (2026-11-18)
- **The jump row:** `median_ms` 1,500 and `p95_ms` 6,000 in `fixtures/baseline.json`, a one-time exception to "tightened, never loosened" (applied in 85c736c, with the log's floor of 1,200 / 3,850 ms in the message). The old 1,000 / 1,000 cells came from one hit at recall 0.25.
- **The `total` row:** `blank_ms` 7,500 and `unblank_ms` 2,500 accepted (7,000 and 2,000 before Q6). No old fixture got worse; the new prayer fixture needs those values.
- With these cells the exit criterion "eval not worse on any fixture" holds, and gate 4 and the CI eval step pass.
- **Still for the user:** approve or decline Q6-01, a ~6-line GUI fix (decision 5 allows no GUI change without approval); and Q6-10 (whether to drop the two unused `SmartConfig` fields).

### Next for the orchestrator (pre-approved, "User decisions for Q6") — done 2026-11-19
PR #82 merged as a60d1ce (CI green on all 5 jobs at 1254374); Q6 tickets filed as #83–#94; #76 and #62 closed; Q6 numbers added to #61, #73, #77 and #78. Original instruction: push `q6/base`, open a draft PR against `main`, get CI green on all jobs (fix forward if needed), and merge only when every CI job is green. Then file the 12 open Q6 tickets (label `q6`), close #76 (Q5-04) and #62 (Q4-04), and add the Q6 numbers to #61, #73, #77 and #78. The first CI run on the merged state also runs gate 5 against the advisory database of that day.

### What Q7 must know
**Q7's task** (prompt pack): QA runs the full fixture eval and the replay determinism test on all three OSes via CI; Security adds a test that runs `voxxad --replay` with networking disabled and asserts zero sockets; DevOps writes CLI release notes and a `docs/INTEGRATIONS.md` section for driving `voxxad` from Companion; SRE writes the runbook "eval shows regression". Q7 starts from `main` after the Q6 merge (the user chose option (a), so Q6 is meant to merge).

**Running `voxxa-eval`** (unchanged from Q5; see `fixtures/README.md`):
```sh
cargo build --manifest-path src-tauri/Cargo.toml --bin voxxad --bin voxxa-eval
cargo run --manifest-path src-tauri/Cargo.toml --bin voxxa-eval                     # replay: what CI runs
cargo run --manifest-path src-tauri/Cargo.toml --bin voxxa-eval -- --full 2>/dev/null  # Whisper Tiny, local
cargo run --manifest-path src-tauri/Cargo.toml --bin voxxa-eval -- --full --record fixtures/recordings/<name>  # re-record one log
```
- **Six fixtures** in `fixtures/recordings/`: `jump-be-thou-my-vision`, `prayer-before-first-song`, `prayer-between-songs`, `prayer-then-reprise`, `prayer-without-pause`, `refrain-it-is-well`. All synthetic spoken audio, no singing (Q5-01, #73).
- **Time in debug** on the reference PC: replay of all six about 3-4.5 min; full mode about 11-12 min (much longer when other jobs load the PC: verifier A took 75 min on 2026-11-13 and 5,518 s on 2026-11-18); the `voxxa_eval` test binary about 290 s.
- **Keep full mode's stderr out of your output.** Whisper prints recognised words to stderr. The command above sends it to `/dev/null`; a verifier that tailed it once saw recognised words in its own tool output (nothing reached a file or message). The content rule still applies: never print setlist or recognised text; print counts and indices only. CI runs replay in the eval step and three more times in `cargo test`, on each OS (#77). Q7's "full fixture eval on all three OSes via CI" can only be replay: CI has no model. Full mode stays local.
- **Regenerating the TTS audio:** `powershell -ExecutionPolicy Bypass -File scripts/fixtures/make_tts_fixture.ps1 [-Fixture <name>]`. It writes WAVs and the truth files; then re-record the fixture's log. Reproducibility is checked on the reference PC only.

**Replay determinism: what holds, and one trap for Q7** (re-checked by verifier A at 97cc21b).
- Two replays of every fixture are byte-identical, also between a CRLF and an LF checkout.
- Full mode with Whisper Tiny on the reference PC reproduces every committed log's recogniser lines byte for byte (575/426/925/481/342/728 input lines over the six fixtures), and its action lines equal replay's on all six.
- **Re-recorded in Q6** (the logs whose actions come from the Q6 conductor): `fixtures/recordings/refrain-it-is-well/session.jsonl` (its setlist gained `sections`, so the fingerprint changed; 2a57b9f), `fixtures/recordings/prayer-before-first-song/session.jsonl` (new fixture), and the test log `src-tauri/tests/fixtures/voxxad/session-pd-hymns.jsonl`. For these, replay's action lines equal the recorded ones byte for byte. In every re-recording the recogniser lines were identical to the earlier recording; only the start line and the actions changed.
- **The other four fixture logs still carry the actions of the 733c7e1 conductor** (the 733c7e1 conductor reproduces them exactly; replay re-runs today's conductor on their recorded recogniser lines). A determinism test that compares replay's actions with the actions recorded in those four logs will fail. Either compare replay with replay, or re-record those four logs with `--full --record` first (their recogniser lines will match; say why in the commit).
- The ponytail re-run also re-recorded `session-pd-hymns.jsonl` on its own branch; that copy is not on `q6/base`.

**Re-recording rules.**
- A setlist that gains `sections` changes its replay fingerprint, so its log must be re-recorded.
- Logs from before Q6 get the `evidence` defaults filled in on replay. Logs recorded in Q6 store their `evidence` block, so a later change to an evidence default is invisible to replay until those logs are re-recorded (Q5-07, #78). So a retuned `EvidenceConfig` default moves CI for four fixtures and not for the other two; `fixtures/README.md` says this since 55e46a8.

**Baseline rule, and a lesson.** Thresholds follow ADR-015 (latency: the next multiple of 500 ms strictly above the measured worst; counts and ratios exact). The user's rule is "tightened, never loosened" (decision 4). Q6 showed a gap: a per-fixture latency threshold reads a recall gain as a regression when a fixture's old hits were few. Q7's runbook "eval shows regression" should tell the reader to check recall and the missed count before treating a latency FAIL as a regression, and to ask the user before any threshold is loosened.

**Conductor facts for the security test and the runbook.**
- `voxxad --replay` builds the conductor from each log's start-line `SmartConfig` and checks the setlist's fingerprint; it loads no model and opens no audio device.
- Each tracker move logs one `[EVIDENCE] song s: slide a -> b: e_now -> e_best` line (`aligner/evidence/mod.rs`); there is no `[CONDUCTOR]` move line any more.
- `voxxad` session-log names and the expiry rule now use the `time` crate (734b467): local time, read in `main()` before any thread starts, falling back to UTC. No new network code; Q6 adds no socket.
- The scripted conductor tests that guard behaviour: `tests/conductor_evidence.rs` (25), `tests/conductor_scripted.rs` (5, from the ponytail re-run, including a 27-step trace of `MachineState`, blanking and `manual_step` recorded at 733c7e1) and `tests/conductor_unchanged.rs` (2 traces). All use placeholder words.

**Budget numbers (after Q6, 97cc21b).**
| Row | Used / cap | Left |
|---|---|---|
| everything else in src-tauri/src | 2,309 / 2,500 | 191 |
| conductor (`aligner/`, without `evidence/`) | 1,173 / 1,200 | 27 |
| conductor evidence (`aligner/evidence/`; new in Q6, user decision) | 215 / 500 | 285 |
| recognizer | 432 / 700 | 268 |
| voxxad binary | 552 / 600 | 48 |
| voxxa-eval | 469 / 600 | 131 |
| presenters | 1,149 / 2,000 | 851 |
| importers + planning_center | 1,386 / 1,600 | 214 |
| GUI | 2,951 / 3,000 | **49** |

Tests: 201 (56 unit + 145 integration). New tests go in `src-tauri/tests/`, which is not counted.

**Open Q6 tickets** (`reviews/2026-11-13-q6-tickets.md`, 12 open, not filed yet; Q6-02, Q6-04 and Q6-05 are closed): Q6-01 the GUI stores the whole `SmartConfig` (medium, needs the user's OK); Q6-03 p95 still 4.87 s (medium); Q6-06 no importer writes `sections`; Q6-07 to Q6-09 fixture notes, one line's public-domain source, an unused generator path; Q6-10 two unused `SmartConfig` fields; Q6-11 ADR-009's graph-wide tracking; Q6-12 every cargo build rewrites `src-tauri/gen/schemas/*.json` line endings; Q6-13 and Q6-14 the tracker's operator bookkeeping is not fully pinned (verifier B's non-blocking survivors); Q6-15 try the ponytail re-run's scoring context for slides being entered.

**Other things to keep.**
- Safety rules for unattended runs are unchanged: no microphone, no keystroke dispatch, `--shadow`/`--replay`/`--presenter none` only, TTS to files only, and stop every process you start, by PID.
- The shared `CARGO_TARGET_DIR` (`C:/dev/q3-10x-target`): building an older commit there overwrites the head's debug binaries. Build exports under a separate target or rebuild the head before measuring.
- After any cargo build, `src-tauri/gen/schemas/desktop-schema.json` and `windows-schema.json` show as modified (line endings only). Do not commit them (Q6-12).
- Write files with the Edit or Write tools, not Bash heredocs (Q3-12, #56). Check `git grep -lIP '[\x00\x0b]'` before every commit, and `git diff --stat` before committing state files.

### Still open from earlier steps
- **Founder checklist** before any pilot leaves shadow mode: QA report §10 (M1–M14) and security review §10, plus Q3-07 (#51): start and stop listening in the GUI (a `transcription` event every 1-3 s of speech), unplug, and one `voxxad --device default --shadow` run on the booth PC. First clear `%LOCALAPPDATA%\com.voxxa.app`; keep `%APPDATA%\voxxa\models\ggml-tiny.bin`.
- **Founder:** record the two public-domain hymn fixtures (Q5-01, #73); they would also re-check ADR-016's numbers on singing.
- **User decision:** the cut-list ADR-012 (evidence in the QA report §9).
- **Q1 A15 follow-ups:** `fixtures/setlists/pd-hymns.json` and `fixtures/pco/plan-sample.json` do not fit the May parsers (the eval's fixtures use the `voxxad --setlist` format instead); re-run the Windows verification (a003992) on the merged head (`handoff-local-windows.md`); finish reconciling `docs/signing-policy.md` with `SIGNING.md`.

**Process rule (user, 2026-11-11):** every prompt (Q2…Q27) also runs independently through ponytail. Run it as two lanes plus cross-review for code steps, or `ponytail` + `ponytail-audit` for verification and close steps (Q7 is a close step). Record the result in `reviews/YYYY-MM-DD-ponytail-<Q>.md`.

---

## Handoff History
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
