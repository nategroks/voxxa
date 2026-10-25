# SDE — church-autoadvance

## Increment 0 (2026-10-25) — reposition, dead weight, licence, CI

### Built (P0.1)
- README rewritten for the church product; landing page `docs/index.html` copy replaced in place (title, meta, hero, six feature cards, three steps, footer licence) with the design untouched; `Cargo.toml` description; release body in `build.yml`. `grep -ri dictat` is clean outside git history.

### Built (P0.2)
- Removed the `mcp` module, the `rmcp` optional dependency and the `mcp` feature; removed `tauri-plugin-store` (code, Cargo, lockfile). Lockfile pruned accordingly.
- Added `LICENSE` (GPL-3.0-or-later, 674 lines), `DCO` (Developer Certificate of Origin 1.1), `CONTRIBUTING.md`, `license` fields in `Cargo.toml` and `package.json`, and `src-tauri/deny.toml` (allow-list of GPL-compatible permissive licences; anything else fails).

### Built (P0.3)
- `.github/workflows/ci.yml`: on pull_request, push to main and manual dispatch; matrix ubuntu-22.04 / windows-latest / macos-latest running frontend build → `cargo fmt --check` → `cargo clippy --all-targets -D warnings` → `cargo test`; separate `cargo deny` job (EmbarkStudios action) and a line-budget job (`scripts/loc_budget.py` via tokei: core Rust ≤ 3,000, GUI ≤ 800). Current counts: 918 and 598.
- `cargo fmt` applied to three files; six clippy findings fixed with minimal edits (`Default` impls for `AudioEngine`, `TranscriptionEngine`, `VadEngine`; derived `Default` for `WhisperModel`; two `2 * 1 * 64` → `2 * 64`).

### Deviations from the prompts (and why)
- Added the `macos-private-api` Cargo feature to `tauri`: the tauri build script refuses a plain `cargo clippy`/`cargo test` while `macOSPrivateApi: true` is set in `tauri.conf.json` (the Tauri CLI adds the feature implicitly). Both the flag and the hidden `overlay.html` that needs it go away in P2.1.
- `cargo deny` surfaced 17 advisories in transitive dependencies. Fixed by compatible updates to `anyhow`, `h2`, `rustls`, `rustls-webpki`, `tokio`, `webpki-root-certs` and `plist` (which carries `quick-xml`). Three `zip` advisories cannot be fixed yet (every `tauri-plugin-updater` release through 2.14.0 requires `zip` 4.x); they are ignored in `deny.toml` with the reasoning and a removal trigger. `unmaintained` is set to `workspace` scope because the `unic-*`, `fxhash` and `proc-macro-error` findings are transitive through tauri with no upgrade.
- Reverted a speculative `tauri-plugin-updater` 2.10.1 → 2.12.0 bump that did not change `zip`; kept the diff to what the gate needed.

### Tech debt created or carried
- `overlay.html` and `macOSPrivateApi` remain until P2.1 deletes them.
- `deny.toml` advisory ignores for `zip` must be removed when the updater plugin moves to `zip` 9.
- No automated tests exist yet; `cargo test` passes trivially. P1.1 adds the first aligner tests.

### Verification
Local (Ubuntu, rust 1.97): `cargo fmt --check` ok; `cargo clippy --all-targets -- -D warnings` ok; `cargo deny check licenses advisories bans` ok; `cargo test` ok; `scripts/loc_budget.py` ok. CI on the three-OS matrix: see the workflow run linked in the commit.
