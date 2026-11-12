# Current Handoff

## From: Q2 (QA Engineer + Security Engineer, 2026-11-11) → To: the user (cut-list ADR-012), then Q3
**Date:** 2026-11-11
**Phase:** Phase 5 Verification done → user decision gate → Increment 1′ (Q3 `voxxad` CLI)

### What Q2 did
- Verified the merged base (1568810) in simulate mode and fixed every crash, data-loss, freeze, listening-impossible and wrong-slide defect found, each with a test that fails without the fix: `reviews/2026-11-11-qa-report.md`.
- Security review with STRIDE-lite over 10 boundaries; S1 (HTTP API, High) and S3 (PDF abort, Critical) fixed; no Critical or High open: `reviews/2026-11-11-security-review.md`.
- Ran as the user's ponytail process: a QA lane and a Security lane, a skeptic, two independent fix lanes (`q2/10x`, `q2/ponytail`), cross-reviews, comparator (base 10x + ponytail grafts), finisher and adversarial verifier. The verifier's one finding (Q2-44) was fixed by the orchestrator: `reviews/2026-11-11-ponytail-q2.md`.
- 34 tickets (14 medium, 20 low), ready to file: `reviews/2026-11-11-q2-tickets.md`.
- Code head: `q2/10x` @ b09373d (24 commits on 1568810), plus the Q2 state commit. `q2/ponytail` (0d3162f) stays local for reference.
- **Gates on Windows:** all seven pass; 45 unit + 33 integration tests; no budget raised.
- **Ponytail-audit re-run (2026-11-12, on 3446e09):** fixed with tests PT2-01, PT2-02, PT2-05 and the older Prev-below-the-first-song defect (f2519ba..b677d69, code head **b677d69**, all 7 gates pass); 10 new tickets Q2-55 … Q2-64 (44 in all); **one fix-now item is open**: a worship phrase during a prayer blank can show the next slide through the in-song advance path; fix it with its test (`reviews/2026-11-11-ponytail-q2.md` §8.6) and re-verify before the PR #1 push and before filing tickets.

### Exit criteria
- Both reviews committed: **met** (this state commit).
- No open crash, data-loss or wrong-slide (audit 1–4) defect: **met** in code; real-app confirmation is the founder checklist (risk R14).
- Security findings triaged as fix-now (done) or ticket: **met**.

### Next user decision: the post-Q2 cut-list ADR (ADR-012, the next free number)
Q2 gathered facts only (QA report §9). The features and what Q2 found:
- **HTTP API:** works; now token-mandatory and browser-proof; 5 unique crates; no pilot request recorded; generated token is not kept across launches (Q2-51).
- **mDNS Auto-detect:** not run (Windows Firewall prompt); 6 unique crates; spoofable LAN answers.
- **Hymnary:** broken (403).
- **PDF importer:** works since Q2-06; 27 unique crates; panics on 4.8 % of mutated inputs (now caught; stack overflow is not).
- **Stage display:** works since Q2-09; never takes focus; ADR-011-compliant by default; copy ticket Q2-30.
- **Flatpak:** no workflow builds it.
- **Signing scaffolds:** `.asc` never released; updater unwired (Q2-28).
- **Unused plugins:** updater (17 unique crates, a second HTTP stack), shell (7), global-shortcut (3); Python prototype still in the tree.

### Open for the orchestrator (pre-approved by the user)
1. Push `q2/10x` to PR #1's branch and watch CI (first Linux/macOS run of the Q2 code: zbus from the single-instance plugin; timing-based download tests; PDF fixtures need `.gitattributes`). Fix forward.
2. Merge PR #1 into `main` once every CI job is green.
3. File the 34 tickets from `reviews/2026-11-11-q2-tickets.md` as GitHub issues.

### What Q3 must know
- **Dispatch has one path:** `commands::dispatch` → `dispatch::drive` / `dispatch::step` (pure, tested with fake presenters in `tests/dispatch.rs`). A `voxxad` non-shadow mode must reuse it, not call presenters directly. `Sent` replaced `last_dispatched_global`; `Sessions` replaced `is_running`.
- **Behaviour changes** to keep: the conductor blanks on the second unmatched transcript; keystrokes are refused while a Voxxa window has focus; goto drivers address the slide within the song and blank on an unconfirmed song change; empty slides are kept; setlists over 50 songs are refused; capture resamples device audio to 16 kHz (`audio::Resampler`, `choose_config`). Full list: QA report §6.
- **Line budget:** "everything else in src-tauri/src" is 2489/2500. Q3's `voxxad` binary and the audio-loop extraction will not fit; plan an explicit, recorded budget decision (smallest raise, recorded as a deviation) or move code into an already-budgeted module. Keep tests in `src-tauri/tests/`.
- **ADR numbering:** Q3's prompt says "write ADR-011 first" if needed; ADR-011 is taken and ADR-012 is reserved for the cut list, so Q3 uses ADR-013.
- **GUI checks:** `scripts/qa/cdp-smoke.mjs` + `scripts/qa/mock_pro7.py` (needs a scratch build with remote debugging; ticket Q2-53).

