# Current Handoff

## From: Q1 (SDE + Staff Engineer + DBA, 2026-11-11) → To: QA Engineer + Security Engineer for Q2
**Date:** 2026-11-11
**Phase:** Reconcile: Q1 merge done → Q2 verification (widened at the Q0 gate)

### What Q1 built
- `origin/claude/affectionate-lamport-8al8B` was merged (9987b62) per ADR-010 Appendix A and A1–A15, followed by logical fix-forward commits. The merge record, every dependency change and the debt are in `decisions/sde/church-autoadvance.md`; the SQLite decision is in `decisions/dba/church-autoadvance.md`.
- Two independent lanes, the 10x-team and ponytail (`full`), did the whole task. Each was reviewed by the other discipline, then compared, finished and adversarially verified: `reviews/2026-11-11-ponytail-q1.md`. The ponytail lane's branch `q1/ponytail` stays local for reference.
- **Gates on Windows:** all green. **Linux/macOS:** CI on PR #1 is the first real build there. Watch for:
  - `ort-sys` downloading ONNX Runtime at build time
  - the bundled SQLite compile
  - LF line endings in the integration tests (there is no `.gitattributes`)

### What Q2 must test and review (in addition to the Q2 prompt)
1. **The 12 confirmed audit defects** in `reviews/2026-11-11-ponytail-q0.md` §4–5. Fix 1–4 (wrong slide) plus crash and data-loss defects now; ticket the rest.
2. **Copyrighted lyric fixtures** still in `src-tauri/src/aligner/mod.rs` tests ("How Great Is Our God", "Way Maker"). Q1 could not touch the conductor. Replace them with PD hymn text (ADR-007).
3. **ADR-011:** check that the stage window cannot default to an audience-facing display. The Q1 reviews note that it is built for a second screen.
4. **Security inputs** (DBA note): EasyWorship SQLite hardening (`trusted_schema=OFF`, defensive mode, `immutable=1`, size cap); the HTTP API token is optional; CSP is null and `withGlobalTauri` is true; there are no capability files (verify `listen()` with one app launch); transcripts are logged into the diagnostics ring buffer.
5. **Known May bugs found in Q1 reviews:** the PDF importer never splits pages; GPG `.asc` signatures never reach the release.
6. **Evidence for the post-Q2 cut-list ADR:** HTTP API, mDNS, Hymnary, PDF, stage display, Flatpak, signing scaffolds.

### Open for the user
- **Pin the CI toolchain and tokei** (R13)? Recommended; it changes the CI bar, so it needs the user's sign-off.
- **A15 follow-ups:** `fixtures/setlists/pd-hymns.json` and `fixtures/pco/plan-sample.json` don't fit the May parsers; re-run the Windows verification (a003992) on the merged head (`handoff-local-windows.md`); finish reconciling `docs/signing-policy.md` with `SIGNING.md`.
- **ADR numbering:** the next free number is ADR-012.

**Process rule (user, 2026-11-11):** every prompt (Q2…Q27) also runs independently through ponytail. Run it as two lanes plus cross-review for code steps (the Q1 pattern), or `ponytail` + `ponytail-audit` for verification and close steps. Record the result in `reviews/YYYY-MM-DD-ponytail-<Q>.md`.

---

## Handoff History
- 2026-11-11 Q1: May branch merged per ADR-010 + A1–A15; two lanes (10x, ponytail) → 10x base with ponytail grafts; all local gates green; build.yml YAML fixed.
- 2026-11-11 Q0 gate: the user accepted ADR-010 as amended; ADR-011; cuts deferred until after Q2; Q2 widened to cover wrong-slide defects.
- 2026-11-11 Q0: claims verified, ADR-010 path table + amendments A1–A15, ponytail lane; stopped at the user gate.
- 2026-11-11 re-planning: May branch discovered; ADR-010 Proposed; prompt pack regenerated for /10x-Team:10x-team.
- 2026-10-24 init → Discovery: `.10x/` created.
- 2026-10-24 Discovery → Strategy: codebase scanned; critical finding: model never loaded, VAD never initialized.
- 2026-10-24 Strategy → Design → Planning: specs + ADR-001…009 + prompt plan written in one pass after five-angle research.
