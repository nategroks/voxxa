# Project Status

## Project: Voxxa — voice-following slide advancer for churches
**Current Phase:** Q6 done on `q6/base` (head 97cc21b plus this state commit): ADR-009's minimum built (ADR-016). The user decided the baseline item on 2026-11-18, the ponytail lane was re-run and compared, and **both verifiers passed at 97cc21b**. Next: CI + PR + merge by the orchestrator, then Q7 (close Increment 1′). Q5 complete and merged into `main` (PR #72, 69f7d27).
**Last Updated:** 2026-11-19 by 10x-team orchestrator
**Health:** On Track

## Progress

### Done
- [x] Discovery Protocol; `[DISCOVERED]` decisions for CTO, Architect, Staff Engineer, DevOps, SDE
- [x] Deep research, 5 angles, confidence-rated: `specs/2026-10-24-research-report.md`
- [x] User research plan: `specs/2026-10-24-user-research-plan.md`
- [x] Phase 1 Strategy: `specs/2026-10-24-business-strategy.md`; `decisions/cto/`, `decisions/product-manager/`
- [x] Phase 2 Design: `specs/2026-10-24-software-strategy.md`; ADR-001…009; `decisions/architect/`, `decisions/staff-engineer/`
- [x] Phase 3 Planning: `specs/2026-10-24-delivery-plan-and-prompts.md`; `decisions/engineering-manager/`, `decisions/senior-engineer/`

- [x] User approved specs and ADR-001…009 (Accepted); GPL confirmed; US LLC confirmed; pilot church #1: The Lighthouse, Midlothian
- [x] Increment 0: P0.1 positioning, P0.2 dead weight + licence + deny, P0.3 CI (`decisions/sde/church-autoadvance.md`)

- [x] CI green on all five jobs (ubuntu/windows/macos fmt+clippy+test, cargo deny, line budgets) at d976698 on PR nategroks/voxxa#1; one real defect found and fixed on the way (icon.ico/icon.icns were PNGs)

- [x] 2026-11-11 Discovery of `origin/claude/affectionate-lamport-8al8B` (44 commits, May 2026, never merged): presenter drivers, importers, Planning Center, conductor, VAD, HTTP API, Flatpak. Notes in role indexes; ADR-010 Proposed; prompt pack regenerated for `/10x-Team:10x-team`.

- [x] 2026-11-11 PR #1 cargo deny went red on nine newly published advisories; fixed by openssl 0.10.82 and moving the tauri family to 2.12.5 (updater 2.14.1 → zip 9, rust-version 1.90, zip ignores deleted). CI green on f177fbe.

- [x] 2026-11-11 **Q0** (Phase 0–2: CTO, Architect, Staff Engineer), with an independent ponytail lane:
  - every [DISCOVERED] claim verified against the code: `reviews/2026-11-11-q0-claim-verification.md`
  - ADR-010 now carries the 110-path table (Appendix A) and amendments A1–A15
  - ponytail review and audit (12 must-fix items, all confirmed by a skeptic): `reviews/2026-11-11-ponytail-q0.md`
  - role indexes updated
  - no merge, no code

- [x] 2026-11-11 **Q0 user gate.** All four recommendations were taken:
  - ADR-010 Accepted as amended
  - feature cuts decided after Q2 in a separate ADR
  - ADR-011: never *project* lyrics, operator previews allowed (amends ADR-007)
  - Q2 widened (M → L) to fix the four wrong-slide defects; the prompt pack is updated

- [x] 2026-11-11 `local/windows-fixtures` merged into this branch (54183c8, clean). It brings the recording kit, PD-hymn setlists, PCO sample, SignPath policy and Windows QA of Increment 0. It went in ahead of Q1 because it touches none of the 110 merge paths: A15, executed early. The May-branch conflict set is unchanged at 6 files. The Drive folder linked in `fixtures/SOURCES.md` was checked: private, owner only.

- [x] 2026-11-11 **Q1** (Phase 4: SDE + Staff Engineer + DBA), run as two independent lanes (10x-team and ponytail), cross-reviewed, compared and verified: `reviews/2026-11-11-ponytail-q1.md`.
  - **Merge commit** 9987b62 (parents 97aa915 and f7b56e6), then 19 logical commits. The 10x lane is the base, with ponytail grafts.
  - **Gates on Windows** (rustc 1.99.1): frontend, fmt, clippy `-D warnings`, test (37 unit + 4 integration), `cargo deny` (no ignores), budgets and the dictat grep all pass. The adversarial verifier passed on round 1.
  - **Dependencies:** zip 9, quick-xml 0.41, pdf-extract 0.12 (lopdf 0.42; no feature gate needed), rusqlite 0.41 bundled, reqwest on `rustls-tls-native-roots`; env_logger and `@tauri-apps/api` removed; A2 floors. Tauri 2.12.5 and openssl 0.10.82 are kept.
  - **Content:** PD examples (It Is Well, Amazing Grace, Be Thou My Vision); GPL metainfo; README and website "never project lyrics" (ADR-011); `loc_budget.py` re-based (GUI 2,966/3,000).
  - **Pre-existing fix:** `build.yml` was invalid YAML (6ef2d13).
  - **Records:** `decisions/sde/church-autoadvance.md` (merge record) and `decisions/dba/church-autoadvance.md` (bundled SQLite).

