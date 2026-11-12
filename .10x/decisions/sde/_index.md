# SDE — Index

## Discovered (what had been built before 10x-team)
- ~1,000 LOC Rust, ~380 LOC JS/HTML, ~260 LOC legacy Python. Pipeline and defects: `cto/church-autoadvance.md`, `staff-engineer/_index.md`.

## Active features
- `church-autoadvance` — Increment 0 complete; **Q1 merge done (2026-11-11)**; merge record, dependency table and debt in `church-autoadvance.md`. Next: Q2.

## [DISCOVERED 2026-11-11] What the May branch already built
See `decisions/architect/_index.md`. Net effect on the plan: P1.1 (wire the loop) is obsolete; P1.2–P1.6 (recognizer trait, eval, CLI) remain; P2.x narrows to "simple mode in front of the existing panels" plus signing; P3.1–P3.4 narrow to read-back, in-app playlist import and recorded-traffic fixtures; P3.5 (Planning Center) exists; P4–P7 unchanged except Flatpak already exists. Regenerated prompts: `specs/2026-11-11-prompts-10x-team.md`.

### Q0 verification (2026-11-11), for the Q1 SDE
- Q1 executes ADR-010 Decision 1 **as amended (A1–A15) and only once Accepted**. The path-by-path sides are in ADR-010 Appendix A.
- **Six textual conflicts:**

  | File | Resolution |
  |---|---|
  | `build.yml` | ours in both `releaseBody` hunks |
  | `README.md` | ours, then A11 |
  | `Cargo.toml` | May's deps block, our `[package]` |
  | `Cargo.lock` | start from ours and let cargo add May's crates; no bare `cargo update` |
  | `commands/mod.rs` | theirs |
  | `vad/mod.rs` | May's `backend,` plus the auto-merged `impl Default` |

- **Also in Q1:**
  - delete `lib.rs:105`
  - zip → 9
  - drop `env_logger`
  - regenerate `gen/schemas`
  - PD rewrite of `setlist.example.json` and the OpenLyrics example
  - GPL tag in the Flatpak metainfo
  - rewrite `loc_budget.py` (A8)
- Known clippy work after `cargo fmt`: `commands:479` (`unsigned_abs`), `commands:486` (`?`), `pptx:82` (collapsible match), `keystroke:38` (derive Default), and 8× `double_must_use` at `presenters/mod.rs:148`.
- Do not fix the 12 audit defects in Q1; they belong to Q2.
