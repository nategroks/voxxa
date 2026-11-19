# qa — Index

Decisions per feature, newest first inside each file.

## Active features
- `church-autoadvance`
  - 2026-11-18 **Q6 close-out**: the user's baseline decision closed the blocking item; ponytail re-run compared and its scripted tests grafted; both verifiers passed at 97cc21b. QA position: ready to merge once CI is green. Final table: `reviews/2026-11-13-q6-eval-before-after.md` §7.
  - 2026-11-13 **Q6** conductor against the budget: six synthetic fixtures; before/after per fixture (false advances 1 → 0, p95 11.23 → 4.87 s); three verifier rounds; round 3 failed on one item, the `jump-be-thou-my-vision` baseline cells, which wait on the user. QA position: do not merge until the user decides. Summary: `reviews/2026-11-13-q6-eval-before-after.md`.
  - 2026-11-13 **Q5** `voxxa-eval` as the QA instrument (ADR-015); see `church-autoadvance.md`.
  - 2026-11-11 **Q2** verification of the merged base: simulate mode; 35/38 commands exercised; 18 fix-now items + Q2-44 fixed with failing-first tests; 34 tickets; ADR-011 stage window not audience-facing by default; cut-list evidence; founder checklist M1–M14. Report: `reviews/2026-11-11-qa-report.md`.
  - 2026-10-25 Windows verification of Increment 0: green with LLVM 20.1.8; bindgen 0.69 + LLVM 23 defect; build.yml YAML error. Install test pending.
