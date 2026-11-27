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

## Amendment 1 (Accepted, FA-50, founder 2026-11-27; written in V7): the dependency licence rule extends to model weights and aligners, as an inventory, not a gate

**Status: Accepted** (founder FA-50, 2026-11-27, batch 3 of `.10x/specs/2026-11-24-founder-action-items.md`; recorded by V7's 10x lane). The founder deviated from the recommendation ("permissive weights only; CC-BY-NC lab-only"): "use whatever licensing is required" to bring the Python lab work and all software into the system; a licence of voxxa's own, MIT-like with a confirmation of ministry use, is wanted later and is parked as **FA-76** (not blocking; needs counsel; not before the product ships).

**Decision.**
1. **Inventory, not a gate.** Every model, weight file and aligner used in the lab or the product is recorded in the table below with its name, version, licence, where it runs (lab-only or product) and the step that added it. The Decision's dependency rule (`cargo deny`, GPL-compatible crates only) keeps gating crates; weights and aligners are not crates, so this table is their record.
2. **Preference and flags.** Permissive licences (MIT, Apache-2.0, BSD, CC-BY, CC0) are preferred. Non-permissive licences (CC-BY-NC, copyleft on weights or tools) are **flagged** in the Licence column; a flag blocks nothing now. A flagged artefact in the product (not lab-only) is reported to the founder in the step's record as a number of flagged rows, so the FA-76 decision can be reconciled later.
3. **One caution, recorded and not decided** (from the FA-50 record): an MIT-like licence with a ministry-use clause is source-available, not open source, and cannot sit over a copyleft dependency; permissive upstreams keep that door open. The inventory is FA-76's input.
4. **Who keeps the table.** The step that adds, changes or removes a model or aligner edits its row in the same PR: V3f for the lab aligners (and it marks every cell below "(cited: verify in V3f)" as (measured) after reading each upstream licence file, naming the file); V3 for a Whisper model change (`--model`); V5 for the kit tools' Python dependencies; V8c if the Linux capture path needs a crate (ADR-036 Decision 10 names this inventory). Every cell is marked (measured), (cited) or (inferred) per the records rule in root `CLAUDE.md`.

**Inventory (V7, 2026-11-27; code references at `0190515`).**

| # | Model / weight / aligner | Version | Licence | Where it runs | Added by | Status |
|---|---|---|---|---|---|---|
| 1 | Whisper `ggml-tiny.bin` (whisper.cpp's ggml conversion of OpenAI Whisper tiny; `src-tauri/src/transcription/mod.rs:29`) | the whisper.cpp model repository, `main` (`transcription/mod.rs:39`) | MIT (whisper.cpp; the OpenAI Whisper weights are MIT) | **product**: downloaded by the operator in Advanced → Models; the pilot booth's model (status.md: 138.2 s booth timing) | Q4 (ADR-014) | (cited: verify in V3f) |
| 2 | Whisper `ggml-small.bin` (`transcription/mod.rs:31`) | same repository | MIT | **product**: selectable; V3's `--model` passthrough | Q4 (ADR-014) | (cited: verify in V3f) |
| 3 | Whisper `ggml-base.bin`, `ggml-medium.bin` (`transcription/mod.rs:30`, `:32`) | same repository | MIT | **product**: in the model list, not used by the pilot | Q4 (ADR-014) | (cited: verify in V3f) |
| 4 | Silero VAD, ONNX model bundled by the `voice_activity_detector` crate 0.2 (`src-tauri/Cargo.toml:71-72`; `src-tauri/src/vad/mod.rs:3-4`, `:21` names the v5 window) | Silero VAD v5 | MIT (Silero VAD); the crate and the ONNX Runtime it links are MIT (the crate is already under `cargo deny`) | **product**: on whenever voxxa listens; RMS-energy fallback if it fails to load | May branch (ADR-010 Appendix A) | (cited: verify in V3f) |
| 5 | Lab aligners under R5 (`.10x/specs/2026-11-26-research-autonomous-following.md`; the lab's `parts.py`, `predict.py` and their references) | — | — (any CC-BY-NC weight is flagged here and stays lab-only unless the founder says otherwise) | **lab-only** | V3f | rows: written by V3f |
| 6 | Kit tools' Python dependencies (`kit/tools/`, ADR-028 kit-tools row) | — | — | church-side (the booth checkout) | V5 | rows: written by V5 |

No row is flagged at V7: rows 1–4 are all MIT (cited). Nothing is blocked. No code or dependency changed in V7.
