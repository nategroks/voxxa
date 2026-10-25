# Staff Engineer — Index

## Discovered conventions
- Rust 2021, anyhow for errors in engine code, String errors across the IPC boundary, `log` + env_logger. Release profile: strip, LTO, opt-level "s", codegen-units 1.
- Frontend: zero dependencies, `window.__TAURI__` globals (withGlobalTauri: true), DOM manipulation by id, no bundler config beyond Vite defaults.
- No test harness in either Rust or JS. No lint config (clippy/fmt not enforced in CI). CI only runs on tags.
- Known defects seen in the scan: `next_slide_manual` calls `aligner.update("__manual_advance__")`, which cannot advance the index (it just pollutes the word buffer); `get_status.model_loaded` is hard-coded true; `TranscriptionEvent` scores are always 0.0; `transcribe()` is never preceded by `load_model()` anywhere in `commands` (start_listening will fail with "No model loaded"); VAD `initialize()` is never called so VAD passes all audio through; `save_settings` is a no-op; CSP is null.
