# Project Status

## Project: Voxxa — voice-following slide advancer for churches
**Current Phase:** Increment 2′. **Q10 merged** (PR #169, 4dc17e6; release-id fix PR #180, 86c140e): an operator-initiated updater with a tested comparator, a strict CSP, and draft releases whose Windows signing (SignPath, else Azure) and updater signatures light up when the founder adds the secrets (ADR-020). The rc tag `v0.1.0-rc.1` (run 45268876214) produced a complete **draft pre-release**, unpublished. The founder's new logo (`voxxa_logo.png`) is applied to the app icons, tray, app header, README, site and quick-start (PR #181, aa37899); the founder uploads `docs/brand/social-preview.png` as the repository's social preview. **Next: Q11** (handoff, "From: Q10 → To: Q11"; ask the founder for its decisions first). The private test song is **New Life** (M17).
**Since Q10 (2026-11-24/25):**
- PR #184 (866b51e) counts the blank slide each WorshipTools Presenter song opens on (#183).
- "media guard" is a required check on `main` (#159 closed).
- **V0 (#59, ADR-021 § 1) is built on `fix/59-msvc-release-flags`, not merged** (the comparator's branch after the lanes `v0/10x` and `v0/ponytail`): Windows release builds compile whisper.cpp with `-O2`, and the next founder-approved rc draft must show the new check passing.
- **Next:** the V0 PR and merge (pre-approved, FA-01), then Q11.

**Last Updated:** 2026-11-25 by the V0 comparator (V0 recorded on `fix/59-msvc-release-flags`)
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

- [x] 2026-11-19: **Q7** Close Increment 1′ (Phases 5 and 6), with the ponytail lane. Records: `reviews/2026-11-19-ponytail-q7.md` (both lanes, the ponytail audit, cross-reviews, the comparator's ruling, grafts, disagreements, verifier rounds), `reviews/2026-11-19-qa-report.md`, `reviews/2026-11-19-security-review.md`, `reviews/2026-11-19-q7-tickets.md`, `decisions/{qa,security,devops,sre}/church-autoadvance.md` ("Q7 final"), `docs/releases/voxxad-cli-preview.md`, `docs/INTEGRATIONS.md` (Companion), `docs/runbooks/eval-regression.md`. **Done and verified locally. Ready for CI, PR and merge once the first CI run shows the evidence below.**
  - **Final head:** `q7/base` @ **6386f6b** (22 commits on 46cb6d1, including the merge), plus the state commit "state: Q7 close Increment 1prime". **Both verifiers passed at 6386f6b** (QA report and security review §11). `q7/10x` (e93b63b) and `q7/ponytail` (4d06345) stay local for reference.
  - **History:** `q7/10x` (e93b63b, 8 commits on 46cb6d1) was merged as 008aa50 (the comparator's base: the only lane whose socket test ran and was mutation-proved, with an installer guard test and the required records). Then:
    - grafts from `q7/ponytail`: c9271f0 + eb58458 (CLI tests fail loudly without `--features cli`; corrected, see below), 5807ba3 (stderr filtered to `voxxa-eval:` lines), afeec76 (`fixtures/README.md` determinism rule), 5fa4c76 (why `ubuntu-22.04`);
    - review fixes: 3472540 (one `committed_logs()`), 171ad76 (`Cargo.toml` comment), b0efc3f ("`--shadow` makes no network connection"), 911d56c (full-eval timing comment);
    - records: 3a4303a and the state commit 86b549d;
    - verifier fixes: aa16e90 (code: the socket test) and 6386f6b (records).
  - **Built in Q7:**
    - a replay determinism test on 3 OSes; four fixture logs re-recorded (20e3301; recogniser lines identical, table unchanged);
    - `full-eval.yml` (report only, model SHA-256 pinned);
    - a zero-sockets test (strace inside `unshare -rn` on Linux, ETW on Windows, skipped on macOS);
    - the tools out of the installers (b9692c5, `tests/packaging.rs`);
    - `ci.yml` read-only token;
    - release notes, the Companion guide, the runbook.
  - **Graft corrected, not taken as written:** ponytail's `option_env!("CARGO_BIN_EXE_voxxad")` guard never fires on cargo 1.99.1, which sets the variable even for a bin skipped by `required-features`. It also let a stale `voxxad.exe` pass `replay_determinism` here. The helpers check `cfg!(feature = "cli")` instead. Q7-04 is closed.
  - **Gates on Windows at 911d56c** (the last code commit; later commits are `.10x/` only), all pass:
    - 1 npm, 2 fmt, 3 clippy with and without `--features cli`;
    - 4 `cargo test --features cli`: 207 passed, 1 ignored, 9 min 11 s;
    - 5 cargo-deny, 6 loc budget (no row changed), 7 the `dictat` grep;
    - the CI eval step: exit 0, table unchanged, 112 s.
    - A plain `cargo test` fails on purpose: 197 passed, 10 failed, with the hint.
    - Only binaries match the control-byte grep.
  - **Re-proved at the finish:**
    - zero sockets (ETW): pass; the replay TCP-connect mutant fails with 11 AFD events; pass again after the revert;
    - determinism: pass; a 1 ms change to one logged action fails it;
    - installer: the NSIS script names neither tool, and the `--features cli` control names both.
  - **Verifier fixes (aa16e90, after the round on 86b549d):** gate 4 was flaky on Windows because `replay_opens_no_socket` counted AFD events by pid alone. It now counts only socket-create and connect events inside each child's start-to-exit window, taken from Kernel-Process events in the same ETW session. Unit tests fail if the window or the event filter is removed. `VOXXA_REQUIRE_NETNS=1` in Linux CI fails the test without `unshare -rn`. Re-proved: 4 of 4 full `cargo test --features cli` runs pass (210 passed, 1 ignored), the CI eval step and the table are unchanged, the socket and determinism mutants fail, and the installer exclusion holds. Records: QA report and security review §10.
  - **Before merge (must come from the first CI run):**
    - (a) the Linux and Windows jobs pass `replay_no_sockets` (since aa16e90 a pass proves strace inside `unshare -rn` and ETW respectively);
    - (b) the throwaway socket mutant turns Linux red, and its revert turns it green;
    - (c) full-eval is green or warns on 3 OSes, with only the table, and its times are recorded;
    - the Node 20 check on the older action pins;
    - the run URLs, recorded in both reports.
  - **Verifier rounds:** the round on 86b549d did not pass (gate 4 flaky on Windows; a green CI log did not show which tracer ran); fixed in aa16e90. **At 6386f6b both verifiers passed:** every gate (210 passed, 1 ignored), a TCP and a UDP socket mutant each fail the socket test, the determinism mutant fails at its fixture only, the NSIS script names neither tool, the documented commands work in shadow and replay, and no test is hidden from CI. Non-blocking nits became Q7-13 or were corrected in the DevOps record.
  - **Ponytail audit of Increment 1′:** healthy for a developer preview; two must-fix items were fixed in both lanes (the installer, the stale logs); new tickets Q7-08 (medium, transcripts in the diagnostics report) to Q7-12.
  - **Tickets:** 12 open (1 medium, 11 low: Q7-01..03, Q7-05..13), not filed yet; Q7-04 closed in c9271f0. Q7 closes #47 and #75 (b9692c5). It changed but did not close #48, #49, #54 and #78, and adds notes to #9, #77, #69, #89 and #70 (tickets file, last section).
  - **Release readiness:** Increment 1′ is complete as a **developer preview** of the CLI. It is not a church release (see "Release readiness (2026-11-19, Increment 1′)" below).

- [x] 2026-11-19: **Q8** Simple mode in front of the existing panels (Phase 4: ADR-017), with the ponytail lane. Records:
  - `reviews/2026-11-19-ponytail-q8.md`: both lanes, the cross-reviews, the comparator's ruling, the grafts, the disagreements, the verifier rounds, and which checks ran in the mocked browser and which in the guarded real app;
  - `reviews/2026-11-19-q8-tickets.md`;
  - `adrs/017-simple-mode-in-front-of-the-existing-panels.md`;
  - `decisions/{sde,qa}/church-autoadvance.md` ("Q8 final", with the usability walk-through table) and `decisions/architect/_index.md`.

  **Implemented; not merged. Verifier A failed round 3, and the PR must not be merged until Q8-10 is fixed and verified** (below).
  - **Final head:** `q8/base` @ **ee431ce** (24 commits on f8500e3, including the merge), plus the state commit "state(implementation): Q8 Simple mode". `q8/10x` (99fe481) and `q8/ponytail` (bdc3348) stay local for reference.
  - **History:**
    - the 10x lane (42abe5c..99fe481, ADR first), merged as b4eacca, the comparator's base;
    - review fixes 3ab4b6c (one toggle at a time), 70e3519 (the presenter's reason in row 5), 68df936 (Space on Simple mode's buttons) and 21b5b77 (the device step's stale guard);
    - ponytail grafts fa87345 (reconnect at start), 91c3fc2 (text written only on change), 9653f48 (green only on a slide change) and 42f7121 (the keystroke hint);
    - round 1 fixes ef84473, 3f75c09 and b191945; round 2 fixes bb46628, 9ef8937 and 808f14b;
    - records in 9e96282, 57fafcb and ee431ce.
  - **Built:**
    - Simple mode in `src/simple/`: a mode switch (Off, Songs, and Songs + Scripture disabled), a status light with one sentence (10 rows, a pure function tested row by row), the song and slide (titles only, ADR-011), Call for help, and Advanced.
    - Call for help shows a "next update" message and calls no command (Q7-08, #102).
    - Every old panel sits unchanged behind Advanced, one click away. The diagnostic modal moved out of `#app`, and `main.js` gained one line.
    - The welcome overlay was replaced by a first-run flow: microphone, presenter auto-detect, model (Tiny), and a 20-second test that is green only on a slide change the presenter took. The flow can be skipped, re-opened from Advanced, and is remembered in `voxxa.firstLaunchDone`.
    - The saved microphone and presenter are re-applied at start.
    - A pre-existing defect was fixed: the `hidden` attribute lost to `display: flex`, so the diagnostic modal covered the window at every start.
    - No change under `src-tauri/`.
  - **Gates on Windows** (rustc 1.99.1) at ee431ce, all pass (finisher; verifiers A and B):
    - frontend, fmt, clippy with and without `--features cli`;
    - `cargo test --features cli`: 210 passed, 1 ignored;
    - cargo-deny, the loc budget and the `dictat` grep;
    - the CI eval step: 21/21, "OK: no metric is worse";
    - `npm run test:gui`: 6/6; `npm run test:gui:e2e`: 106/106.
    - Only binaries match the control-byte grep.
  - **Budgets:** new row Simple mode (`src/simple/*`) **352 / 600** (user decision, recorded deviation, ADR-017 § 5). GUI 2,951 → **2,780** / 3,000. No other row changed, no cap raised, no npm runtime dependency.
  - **Testing, two ways:**
    - (a) Mocked browser (`tests/gui/e2e.mjs` with `tests/gui/mock-tauri.js`, headless Edge, in CI on Windows): every status row with its computed colour, Off and Songs, the 20-second test, and every Advanced panel.
    - (b) The guarded real app (a scratch identifier per agent, CDP, a ProPresenter mock): 27/27, plus probes. It never listened and never used the keystroke presenter, the stage window or the HTTP API.
  - **Usability walk-through** (research plan § 2 and O7): passes on the mocked GUI by mouse; U6 fails by keyboard (Q8-10). On the real app, every step that is safe without a microphone passes. The founder's own run is open (Q8-01; founder checklist below; not a merge gate).
  - **Verifier rounds:**
    - Round 1 (9e96282) did not pass: row 9 after a reload, panels leaking past the check, a hidden Skip.
    - Round 2 (57fafcb) did not pass: a slow start in the 20-second test, the light's colour unchecked, controls off screen or transparent.
    - Each finding was fixed with a check that fails without it.
    - **Round 3 at ee431ce: verifier B passed** (scope clean, 26 of 28 mutants killed, two minor survivors: Q8-11, Q8-12). **Verifier A failed:** with the focus on the page body, the set-up guide lets `main.js`'s shortcut keys through. Space starts listening behind the guide before a microphone is chosen, and during the 20-second test Space stops listening (a false "couldn't hear" verdict) and `s` opens the stage window (`src/simple/firstrun.js:31`; Q8-10).
  - **Tickets:** 16 open (2 medium, 14 low), not filed yet (label `q8`). Q8 closes no earlier issue outright: #7 is fixed in part (fa87345), and comments go to #7, #44, #34, #9, #102 and #37 (tickets file, last sections).

- [x] 2026-11-19: **Q9** Call for help, runbooks, quick-start (Phase 4: ADR-018), with the ponytail lane. Records:
  - `reviews/2026-11-19-ponytail-q9.md`: both lanes, the cross-reviews, the comparator's ruling, the grafts, the disagreements, the verifier rounds, and which checks ran in the mocked browser, in Rust tests and in the guarded real app;
  - `reviews/2026-11-19-q9-tickets.md`;
  - `adrs/018-call-for-help-bundle-runbooks-quick-start.md` (ADR-017 §§ 1 and 2 amended);
  - `decisions/{sde,qa,security}/church-autoadvance.md` ("Q9 final") and `decisions/architect/_index.md`.

  **Done; not merged yet.** Both exit criteria are met: the bundle is produced, and the links resolve.
  - **Final head:** `q9/base` @ **89f4ccb** (23 commits on 7797556, including the merge), plus the state commit. `q9/10x` (dadcdbb) and `q9/ponytail` (8f87206) stay local for reference.
  - **History:**
    - the 10x lane (f105973..dadcdbb; ADR-018 first, and #102 fixed in the first code commit, 2c28c15, with its test), merged as cd28ab5, the comparator's base;
    - ponytail grafts 064c4c8 (OpenSong's API key in the runbook), 42ca27f (a presentation app on another computer), 82d24e9 (`runbook_url`, exact URL tested) and caa67b1 (no overflow in the words window);
    - the 10x review's fix 9917115 (the folder opens first, whatever happens to the e-mail draft); records 8ade143;
    - round 1 fixes 3e3efd7 (redaction), 5ba1c00 and 708fa62 (the real preview and Save steps tested), 84ca89c (quick-start on one page); records 64fc4a0 and c0fae68;
    - round 2 fix 82ff710 (no releases page in the quick-start); record 89f4ccb.
  - **Built:**
    - #102: the listening loop logs `Heard N words`, never the text, so no recognised text reaches the diagnostics buffer or report.
    - Call for help (`src-tauri/src/help/mod.rs`, `src/simple/help.js`): a review screen lists every file (the diagnostic report, the redacted `voxxa.*` settings, and the last 2 minutes of words from the newest `voxxad` log, behind a checkbox that is on by default) and writes one zip to the Desktop only on Save. Unchecked, the words file is absent. Then the zip's folder opens, and an e-mail draft with no recipient names the file.
    - Four new commands, no existing command changed, no capability change, no new crate, no network call. `VOXXA_HELP_DIR` sandboxes it for QA and opens nothing.
    - "What to do" links on red rows 1, 5 and 6 and grey row 3 open the matching runbook on GitHub.
    - `docs/quick-start.md` (one printed page) and `docs/runbooks/{no-audio-device,presentation-app-not-reachable,model-missing}.md`.
  - **Gates on Windows** (rustc 1.99.1) at 89f4ccb, all pass (finisher; verifiers A and B): frontend, fmt, clippy with and without `--features cli`; `cargo test --features cli` 221 passed, 1 ignored; cargo-deny; the loc budget; the `dictat` grep; the CI eval step 21/21, "OK: no metric is worse"; `test:gui` 7/7; e2e 160/160. Only binaries match the control-byte grep.
  - **Budgets:** everything else in `src-tauri/src` 2,309 → **2,494 / 2,500 (6 left)**; Simple mode 352 → **436 / 600**; GUI 2,780 / 3,000 unchanged. No cap raised, no row added (user decision 3).
  - **Testing:** (a) mocked browser: every Call for help path, including Save and the e-mail and folder step, and every runbook link; (b) Rust tests with temp folders, including the real preview and Save commands; (c) the guarded real app (scratch identifiers, `VOXXA_HELP_DIR` a scratch folder): the review screen and the bundle written by the real backend. The real app never listened, never pressed Save, and never opened a mail program or Explorer; nothing was written to the real Desktop.
  - **Verifier rounds:** round 1 (8ade143) and round 2 (c0fae68) did not pass; each finding was fixed with a test that fails without it. **Round 3 at 89f4ccb: both passed** (verifier B: 29 of 30 mutants killed; the survivor is Q9-11).
  - **Tickets:** 16 open (1 medium, 15 low), not filed yet (label `q9`). Q9 fixes **#102** (close it at merge); comments go to #53, #49, #99, #116, #109, #34, #8 and #121 (tickets file, last sections).

- [x] 2026-11-20: **Q9W** WorshipTools Presenter: found by process, keys guarded, no step across songs (Phase 2 check + Phase 4: ADR-019 §§ 1-4 and 10), with the ponytail lane. Records:
  - `reviews/2026-11-20-ponytail-q9w.md`: both lanes, the cross-reviews, the comparator's ruling, the grafts, the disagreements, the verifier rounds, and which checks ran in the mocked browser, in Rust tests and in the guarded real app;
  - `reviews/2026-11-20-q9w-tickets.md`;
  - ADR-019's four Q9W amendments (Phase 2 check, base merge, verifier fixes, verifier round 2);
  - `decisions/{sde,qa,security}/church-autoadvance.md` ("Q9W final") and `decisions/architect/_index.md`.

  **Done; not merged yet.** Every exit criterion is met.
  - **Final head:** `q9w/base` @ **b0de71b** (23 commits on b154a55, including the merge), plus the state commit. `q9w/10x` (84acaeb) and `q9w/ponytail` (ee4d431) stay local for reference.
  - **History:**
    - the 10x lane (fdd5f85..84acaeb; the ADR-019 amendment and the budget row before any code), merged as aaca033, the comparator's base;
    - the 10x review's must-fix 1 (705605a: the stage and the song strip follow the song-change ask) and nit 2 (71ab2b5: the operator's asks as info toasts);
    - ponytail grafts 7e5473d (the shared blank rule pinned for ProPresenter WS), 71ab2b5 (no red toast after an ask) and 38f043d (the runbook's red own-window row and the missed-ask symptom, corrected: click the slide being sung, not the song chip); ADR-019 amendment 492407a; records 1927856;
    - round 1 fixes e47b2c0, 8022bfd, 63b03d6, 97279a8, 4fb1346; records 006372c;
    - round 2 fixes d538a4b, 7a8a457; records b0de71b.
  - **Built:**
    - WorshipTools Presenter is found by its process on Windows (read-only kernel32 FFI, no crate) and returned by discovery as keystroke, profile `worship_tools_presenter`, source "process".
    - With that profile, a key is sent only while Presenter's window is in front; otherwise amber "Bring WorshipTools Presenter to the front.".
    - A song change presses nothing, records the new song's first slide and asks amber 'Put "<song>" live in Presenter.'; the stage and the song strip follow it. Steps inside a song and the raw Next/Prev are unchanged.
    - The key map is Right, Left and no blank; `can_blank` comes from the key map (false); the conductor's blank is skipped (no red error at a prayer) for every driver that cannot blank, ProPresenter 7/6 WebSocket included; the operator's Blank is "not supported"; the light says Voxxa can't clear the screen.
    - A saved non-universal keystroke form is reconnected at start; first-run step 2 names Presenter in plain words and connects it when it is the only find; Advanced → Auto-detect lists it.
    - Runbook WorshipTools section (administrator note included) and `docs/INTEGRATIONS.md` Tier 3, MIDI deferred.
  - **Gates on Windows** (rustc 1.99.1) at b0de71b, all pass (finisher; verifiers A and B): frontend, fmt, clippy with and without `--features cli`; `cargo test --features cli` 237 passed, 1 ignored; cargo-deny; the loc budget; the `dictat` grep; the CI eval step 21/21, "OK: no metric is worse"; `test:gui` 9/9; e2e 188/188. The control-byte grep matches nothing.
  - **Budgets:** new row discovery (`src/discovery/`) **223 / 250** (D5, ADR-019 § 10); everything else in `src-tauri/src` cap lowered 2500 → 2317 when discovery moved out, now **2315 / 2317 (2 left)**; presenters 1149 → 1374 / 2000 (`keystroke.rs` 396 / 400); GUI 2784 / 3000; Simple mode 436 → 450 / 600. No cap raised, no new crate.
  - **Testing:** (a) mocked browser: step 2, both amber sentences, info toasts, the can't-clear row, the restart reconnect, Auto-detect; (b) Rust tests with fake presenters, fake process lists and a recording sender (no test can reach enigo); (c) the guarded real app (`qa9wx`, `qa9wp`, `qa9wf`, `qa9wa`, `qa9wb`): **the real first-run step 2 named the running WorshipTools Presenter every time**, the real discovery returned the process hit, and the profile survived a real restart. The real app never listened and never sent a key; Presenter was never touched.
  - **Verifier rounds:** round 1 (1927856) and round 2 (006372c) did not pass (surviving mutants at the guard's call site, the discovery merge and wiring, the GUI tell, Advanced → Auto-detect; one docs overstatement); each was fixed with a test that fails without it. **Round 3 at b0de71b: both passed** (verifier B: 35 Rust and 18 GUI mutants, all killed).
  - **Tickets:** 12 open (2 medium, 10 low), not filed yet (label `q9w`), including "Founder: confirm the F10 unblank key (T3)" (Q9W-02) and the halfway multi-key step (Q9W-01). Q9W fixes **#117** (close it at merge); comments go to #6, #13, #42, #44, #48, #33 and #39 (tickets file, last sections).
  - **Founder checklist:** M15 (keys with WorshipTools Presenter) and M16 (song change, the missed-ask symptom, Left on a first slide) are in `handoff.md`, "Still open from earlier steps".

- [x] 2026-11-20/21: **Q9V** Private "See a Victory" fixture and the media guard (Phase 4 + 5: ADR-019 §§ 7-9, ADR-015), with the ponytail lane. Records:
  - `reviews/2026-11-20-ponytail-q9v.md`: both lanes, the cross-reviews, the comparator's ruling, the grafts, the disagreements, the verifier rounds, the planted-file evidence and the `--full` evidence;
  - `reviews/2026-11-20-q9v-tickets.md`;
  - ADR-019 "Amendment (Q9V)";
  - `decisions/{sde,qa,security}/church-autoadvance.md` ("Q9V final").

  **Done; not merged yet.** Every exit criterion is met.
  - **Final head:** `q9v/base` @ **be4819b** (17 commits on 84a3966, including the merge), plus the state commit "state(implementation): Q9V media guard, numbers-only eval, private fixture steps". `q9v/10x` (c5ffb23) and `q9v/ponytail` (ce65275) stay local for reference.
  - **History:**
    - the 10x lane (b73a9a9..c5ffb23; ADR-019 amendment included), merged as c847eab, the comparator's base;
    - ponytail grafts 2b365b8 (runbook and release note: `--full` discards Whisper's log), ddb96b8 (README step 7: what "voxxad failed with exit code: 1" means), 4b53c67 (the `media` job on `ubuntu-latest`), e17fe19 (two look-alike paths in the guard's fine list);
    - the 10x review's fix 1931c59 (the BOM setlist error as printed); 325a467 (README step 6: `FAIL total` is normal for one fixture); records a4bd409 (M17);
    - verifier round 1 fixes 3980585, efa5b7d, 6afd02f (tests only); records be4819b.
  - **Built:**
    - **The repo refuses media.** `src-tauri/tests/media_guard.rs` (5 tests) and the ci.yml `media` job (its own job, bash, `ubuntu-latest`) read `git ls-files -z`, lower-case each path, and refuse any tracked `.mp4 .mov .m4v .mkv .webm .wav .flac .mp3 .m4a` and anything under `fixtures/private/` (root or nested), naming the path. Exactly the 15 public-domain WAVs are allowed, by exact path. The suite runs the CI step's own bash on the real tree and on 17 planted paths in a temporary repo.
    - **`voxxa-eval --full` prints numbers only.** `voxxad`'s stderr goes to null; a failure names its exit status; error lines, bad lines and bad setlists are named by number (or line and column). Replay (CI) still shows `voxxad`'s text. `tests/eval_numbers_only.rs` proves it with a fake `voxxad` and placeholder words (6 checks).
    - **Founder docs.** `fixtures/README.md` "Making the See a Victory fixture" (9 steps: ffmpeg, the BOM-free setlist snippet, `fixtures/tools/tap-truth.ps1` or truth by hand, a setup check, the eval, the `LICENSED_RUNS.md` row, the pre-commit guard). No `voxxad` subcommand.
  - **Gates on Windows** (rustc 1.99.1) at 6afd02f (finisher) and be4819b (verifiers A and B), all pass: frontend; fmt; clippy with and without `--features cli`; `cargo test --features cli` 242 passed, 1 ignored, plus `eval_numbers_only` 6 checks; cargo-deny; the loc budget; the dictat grep; the CI eval step 21/21, "OK: no metric is worse" (unchanged); the ci.yml media step as written (287 tracked paths, exit 0); `test:gui` 9/9; e2e 188/188. The control-byte grep matches nothing.
  - **Budgets:** voxxa-eval 469 → **488 / 600**; voxxad 552 / 600, everything else 2315 / 2317 and discovery 223 / 250 unchanged. No cap raised, no row or crate added.
  - **Exit demo (6afd02f):** the guard and the CI step pass on the real tree, and both named all three empty dummies (`Músicas/canción.wav`, `clips/Song A.MP4`, `fixtures/private/see-a-victory/setlist.json`) staged in a throwaway clone, which was then deleted. `voxxa-eval --full` on `prayer-then-reprise` with real Whisper Tiny printed the table and a `FAIL total` line only, with 0 bytes on stderr. `git log --all -- fixtures/private` is empty.
  - **Verifier rounds:** round 1 (a4bd409) did not pass (R2: `-z` parsing unproved; C1-C6: the CI step's bash untested; E4c, E4d: replay's error text unpinned); each was fixed with a test that fails under its mutant. **Round 2 at be4819b: both passed** (verifier B: every named mutant killed; two minor survivors, Q9V-03).
  - **Tickets:** 10 open (1 medium, 9 low), not filed yet (label `q9v`). Q9V fixes **#105** (Q7-11; close it at merge); comments go to #74, #79 and #107 (tickets file, last sections).
  - **Founder checklist:** M17 (the private eval; R0's record calls it "M16") in `handoff.md`, "Still open from earlier steps".

- [x] 2026-11-23: **Q10** Windows signing, winget, updater hardening (Phase 4 + 6, per ADR-008; ADR-020), with the ponytail lane. Records:
  - `reviews/2026-11-23-ponytail-q10.md`: both lanes, the reviews of each, the comparator's ruling, the grafts, the disagreements, the verifier rounds, the CSP evidence and the workflow trace for each secret combination;
  - `reviews/2026-11-23-q10-tickets.md`;
  - ADR-020 (new, Accepted) and an ADR-008 amendment; `docs/RELEASING.md` (new release runbook);
  - `decisions/{sde,qa,security,devops}/church-autoadvance.md` ("Q10 final") and `decisions/architect/_index.md`.

  **Done; not merged yet.** Every exit criterion that can be proved on this PC is met; the tag build itself is the orchestrator's rc tag run.
  - **Final head:** `q10/base` @ **f424c74** (25 commits on 68adf2d, including the merge), plus the state commit "state(implementation): Q10 signing paths, draft releases, operator-initiated updater, strict CSP". `q10/10x` (5eda044) and `q10/ponytail` (810c203) stay local for reference.
  - **History:**
    - the 10x lane (2738c00..5eda044; ADR-020 before any code), merged as b5d8df0, the comparator's base;
    - grafts 19b6991 (every action in `build.yml` and `winget.yml` pinned by SHA), 8319140 (the `latest.json` job never gets the key), d97bc54 (`latest.json` needs all four platforms), 0571ea6 (the ponytail lane's two findings as tickets);
    - the 10x review's fixes e134b6d (Azure only with all six values), 7df6874 ("no update published yet" is its own sentence, never "up to date"), 615a681, bebe660 (an rc installs as X.Y.Z); d7ef67d (ADR-020); records 7db02a9;
    - verifier round 1 fixes 94d5ac5 + 1244468 (macOS gets each `APPLE_*` only when set), 833da3c (the listening rule and "never at startup" tested where enforced), 6b2ff4e (`default.json` is the only capability); records f424c74.
  - **Built:**
    - **Updater, option B:** **Check for updates** in Advanced → Settings only; never at startup, on a timer or in Simple mode; refused while listening ("Stop listening first. Voxxa never checks for updates while it is listening."), before any request and again after the download. Voxxa's own `check_for_update` / `install_update`; no webview updater permission. A pure comparator (semver precedence, strictly greater; equal, lower, build-metadata-only and malformed refused). Install asks first ("Voxxa will close, install the update and start again.") and installs only the confirmed version. Endpoint `releases/latest`, so drafts and pre-releases are never offered.
    - **Strict CSP** (`default-src 'self'`, no `unsafe-eval`, no `unsafe-inline`; the three inline styles removed); a Vite-only `devCsp`.
    - **Release pipeline:** every tag build is a **draft** (pre-release for a `-` tag); the release job checks the tag, the version, the commit and `main`, and refuses a published release. Windows: NSIS per-user (`currentUser`) + MSI, WebView2 `embedBootstrapper`; signing SignPath (all four values) → else Azure Trusted Signing (all six) → else unsigned with a warning; Authenticode checked; updater signatures over the final bytes; `latest.json` written once when the key exists. macOS on `macos-latest` and `macos-15-intel` (replaces `macos-13`); the dead `*.nsis` glob gone; the portable ggml build (#142) and Rust 1.99.1 in every release job; every action pinned by SHA. winget template filled in the tag build as an artifact; `winget.yml` submits a published full release only when `WINGET_TOKEN` exists.
    - **Privacy text:** network only for a model download, a Planning Center import, a Hymnary.org lookup or an update check, all operator-started (README, quick-start § 5, release body, signing policy, the privacy card).
  - **Gates on Windows** (rustc 1.99.1) at 6b2ff4e (finisher) and f424c74 (verifiers A and B), all pass: frontend; fmt; clippy with and without `--features cli`; `cargo test --features cli` **266 passed, 0 failed, 1 ignored**; cargo-deny; the loc budget; the `dictat` grep; the CI eval step 21/21, "OK: no metric is worse" (unchanged); the media guard 5/5; `test:gui` 18/18; e2e 209/209 with zero CSP violations; `scripts/release` 11/11; actionlint 1.7.7 and PyYAML on every workflow. The control-byte grep matches nothing.
  - **Budgets:** new row **updater (`src/updater/`) 98 / 150** (ADR-020 § 5, the D5 precedent); everything else in `src-tauri/src` 2315 → **2316 / 2317 (1 left)**; GUI 2784 → **2859 / 3000**; Simple mode 450 / 600. No cap raised.
  - **Testing:** (a) mocked browser under the app's own CSP header: every Updates state, zero violations, a planted inline style refused; (b) the guarded real app (`qa10x`, `qa10p`, `qa10f` twice, `qa10a`): zero CSP violations in the main and stage windows, Simple mode, Call for help and Advanced; **Check for updates seen, never clicked**; outbound 0. No installer installed, no key or secret handled.
  - **Verifier rounds:** round 1 (7db02a9) asked for four fixes; each got a test that fails without it (eight mutations, all killed). **Round 2 at f424c74: both passed** (verifier B: 27 of 30 mutants killed; the three survivors are Q10-08).
  - **Tickets:** 10 open (2 medium, 8 low), not filed yet (label `q10`), including Q10-01 (`loadSettings` ReferenceError) and Q10-02 (first run advanced by itself once), both found by the ponytail lane. Q10 fixes **#8, #10, #142, #148 and #160** (close them at merge); comments go to #159, #59, #100 and #134 (tickets file, last sections).

- [x] 2026-11-24: **PR #184 merged into `main` as 866b51e** (fixes #183, the WorshipTools leading blank). Every song in WorshipTools Presenter opens on a blank slide, so voxxa ran one slide behind. `presenters::step_from` now counts the blank: one Right shows a song's first lyrics, and the step into the next setlist song passes its blank (ADR-019 amendment).

- [x] 2026-11-25: **V0** Windows release builds compile whisper.cpp with `-O2` (#59; ADR-021, § 1 Accepted by the founder, FA-01). Two lanes (`v0/10x` @ bbd2f5d, `v0/ponytail` @ 9001cfa, both off 866b51e); the comparator built `fix/59-msvc-release-flags` on the ponytail lane and grafted parts of the 10x lane. Records: `reviews/2026-11-25-ponytail-v0.md` (the ponytail lane's record and the adjudication), `reviews/2026-11-25-v0-tickets.md` (the PR body's cause section and 3 low candidates), `decisions/{devops,qa}/church-autoadvance.md` ("V0").
  - **Cause:** on MSVC the `cmake` crate 0.1.58 defines whisper.cpp's `CMAKE_{C,CXX}_FLAGS_RELEASE` without any `/O` flag, and whisper-rs-sys always builds `--config Release`, so every Windows release build ran Whisper unoptimised. Debug builds are optimised: the override lands on the RelWithDebInfo line, which the Visual Studio generator does not use.
  - **Built:** `build.yml`'s Windows job sets `CMAKE_C_FLAGS_RELEASE` and `CMAKE_CXX_FLAGS_RELEASE` = `-MD -O2 -Ob2 -DNDEBUG`; its cache key is `release-portable-ggml-o2`; "Check that whisper.cpp was built with -O2" (`scripts/ci/check-msvc-release-flags.sh`) runs right after the installer build, and its test runs in `ci.yml`'s `budgets` job; `release_config.rs` pins all three. `ci.yml` and `full-eval.yml` get no env step: they build debug only. Local builds: `scripts/dev/msvc-release-flags.ps1` (`docs/RELEASING.md` § 7, `fixtures/README.md`).
  - **Gates on Windows:** `release_config` 12 passed (13 `build.yml` mutants killed, a no-op stays green); the flag-check test 6/6 (7 checker mutants killed); `cargo fmt --check` clean. Not run: `loc_budget.py` (tokei not installed; no counted file changed), actionlint and PyYAML (not installed).
  - **Booth number:** 130 s of compute for Tiny on the private song P1 (461 s before), measured by the orchestrator with the same two values set by hand; the plan asks ≤ 150 s. The founder's own check with the script is Pending (below).

### Release readiness (2026-11-23, after Q10)
- **Ready:** a tag on `main` builds every installer into a draft release that only the founder can publish; it works today with no secret and signs per secret as the founder adds them. The updater can never surprise a service.
- **Not proven yet:** the release jobs on GitHub's runners (the orchestrator's `v0.1.0-rc.1` run), and every signing path until its secrets exist.
- **Before the founder publishes v0.1.0:** fix #59 (the Windows installers carry an unoptimised whisper.cpp: about 6.5 s per 1 s hop; V0 fixes it in `build.yml`, so a new rc built after the V0 merge is needed, and its Windows job must show the check passing); have at least SignPath or Azure working (otherwise SmartScreen warns); confirm or replace the updater key pair and add it as a secret (every installed copy trusts only the pubkey it was built with, so a key changed after the first published release strands those copies, and a release published without the key carries no update for anyone); then update README, the home page and the quick-start (#134). The Increment 1′ "not met" list below still holds (p95 latency, synthetic fixtures, no real booth run).

### Release readiness (2026-11-19, Increment 1′ after Q7)
- **What Increment 1′ delivers:** `voxxad`, a headless listening loop with a replayable JSON-lines session log (ADR-013); Whisper behind a recogniser trait with a 1 s hop (ADR-014); `voxxa-eval` with six fixtures and a CI gate (ADR-015); the evidence tracker in the conductor (ADR-016); and in Q7 a determinism test, a zero-sockets test, a report-only full eval on three OSes, release notes, a Companion guide and the runbook "eval shows regression".
- **Ready to merge** once the first CI run shows: the Linux and Windows jobs pass `replay_no_sockets`; a throwaway mutant turns Linux red and its revert green; full-eval is green or warns on three OSes with only the table, its times recorded and the Node 20 check done. Then one manual full-eval run on `main`.
- **Ready as a developer preview:** build from source with `--features cli` (`docs/releases/voxxad-cli-preview.md`). No tag, no GitHub release, no signed build (user decision). The church installers carry the GUI only, proved with the NSIS script and guarded by `tests/packaging.rs`.
- **Met:** 0 false and 0 double advances on all six fixtures (21/21 hits, precision and recall 1.00); replay byte-identical and tested on three OSes; replay opens no socket (Linux and Windows in CI); shadow makes no network connection (Windows, locally); no Critical or High security finding.
- **Not met, so not a church release:**
  - p95 latency 4.87 s against a 2.5 s budget (#84), and it is stream time at file speed, not live (#60);
  - the fixtures are synthetic speech, not singing (#73 closes as "local eval only": real sung audio is evaluated locally on LHMEZPC01, ADR-019 § 9);
  - no real microphone, booth or presentation app has been used (#51; founder checklist M1–M14 and the security checklist);
  - privacy before any pilot leaves shadow mode: the diagnostics report carries transcripts (Q7-08), session logs sit in the roaming profile (#49), and `voxxad` has no keystroke focus guard (#48).
- **Not yet run anywhere:** the Linux strace path, ETW on a hosted runner, the full-eval workflow, and MSI, deb, AppImage and `.app` bundles.

### Release readiness (2026-11-11, after Q2)
- **Code:** Q2's exit criteria are met. No open crash, data-loss, freeze or wrong-slide defect from the audit or Q2; no open Critical or High security finding.
- **Not yet ready for a live service.** The fixes are proven against mocks and synthetic audio only. Before a pilot leaves shadow mode, the founder must pass checklist M1–M14 in the QA report (real mic, unplug, Keystroke driver with PowerPoint/ProPresenter, M15 Keystroke with WorshipTools Presenter and M16 song change (handoff, R0), FreeShow/OpenLP/OpenSong, displays, Stream Deck) and the security checklist.
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
- [ ] Founder: run the usability script from the research plan on the founder's machine (Q8-01; user decision 3, not a merge gate). Use a fresh profile (clear `%LOCALAPPDATA%\com.voxxa.app`, keep `%APPDATA%\voxxa\models\ggml-tiny.bin`). Run U1-U13 from `decisions/qa/church-autoadvance.md` ("Q8 final") with a real microphone and presentation app. Look through every old panel once (the `[hidden]` fix changed what shows), and confirm amber for `blank_hold`. Add Call for help and a red light's "What to do" to the run (Q9-07): press Save once with the installed mail program and once on a PC without one (Q9-12), then delete the zip.
- [ ] **Founder (Q9W, 2026-11-20): checklist M15 and M16 on LHMEZPC01** (full text in `handoff.md`, "Still open from earlier steps"). M15: with Presenter open, step 2 names it; keys move it only while it is in front; nothing is sent and the light is amber when it is not; Presenter is not run as administrator. M16: at a song change Voxxa presses nothing and asks to put the song live; if the light goes green but Presenter shows the wrong item, put the song live and click the slide being sung; check what Left does on a song's first slide. Not a merge gate.
- [ ] **Founder (R0, 2026-11-20):** confirm which key brings the lyric back after F10 in Presenter (T3; then `can_blank` can be switched on; ticket Q9W-02).
- [ ] **Founder (Q9V; updated 2026-11-23): checklist M17, the private-fixture eval, on LHMEZPC01** (every command in `handoff.md`, "Still open from earlier steps"). The song is now **New Life** (the church's own song, Lighthouse Worship; ADR-019 amendment 2026-11-23). Its `audio.wav` and `setlist.json` are made in `C:\dev\voxxa-private` (local only). Left: tap `truth.jsonl` with `fixtures/tools/tap-truth.ps1`, run `voxxa-eval --full fixtures/private/new-life`, and fill row 1 of `fixtures/LICENSED_RUNS.md` with the summary numbers only (the church's CCLI number 2919521 is recorded). A report, not a merge gate.
- [x] **Q10 close-out (orchestrator; pre-approved by the founder on 2026-11-23: push, PR, CI fixes, merge, one rc tag that creates a draft release, tickets), done 2026-11-23.** PR #169 merged as 4dc17e6; tickets #170-#179; #8, #10, #142, #148 and #160 closed; comments on #159, #59, #100 and #134. The rc tag run found one release bug, fixed forward in PR #180 (86c140e): the draft job got an empty release id, so tauri-action skipped every macOS and Linux upload. The draft and the tag were deleted and `v0.1.0-rc.1` was re-cut on 86c140e (same tag, RELEASING.md § 5). Run 45268876214 (Apple Silicon re-run once: no GitHub arm64 runner free) gives a **draft pre-release** with every expected asset and no `.sig`, `.asc` or `latest.json`; details in `decisions/devops/church-autoadvance.md` ("Q10 tag run"). Not published. The items below are the original plan. Full list: `handoff.md`, "From: Q10 → To: Q11", and `decisions/devops/church-autoadvance.md` ("Q10 final").
  1. [ ] Push `q10/base` (with the `gh` credential helper and the `workflow` scope: it edits `build.yml`, `ci.yml` and adds `winget.yml`) and open a PR against `main`. Every CI job must be green; the "Release script tests" step and the new Rust tests run on Linux and macOS for the first time.
  2. [ ] Merge.
  3. [ ] Create the label `q10`, file the 10 tickets (`reviews/2026-11-23-q10-tickets.md`), close #8, #10, #142, #148 and #160 with the commits named there, and add the comments to #159, #59, #100 and #134.
  4. [ ] Tag **`v0.1.0-rc.1`** on `main`'s merge commit, once. It must produce a **DRAFT** pre-release. Check the log lines, the asset list (`Voxxa_0.1.0_x64-setup.exe`, `Voxxa_0.1.0_x64_en-US.msi`, two `.dmg`, an `.AppImage`, a `.deb`; no `.sig`, `.asc` or `latest.json`), the updater-manifest skip line and the `voxxa-winget-manifest` artifact, and record the run ID in the DevOps record. **Do not publish the draft.** A failure is fixed forward; a second rc tag needs the founder.
  5. [ ] Then start Q11 (handoff, "From: Q10 → To: Q11").
- [ ] **Founder (Q10, 2026-11-23): release secrets and variables** (GitHub → Settings → Secrets and variables → Actions; all optional, each lights up one step; `docs/RELEASING.md` § 1):
  - **SignPath Foundation (preferred), all four or none:** secrets `SIGNPATH_API_TOKEN` and `SIGNPATH_ORGANIZATION_ID`; variables `SIGNPATH_PROJECT_SLUG` and `SIGNPATH_SIGNING_POLICY_SLUG`. In SignPath, set `packaging/signpath/artifact-configuration.xml` as the project's default artifact configuration and use a signing policy with manual approval; approve each request within an hour of the tag.
  - **Azure Trusted Signing (fallback, used only without a full SignPath setup), all six or none:** secrets `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`, `AZURE_TENANT_ID`, `AZURE_CODE_SIGNING_ENDPOINT`, `AZURE_CODE_SIGNING_ACCOUNT`, `AZURE_CODE_SIGNING_CERTIFICATE_PROFILE`.
  - **Updater key:** secrets `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
  - **winget:** secret `WINGET_TOKEN` (a classic personal access token with the `public_repo` scope). Confirm the identity `nategroks.Voxxa` / publisher `nategroks` before the first submission.
  - Unchanged and optional: the `APPLE_*` secrets in `SIGNING.md`; `GPG_PRIVATE_KEY`, `GPG_PASSPHRASE`.
- [ ] **Founder (Q10): confirm or regenerate the updater key pair.** Confirm you hold the private key and its password for the pubkey in `tauri.conf.json` (minisign key ID **84F2CD98C608E9C9**). If not, generate a new pair yourself, offline (`npx tauri signer generate -w voxxa-updater.key`), and replace the pubkey in a PR. Do it before the first published release: installed copies trust only the pubkey they were built with. Keep two offline copies (`docs/RELEASING.md` § 6). Agents never generate, read or handle the key.
- [ ] **Founder (Q10): publish the draft release when ready** (`docs/RELEASING.md` §§ 3-4). The rc draft is for checking only and never goes on a booth PC. Before publishing a full v0.1.0: #59 fixed, a signing path working, the updater key in place; afterwards update README, the home page and the quick-start (#134). `winget.yml` then submits it if `WINGET_TOKEN` exists.
- [ ] **Founder (2026-11-23): upload `docs/brand/social-preview.png`** in GitHub → Settings → General → Social preview.
- [x] **Founder (Q9V-02, #159): make "media guard" a required status check on `main`** (branch protection; only the founder can change it). Done 2026-11-24 through the ruleset `main` (id 27988429); #159 closed. Merges to `main` now need "media guard" green.
- [ ] **V0 proof (orchestrator, at the next founder-approved rc):** the draft's Windows job must pass "Check that whisper.cpp was built with -O2" and print two flag lines with `-O2`. Record the run ID here, in `handoff.md` and in `decisions/devops/church-autoadvance.md`. If it fails with `$'\r'` errors, the script was checked out with CRLF: add `*.sh text eol=lf` to `.gitattributes`.
- [ ] **Founder (V0, 2026-11-25): booth timing with the fix.** Not a merge gate. On the booth PC, numbers only: in Windows PowerShell from the repository root, dot-source `scripts/dev/msvc-release-flags.ps1`; once, `cargo clean --manifest-path src-tauri/Cargo.toml -p whisper-rs-sys --release`; build a release `voxxad` in that window (`docs/RELEASING.md` § 7); in Git Bash, `bash scripts/ci/check-msvc-release-flags.sh src-tauri/target/release` must print two `-O2` lines; then private song P1 with Tiny and the 30 s encoder. Pass: ≤ 150 s of compute (it was 461 s).
- [x] **Q9V close-out (orchestrator; pre-approved, D6), 2026-11-21.** PR #157 merged into `main` as 5bbac72; tickets filed as #158-#167; #105 closed. The items below are the original plan.
  1. [ ] Push `q9v/base` (with the `gh` credential helper; the push edits `ci.yml`, so it needs the `workflow` scope) and open a draft PR against `main`. Every CI job must be green, including the new "media guard" job; `eval_numbers_only` and the CI-step test run on Linux and macOS for the first time.
  2. [ ] Merge.
  3. [ ] Create the label `q9v`, file the 10 open tickets (`reviews/2026-11-20-q9v-tickets.md`), close #105 with 7f71681, and add the comments to #74, #79 and #107.
  4. [ ] Then start Q10 (handoff, "From: Q9V → To: Q10" and "From: Q9 → To: Q10").
- [ ] **Founder: set the support e-mail address** (Q9-01; user decision 2 of Q9). Choose the address Call for help's e-mail draft should go to (a support inbox, or one per church). Then `SUPPORT_EMAIL` (`src-tauri/src/help/mod.rs:16`), its test, the quick-start's Call for help section and the dialog's sentence change together. Until then, the volunteer types the tech lead's address.
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
- [x] **Q6 close-out (orchestrator).**
  1. [x] The user's choice applied in its own commit (85c736c) and recorded in ADR-016 (amendment of 2026-11-18); the ponytail lane re-run, reviewed and compared; grafts applied; gate 5 fixed (b7a5382, 734b467); verifiers passed at 97cc21b.
  2. [x] Pushed `q6/base`, opened PR #82; CI green on all 5 jobs at 1254374 (run 44480642855); merged into `main` as a60d1ce (2026-11-19). #76 closed by the merge, #62 closed by hand.
  3. [x] Filed the 12 open Q6 tickets as #83–#94 (label `q6`); added the Q6 numbers to #61, #73, #77 and #78. #76 and #62 close when PR #82 merges.

  The user pre-approved the push, PR, CI fixes, merge and filing (handoff, "User decisions for Q6").
- [x] **Q7 close-out (orchestrator, pre-approved in handoff "User decisions for Q7").**
  1. [x] Pushed `q7/base`; draft PR #95.
  2. [x] First-CI-run evidence collected and recorded (QA report §12, security review §12-§13). The Windows ETW check failed once in CI and was fixed (5d69368). The throwaway mutant (f18ab4c) turned Linux and Windows red and was reverted (fc93326). full-eval was green on 3 OSes and equal to replay. The Node 20 pins were bumped (230a42e).
  3. [x] Merged into `main` as c448139 (2026-11-19). CI green on all 5 jobs at a7f6c89 (run 44545240949) and on `main` (run 44548298909); full-eval green on 3 OSes with no warnings at a7f6c89 (run 44545241184).
  4. [x] full-eval run by hand on `main` (run 44548333928, at c448139): green on all three OSes; each table equals the replay table (QA report §13).
  5. [x] Filed the 12 open Q7 tickets as #96–#107 (label `q7`); notes added to the nine earlier issues. #47 closed by the merge, #75 closed by hand. The notes were added to #48, #49, #54, #78, #9, #77, #69, #89 and #70 (tickets file, last section).
- [x] Increment 1′ (Q4–Q7) … per the 2026-11-11 prompt pack; Q4 merged (571cf0d); Q5 merged (69f7d27); Q6 merged (a60d1ce); Q7 merged (c448139). **Increment 1′ closed 2026-11-19.**
- [x] **Q8 close-out (orchestrator).** The push, PR, CI fixes, merge and filing are pre-approved (handoff, "User decisions for Q8").
  1. [x] **Merge blocker, Q8-10.** Done in 88aa63b (see below). Original plan: In `src/simple/firstrun.js`, replace the key block on the overlay box (line 31) with a capture-phase `keydown` listener on `window` while the guide is shown. Add e2e checks with the focus on `document.body`: on step 1, Space and `s` call neither `start_listening` nor `toggle_stage_display`; during the 20-second test, Space does not call `stop_listening` and the verdict is unchanged. Each check must fail on ee431ce.
  2. [x] Q8-10 was fixed by the orchestrator in 88aa63b: a capture-phase `keydown` listener on `window` blocks every key while the guide is shown. Four new e2e checks press Space and `s` with the focus on the page body, on step 1 and during the 20-second test. All four fail on the old code (41 passed, 6 failed) and pass with the fix. An independent verifier confirmed it at 88aa63b: 13 shortcut keys were blocked with the focus on the body, the buttons and the select, and the guide's own keyboard controls still work. The old-code mutant and the bubble-phase mutant both fail. The verifier's one survivor, keys blocked even after the guide closes, is killed by fce3e58 ("s reaches main.js again" after Skip; 117 e2e checks). Original plan: a verifier re-runs the gates, `npm run test:gui` and `npm run test:gui:e2e`, and confirms the fix. Record the result in the QA record and in `reviews/2026-11-19-ponytail-q8.md` § 6.
  3. [x] Pushed `q8/base`, PR #108; CI green on all 5 jobs at 0dffe86 (run 44607712336), including the browser test's first GitHub run on Windows (117 passed). Original plan: push `q8/base` and open a draft PR against `main`. Every CI job must be green, including the GUI logic test on 3 OSes and the browser test on Windows, which has never run on GitHub (Q8-04).
  4. [x] Merged into `main` as a3967e8 (2026-11-19).
  5. [x] Filed the 15 open Q8 tickets as #109–#123 (label `q8`; Q8-10 closed, not filed); comments added. Original plan: file the open Q8 tickets as issues (label `q8`), apart from Q8-10 if its fix closed it. Add the comments to #7, #44, #34, #9, #102 and #37.
- [x] **Q9 close-out (orchestrator), 2026-11-19/20.** PR #124 CI green at 7ad24fc (run 44655529197); merged into `main` as e9328e2. Main CI then crashed on windows-latest with STATUS_ILLEGAL_INSTRUCTION in `model_download`: whisper.cpp was built with `-march=native` and restored from rust-cache on a different CPU. Fixed by PR #141 (df40328, `CMAKE_PROJECT_INCLUDE` → `src-tauri/cmake/ggml-portable.cmake`, GGML_NATIVE off); main CI green (run 44663429693). Ticket #142: installer builds have the same `-march=native` default (Q10). 16 Q9 tickets filed as #125–#140 (label `q9`); #102 closed by the merge; comments on #53, #49, #99, #116, #109, #34, #8 and #121. The four leftover Edge processes were stopped and the Q9 worktrees removed. The items below are the original plan.
  1. [ ] Push `q9/base` (with the `gh` credential helper, as before) and open a draft PR against `main`. Every CI job must be green, including the GUI logic test on 3 OSes and the browser test on Windows. Q9 has not run in CI yet.
  2. [ ] Merge. After the merge, check that the three runbook URLs open on GitHub (they return 404 until then).
  3. [ ] File the 16 open Q9 tickets (`reviews/2026-11-19-q9-tickets.md`) as issues with label `q9` (create the label first). Close #102 with the commit (2c28c15). Add the comments to #53, #49, #99, #116, #109, #34, #8 and #121.
  4. [ ] Housekeeping: four headless Edge root processes with `%TEMP%\voxxa-verifya3-*` profiles were still running at the end of Q9 (PIDs 14608, 14632, 14872, 18796; started 1:16-1:18 PM, 2026-11-19). No Q9 agent started them. Stop them by PID if they are still there and nobody claims them.
- [x] **Q9W close-out (orchestrator), 2026-11-20.** PR #144 merged into `main` as 84a3966; the 12 Q9W tickets are filed as #145-#156 (label `q9w`); #117 is closed. (The issue comments were not re-checked by the Q9V finisher.) The items below are the original plan.
  1. [ ] Push `q9w/base` (with the `gh` credential helper) and open a draft PR against `main`. Every CI job must be green; Q9W has not run in CI, and the Linux and macOS jobs run the off-Windows stubs (empty process list, guard passes) for the first time.
  2. [ ] Merge.
  3. [ ] Create the label `q9w`, file the 12 open tickets (`reviews/2026-11-20-q9w-tickets.md`), close #117 with 7602872, and add the comments to #6, #13, #42, #44, #48, #33 and #39.
  4. [ ] Then start Q9V (handoff, "From: Q9W → To: Q9V").
- [ ] Increment 2′ (Q8–Q11, plus Q9W and Q9V from R0): Q8 merged (a3967e8); Q9 merged (e9328e2); R0 done (ADR-019); Q9W merged (84a3966); Q9V merged (5bbac72); Q10 done on `q10/base` (f424c74, both verifiers passed; CI, PR, merge and the `v0.1.0-rc.1` draft run pending); then Q11 (close Increment 2′ on LHMEZPC01 with WorshipTools Presenter)
  - [x] 2026-11-13 Q5 verifier fixes on `q5/base`: every surviving mutant named by the verifier is now killed by a test (song not compared, false advance lowering recall, EARLY_MS 999 and `<=`→`<`, the total row's false/double sums, unblank not resetting the blank, a failing `voxxad` read as success); a metric with a value but no threshold now fails the gate (ADR-015 D8 amended); voxxa-eval exits 2 when `voxxad` ran in the wrong mode (a replay that ran Whisper); both session logs re-recorded with the final `--full --record` (only the start line's `log` field changed).
  - [x] 2026-11-13 Q5 second verifier pass on `q5/base`: tests now kill the round-2 survivors: M42 (main.rs checking only the total row; a binary-level test breaks only per-fixture thresholds and needs both FAIL lines and no FAIL total), M25/M26 (one metric's comparison reversed), M37 (the table's count cells) and M47/M48 (the audio recipe's 16 kHz mono 16-bit check and header byte rate). Each test was run against its mutant and fails. No product code changed; voxxa_eval tests 23 to 26.
  - [x] 2026-11-13 Q5 verifier round 3 at c10e963: verifier A passed; verifier B failed on six surviving mutants (Q5-08 … Q5-10). The four that mattered were closed with tests in 5858872 (orchestrator, checked against each mutant); M57 and M68 deferred (Q5-10).
- [ ] Founder: apply to SignPath; open Partner Center; e-mail CCLI; form LLC; start research screener
- [ ] Phases 4–6 per increment

### Blocked
- None.

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

  After Q8 (ee431ce): GUI 2,780 / 3,000 (220 left; the welcome overlay was deleted), and the new row Simple mode (`src/simple/*`) 352 / 600 (248 left; Q8 user decision 1, ADR-017 § 5). Every Rust row is unchanged.

  After Q9 (89f4ccb): **everything else in `src-tauri/src` 2,494 / 2,500 (6 left)**; Simple mode 436 / 600 (164 left); GUI 2,780 / 3,000 unchanged. No cap raised (Q9 user decision 3). The backend row is now the tightest: any new backend code outside its own rows needs lines removed first or a budget decision before code.

  After Q9W (b0de71b): the new row discovery (`src/discovery/`) 223 / 250 (D5, ADR-019 § 10); "everything else in `src-tauri/src`" cap lowered 2,500 → 2,317 when discovery moved out, now **2,315 / 2,317 (2 left)**; presenters 1,374 / 2,000 (`keystroke.rs` 396 / 400); GUI 2,784 / 3,000; Simple mode 450 / 600. No cap raised. Q10's Rust needs lines found or a budget decision first (Q9W-04).

  After Q9V (be4819b): voxxa-eval 469 → 488 / 600 (112 left); every other row unchanged; the new tests and the ci.yml job are not counted.

  After Q10 (f424c74): the new row updater (`src/updater/`) 98 / 150 (ADR-020 § 5, the D5 precedent); "everything else in `src-tauri/src`" **2,316 / 2,317 (1 left)**, the wiring offset by inlining two `let`s; GUI 2,784 → 2,859 / 3,000 (141 left); Simple mode 450 / 600. No cap raised. Any further backend code outside its own row needs lines removed first or a budget decision.

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
- R5 SmartScreen friction — Store listing + SignPath/Trusted Signing + winget. Q10: the pipeline signs with SignPath or Azure as soon as the founder adds either set of secrets, and fills the winget manifest; until then the installers are unsigned and SmartScreen warns (said in the release log).
- R16 (new, Q10) The updater key. Every installed copy trusts only the pubkey it was built with (84F2CD98C608E9C9). If the founder does not hold its private half, or loses or leaks it after the first published release, those copies cannot be updated safely. Mitigation: confirm or regenerate the pair before the first published release; two offline holders (`docs/RELEASING.md` § 6); `requireSignedVersion` once the CLI supports it (Q10-03).
- R6 Founder on-call load — bounded Sunday window; MSP partners carry their churches
- R7 Windows-first vs Mac churches — unsigned previews; ADR-002 review trigger
