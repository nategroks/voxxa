# CTO — Index

> Strategy, build/buy, tech direction. Cross-cutting principles first, then one file per feature.

## Principles (discovered + proposed)
- [DISCOVERED] Local-first: all speech recognition runs on-device (whisper.cpp). No cloud, no account. This is the product's trust story for churches.
- [DISCOVERED] Rust + Tauri 2.0 desktop binary with a vanilla HTML/JS frontend; no framework. Already pivoted from "voice dictation" to "worship slide auto-advancer" (commit 0a15de1, 2026-05-26). README, Cargo description, website copy and release notes still describe dictation.
- [DISCOVERED] Legacy Python prototype (faster-whisper + rapidfuzz + pyautogui) kept in `voxxa/` as reference; the Rust port mirrors it 1:1 including the partial_ratio alignment logic.

## Active features
- `church-autoadvance` — business + software strategy for the free binary / paid support model. See `church-autoadvance.md`.
