# ADR-028: Line budgets for the follower increment: a recorded "wiring" row ends the "everything else" wall; new code lands in new rows; no existing cap is raised

**Status:** Proposed (2026-11-24) · **Date:** 2026-11-24 · **Feature:** church-autoadvance · **Reversibility:** Easy (caps in one script) · **Owner:** nategroks · **Author:** 10x-Team synthesiser (Engineering Manager + Staff Engineer), from review 21 § 4.3 and drafts 10–14

Follows the precedent of ADR-019 § 10 (discovery row) and ADR-020 § 5 (updater row).

## Context
- Budgets are tokei code lines per area and "can only go down" (`scripts/loc_budget.py:6`). "Everything else in src-tauri/src" is everything minus the listed rows (`scripts/loc_budget.py:81`) and sits at **2,316 / 2,317, 1 line left** (`.10x/handoff.md:102`, `:111`).
- That row holds `lib.rs` (every `mod` line and `generate_handler!`), `commands/`, `http_api/`, `listen.rs`, `dispatch.rs`, `tray/`, `audio/`, `vad/`, `help/` (review 21 § 4.3). Every step in this plan touches at least one of them: dispatch safety (ADR-025), HTTP routes and hotkeys (ADR-024), session-log location (ADR-023), the follower's loop wiring (ADR-022), module lines for new rows.
- Other rows today (`.10x/handoff.md:109-122`): conductor 1,173/1,200; evidence 215/500; recognizer 432/700; voxxad 552/600; voxxa-eval 488/600; presenters 1,374/2,000 (`keystroke.rs` 396/400); importers + planning_center 1,386/1,600; discovery 223/250; updater 98/150; GUI 2,859/3,000; Simple mode 450/600.
- The strategy bounds the core engine at ≤ 3k Rust lines (`.10x/specs/2026-10-24-software-strategy.md:30`).
- The four 10x drafts asked for about +3,450 Rust, +2,050 GUI and +1,800 Python lines in new rows; the ponytail lane about +800 Rust (review 21 § 2.1).

## Decision
1. **Wiring row (one founder-approved increase of 150 lines):** move the count of `dispatch.rs`, `listen.rs`, `http_api/` and `commands/` out of "everything else" into a new recorded row "wiring (dispatch.rs, listen.rs, http_api/, commands/)" with cap = their measured size at the split + 150; lower the "everything else" cap by exactly the measured size, so its own headroom stays 1 line. No file moves; only `loc_budget.py` changes. The ADR records the measured numbers at merge.
2. **New rows land with their code, at these caps (never raised later):**

| Row | Path | Language | Cap | Owner | Step |
|---|---|---|---|---|---|
| follower | `src-tauri/src/aligner/follower/` (or inside evidence if the port ≤ 250 lines) | Rust | 1.3 × measured port, ≤ 500 | Sr Eng 1 | V11 |
| practice | `src-tauri/src/practice/` | Rust | 300 | Sr Eng 1 | V12 |
| layout | `src-tauri/src/layout/` (validator, compiler, trigger engine, hotkeys) | Rust | 300 | Sr Eng 2 | V13 |
| Run screen | `src/run/*` (subtracted from the GUI row like Simple mode) | JS/HTML/CSS | 350 | UI consultant | V14 |
| kit tools | `kit/tools/` | Python (tokei) | 300 | Sr Eng 3 | V5 |

3. **Existing rows absorb only small, stated changes; new logic goes in the new rows** (practice row 300, layout row 300). A step that would exceed its stated share stops and reports. Every share:
   - **voxxad** (552/600, 48 lines left) holds only flag parsing and wiring:

     | Step | Change | Lines |
     |---|---|---|
     | V5c | `--version` | ≤ 3 |
     | V8 | `--target-exe` | ≤ 5 |
     | V12 | `--learn` / `--confirm` / `--out` / `--practice-dir` parsing and dispatch into the practice module (logic in the practice row) | ≤ 20 |
     | V13 | `--check-layout` parsing and dispatch into the layout module (logic in the layout row) | ≤ 10 |

     Total ≤ 38, with 10 spare.
   - **conductor** (1,173/1,200, 27 left): V11 follower routing ≤ 20 plus V5 step 9's `Slide.kind` ≤ 3, so ≤ 23.
   - **importers + planning_center** (1,386/1,600): V5 step 9 (section labels and the credit kind in the `.txt` importer) ≤ 40.
   - **voxxa-eval** ≤ 100 (metrics v2); **recognizer** ≤ 100 (V3 decoder settings ≤ 60, V4 GPU flag ≤ 40); **Simple mode** ≤ 65 (V9 help-bundle default ≤ 5, V14 status rows ≤ 60).
4. **Core-engine check:** conductor 1,200 + evidence 500 + follower ≤ 500 + practice 300 = ≤ 2,500 in caps (≤ 3,000 strategy bound). Layout is integration, not core.
5. **Python counting:** `loc_budget.py` extends its per-file languages with Python only for `kit/tools/` (draft 12 § 6.4).

