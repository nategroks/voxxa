# CTO — church-autoadvance (Discovery notes)

Status: [DISCOVERED] — reverse-engineered from code and git history on 2026-10-24. Strategy decisions follow in the spec after research.

## What exists
- A working skeleton: mic capture (cpal) → energy VAD → 5 s chunks → whisper.cpp (whisper-rs 0.13, greedy, single segment) → LCS-based partial_ratio vs current/next slide → Right-Arrow keypress (enigo). Tray icon, model downloader, updater plumbing, GitHub Actions release matrix for macOS (arm64 + x86_64), Windows (MSI/NSIS) and Linux (AppImage/deb).
- No tests. No persistent settings (save_settings is a no-op). No presentation-software adapters beyond keystroke injection. No scripture handling. No setlist import beyond a hand-written JSON.

## Strategic observations
- Keystroke injection is the only universal control path today; it requires the presentation app to hold keyboard focus, which breaks the moment an operator clicks elsewhere. This is the largest product risk after recognition accuracy.
- The "dictation" positioning (website, README, release body, Cargo description) contradicts the shipped behaviour. Positioning must be fixed before any church sees it.
- `mcp` module and feature flag are dead weight for the church product; candidate for deletion (ponytail rung 1).
