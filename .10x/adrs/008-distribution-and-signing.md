# ADR-008: Distribute signed GitHub releases plus a paid Microsoft Store listing; notarize on macOS; AppImage with embedded update info

**Status:** Proposed · **Date:** 2026-10-24 · **Feature:** church-autoadvance · **Reversibility:** Costly (publisher identity and reputation accrue to one identity; D3) · **Owner:** nategroks

## Context
Volunteers abandon installs at SmartScreen; EV certificates no longer bypass it; Store apps never trigger it; individual developer registration is free (research D3). SignPath Foundation signs OSI-licensed projects for free; Azure Trusted Signing costs ~$10/month with eligibility caveats (D4). Krita's free-site plus paid-Store model out-earned donations eightfold (D2). Apple requires notarization and Homebrew will disable unsigned casks in September 2026 (D5). Tauri's updater needs a signature and the key is unrecoverable (D6).

## Decision
- **Windows:** GitHub Releases carry NSIS per-user and MSI installers signed via SignPath Foundation (apply with the GPL repo and a public signing policy) or Azure Trusted Signing (individual US/CA account) if SignPath declines; `winget` manifest submitted per release; **Microsoft Store listing at ~$19.99** of the identical build as the frictionless path and a revenue line. One publisher identity for all Windows artefacts so reputation accrues once.
- **macOS:** Developer ID + notarization from Increment 6; Homebrew cask; Mac App Store only on demand.
- **Linux:** AppImage built on Ubuntu 22.04 with webkit2gtk 4.1, zsync update info embedded, `.deb` as a by-product; Flathub only if asked.
- **Updates:** existing minisign key; private key offline with two holders; client enforces version monotonicity (defends the unsigned-manifest downgrade class); support-tier churches get a "stable" channel that lags community by two weeks.
- Unsigned "preview" builds of macOS/Linux continue from the existing CI from day one, clearly labelled.

## Alternatives
| Alternative | Why not |
|---|---|
| Buy an OV/EV certificate ($219–620/yr + hardware key) | Costs more, needs a hardware token in CI, and EV buys nothing at SmartScreen any more (D3). |
| Store only | Excludes churches with Store-blocked PCs and breaks the "free download" promise. |
| GitHub only | SmartScreen friction for weeks or months; no revenue line. |
| Flatpak first on Linux | Flathub is large (438 M downloads in 2025) but the brief asks for AppImage; FreeShow/OpenLP users are the Linux audience and already handle AppImages. Revisit on request. |

## Consequences
Good: SmartScreen-free path on day one, zero certificate cost if SignPath accepts, a revenue line that funds the certificates. Bad: Store packaging (MSIX) and Partner Center review add a day per release; mitigated by automating it in CI. Risks: SignPath acceptance is not guaranteed; Trusted Signing eligibility contradictions (D4) — apply to both in week 1.

## Review trigger
Re-open if Store sales < 50 in the first 90 days (drop the Store release work) or if SmartScreen reputation is established on GitHub builds (keep the Store as revenue only).
