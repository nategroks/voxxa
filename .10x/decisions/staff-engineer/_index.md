# Staff Engineer — Index

## Discovered conventions
- Rust 2021, anyhow for errors in engine code, String errors across the IPC boundary, `log` + env_logger. Release profile: strip, LTO, opt-level "s", codegen-units 1.
- Frontend: zero dependencies, `window.__TAURI__` globals (withGlobalTauri: true), DOM manipulation by id, no bundler config beyond Vite defaults.
- No test harness in either Rust or JS. No lint config (clippy/fmt not enforced in CI). CI only runs on tags.
- Known defects seen in the scan: `next_slide_manual` calls `aligner.update("__manual_advance__")`, which cannot advance the index (it just pollutes the word buffer); `get_status.model_loaded` is hard-coded true; `TranscriptionEvent` scores are always 0.0; `transcribe()` is never preceded by `load_model()` anywhere in `commands` (start_listening will fail with "No model loaded"); VAD `initialize()` is never called so VAD passes all audio through; `save_settings` is a no-op; CSP is null.

## [DISCOVERED 2026-11-11] Quality bar of `claude/affectionate-lamport-8al8B` against Increment 0's CI
- `cargo fmt --check`: 51 diffs. `cargo deny` (our policy): advisories FAILED on zip 2.x, quick-xml 0.36, h2, rustls, rustls-webpki, openssl (reqwest 0.12 native-tls), libsqlite3-sys (bundled SQLite CVE), lopdf (pdf-extract); licences/bans not yet evaluated separately. Line budgets: core 5,400 / 3,000, GUI 2,595 / 800 (budgets must be re-based, ADR-010). Clippy (`--all-targets -D warnings`): exit 101, 10 error lines; most frequent: 3 error: you should co;2 error: this `impl` ca;2 error: could ;1 error: this block may be rewritte;1 error: this `if` ca;
- Patterns there worth keeping: `Capabilities` flags + parity contract test; `Action` enum dispatched by capability; conductor unit tests with scripted transcripts; `docs/REVIEW.md` punchlist discipline.
- Patterns to replace: 5 s batch transcription; character-level partial ratio on 40-word buffers; localStorage for operator state (support bundle cannot see it).