- [x] 2026-11-11 **Q2** (Phase 5: QA Engineer + Security Engineer), with the ponytail lane: `reviews/2026-11-11-qa-report.md`, `reviews/2026-11-11-security-review.md`, `reviews/2026-11-11-q2-tickets.md`, `reviews/2026-11-11-ponytail-q2.md`.
  - **Method:** simulate mode (user decision): GUI driven over WebView2 remote debugging, presenter on a local mock, no mic, no live keystrokes; one real Whisper Tiny download; transcription on locally synthesised PD speech. 35 of 38 commands exercised; HTTP API with and without a token; device loss by code reading only.
  - **Fixed with tests** on `q2/10x` (1568810..b09373d, 24 commits): Q0 audit 1–13, 15, 16 and the ADR-007 fixtures, plus PDF page split, single-instance guard and one verifier finding (Q2-44). Two fix lanes (10x, ponytail), cross-reviewed; base 10x with ponytail grafts; adversarial verifier.
  - **Security:** HTTP API token mandatory and browser/rebinding requests refused (High, fixed); PDF abort (Critical, fixed); capability file least-privilege. No Critical/High open. Outbound counter counts every reqwest send; no credentials logged at Info.
  - **ADR-011:** the stage window cannot default to an audience output; copy and placement are a Low ticket.
  - **Gates on Windows** (rustc 1.99.1): frontend (clean export), fmt, clippy `-D warnings`, test (45 unit + 33 integration), `cargo deny`, budgets (no budget raised; everything else 2489/2500), dictat grep.
  - **Tickets:** 34 (14 medium, 20 low), ready to file as GitHub issues.
