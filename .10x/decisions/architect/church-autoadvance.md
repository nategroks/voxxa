# Architect — church-autoadvance

Date: 2026-10-24 · Status: Proposed · Full design: `.10x/specs/2026-10-24-software-strategy.md` · ADRs: 001 (headless core + thin GUI), 002 (platform sequence), 003 (TOML + JSON-lines), 004 (recognizer trait + bake-off), 006 (adapters), 008 (distribution), 009 (position tracking).

## Shape
`voxxad` (headless Rust: capture → Silero VAD → Recognizer trait → SongGraph/PositionTracker → Supervisor → Adapter trait) + `voxxa-gui` (Tauri thin client over JSON lines) + `voxxa-eval`. One `voxxa.toml`; files not databases.

## Boundaries and failure modes
- Mic → core: audio never persisted or sent; CI test asserts zero sockets during replay.
- Core → presentation app (LAN): host-bound, identity-confirmed adapters; Tier 1 read position back and re-sync; Tier 2/3 run blind and say so.
- Recognizer stall → amber in 8 s, no commands. Adapter disagreement twice → re-sync or red. Operator keys always win.
- Update channel: minisign + version monotonicity.

## Key trade-offs accepted
- Latency spent for correctness (late advance forgivable, wrong advance not).
- Flexibility spent for operability (three GUI controls; everything else in TOML).
- Cloud accuracy forgone for local-first trust.

## Budgets
Core ≤ 3,000 LOC; adapter ≤ 300; GUI ≤ 800; ≤ 25 direct crates; ≤ 30 MB binary; cold start ≤ 2 s.

## 2026-11-13 — Q5: `voxxa-eval` and the first fixtures (ADR-015), merged on `q5/base`
- **Shape:** `voxxa-eval` (new binary) → subprocess `voxxad --replay <session.jsonl>` (CI, default) or `voxxad --stdin-audio <wav> --shadow --model tiny` (local, `--full`) → `action` lines → pure scoring against `truth.jsonl` → plain-text table → gate against `fixtures/baseline.json` (exit 1 on a worse metric, 2 on an error).
- **Boundaries:** the eval links no conductor or recogniser code; it reads schema v1 only. It never builds a presenter (only `--shadow`/`--replay` are passed). Full-mode logs go to a temporary working folder with `--log-dir .`, so the log path is relative and the roaming `%APPDATA%` gets no log.
- **Failure modes:** a missing `session.jsonl` in replay mode, a setlist that does not match the log (replay refuses it), bad truth rows (including a song or slide outside the setlist) are errors (exit 2); a missing baseline row or an unknown metric name fails the gate (exit 1); never silent skips.
- **Trade-offs accepted:** the stand-ins are synthetic speech; replay measures the conductor only, with the `SmartConfig` stored in each log (a changed default needs a re-record, Q5-07). Details and alternatives: ADR-015.

## 2026-11-13 — Q6: the conductor against the budget (ADR-016), final on `q6/base` (4edd7f6), merge on hold
- **Decision:** the conductor at 733c7e1 missed both limbs of the budget (1.67 false advances per 10 sung songs; p95 11.23 s, stream time at file speed), measured independently by two lanes. So ADR-009's minimum was built, not "ADR-009 deferred".
- **Shape:** inside the conductor, the next-slide check is replaced by a position tracker (`aligner/evidence/`): candidates are every non-empty slide of the committed song; a prior per transition (stay, next, chorus start, section start, other; 0 for the slide an operator just left); a score from the newest words on a 12-word window; an accumulator with decay; a move at a threshold, a margin and `min_dwell_ms`. Moves are ordinary `goto(song, slide)` actions, so dispatch, adapters, the GUI and event schema v1 do not change. Song changes stay with song detection; while the screen is blank it reads only the newest transcript (Q5-04).
- **Boundaries kept:** `MachineState`, blanking, `manual_step`, `jump_to_song` and the conductor's public API unchanged; the tracker notices an operator's move itself; the operator always wins. Setlist v1 gains one optional field (`sections[].kind`), not serialised when empty, so old setlists and replay fingerprints are unchanged. `SmartConfig` gains `evidence` with a serde default. No new crates, no sockets.
- **Budget:** one new row, "conductor evidence (ADR-009 minimum)", 215 / 500 (user decision 3, a recorded deviation from ADR-010 Decision 3); the conductor row went down to 1,173 / 1,200.
- **Result:** false advances 1 → 0 (met), recall 0.68 → 1.00, p95 11.23 → 4.87 s (still over 2.5 s; now bounded mostly by the recogniser's first transcript of each part, Q6-03). One fixture's two latency cells got worse because its recall rose (0.25 → 1.00); that blocks the merge until the user decides (ADR-016, second amendment, item 5).
- **Trade-offs accepted:** numbers tuned on synthetic speech; labels help only where a setlist carries them (no importer writes them yet, Q6-06); the GUI's stored settings can pin old evidence defaults (Q6-01); ADR-009's graph-wide tracking (next song as a candidate, adapter re-sync) stays open (Q6-11).

## 2026-11-18 — Q6 close-out: baseline decision, ponytail re-run, final on `q6/base` (97cc21b)
- **User decision (2026-11-18):** the `jump-be-thou-my-vision` baseline row set to 1,500 / 6,000 ms as a one-time exception (85c736c); the `total` row's blank 7,500 and unblank 2,500 accepted. The merge is no longer blocked.
- **A second design, compared:** the ponytail re-run (`q6/ponytail` @ 7d4b4b6) built the same core on its own (a 12-token window, an LCS score, evidence with decay) in 117 lines and left song detection alone. It scores a slide being entered against the current slide's tail plus that slide's first words. The comparator kept `q6/base`: base is better on four fixture rows and the total, mostly because of its Q5-04 change to song detection while blank, and it already handles an operator undoing a tracker move. The re-run's scoring reached the jump row's log floor (p95 3,850 ms against base's 5,850); trying it in base is ticket Q6-15.
- **Kept, against the re-run:** `slide_advance_debounce_ms` and `advance_margin` stay in `SmartConfig` (ADR-016 Decision 7, so older replays still read new logs; Q6-10). Section labels stay in base's `sections[].slides` form.
- **Grafted:** tests (the re-run's scripted scenarios and its 27-step trace from 733c7e1, which passes unchanged on base), a README sentence on missing config fields, and a clamp of `decay` to 0..=1. No change to the conductor's shape.
- **Dependencies:** zip 9.0.0 → 9.0.5 and chrono replaced by `time` (already in the tree) to clear two new RustSec advisories. Two small transitive crates came in (nix, MIT; num_threads on Unix, MIT OR Apache-2.0). No advisory ignored.
- **Result at 97cc21b:** total 21/21 hits, precision and recall 1.00, 0 false advances (budget met), median 2.31 s, p95 4.87 s (the 2.5 s budget is still missed, Q6-03). Every gate and the CI eval step pass; both verifiers passed. Next: CI, PR and merge by the orchestrator.
