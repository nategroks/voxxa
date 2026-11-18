# Q6 before/after: ADR-009 minimum (10x lane, branch `q6/10x`)

**Date:** 2026-11-13 · **Design:** ADR-016 (with its measurement amendment) · **Before:** 733c7e1 (`q6/base`) · **After:** `q6/10x` head (see the commit that adds this file).

All numbers are `voxxa-eval` in replay mode (what CI runs), debug build, Windows 11. Latency is **stream time at file speed**: `voxxad` without `--realtime` stamps `t` from a sample clock (ADR-013), so it leaves out inference time; it is not wall-clock time. Live, one hop costs more than 1 s on the reference PC even optimised (Q4-02), so hypotheses come every 2-5 s and every latency here would be up to one decode plus one hop higher. Hymn text is never quoted; slides are `song/slide`.

## What changed in the inputs
- `refrain-it-is-well/setlist.json`: It Is Well gained `sections` (verse, refrain, verse, verse, verse). Its `session.jsonl` was re-recorded with `voxxa-eval --full --record` (fingerprint changed). All 728 `tick`, `speech`, `hypothesis` and `window` lines are identical to the 733c7e1 recording; only the start line and the recorded actions differ. Full mode gives the same row as replay (median 2,120, p95 3,900).
- New fixture `prayer-before-first-song`: Q5-04's reproduction from the Q5 10x lane (30e8990), `audio.json` converted to the merged list format, the blank row in the merged form. Its log was re-recorded with `--full --record` at the Q6 head (389 input lines identical to the 30e8990 recording); full mode equals replay.
- `src-tauri/tests/fixtures/voxxad/session-pd-hymns.jsonl` (a test fixture, not an eval fixture) was re-recorded the same way (3 s, Amazing Grace v1, 1 s, v2, 4 s, the prayer, 6 s): 342 input lines identical; the recorded goto 0/1 moves from 25,000 to 18,000 ms, which is what the replay test pins.
- The other four fixture logs are unchanged; replay fills in the `evidence` defaults (ADR-016 Decision 7).

## Before (733c7e1; both Q6 lanes measured it independently)
| fixture | slides | gotos | hits | missed | prec | recall | median ms | p95 ms | false | double | blank ms | unblank ms |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| jump-be-thou-my-vision | 4 | 1 | 1 | 3 | 1.00 | 0.25 | 900 | 900 | 0 | 0 | - | - |
| prayer-between-songs | 4 | 4 | 4 | 0 | 1.00 | 1.00 | 8490 | 11230 | 0 | 0 | 7280 | 8490 |
| prayer-then-reprise | 3 | 3 | 3 | 0 | 1.00 | 1.00 | 2900 | 8900 | 0 | 0 | 200 | 1500 |
| prayer-without-pause | 2 | 2 | 2 | 0 | 1.00 | 1.00 | 2900 | 8900 | 0 | 0 | 6600 | - |
| refrain-it-is-well | 6 | 4 | 3 | 3 | 0.75 | 0.50 | 4120 | 10310 | 1 | 0 | - | - |
| total | 19 | 14 | 13 | 6 | 0.93 | 0.68 | 4120 | 11230 | 1 | 0 | 7280 | 8490 |

Q5-04's reproduction at 733c7e1 (from the Q5 tickets, not a committed fixture then): 2 slides, 1 goto, 1 hit, recall 0.50, latency 6,710 ms, blank miss.

