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

## 2026-11-13 — Q5: `voxxa-eval` and the first fixtures (ADR-015), 10x lane `q5/10x`
- **Shape:** `voxxa-eval` (new binary) → subprocess `voxxad --replay <session.jsonl>` (CI) or `voxxad --stdin-audio <wav> --shadow` (local, `--full`) → `action` lines → pure metrics against `truth.jsonl` → plain-text table → optional gate against `fixtures/baseline.json` (exit 1 on a worse metric, 2 on an error).
- **Boundaries:** the eval links no conductor or recogniser code; it reads schema v1 only. It never builds a presenter (only `--shadow`/`--replay` are passed). Full-mode logs go to a temporary folder, not the roaming `%APPDATA%`.
- **Failure modes:** a fixture without `session.jsonl` in replay mode, a setlist that does not match the log (replay refuses it), bad truth rows and unknown baseline keys are errors (exit 2), never silent skips.
- **Trade-off accepted:** the stand-ins are synthetic speech; replay measures the conductor only. Details and alternatives: ADR-015.
