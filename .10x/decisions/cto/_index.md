# CTO — Index

> Strategy, build/buy, tech direction. Cross-cutting principles first, then one file per feature.

## Principles (discovered + proposed)
- [DISCOVERED] Local-first: all speech recognition runs on-device (whisper.cpp). No cloud, no account. This is the product's trust story for churches.
- [DISCOVERED] Rust + Tauri 2.0 desktop binary with a vanilla HTML/JS frontend; no framework. Already pivoted from "voice dictation" to "worship slide auto-advancer" (commit 0a15de1, 2026-05-26). README, Cargo description, website copy and release notes still describe dictation.
- [DISCOVERED] Legacy Python prototype (faster-whisper + rapidfuzz + pyautogui) kept in `voxxa/` as reference; the Rust port mirrors it 1:1 including the partial_ratio alignment logic.

## Active features
- `church-autoadvance` — business + software strategy for the free binary / paid support model. See `church-autoadvance.md`.

## [DISCOVERED 2026-11-11] A second implementation branch exists
- `origin/claude/affectionate-lamport-8al8B`: 44 commits by a Claude session, 2026-05-28 → 2026-06-01, based on the same `0a15de1` as `main` and as our branch; never merged; not visible to the October discovery because remote branches were not fetched. ~5,400 LOC Rust, ~2,600 LOC GUI, 35 unit + 4 integration tests, docs (INTEGRATIONS, LINUX, REVIEW, SETLIST_FORMATS), Flatpak packaging, signing scaffolds.
- It implements most of Increments 1–3 of the October plan in a different shape (in-process monolith, feature-rich GUI). Strategic call recorded in ADR-010: adopt it as the code base, keep Increment 0's governance, re-scope the plan to the real gaps (measurement, volunteer-simple mode, read-back/contract fixtures, scripture, signing/Store).
