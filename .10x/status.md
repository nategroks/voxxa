# Project Status

## Project: Voxxa — voice-following slide advancer for churches
**Current Phase:** Q3 implemented on `q3/base` (code head 912b978, 2026-11-12). Verifier round 3 left two items: Q3-01 is closed with a test (912b978) and Q3-02 is deferred to Q4 by a recorded decision. Next: push `q3/base`, draft PR against `main`, CI, merge, file the Q3 tickets; then Q4 (Recognizer trait and streaming hop).
**Last Updated:** 2026-11-12 by 10x-team orchestrator
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

- [x] 2026-11-11 **Q2** (Phase 5: QA Engineer + Security Engineer), with the ponytail lane: `reviews/2026-11-11-qa-report.md`, `reviews/2026-11-11-security-review.md`, `reviews/2026-11-11-q2-tickets.md`, `reviews/2026-11-11-ponytail-q2.md`.
  - **Method:** simulate mode (user decision): GUI driven over WebView2 remote debugging, presenter on a local mock, no mic, no live keystrokes; one real Whisper Tiny download; transcription on locally synthesised PD speech. 35 of 38 commands exercised; HTTP API with and without a token; device loss by code reading only.
  - **Fixed with tests** on `q2/10x` (1568810..b09373d, 24 commits): Q0 audit 1–13, 15, 16 and the ADR-007 fixtures, plus PDF page split, single-instance guard and one verifier finding (Q2-44). Two fix lanes (10x, ponytail), cross-reviewed; base 10x with ponytail grafts; adversarial verifier.
  - **Security:** HTTP API token mandatory and browser/rebinding requests refused (High, fixed); PDF abort (Critical, fixed); capability file least-privilege. No Critical/High open. Outbound counter counts every reqwest send; no credentials logged at Info.
  - **ADR-011:** the stage window cannot default to an audience output; copy and placement are a Low ticket.
  - **Gates on Windows** (rustc 1.99.1): frontend (clean export), fmt, clippy `-D warnings`, test (45 unit + 33 integration), `cargo deny`, budgets (no budget raised; everything else 2489/2500), dictat grep.
  - **Tickets:** 34 (14 medium, 20 low), ready to file as GitHub issues.