## After (`q6/10x`)
| fixture | slides | gotos | hits | missed | prec | recall | median ms | p95 ms | false | double | blank ms | unblank ms |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| jump-be-thou-my-vision | 4 | 4 | 4 | 0 | 1.00 | 1.00 | 1200 | 5850 | 0 | 0 | - | - |
| prayer-before-first-song (new) | 2 | 2 | 2 | 0 | 1.00 | 1.00 | 1710 | 2800 | 0 | 0 | 6800 | 1710 |
| prayer-between-songs | 4 | 4 | 4 | 0 | 1.00 | 1.00 | 2490 | 4870 | 0 | 0 | 7280 | 2490 |
| prayer-then-reprise | 3 | 3 | 3 | 0 | 1.00 | 1.00 | 2900 | 2900 | 0 | 0 | 200 | 1500 |
| prayer-without-pause | 2 | 2 | 2 | 0 | 1.00 | 1.00 | 1900 | 2900 | 0 | 0 | 6600 | - |
| refrain-it-is-well | 6 | 6 | 6 | 0 | 1.00 | 1.00 | 2120 | 3900 | 0 | 0 | - | - |
| total | 21 | 21 | 21 | 0 | 1.00 | 1.00 | 2310 | 4870 | 0 | 0 | 7280 | 2490 |

