# ADR-028: Line budgets for the follower increment: a recorded "wiring" row ends the "everything else" wall; new code lands in new rows; no existing cap is raised

**Status:** Accepted (2026-11-27; FA-10 approved 2026-11-26, FA-54 and FA-65 answered 2026-11-27, FA-74 and FA-77 answered 2026-11-27) · **Date:** 2026-11-24 (Proposed); 2026-11-27 (Accepted with amendment 1, V7) · **Feature:** church-autoadvance · **Reversibility:** Easy (caps in one script) · **Owner:** nategroks · **Author:** 10x-Team synthesiser (Engineering Manager + Staff Engineer), from review 21 § 4.3 and drafts 10–14

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
   - **voxxad** (552/600, 48 lines left; measured again at 0190515 in V7, unchanged) holds only flag parsing and wiring. Table updated in place by amendment 1 (2026-11-27, prompt pack V7 task 7):

     | Step | Change | Lines |
     |---|---|---|
     | V12 | `--learn` / `--confirm` / `--out` / `--practice-dir` parsing and dispatch into the practice module (logic in the practice row); amendment 1 adds `--review` (the pack's V12 line also names `--status` and `--reason`, inside the same share) | ≤ 20 |
     | V8b | `--watch` (the read-only FreeShow watch helper, ADR-032) | ≤ 10 |
     | V13s | `--check-ready` parsing and dispatch into the readiness module (logic in the readiness row, if the founder approves it) | ≤ 8 |
     | V3b task 6 | `Kind::Trace`, the recorder arm and the replay skip in `events.rs` (ADR-003 amendment 1; once, for every later kind) | ≤ 4 |
     | V5c | `--version` | ≤ 3 — **deferred** with V5c (R17: behind the first autonomous Sunday) |
     | V8 | `--target-exe` | ≤ 5 — **deferred** with V8 steps 2–4 (V8 is reduced to V8-lite) |
     | V13 | `--check-layout` parsing and dispatch into the layout module | ≤ 10 — **deferred** with V13b (the layout engine) |

     Active shares ≤ 42, with 6 spare. The deferred shares (≤ 18) come back with their steps and must fit what is left then; none is pre-allocated.
   - **conductor** (1,173/1,200, 27 left): V11 follower routing ≤ 20 plus V5 step 9's `Slide.kind` ≤ 3, so ≤ 23.
   - **importers + planning_center** (1,386/1,600): V5 step 9 (section labels and the credit kind in the `.txt` importer) ≤ 40.
   - **voxxa-eval** ≤ 100 (metrics v2); **recognizer** ≤ 100 (V3 decoder settings ≤ 60, V4 GPU flag ≤ 40); **Simple mode** ≤ 65 (V9 help-bundle default ≤ 5, V14 status rows ≤ 60).
4. **Core-engine check:** conductor 1,200 + evidence 500 + follower ≤ 500 + practice 300 = ≤ 2,500 in caps (≤ 3,000 strategy bound). Layout is integration, not core. (Restated by amendment 1 § 6: onset, input chain and position are outside the sum too; FA-74 (b) keeps the earlier rows' caps.)
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
Any row reaches 95% of its cap; the follower port measures > 500 lines (**reached 2026-11-27: the V3d port plan estimates about 860 at a measured 2.37 × factor; FA-78 open**); a church needs the deferred library editor or bindings executor. Added by amendment 1: a tripled row passes its original estimate (onset 250, input chain 200, position 300, health check 150).

## Amendment 1 (Accepted 2026-11-27, V7; founder answers FA-10, FA-54, FA-65, FA-74 and FA-77)

**Status of this amendment: Accepted** (V7, 2026-11-27). Drafted the same morning from the founder's answers; filled with the measured numbers and accepted by V7's 10x lane (Engineering Manager + Staff Engineer) on `v7/10x`. Unless a cell says otherwise, every number below is **(measured, V7 10x lane, 2026-11-27, tokei 15.0.0, `tokei -o json` code lines at `0190515` = `origin/main` after the Records PR #187, read back with `python scripts/loc_budget.py` before and after the change)**. The orchestrator's independent count the same day gave the same wiring size (147 + 159 + 238 + 744 = 1,288) and the same `vad/` size (125); the prompt pack's older `vad/` figure of 189 (cited at `a5f7e2e`) is superseded by the measurement.

**What the founder decided.** FA-10 (2026-11-26): approved, option (a), the one 150-line wiring row. FA-54 (2026-11-27): yes to the new rows, with the rule "take the number you need and multiply times 3", and the direction that this be done comprehensively and exceedingly well. FA-65 (2026-11-27): yes, option (a), one `trace` line type. FA-74 (2026-11-27): option (b), the × 3 rule covers FA-54's rows now; the five earlier rows keep their caps and ask for 3 × the measured number only on a measured overrun. FA-77 (2026-11-27): position sources in the order hashes 1, navigation-key hook 2, APIs 3, with ADR-036 Decision 10's `position` row.

### 1. The wiring row (Decision 1; FA-10)

| File | Code lines at 0190515 |
|---|---|
| `src-tauri/src/dispatch.rs` | 147 |
| `src-tauri/src/listen.rs` | 159 |
| `src-tauri/src/http_api/` (one file, `mod.rs`) | 238 |
| `src-tauri/src/commands/` (one file, `mod.rs`) | 744 |
| **measured size** | **1,288** |
| **cap** (1,288 + 150) | **1,438**; 150 left at merge |

Row label in `scripts/loc_budget.py`: `wiring (dispatch.rs, listen.rs, http_api/, commands/)`; language Rust; owner the Engineering Manager (the row belongs to no single step: each step spends a stated share, § 5); step V7. No file moved. Reason for the headroom: the 150 lines are the founder's one approved increase, spent only through the shares in § 5.

### 2. The `vad/` move (item 3; a move, not an increase)

`src-tauri/src/vad/` (one file, `mod.rs`): **125** code lines. It leaves "everything else" and joins the recognizer row (`transcription/` + `vad/`) under the same rule as the wiring split: the receiving row's cap rises by exactly the moved size and the giving row's cap falls by the same, so neither headroom changes.

| Row | Before (used / cap, left) | After (used / cap, left) |
|---|---|---|
| recognizer (`transcription/`; from V7 `transcription/` + `vad/`) | 432 / 700, **268** | 557 / 825, **268** |
| everything else in `src-tauri/src` | 2,313 / 2,317, **4** | 900 / 904, **4** (2,317 − 1,288 wiring − 125 `vad/` = 904) |

No row can hold one more line than it could before. V3b's energy gate (≤ 40 lines, ADR-030 amendment 1) lands in `vad/` inside the recognizer row's share (§ 5).

### 3. "Everything else" after V7: what it holds and why it has 4 lines

Decision 1 said "headroom stays 1 line": that was Q10's figure (2,316 / 2,317 at f424c74, `.10x/handoff.md`). At 0190515 the row measures 2,313 / 2,317: #184 (866b51e, 2026-11-24) rewrote part of `dispatch.rs` and gave 3 lines back. After V7 the row is 900 / 904 and holds `lib.rs` 148, `main.rs` 4, `audio/` 162, `diagnostics/` 141, `help/` 176, `hymnary/` 188, `net_stats/` 15, `tray/` 66. The 4 lines are spoken for by the `mod` lines new modules need in `lib.rs` (18 `mod` lines today): `input` (V3b task 6), `position` (V8c), `readiness` (V13s, if the founder says yes to ADR-034 § 11), `practice` (V12). `onset` takes its `mod` line in `aligner/mod.rs` (conductor row, § 5). Any other change to these eight files needs lines freed in them first; `layout` (V13b, deferred) would need its `mod` line freed too.

### 4. The new rows (Decision 2, item 2; FA-54's rule; ADR-036 Decision 10 and FA-77 for `position`)

| Row (label in `loc_budget.py`) | Path | Language | Estimate | Cap (× 3) | Used at merge | Owner | Step | Where the estimate comes from |
|---|---|---|---|---|---|---|---|---|
| `onset (src-tauri/src/aligner/onset/)` | `src-tauri/src/aligner/onset/` (detector, arm/fire rules, break event; ADR-030) | Rust | 250 | **750** | 0 | Sr Eng 1 | V11 (the port); planned by V3b from the lab's port plan | the lab onset detector plus the arm/fire rules as counted in the port plans (cited, `round2/47` § 3). Split from the follower row's future scope: the conductor row's cap stays 1,200 and, from V7, its count excludes `aligner/onset/` the way it excludes `aligner/evidence/` |
| `input chain (src-tauri/src/input/)` | `src-tauri/src/input/` (100 Hz high-pass, gain, level and hum numbers, the `degraded` flag; ADR-030 amendment 1) | Rust | 200 | **600** | 0 | V3b's senior engineer | V3b task 6 | one biquad, one gain, four hum bins and the `level` line's struct, as sized in the V3b prompt (cited) |
| `position (src-tauri/src/position/)` | `src-tauri/src/position/` (`PositionSource`; `screen` 256-bit text-region hashes with settle and attribution rules; `keys` scoped navigation hint; `api` read-back adapters as data; ADR-036 revision 1) | Rust | 300 | **900** | 0 | V8c's principal architect + senior engineer | V8c (after task 0, the booth capture probe) | ADR-036 Decision 10 (FA-77): a hash of about 30 lines plus the OS capture Tauri already links, the interface and the three sources; no OCR, no model, no new service |
| `health check (src/health/)` | `src/health/` (the Sunday health page; subtracted from the GUI row the way `src/simple/` is) | JS/HTML/CSS | 150 | **450** | 0 | UI consultant | V13s | six rows with one sentence each (ADR-034; `round2/44` § 2.2, cited) |

All four start at **0**: the paths do not exist at 0190515, and `loc_budget.py` sums the files that match a prefix, so an absent path counts 0 (the run below shows `0 / cap` for each). Code lands with its step. Caps only go down once set. The × 3 is headroom, not a target: the estimate stays beside the cap so a reviewer sees when a row runs far past what it was expected to need, and the review trigger fires when a row passes its estimate.

**Named, not added.** `align` (`src-tauri/src/align/`, cap 1,200 = 400 × 3) stays conditional on R5 (a V3f pass and V6's held-out numbers); the step that earns it adds the row and writes ADR-033. `readiness` (`src-tauri/src/readiness/`, 900 = 300 × 3, ADR-034 § 11) needs the founder's own yes at V13s's Phase 2 check. **Lab scripts (`scripts/lab/`): no row** (Engineering Manager, V7, closing the prompt pack § 4 line "a lab scripts budget row? EM decision at V7 or V3e"): they are lab tooling, uncounted like `scripts/qa/`, `scripts/release/` and the tests; V3e records their line count in its review as a number; anything that reaches a church goes through `kit/tools/` (its own row, 300) or a product row.

### 5. Decision 3's share tables as updated (item 8; prompt pack V7 task 7)

**voxxad** (552 / 600, 48 left): the table in Decision 3 above, updated in place. Active shares V12 ≤ 20 (adds `--review`), V8b `--watch` ≤ 10, V13s `--check-ready` ≤ 8, V3b `Kind::Trace` ≤ 4: ≤ 42, 6 spare. V5c `--version` (≤ 3), V8 `--target-exe` (≤ 5) and V13 `--check-layout` (≤ 10) are deferred with their steps (R17) and come back only inside what is left then.

**wiring** (1,288 / 1,438, 150 left). The prompt pack's "How to use" table, plus the two steps that table predates:

| Step | What the lines are | Ceiling |
|---|---|---|
| V8-lite | the `position_uncertain` flag, its setter and the per-key clear (ADR-025) | ≤ 20 |
| V8b | `select_presentation` before `presentation_is`; consuming the verify result (ADR-032) | ≤ 30 |
| V13a | the `holding` and `following_stopped` flags with one setter and one clearer each, the four routes, the hotkey registration call sites, the `operator` trace call | ≤ 40 |
| V9 | the shared session-folder helper and the one-time move (ADR-023) | ≤ 10 |
| V13s | the heartbeat publish calls in the loop | ≤ 15 |
| V11 | Tier A/B dispatch of `Action::Goto` (ADR-031 § 7) and the follower hand-off | ≤ 20 |
| *sum of the pack's table* | | *135* |
| V8c (its own prompt's line; the step was added after the table) | `PositionSource` owned and polled by dispatch, the `reanchor(slide, t)` call, the `position` config field | ≤ 20 |
| V3b task 6 (added by this amendment) | the `input` config field, the chain call in the loop, the per-second `level` emission, `Event::Trace` and the `Trace` enum (ADR-003 amendment 1) | ≤ 15 |
| *sum of all ceilings* | | *170* |

The ceilings sum to 170 against 150 left, so they are **ceilings, not allocations**. The row is spent in merge order: at its Phase 2 check a step reads the wiring row's "left" from `loc_budget.py` and starts only if its ceiling fits, or if its record states a smaller share and names what moved out of wiring into the step's own row behind one call line (logic never moves into "everything else"). A step that cannot fit stops and reports. FA-74 (b) does not apply to this row (it is not one of the five earlier rows); the cap is never raised. The real spend is expected under the ceilings: the pack's own reading of `dispatch.rs:99-180` sizes V8-lite's flag at about 8 lines (cited).

**The other rows' shares, measured at 0190515:** conductor 1,173 / 1,200 (27 left): V5a `Slide.kind` ≤ 3, V11 follower routing ≤ 20, V11 `pub mod onset;` 1, so ≤ 24. Recognizer 557 / 825 (268 left): V3 ≤ 60 (incl. `--model`), V4 ≤ 40, V3b energy gate in `vad/` ≤ 40, so 140. voxxa-eval 488 / 600 (112 left): V2 ≤ 105. Presenters 1,396 / 2,000 (604 left; `keystroke.rs` 396 / 400; the handoff's 1,374 was Q10's figure, #184 added 22 lines in `presenters/mod.rs` and `keystroke.rs`): V8b ≤ 150 (`freeshow.rs` ≤ 400 per file), V8c `read_back` loader ≤ 30. Importers + planning_center 1,386 / 1,600 (214 left): V5a ≤ 40. GUI 2,860 / 3,000 (140 left; 2,859 at Q10): V14 stage strip net ≤ +15. Simple mode 450 / 600 (150 left): V9 ≤ 5, V14 ≤ 60, V13s ≤ 10. Discovery 223 / 250 and updater 98 / 150: no share this round.

### 6. Decision 4 restated (the core-engine check; FA-74 (b))

Core sum in caps: conductor 1,200 + evidence 500 + follower ≤ 500 + practice 300 = **2,500 ≤ 3,000** (`.10x/specs/2026-10-24-software-strategy.md:30`). Outside the sum: onset 750 and input chain 600 (the audio front end), position 900 (a position source is an input to re-anchor, not the follower: ADR-036 Decision 1 says the follower never changes), wiring 1,438 (integration), layout 300 (integration), readiness 900 if approved (operations). The five rows proposed before FA-54 (follower ≤ 500, practice 300, layout 300, Run 350, kit tools 300) keep their caps; under FA-74 (b) a step whose measured code cannot fit stops at its Phase 2 check and asks for 3 × the measured number for that row only, and the 3,000-line bound stays (extending × 3 to follower and practice would make the core sum 4,100; the founder chose not to).

### 7. The `trace` line (item 7; FA-65)

Recorded in ADR-003 amendment 1 (the protocol ADR carries the text) with a cross-reference in ADR-001 amendment 1. Budget consequence: the `Trace` enum and `Event::Trace` in `listen.rs` are wiring (inside V3b's ceiling in § 5, then 1 line per later kind inside that step's share); `Kind::Trace`, the recorder arm and the replay skip in `events.rs` are voxxad (≤ 4, once, V3b's share in Decision 3). Each kind's payload struct lives in the emitting module's own row. No code in V7.

### 8. Verification at merge (V7, 2026-11-27)

`python scripts/loc_budget.py` exits 0 with every row OK: wiring 1,288 / 1,438 (150 left); recognizer 557 / 825 (268 left, as before the move); everything else 900 / 904 (4 left, as before the split); onset 0 / 750; input chain 0 / 600; position 0 / 900; health check 0 / 450; GUI 2,860 / 3,000 (the health row subtracts 0 today); the conductor, evidence, presenters, importers, voxxad, voxxa-eval, discovery, updater, Simple mode and per-driver rows unchanged. `python -m py_compile scripts/loc_budget.py` is clean. No test or workflow pins a cap: `grep -rn "2317\|loc_budget" tests/ src-tauri/tests/ scripts/ci/ .github/workflows/` finds only `ci.yml:156-157` (tokei pinned at 15.0.0 and the script run). No Rust changed, no dependency added, no workflow changed.

**Review trigger (added):** a tripled row passes its original estimate (onset 250, input chain 200, position 300, health check 150): the step reports what the extra lines hold before merging. The 95 % trigger stands.

**Founder's quality bar for V7 (FA-54).** Every row above carries its path, language, estimate, cap, owner, step, the measured number at merge and the reason for its headroom; the two questions the pack left open at V7 (the lab scripts row; V8c's and V3b's wiring shares) are decided here, not later.
