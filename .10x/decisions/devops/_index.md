# DevOps — Index

## Discovered
- `.github/workflows/build.yml`: tag-triggered (`v*`) matrix build with tauri-action, uploads dmg/msi/nsis/AppImage/deb, generates `latest.json` for tauri-plugin-updater, optional Apple signing/notarization via secrets, minisign updater key already configured (pubkey in tauri.conf.json).
- No Windows Authenticode signing step (certificateThumbprint null) → SmartScreen warnings on every download.
- `.github/workflows/pages.yml`: deploys `docs/` to GitHub Pages; the page still markets "Local Voice Dictation".
- No PR/branch CI: nothing runs cargo test/clippy/fmt on push.
