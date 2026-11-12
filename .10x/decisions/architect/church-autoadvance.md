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
