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

### Q0 verification (2026-11-11)
Log: `reviews/2026-11-11-q0-claim-verification.md`.
- **fmt:** 51 hunks in 20 files. Confirmed.
- **Tests:** 35 unit + 4 integration, all passing. Confirmed. 16 of them are conductor tests.
- **Clippy (corrected):** rustc/clippy 1.99.1 reports **16 errors across 7 lints** (18 `error` lines). Eight of them are `double_must_use` from the `#[async_trait]` expansion at `presenters/mod.rs:148`, so the result depends on the toolchain; CI installs unpinned stable. Four of the remaining lints (audio:22, transcription:51 and :75, vad:52) are already fixed by our side and survive the auto-merge.
- [DISCOVERED] **The May tip does not build on Windows.** `icons/icon.ico` and `icon.icns` are PNGs, and RC2175 panics in tauri-winres. Our d976698 fix is required.
- **`cargo deny` advisories:** confirmed against a shallow clone of rustsec/advisory-db (cargo-deny is not installed locally). Also missed: `webpki-root-certs` 1.0.6. rustls, rustls-webpki, zip 4.6.1 and a second reqwest 0.13 come from `tauri-plugin-updater` 2.10.1, which our 2.14.1 already replaces. Our own reqwest 0.12 uses native-tls/openssl.
- **LOC units:**
  - The architect figures are raw `wc -l`; the budgets are tokei code lines: aligner 716, presenters 1,121, importers 1,041, planning_center 246.
  - Under ADR-010 Decision 3, "everything else" is **2,522 > 2,500**. Amendment A8 counts `planning_center/` with the importers, giving 2,276.
  - The GUI is 2,595 in `src/`, or 2,982 with the root `index.html` and `stage.html`, which `loc_budget.py` currently ignores.
  - The `-e adapters` exclusion matches nothing.
  - `package.json` declares an unused runtime dependency, `@tauri-apps/api`, which a literal "no runtime npm deps" check would fail.
- **Patterns worth keeping (qualified):** the Capabilities pattern is good, but the "parity" test is one-way and skips unblank, and the GUI ignores the flags. "Action dispatched by capability" holds for Goto only.
- **Unused dependency:** `env_logger`.
- **Tracked prototype:** a Python prototype (`main.py`, `requirements.txt`, `voxxa/`) is still tracked on both branches. It is outside the merge diff.