- [x] 2026-11-12 **Q2 ponytail-audit re-run** on 3446e09 (the first run's output was blocked): 16 findings; 3 new fix-now items (PT2-01 short or unrelated transcripts start a song or re-show lyrics, PT2-02 tray Blank inverts the Keystroke toggle, PT2-05 ProPresenter WS/OpenSong goto hits the wrong presentation) and one older wrong-slide defect found while checking b09373d (Prev below the first song) fixed with tests in f2519ba, 8846f36, a50ee6a, bde776b, b677d69; all 7 gates pass at b677d69 (53 unit tests; budget 2482/2500); tickets now 44 (Q2-55 … Q2-64 added); the last wrong-slide path (a worship phrase during a prayer blank could show the next slide through the in-song advance) is fixed with a test in 4129dcd, and the PT2-02 focus check is now guarded by a pure-helper test (713f40f, part of Q2-60); a second verifier passed at 713f40f: all 7 gates pass (54 unit tests; budget 2483/2500, no budget raised), every mutation caught by a test or a gate (`reviews/2026-11-11-ponytail-q2.md` §8.6). Q2's "no open wrong-slide defect" is met again; the hold on the PR #1 push and on filing the tickets is lifted.

- [x] 2026-11-12 **Q3** `voxxad` CLI (Phase 2 check + Phase 4: ADR-013), with the ponytail lane: `reviews/2026-11-12-ponytail-q3.md`, `reviews/2026-11-12-q3-tickets.md`, `adrs/013-voxxad-cli-shared-loop-and-replay.md`, record in `decisions/sde/church-autoadvance.md`.
  - **Head:** `q3/base` @ **f2bcea1** (24 commits on 74f6bd2): `q3/10x` merged (f6d18b3), the comparator's 10x fix and six ponytail grafts (7cc75b2..1f5e31b), and test-only fixes from verifier rounds 1 and 2 (c54f16e..37dd632). `q3/ponytail` (3550b05) stays local for reference.
  - **Built:** one listening loop, `voxxa_lib::listen::run`, shared by the GUI and the new `voxxad` binary. Every conductor input goes through `Conductor::apply`. JSON-lines schema v1. A 14-day session log. `--replay` is byte-identical on action lines without the model. `--shadow` builds no presenter.
  - **Gates on Windows** (rustc 1.99.1): all 7 pass at f2bcea1.
  - **Tests:** 108 (54 unit + 54 integration; `voxxad_events` 19). None needs the Whisper model or an audio device.
  - **Budgets:** everything else 2,496/2,500 (+13); conductor 1,153/1,200; new row voxxad 531/600 (user decision, recorded deviation); GUI 2,951/3,000; no other row changed.
  - **Exit demo** (Whisper Tiny, public-domain speech): goto Amazing Grace/0 at 8.1 s, goto Amazing Grace/1 at 22.8 s, blank at 27.8 s, prayer shows nothing. Replay byte-identical (`cmp`). The ProPresenter mock's `/_log` stayed empty under `--shadow`.
  - **Verifiers:** round 1 (M2–M6) and round 2 (L1–L5, R2) closed with tests. **Round 3 at f2bcea1: verifier A passed; verifier B failed.** Open: Q3-01 (deleting the `--shadow` guard at `bin/voxxad/main.rs:312-314` passes every gate) and Q3-02 (three mutations of the GUI event sink in `commands/mod.rs` pass every gate).
  - **Close-out by the orchestrator (912b978):** Q3-01 closed with `shadow_never_connects_to_the_presenter` (model-free; the N9 mutation now fails it). Q3-02 deferred to Q4 (recorded in the ticket: no GUI behaviour change, no protection lost against 74f6bd2, not an exit criterion, does not fit the 4 lines left). All 7 gates pass again at 912b978: 109 tests. The change is test-only, so verifier A's exit demo at f2bcea1 still applies.
  - **Tickets:** 12 to file (6 medium, 6 low); Q3-01 closed before filing.

### Release readiness (2026-11-11, after Q2)
- **Code:** Q2's exit criteria are met. No open crash, data-loss, freeze or wrong-slide defect from the audit or Q2; no open Critical or High security finding.
- **Not yet ready for a live service.** The fixes are proven against mocks and synthetic audio only. Before a pilot leaves shadow mode, the founder must pass checklist M1–M14 in the QA report (real mic, unplug, Keystroke driver with PowerPoint/ProPresenter, FreeShow/OpenLP/OpenSong, displays, Stream Deck) and the security checklist.
- **Linux/macOS:** first judged by CI on PR #1 (new: single-instance plugin pulls zbus on Linux; timing-based download tests).
- **Before a public release:** Q2-28 (release pipeline pinning, `.asc`, updater) and the cut-list ADR.

### In Progress
- [x] 2026-11-12 CI green on PR #1 (head 69b18ce, 5/5 jobs); PR #1 merged into `main` as 7bd84a1; CI green on `main` (5/5). The "Deploy Website" workflow fails because GitHub Pages is not enabled (pre-existing since April; user decision).
- [ ] Q3 close-out (orchestrator).
  1. [x] Q3-01: model-free `--shadow` test added (912b978); the N9 mutation fails it.
  2. [x] Q3-02: deferred to Q4 by a recorded decision (tickets file).
  3. [x] All 7 gates on 912b978 (109 tests); test-only change, so the f2bcea1 exit demo stands.
  4. [ ] Push `q3/base`, open the draft PR against `main`, fix CI forward, merge when every job is green.
  5. [ ] File the 12 Q3 tickets as GitHub issues (label `q3`).

  The user pre-approved the push, PR, merge and filing in steps 4 and 5.
- [x] 2026-11-12 Filed the 43 open Q2 tickets as GitHub issues #2–#44 (label `q2`); Q2-60 was closed by 713f40f before filing.

### Pending
- [ ] User decision: post-Q2 cut-list ADR (ADR-012) — HTTP API, mDNS, Hymnary, PDF, stage display, Flatpak, signing scaffolds, unused plugins; evidence in the QA report §9
- [ ] Founder: QA checklist M1–M14 and the security checklist on the booth PC
- [ ] Increment 1′ (Q4–Q7) … per the 2026-11-11 prompt pack; next is Q4 (Recognizer trait and streaming hop)
- [ ] Founder: record two PD-hymn fixtures; apply to SignPath; open Partner Center; e-mail CCLI; form LLC; start research screener
- [ ] Phases 4–6 per increment

### Blocked
(none)

## Risks
- R8 **Closed (Q2).** Unreviewed May-branch code: Q2 ran QA and security over the merged base; all 12 Q0 must-fix defects and every new crash, data-loss and wrong-slide defect are fixed with tests; the rest are 43 tracked tickets (GitHub issues #2–#44).
- R10 **Closed in code (Q2).** Wrong slide on screen (audit 1–4, plus audit 5, PDF page split, single instance, Q2-44): fixed with dispatcher and conductor tests. Confirmation on real presentation apps moves to R14.
- R14 (new, Q2) Fixes proven only against mocks and synthetic audio. No real mic, keystroke driver, PowerPoint, ProPresenter, FreeShow, OpenLP or OpenSong was used. Mitigation: founder checklist M1–M14 before any pilot leaves shadow mode; promote Q2-20/Q2-22/Q2-47 if M3/M8 fail.
- R15 (new, Q2; updated Q3) Tight line budget. After Q3:
  - "everything else in src-tauri/src" is at 2,496/2,500 (4 lines left; it holds `listen.rs`, the shared loop);
  - conductor 1,153/1,200 (47 left);
  - the new `voxxad` row 531/600 (69 left; user decision, a recorded deviation from ADR-010 Decision 3);
  - GUI 2,951/3,000; presenters 1,149/2,000; importers 1,362/1,600.

  Mitigation: keep new tests in `src-tauri/tests/`. Q3-02 (deferred to Q4) must fit by simplifying code in the same bucket. Q4's streaming hop lives in `listen.rs`, so it needs an explicit budget decision before code (move code into a budgeted module or a recorded new row), never a silent raise.
- R13 (new, Q1) Fragile CI gates.
  - The GUI fits its budget with 49 lines to spare (2951/3000 after Q2), partly by avoiding a tokei 15 HTML apostrophe miscount.
  - CI installs the latest tokei, the latest stable Rust (new clippy lints under `-D warnings`) and a fresh advisory DB.
  - Mitigation (approved and done 2026-11-11): `ci.yml` pins Rust 1.99.1 and tokei 15.0.0. Bump them deliberately in their own PR.
- R11 (new, Q0) ADR-007/005 content in the May branch: copyrighted lyrics in `setlist.example.json`, a mislabelled "Public Domain" example, an MIT tag in the Flatpak metainfo. Mitigation: amendments A9/A10 in the Q1 merge.
- R12 (new, Q0) Toolchain-dependent clippy: 8 `double_must_use` errors from `async_trait` on rustc 1.99, and CI uses unpinned stable. Mitigation: fix in Q1. Consider pinning the toolchain.
- R9 Dependency advisories in pdf-extract/lopdf and rusqlite may force crate replacement — Q1 gates on cargo deny; PDF importer may go behind an off-by-default feature. Q2: pdf-extract panics on 4.8 % of mutated PDFs (now caught; stack overflow is not) — cut-list evidence
- R1 Recognition on sung vocals (Whisper WER 21–56% on singing) — mitigated by ADR-009 + eval harness in Inc 1; kill criterion week 14
- R2 Support conversion ≈ 1% — Setup & Calibration + Store listing as independent lines; decision gate month 12
- R3 CCLI/publisher objection to lyric indexing — no lyrics shipped; written question before launch
- R4 Vendors ship lyric-following natively — ship first on open platforms; contribute upstream
- R5 SmartScreen friction — Store listing + SignPath/Trusted Signing + winget
- R6 Founder on-call load — bounded Sunday window; MSP partners carry their churches
- R7 Windows-first vs Mac churches — unsigned previews; ADR-002 review trigger
