# Project Status

## Project: Voxxa — voice-following slide advancer for churches
**Current Phase:** Q1 complete locally (2026-11-11): May branch merged per ADR-010 + A1–A15, all gates green on Windows; CI on PR #1 judges Linux/macOS. Next: Q2 (verification, widened).
**Last Updated:** 2026-11-11 by 10x-team orchestrator
**Health:** On Track

## Progress

### Done
- [x] Discovery Protocol; `[DISCOVERED]` decisions for CTO, Architect, Staff Engineer, DevOps, SDE
- [x] Deep research, 5 angles, confidence-rated: `specs/2026-10-24-research-report.md`
- [x] User research plan: `specs/2026-10-24-user-research-plan.md`
- [x] Phase 1 Strategy: `specs/2026-10-24-business-strategy.md`; `decisions/cto/`, `decisions/product-manager/`
- [x] Phase 2 Design: `specs/2026-10-24-software-strategy.md`; ADR-001…009; `decisions/architect/`, `decisions/staff-engineer/`
- [x] Phase 3 Planning: `specs/2026-10-24-delivery-plan-and-prompts.md`; `decisions/engineering-manager/`, `decisions/senior-engineer/`

- [x] User approved specs and ADR-001…009 (Accepted); GPL confirmed; US LLC confirmed; pilot church #1: The Lighthouse, Midlothian
- [x] Increment 0: P0.1 positioning, P0.2 dead weight + licence + deny, P0.3 CI (`decisions/sde/church-autoadvance.md`)

- [x] CI green on all five jobs (ubuntu/windows/macos fmt+clippy+test, cargo deny, line budgets) at d976698 on PR nategroks/voxxa#1; one real defect found and fixed on the way (icon.ico/icon.icns were PNGs)

- [x] 2026-11-11 Discovery of `origin/claude/affectionate-lamport-8al8B` (44 commits, May 2026, never merged): presenter drivers, importers, Planning Center, conductor, VAD, HTTP API, Flatpak. Notes in role indexes; ADR-010 Proposed; prompt pack regenerated for `/10x-Team:10x-team`.

- [x] 2026-11-11 PR #1 cargo deny went red on nine newly published advisories; fixed by openssl 0.10.82 and moving the tauri family to 2.12.5 (updater 2.14.1 → zip 9, rust-version 1.90, zip ignores deleted). CI green on f177fbe.

- [x] 2026-11-11 **Q0** (Phase 0–2: CTO, Architect, Staff Engineer), with an independent ponytail lane:
  - every [DISCOVERED] claim verified against the code: `reviews/2026-11-11-q0-claim-verification.md`
  - ADR-010 now carries the 110-path table (Appendix A) and amendments A1–A15
  - ponytail review and audit (12 must-fix items, all confirmed by a skeptic): `reviews/2026-11-11-ponytail-q0.md`
  - role indexes updated
  - no merge, no code

- [x] 2026-11-11 **Q0 user gate.** All four recommendations were taken:
  - ADR-010 Accepted as amended
  - feature cuts decided after Q2 in a separate ADR
  - ADR-011: never *project* lyrics, operator previews allowed (amends ADR-007)
  - Q2 widened (M → L) to fix the four wrong-slide defects; the prompt pack is updated

- [x] 2026-11-11 `local/windows-fixtures` merged into this branch (54183c8, clean). It brings the recording kit, PD-hymn setlists, PCO sample, SignPath policy and Windows QA of Increment 0. It went in ahead of Q1 because it touches none of the 110 merge paths: A15, executed early. The May-branch conflict set is unchanged at 6 files. The Drive folder linked in `fixtures/SOURCES.md` was checked: private, owner only.