- [x] 2026-11-12 **Q2 ponytail-audit re-run** on 3446e09 (the first run's output was blocked): 16 findings; 3 new fix-now items (PT2-01 short or unrelated transcripts start a song or re-show lyrics, PT2-02 tray Blank inverts the Keystroke toggle, PT2-05 ProPresenter WS/OpenSong goto hits the wrong presentation) and one older wrong-slide defect found while checking b09373d (Prev below the first song) fixed with tests in f2519ba, 8846f36, a50ee6a, bde776b, b677d69; all 7 gates pass at b677d69 (53 unit tests; budget 2482/2500); tickets now 44 (Q2-55 … Q2-64 added); the last wrong-slide path (a worship phrase during a prayer blank could show the next slide through the in-song advance) is fixed with a test in 4129dcd, and the PT2-02 focus check is now guarded by a pure-helper test (713f40f, part of Q2-60); a second verifier passed at 713f40f: all 7 gates pass (54 unit tests; budget 2483/2500, no budget raised), every mutation caught by a test or a gate (`reviews/2026-11-11-ponytail-q2.md` §8.6). Q2's "no open wrong-slide defect" is met again; the hold on the PR #1 push and on filing the tickets is lifted.

- [x] 2026-11-12 **Q3** `voxxad` CLI (Phase 2 check + Phase 4: ADR-013), with the ponytail lane: `reviews/2026-11-12-ponytail-q3.md`, `reviews/2026-11-12-q3-tickets.md`, `adrs/013-voxxad-cli-shared-loop-and-replay.md`, record in `decisions/sde/church-autoadvance.md`.
  - **Head:** `q3/base` @ **f2bcea1** (24 commits on 74f6bd2): `q3/10x` merged (f6d18b3), the comparator's 10x fix and six ponytail grafts (7cc75b2..1f5e31b), and test-only fixes from verifier rounds 1 and 2 (c54f16e..37dd632). `q3/ponytail` (3550b05) stays local for reference.
  - **Built:** one listening loop, `voxxa_lib::listen::run`, shared by the GUI and the new `voxxad` binary. Every conductor input goes through `Conductor::apply`. JSON-lines schema v1. A 14-day session log. `--replay` is byte-identical on action lines without the model. `--shadow` builds no presenter.
  - **Gates on Windows** (rustc 1.99.1): all 7 pass at f2bcea1.
  - **Tests:** 108 (54 unit + 54 integration; `voxxad_events` 19). None needs the Whisper model or an audio device.
  - **Budgets:** everything else 2,496/2,500 (+13); conductor 1,153/1,200; new row voxxad 531/600 (user decision, recorded deviation); GUI 2,951/3,000; no other row changed.
  - **Exit demo** (Whisper Tiny, public-domain speech): goto Amazing Grace/0 at 8.1 s, goto Amazing Grace/1 at 22.8 s, blank at 27.8 s, prayer shows nothing. Replay byte-identical (`cmp`). The ProPresenter mock's `/_log` stayed empty under `--shadow`.
  - **Verifiers:** round 1 (M2–M6) and round 2 (L1–L5, R2) closed with tests. **Round 3 at f2bcea1: verifier A passed; verifier B failed.** Open: Q3-01 (deleting the `--shadow` guard at `bin/voxxad/main.rs:312-314` passes every gate) and Q3-02 (three mutations of the GUI event sink in `commands/mod.rs` pass every gate).
  - **Close-out by the orchestrator (912b978):** Q3-01 closed with `shadow_never_connects_to_the_presenter` (model-free; the N9 mutation now fails it). Q3-02 deferred to Q4 (recorded in the ticket: no GUI behaviour change, no protection lost against 74f6bd2, not an exit criterion, does not fit the 4 lines left). All 7 gates pass again at 912b978: 109 tests. The change is test-only, so verifier A's exit demo at f2bcea1 still applies.
  - **Tickets:** 12 filed as GitHub issues #46–#57 (label `q3`; 6 medium, 6 low); Q3-01 closed before filing.

- [x] 2026-11-13 **Q4** Recognizer trait and streaming hop (Phase 4: ADR-014), with the ponytail lane: `reviews/2026-11-12-ponytail-q4.md`, `reviews/2026-11-12-q4-tickets.md`, `adrs/014-recognizer-trait-and-streaming-hop.md` (ADR-013 amended), record in `decisions/sde/church-autoadvance.md` ("Q4 final").
  - **Head:** `q4/base` @ **a4bae0e** (25 commits on a515d4d, including the merge): `q4/10x` merged (0d16b0e), three ponytail grafts and the review fixes (b43151f..4722786), verifier round 1 fixes (9f666f1..bafa0b4) and round 2 fixes (b2b8d0b..a4bae0e). `q4/ponytail` (129d795) stays local for reference.
  - **Built:** ADR-004's `Recognizer` trait; Whisper behind it as a 5 s window decoded every 1 s (`single_segment`, token timestamps, `initial_prompt` only with `bias_enabled`, off by default); the 5 s batch in `listen::run` replaced by the hop loop. A gate skips hops when the loop falls behind, so work stays bounded and file speed stays deterministic. A stitcher sends the conductor only new words (a `hypothesis` line every 1-3 s of speech); a session-log-only `window` line records every decoded hop (every 1000 ms of speech at file speed). The GUI event mapping is pure and tested (closes #46). Conductor API unchanged; three blank and re-show fixes (closes #52).
  - **Gates on Windows** (rustc 1.99.1): all 7 pass at a4bae0e.
  - **Tests:** 142 (55 unit + 87 integration; `voxxad_events` 31, `recognizer` 11, `gui_events` 10). Only the real-model 10 s WAV test needs `ggml-tiny.bin`; it ran locally and passed, and it skips without the model (CI).
  - **Budgets:** everything else 2,286/2,500; new row recognizer 432/700 (user decision, recorded deviation); conductor 1,188/1,200; voxxad 548/600; GUI 2,951/3,000; presenters and importers unchanged. No budget raised.
  - **Exit demo** (Whisper Tiny, debug, file speed, `--shadow`, 44.4 s public-domain service): `window` lines every 1000 ms of speech; goto Amazing Grace/0 at 6.0 s (Q3: 8.1 s), goto Amazing Grace/1 at 25.0 s, blank 0.3 s into the prayer, nothing shown during it; replay byte-identical on the action lines. Inference 1.8-1.9 s per hop in debug; release builds are unoptimised on Windows (Q4-01).
  - **Verifiers:** round 1 failed (exit cadence and test gaps), round 2 failed (prayer not blanked, re-shown slide after a pause, session-log `t` order); all fixed with tests. **Round 3 at a4bae0e: both verifiers passed.**
  - **Tickets:** 13 open (4 medium, 9 low), filed 2026-11-13 as GitHub issues #59–#71 (label `q4`).

- [x] 2026-11-13 **Q5** `voxxa-eval` and the first fixtures (Phase 4: ADR-015), with the ponytail lane: `reviews/2026-11-13-ponytail-q5.md`, `reviews/2026-11-13-q5-tickets.md`, `adrs/015-voxxa-eval-fixtures-and-replay-gate.md`, record in `decisions/sde/church-autoadvance.md` ("Q5 final"). **Implemented; not merged** (see the verifiers below).
  - **Head:** `q5/base` @ **c10e963** (19 commits on 3af8a1a, including the merge): `q5/ponytail` merged as the comparator's base (6f2314d), seven 10x grafts and the ponytail review's fixes (7736464..a0da74d), verifier round 1 fixes (7660b26..e31ddb8) and round 2 fixes (624acad..c10e963). `q5/10x` (b6650a2) and `q5/ponytail` (c9da97c) stay local for reference. 1b0cc5d committed an empty `status.md` by mistake; c10e963 restores it (no history rewritten).
  - **Built:** a third binary, `voxxa-eval` (`src-tauri/src/bin/voxxa-eval/`), that runs the `voxxad` beside it as a subprocess and scores its `action` lines against `truth.jsonl`. Replay of the committed session logs by default (CI, no model); `--full` runs Whisper Tiny with `--shadow` on audio rebuilt from an `audio.json` recipe; `--record` re-records the logs. Two stand-in fixtures (`fixtures/recordings/prayer-then-reprise`, `prayer-without-pause`; JSON only), `fixtures/baseline.json` (per-row thresholds), `fixtures/README.md` (metric definitions, baseline rules, recording), `fixtures/SOURCES.md` (stand-ins labelled). A CI step in the `check` job on all three OSes. No GUI, conductor, `voxxad`, library or `Cargo.toml` change; no new crates.
  - **Gates on Windows** (rustc 1.99.1): all 7 pass at c10e963 (finisher and both round-3 verifiers); the CI eval step exactly as `ci.yml` runs it exits 0.
  - **Tests:** 168 (55 unit + 113 integration; `voxxa_eval` 26, `voxxad_events` 31, `recognizer` 11, `dispatch` 12, `gui_events` 10). None needs the model except the Q4 real-model test, which skips without it.
  - **Budgets:** new row voxxa-eval 469/600 (user decision, recorded deviation, ADR-015 D11); everything else 2,286/2,500; conductor 1,188/1,200; voxxad 548/600; recognizer 432/700; GUI 2,951/3,000; presenters 1,149/2,000; importers 1,362/1,600. No budget raised.
  - **Eval table** (Whisper Tiny full mode; replay prints the same):

    | Fixture | Slides / gotos / hits / missed | Precision | Recall | Median / p95 | False / double | Blank | Unblank |
    |---|---|---|---|---|---|---|---|
    | prayer-then-reprise | 3 / 3 / 3 / 0 | 1.00 | 1.00 | 2,900 / 8,900 ms | 0 / 0 | 200 ms | 1,500 ms |
    | prayer-without-pause | 2 / 2 / 2 / 0 | 1.00 | 1.00 | 2,900 / 8,900 ms | 0 / 0 | 6,600 ms | - |
    | total | 5 / 5 / 5 / 0 | 1.00 | 1.00 | 2,900 / 8,900 ms | 0 / 0 | 6,600 ms | 1,500 ms |

    Q4-04 measured (p95 8.9 s against Q6's 2.5 s budget); Q4-03 measured (blank 6.6 s into a prayer with no pause). The 10x lane also found Q5-04: verse 1 after an opening prayer is never shown.
  - **Exit demo:** (a) full mode prints the table (2m35s at c10e963; verifier A reproduced it on a fresh clone); (b) replay prints the same table and exits 0 against `baseline.json`; (c) a scratch baseline with `total.p95_ms` 1000 exits 1 with `FAIL total: p95_ms is 8900, baseline needs <= 1000`; (d) `cargo test` covers the metric code with hand-computed cases.
  - **Verifiers:** rounds 1 and 2 found surviving mutants and two gate gaps (a dropped threshold switched its gate off; a replay that ran Whisper passed locally); all fixed with tests and two small code fixes. **Round 3 at c10e963: verifier A passed; verifier B failed** (63 of 70 mutants killed). **Open:** Q5-08 (M65: an action before the first truth row's window could be scored as a hit; important), Q5-09 (M66: a total row built from one fixture passes), and Q5-10 (M52 unknown action kind skipped, M55 unblank timed at a double blank, M57 mode check unpinned, M68 cosmetic). **The PR must not be merged until Q5-08 and Q5-09 (and M52, M55) have tests that fail under their mutants and a verifier confirms it.** M57 and M68 may be deferred with a recorded reason.
  - **Tickets:** 11 open (4 medium, 7 low), not filed yet (label `q5`). No earlier issue is closed by Q5.

- [x] 2026-11-13, closed out 2026-11-18: **Q6** Conductor against the budget (Phase 2 → 4: ADR-016), with the ponytail lane: `reviews/2026-11-13-ponytail-q6.md`, `reviews/2026-11-13-q6-eval-before-after.md` (summary), `reviews/2026-11-13-q6-before-after.md` (per hit), `reviews/2026-11-13-q6-tickets.md`, `adrs/016-adr-009-minimum-implemented.md`; records in `decisions/{architect,sde,dba,qa,security}/church-autoadvance.md` ("Q6 final" and "Q6 close-out"). **Implemented and verified; ready for CI, PR and merge.**
  - **Final head:** `q6/base` @ **97cc21b** (31 commits on 77be270, including the merge), plus the state commit "state: Q6 ponytail re-run, baseline decision, close-out".
    - Up to 4edd7f6 (2026-11-13): three synthetic TTS fixtures (733c7e1); `q6/10x` merged as the comparator's base (d92800c; the lane's commits e9573cc..ca84cd9); two review fixes (cf1a4c2, e2fd129); verifier round 1 fixes (69795e4, 0dc1e35) and round 2 fix (b4f3710); records (c56be81, 906e78b, 88f7870, 4edd7f6); state 151a8b3.
    - 2026-11-18: the user's baseline decision (85c736c); grafts from the ponytail re-run (d2fae8e, 8b866d9, 55e46a8, 587d2ad); records c5ae273; dependency fixes for gate 5 (b7a5382 zip 9.0.5, 734b467 chrono replaced by `time`); test pins (2aea4fd, b402c87, 0050b32, d41eb35, dca4637); records d57f060, 97cc21b.
    - `q6/10x` (ca84cd9) and `q6/ponytail` (7d4b4b6) stay local for reference.
  - **Ponytail re-run (2026-11-18):** the first ponytail build lane was stopped by an output content filter. The re-run (`q6/ponytail` @ 7d4b4b6, 4 commits on 733c7e1) built a smaller tracker (117 lines) that leaves song detection alone: total 19/19 hits, 0 false, median 2,310 ms, p95 8,490 ms on the five old fixtures. Its review asked for one must-fix (the tracker did not reset after its own move). The comparator kept `q6/base` (better on four rows and the total, mainly from the Q5-04 fix) and grafted tests, docs, the baseline cells and the decay clamp. Record: `reviews/2026-11-13-ponytail-q6.md` §10.
  - **Decision** (two independent measurements of 733c7e1, identical): 1 false advance over 6 sung songs = **1.67 per 10 songs** (limit 1) and **p95 11.23 s** over 13 hits (limit 2.5 s; median 4.12 s). Both limbs fail, so ADR-009's minimum was built (ADR-016, written before any code). Latency is stream time at file speed: no inference time, not wall clock; live it would be higher (Q4-02, #60).
  - **Built:** a position tracker in `src-tauri/src/aligner/evidence/`: optional `sections[].kind` in setlist v1 (old setlists serialise byte for byte), repeat-aware priors in `SmartConfig.evidence`, word-token scoring on the last 12 recognised words, an accumulator with decay 0.3, threshold 0.35, margin 0.05 and `min_dwell_ms` 1,500; the operator always wins. In `aligner/mod.rs` the next-slide check is replaced by one call to the tracker, and while blank song detection reads only the newest transcript (Q5-04). `MachineState`, blanking, `manual_step`, the conductor's public API and event schema v1 are unchanged. No GUI, `voxxa-eval` or recogniser change; no sockets. `Cargo.toml` changed only to clear two RustSec advisories for gate 5: zip 9.0.0 → 9.0.5, and chrono replaced by `time` (already in the tree; diagnostics stamps and `voxxad` log names now use it; no advisory ignored, `deny.toml` unchanged). Fixtures: `refrain-it-is-well`, `jump-be-thou-my-vision`, `prayer-between-songs` (synthetic TTS, WAVs in `src-tauri/tests/fixtures/voxxad/`) and `prayer-before-first-song` (Q5-04's reproduction); six in all.
  - **Gates on Windows** (rustc 1.99.1) at 97cc21b, all pass: 1 npm, 2 fmt, 3 clippy `-D warnings`, 4 `cargo test`, 5 cargo-deny (advisories, bans and licences ok; the old no-license-field warning for voice_activity_detector), 6 loc budget, 7 the `dictat` grep, and the CI eval step (exit 0, "OK: no metric is worse than ... baseline.json"). Only binaries match the control-byte grep. The finisher ran them on dca4637 (the last code or test commit; 97cc21b changes only docs, and gates 6 and 7 were re-run on it); verifier A ran them on a `git archive` export of 97cc21b and verifier B in a detached worktree at 97cc21b.
    - On 2026-11-13 (4edd7f6) gate 4 and the CI eval step failed on the jump cells; the user's decision fixed that. On 2026-11-18 gate 5 turned red with an unchanged `Cargo.lock` (new advisory data); b7a5382 and 734b467 fixed it.
  - **Tests:** 201 in 21 test binaries, 0 failed (56 unit + 145 integration; `conductor_evidence` 25, `conductor_scripted` 5 (ported from the ponytail re-run), `conductor_unchanged` 2, `voxxa_eval` 26, `voxxad_events` 31).
  - **Budgets** at 97cc21b: new row conductor evidence (`aligner/evidence/`) **215/500** (user decision 3, recorded deviation, ADR-016 Decision 8); conductor 1,188 → **1,173**/1,200; everything else 2,309/2,500 (+23 in Q6, mostly the chrono → `time` change in diagnostics); importers 1,386/1,600 (+24, `sections: Vec::new()` in `Song` literals); voxxad 552/600; recognizer 432/700; voxxa-eval 469/600; presenters 1,149/2,000; GUI 2,951/3,000. No cap raised.
  - **Eval, before (733c7e1) → after (97cc21b; the same numbers as at 4edd7f6)** (replay; full mode with Whisper Tiny gives the same table, checked by the finisher on dca4637 and by verifier A on 97cc21b): total precision 0.93 → 1.00, recall 0.68 → 1.00, false advances 1 → 0, double 0 → 0, median 4,120 → 2,310 ms, p95 11,230 → 4,870 ms, blank 7,280 ms unchanged, unblank 8,490 → 2,490 ms. Per fixture (median / p95 ms): jump 900/900 → **1,200/5,850** (recall 0.25 → 1.00); prayer-before-first-song 6,710/6,710 → 1,710/2,800 (recall 0.50 → 1.00); prayer-between-songs 8,490/11,230 → 2,490/4,870; prayer-then-reprise 2,900/8,900 → 2,900/2,900; prayer-without-pause 2,900/8,900 → 1,900/2,900; refrain 4,120/10,310 → 2,120/3,900 (recall 0.50 → 1.00, false 1 → 0). Rule after: **0 false advances per 10 songs (met); p95 4.87 s (not met)**.
  - **Exit criteria: met.** The decision file (ADR-016, now with five amendments) and the before/after numbers exist (`reviews/2026-11-13-q6-eval-before-after.md` §7). The eval is not worse on any fixture under the user's decision: only `jump-be-thou-my-vision`'s median and p95 rose (900/900 → 1,200/5,850, recall 0.25 → 1.00), inside the cells the user approved.
  - **User decision (2026-11-18, binding):** the jump row's `median_ms` 1,500 and `p95_ms` 6,000, a one-time exception to "tightened, never loosened" (applied in 85c736c, with the log's floor of 1,200 / 3,850 ms in the message); the `total` row's `blank_ms` 7,500 and `unblank_ms` 2,500 accepted (raised by adding the `prayer-between-songs` fixture, not by any fixture getting worse).
  - **Verifiers:**
    - 2026-11-13: rounds 1 and 2 found conductor lines that no test observed; each got a test. Round 3 at 4edd7f6: both verifiers failed, on the jump cells; B found Q6-04 and Q6-05.
    - 2026-11-18, after the re-run and the finish: the round at c5ae273 did not pass (gate 5 advisories; `LEAD_SLACK` survived; a weak operator scenario), and the round at d57f060 did not pass (three survivors: the resume slide, the `EvidenceConfig` values, apostrophes). Each was fixed with a dependency change or a test.
    - **At 97cc21b both passed.** A: every gate, replay and full mode equal on 733c7e1 and on the head, no fixture worse apart from the approved cells, every goto in its audio segment, no goto during a prayer, replay byte-identical across CRLF and LF checkouts. B: scope clean, 51 mutants, all killed except the Q5-04 reversion (killed by the CI eval step), three equivalent ones and four non-blocking survivors in the operator bookkeeping (Q6-13, Q6-14).
  - **Tickets:** 12 open (2 medium, 10 low), filed 2026-11-19 as #83–#94 (label `q6`); Q6-02 (b4f3710), Q6-04 and Q6-05 (d2fae8e) closed. New: Q6-13 and Q6-14 (verifier B's survivors), Q6-15 (try the ponytail re-run's scoring context). Closes #76 (Q5-04) and #62 (Q4-04) when merged.

