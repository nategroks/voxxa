# DevOps — Index

## Discovered
- `.github/workflows/build.yml`: tag-triggered (`v*`) matrix build with tauri-action, uploads dmg/msi/nsis/AppImage/deb, generates `latest.json` for tauri-plugin-updater, optional Apple signing/notarization via secrets, minisign updater key already configured (pubkey in tauri.conf.json).
- No Windows Authenticode signing step (certificateThumbprint null) → SmartScreen warnings on every download.
- `.github/workflows/pages.yml`: deploys `docs/` to GitHub Pages; the page still markets "Local Voice Dictation".
- No PR/branch CI: nothing runs cargo test/clippy/fmt on push.

## church-autoadvance
- 2026-11-19 **Q7**: `voxxad`/`voxxa-eval` out of the installers (moved to `src/cli/`, `cli` feature; tauri-cli also bundles every `src/bin/` entry; `voxxa-eval.exe` was in the NSIS installer before), proved with the generated NSIS script; `.github/workflows/full-eval.yml` (report only, pinned model and actions); release notes `docs/releases/voxxad-cli-preview.md` (no tag, no release); Companion section in `docs/INTEGRATIONS.md`.
- 2026-10-25 `church-autoadvance.md`: signing-policy draft and SignPath readiness gaps; Store account = Company after LLC (founder decision).
