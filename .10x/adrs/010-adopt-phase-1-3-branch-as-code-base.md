# ADR-010: Adopt the May implementation branch as the code base, keep Increment 0's governance, re-scope the plan to the gaps

**Status:** Proposed; reviewed by Q0 on 2026-11-11, with amendments A1–A15 awaiting the user's approval · **Date:** 2026-11-11 · **Feature:** church-autoadvance · **Reversibility:** Costly (a merge, then every later commit builds on it) · **Owner:** nategroks

## Context
Discovery in October ran on `main` only. `origin/claude/affectionate-lamport-8al8B` (44 commits, 2026-05-28 → 06-01, same base as `main`) already implements presenter drivers for ProPresenter 7.9+ REST and 7.0–7.8 WS, FreeShow, OpenLP v2, OpenSong and keystroke profiles (plus a ProPresenter 6 WS variant); eight file-format setlist importers (nine formats counting native JSON; corrected by Q0); Planning Center import; a smart-blanking conductor with parallel song detection and unit tests; Silero VAD; discovery; a local HTTP API; diagnostics; a stage display; Flatpak packaging; and signing scaffolds. It was never merged, its README still says "voice dictation", and it fails the Increment 0 quality bar (51 fmt diffs, 15+ `cargo deny` advisories, line budgets exceeded; Q0 adds: it does not build on Windows because `icons/icon.ico` is a PNG, and clippy 1.99 reports 16 errors). Our branch holds the strategy, ADR-001…009, positioning, GPL licence, `cargo deny`, CI and the icon fix, on a codebase that cannot yet transcribe. Rebuilding what the May branch has would cost ~15 prompts and weeks; its conductor is designed around the product's core rule ("a bad blank beats a wrong slide"), though Q0 found it does not yet enforce it (see § Q0 review).

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

---

## Q0 review (2026-11-11): Architect, Staff Engineer, CTO, with an independent ponytail lane

**Method.** Four verifier agents read the May branch's code (worktree of `f7b56e6`) and checked every [DISCOVERED 2026-11-11] claim, and each group went to a refuting skeptic. One agent built the full path table against Decision 1. A separate ponytail lane (`ponytail` at level full, plus `ponytail-audit`) did the same review without seeing the 10x lane's output. A third agent adjudicated every path where the two lanes disagreed by reading `git diff 0a15de1 <tip> -- <path>`. Results:
- the full claim log: `reviews/2026-11-11-q0-claim-verification.md`
- the ponytail lane: `reviews/2026-11-11-ponytail-q0.md`
- the path table: Appendix A below

Nothing was merged and no code was changed.