### Release readiness (2026-11-11, after Q2)
- **Code:** Q2's exit criteria are met. No open crash, data-loss, freeze or wrong-slide defect from the audit or Q2; no open Critical or High security finding.
- **Not yet ready for a live service.** The fixes are proven against mocks and synthetic audio only. Before a pilot leaves shadow mode, the founder must pass checklist M1–M14 in the QA report (real mic, unplug, Keystroke driver with PowerPoint/ProPresenter, FreeShow/OpenLP/OpenSong, displays, Stream Deck) and the security checklist.
- **Linux/macOS:** first judged by CI on PR #1 (new: single-instance plugin pulls zbus on Linux; timing-based download tests).
- **Before a public release:** Q2-28 (release pipeline pinning, `.asc`, updater) and the cut-list ADR.

### In Progress
- [x] 2026-11-12 CI green on PR #1 (head 69b18ce, 5/5 jobs); PR #1 merged into `main` as 7bd84a1; CI green on `main` (5/5). The "Deploy Website" workflow fails because GitHub Pages is not enabled (pre-existing since April; user decision).
- [x] Q3 close-out (orchestrator).
  1. [x] Q3-01: model-free `--shadow` test added (912b978); the N9 mutation fails it.
  2. [x] Q3-02: deferred to Q4 by a recorded decision (tickets file).
  3. [x] All 7 gates on 912b978 (109 tests); test-only change, so the f2bcea1 exit demo stands.
  4. [x] `q3/base` pushed; PR #45 CI green (5/5 jobs at 035d381); merged into `main` as ec6d85e.
  5. [x] Filed the 12 Q3 tickets as GitHub issues #46–#57 (label `q3`).

  The user pre-approved the push, PR, merge and filing in steps 4 and 5.
