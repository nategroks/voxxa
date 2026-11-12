# ADR-005: License the application GPL-3.0-or-later; publish the protocol and schemas under MIT

**Status:** Accepted · **Date:** 2026-10-24 · **Feature:** church-autoadvance · **Reversibility:** One-way in practice (relicensing needs every contributor's consent) · **Owner:** nategroks

## Context
Revenue comes from support and a Store convenience listing, not from the code. Free Windows signing via SignPath Foundation requires an OSI-approved licence with no commercial dual-licensing (research D4). The Microsoft Store accepts GPL apps (Krita, D2). All current dependencies are permissive (whisper.cpp MIT, whisper-rs Unlicense, Tauri MIT/Apache, cpal Apache, enigo MIT, sherpa-onnx Apache). FreeShow and OpenLP, the open-source peers, are GPL.

## Decision
- Application code: **GPL-3.0-or-later**. A proprietary fork repackaged with "AI" marketing is the realistic threat to a support business; copyleft keeps forks open and keeps the community in one place.
- `voxxad` JSON-lines protocol specification, `setlist.json` schema and `voxxa.toml` reference: **MIT** (documents, not code), so Companion modules, Stream Deck plugins and presentation vendors can implement them without licence questions.
- Dependency rule enforced by `cargo deny`: GPL-compatible only; avoid AGPL crates (`bible-io`) and non-commercial model licences (Parakeet's status is contested, E5).
- No contributor licence agreement; use a Developer Certificate of Origin sign-off, which keeps relicensing honest and SignPath's "no dual licensing" condition true.

## Alternatives
| Alternative | Why not |
|---|---|
| MIT/Apache for everything | Maximises adoption but lets a vendor ship a closed fork and sell support against the project; the business model has no other moat. |
| AGPL | Overkill for a desktop app with no network service; scares church IT partners. |
| BSL/FSL source-available | Disqualifies free signing (D4) and the "open source" positioning; Sentry itself says FSL is not OSI open source. |
| MPL-2.0 | Reasonable middle; chosen against because the app is an end product, not a library to embed. |

## Consequences
Good: free signing path, clear positioning, aligned with FreeShow/OpenLP. Bad: cannot later sell a proprietary edition without rewriting or re-consent. Risks: contributors who dislike GPL; acceptable for this audience.

## Review trigger
Re-open only if a vendor partnership requires embedding Voxxa in a proprietary product; the answer then is a separate MIT core crate, not relicensing the app.
