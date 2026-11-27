# Lab record, 2026-11-27: V3d break-gate arm, the head-word rule, before/after at both lags

Reported by the lab session (private lab, nothing committed); recorded by the orchestrator. Numbers only; take ids;
line numbers refer to the private lab files at this date. Every number is (measured) unless marked. The OLTW arm was
not built (see the end).

## Result in one line

The break gate alone is a **loss at the measured live lag** (2,150 ms; `reviews/2026-11-27-lab-r14-reanchor-lag.md`),
which would have triggered FA-63's flip-back rule. With a new rule, **a part is entered only on ≥ 2 of its own words**
(`HEAD_N` 2), the gate is a **win at both lags**. Lab defaults are now `BREAK_GATE` on and `HEAD_N` 2.

## What was changed in the lab (`parts.py`, by line)

| Line | Change |
|---|---|
| :43 | `BREAK_GATE` True, `BREAK_GRACE_MS` 1000, `BREAK_GAP_MS` 300, `BREAK_RATES` (1.5, 3.5) words/s |
| :44 | `HEAD_N` 2 |
| :139, :399 | a break list; `onset()` records every onset with a gap ≥ 300 ms before any early return |
| :383 | `_after_break(t, j)`: windows carry no word times, so the start of the jumped-to words is bracketed from their count (j + 1) at 1.5–3.5 words/s; one window suffices if a break lies in [t − (j+1)/1.5 s − 1 s, t − (j+1)/3.5 s + 1 s] |
| :369 | uses it as the CONFIRM count in the gate; `M_OFF` and the look-alike rule unchanged |
| :317, :323 | `HEAD_N`: on a branch path, a candidate inside the new part needs ≥ `HEAD_N` matched words in a head-only alignment |
| :465, :468 | `score_ids` adds `early_over_0_3s`, `within_0_5s`, `within_1_5s` (V3d's timing columns) |
| `test_parts.py` | 3 new tests: `break_gate_accepts_a_jump_on_one_window_after_a_break` (gate off: no move), `break_gate_ignores_a_break_long_before_the_new_words`, `a_new_part_is_not_entered_on_one_stray_word` (`HEAD_N` 0 moves, 2 does not); all 21 lab tests pass |

**Root cause the head-word rule fixes** (traced on T1 at `--lag-ms 2150`): an alignment running through the end of the
chorus picked up one stray word of verse 3's head. That counted as a "natural" part-end entry, so no gate applied, and
the slide moved 8.6 s early (139.1 s against truth 147.7 s). At lag 1,000 the pace reach hid it. The rule is a
correctness fix in its own right: a slide moves on words of that slide.

## Before / after (7 sung takes, live4 excluded; wrong moves / early > 1 s / wrong-slide seconds)

| Lag (ms) | Base (gate off, `HEAD_N` 0) | Gate only | `HEAD_N` 2 only | Gate + `HEAD_N` 2 |
|---|---|---|---|---|
| 1,000 (the offline assumption) | 4 / 0 / 161.0 | 4 / 0 / 152.0 | 4 / 0 / 162.0 | 4 / 0 / **153.0** |
| 2,150 (the measured live lag) | 6 / 1 / 235.8 | 8 / 1 / 229.8 (T1 wrong 2 → 4: the gate sped up a back-and-forth after the early move) | 4 / 0 / 223.3 | 4 / 0 / **214.3** |

- Gains: F2 −4.0 s and F3 −4.0 s at both lags; T1 at lag 2,150: 24.1 → 19.0 s, wrong 2 → 1, early 1 → 0.
- Cost: T1 at lag 1,000: +1.0 s (within 1.5 s 0.70 → 0.60).
- `HEAD_N` 3 is identical to 2.
- live4 (approximate truth, not citable) at lag 2,150: 49.6 → 43.0 s.
- Tried and dropped (code removed): skipping windows whose audio ended before the current position's time: 238.8 s at
  lag 2,150, worse than base.

## What the record changes

1. **ADR-031 amendment 4:** `PartsConfig.break_gate` ships **on by default only together with
   `PartsConfig.head_min_words` = 2** (default on). FA-63's "on by default" stands; its flip-back clause is satisfied by
   the combination's win at both lags, and the gate alone is recorded as the loss it is.
2. **V3d:** the before/after table is reported at `--lag-ms 1000` **and** `2150`. The lag-1,000 numbers hid both the
   head-word flaw and the gate's T1 loss.
3. **V11:** the port carries `head_min_words` beside `break_gate`; the three new lab tests join the port's test list.

## Not done: the OLTW arm

It needs a prior take of the same song as its reference. F1–F4 are re-cuts of T1, so T1 as reference would be circular;
only live4 (T1 reference) and T2 (T1 reference) are fair pairs, too few to decide R5. Estimate: about 150 lab lines
(log-band features, online DTW within the current part, firing only when armed), half a day. **Decision (orchestrator,
2026-11-27): build it after the founder's FA-45 solo takes give fair reference pairs**, unless the founder asks for it
sooner. R5 and ADR-033 are unchanged.