## Alternatives Considered
| Alternative | Pros | Cons | Why Not |
|---|---|---|---|
| A refactor PR that frees lines in "everything else" first | no cap increase at all | churns `dispatch.rs`/`listen.rs` right before the follower wires into them; unknown yield | slower and riskier than a recorded split |
| Raise "everything else" | simplest | breaks the "caps only go down" rule | never |
| The drafts' full row set (follower 700, practice 400, store 600, library CLI 300, library editor 700, Prepare 1,000, bindings 400, cue engine 450, lab 1,500 …) | room for everything | +42% Rust, +62% GUI; hits the 3k core bound in one increment | cut per review 21 § 7 |
| Put each change in whichever row has room | no ADR needed | hides growth; reviewers cannot see what was spent where | rows must say what they hold |

## Consequences
### Positive
- Every planned step can merge without a surprise budget stop.
- The next increment adds about +1,250 Rust (follower ≤ 500, practice 300, layout 300, wiring +150), +350 GUI and +300 Python in new caps, plus about 300 Rust lines inside existing rows' headroom (conductor 23, voxxad 38, importers 40, voxxa-eval 100, recognizer 100) and 65 GUI lines in Simple mode.
### Negative
- One real increase (150 lines) the founder must approve explicitly.
### Risks
- The follower port's size is unknown until V6 measures the prototype; the 500 ceiling may force a smaller port or a second ADR.

## Dependencies
- Depends on: ADR-010 Decision 3 (budgets), ADR-019 § 10, ADR-020 § 5.
- Constrains: ADR-022, ADR-023, ADR-024, ADR-025.
- Plan steps: V7 (row split), then each step's own row.

## Review trigger
Any row reaches 95% of its cap; the follower port measures > 500 lines; a church needs the deferred library editor or bindings executor.

## Amendment 1 (draft, 2026-11-27, founder answers FA-10, FA-54 and FA-65)

**What the founder decided.** FA-10 (2026-11-26): approved, option (a), the one 150-line wiring row. FA-54 (2026-11-27): yes to the new rows, with the rule "take the number you need and multiply times 3", and the direction that this be done comprehensively and exceedingly well. FA-65 (2026-11-27): yes, one `trace` line type.

**What changes in this ADR (applied in V7).**
1. Decision 1 stands as written: wiring row = measured size + 150; "everything else" lowered by the measured size, headroom 1 line.
2. Decision 2's table gains three rows, each at three times the engineers' estimate, with both numbers recorded:

| Row | Path | Language | Estimate | Cap (× 3) | Owner | Step |
|---|---|---|---|---|---|---|
| onset | `src-tauri/src/aligner/onset/` (detector, arm/fire rules, break event; split from the follower row) | Rust | 250 | 750 | Sr Eng 1 | V11 (port); planned in V3b |
| input chain | `src-tauri/src/input/` (high-pass, gain, level and hum numbers, `degraded` flag) | Rust | 200 | 600 | V3b's senior engineer | V3b task 6 |
| health check | `src/health/` (subtracted from the GUI row like Simple mode) | JS/HTML/CSS | 150 | 450 | UI consultant | V13s |

   The rows start at 0 lines in V7; code lands with its step. Caps still only go down once set.
3. `vad/` moves into the recognizer row: that cap rises by exactly `vad/`'s measured size and "everything else" falls by the same, so both headrooms are unchanged to the line (a move, not an increase).
4. `align` (`src-tauri/src/align/`) is named but not added: it lands only on an R5 pass (V3f bar met and V6 reported), and then at 1,200 (the 400 estimate × 3 under the same rule).
5. The `readiness` row proposed by ADR-034 § 11 is a separate founder yes at V13s; if given, it lands at 900 (300 × 3).
6. The Decision-2 rows proposed before FA-54 was asked (follower ≤ 500, practice 300, layout 300, Run 350, kit tools 300) keep their caps (FA-74 answered 2026-11-27, option (b): a step whose measured code cannot fit stops at its Phase 2 check and asks for 3 × the measured number for that row; the 3,000-line core bound stays). **Decision 4 restated:** onset and the input chain are the audio front end, counted outside the core sum the way layout is integration, so 2,500 ≤ 3,000 still holds; extending × 3 to follower and practice would put the core sum at 4,100, which is FA-74's question.
7. The `trace` line (FA-65) is ADR-001/003's amendment, written in V7 task 6: one line type `trace` with a required `kind` field (`follow`, `verify`, `screen`, `watchdog`, `operator`, `level`, `cue`), numbers-only fields per kind, replay-skipped like `window`; no `follow`, `cue` or read-back line type is ever added; a `kind` that carries text fails `events.rs`'s serialisation test.
8. Decision 3's voxxad share table is updated as `plan-round3.md` V7 task 7 says (V12 ≤ 20, V8b `--watch` ≤ 10, V13s `--check-ready` ≤ 8; V5c, V8 `--target-exe` and V13 `--check-layout` deferred with their steps), and the wiring-row share table is added.

**Why "× 3" is recorded as headroom, not as a target.** The estimate stays beside the cap so a reviewer can see when a row runs far past what it was expected to need.

**Review trigger added:** a tripled row passes its original estimate (onset 250, input chain 200, health check 150): the step reports what the extra lines hold before merging. The existing 95 % trigger stands.

**Founder's quality bar for V7 (FA-54).** The amendment lists every row with path, language, estimate, cap, owner and step, the measured numbers at merge, and the reason for each cap; nothing is left "to be decided later".