- [x] 2026-11-11 **Q1** (Phase 4: SDE + Staff Engineer + DBA), run as two independent lanes (10x-team and ponytail), cross-reviewed, compared and verified: `reviews/2026-11-11-ponytail-q1.md`.
  - **Merge commit** 9987b62 (parents 97aa915 and f7b56e6), then 19 logical commits. The 10x lane is the base, with ponytail grafts.
  - **Gates on Windows** (rustc 1.99.1): frontend, fmt, clippy `-D warnings`, test (37 unit + 4 integration), `cargo deny` (no ignores), budgets and the dictat grep all pass. The adversarial verifier passed on round 1.
  - **Dependencies:** zip 9, quick-xml 0.41, pdf-extract 0.12 (lopdf 0.42; no feature gate needed), rusqlite 0.41 bundled, reqwest on `rustls-tls-native-roots`; env_logger and `@tauri-apps/api` removed; A2 floors. Tauri 2.12.5 and openssl 0.10.82 are kept.
  - **Content:** PD examples (It Is Well, Amazing Grace, Be Thou My Vision); GPL metainfo; README and website "never project lyrics" (ADR-011); `loc_budget.py` re-based (GUI 2,966/3,000).
  - **Pre-existing fix:** `build.yml` was invalid YAML (6ef2d13).
  - **Records:** `decisions/sde/church-autoadvance.md` (merge record) and `decisions/dba/church-autoadvance.md` (bundled SQLite).

### In Progress
- [ ] CI on PR #1 for the Q1 head: the first Linux and macOS builds of the merged tree.

### Pending
- [ ] Q0 discovery/approval → Q1 merge + CI bar → Q2 verification → Increment 1′ (Q3–Q7) … per the 2026-11-11 prompt pack
- [ ] Founder: record two PD-hymn fixtures; apply to SignPath; open Partner Center; e-mail CCLI; form LLC; start research screener
- [ ] Phases 4–6 per increment

### Blocked
(none)

## Risks
- R8 Unreviewed May-branch code. REVIEW.md's six critical bugs are fixed, but Q0 confirmed 12 more must-fix defects, including 4 wrong-slide defects. Q2 runs QA + security before any new feature; its scope is a gate question.
- R13 (new, Q1) Fragile CI gates.
  - The GUI fits its budget with 34 lines to spare, partly by avoiding a tokei 15 HTML apostrophe miscount.
  - CI installs the latest tokei, the latest stable Rust (new clippy lints under `-D warnings`) and a fresh advisory DB.
  - Mitigation: pin tokei and the toolchain in `ci.yml`. This needs the user's sign-off because it changes the CI bar.
- R10 (new, Q0) Wrong slide on screen: unmatched speech re-shows lyrics, keystroke unblank is a toggle, and Goto uses a setlist-global index. Mitigation: fix in Q2 before any pilot leaves shadow mode.
- R11 (new, Q0) ADR-007/005 content in the May branch: copyrighted lyrics in `setlist.example.json`, a mislabelled "Public Domain" example, an MIT tag in the Flatpak metainfo. Mitigation: amendments A9/A10 in the Q1 merge.
- R12 (new, Q0) Toolchain-dependent clippy: 8 `double_must_use` errors from `async_trait` on rustc 1.99, and CI uses unpinned stable. Mitigation: fix in Q1. Consider pinning the toolchain.
- R9 Dependency advisories in pdf-extract/lopdf and rusqlite may force crate replacement — Q1 gates on cargo deny; PDF importer may go behind an off-by-default feature
- R1 Recognition on sung vocals (Whisper WER 21–56% on singing) — mitigated by ADR-009 + eval harness in Inc 1; kill criterion week 14
- R2 Support conversion ≈ 1% — Setup & Calibration + Store listing as independent lines; decision gate month 12
- R3 CCLI/publisher objection to lyric indexing — no lyrics shipped; written question before launch
- R4 Vendors ship lyric-following natively — ship first on open platforms; contribute upstream
- R5 SmartScreen friction — Store listing + SignPath/Trusted Signing + winget
- R6 Founder on-call load — bounded Sunday window; MSP partners carry their churches
- R7 Windows-first vs Mac churches — unsigned previews; ADR-002 review trigger