**Verdict.** The direction of Decision 1 (merge, don't rebuild) holds. Both lanes agree on it. **The conflict rule as written is incomplete and in two places harmful**, so it needs amendments A1–A15 before Q1:
- "Take theirs for `Cargo.lock`/`Cargo.toml` dependencies" would silently revert f177fbe, which moved the tauri family to 2.12.5 and openssl to 0.10.82 to clear advisories. f177fbe was written *after* this ADR.
- "Take the May branch for all of `src-tauri/src/`" would drop the `impl Default` fixes that git auto-merges correctly, so clippy fails. It would also keep a `tauri-plugin-store` registration (`lib.rs:105`) for a crate that the same rule removes.

**What a real merge looks like** (`git merge-tree`, base `0a15de1`, 10 commits ours / 44 May):
- **Textual conflicts in 6 files only:** `build.yml`, `README.md`, `Cargo.toml`, `Cargo.lock`, `commands/mod.rs` and `vad/mod.rs`. `audio`, `lib` and `transcription` auto-merge.
- **Of the 110 differing paths:**

| Resolution | Paths |
|---|---|
| git's default is already correct | 54 |
| take the May version | 45 |
| combine both sides | 3 |
| take ours | 2 |
| regenerate | 5 |
| delete | 1 |

**Verified corrections to the [DISCOVERED] notes** (all survived a refuting skeptic):
- **The May tip does not build on Windows.** `src-tauri/icons/icon.ico` and `icon.icns` are PNGs, and tauri-winres fails with RC2175. Our d976698 fix is load-bearing, and the merge keeps it.
- **The conductor does not yet enforce "a bad blank beats a wrong slide".** After a blank, any unmatched speech (a prayer, an announcement) re-shows the current song's slide (`aligner/mod.rs:331-340`). `min_song_match = 40` sits below the score that random English text gets. And with a one-song setlist the softmax gate always passes. Reproduced with a scratch copy of `aligner/mod.rs`; the 16 existing tests still pass.
- **Driver capabilities:**

  | Driver | Capabilities |
  |---|---|
  | ProPresenter 7.9+ REST | blank, goto, query |
  | FreeShow | blank, goto |
  | OpenLP v2 | blank, query (no goto, by design) |
  | OpenSong | blank as a toggle, goto |
  | ProPresenter 6 WebSocket | the seventh kind, never mentioned in the notes |

  Only Goto is dispatched by capability. Blank and Unblank ignore `can_blank`, and the GUI never reads the flags.
- **Importers: eight file formats,** not nine.
- **HTTP API: the token is optional and blank by default,** and there is no Origin or Host check. With the API enabled and no token, any web page on the booth PC can POST `/api/v1/blank`, `/next` or `/listen/start`.
- **`tauri-plugin-store` is also registered at runtime** (`lib.rs:105`), not just declared. The shell, global-shortcut and updater plugins are registered and never used, and nothing calls the updater's `check()`.
- **Clippy (`--all-targets -D warnings`, rustc 1.99.1): 16 errors across 7 lints,** not 10. Eight of them are `double_must_use` coming from the `#[async_trait]` expansion at `presenters/mod.rs:148`. These depend on the toolchain, and CI uses unpinned stable.
- **The [DISCOVERED] `cargo deny` advisory list is confirmed** against a clone of rustsec/advisory-db. It also misses `webpki-root-certs` 1.0.6. rustls, rustls-webpki and zip 4.6.1 come from `tauri-plugin-updater` 2.10.1, which our 2.14.1 already fixes.
- **Line budgets:**
  - The architect's module figures are raw `wc -l` counts; the budgets use tokei code lines (aligner 716, presenters 1,121, importers 1,041, planning_center 246).
  - Under Decision 3, "everything else" is already **2,522 > 2,500**.
  - The GUI is 2,595 in `src/`, or 2,982 counting the root `index.html` and `stage.html`, which `loc_budget.py` does not count.
- **REVIEW.md's six critical bugs came from two passes, and all six are fixed in code.** Audio item 8 is half fixed: there is still no resampler, so a 48 kHz-only device cannot be opened.

**New material findings** (the ponytail audit and the missed-items sweep; all 12 audit must-fix items confirmed by a skeptic):
- (1) unmatched speech re-shows lyrics
- (2) keystroke/OpenSong unblank is the blank toggle, so the first detection blacks the screen
- (3) keystroke presses go to Voxxa's own focused window and loop
- (4) Goto sends the *setlist-global* slide index to apps that address a slide in the *active presentation*
- (5) any setlist reload restarts the song at slide 1
- (6) `download_model` holds the transcription lock for the whole download, so slide control freezes
- (7) there are no Tauri capability files, so every `listen()` (`plugin:event|listen`) should be rejected by the ACL. This is static analysis; one app launch settles it
- (8) capture requires a 16 kHz device
- (9) `is_running` sticks after a failed start
- (10) the HTTP API is open with no token
- (11) sync import commands run on the main thread; a pdf-extract panic can abort the app
- (12) a failed or partial model download is saved as "Downloaded"

Also:
- **ADR-007:** `setlist.example.json` ships lyrics of three contemporary songs, and `examples/how-great-thou-art.openlyrics` labels Hine's 1949/1953 English text "Public Domain".
- **ADR-005:** the Flatpak metainfo declares MIT.
- **Privacy:** every transcript is logged into the ring buffer that the diagnostics report embeds.

These go to Q2. None of them is fixed in the merge, except where an amendment says so.

### Proposed amendments to Decision 1 (A1–A15)
- **A1 `Cargo.lock`:** start from **ours**. Apply the resolved `Cargo.toml` and let cargo add only the crates the May branch adds. Never run a bare `cargo update`. Afterwards verify: tauri ≥ 2.12.5, tauri-plugin-updater 2.14.x, openssl ≥ 0.10.82, no `tauri-plugin-store`.
- **A2 dependency floor:** take **ours for the whole `[package]` table** (description, license, `rust-version = "1.90"`). Raise the Cargo.toml requirements to the post-f177fbe floor (`tauri = "2.12"`, `tauri-plugin-updater = "2.14"`, and shell/global-shortcut at their 2.12-family minimums), so a lock regeneration can never fall back.
- **A3 `src-tauri/src`:** resolve **only the conflicting hunks** in favour of the May branch, and keep git's auto-merge everywhere else.
  - `commands/mod.rs`: theirs in both hunks (ours was rustfmt only).
  - `vad/mod.rs`: May's `backend,` plus the auto-merged `impl Default`.
  - Delete `lib.rs:105` (`tauri_plugin_store`) together with the crate.
  - Never `git checkout --theirs` a whole file.
- **A4 fix-forward:** edits to May code inside the merge PR are allowed only to pass fmt, clippy, test, deny or budgets. Run `cargo fmt` after resolution; formatting hunks are never preserved by hand.
- **A5 the security refresh, defined:**
  - `zip = "2"` → `"9"` (adapt `importers/pptx.rs` and `importers/easyworship.rs` if the API moved)
  - remove the unused `env_logger`
  - then `cargo deny` decides `pdf-extract`/`lopdf`, the `rusqlite` bundled SQLite, `quick-xml` 0.36 and `webpki-root-certs`. A crate with no fixed release goes behind an off-by-default Cargo feature (as Q1 already says) or gets a dated `ignore` with a reason. Never a silent ignore.
- **A6 uncovered paths:**
  - `src-tauri/tauri.conf.json`, `vite.config.js` and `src-tauri/tests/` → May (coupled to `stage.html` and `examples/`)
  - `src-tauri/gen/schemas/*` → regenerate by building, and commit the output
  - `.10x/**` → ours
  - `SIGNING.md` → May, edited to say SignPath is primary and Microsoft Artifact Signing is the ADR-008 fallback
- **A7 exact paths:** the clauses name `src-tauri/deny.toml` and `scripts/loc_budget.py`, not `deny.toml` and `scripts/`.
- **A8 budgets in the same PR:** rewrite `scripts/loc_budget.py` before CI is judged.
  - Count in tokei code lines.
  - Count the root `index.html` and `stage.html` as GUI.
  - Drop the `-e adapters` exclusion, which matches nothing.
  - Count `planning_center/` with the importers (1,287 ≤ 1,600), which brings "everything else" to 2,276 ≤ 2,500.
  - Remove the unused runtime npm dependency `@tauri-apps/api` from `package.json`.
- **A9 ADR-007 content:** rewrite `setlist.example.json` as three public-domain hymns in the May `Setlist` shape (`aligner/mod.rs:16-30`). Replace `examples/how-great-thou-art.openlyrics` with a verified PD hymn and update `src-tauri/tests/importer_examples.rs:33-45`.
- **A10 licence declarations:** every licence string that arrives from the May branch becomes GPL-3.0-or-later (`packaging/flatpak/com.voxxa.app.metainfo.xml:5` says MIT).
- **A11 README:** take ours, then correct what the merged code contradicts:
  - "never shows a word on screen" → "never projects lyrics" (the operator window shows slide text)
  - "no cloud" → "no network traffic unless the operator starts it: model download, Planning Center, Hymnary"
  - mark Songs + Scripture, the three-position switch and Call for help as roadmap
  - move May's "Presenter drivers" section to `CONTRIBUTING.md`
  - drop "Rust 1.77+"
- **A12 `build.yml`:** take ours in both `releaseBody` hunks and keep every auto-merged May signing step. Pin `trusted-signing-cli` with `--locked --version`, and fix the comment claiming `signCommand` comes from `tauri.conf.json`.
- **A13 ADR-007 wording** (needs the user, question 3): the operator window has always shown slide text, and the May stage window shows current and next. Amend to "Voxxa never *projects* lyrics; operator-only previews on the booth machine are allowed", or ship the stage window disabled.
- **A14 done criterion and order:** resolve per Appendix A → `cargo fmt` → `clippy --all-targets -D warnings` → `cargo test` → `cargo deny check licenses advisories bans` → `loc_budget.py`, green in `ci.yml` on all three OSes. The review trigger ("two prompts") counts from the merge commit.
- **A15 `local/windows-fixtures`:** it has 3 commits on d976698 and exists only in the founder's checkout. It touches no merged path. Push it, then merge it into this branch right after the reconcile merge. Its `pd-hymns.json` (`{v,songs:[{sections}]}`) and `pco/plan-sample.json` (a digest, not JSON:API) do not fit May's parsers yet. Re-run the Windows verification (a003992) on the merged head.

### Where the lanes disagreed (for the user)
Ponytail would also **delete** these in the merge:
- the HTTP API
- mDNS discovery
- the Hymnary lookup
- the PDF importer
- the stage display
- Flatpak
- the signing scaffolds and `SIGNING.md`/`INTEGRATIONS.md`

It would also cut the unused shell, store and global-shortcut plugins (31 commands instead of 38) and set the budgets as a ratchet on post-cut counts. The adjudicator kept Decision 4 ("no existing feature is deleted") as the rule in force: cutting features is a product decision with its own ADR, not a merge resolution. Ponytail also argues to defer the `voxxad` CLI (ADR-001 as adjusted) until `voxxa-eval` needs it; that is a Q3 question.

### Open questions at the Q0 gate
1. Accept ADR-010 with amendments A1–A15?
2. Ponytail's cut list: keep everything (Decision 4 as written), or cut in a separate commit/ADR?
3. ADR-007 and on-screen text (A13).
4. Q2 scope: fix the wrong-slide defects (audit 1–4) now, or only crash and data-loss defects as Q2 says?

## Appendix A: every differing path (`git diff --name-status origin/claude/trusting-newton-v71ebu origin/claude/affectionate-lamport-8al8B`)

"Ours/May since base" = `git diff --name-status 0a15de1 <tip>`. "git default" = git's automatic result is the right one. "Ponytail" shows where the independent lane chose differently (bold); the adjudicated side is in the Side column.

| # | Path | Ours since base | May since base | Textual conflict | Side | ADR-010 clause | Note | Ponytail |
|---|---|---|---|---|---|---|---|---|
| 1 | `.10x/adrs/.gitkeep` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 2 | `.10x/adrs/001-headless-core-thin-gui.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 3 | `.10x/adrs/002-platform-sequence.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 4 | `.10x/adrs/003-single-config-and-jsonl-protocol.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 5 | `.10x/adrs/004-recognizer-behind-a-trait-bakeoff.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 6 | `.10x/adrs/005-license-gpl3-with-mit-protocol.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 7 | `.10x/adrs/006-adapters-native-apis-first-keystroke-fallback.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 8 | `.10x/adrs/007-text-licensing-no-copyrighted-text-shipped.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 9 | `.10x/adrs/008-distribution-and-signing.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 10 | `.10x/adrs/009-position-tracking-over-transcription.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 11 | `.10x/adrs/010-adopt-phase-1-3-branch-as-code-base.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 12 | `.10x/decisions/architect/_index.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 13 | `.10x/decisions/architect/church-autoadvance.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 14 | `.10x/decisions/cto/_index.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 15 | `.10x/decisions/cto/church-autoadvance.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 16 | `.10x/decisions/dba/_index.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 17 | `.10x/decisions/devops/_index.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 18 | `.10x/decisions/engineering-manager/_index.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 19 | `.10x/decisions/engineering-manager/church-autoadvance.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 20 | `.10x/decisions/product-manager/_index.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 21 | `.10x/decisions/product-manager/church-autoadvance.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 22 | `.10x/decisions/qa/_index.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 23 | `.10x/decisions/sde/_index.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 24 | `.10x/decisions/sde/church-autoadvance.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 25 | `.10x/decisions/security/_index.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 26 | `.10x/decisions/senior-engineer/_index.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 27 | `.10x/decisions/senior-engineer/church-autoadvance.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 28 | `.10x/decisions/sre/_index.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 29 | `.10x/decisions/staff-engineer/_index.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 30 | `.10x/decisions/staff-engineer/church-autoadvance.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 31 | `.10x/handoff-local-windows.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 32 | `.10x/handoff.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 33 | `.10x/reviews/.gitkeep` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 34 | `.10x/specs/.gitkeep` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 35 | `.10x/specs/2026-10-24-business-strategy.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 36 | `.10x/specs/2026-10-24-delivery-plan-and-prompts.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 37 | `.10x/specs/2026-10-24-research-report.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 38 | `.10x/specs/2026-10-24-software-strategy.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 39 | `.10x/specs/2026-10-24-user-research-plan.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 40 | `.10x/specs/2026-11-11-prompts-10x-team.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 41 | `.10x/status.md` | A | - | no | **git default** | NOT COVERED | Only on our side; git keeps it. | agrees |
| 42 | `.github/workflows/build.yml` | M | M | **yes** | **combine** | combine .github/workflows/build.yml (their signing scaffolds + our release body) | Both conflict hunks are inside releaseBody (merged-tree fd599904 lines 166-173 and 184-203). Take ours in both. Keep all of May's auto-merged steps: Apple API key, trusted-signing-cli, the --config override and GPG. The reason ponytail's 'ours' is rejected: it contradicts the explicit ADR-010 clause, and ADR-008's Windows bullet names Azure Trusted Signing as the SignPath fallback. Every step exits 0 when its secret is missing, so CI is unaffected. Follow-ups: (1) trusted-signing-cli is installed without --version, so pin it. (2) The env comment says 'signCommand in tauri.conf.json', but May src-tauri/tauri.conf.json has no signCommand (grep finds none); the override is injected via --config, so fix the comment. | **ours** |
| 43 | `.github/workflows/ci.yml` | A | - | no | **git default** | take ours for .github/workflows/ci.yml | Git keeps ours. It is the green bar: fmt --check, clippy --all-targets -D warnings and test on 3 OSes, then cargo deny (licenses, advisories, bans), then budgets via tokei. | agrees |
| 44 | `CONTRIBUTING.md` | A | - | no | **git default** | take ours for CONTRIBUTING.md | Git keeps ours. Move May README's 'Presenter drivers' developer section here. | agrees |
| 45 | `DCO` | A | - | no | **git default** | take ours for DCO | Git keeps ours. | agrees |
| 46 | `LICENSE` | A | - | no | **git default** | take ours for LICENSE | Git keeps ours (GPL-3.0-or-later). | agrees |
| 47 | `README.md` | M | M | **yes** | **ours** | take ours for README.md | 1 conflict hunk (fd599904 lines 16-44). Take ours, then fix four positioning claims the merged code contradicts. (1) 'never shows a word on screen itself' (line 3) and 'never displays lyrics': the operator window shows current and next slide text (May src/main.js:17-18), and so does the stage window. Reword to 'never projects lyrics to the congregation'. (2) 'no cloud': Hymnary lookup (hymnary/mod.rs:20, https://hymnary.org, user-initiated via the main.js:63 button), Planning Center and model download are network calls the user starts. Say so. (3) Songs+Scripture mode, the three-position switch and Call for help have no code in the May tree. Mark them as roadmap. (4) Move May's 'Presenter drivers' section to CONTRIBUTING.md. | agrees |
| 48 | `SIGNING.md` | - | A | no | **May** | NOT COVERED | Git takes theirs. Keep it, because build.yml keeps the scaffolds it documents (ponytail's delete assumed those steps were dropped). Edit the Windows section (SIGNING.md:69) to say SignPath is primary and Microsoft Artifact Signing is the ADR-008 fallback. Reconcile with docs/signing-policy.md when local/windows-fixtures lands. | **delete** |
| 49 | `docs/INTEGRATIONS.md` | - | A | no | **May** | take the May branch for docs/{INTEGRATIONS,LINUX,REVIEW,SETLIST_FORMATS}.md | Git takes theirs. Kept because http_api is kept (criterion 2, ADR-010 Decision 4). Its 'Security notes' section needs updating once Q2 makes the token mandatory (see the http_api row). | **delete** |
| 50 | `docs/LINUX.md` | - | A | no | **May** | take the May branch for docs/{INTEGRATIONS,LINUX,REVIEW,SETLIST_FORMATS}.md | Git takes theirs. Check it against ADR-008, where AppImage is first and Flatpak second. | agrees |
| 51 | `docs/REVIEW.md` | - | A | no | **May** | take the May branch for docs/{INTEGRATIONS,LINUX,REVIEW,SETLIST_FORMATS}.md | Git takes theirs. It lists the critical bugs that Q2 must re-verify. | agrees |
| 52 | `docs/SETLIST_FORMATS.md` | - | A | no | **May** | take the May branch for docs/{INTEGRATIONS,LINUX,REVIEW,SETLIST_FORMATS}.md | Git takes theirs. | agrees |
| 53 | `docs/index.html` | M | - | no | **git default** | take ours for docs/index.html | Only our side modified it. Git keeps ours. | agrees |
| 54 | `examples/amazing-grace.txt` | - | A | no | **May** | take the May branch for examples/ | Git takes theirs. Newton text (verses 1-4) is public domain. OK under ADR-007. | agrees |
| 55 | `examples/be-thou-my-vision.chordpro` | - | A | no | **May** | take the May branch for examples/ | Git takes theirs. Public-domain hymn (Byrne/Hull translation, 1905/1912). | agrees |
| 56 | `examples/how-great-thou-art.openlyrics` | - | A | no | **delete** | take the May branch for examples/ (conflicts with ADR-007) | Labels Stuart K. Hine's English text 'Public Domain' (lines 9-11). That text is a 1949/1953 copyrighted translation, so licence status needs confirming. Replace with a PD hymn in OpenLyrics format and update src-tauri/tests/importer_examples.rs:35. | agrees |
| 57 | `index.html` | - | M | no | **May** | take the May branch for index.html | Git takes theirs. Decision 4 ('Simple mode') is later work, not part of the merge. | agrees |
| 58 | `package.json` | M | - | no | **git default** | take ours for package.json licence field | Only ours changed it (+license GPL-3.0-or-later). May did not change package.json. Git keeps ours. | agrees |
| 59 | `packaging/flatpak/build-local.sh` | - | A | no | **May** | take the May branch for packaging/flatpak | Git takes theirs. ADR-010 Decision 2 explicitly keeps the Flatpak manifest as a second channel, which overrides ADR-008's 'Flathub only if asked'. Ponytail's delete contradicts that and criterion 2. No CI job builds it. | **delete** |
| 60 | `packaging/flatpak/com.voxxa.app.desktop` | - | A | no | **May** | take the May branch for packaging/flatpak | Git takes theirs. | **delete** |
| 61 | `packaging/flatpak/com.voxxa.app.metainfo.xml` | - | A | no | **May** | take the May branch for packaging/flatpak | Git takes theirs, but it needs a mandatory governance fix: line 5 says <project_license>MIT</project_license>. It must be GPL-3.0-or-later (ADR-005, LICENSE, Cargo.toml license). Also check the <summary> (line 7, 'Voice-driven slide control for worship services') and the description against the church positioning. | **delete** |
| 62 | `packaging/flatpak/com.voxxa.app.yml` | - | A | no | **May** | take the May branch for packaging/flatpak | Git takes theirs. | **delete** |
| 63 | `scripts/loc_budget.py` | A | - | no | **ours** | take ours for scripts/ (contradicted by Decision 3) | Git keeps ours, but ours enforces core Rust <=3000 and GUI <=800, and excludes a non-existent 'adapters' dir (scripts/loc_budget.py:7-9). The May tree has 6,764 raw Rust lines in src-tauri/src and 2,992 raw lines in src/ (wc -l; tokei code count unverifiable: tokei not installed). The budgets job will very likely fail unless the merge commit re-bases budgets per Decision 3 (presenters/, importers/, conductor). | agrees |
| 64 | `setlist.example.json` | - | M | no | **regenerate** | NOT COVERED | Git silently takes theirs. May ships lyrics from How Great Is Our God, the My Chains Are Gone chorus and Way Maker, all contemporary copyrighted songs (ADR-007). The base file already had 3 lines of the first. Rewrite it with 3 public-domain songs in the shape {setlist:[{title,slides:[{id,text}]}]}, because src-tauri/tests/importer_examples.rs:73 asserts 3 songs. Sources: May examples/amazing-grace.txt (Newton verses), examples/be-thou-my-vision.chordpro, and the PD hymn that replaces how-great-thou-art. local/windows-fixtures pd-hymns.json has only 2 songs (titles at lines 5 and 26), so it is not enough on its own. | agrees |
| 65 | `src-tauri/Cargo.lock` | M | M | **yes** | **regenerate** | NOT COVERED | 18 conflict hunks (fd599904). Start from ours (tauri 2.12.5, updater 2.14.1, openssl 0.10.82, zip 9). Let cargo resolve only the crates May adds; no bare cargo update. Taking theirs would revert f177fbe: the local registry holds tauri-2.10.3, zip-2.4.2 and zip-4.6.1 from May builds. Gate: cargo deny check. | agrees |
| 66 | `src-tauri/Cargo.toml` | M | M | **yes** | **combine** | take theirs for Cargo.toml dependencies, then apply our removals (tauri-plugin-store) and the security refresh; take ours for description/license | 1 hunk (fd599904 lines 52-78): May's added-deps block against nothing on ours. Take theirs. Auto-merge already does three things: [package] is ours (description, license GPL-3.0-or-later, rust-version 1.90); tauri-plugin-store is gone; rmcp/mcp is gone. grep of the merged Cargo.toml and lib.rs finds no store or mcp. Security refresh: change zip "2" to "9" so the importers match the updater's zip and the advisories cleared in f177fbe stay cleared. Whether zip 2.4.2 is in the RUSTSEC-2026-0367..0369 ranges is unverifiable offline (no advisory-db locally). Whether zip 9's API matches pptx.rs:16 (ZipArchive::new) and the 'deflate' feature name is unverifiable (zip 9 not in the local registry). Keep axum, mdns-sd and pdf-extract (criterion 2). Ponytail's removals of shell and global-shortcut are base deps on both sides, not part of this merge. | agrees |
| 67 | `src-tauri/deny.toml` | A | - | no | **git default** | take ours for deny.toml (ADR names 'deny.toml'; real path is src-tauri/deny.toml) | Git keeps ours, in the f177fbe form without the zip ignores. Add no ignores just to pass the May deps. If pdf-extract or rusqlite advisories fire, handle them per ADR-010 Consequences (Q1). | agrees |
| 68 | `src-tauri/gen/schemas/acl-manifests.json` | M | - | no | **regenerate** | NOT COVERED | Generated by tauri-build. Git keeps ours (2.12, no store). May adds no tauri plugins, so it may come out unchanged. Rebuild and commit whatever tauri-build writes. | agrees |
| 69 | `src-tauri/gen/schemas/desktop-schema.json` | M | - | no | **regenerate** | NOT COVERED | Same as acl-manifests.json. | agrees |
| 70 | `src-tauri/gen/schemas/linux-schema.json` | M | - | no | **regenerate** | NOT COVERED | Same as acl-manifests.json. | agrees |
| 71 | `src-tauri/icons/icon.icns` | M | - | no | **git default** | take ours for src-tauri/icons/ | Git keeps ours (the d976698 icon fix). | agrees |
| 72 | `src-tauri/icons/icon.ico` | M | - | no | **git default** | take ours for src-tauri/icons/ | Git keeps ours (the d976698 icon fix). Ponytail reports an RC2175 build failure with the base ico. Not re-verified here. | agrees |
| 73 | `src-tauri/src/aligner/mod.rs` | - | M | no | **May** | take the May branch for all of src-tauri/src/ | Git takes theirs (conductor, +832/-?). Ours did not change it. | agrees |
| 74 | `src-tauri/src/audio/mod.rs` | M | M | no | **git default** | take the May branch for all of src-tauri/src/ (literal reading would drop our change) | Clean merge. Our #[derive(Default)] + Self::default() (clippy new_without_default) survives in the merged tree, line 16. Taking theirs wholesale would drop it and fail clippy -D warnings (May audio/mod.rs:22 has new() with no Default). | agrees |
| 75 | `src-tauri/src/commands/mod.rs` | M | M | **yes** | **May** | take the May branch for all of src-tauri/src/ | 2 conflict hunks (fd599904 lines 124-147 and 336-403). Ours changed only rustfmt formatting, around the old aligner and text_insert path, which May removed. Take theirs in both hunks, then run cargo fmt and fix clippy findings. Likely candidates, unverified because clippy was not run: :479 'delta.abs() as usize' (cast_abs_to_unsigned) and :486-488 'if let Err(e) = r { return Err(e); }' (question_mark). Keep the hymnary, discovery, stage and http_api commands. Ponytail's deletions conflict with criterion 2 and ADR-010 Decision 4. | agrees |
| 76 | `src-tauri/src/diagnostics/mod.rs` | - | A | no | **May** | take the May branch for all of src-tauri/src/ | Git takes theirs. ADR-003 says to export operator state into this bundle. | agrees |
| 77 | `src-tauri/src/discovery/mod.rs` | - | A | no | **May** | take the May branch for all of src-tauri/src/ | Git takes theirs (mdns-sd). This is May functionality. Cutting it would be a feature decision for a separate commit with its own ADR, not a merge resolution. | **delete** |
| 78 | `src-tauri/src/http_api/mod.rs` | - | A | no | **May** | take the May branch for all of src-tauri/src/ | Git takes theirs. It binds 127.0.0.1 and is opt-in: started only by the toggle (main.js:1555-1562), and enabled is not restored on launch (main.js:1582-1587). Security defect, confirmed: with no token, check_token returns Ok (http_api/mod.rs:116-118), and there is no Origin or CORS handling, so while the API is enabled any web page on the booth PC can POST /api/v1/next or /blank. This is a required Q2 fix (token mandatory or reject requests with an Origin header) before a release. It is not a reason to delete the module in the merge. | **delete** |
| 79 | `src-tauri/src/hymnary/mod.rs` | - | A | no | **May** | take the May branch for all of src-tauri/src/ | Git takes theirs. Makes outbound calls to https://hymnary.org (line 20) only when the user clicks search (main.js:63). It returns PD hymn text only, so it is consistent with ADR-007. The README 'no cloud' wording must allow user-initiated lookups. | **delete** |
| 80 | `src-tauri/src/importers/chordpro.rs` | - | A | no | **May** | take the May branch for all of src-tauri/src/ | Git takes theirs. | agrees |
| 81 | `src-tauri/src/importers/easyworship.rs` | - | A | no | **May** | take the May branch for all of src-tauri/src/ | Git takes theirs (rusqlite bundled). | agrees |
| 82 | `src-tauri/src/importers/mod.rs` | - | A | no | **May** | take the May branch for all of src-tauri/src/ | Git takes theirs. | agrees |
| 83 | `src-tauri/src/importers/openlyrics.rs` | - | A | no | **May** | take the May branch for all of src-tauri/src/ | Git takes theirs (quick-xml 0.36). | agrees |
| 84 | `src-tauri/src/importers/opensong.rs` | - | A | no | **May** | take the May branch for all of src-tauri/src/ | Git takes theirs. | agrees |
| 85 | `src-tauri/src/importers/pdf.rs` | - | A | no | **May** | take the May branch for all of src-tauri/src/ | Git takes theirs (pdf-extract 0.8.2, license MIT). Advisory risk is already handled by ADR-010 Consequences: if cargo deny fails, replace or drop the crate in Q1. It is not cut pre-emptively. | **delete** |
| 86 | `src-tauri/src/importers/pptx.rs` | - | A | no | **May** | take the May branch for all of src-tauri/src/ | Git takes theirs. Uses zip 2. Bumping zip to 9 for deny may need API edits here (unverified). | agrees |
| 87 | `src-tauri/src/importers/pro7.rs` | - | A | no | **May** | take the May branch for all of src-tauri/src/ | Git takes theirs. | agrees |
| 88 | `src-tauri/src/importers/txt.rs` | - | A | no | **May** | take the May branch for all of src-tauri/src/ | Git takes theirs. | agrees |
| 89 | `src-tauri/src/lib.rs` | M | M | no | **git default** | take the May branch for all of src-tauri/src/ (literal reading breaks the build) | Clean merge. It keeps our removal of tauri_plugin_store and mod mcp; merged plugins are at lines 104-106. Taking May's lib.rs wholesale would bring back .plugin(tauri_plugin_store...) (May lib.rs:105) while Cargo.toml drops the crate, which is a compile error. | agrees |
| 90 | `src-tauri/src/net_stats/mod.rs` | - | A | no | **May** | take the May branch for all of src-tauri/src/ | Git takes theirs. | agrees |
| 91 | `src-tauri/src/planning_center/mod.rs` | - | A | no | **May** | take the May branch for all of src-tauri/src/ | Git takes theirs. Parses the PCO JSON:API shape (data/attributes/included). windows-fixtures' fixtures/pco/plan-sample.json is a flattened digest and cannot feed it. | agrees |
| 92 | `src-tauri/src/presenters/freeshow.rs` | - | A | no | **May** | take the May branch for all of src-tauri/src/ | Git takes theirs. | agrees |
| 93 | `src-tauri/src/presenters/keystroke.rs` | - | A | no | **May** | take the May branch for all of src-tauri/src/ | Git takes theirs (enigo). Replaces text_insert. | agrees |
| 94 | `src-tauri/src/presenters/mod.rs` | - | A | no | **May** | take the May branch for all of src-tauri/src/ | Git takes theirs (PresentationController trait). | agrees |
| 95 | `src-tauri/src/presenters/openlp_v2.rs` | - | A | no | **May** | take the May branch for all of src-tauri/src/ | Git takes theirs. | agrees |
| 96 | `src-tauri/src/presenters/opensong.rs` | - | A | no | **May** | take the May branch for all of src-tauri/src/ | Git takes theirs. | agrees |
| 97 | `src-tauri/src/presenters/propresenter7_rest.rs` | - | A | no | **May** | take the May branch for all of src-tauri/src/ | Git takes theirs. | agrees |
| 98 | `src-tauri/src/presenters/propresenter7_ws.rs` | - | A | no | **May** | take the May branch for all of src-tauri/src/ | Git takes theirs (tokio-tungstenite 0.24). | agrees |
| 99 | `src-tauri/src/text_insert/mod.rs` | - | D | no | **git default** | take the May branch for all of src-tauri/src/ | May deleted it and ours left it unchanged, so git deletes it. The reviewers agree on the result. Its only remaining caller is the ours side of the commands/mod.rs conflict, which is discarded. | **delete** |
| 100 | `src-tauri/src/transcription/mod.rs` | M | M | no | **git default** | take the May branch for all of src-tauri/src/ (literal reading would drop our change) | Clean merge. Our #[derive(Default)]/#[default] on WhisperModel and impl Default for TranscriptionEngine survive (merged lines 6, 11, 69). May keeps a manual impl Default for WhisperModel (May transcription/mod.rs:51), which clippy -D warnings flags as derivable_impls. | agrees |
| 101 | `src-tauri/src/tray/mod.rs` | - | M | no | **May** | take the May branch for all of src-tauri/src/ | Git takes theirs. | agrees |
| 102 | `src-tauri/src/vad/mod.rs` | M | M | **yes** | **combine** | take the May branch for all of src-tauri/src/ (only for the conflicting hunk) | 1 hunk (fd599904 lines 76-82) in VadEngine::new's struct literal. Take May's 'backend,'. Drop ours initialized/state_h/state_c: those fields do not exist in May's struct (vad/mod.rs:40-49). Keep the auto-merged impl Default for VadEngine (fd599904 lines 51-55); May has new() without Default, so clippy new_without_default would fire. | agrees |
| 103 | `src-tauri/tauri.conf.json` | - | M | no | **May** | NOT COVERED | Git takes theirs: the 'overlay' window is replaced by a hidden 'stage' window at stage.html. macOSPrivateApi:true (line 13) matches both sides' macos-private-api feature. Follow-up for Q2: csp is null (line 47) and withGlobalTauri is true (line 14). | agrees |
| 104 | `src-tauri/tests/importer_examples.rs` | - | A | no | **May** | NOT COVERED (the rule says src-tauri/src/, not src-tauri/tests/) | Take theirs, then make one required edit: replace parses_how_great_thou_art_openlyrics (lines 33-45) with a test against the PD OpenLyrics hymn that replaces examples/how-great-thou-art.openlyrics. Keep example_setlist_json_loads_cleanly (asserts 3 songs at :73) and make it pass with the regenerated setlist.example.json. | agrees |
| 105 | `src/main.js` | - | M | no | **May** | take the May branch for src/ | Git takes theirs. | agrees |
| 106 | `src/stage.js` | - | A | no | **May** | take the May branch for src/ (conflicts with ADR-007 as worded) | Shows current and next slide text (lines 37-38) in a hidden, operator-toggled window (commands toggle_stage_display). Base already showed the same text in the main operator window (base main.js:14-15), so this is not a new ADR-007 breach. Keep it under ADR-010 Decision 4 ('No existing feature is deleted'), on condition that ADR-007 is reworded before the merge (see rule_gaps). If the user rejects that wording, the fallback is to keep the code and remove the toggle from the UI, not to delete the files. | **delete** |
| 107 | `src/styles/main.css` | - | M | no | **May** | take the May branch for src/ | Git takes theirs. | agrees |
| 108 | `src/styles/stage.css` | - | A | no | **May** | take the May branch for src/ | Goes with the stage display. | **delete** |
| 109 | `stage.html` | - | A | no | **May** | take the May branch for stage.html | Goes with the stage display. Same ADR-007 condition as src/stage.js. | **delete** |
| 110 | `vite.config.js` | - | M | no | **May** | NOT COVERED | Git takes theirs. It adds the stage.html rollup input, which the stage window needs. import.meta.dirname needs Node >= 20.11; ci.yml uses Node 22. | **ours** |