Per hit (action `t` − truth `t_ms`, ms):
- jump: 0/0 900; 0/2 1200; 0/3 2410; 0/0 again 5850 (Whisper Tiny gives no transcript between 41 and 47 s, then needs a second one).
- prayer-before-first-song: 0/0 2800 (was never shown); blank 6800; 0/1 1710.
- prayer-between-songs: 0/0 3900; 0/1 4870 (the first transcript of verse 2 is "holy, holy, holy", which opens every slide of that hymn); blank 7280; 1/0 2490 (was 8490); 1/1 2230 (was 11230).
- prayer-then-reprise: 0/0 2900; 0/1 2900 (was 8900); blank 200; 0/1 again 1500.
- prayer-without-pause: 1/0 2900; 1/1 1900 (was 8900); blank 6600.
- refrain: 0/0 3900; 0/1 1235; 0/2 2120; 0/1 1425; 0/3 2310; 0/1 2215 (the three refrain rows were missed before, and the refrain goto that landed in verse 2's span was the false advance).

## Per fixture, per metric: better, same or worse
| fixture | better | same | worse |
|---|---|---|---|
| jump-be-thou-my-vision | gotos/hits 1 → 4, missed 3 → 0, recall 0.25 → 1.00 | precision, false, double | **median 900 → 1200, p95 900 → 5850** |
| prayer-between-songs | median 8490 → 2490, p95 11230 → 4870, unblank 8490 → 2490 | precision, recall, false, double, blank | none |
| prayer-then-reprise | p95 8900 → 2900 | precision, recall, median, false, double, blank, unblank | none |
| prayer-without-pause | median 2900 → 1900, p95 8900 → 2900 | precision, recall, false, double, blank | none |
| refrain-it-is-well | precision 0.75 → 1.00, recall 0.50 → 1.00, median 4120 → 2120, p95 10310 → 3900, false 1 → 0 | double | none |
| total | precision, recall, median, p95, false, unblank | double, blank | none |

**Exit criterion "eval not worse on any fixture": NOT met on one fixture.** `jump-be-thou-my-vision` is worse on `median_ms` and `p95_ms`. The cause is the recall gain, not a slower old hit: its one hit before (0/0, 900 ms) is unchanged, and the three slides it missed are now hit at 1,200, 2,410 and 5,850 ms. The earliest the first new hit can come is 1,200 ms (the first transcript of verse 3 is at 17,000 for a verse that starts at 15,800), so no conductor that follows the jump can keep that fixture's median under its 1,000 ms threshold.

## Baseline (`fixtures/baseline.json`)
- Tightened everywhere a metric improved (latency thresholds by the ADR-015 rule: the next multiple of 500 ms strictly above the measured worst; counts and ratios exact): prayer-between-songs median 2500, p95 5000, unblank 2500; prayer-then-reprise p95 3000; prayer-without-pause median 2000, p95 3000; refrain precision 1.0, recall 1.0, median 2500, p95 4000, false 0; jump recall 1.0; total precision 1.0, recall 1.0, median 2500, p95 5000, false 0, unblank 2500.
- New row `prayer-before-first-song`: precision 1.0, recall 1.0, median 2000, p95 3000, false 0, double 0, blank 7000, unblank 2000.
- **Not loosened:** jump `median_ms` 1000 and `p95_ms` 1000 (user decision 4, binding). So the replay gate, the CI step and two tests (`replay_of_the_committed_fixtures_passes_the_baseline`, `a_broken_per_fixture_threshold_fails_the_gate_while_total_passes`) fail on exactly `FAIL jump-be-thou-my-vision: median_ms is 1200, baseline needs <= 1000` and `FAIL jump-be-thou-my-vision: p95_ms is 5850, baseline needs <= 1000`. With those two cells at 1500 and 6000 (the ADR-015 rule applied to the new numbers) the gate prints OK; that change is the user's to approve.

## The Q6 decision rule, after
- False advances: 0 over 7 sung songs (the six fixtures' truth: jump 1, prayer-before-first-song 1, prayer-between 2, reprise 1, without-pause 1, refrain 1) = **0 per 10 songs: met** (before: 1.67).
- p95 latency over all 21 matched advances: **4.87 s: not met** (limit 2.5 s; before 11.23 s). Over the five fixtures of the decision alone: 5.85 s (19 hits). Median 2.31 s.
- What bounds it now is mostly the recogniser at file speed: a part's first transcript arrives 1-3 s after it starts (the stitcher holds 4 words), so 7 of the 21 hits are under 2 s, 10 are at 2.1-2.9 s, and 4 are over 3 s: two first slides of a service (3.9 s each: the first transcript arrives at 6 s for singing that starts at 2.1 s) and the two cases above (4.87 and 5.85 s).

## Scripted conductor tests (`src-tauri/tests/conductor_evidence.rs`, `conductor_unchanged.rs`)
Text comes from the committed public-domain fixture setlists; transcripts are 5-word chunks every 1.5 s of stream time.
- Repeated chorus: It Is Well sung v1, refrain, v2, refrain, v3, refrain gives exactly gotos 0/0, 0/1, 0/2, 0/1, 0/3, 0/1, labelled and unlabelled. With one word in five misheard in verse 3, the labelled song moves on verse 3's first transcript (the refrain's "next" is the verse after the last one); the unlabelled one needs more.
- Bridge jump: v1 then straight to a slide labelled bridge, then the refrain: 0/0, 0/4, 0/1. Unlabelled Be Thou My Vision v1, v3, v4, v1: 0/0, 0/2, 0/3, 0/0.
- Dropped words: every third word dropped and every seventh misheard: the same six gotos and no others; a transcript of words from no slide in the middle of a verse moves nothing.
- Operator override: an early `manual_step(+1)` is never undone by the old slide's last words and gives no second goto; after `manual_step(-1)` the room singing the slide the operator left does not pull the screen forward, and the next verse is still followed; after `jump_to_song` to the top, the refrain becomes reachable again once verse 1 is sung.
- `min_dwell_ms`: with a 60 s dwell only the song start moves.
- Unchanged behaviour: a 23-step script (ticks, silence to inter-verse silence to blank hold, re-show after a blank, `manual_step` +1/+5/clamped/−1/−10/0, unmatched speech blanking on a tick, an external blank) gives the same trace at 733c7e1 (run on a `git archive` export of that commit) and at the Q6 head.
- Schema: old setlists round-trip without a `sections` key; kinds incl. `refrain` and an unknown kind; four kinds of bad labels refused by `check_setlist` with the song's title; an old log's `SmartConfig` gets the ADR-016 defaults.

## Process
- No microphone, no presenter: only `voxxad --replay` and `voxxa-eval --full` (which passes `--shadow`). Every process started was stopped or exited; two full runs were stopped by PID (7820, 276, 13020, 6864) because three parallel Whisper runs starved each other, then re-run one at a time.
- The other lane (`q6/ponytail`) was not read.

## Finish on `q6/base` (after the merge d92800c and the review fixes)
**Before, measured again.** The finisher re-measured 733c7e1 on a `git archive` export of that commit, in replay. Every cell of the "Before" table above came out the same. The finisher also replayed `prayer-before-first-song`'s committed log with the 733c7e1 conductor: 2 slides, 1 goto, 1 hit, 1 missed, recall 0.50, latency 6,710 ms, blank and unblank `miss`. That matches the Q5 ticket's number.

**After, on the `q6/base` head, in replay.** Every cell equals the "After (`q6/10x`)" table above. Full mode (Whisper Tiny, `--shadow`, debug, file speed, 10 min 26 s for the six fixtures) printed the same table, cell for cell, and the same two FAIL lines. Neither review fix changes behaviour: one is a test, and the other removes a log line and adds the song index to another.

| fixture | before (733c7e1) median / p95 ms | after median / p95 ms | before → after recall | false before → after |
|---|---|---|---|---|
| jump-be-thou-my-vision | 900 / 900 | 1200 / 5850 | 0.25 → 1.00 | 0 → 0 |
| prayer-before-first-song | 6710 / 6710 (log replayed with the old conductor) | 1710 / 2800 | 0.50 → 1.00 | 0 → 0 |
| prayer-between-songs | 8490 / 11230 | 2490 / 4870 | 1.00 → 1.00 | 0 → 0 |
| prayer-then-reprise | 2900 / 8900 | 2900 / 2900 | 1.00 → 1.00 | 0 → 0 |
| prayer-without-pause | 2900 / 8900 | 1900 / 2900 | 1.00 → 1.00 | 0 → 0 |
| refrain-it-is-well | 4120 / 10310 | 2120 / 3900 | 0.50 → 1.00 | 1 → 0 |
| total (5 fixtures before, 6 after) | 4120 / 11230 | 2310 / 4870 | 0.68 → 1.00 | 1 → 0 |

**Gate status.** The replay gate, the CI step and two `voxxa_eval` tests still fail on the two `jump-be-thou-my-vision` cells (`median_ms` 1200 against 1000, `p95_ms` 5850 against 1000). That is ADR-016's second amendment, item 5, and it is pending the user's decision. With option (a) on a scratch copy of `baseline.json` (`median_ms` 1500, `p95_ms` 6000), `voxxa-eval --baseline <copy>` prints OK and exits 0, and `cargo test --test voxxa_eval` passes 26 of 26. The committed `baseline.json` was not changed.

## Verifier fixes on `q6/base` (tests only, after 906e78b)
Two test commits close the verifier's mutation survivors (ADR-016, amendment "Q6 verifier fixes"): decay, margin, the 12-word window (11 and 13), the dwell after `manual_step`, `manual_step` clearing a blank and the buffer, and `blank_hold` clearing the buffer (not redundant: the re-show path keeps the buffer). Each mutant was applied and run; each fails its new test on the asserted step. Both `conductor_unchanged` traces also pass on a `git archive` export of 733c7e1. No conductor, metric, fixture or baseline file changed.

**Eval, re-run after these commits (replay, debug, stream time at file speed).** Before: `voxxa-eval` built from a `git archive` export of 733c7e1, on that commit's fixtures. After: the CI step on the new head. Every cell equals the tables above:

| fixture | before (733c7e1) median / p95 ms | after median / p95 ms | recall before → after | false before → after |
|---|---|---|---|---|
| jump-be-thou-my-vision | 900 / 900 | 1200 / 5850 | 0.25 → 1.00 | 0 → 0 |
| prayer-before-first-song | not a fixture then | 1710 / 2800 | - → 1.00 | - → 0 |
| prayer-between-songs | 8490 / 11230 | 2490 / 4870 | 1.00 → 1.00 | 0 → 0 |
| prayer-then-reprise | 2900 / 8900 | 2900 / 2900 | 1.00 → 1.00 | 0 → 0 |
| prayer-without-pause | 2900 / 8900 | 1900 / 2900 | 1.00 → 1.00 | 0 → 0 |
| refrain-it-is-well | 4120 / 10310 | 2120 / 3900 | 0.50 → 1.00 | 1 → 0 |
| total | 4120 / 11230 | 2310 / 4870 | 0.68 → 1.00 | 1 → 0 |

**Gates on the new head.** 1 npm, 2 fmt, 3 clippy, 5 cargo-deny, 6 loc budget and 7 the `dictat` grep pass. Gate 4 fails only on the two `voxxa_eval` tests above, and the CI eval step exits 1 on the same two `jump-be-thou-my-vision` cells. Both still wait on the user's decision (ADR-016, second amendment, item 5). **The exit criterion "eval not worse on any fixture" is still not met** on that fixture's `median_ms` and `p95_ms`.

## Second verifier round on `q6/base` (a test, after 88f7870)
One test commit closes the last mutation survivor: `follow()` (`src-tauri/src/aligner/mod.rs`) no longer clearing the 40-word buffer after a tracker move. `a_tracker_move_clears_the_song_detection_buffer` (`src-tauri/tests/conductor_evidence.rs`) moves song 0 of the refrain setlist to its refrain through the tracker, then sings song 1 straight after, with no blank. With the clear, song 1 starts on its first transcript. With the mutant, it starts on its fifth, and the test fails. Ticket Q6-02 is closed: the `is_start`-always-true mutant now fails `evidence_is_kept_across_transcripts_with_decay`. No conductor, metric, fixture or baseline file changed.

**Eval, re-run (debug, stream time at file speed, not wall clock; Q4-02 applies live).** Before: replay with `voxxa-eval` built from a `git archive` export of 733c7e1, on that commit's fixtures, plus `prayer-before-first-song`'s committed log replayed with that conductor. After: the CI step on the new head (replay), and `voxxa-eval --full` (Whisper Tiny, `--shadow`, 10 min 6 s). Full mode printed the same table, cell for cell, and the same two FAIL lines. Every cell equals the table in the previous section. That covers jump 900 / 900 → 1200 / 5850 ms, recall 0.25 → 1.00, and the total 4120 / 11230 → 2310 / 4870 ms, false advances 1 → 0.

**Gates on the new head.** 1, 2, 3, 5, 6 and 7 pass. Gate 4 (`cargo test --no-fail-fast`) fails only `replay_of_the_committed_fixtures_passes_the_baseline` and `a_broken_per_fixture_threshold_fails_the_gate_while_total_passes`; every other test binary passes. The CI eval step exits 1 on the same two `jump-be-thou-my-vision` cells. **The exit criterion "eval not worse on any fixture" is still not met.** It waits on the user's decision (ADR-016, second amendment, item 5): (a) a one-time exception to user decision 4 setting that row's `median_ms` to 1500 and `p95_ms` to 6000, or (b) keeping `q6/base` out of main. `fixtures/baseline.json` was not changed.

## Finish on `q6/base`, 2026-11-18 (the user's baseline decision, the last mutation pins, ponytail grafts)
Commits after 151a8b3, each on its own:
1. `fixtures/baseline.json`: `jump-be-thou-my-vision` `median_ms` 1000 → 1500 and `p95_ms` 1000 → 6000, the user's decision 3 of 2026-11-18 (a one-time exception; the old cells came from 1 of 4 slides hit; the recorded log's floor is 1200 / 3850 ms). The total row's `blank_ms` 7500 and `unblank_ms` 2500 were already in place and are accepted by the same decision. ADR-016 amendment of 2026-11-18.
2. Tests for the verifier's two mutation survivors (`prior_chorus`; the tie-break order). Each mutant was applied by hand and fails its test: `prior_chorus` → `prior_section`; `>` → `>=`, `max_by` over `0..n`, and a fold over `0..n` in the tie-break.
3. Ponytail graft: its scripted scenarios and 27-step trace, as `src-tauri/tests/conductor_scripted.rs`. On a `git archive` export of 733c7e1 the trace passes and the four tracker scenarios fail (the old conductor cannot follow them); on `q6/base` all five pass. So the trace still records pre-Q6 behaviour, and base's Q5-04 change does not alter it.
4. Ponytail graft: `fixtures/README.md` now says fields missing from a log's start-line config take today's defaults (the `evidence` block in 4 of 6 logs).
5. Ponytail graft: `decay` clamped to 0..=1 (no new code line; replay unchanged cell for cell; pinned by a test).