- [x] 2026-11-12 Filed the 43 open Q2 tickets as GitHub issues #2–#44 (label `q2`); Q2-60 was closed by 713f40f before filing.

### Pending
- [ ] User decision: post-Q2 cut-list ADR (ADR-012) — HTTP API, mDNS, Hymnary, PDF, stage display, Flatpak, signing scaffolds, unused plugins; evidence in the QA report §9
- [ ] Founder: QA checklist M1–M14 and the security checklist on the booth PC
- [x] Q4 close-out (orchestrator): PR #58 CI green (5/5 jobs at 2807ed7); merged into `main` as 571cf0d; 13 Q4 tickets filed as #59–#71 (label `q4`); #46 and #52 closed.
- [x] **Q5 close-out (orchestrator).**
  1. [x] Tests for Q5-08 (M65), Q5-09 (M66) and Q5-10's M52 and M55 added in 5858872 (test-only); each mutant applied by hand fails `cargo test --test voxxa_eval` (26 tests). M57 and M68 deferred with a reason (Q5-10).
  2. [x] The orchestrator re-ran the four mutants (all killed) and all 7 gates plus the CI eval step.
  3. [x] `q5/base` pushed; draft PR #72; CI green on all 5 jobs at b670607 (run 43468103792), including the eval step's first Linux and macOS run.
  4. [x] Gate proven in real CI: d42e6ce set `total.p95_ms` to 1000; run 43469439549 failed on all three OSes at `cargo test` (`replay_of_the_committed_fixtures_passes_the_baseline`: "FAIL total: p95_ms is 8900, baseline needs <= 1000"; the eval step was skipped after the failed test step); reverted in bdf680f.
  5. [x] Merged into `main` as 69f7d27 (PR #72; CI green on all 5 jobs at 65b3405, run 43471805675); 9 open Q5 tickets filed as #73–#81 (label `q5`); Q5 numbers added to #61 (Q4-03) and #62 (Q4-04).

  The user pre-approved the push, PR, CI fixes, gate proof, merge and filing (handoff, "User decisions for Q5").
- [x] **User decision (2026-11-18):** the `jump-be-thou-my-vision` baseline cells set to 1,500 / 6,000 as a one-time exception (option (a)); the two looser `total` cells (blank 7,500, unblank 2,500) accepted. Applied in 85c736c.
- [ ] **Still for the user:** approve or decline the GUI fix in Q6-01 (decision 5); decide whether to drop the two unused `SmartConfig` fields (Q6-10).
- [ ] **Q6 close-out (orchestrator).**
  1. [x] The user's choice applied in its own commit (85c736c) and recorded in ADR-016 (amendment of 2026-11-18); the ponytail lane re-run, reviewed and compared; grafts applied; gate 5 fixed (b7a5382, 734b467); verifiers passed at 97cc21b.
  2. [ ] Pushed `q6/base` and opened draft PR #82 against `main` (2026-11-19). Still to do: get CI green on all jobs (fix forward if needed); merge only when every CI job is green and the exit criteria hold.
  3. [x] Filed the 12 open Q6 tickets as #83–#94 (label `q6`); added the Q6 numbers to #61, #73, #77 and #78. #76 and #62 close when PR #82 merges.

  The user pre-approved the push, PR, CI fixes, merge and filing (handoff, "User decisions for Q6").
- [ ] Increment 1′ (Q4–Q7) … per the 2026-11-11 prompt pack; Q4 merged (571cf0d); Q5 merged (69f7d27); Q6 done and verified on `q6/base` (97cc21b), CI + PR + merge next; Q7 (close Increment 1′) after
  - [x] 2026-11-13 Q5 verifier fixes on `q5/base`: every surviving mutant named by the verifier is now killed by a test (song not compared, false advance lowering recall, EARLY_MS 999 and `<=`→`<`, the total row's false/double sums, unblank not resetting the blank, a failing `voxxad` read as success); a metric with a value but no threshold now fails the gate (ADR-015 D8 amended); voxxa-eval exits 2 when `voxxad` ran in the wrong mode (a replay that ran Whisper); both session logs re-recorded with the final `--full --record` (only the start line's `log` field changed).
  - [x] 2026-11-13 Q5 second verifier pass on `q5/base`: tests now kill the round-2 survivors: M42 (main.rs checking only the total row; a binary-level test breaks only per-fixture thresholds and needs both FAIL lines and no FAIL total), M25/M26 (one metric's comparison reversed), M37 (the table's count cells) and M47/M48 (the audio recipe's 16 kHz mono 16-bit check and header byte rate). Each test was run against its mutant and fails. No product code changed; voxxa_eval tests 23 to 26.
  - [x] 2026-11-13 Q5 verifier round 3 at c10e963: verifier A passed; verifier B failed on six surviving mutants (Q5-08 … Q5-10). The four that mattered were closed with tests in 5858872 (orchestrator, checked against each mutant); M57 and M68 deferred (Q5-10).
- [ ] Founder: record two PD-hymn fixtures; apply to SignPath; open Partner Center; e-mail CCLI; form LLC; start research screener
- [ ] Phases 4–6 per increment

### Blocked
(none)

## Risks
- R8 **Closed (Q2).** Unreviewed May-branch code: Q2 ran QA and security over the merged base; all 12 Q0 must-fix defects and every new crash, data-loss and wrong-slide defect are fixed with tests; the rest are 43 tracked tickets (GitHub issues #2–#44).
- R10 **Closed in code (Q2).** Wrong slide on screen (audit 1–4, plus audit 5, PDF page split, single instance, Q2-44): fixed with dispatcher and conductor tests. Confirmation on real presentation apps moves to R14.
- R14 (new, Q2) Fixes proven only against mocks and synthetic audio. No real mic, keystroke driver, PowerPoint, ProPresenter, FreeShow, OpenLP or OpenSong was used. Mitigation: founder checklist M1–M14 before any pilot leaves shadow mode; promote Q2-20/Q2-22/Q2-47 if M3/M8 fail.
- R15 (new, Q2; updated Q6) Tight line budget. After Q6 (97cc21b):
  - "everything else in src-tauri/src" 2,309/2,500 (191 left; +23 in Q6, mostly the chrono → `time` change in diagnostics);
  - conductor (`aligner/`, without `evidence/`) 1,173/1,200 (27 left; down from 1,188, the replaced next-slide code was deleted);
  - the new conductor evidence row (`aligner/evidence/`) 215/500 (285 left; Q6 user decision 3, a recorded deviation from ADR-010 Decision 3, ADR-016 Decision 8);
  - voxxa-eval 469/600 (131 left); recognizer 432/700 (268 left); `voxxad` 552/600 (48 left; +4 for local-time log names with `time`);
  - GUI 2,951/3,000 (**49 left**; Q6-01's fix needs about 6); presenters 1,149/2,000; importers 1,386/1,600 (214 left; +24 in Q6).

  Mitigation: keep new tests in `src-tauri/tests/` (uncounted; Q6's 32 conductor tests are there: `conductor_evidence` 25, `conductor_scripted` 5, `conductor_unchanged` 2). No cap was raised in Q6. The GUI row is now the tightest: any GUI change (Q6-01, Q8's simple mode) needs code removed first or a budget decision before code, never a silent raise.
- R13 (new, Q1) Fragile CI gates.
  - The GUI fits its budget with 49 lines to spare (2951/3000 after Q2), partly by avoiding a tokei 15 HTML apostrophe miscount.
  - CI installs the latest tokei, the latest stable Rust (new clippy lints under `-D warnings`) and a fresh advisory DB.
  - Mitigation (approved and done 2026-11-11): `ci.yml` pins Rust 1.99.1 and tokei 15.0.0. Bump them deliberately in their own PR.
  - Q6 (2026-11-18): the fresh advisory DB turned gate 5 red with an unchanged `Cargo.lock` (RUSTSEC-2026-0543 chrono, RUSTSEC-2026-0548 zip). Fixed in code (zip 9.0.5; chrono replaced by `time`), with no advisory ignored. Expect this again; a green gate 5 is only true on the day it ran.
- R11 (new, Q0) ADR-007/005 content in the May branch: copyrighted lyrics in `setlist.example.json`, a mislabelled "Public Domain" example, an MIT tag in the Flatpak metainfo. Mitigation: amendments A9/A10 in the Q1 merge.
- R12 (new, Q0) Toolchain-dependent clippy: 8 `double_must_use` errors from `async_trait` on rustc 1.99, and CI uses unpinned stable. Mitigation: fix in Q1. Consider pinning the toolchain.
- R9 Dependency advisories in pdf-extract/lopdf and rusqlite may force crate replacement — Q1 gates on cargo deny; PDF importer may go behind an off-by-default feature. Q2: pdf-extract panics on 4.8 % of mutated PDFs (now caught; stack overflow is not) — cut-list evidence
- R1 Recognition on sung vocals (Whisper WER 21–56% on singing) — mitigated by ADR-009 + eval harness in Inc 1; kill criterion week 14. Q5: the harness (`voxxa-eval`) exists, but its fixtures are spoken synthetic stand-ins, so singing is still unmeasured (Q5-01). Q6: ADR-009's minimum is built (false advances 1 → 0 per the six synthetic fixtures; p95 11.23 → 4.87 s, still over the 2.5 s budget, stream time at file speed); its numbers are tuned on synthetic speech only, and live latency is unmeasured (Q6-03, Q4-02)
- R2 Support conversion ≈ 1% — Setup & Calibration + Store listing as independent lines; decision gate month 12
- R3 CCLI/publisher objection to lyric indexing — no lyrics shipped; written question before launch
- R4 Vendors ship lyric-following natively — ship first on open platforms; contribute upstream
- R5 SmartScreen friction — Store listing + SignPath/Trusted Signing + winget
- R6 Founder on-call load — bounded Sunday window; MSP partners carry their churches
- R7 Windows-first vs Mac churches — unsigned previews; ADR-002 review trigger
