# engineering-manager — Index

Feature files are added per `<feature-slug>` as phases complete.

## Cross-cutting rules
- Line budgets are tokei code lines per row (`scripts/loc_budget.py`); caps only go down; a new module gets a new recorded row through an ADR (ADR-010 Decision 3; the precedents ADR-013/014/015/016/017/019 § 10/020 § 5/028). New rows since 2026-11-27 land at 3 × the engineers' estimate with the estimate recorded beside the cap (FA-54); the rows proposed before that keep their caps and ask for 3 × the measured number only on a measured overrun (FA-74 (b)).
- Shares inside a tight row are ceilings spent in merge order; a step's Phase 2 check reads what is left before it starts (ADR-028 amendment 1 § 5).
- Lab tooling (`scripts/lab/`), tests and `scripts/qa/`, `scripts/release/` are uncounted; church-side code is counted (`kit/tools/` row).

## Active features
- `church-autoadvance` — `church-autoadvance.md`: 2026-10-24 delivery plan (8 increments, 33 prompts); 2026-11-27 **V7** (the wiring row 1,288 / 1,438, the `vad/` move, the rows onset 750 / input chain 600 / position 900 / health check 450, "everything else" 900 / 904; the before/after table and the six decisions).
