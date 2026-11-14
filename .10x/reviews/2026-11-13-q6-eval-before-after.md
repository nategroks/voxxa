# Q6 eval: the decision, and the conductor before and after (2026-11-13)

**Step:** Q6, conductor against the budget. **Branch:** `q6/base`. **Before:** 733c7e1 (Q5's conductor plus the three Q6 fixtures). **After:** 4edd7f6 (ADR-009's minimum, ADR-016, with the review and verifier fixes).
**Result:** the conductor missed the budget, so ADR-009's minimum was built. It removes the false advance and every missed slide, and cuts p95 latency from 11.23 s to 4.87 s. It still misses the 2.5 s latency budget. **One fixture is worse on two latency cells, so Q6's exit criterion is not met and the PR must not be merged yet** (see "Exit criterion" below).

The lane-level detail, with every per-hit latency, is in `2026-11-13-q6-before-after.md`. This file is the orchestrator's summary. Slides are named `song/slide`; no hymn text is quoted.

## 1. The decision rule
The prompt's rule, normalised as the user's facts say:
- **False advances:** total false advances over all fixtures ÷ total songs sung × 10 must be ≤ 1. "Songs sung" comes from each fixture's `truth.jsonl`, not from the setlist (a setlist lists songs nobody sang).
- **Latency:** p95 over all matched advances across all fixtures (nearest rank, as the eval's `total` row computes it) must be ≤ 2,500 ms.
- If both hold, write "ADR-009 deferred" and stop. Otherwise build ADR-009's minimum.

**What kind of latency this is.** Stream time at file speed. `voxxad --stdin-audio` without `--realtime` stamps `t` from a sample clock (ADR-013), and replay re-uses those `t` values. So the number leaves out inference time. It is not wall-clock time with `--realtime`. Live, Q4-02 (#60) applies: one hop costs more than 1 s on the reference PC in debug, so live hypotheses arrive every 2-5 s and every latency below could be up to one decode plus one hop higher. That can only make a fail worse.

## 2. Two independent decisions (both: build the minimum)
Two agents measured 733c7e1 separately, each in its own build directory, and neither saw the other's work.

| | Decision 1 (10x lane) | Decision 2 (ponytail lane) |
|---|---|---|
| Build | debug, `CARGO_TARGET_DIR=C:/dev/q3-10x-target` | debug, `CARGO_TARGET_DIR=C:/dev/q3-pony-target` |
| Replay (the CI step) | exit 0, "OK", 3 min 44 s | exit 0, "OK", 4 min 22 s |
| Full mode (Whisper Tiny, `--shadow`) | started, did not finish before the report | finished: exit 0, the identical table, 17 min 41 s for about 320 s of audio |
| False advances | 1 over 6 sung songs = **1.67 per 10 songs** | 1 over 6 sung songs = **1.67 per 10 songs** |
| p95 latency | **11.23 s** over 13 hits (median 4.12 s) | **11.23 s** over 13 hits (median 4.12 s; recomputed by hand from the per-hit list) |
| Meets the budget | no, on both limbs | no, on both limbs |
| Verdict | build ADR-009's minimum | build ADR-009's minimum |

- Songs sung: jump 1, prayer-between-songs 2, prayer-then-reprise 1, prayer-without-pause 1, refrain 1, so 6.
- Counting setlist entries instead (15) gives 0.67 per 10 songs, which passes. Both decisions rejected that count because it includes songs nobody sang. The verdict does not depend on it: latency fails by 8.7 s either way.
- Only 2 of the 13 hits were at or under 2.5 s.
- The decision record is ADR-016 (`adrs/016-adr-009-minimum-implemented.md`), committed before any code (e9573cc).

## 3. Before and after, every fixture
Replay mode (what CI runs), debug build. Full mode (Whisper Tiny, `--shadow`) was also run on the head by the finisher (10 min 6 s) and by verifier A (about 75 min on a loaded PC): it printed the same table, cell for cell. "Before" was re-measured on a `git archive` export of 733c7e1 by the finisher and by verifier A, and matched both decisions.

Cells: slides / gotos / hits / missed, precision, recall, median and p95 latency (ms), false and double advances, worst blank and unblank latency (ms). `-` means the fixture has no such row; `miss` means Voxxa never did it.

| fixture | when | slides/gotos/hits/missed | prec | recall | median | p95 | false | double | blank | unblank |
|---|---|---|---|---|---|---|---|---|---|---|
| jump-be-thou-my-vision | before | 4/1/1/3 | 1.00 | 0.25 | 900 | 900 | 0 | 0 | - | - |
| | after | 4/4/4/0 | 1.00 | 1.00 | **1200** | **5850** | 0 | 0 | - | - |
| prayer-before-first-song (new in Q6) | before¹ | 2/1/1/1 | 1.00 | 0.50 | 6710 | 6710 | 0 | 0 | miss | miss |
| | after | 2/2/2/0 | 1.00 | 1.00 | 1710 | 2800 | 0 | 0 | 6800 | 1710 |
| prayer-between-songs | before | 4/4/4/0 | 1.00 | 1.00 | 8490 | 11230 | 0 | 0 | 7280 | 8490 |
| | after | 4/4/4/0 | 1.00 | 1.00 | 2490 | 4870 | 0 | 0 | 7280 | 2490 |
| prayer-then-reprise | before | 3/3/3/0 | 1.00 | 1.00 | 2900 | 8900 | 0 | 0 | 200 | 1500 |
| | after | 3/3/3/0 | 1.00 | 1.00 | 2900 | 2900 | 0 | 0 | 200 | 1500 |
| prayer-without-pause | before | 2/2/2/0 | 1.00 | 1.00 | 2900 | 8900 | 0 | 0 | 6600 | - |
| | after | 2/2/2/0 | 1.00 | 1.00 | 1900 | 2900 | 0 | 0 | 6600 | - |
| refrain-it-is-well | before | 6/4/3/3 | 0.75 | 0.50 | 4120 | 10310 | 1 | 0 | - | - |
| | after | 6/6/6/0 | 1.00 | 1.00 | 2120 | 3900 | 0 | 0 | - | - |
| **total** | before (5 fixtures) | 19/14/13/6 | 0.93 | 0.68 | 4120 | 11230 | 1 | 0 | 7280 | 8490 |
| | after (6 fixtures) | 21/21/21/0 | 1.00 | 1.00 | 2310 | 4870 | 0 | 0 | 7280 | 2490 |

¹ `prayer-before-first-song` is Q5-04's reproduction (#76), added as a fixture in Q6 (2a57b9f). Its "before" is its committed log replayed with the 733c7e1 conductor; it matches the Q5 ticket.

**What changed, in plain words.**
- **Repeated chorus (refrain-it-is-well):** before, the conductor could only step to the next slide, so it never went back to the refrain, and its one refrain goto landed while the next verse was sung (the false advance). After, it follows all six rows: 0/0, 0/1, 0/2, 0/1, 0/3, 0/1.
- **Jump (jump-be-thou-my-vision):** before, one goto and nothing after the skip. After, it follows the skip from verse 1 to verse 3, on to verse 4, and the return to verse 1.
- **Late advances (Q4-04, #62):** the old 40-word buffer held the previous verse; advances into a new verse took 8.9-11.2 s. The 12-word tracker cuts them to 1.2-4.9 s.
- **A song after a prayer (Q5-04, #76):** while the screen is blank, song detection now reads only the newest transcript. The song after the prayer is shown 2.49 s in (was 8.49 s), and a first song after an opening prayer is shown at all (was never).
- **Blank timing (Q4-03, #61):** unchanged, as intended (out of scope). 6.6-7.3 s into a prayer with no pause.

**The decision rule, after.**
- False advances: 0 over 7 sung songs = **0 per 10 songs: met** (was 1.67).
- p95 latency: **4.87 s over 21 hits: not met** (was 11.23 s; the limit is 2.5 s). On the five fixtures of the decision alone: 5.85 s over 19 hits. Median 2.31 s.
- Same kind of latency as before: stream time at file speed, no inference time, Q4-02 applies live.
- What bounds latency now is mostly the recogniser: at file speed the first transcript of a part arrives 1-3 s after it starts, because the stitcher holds 4 words. The two slowest hits are the jump's return to verse 1 (5,850 ms: Whisper Tiny gives no transcript for 6 s, then the window needs a second one) and the second verse of the first hymn in prayer-between-songs (4,870 ms: its first transcript opens every slide of that hymn).

## 4. Exit criterion: "eval not worse on any fixture" is NOT met
- **Worse:** `jump-be-thou-my-vision`, `median_ms` 900 → 1,200 and `p95_ms` 900 → 5,850. Every other fixture and the total are better or the same on every metric.
- **Why:** before, the fixture had a single hit (900 ms) at recall 0.25. That hit is unchanged. The three slides it used to miss are now hit, and latency only counts hits, so the new hits raise the median and p95.
- **No conductor can meet the old cells.** The recorded log sets a floor: the first hypothesis after verse 3 starts (truth 15,800 ms) comes at 17,000 ms, and the first after the return to verse 1 (truth 43,150 ms) comes at 47,000 ms. Any conductor that follows the jump has a latency of at least 1,200 ms on the first and at least 3,850 ms on the second, so `p95_ms` cannot be ≤ 1,000.
- **Effect on the gates:** the CI step "voxxa-eval (fixtures vs baseline)" exits 1 with exactly two lines, `FAIL jump-be-thou-my-vision: median_ms is 1200, baseline needs <= 1000` and `FAIL jump-be-thou-my-vision: p95_ms is 5850, baseline needs <= 1000`. Gate 4 (`cargo test`) fails two tests in `tests/voxxa_eval.rs` on the same two cells.
- **Not loosened.** User decision 4 says `fixtures/baseline.json` is tightened, never loosened, and no agent may grant an exception. The baseline still says 1,000 / 1,000 for that row.
- **The user's choice (ADR-016, second amendment, item 5):**
  - **(a), recommended:** a one-time exception to decision 4 that sets that row's `median_ms` to 1,500 and `p95_ms` to 6,000 (ADR-015's rule: the next multiple of 500 ms strictly above the measured worst). Checked on a scratch copy: the gate prints OK and `voxxa_eval` passes 26 of 26. The commit must give the floor above and say the old cells came from one hit at recall 0.25.
  - **(b):** keep `q6/base` out of `main`.

## 5. The baseline across Q6 (a second point for the user)
`git diff 77be270..4edd7f6 -- fixtures/baseline.json` (the state before Q6 against the head):
- **Per-fixture rows of the two Q5 fixtures:** only tightened (prayer-then-reprise p95 9,000 → 3,000; prayer-without-pause median 3,000 → 2,000 and p95 9,000 → 3,000).
- **New rows:** refrain-it-is-well, jump-be-thou-my-vision, prayer-between-songs, prayer-before-first-song.
- **The `total` row:** median 3,000 → 2,500 and p95 9,000 → 5,000 (tighter); precision, recall, false and double unchanged; **blank 7,000 → 7,500 and unblank 2,000 → 2,500 (looser).** The total covers every fixture, and the new prayer-between-songs fixture has a blank of 7,280 ms and an unblank of 2,490 ms. No old fixture got worse on blank or unblank.
- On the way, 733c7e1 (the fixtures commit) loosened seven `total` cells to admit the new fixtures under the old conductor; the conductor commit (2a57b9f) tightened five of them back. The fixture audit called this an explicit deviation from decision 4 caused by decision 2 (adding fixtures): both cannot hold literally for a `total` row. **The user should confirm the two looser `total` cells** together with the jump decision.

## 6. The fixtures' limits
- **All six fixtures are synthetic spoken audio, not singing.** Three come from Q6 (offline Windows text-to-speech, System.Speech, voices "Microsoft David Desktop" and "Microsoft Zira Desktop", 16 kHz mono 16-bit, written to WAV files only); three come from Q5's SAPI stand-ins and Q5-04's reproduction. One voice per part, no music, no room, no congregation. They catch regressions and show whether a change moves latency; they do not show accuracy in a church (Q5-01, #73).
- **The hymns and the prayer** are public-domain hymns (written before 1929) and a prayer written for Voxxa. One line of the Be Thou My Vision setlist follows a common hymnal variant whose pre-1929 print was not traced (ticket Q6-08).
- **Truth times are derived, not hand-marked:** a part's offset in the joined audio plus the synthesiser's first-word position (100 ms in every WAV). The fixture audit re-derived them three ways and checked them against the energy onset (101-137 ms) and the Whisper text inside each span.
- **The numbers were tuned on these fixtures** (ADR-016's decay, threshold and leading-words term), so they may not transfer to singing. Real recordings must re-check them.
- **Byte reproducibility** of the TTS output was checked on the reference PC only. Another Windows build or voice version may give different WAVs and truth times.
- **Latency is stream time at file speed** (see §1). Live latency on this PC would be higher (Q4-02).
- **Small sets:** 7 songs and 21 slides in all. "Per 10 songs" is extrapolated from that.
- **Slow in debug:** replay of all six fixtures takes about 3-4.5 min, full mode about 10 min, and the `voxxa_eval` test binary about 5 min (#77).