### Founder checklist (before any pilot leaves shadow mode)
QA report §10 (M1–M14: real mic, failed start, singing with the band, Keystroke toggle with PowerPoint/ProPresenter, own-window guard, unplug, Auto-detect, pptx order, real presentation apps, displays, tray, export, close and relaunch, Stream Deck) and security review §10. First clear `%LOCALAPPDATA%\com.voxxa.app` (Q2 left a mock presenter config at 127.0.0.1:18025); keep `%APPDATA%\voxxa\models\ggml-tiny.bin`.

### Housekeeping left by Q2
- A vite dev server (PID 1580) and esbuild (PID 7944) started for the CDP smoke still run from `C:/dev/q2-10x/node_modules`; stopping them was refused by the permission classifier. End them, then run `npm ci` in `C:/dev/q2-10x` (an `npm ci` attempt partly removed `node_modules`). Gate 1 passed on a clean `git archive` export of HEAD.
- Untracked scratch: `C:/dev/q2-find/target-scratch/` (QA logs, mocks, crash backtrace); CDP results in the session scratchpad.

### What an unattended run needs (answer to the user's question)
1. **Remote debugging for GUI checks:** permission to build a scratch copy with `additionalBrowserArgs: --remote-debugging-port=…` (the WebView2 env var is ignored), or a committed dev-only switch (ticket Q2-53).
2. **Process control:** a rule allowing agents to stop processes they started, by PID (`taskkill /PID`, `Stop-Process -Id`), including dev servers. Killing by image name is correctly refused. This was the one blocker in Q2.
3. **CDP dispatch rule:** pre-approve `node scripts/qa/cdp-smoke.mjs`-style calls against the local mock, including dispatch while disconnected (one such call was refused, so that case is untested).
4. **mDNS:** decide whether agents may trigger the Windows Firewall prompt, or accept code reading only.
5. **Hymnary:** accept that it is broken (Q2-29), or give a working endpoint.
6. **Planning Center:** a debug-only base-URL override to test against `fixtures/pco` (ticket Q2-35).
7. **Network:** keep crates.io access (a new crate was added) and the Hugging Face download.
8. **Shared state:** separate `CARGO_TARGET_DIR` and app-data profiles per lane (two lanes shared `%APPDATA%\voxxa` and the WebView2 profile in Q2); copy an exe aside before launching it from a shared target dir.
9. **PowerShell:** `.ps1` files are blocked by the execution policy; inline PowerShell works.

### Still open from Q1
- **A15 follow-ups:** `fixtures/setlists/pd-hymns.json` and `fixtures/pco/plan-sample.json` don't fit the May parsers; re-run the Windows verification (a003992) on the merged head (`handoff-local-windows.md`); finish reconciling `docs/signing-policy.md` with `SIGNING.md`.

**Process rule (user, 2026-11-11):** every prompt (Q2…Q27) also runs independently through ponytail. Run it as two lanes plus cross-review for code steps, or `ponytail` + `ponytail-audit` for verification and close steps. Record the result in `reviews/YYYY-MM-DD-ponytail-<Q>.md`.

---

## Handoff History
- 2026-11-11 Q2: QA + security verification in simulate mode; 18 fix-now items + verifier finding Q2-44 fixed with tests; two fix lanes (10x, ponytail) → 10x base with ponytail grafts; 34 tickets; ADR-011 stage window compliant by default; cut-list evidence for ADR-012.
- 2026-11-11 Q1: May branch merged per ADR-010 + A1–A15; two lanes (10x, ponytail) → 10x base with ponytail grafts; all local gates green; build.yml YAML fixed.
- 2026-11-11 Q0 gate: the user accepted ADR-010 as amended; ADR-011; cuts deferred until after Q2; Q2 widened to cover wrong-slide defects.
- 2026-11-11 Q0: claims verified, ADR-010 path table + amendments A1–A15, ponytail lane; stopped at the user gate.
- 2026-11-11 re-planning: May branch discovered; ADR-010 Proposed; prompt pack regenerated for /10x-Team:10x-team.
- 2026-10-24 init → Discovery: `.10x/` created.
- 2026-10-24 Discovery → Strategy: codebase scanned; critical finding: model never loaded, VAD never initialized.
- 2026-10-24 Strategy → Design → Planning: specs + ADR-001…009 + prompt plan written in one pass after five-angle research.