Grafts not taken: none of the six was skipped. The ponytail lane's deletion of `slide_advance_debounce_ms` and `advance_margin` was not proposed as a graft (ADR-016 Decision 7 keeps them; the user's call), and its scoring idea (the current slide's tail as context for the slide being entered) stays a ticket, not a graft.

**Eval (debug, stream time at file speed, not wall clock; Q4-02 applies live).** Before: `voxxa-eval` built from a `git archive` export of 733c7e1, replaying that commit's fixtures, plus `prayer-before-first-song`'s committed log replayed with that conductor; every cell equals the Before table above. After: the CI step on the new head (replay); it prints `OK: no metric is worse than ... baseline.json`.

| fixture | hits before → after | recall before → after | median / p95 ms before | median / p95 ms after | false before → after | blank / unblank ms before | blank / unblank ms after |
|---|---|---|---|---|---|---|---|
| jump-be-thou-my-vision | 1 → 4 of 4 | 0.25 → 1.00 | 900 / 900 | 1200 / 5850 | 0 → 0 | - / - | - / - |
| prayer-before-first-song | 1 → 2 of 2 | 0.50 → 1.00 | 6710 / 6710 | 1710 / 2800 | 0 → 0 | miss / miss | 6800 / 1710 |
| prayer-between-songs | 4 → 4 of 4 | 1.00 → 1.00 | 8490 / 11230 | 2490 / 4870 | 0 → 0 | 7280 / 8490 | 7280 / 2490 |
| prayer-then-reprise | 3 → 3 of 3 | 1.00 → 1.00 | 2900 / 8900 | 2900 / 2900 | 0 → 0 | 200 / 1500 | 200 / 1500 |
| prayer-without-pause | 2 → 2 of 2 | 1.00 → 1.00 | 2900 / 8900 | 1900 / 2900 | 0 → 0 | 6600 / - | 6600 / - |
| refrain-it-is-well | 3 → 6 of 6 | 0.50 → 1.00 | 4120 / 10310 | 2120 / 3900 | 1 → 0 | - / - | - / - |
| total (5 fixtures before, 6 after) | 13 → 21 | 0.68 → 1.00 | 4120 / 11230 | 2310 / 4870 | 1 → 0 | 7280 / 8490 | 7280 / 2490 |

- Double advances are 0 everywhere, before and after. Precision before was 0.75 on the refrain fixture (0.93 total); after, 1.00 everywhere.
- **Not worse on any fixture, with one cell pair read the user's way:** `jump-be-thou-my-vision`'s median and p95 are higher (900 / 900 → 1200 / 5850) because 3 hits were added to its single fast one; the user accepted that with the 1500 / 6000 cells. Every other cell is the same or better.
- **The Q6 budget:** 0 false advances over 7 sung songs (met; before 1.67 per 10 songs); p95 4.87 s against 2.5 s (**not met**; before 11.23 s). ADR-016 is the decision file.

**Full mode.** `voxxa-eval --full` (Whisper Tiny, `--shadow`, `--stdin-audio`, debug build, 11 min 36 s) on the final head printed the same table as replay, cell for cell, and `OK: no metric is worse than ... baseline.json`.

**Gates on the final head (587d2ad).** 1 `npm ci && npm run build`, 2 fmt, 3 clippy `-D warnings`, 4 `cargo test` (21 test binaries, 0 failures; `conductor_evidence` 22 tests, `conductor_scripted` 5), 6 loc budget (conductor 1173 / 1200, conductor evidence 215 / 500) and 7 the `dictat` grep pass, and so does the CI eval step (exit 0). **Gate 5 (cargo-deny) fails on advisories**: RUSTSEC-2026-0543 (chrono, unmaintained) and RUSTSEC-2026-0548 (zip, vulnerability); bans and licences are ok. `src-tauri/Cargo.lock` and `Cargo.toml` are identical to 733c7e1, so this is new advisory data, not a Q6 change; the earlier "gate 5 passes" in this file was true when it was run. Ignoring the advisories or upgrading those crates is the user's decision.
