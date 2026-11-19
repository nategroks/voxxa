# Current Handoff

## User decisions for Q7 (2026-11-18, before the unattended run)
- **Full Whisper eval:** a separate, report-only CI workflow (`workflow_dispatch`, plus `pull_request` with a paths filter on that workflow file only, so it can run before it reaches `main`). It downloads Whisper Tiny pinned by SHA-256, runs `voxxa-eval --full` and uploads the table; it never fails the PR. The PR gate stays replay plus the determinism test, on all three OSes.
- **Shipping the CLI:** `voxxad` and `voxxa-eval` do **not** ship in installers yet. If the Tauri bundle would include them, exclude them and prove it; CI and `cargo test` must still build and test both. Release notes for the CLI are a file in the repo; **no tag and no GitHub release**.
- **Pre-approved:** push `q7/base`, open a draft PR against `main`, fix CI forward; run the manual full-eval job; merge once Q7's exit criteria are met and every CI job is green; file Q7 tickets as GitHub issues (label `q7`); agents may stop processes they started, by PID.
- **QA mode** (as in Q2–Q6): no microphone, no keystroke dispatch; `--shadow`, `--replay` or `--presenter none` only; public-domain text only, and agents never echo hymn text; `ggml-tiny.bin` is present.
- The decisions for Q6 are in git history (a60d1ce).

---

## From: Q7 → To: Q8 (Simple mode)
**Date:** 2026-11-19
**Phase:** Q7 (Phases 5 and 6, the close of Increment 1′) is done and verified on `q7/base`: final head **6386f6b**, plus the state commit "state: Q7 close Increment 1prime". **Both verifiers passed at 6386f6b.** Next: the orchestrator's CI, PR, manual full-eval run and merge (pre-approved above), then Increment 2′ with Q8.

### What Q7 did
- **Process:** two lanes from 46cb6d1. `q7/10x` (8 commits) was merged into `q7/base` as the comparator's base (008aa50); `q7/ponytail` (5 commits, 4d06345) stays local for reference. Four ponytail grafts and the 10x review's fixes followed, one commit each (c9271f0..911d56c). The first verifier round (on 86b549d) found the Windows socket check flaky; aa16e90 fixed it with unit tests. Record: `reviews/2026-11-19-ponytail-q7.md`.
- **QA:** `tests/replay_determinism.rs` runs on 3 OSes in `ci.yml`. Every committed log must replay to its own action lines, and two replays must agree. Four fixture logs were re-recorded first (20e3301); their recogniser lines and the table did not change. `.github/workflows/full-eval.yml` is the report-only Whisper Tiny eval, with the model pinned by SHA-256. On the reference PC, full mode equals replay cell for cell.
- **Security:** `tests/replay_no_sockets.rs` traces `voxxad --replay` on every committed log: strace inside `unshare -rn` on Linux, ETW on Windows (only socket and connect events inside each child's own lifetime count), a positive control first, and `VOXXA_REQUIRE_SOCKET_TRACE=1` plus, on Linux, `VOXXA_REQUIRE_NETNS=1` in CI. macOS is skipped and says so (Q7-01). The claim is "`--replay` opens no socket; `--shadow` makes no network connection" (on Linux and macOS tokio's signal driver creates one local AF_UNIX socketpair). `ci.yml` has `permissions: contents: read`.
- **DevOps:** `voxxad` and `voxxa-eval` are out of every installer (b9692c5). They live in `src-tauri/src/cli/` behind the off-by-default `cli` feature, and `tests/packaging.rs` guards this. Release notes: `docs/releases/voxxad-cli-preview.md` (no tag, no release). Companion: `docs/INTEGRATIONS.md`, "Driving `voxxad` from Bitfocus Companion".
- **SRE:** the runbook `docs/runbooks/eval-regression.md`.
- **Records:** `reviews/2026-11-19-qa-report.md` and `reviews/2026-11-19-security-review.md` (§11 is the final round), `reviews/2026-11-19-q7-tickets.md` (12 open: 1 medium, 11 low), `decisions/{qa,security,devops,sre}/church-autoadvance.md` ("Q7 final").

