# V7 comparator record (2026-11-27): two lanes, one final branch

Step V7, "Wiring budget row and the ADR-028 amendment" (full scope: FA-10, FA-54, FA-65, FA-74, FA-77, FA-50). Two
independent lanes ran from `origin/main` at 0190515 in separate worktrees, neither seeing the other: the **10x lane**
(`/10x-Team:10x-team`, Engineering Manager + Staff Engineer; branch `v7/10x`, commits bee86c6, 09bd5a8, 79a268e) and the
**ponytail lane** (ponytail at level full, then ponytail-review; branch `v7/ponytail`, commits c919e6d, 80db5d6,
76553aa). The orchestrator compared them and built `v7/final`.

## Where the lanes agree (every number identical)

| Measure (tokei 15.0.0, code lines at 0190515) | Both lanes | Orchestrator's independent count |
|---|---|---|
| `dispatch.rs` + `listen.rs` + `http_api/` + `commands/` | 147 + 159 + 238 + 744 = **1,288** | 1,288 |
| `vad/` | **125** (the prompt pack's 189 superseded) | 125 |
| "everything else" before → after | 2,313 / 2,317 (4 left) → 900 / 904 (4 left) | 2,313 / 2,317 |
| wiring row | 1,288 / 1,438, 150 left | — |
| recognizer (`transcription/` + `vad/`) | 432 / 700 → 557 / 825, 268 left both times | — |
| onset / input chain / position / health check | 0 / 750, 0 / 600, 0 / 900, 0 / 450 | — |

Both: `scripts/loc_budget.py` exits 0 with every row OK; `onset` is also subtracted from the conductor row (it sits
under `aligner/`); `health` is subtracted from the GUI row like `src/simple/`; absent paths count 0 with no special
case; nothing in tests or workflows pins a cap (only `ci.yml` pins tokei 15.0.0). Both set ADR-028 to Accepted with
amendment 1 as the record, wrote the `trace` amendment on ADR-003 with a cross-reference on ADR-001, and the licence
inventory on ADR-005 with the Whisper models and Silero VAD as "(cited: verify in V3f)". Both decided `trace` is
**session-log only**, the strict reading of "like `window`". Both decided **no lab-scripts row** in V7 (V3e records its
count). Both lane scans: 0 private-term hits and 0 regex hits in added lines for every file; the pre-existing name lines
in `status.md` (8) and `handoff.md` (9) stay per FA-12 (c).

## Where they differ, and what the final branch takes

| Topic | 10x lane | Ponytail lane | Final |
|---|---|---|---|
| Wiring share table | the pack's six rows (135) **plus V8c ≤ 20 and V3b task 6 ≤ 15, sum 170 > 150**, recorded as ceilings spent in merge order with a Phase 2 rule (start only if it fits what `loc_budget.py` shows left, else a smaller share with logic moved into the step's own row) | the pack's six rows, 135 of 150 | **10x**: the two omitted steps are real wiring spenders; the ceiling rule keeps the cap honest |
| voxxad share table | adds V3b `Kind::Trace` ≤ 4 (active ≤ 42 of 48) | the pack's three live shares (≤ 38) | **10x**: the trace recorder arm is voxxad code and needed a share |
| Trace field-list owners | `screen` → V8c (V8b adds), `operator` → V13a (V13s confirms), `cue` → V13b (deferred) | `screen` → V8b (V8c if first), `operator` → V13s, `cue` → first emitter | **10x**: V13a lands the controls, V8c lands Source B, V13b owns cues |
| ADR-028 amendment shape | eight sections: wiring table, vad move, what "everything else" holds (lib.rs 148, main.rs 4, audio/ 162, diagnostics/ 141, help/ 176, hymnary/ 188, net_stats/ 15, tray/ 66) and why 4 lines, the new rows with estimate sources, share tables, Decision 4 restated, trace consequence, verification at merge | one per-row table (path, language, estimate, cap / used / left before → after, why this headroom, owner, step) plus numbered items | **10x** text; the ponytail table's content is covered |
| Serialisation test | `every_event_type_serialises_as_documented_and_round_trips` gains a sample per kind + new `a_trace_line_carries_numbers_only` (only `type` and `kind` may be strings) | the same test gains an example line + an assertion that the only string value is `kind` | **10x** (names both tests; `type` is a string too) |
| Review record | none (10x lanes write role decisions) | `.10x/reviews/2026-11-27-ponytail-v7.md` with the ponytail-review ("Looks good. Ship.") | **grafted** from ponytail |
| Row label | `recognizer (transcription/ + recognizer code + vad/)` | `recognizer (transcription/ + vad/)` | **10x** (keeps the old label's words) |

One wording change by the comparator on the final branch: `CLAUDE.md` § Budget rows said the 4 remaining lines "are"
the future `mod` lines; they are **reserved for** them (a plan, not a count).

## Final branch

`v7/final` = `v7/10x` fast-forwarded (79a268e) + the ponytail review record + this record + the two wording edits.
`python scripts/loc_budget.py` on `v7/final`: exit 0, every row OK, the same numbers as above. Lane scan on the staged
files of the final commit: 0 private-term hits in added lines; regex hits only the allowed classes. The PR carries the
numbers; the next records commit (V8c's start) writes the PR number and merge hash into `status.md`.
