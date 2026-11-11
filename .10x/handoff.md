# Current Handoff

## From: Q0 (CTO + Architect + Staff Engineer, 2026-11-11) → To: the SDE for Q1
**Date:** 2026-11-11
**Phase:** Discovery of the May branch complete → user gate **passed** → Reconcile (Q1 merge, Q2 verification)

### What Q0 did
- Verified every [DISCOVERED 2026-11-11] statement against the code of `origin/claude/affectionate-lamport-8al8B` @ f7b56e6. Corrections are inline in the role indexes; the full log is `reviews/2026-11-11-q0-claim-verification.md`.
- Reviewed ADR-010 Decision 1 against the real merge (`git merge-tree`).
  - Only 6 files conflict textually.
  - The rule as written would revert f177fbe (tauri 2.12.5 / openssl 0.10.82) through `Cargo.lock`, and would drop our clippy fixes through "theirs for all of src-tauri/src".
  - ADR-010 now has amendments A1–A15 and Appendix A: every one of the 110 differing paths, with a side.
- Ran the same prompt independently through ponytail (full) and ponytail-audit, as the user asked: `reviews/2026-11-11-ponytail-q0.md`. Its 12 must-fix defects were all confirmed by a refuting skeptic. They are Q2 input.
- Nothing merged, no code changed. The worktrees `C:\dev\voxxa-newton` (this branch) and `C:\dev\lamport` (May, detached) exist locally. Build artefacts are in `C:\dev\lamport-target`.

### User decisions at the Q0 gate (2026-11-11)
1. **ADR-010 Accepted as amended (A1–A15).**
2. **Cut list:** merge everything now. Cuts are decided after Q2, in a separate ADR.
3. **ADR-011 Accepted:** Voxxa never *projects* lyrics; operator-only previews are allowed (amends ADR-007).
4. **Q2 widened:** it also fixes the wrong-slide defects (audit 1–4). The Q2 prompt in the pack is updated.

### Still open
- PR target: the same branch, so PR #1 updates. Default; nobody objected.
- Push `local/windows-fixtures` (3 commits, only in `C:\dev\voxxa`) so it can be merged right after the reconcile merge (A15).
- ADR numbering: the next free number is **ADR-012**. Prompts that say "write ADR-011" use the next free number.

### For the next session (Q1, after acceptance)
Execute Decision 1 **as amended**, path by path from Appendix A, in the A14 order: resolve → fmt → clippy → test → deny → budgets, green in `ci.yml` on all three OSes. The SDE notes are in `decisions/sde/_index.md` § Q0 verification. Do not fix the audit defects in Q1.

**Process rule (user, 2026-11-11):** every prompt in the pack (Q1…Q27) also runs **independently through ponytail** (github.com/dietrichgebert/ponytail) in addition to the 10x-team step.
- Use `ponytail` (full) for the step itself.
- Add `ponytail-review` for code steps and `ponytail-audit` for discovery and close steps.
- Record the result in `reviews/YYYY-MM-DD-ponytail-<Q>.md` and adjudicate disagreements.

---

## Handoff History
- 2026-11-11 Q0 gate: the user accepted ADR-010 as amended; ADR-011; cuts deferred until after Q2; Q2 widened to cover wrong-slide defects.
- 2026-11-11 Q0: claims verified, ADR-010 path table + amendments A1–A15, ponytail lane; stopped at the user gate.
- 2026-11-11 re-planning: May branch discovered; ADR-010 Proposed; prompt pack regenerated for /10x-Team:10x-team.
- 2026-10-24 init → Discovery: `.10x/` created.
- 2026-10-24 Discovery → Strategy: codebase scanned; critical finding: model never loaded, VAD never initialized.
- 2026-10-24 Strategy → Design → Planning: specs + ADR-001…009 + prompt plan written in one pass after five-angle research.