### Numbers at the finish (unchanged from Q6)
Replay, exit 0: 21 of 21 hits, precision and recall 1.00, 0 false and 0 double advances, median 2,310 ms, p95 4,870 ms (budget 2,500 ms, #84), blank 7,280 ms, unblank 2,490 ms. `baseline.json` is unchanged. Tests: 210 passed with `--features cli` (56 unit + 154 integration), plus 1 ignored helper.

### Next for the orchestrator (pre-approved), before Q8 starts
1. Push `q7/base` and open a draft PR against `main`.
2. **The first CI run must show** (QA report and security review, §9-§11):
   - (a) the `ubuntu-22.04` and `windows-latest` jobs pass `replay_no_sockets` (a pass proves strace inside `unshare -rn` and ETW respectively; no `--nocapture` run is needed);
   - (b) a throwaway commit adding `let _ = std::net::TcpStream::connect("127.0.0.1:9");` to `voxxad`'s replay branch turns the Linux job red, and its revert turns it green again;
   - (c) `full-eval.yml` is green, or only warns, on 3 OSes; the summary and the artifact hold only the table; record the time per OS. It runs on every push to the PR (paths filter). Check it for Node 20 deprecation warnings on checkout v4.2.2, rust-cache v2.7.8, cache v4.2.3, setup-node v4.4.0 and upload-artifact v4.6.2, and bump any pin that warns (SHA-pinned).
   - Record the run URLs in both reports.
3. Merge when every job is green. Then run full-eval once by hand on `main` (`workflow_dispatch`) and record its three tables and times in the QA report.
4. File the 12 open Q7 tickets (label `q7`). Close #47 and #75. Add the notes to #48, #49, #54, #78, #9, #77, #69, #89 and #70 (tickets file, last section).

### Q8's task (prompt pack, `specs/2026-11-11-prompts-10x-team.md`, Stage C)
**Q8 — Simple mode in front of the existing panels (M), Phase 4,** per ADR-010 Decision 4 and `specs/2026-10-24-software-strategy.md` § "Simple mode". The default view gets: the three-position mode switch (Off / Songs / Songs + Scripture, the last disabled until Increment 4′), the status light with one plain sentence (grey/amber/green/red, mapped from `MachineState`, the listening state and the presenter connection), the current song and slide, and a "Call for help" button. Every existing panel moves behind one "Advanced" link; nothing is deleted. The welcome overlay becomes the first-run flow: device → auto-detect presenter → model download → 20-second test. Keep the GUI under ADR-010's budget and dependency-free. Do not change backend commands. Exit: a volunteer sees three controls; everything else is one click away; the usability script from `specs/2026-10-24-user-research-plan.md` passes on the founder's machine.

### What Q8 must know
- **The GUI budget is the constraint.** GUI 2,951/3,000: **49 lines left**. Simple mode will not fit by adding code. Free lines first (move or merge markup) or ask the user for a budget decision before writing code; never raise a cap silently (R15). Q6-01 (#83) needs about 6 GUI lines and the user's approval.
- **Q8 is a GUI step, so its QA mode differs from Q2–Q7.** Starting the GUI was forbidden in Q7. Q2 drove the GUI in simulate mode over WebView2 remote debugging, with a mocked presenter and no microphone (`reviews/2026-11-11-qa-report.md`; ticket Q2-53, #34). The 20-second test and the device step need a real microphone, so they belong to the founder's run of the usability script, not to an unattended agent. Ask the user for Q8's decisions before the run, as for Q3–Q7.
- **"Do not change backend commands."** The status sentence must be built from what the GUI already receives (`MachineState`, the listening events, presenter status). The mapping of GUI events is pure and tested (`tests/gui_events.rs`, ADR-014).
- **Privacy before a volunteer sees "Call for help":** the diagnostics report carries every transcript the GUI heard (Q7-08, medium). Q9 builds the help bundle on that report. Fix Q7-08 before or with Q9, and before any pilot leaves shadow mode.
- **Model download in the first-run flow:** the app still checks no hash (#9). `full-eval.yml` pins SHA-256 `be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21` for the same URL; the first-run flow should check the same value if it touches the download.
- **The tools:** build and test `voxxad` and `voxxa-eval` with `--features cli`. A plain `cargo test` fails 10 tests on purpose ("run: cargo test --features cli"). Never move a tool back to `src-tauri/src/bin/` (tauri-cli bundles everything there; `tests/packaging.rs` fails).
- **The eval still guards the conductor.** A Q8 change should not move any action; if one does, re-record the affected logs with `voxxa-eval --full --record` and say why in the commit. So must any `SmartConfig` or `EvidenceConfig` default change, because all six logs store the whole config (`fixtures/README.md`, #78).
- **CI:** keep the Linux runner on `ubuntu-22.04` until `unshare -rn` is shown to work on a newer image (comment in `ci.yml`). On Windows, `replay_no_sockets` counts only socket-create and connect events (AFD 1000, 1017, 1018, 1020, 1021) inside each child's start-to-exit window; keep it that way, because pid alone was flaky.

**Running `voxxa-eval`** (see `fixtures/README.md` and `docs/runbooks/eval-regression.md`):
```sh
cargo build --manifest-path src-tauri/Cargo.toml --features cli --bin voxxad --bin voxxa-eval
cargo run --manifest-path src-tauri/Cargo.toml --features cli --bin voxxa-eval                     # replay: what CI runs
cargo run --manifest-path src-tauri/Cargo.toml --features cli --bin voxxa-eval -- --full 2> >(grep '^voxxa-eval' >&2)  # Whisper Tiny, local (bash)
cargo run --manifest-path src-tauri/Cargo.toml --features cli --bin voxxa-eval -- --full --record fixtures/recordings/<name> 2> >(grep '^voxxa-eval' >&2)  # re-record one log
```
- Six fixtures in `fixtures/recordings/`, all synthetic speech (#73). Never print setlist or recognised text: Whisper prints recognised words to stderr, so keep only `voxxa-eval:` lines; print counts and indices only.
- Time in debug on the reference PC: replay of all six about 2-4 min; full mode about 12 min idle, 38 min or more when another job shares the PC; `cargo test --features cli` about 7-9 min.

**Budget numbers (after Q7, unchanged from Q6).**
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
| GUI | 2,951 / 3,000 | **49** |

New tests go in `src-tauri/tests/`, which is not counted.

**Other things to keep.**
- Safety rules for unattended runs: no microphone, no keystroke dispatch, `--shadow`/`--replay`/`--presenter none` only, TTS to files only, and stop every process you start, by PID.
- The shared `CARGO_TARGET_DIR` (`C:/dev/q3-10x-target`): building an older commit there overwrites the head's debug binaries, and two `cargo test` runs in it at once can crash rustc. Use a separate target for parallel work.
- After any cargo or tauri build, `src-tauri/gen/schemas/*.json` and sometimes `src-tauri/Cargo.toml` show as modified (line endings only). Restore them; do not commit them (Q6-12, #91).
- Write files with the Edit or Write tools, not Bash heredocs (Q3-12, #56). Check `git grep -lIP '[\x00\x0b]'` before every commit, and `git diff --stat` before committing state files.

### Still open from earlier steps
- **Founder checklist** before any pilot leaves shadow mode: QA report Q2 §10 (M1–M14) and the security checklist, plus Q3-07 (#51): start and stop listening in the GUI, unplug, and one `voxxad --device default --shadow` run on the booth PC (with `--log-dir` set to a local folder). First clear `%LOCALAPPDATA%\com.voxxa.app`; keep `%APPDATA%\voxxa\models\ggml-tiny.bin`.
- **Founder:** record the two public-domain hymn fixtures (Q5-01, #73); they would also re-check ADR-016's numbers on singing.
- **User decisions:** the cut-list ADR-012 (evidence in the Q2 QA report §9); Q6-01 (#83, a ~6-line GUI fix) and Q6-10 (#89, two unused `SmartConfig` fields).
- **Q1 A15 follow-ups:** `fixtures/setlists/pd-hymns.json` and `fixtures/pco/plan-sample.json` fit no parser (Q7-12); re-run the Windows verification (a003992) on the merged head (`handoff-local-windows.md`); finish reconciling `docs/signing-policy.md` with `SIGNING.md`.

**Process rule (user, 2026-11-11):** every prompt (Q2…Q27) also runs independently through ponytail. Run it as two lanes plus cross-review for code steps (Q8 is a code step), or `ponytail` + `ponytail-audit` for verification and close steps. Record the result in `reviews/YYYY-MM-DD-ponytail-<Q>.md`.

---

## Handoff History
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
