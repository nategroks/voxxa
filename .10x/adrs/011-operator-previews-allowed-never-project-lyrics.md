# ADR-011: Voxxa never projects lyrics; operator-only previews on the booth machine are allowed (amends ADR-007)

**Status:** Accepted · **Date:** 2026-11-11 · **Feature:** church-autoadvance · **Reversibility:** Easy · **Owner:** nategroks · **Amends:** ADR-007 (the "never displays lyrics" bullet only)

## Context
ADR-007 says "Voxxa **never displays lyrics**; it only emits slide positions." Q0 found that the code has never matched that sentence:
- The base operator window shows the current and next slide text (base `src/main.js:14-15`).
- The May branch, adopted by ADR-010, keeps that and adds a hidden stage window (`src/stage.js:37-38`) that the operator toggles.

The CCLI licence concern behind ADR-007 is projection and distribution. A preview the operator sees on the church's own machine, from text the church's own licensed software supplied, is neither of those.

## Decision
- **Voxxa never projects lyrics.** It never shows them on an audience-facing output, and the projected display stays with the presentation app.
- **Operator-only previews on the booth machine are allowed.** This covers the operator window and the stage window, using text read from the church's own sources.
- Every other ADR-007 bullet is unchanged: Voxxa ships no lyrics, ships no copyrighted Bible text, matches scripture against public-domain texts, and writes `usage.csv` locally.
- The README and website say "never projects lyrics", not "never shows a word on screen" (ADR-010 amendment A11).

## Alternatives
| Alternative | Why not |
|---|---|
| Keep "never displays" and ship the stage window disabled | The operator window already shows slide text, so the sentence would stay false. |
| Remove the stage display | Same problem, and it deletes a feature against ADR-010 Decision 4. Cuts are decided after Q2. |

## Consequences
Good: the ADR now matches the code, and the licensing line sits where the risk is (projection). Bad: if the stage window is moved to a second monitor, it could be audience-facing. Q2's security and QA review checks that the stage window cannot target an output display by default.

## Review trigger
Re-open if CCLI's written answer (business strategy R3) treats operator previews as display needing a licence.
