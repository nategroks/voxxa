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
