# ADR-010: Adopt the May implementation branch as the code base, keep Increment 0's governance, re-scope the plan to the gaps

**Status:** Proposed (needs the user's approval; prompt Q0 presents it) · **Date:** 2026-11-11 · **Feature:** church-autoadvance · **Reversibility:** Costly (a merge, then every later commit builds on it) · **Owner:** nategroks

## Context
Discovery in October ran on `main` only. `origin/claude/affectionate-lamport-8al8B` (44 commits, 2026-05-28 → 06-01, same base as `main`) already implements presenter drivers for ProPresenter 7.9+ REST and 7.0–7.8 WS, FreeShow, OpenLP v2, OpenSong and keystroke profiles; nine setlist importers; Planning Center import; a smart-blanking conductor with parallel song detection and unit tests; Silero VAD; discovery; a local HTTP API; diagnostics; a stage display; Flatpak packaging; and signing scaffolds. It was never merged, its README still says "voice dictation", and it fails the Increment 0 quality bar (51 fmt diffs, 15+ `cargo deny` advisories, line budgets exceeded). Our branch holds the strategy, ADR-001…009, positioning, GPL licence, `cargo deny`, CI and the icon fix, on a codebase that cannot yet transcribe. Rebuilding what the May branch has would cost ~15 prompts and weeks; its conductor already embodies the product's core rule ("a bad blank beats a wrong slide").

## Decision
1. **Code base:** merge `claude/affectionate-lamport-8al8B` into `claude/trusting-newton-v71ebu` (merge commit, no history rewrite). Conflict rule: take the May branch for all of `src-tauri/src/`, `src/`, `index.html`, `stage.html`, `examples/`, `docs/{INTEGRATIONS,LINUX,REVIEW,SETLIST_FORMATS}.md`, `packaging/flatpak`; take ours for `README.md`, `docs/index.html`, `LICENSE`, `DCO`, `CONTRIBUTING.md`, `deny.toml`, `.github/workflows/ci.yml`, `scripts/`, `src-tauri/icons/`, `package.json` licence field, and the Cargo `description`/`license` fields; combine `.github/workflows/build.yml` (their signing scaffolds + our release body); take theirs for `Cargo.toml` dependencies, then apply our removals (`tauri-plugin-store`, unused by the GUI) and the security refresh needed for `cargo deny`.
2. **ADRs adjusted, not reversed:** ADR-001 stands as "a `voxxad` CLI over `voxxa_lib`" rather than a GUI rewrite; the GUI stays in-process. ADR-003's intent (one readable config, reproducible support bundle) is met first by exporting the operator state into the diagnostics bundle and a `--replay` session log; the TOML migration is deferred until support tickets show it is needed. ADR-008 keeps AppImage as the first-class Linux artefact and keeps the existing Flatpak manifest as a second channel. ADR-009's section-aware graph and evidence accumulator are implemented only if `voxxa-eval` shows the conductor exceeds the false-advance budget on fixtures.
3. **Line budgets re-based per module** (`scripts/loc_budget.py`): conductor ≤ 1,200; presenters ≤ 2,000 total and ≤ 400 per driver; importers ≤ 1,600; everything else in `src-tauri/src` ≤ 2,500; GUI ≤ 3,000 with no runtime npm dependencies. Budgets can only go down after this ADR.
4. **The one-screen spec becomes "Simple mode"**: default view with the three controls, status light and Call for help; existing panels live behind "Advanced". No existing feature is deleted to satisfy the spec.
5. **Plan re-scope:** the regenerated prompt pack in `specs/2026-11-11-prompts-10x-team.md` replaces §7 of the October delivery plan. Increment 0 stays done; the sequence is now: reconcile → measure → volunteer-ready Windows → read-back and fixtures with pilots → scripture → bake-off, Store, support → macOS → Linux/1.0.

## Alternatives
| Alternative | Why not |
|---|---|
| Ignore the May branch and continue the October plan | Rebuilds ~5,400 lines that exist and are partly tested; the user pays twice. |
| Adopt the May branch wholesale and drop the October ADRs | Loses the measurement discipline (no eval harness exists there), the licence/CI bar, the signing and Store strategy and the volunteer-first UI decision. |
| Cherry-pick modules from the May branch into ours | Every module depends on the conductor/AppState shape; a merge is cheaper and keeps history. |

## Consequences
Good: ~15 prompts removed from the plan; adapters, importers and Planning Center arrive on day one; the conductor's blanking behaviour is already the right product instinct. Bad: a larger surface to secure and maintain (38 commands, HTTP API, network lookups); budgets loosen; the merge is a real day of work. Risks: hidden defects in unreviewed code (its own `REVIEW.md` found six critical bugs in one pass) → prompt Q2 runs QA and security before any new feature; dependency advisories in `pdf-extract`/`rusqlite` may require replacing crates → handled in Q1 with `cargo deny` as the gate.

## Review trigger
Re-open if the merge cannot reach green CI within two prompts, or if `voxxa-eval` shows the conductor cannot meet the false-advance budget and ADR-009's tracker must replace it.
