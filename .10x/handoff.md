# Current Handoff

## User decisions for Q5 (2026-11-13, before the unattended run)
- **Fixtures:** there are no founder recordings yet. Q5 builds its two fixtures from the committed public-domain WAVs (spoken, synthetic Amazing Grace v1/v2 and the prayer in `src-tauri/tests/fixtures/voxxad/`). `fixtures/SOURCES.md` labels them as stand-ins; a Q5 ticket tracks replacing them with the founder's two public-domain hymn recordings. No new audio is committed: `fixtures/` holds only JSON, `README.md` and `SOURCES.md`, and each fixture's audio is rebuilt reproducibly from the committed WAVs.
- **CI gate:** CI has no Whisper model. Each fixture commits a recorded hypothesis log (voxxad session JSONL from a Whisper Tiny run). The CI job replays it (`voxxad --replay`, no recognizer, no audio), scores the actions against `truth.jsonl` and fails when a metric is worse than `fixtures/baseline.json`. The full mode (`voxxad --stdin-audio --shadow`, real model) runs locally and can re-record the logs.
- **Line budget:** one new recorded row, "voxxa-eval (src/bin/voxxa-eval*)", capped at **600** and excluded from "everything else". No other budget is raised.
- **Pre-approved:** push `q5/base`, open a draft PR against `main`, add the eval job to `.github/workflows/ci.yml` and fix CI forward; prove the CI gate in real CI with a deliberately broken threshold (then revert); merge into `main` once Q5's exit criteria are met and every CI job is green; file Q5 tickets as GitHub issues (label `q5`); agents may stop processes they started, by PID.
- **ADR:** ADR-012 is still reserved for the cut list; Q5 writes ADR-015.
- **QA mode** (as in Q2–Q4): no microphone, no keystroke dispatch; `--shadow`, `--replay` or `--presenter none` only; public-domain WAVs; `ggml-tiny.bin` is present.

---

