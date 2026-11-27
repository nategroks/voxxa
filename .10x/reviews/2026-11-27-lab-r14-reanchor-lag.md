# Lab record, 2026-11-27: R14 fixes, the re-anchor hook, the capture probe, and the live read lag

Reported by the lab session (private lab, nothing committed); recorded by the orchestrator. Numbers only; take ids;
line numbers refer to the private lab files at this date. Every number is (measured) unless marked.

## What was done in the lab

| File | Change (line) |
|---|---|
| `parts.py` | :42 `MANUAL_GRACE_MS` 2000 and `CUT_SPAN_MS` 6000; :134 grace / last-window / cut state; :183 `reanchor(t_known, slide, t)` (re-derives the pointer with `_enter` when the part changes, logs an act `why='manual'`, sets the position to (slide, 0), clears pending, boundary and review, starts the grace and the cut); :200 `_after_cut` (windows carry no word times, so "drop words stamped before t" is approximated: for one window span of 6 s, whatever a later window shares with the last window heard before the move, Smith-Waterman score ≥ 2, is cut from its front); :278 cut and grace applied in `window()`; :377 grace applied in `onset()`; :353 relabel to `jump` **before** the gate block (planned-next from mid-part and skips inside a part now need `M_OFF` and 2 windows; the old post-gate relabel removed); :166 review rewind restores the pointer only when the jump leaves the shown part; :366 new rule: a `words` move inside the guessed part ends the review (the guess is confirmed); :411 `score_ids` counts `manual` acts for wrong-slide seconds only; :25 docstring. |
| `live.py` | :323 passes voxxad's `t` as the window's audio time (clamped to now) and the read time as `t_known`; :279 stores `vx_t_ms` in acts; :263 writes `windows.jsonl` with `t` and `t_known` (numbers only) so replays can use real read times; :319 mic queue depth; :334 "lag ms" and "mic queue max" on the 5 s status line; :348 `windows`, `mic_queue_max_blocks`, `read_lag_ms` median / p95 in `run.json`. |
| `evalparts.py` | :36 / :62 `--lag-ms` (latency only). `--pass-ms` also drops windows, which is the wrong model for live: there the cadence stayed at 1/s and only the lag grew. |
| `test_parts.py` (new) | 12 tests, made-up words: the two named in ADR-031 amendment 3 plus `planned_next_at_part_end_moves_on_one_window`, `confirmed_guess_keeps_the_plan_pointer_when_the_part_is_left_early`, `reanchor_grace_blocks_onsets_and_words`, `reanchor_ends_review_and_boundary`, `score_ids_counts_manual_moves_only_as_screen_time`, and the baseline set. |
| `test_onset.py` (new) | 6 tests: hum alone, a swelling hum, a voice control, the 1 s median floor, no onset from warm-up, digital silence. |
| `capture_probe.py` (new; public-safe, moves to `scripts/lab/` in V8c's PR) | 150 lines, PEP 723, stdlib plus GDI through ctypes: StretchBlt HALFTONE to 132 × 32, 4 × 4 box means, 33 × 8 dHash, τ = 3. Flags `--list`, `--monitor N --region`, `--keys`, `--selftest`. `--keys` polls `GetAsyncKeyState` for the eight navigation codes only (not a hook), opt-in, timestamps only. |

All 18 tests pass. Against the pre-fix `parts.py`, the 7 tests of new behaviour fail and the 5 baseline tests pass.
The test recording folder was deleted afterwards. A lab backup exists in the lab session's scratchpad.

## The review-rewind defect the tests caught (why rule :366 exists)

Plan V1 C V2 C: after a confirmed guess of V2, leaving the part early for the chorus rewound the plan pointer to 1
instead of 3. The new rule ends the review when a `words` move lands inside the guessed part, so the pointer stays.

## Numbers (replay, numbers only)

| Measure | Before | After | Note |
|---|---|---|---|
| Fixes alone, T1 / T2 / F1–F4 / live4 | — | unchanged | |
| Fixes alone, P1 wrong-slide seconds | 66.0 | 67.0 | skips inside a part now wait for a second window |
| Live read lag (dry-run T1 replay through `live.py`, GPU live build, 47 windows at 1/s) | assumed 1,000 ms | **median 2,148 ms, p95 2,183 ms** | mic queue max 0, 0 dropouts |
| Offline optimism | — | ≈ 1.15 s | replays assume `known = t + 1,000`; the top of ADR-031 amendment 1's 0.5–1.2 s band |
| Parts map, wrong-slide seconds over the six sung takes, `--lag-ms 2150` | 94.0 | 160.4 | wrong moves: T1 1 → 2, F3 1 → 2 |
| Plan-order follower at the same lag | — | 322.4 | |
| live4 (approximate truth, not citable) | 17.6 | 49.6 s, 3 early | |
| Capture probe self-test (synthetic blank-text renders) | — | 9 distinct states; min pair 36 of 256 bits; transition 500 ms; key → change 120 ms; key → settled 620 ms | |
| Capture probe on this PC | — | `--list` 3 monitors; 3 s capture of the primary monitor held 10 Hz; still screen ≤ 3 bits | no frame saved |

**Where the lag is (inferred):** the pass itself was about 0.5 s in the earlier lab, so about 1.6 s sits in the hop
wait, the stdin pipe, the log flush and the read. This is the largest single delay measured so far. It becomes
**V3b task 8, the lag-breakdown row** (`specs/2026-11-26-prompts-round-3.md`).

**Baseline kept in the lab for V3b task 8:** `<lab>aselines6-11-27-dryrun-T1-read-lag.json` (numbers only: the run's
numbers, the nine 5 s lag readings 2,091–2,177 ms, build and environment, the 1/s window cadence, the first window at
about 28 s of audio, the gain-22 clipping caveat). The run folder itself was deleted.

**Replay note:** `live.py` applies `--gain-db 22` to an already-gained recording (151k clipped samples); replays of
such recordings pass `--gain-db 0`.

## Not done (outside the agreed list)

The break-gate arm (FA-63, on by default) and the OLTW arm (V3d task 3); `live.py --parts/--freeshow` (V3e's edit
list); the spoken-aside, two-singer and full-mix takes (founder).
