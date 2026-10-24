# ADR-002: Ship Windows first, then macOS, then Linux AppImage, from one codebase and one CI matrix

**Status:** Proposed
**Date:** 2026-10-24
**Feature:** church-autoadvance
**Reversibility:** Easy (sequence only; the code is already cross-platform)
**Owner:** nategroks
**Author:** 10x-Team (CTO + DevOps)

## Context
The CI matrix already builds all three platforms on every tag. The brief asks for Windows first, then macOS, then AppImage. Supporting a platform is not "it compiles": it is signed installers, a tested audio path, a tested key-injection/adapter path, a quick-start card, and a support runbook. A 1–2 person team can do that properly for one platform at a time.

## Decision
- **Windows 10/11 x64 is the launch platform.** Signed NSIS per-user installer + MSI, bundled WebView2 bootstrapper, winget manifest. Audio via WASAPI (cpal). Keystroke fallback via enigo (SendInput).
- **macOS (arm64 + x86_64) is second.** Notarized DMG, Homebrew cask. Keystroke fallback needs Accessibility permission; the GUI must guide the user through it. Audio via CoreAudio (cpal).
- **Linux AppImage is third.** Single-file AppImage with embedded webkit2gtk, `.deb` as a by-product, Flathub later if asked. X11 key injection via `libxdo`; on Wayland keystrokes are unavailable, so Linux users rely on network adapters (FreeShow/OpenLP are the Linux-native presentation apps anyway).
- The CI matrix keeps building all three on every tag so that "second" and "third" never rot; only signing, docs and support commitment are sequenced.

## Alternatives Considered
| Alternative | Pros | Cons | Why Not |
|---|---|---|---|
| macOS first | ProPresenter's historic home; many larger churches | Larger churches already have paid staff and are the least price-sensitive segment for *support*; notarization + Accessibility prompts add onboarding friction | Windows is where EasyWorship/MediaShout/FreeShow-on-PC volunteers live and where the free-software + support model lands best |
| All three at once | One launch | Three signing pipelines, three audio stacks, three runbooks on day one with no users yet | Violates "architecture the team cannot operate is a failed architecture" |
| Windows-only forever | Simplest | Excludes ProPresenter-on-Mac churches, the richest API we have | The brief is explicit about all three |

## Consequences
### Positive
- One quick-start card, one signing identity, one support runbook at launch.
- Linux support arrives via network adapters without fighting Wayland.
### Negative
- macOS churches wait; mitigated by unsigned "preview" builds from the existing CI for early adopters who accept the Gatekeeper right-click dance.
### Risks
- Windows SmartScreen reputation takes weeks to build even with an OV certificate; mitigated by Azure Trusted Signing or SignPath Foundation (research pending on eligibility and cost; see ADR-005 placeholder in the spec).

## Dependencies
- Depends on: ADR-001 (core/GUI split so the headless core can be tested on all three in CI regardless of GUI polish).
- Constrains: the delivery plan's increment order.

## Review trigger
Re-open if > 30% of inbound interest in the first 90 days is from macOS churches.
