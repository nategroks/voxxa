# Engineering Manager — church-autoadvance

Date: 2026-10-24 · Status: Proposed · Plan: `.10x/specs/2026-10-24-delivery-plan-and-prompts.md`

- 8 increments (0–7), 33 Claude Code prompts (9 S, 19 M, 5 L), each ≤ one session; ponytail on.
- Budget: ≈ 2.4–4.8M output tokens, ≈ 0.7–1.6B total tokens, ≈ $340–800 API list (≈ $500–1,200 with rework). Per day: part-time 1 prompt ≈ 75–150k output / 20–50M total; full-time 2–3 prompts ≈ 150–400k / 50–150M.
- Calendar: 22 weeks part-time, ≈ 12 weeks full-time; gated by pilots (weeks 3–8), signing/Store/Apple paperwork and founder-recorded fixtures, not by tokens.
- Sequencing rule: Inc 1 (measured loop) before any GUI work; adapters before scripture; bake-off only after the pilot corpus exists.
- Risks tracked in `status.md`; kill criterion in `decisions/cto/church-autoadvance.md`.

## V7 (2026-11-27): the wiring row, the `vad/` move and the four new rows (ADR-028 Accepted with amendment 1)

Lane `v7/10x` off `0190515` (`origin/main` after the Records PR #187). Roles: Engineering Manager (this record: the row decisions) + Staff Engineer (`decisions/staff-engineer/church-autoadvance.md`, "V7": the `trace` shape and the licence inventory). Phase 2 check first: ADR-028 exists with its draft amendment; FA-10, FA-54, FA-65, FA-74 and FA-77 are all answered; ADR-036 Decision 10 names the `position` row; nothing in `tests/`, `src-tauri/tests/`, `scripts/ci/` or `.github/workflows/` pins a cap (`ci.yml:156-157` pins tokei 15.0.0 and runs the script). Then Phase 4: `scripts/loc_budget.py` only; no Rust, no dependency, no workflow. Every number here is (measured, 2026-11-27, tokei 15.0.0, `tokei -o json` code lines at 0190515) unless marked.

**Before / after (`python scripts/loc_budget.py`, exit 0 both times, every row OK).**

| Row | Before V7 (used / cap, left) | After V7 (used / cap, left) | What changed |
|---|---|---|---|
| everything else in src-tauri/src | 2,313 / 2,317, 4 | 900 / 904, 4 | −1,288 (wiring) and −125 (`vad/`) on both sides; headroom unchanged to the line |
| wiring (`dispatch.rs` 147, `listen.rs` 159, `http_api/` 238, `commands/` 744) | — (inside "everything else") | 1,288 / 1,438, 150 | new (FA-10, option (a): measured + 150; no file moved) |
| recognizer (`transcription/` + `vad/`) | 432 / 700, 268 | 557 / 825, 268 | +125 on both sides (`vad/` moved in); headroom unchanged |
| onset (`aligner/onset/`) | — | 0 / 750, 750 | new; estimate 250 × 3 (FA-54); split from the follower row's future scope, so the conductor count now excludes it and the conductor cap stays 1,200 |
| input chain (`src-tauri/src/input/`) | — | 0 / 600, 600 | new; estimate 200 × 3 |
| position (`src-tauri/src/position/`) | — | 0 / 900, 900 | new; estimate 300 × 3 (ADR-036 Decision 10, FA-77) |
| health check (`src/health/`) | — | 0 / 450, 450 | new; estimate 150 × 3; subtracted from the GUI row like `src/simple/` |
| conductor, evidence, presenters, importers, voxxad, voxxa-eval, discovery, updater, GUI, Simple mode, per-driver | unchanged | unchanged | GUI 2,860 / 3,000 both times (the health row subtracts 0 today) |
| sum of caps | 13,517 | 16,367 | +150 (FA-10) + 2,700 (the four new rows); no existing cap raised; the two moves net to zero |

**Decisions taken in this step (Engineering Manager).**
1. **"Everything else" has 4 lines, not 1.** The handoff's 2,316 / 2,317 was Q10's finish; #184 (866b51e) rewrote part of `dispatch.rs` and the row measured 2,313 / 2,317 at 0190515. Recorded as measured; the ADR's "headroom stays 1 line" is corrected in amendment 1 § 3, not hidden. The 4 lines are spoken for by the `lib.rs` `mod` lines for `input` (V3b), `position` (V8c), `readiness` (V13s, if approved) and `practice` (V12); `onset`'s `mod` line is the conductor row's. The row holds `lib.rs` 148, `main.rs` 4, `audio/` 162, `diagnostics/` 141, `help/` 176, `hymnary/` 188, `net_stats/` 15, `tray/` 66.
2. **The wiring shares are ceilings, not allocations.** The prompt pack's table sums to 135; V8c (≤ 20, its own prompt) and V3b task 6 (≤ 15, added by me for the input hook and the `Trace` enum) were missing, so the ceilings sum to 170 against 150. Rule recorded in ADR-028 amendment 1 § 5: spent in merge order; a step starts only if its ceiling fits what `loc_budget.py` shows left, or records a smaller share with the logic moved into its own row behind one call line; FA-74 (b) does not apply to this row; the cap is never raised. V8c's Phase 2 check is the first to read it.
3. **No lab-scripts row** (closes the prompt pack § 4 line "EM decision at V7 or V3e"): `scripts/lab/` is lab tooling, uncounted like `scripts/qa/` and `scripts/release/`; V3e records its line count in its review; church-side code goes through `kit/tools/` (row 300) or a product row.
4. **The voxxad table** gains V3b's `Kind::Trace` share (≤ 4) beside V12 ≤ 20 (adds `--review`), V8b `--watch` ≤ 10, V13s `--check-ready` ≤ 8: active ≤ 42 of 48; V5c, V8 `--target-exe` and V13 `--check-layout` marked deferred with their steps, not pre-allocated.
5. **Presenters 1,396 and GUI 2,860** replace the handoff's 1,374 and 2,859 (#184 and the logo PR moved them); shares unchanged.
6. **Root `CLAUDE.md` § Budget rows** updated to the post-V7 numbers (its two bullets said "1 line left until V7 lands"); a stale instruction to every later lane was worse than a small out-of-list edit. Reported to the orchestrator.

**Not done, on purpose.** No `align` row (R5), no `readiness` row (the founder's own yes at V13s), no Rust, no docs under `docs/` (the `trace` line reaches the protocol docs with its first emitter, V3b). No ticket file: nothing found to ticket (no test pins a cap; the stale handoff numbers are fixed here).

**Exit check (plan V7 + prompt pack).** Budgets green locally (`loc_budget.py` exit 0; `py_compile` clean); wiring shows exactly 150 left; the recognizer row shows 268 left before and after the move; "everything else" shows the same headroom before and after (4; the plan's "1" was the older figure, recorded as such); the four new rows show 0 used; ADR-028 Accepted; ADR-001/003 and ADR-005 amendments committed as Accepted with FA-10/54/65/74/77 and FA-50 recorded.
