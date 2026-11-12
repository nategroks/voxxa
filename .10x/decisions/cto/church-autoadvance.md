# CTO — church-autoadvance (Discovery notes)

Status: [DISCOVERED] — reverse-engineered from code and git history on 2026-10-24. Strategy decisions follow in the spec after research.

## What exists
- A working skeleton: mic capture (cpal) → energy VAD → 5 s chunks → whisper.cpp (whisper-rs 0.13, greedy, single segment) → LCS-based partial_ratio vs current/next slide → Right-Arrow keypress (enigo). Tray icon, model downloader, updater plumbing, GitHub Actions release matrix for macOS (arm64 + x86_64), Windows (MSI/NSIS) and Linux (AppImage/deb).
- No tests. No persistent settings (save_settings is a no-op). No presentation-software adapters beyond keystroke injection. No scripture handling. No setlist import beyond a hand-written JSON.

## Strategic observations
- Keystroke injection is the only universal control path today; it requires the presentation app to hold keyboard focus, which breaks the moment an operator clicks elsewhere. This is the largest product risk after recognition accuracy.
- The "dictation" positioning (website, README, release body, Cargo description) contradicts the shipped behaviour. Positioning must be fixed before any church sees it.
- `mcp` module and feature flag are dead weight for the church product; candidate for deletion (ponytail rung 1).

---

## Phase 1 decisions (2026-10-24) — see `.10x/specs/2026-10-24-business-strategy.md`
- **Build vs buy:** build. No product follows sung lyrics by listening (research E1); scripture detection is commoditised (FreeShow, SmartVerses, Pewbeam) and is table stakes, not the product.
- **Business model:** free GPL software; revenue from Sunday Support (attendance bands $190/$390/$790 per year, to be price-tested), Setup & Calibration ($249 remote / $599 on-site), and a ~$19.99 Microsoft Store convenience listing (Krita model). Donations are goodwill only. Support-only conversion is expected at ~1%; year one proves conversion, it does not fund a salary.
- **Channels:** pilot churches → demo video; church IT MSPs as resellers; FreeShow/OpenLP communities; denominational discount programmes; church-tech creators. No paid ads.
- **Entity and licence:** single-member LLC; GPL-3.0-or-later app, MIT protocol/schemas (ADR-005). Written question to CCLI on index-only lyric use before launch (ADR-007).
- **Tech direction:** keep Rust + Tauri; headless core + thin GUI; recognizer behind a trait with a data-driven bake-off; native adapters for ProPresenter/FreeShow/OpenLP; Windows → macOS → AppImage.
- **Kill criteria:** if the pilot corpus cannot get false advances under 1 per 10-song set with any recognizer by week 14, pivot Voxxa to a scripture-first, operator-assist tool and say so publicly.
