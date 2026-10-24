# Staff Engineer — church-autoadvance

Date: 2026-10-24 · Status: Proposed

## Patterns to follow
- Ponytail ladder on every change; `shortcut:` comments for known limits; one small test per new branch/loop/parser.
- Errors: `anyhow` inside engine code, plain-English `String` across IPC and in JSON `error` events with a stable `code`.
- Traits with capability flags (`Recognizer`, `Adapter`); config-selected implementations; no plugin loader.
- Fixtures over mocks: recorded HTTP/WS traffic for adapters; WAV + truth for the tracker; `--replay` for every bug.
- Protocol and config are append-only (`"v":1`, new TOML keys with defaults).
- Keep the hand-rolled LCS (no crate has `partial_ratio`, research E8) but apply it to word tokens.

## Cross-cutting
- Logging: `log` + `env_logger` stay; every decision logged with scores to `session-*.jsonl`.
- Secrets: OS keychain via `keyring`; never in TOML.
- CI: fmt, clippy -D warnings, test, deny, tokei budgets, eval gate on fixtures.
- Delete now: `mcp`, `tauri-plugin-store`, GUI sliders, `overlay.html` (after P2.1).

## Known defects to fix first (P1.1)
`load_model` never called; `VadEngine::initialize` never called; `next_slide_manual` pushes a sentinel word; `model_loaded` hard-coded; scores always 0.0; `save_settings` no-op; CSP null.
