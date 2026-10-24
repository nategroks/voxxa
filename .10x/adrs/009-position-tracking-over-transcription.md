# ADR-009: Track position in a song-structure graph with an evidence accumulator instead of matching transcripts to the next slide

**Status:** Proposed · **Date:** 2026-10-24 · **Feature:** church-autoadvance · **Reversibility:** Easy (contained in `aligner`) · **Owner:** nategroks

## Context
The current aligner compares a 40-word transcript buffer to the current and next slide only, by character LCS, and advances on a threshold. Worship songs repeat choruses and jump to bridges; recognizers drop and hallucinate words on singing (research E2); a 5 s batch adds latency; and there is no measurement. Published real-time lyric alignment treats the problem as tracking a position against known text with past-only context (E3).

## Decision
- Represent the setlist as a graph: songs → sections (verse, chorus, bridge, tag) → slides. Candidate positions are slides; transitions carry configurable priors (stay, next, any section start in the song, next song, other).
- Score each candidate per recognizer hop with word-token similarity (normalised; the existing LCS ratio reused on tokens) against the candidate's leading words and the previous slide's trailing words; accumulate evidence with exponential decay.
- Decide at most once per hop: move when best ≠ current, evidence ≥ threshold, evidence ≥ current + margin, and dwell ≥ `min_dwell_ms`. Emit `goto(song, slide)` rather than `advance` so repeats and jumps are first-class; adapters without `goto` receive the equivalent number of `advance`/`prev` presses only when the move is ±1, otherwise they go red.
- Re-sync from the adapter's `current()` whenever it disagrees for two reads; the operator always wins.
- Every decision is logged with its scores; `voxxa-eval` scores precision, recall, latency and false advances against operator truth; CI blocks regressions.

## Alternatives
| Alternative | Why not |
|---|---|
| Keep next-slide-only matching, tune thresholds | Cannot follow a repeated chorus or a bridge; the research plan's O3 says one wrong advance ends the trial. |
| Full OLTW/DTW alignment on acoustic features (Park 2024) | Best published accuracy but no library, heavy to build; keep as the escalation path if all recognizers fail the false-advance budget. |
| LLM-based "where are we" over transcripts | Latency and cost on-device; non-deterministic; unmeasurable in CI. |

## Consequences
Good: repeats and jumps handled; decisions explainable from the log; thresholds tunable per room via calibration. Bad: more state than today (~400 LOC instead of ~200); mitigated by property tests on the graph and the eval harness. Risks: priors tuned on one church may not transfer; calibration step in Setup & Calibration exists for that.

## Review trigger
Re-open if the pilot corpus shows false advances > 1 per 10-song set with the best recognizer; then escalate to acoustic alignment.