## From: Q5 → To: Q6 (conductor against the budget)
**Date:** 2026-11-13
**Phase:** Phase 4 Implementation done on `q5/base` (code head **c10e963**, plus the state commit). **Not merged: verifier B failed in round 3.** The orchestrator first closes Q5-08 and Q5-09 (and Q5-10's M52 and M55) with tests and has a verifier confirm them, then pushes, opens the PR, proves the CI gate in real CI, merges (all pre-approved) and files the tickets. Q6 starts from `main` after that merge.

### What Q5 did
- Added `voxxa-eval` (`src-tauri/src/bin/voxxa-eval/`: `main.rs`, `score.rs`, `audio.rs`), a third binary that runs the `voxxad` beside it as a subprocess and scores its `action` lines against an operator's `truth.jsonl`. Design: ADR-015 (Accepted; Decision 8 amended at Q5 close).
- Two stand-in fixtures, `fixtures/recordings/prayer-then-reprise` and `prayer-without-pause` (JSON only); `fixtures/baseline.json`; `fixtures/README.md` (running, `truth.jsonl`, exact metric definitions, baseline rules, recording a real fixture); `fixtures/SOURCES.md` (stand-ins labelled as synthetic speech, real recordings to replace them, Q5-01).
- A CI step, "voxxa-eval (fixtures vs baseline)", in the `check` job on Ubuntu, Windows and macOS (replay, no model).
- Ran as two lanes (`q5/10x`, `q5/ponytail`) with cross-reviews, a comparator (base ponytail, seven 10x grafts, the review's fixes), a finisher and three verifier rounds: `reviews/2026-11-13-ponytail-q5.md`.
- All 7 gates pass on Windows at c10e963; 168 tests (55 unit + 113 integration; `voxxa_eval` 26).
- No GUI, conductor, `voxxad`, library or `Cargo.toml` change; no new crates.
- Record: `decisions/sde/church-autoadvance.md` ("Q5 final"). Tickets: `reviews/2026-11-13-q5-tickets.md` (11 open: 4 medium, 7 low; label `q5`).

### Open before the Q5 merge (verifier B, round 3)
- **Q5-08 (M65, important):** no test covers an action before the first truth row's window; a lookup that falls back to row 0 would score an early first-slide goto (during an opening prayer) as a hit.
- **Q5-09 (M66):** no test checks that the total row adds up every fixture.
- **Q5-10:** M52 (an unknown action kind skipped silently) and M55 (an unblank timed at a double blank) should be closed with the two above; M57 (the mode check is unpinned) and M68 (a stderr message) may be deferred with a reason.
- All are test-only additions to `src-tauri/tests/voxxa_eval.rs`; each ticket gives the exact test.

### What Q6 must know
**Running `voxxa-eval`** (from the repository root; see `fixtures/README.md`):
```sh
cargo build --manifest-path src-tauri/Cargo.toml --bin voxxad --bin voxxa-eval
cargo run --manifest-path src-tauri/Cargo.toml --bin voxxa-eval                     # replay: what CI runs
cargo run --manifest-path src-tauri/Cargo.toml --bin voxxa-eval -- --full 2>/dev/null  # Whisper Tiny, local
cargo run --manifest-path src-tauri/Cargo.toml --bin voxxa-eval -- --baseline <file>   # another thresholds file
cargo run --manifest-path src-tauri/Cargo.toml --bin voxxa-eval -- fixtures/recordings/<name>  # one fixture
```
- **Replay (default, CI):** `voxxad --replay <fixture>/session.jsonl --setlist <fixture>/setlist.json` for every folder in `fixtures/recordings`. No model, no audio, the same result on every OS. About 27 s for both fixtures in a debug build (Q5-05).
- **Full (`--full`, local only):** rebuilds `audio.wav` from `audio.json` (git-ignored), then runs `voxxad --stdin-audio audio.wav --setlist … --shadow --model tiny --log-dir .` in a temporary folder, at file speed. Needs `%APPDATA%\voxxa\models\ggml-tiny.bin` (present). Debug: about 2.5-6 minutes for both fixtures (93 s of audio). Do not use a release build on Windows (Q4-01).
- **Exit codes:** 0 when no metric is worse than the baseline; 1 when one is (`FAIL <row>: <metric> is <v>, baseline needs <=|>= <limit>`); 2 on an error, including `voxxad` running in the wrong mode.
- **Safety:** `voxxa-eval` only ever passes `--replay` or `--shadow` to `voxxad`.

**What replay measures, and what it does not.**
- Replay re-runs today's conductor on the recorded recogniser output (`hypothesis`, `tick` and `speech` lines). A conductor-logic change in Q6 is measured by replay and by CI alone.
- **But replay builds the conductor from the `SmartConfig` stored in each log's start line (`info.config`), not from today's defaults (Q5-07).** If Q6 changes a default (`silence_to_blank_secs`, `unrecognized_speech_to_blank_secs`, a threshold, the buffer size if it lives there), replay and CI will not see it until the logs are re-recorded or Q5-07's `--current-config` option is built (about 10 lines, suggested in the ticket). Decide which before tuning.
- A recogniser change (window, hop, stitcher, model) always needs the logs re-recorded.

**Current numbers against Q6's thresholds** (Whisper Tiny on the stand-ins; full mode and replay agree):

| Metric | Q6 threshold | Now (total) | Status |
|---|---|---|---|
| p95 latency | ≤ 2,500 ms | **8,900 ms** (median 2,900 ms) | fails by 6.4 s: the advance into verse 2 comes 8.9 s after it starts (Q4-04) |
| False advances | ≤ 1 per 10-song set | 0 per fixture | met, but each stand-in fixture is a one-song set of 2-3 slides; a 10-song set is not measured yet |
| Precision / recall | (no Q6 number) | 1.00 / 1.00 | |
| Double advances | (none expected) | 0 | |
| Blank | (no Q6 number) | 200 ms after a 4 s pause; **6,600 ms** into a prayer with no pause (Q4-03) | |
| Unblank | (no Q6 number) | 1,500 ms | |

Per fixture: `prayer-then-reprise` 3 slides, latencies 2,900 / 8,900 / 1,500 ms, blank 200 ms, unblank 1,500 ms; `prayer-without-pause` 2 slides, latencies 2,900 / 8,900 ms, blank 6,600 ms. Thresholds in `baseline.json`: precision and recall 1.0, false and double 0, median 3,000, p95 9,000, blank 500 (`prayer-then-reprise`) and 7,000, unblank 2,000.

**Known conductor defect for Q6 (Q5-04, medium):** a song that starts after an opening prayer is not shown until its next verse. Before a song is committed, the conductor keeps the prayer's words in its 40-word buffer (`src-tauri/src/aligner/mod.rs:370-411`). With the prayer's lines removed, verse 1 shows 2.8 s after it starts. Reproduction: the 10x lane's fixture `standin-prayer-between` at commit 30e8990 on `q5/10x` (rewrite its blank row as `{"t_ms":23200,"blank":true}`). When Q6 fixes it, add that shape as a third fixture in the same commit, with its `baseline.json` row and an updated `total`. Today it cannot be added: Voxxa never blanks into that prayer, so its `blank_ms` is `miss`.

**The fixtures are stand-ins.** They are Amazing Grace verses 1 and 2 and a synthetic prayer, read by the Windows SAPI voice: spoken, not sung, one voice, a quiet room. Use them to catch regressions and to see whether a conductor change moves latency and blank timing, not to claim accuracy in a church. Real recordings replace them later (Q5-01: the founder's two public-domain hymn recordings, made with `fixtures/tools/record-session.ps1`; the README lists the hand-added first slide row and blank rows, Q5-02).

**Re-recording the logs and updating `baseline.json`:**
1. Re-record when the recogniser changes, or when a `SmartConfig` default changes and Q5-07 is not built: `cargo run --manifest-path src-tauri/Cargo.toml --bin voxxa-eval -- --full --record` (all fixtures) or add `fixtures/recordings/<name>` for one. It writes `<fixture>/session.jsonl` with a relative log path (no user folder). Say why in the commit.
2. Run replay and read the table.
3. Thresholds, not last numbers: each latency is the next multiple of 500 ms strictly above the measured worst (8,900 → 9,000; 1,500 → 2,000); counts and ratios are exact. Every metric that has a value in a row needs a threshold in that row.
4. **Tighten** in the same commit as a change that improves a metric (Q6 aims at p95 ≤ 2,500 ms). Under the README, **loosen** only with the reason in the commit message. The orchestrator's notes record a stricter user choice for Q6 (2026-11-13): the baseline may be tightened, never loosened. Confirm it at the start of Q6.
5. Adding a fixture: add its row and update `total`; the gate checks the full set.

**Also noted for Q6** (orchestrator's notes of the user's choices, 2026-11-13; confirm at the start of Q6): add 2-3 offline Windows-TTS fixtures of public-domain hymns with a refrain or a jump, WAVs committed beside the test WAVs and labelled synthetic; a new budget row "conductor evidence (ADR-009 minimum)" capped at 500, with the conductor row kept at 1,200.

**Budget numbers (after Q5, c10e963).**
| Row | Used / cap | Left |
|---|---|---|
| everything else in src-tauri/src | 2,286 / 2,500 | 214 |
| conductor (`aligner/`) | 1,188 / 1,200 | **12** |
| recognizer (`transcription/` + recognizer code) | 432 / 700 | 268 |
| voxxad binary (`src/bin/voxxad*`) | 548 / 600 | 52 |
| voxxa-eval (`src/bin/voxxa-eval*`; new in Q5, user decision) | 469 / 600 | 131 |
| presenters | 1,149 / 2,000 | 851 |
| importers + planning_center | 1,362 / 1,600 | 238 |
| GUI | 2,951 / 3,000 | 49 |

The conductor has 12 lines left. Q6's conductor work (ADR-009 minimum, Q4-04, Q5-04) needs code removed first or a budget decision before code, never a silent raise. Tests in `src-tauri/tests/` are not counted.

**Other things to keep.**
- Event schema v1, replay rules and the hop cadence are unchanged from Q4 (ADR-013, ADR-014).
- Safety rules for unattended runs are unchanged: no microphone, no keystroke dispatch, a local mock presenter only, and stop every process by PID.
- Write files with the Edit or Write tools, not Bash heredocs (Q3-12, #56: in Q5 a heredoc again put a raw line break into a string literal, in the 10x lane). A mutation harness that restores sources with `mv` leaves them older than the build, so cargo keeps the mutant's binary: touch the sources after restoring. An edit script that opens a file for writing and then fails empties it (1b0cc5d): check `git diff --stat` before committing state files.
- Check `git grep -lIP '[\x00\x0b]'` before every commit.

### Still open from earlier steps
- **Founder checklist** before any pilot leaves shadow mode: QA report §10 (M1–M14) and security review §10, plus Q3-07 (#51): start and stop listening in the GUI (a `transcription` event every 1-3 s of speech), unplug, and one `voxxad --device default --shadow` run on the booth PC. First clear `%LOCALAPPDATA%\com.voxxa.app`; keep `%APPDATA%\voxxa\models\ggml-tiny.bin`.
- **Founder:** record the two public-domain hymn fixtures (Q5-01).
- **User decision:** the cut-list ADR-012 (evidence in the QA report §9).
- **Q1 A15 follow-ups:** `fixtures/setlists/pd-hymns.json` and `fixtures/pco/plan-sample.json` do not fit the May parsers (the eval's fixtures use the `voxxad --setlist` format instead); re-run the Windows verification (a003992) on the merged head (`handoff-local-windows.md`); finish reconciling `docs/signing-policy.md` with `SIGNING.md`.

**Process rule (user, 2026-11-11):** every prompt (Q2…Q27) also runs independently through ponytail. Run it as two lanes plus cross-review for code steps, or `ponytail` + `ponytail-audit` for verification and close steps. Record the result in `reviews/YYYY-MM-DD-ponytail-<Q>.md`.

---

## Handoff History
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
