# Current Handoff

## User decisions for Q9W/Q9V (2026-11-20)
- **Spec:** `specs/2026-11-20-prompts-worshiptools-revision.md`, with its two founder addenda. **ADR-019** is Accepted. R0 (planning only) added Q9W and Q9V to the pack after Q9 and applied M1–M8.
- **D1 Order:** Q9W, then Q9V, then Q10. The "From: Q9 → To: Q10" section below still applies to Q10.
- **D2:** keys only when WorshipTools Presenter is in front (amber "Bring WorshipTools Presenter to the front."). MIDI is deferred and is the named next step.
- **T2 (end-of-song risk):** Right on a song's last slide moves Presenter to the next service item. Voxxa never sends Right past a song's last slide for this profile. A cross-song Goto presses nothing and asks "Put "<song>" live in Presenter." A test proves it (ADR-019 § 3).
- **D4/T3/T4:** `.` is removed (it does nothing). F10 clears the lyrics, but the unblank was not tested, so the profile reports `can_blank: false` and F10 is only the candidate.
- **D3 dropped:** A1 was wrong; no video plays during services. Q9V is the private fixture and the media guard only.
- **D5 Budget:** detection in the presenters row. The discovery call site and the dispatch change use the 6 lines left in "everything else", or **one** new budget row recorded in ADR-019 and `scripts/loc_budget.py`. Never raise an existing cap.
- **D6 Pre-approved:** R0: push `plan/worshiptools` and open a draft PR. Q9W and Q9V: push, draft PR, CI fixes, merge when green (R0's planning PR too), file tickets, stop processes the agents started (by PID).
- **Licensed test audio (addendum 1):**
  - The "See a Victory" .mp4 stays in Presenter's media folder. Agents never copy, upload, move or open it. The founder converts it to `fixtures/private/see-a-victory/audio.wav` with ffmpeg (path supplied in Q9V). Nothing made from it leaves LHMEZPC01, Google Drive included.
  - `fixtures/LICENSED_RUNS.md` holds the licence record and the numbers only. See a Victory is CCLI Song # 7129060 (read-only lookup in Presenter's local data). The church's CCLI License # is pending (founder).
  - The repo refuses media: root `.gitignore` (R0). In Q9V, a `git ls-files` test and a `ci.yml` step. They allow exactly the 15 public-domain recordings already tracked in `src-tauri/tests/fixtures/voxxad/`, by name; history is not rewritten.
- **#73** closes as "local eval only": a CCLI song in `fixtures/private/` can't run in CI.
- **R0 ponytail audit adopted** (`reviews/2026-11-20-ponytail-r0.md`): the profile must survive a restart; `can_blank` false must not raise a red error at every prayer; the song-change rule lives in dispatch and records the new song's first slide; the amber sentences are shown word for word; Q9V's eval prints numbers only and needs a `truth.jsonl` route; the media guard uses `git ls-files -z`. Discovery takes D5's new row.
- **Pilot facts:** the booth PC is LHMEZPC01 (this machine). Presenter's process is `Presenter` (`C:\Program Files\Presenter\Presenter.exe`). Test card T is in `reviews/2026-11-20-worshiptools-founder-test.md`.
- **Content:** never write, echo, commit or upload the lyrics of any CCLI song, its .mp4, or audio made from either. Agents never press keys in Presenter.

## User decisions for Q9 (2026-11-19, before the unattended run)
- **Privacy first:** fix #102 (Q7-08) before anything else, in its own commit with a test, so that no transcript text reaches the diagnostics buffer or `generate_diagnostic_report`.
  - The help bundle carries the last 2 minutes of recognised words **only** as a separate file in the zip.
  - A review screen is shown before anything is written. It lists every file and shows the words file. A checkbox, **on by default**, "Include the last 2 minutes of words the app heard", controls that file; unchecked means the file is not written at all.
  - Secrets (presenter passwords, the HTTP API token, Planning Center credentials, anything named like token/secret/password/key) are redacted from the operator-state export, with a test.
  - Nothing is written before the user confirms. No network call of any kind.
- **Support e-mail:** the draft has **no recipient yet**; the address is one named constant left empty (a Q9 ticket). A `mailto:` link cannot attach files, so after writing, open the draft (subject plus a body saying which file to attach and where it is) and open the zip's folder.
- **Line budget:** fit the existing rows: backend in "everything else in src-tauri/src", GUI in "Simple mode (src/simple/*)" (or the GUI row, without growing existing panels). No cap is raised and no row is added; if it cannot fit, stop and report the shortfall.
- **GUI testing:** as in Q8 (mocked browser plus a guarded scratch build of the real app with its own identifier). In the real app, a test bundle may be written only to a scratch folder, **never the user's Desktop**, and the e-mail draft and open-folder step are **never** triggered there.
- **Backend:** new commands for the bundle are allowed. Existing command signatures and behaviour stay unchanged, apart from the #102 fix. Reuse `zip` and `dirs`. The ADR is **ADR-018**.
- **Pre-approved:** push `q9/base`, open a draft PR against `main`, fix CI forward; merge once Q9's exit criteria are met and every CI job is green; file Q9 tickets as GitHub issues (label `q9`) and close the earlier issues Q9 fixes (e.g. #102); agents may stop processes they started, by PID.
- **Content:** public-domain text only; agents never write or echo hymn text or real recognised words (fixtures use placeholder words).
- The decisions for Q8 are in git history (a3967e8).

---

## From: Q9V → To: Q10 (Windows signing, winget, updater hardening)
**Date:** 2026-11-21
**Phase:** Q9V (Phase 4 + 5: the private "See a Victory" fixture and the media guard; ADR-019 §§ 7-9, ADR-015) is done on `q9v/base`: head **be4819b**, plus the state commit "state(implementation): Q9V media guard, numbers-only eval, private fixture steps". Both verifiers passed in round 2 at be4819b, and every exit criterion is met. Next: CI, PR and merge by the orchestrator (pre-approved, D6), then Q10. **Start Q10's lanes from `main` after the Q9V merge.** The "From: Q9 → To: Q10" section below still holds for Q10's task and its open questions; this section adds what changed since.

### What Q9V did
- **Process:** two lanes from 84a3966. `q9v/10x` (5 commits, ADR-019 amendment included) was merged as the comparator's base (c847eab). `q9v/ponytail` (3 commits, ce65275) stays local for reference. Then four ponytail grafts, two review fixes, and three test-only fixes from verifier round 1, one commit each. Record: `reviews/2026-11-20-ponytail-q9v.md`.
- **Built:**
  - **The repo refuses media.** `src-tauri/tests/media_guard.rs` and the new ci.yml job `media` ("media guard", `ubuntu-latest`, bash) read `git ls-files -z`, lower-case each path, and refuse any tracked `.mp4 .mov .m4v .mkv .webm .wav .flac .mp3 .m4a` and anything under `fixtures/private/`, naming the path. Exactly the 15 public-domain WAVs in `src-tauri/tests/fixtures/voxxad/` are allowed, by exact path.
  - **`voxxa-eval --full` prints numbers only:** `voxxad`'s stderr to null, failures by exit status, bad lines by number (`tests/eval_numbers_only.rs`). Replay (CI) unchanged.
  - **Founder docs:** `fixtures/README.md` "Making the See a Victory fixture" (9 steps) and `fixtures/tools/tap-truth.ps1`. M17 below.
- **Records:** ADR-019 "Amendment (Q9V)"; `decisions/{sde,qa,security}/church-autoadvance.md` ("Q9V final"); `reviews/2026-11-20-q9v-tickets.md`: 10 open (1 medium, 9 low). Q9V fixes #105 (Q7-11).

### The orchestrator's Q9V close-out (before Q10)
1. Push `q9v/base` with the `gh` credential helper (the branch edits `ci.yml`, so use the `workflow` scope: `git -c credential.helper= -c 'credential.helper=!gh auth git-credential' push`), open a draft PR against `main`, and get every CI job green. Q9V has not run in CI: the "media guard" job runs for the first time, and `eval_numbers_only` and the CI-step test run on Linux and macOS for the first time (on a macOS runner with bash 3.2 the CI-step test prints SKIPPED; the job itself runs on Ubuntu).
2. Merge.
3. Create the label `q9v`, file the 10 tickets, close #105 with 7f71681, and add the comments to #74, #79 and #107 (tickets file, last sections).

### What Q10 must know
- **Ask the user for Q10's decisions before the run** (the list in "From: Q9 → To: Q10": tags and releases, signing secrets, the updater, the budget, pre-approvals in one batch). Add one: **make "media guard" a required status check on `main`** (Q9V-02). Q10 is the CI and release step, so it is the natural moment, and only the founder can change branch protection.
- **Keep the `media` job intact.** `media_guard.rs` reads ci.yml: it finds the job by its `media:` key, runs its `run: |` block with bash, compares its 15 paths and nine extensions with the test's, and fails if the job has `continue-on-error`. If Q10 reorganises `ci.yml`, keep the job's key and its `run: |` block, or change the test in the same commit. Never add `continue-on-error` or an `if:` to it (an `if:` is not caught yet, Q9V-03).
- **Media in build outputs.** The guard reads only tracked paths, so installers and build artifacts are not affected. Do not commit any audio or video file for Q10 (for example a sound in an installer): the guard refuses it, and an allowlist change is an ADR-019 § 8 decision for the founder.
- **Budget** (`python scripts/loc_budget.py` at be4819b): **everything else in `src-tauri/src` 2,315 / 2,317 (2 left)**, not the 2,494 / 2,500 in the Q9 table below; its cap was lowered from 2,500 when discovery got its own row (223 / 250, ADR-019 § 10). voxxa-eval 488 / 600; voxxad 552 / 600; presenters 1,374 / 2,000 (`keystroke.rs` 396 / 400); GUI 2,784 / 3,000; Simple mode 450 / 600. Q10's Rust for the updater check needs lines found or a budget decision first (Q9W-04). Tests in `src-tauri/tests/` are not counted.
- **CSP** (Q9W's note): a strict CSP must keep Simple mode's amber rows and info toasts working (`src/simple/logic.js:11-25`, `src/main.js:560,877`) and Call for help's dialog. Q9V changed no GUI file.
- **Numbers at the finish (Q9V):** `cargo test --features cli` 242 passed, 1 ignored (`media_guard` 5, `eval_numbers_only` 6 checks); `test:gui` 9/9; e2e 188/188. The CI eval step: 21/21 hits, precision and recall 1.00, 0 false and 0 double, median 2,310 ms, p95 4,870 ms (#84), unchanged since Q6. `baseline.json` unchanged.
- **Scratch builds:** Q9V built no GUI app. `C:/dev/q3-10x-target/release/voxxa.exe` and `C:/dev/q3-pony-target` still hold Q9W QA builds with scratch identifiers. Never ship or sign them; build releases in CI or in a clean target folder. Builds rewrite the line endings of `src-tauri/gen/schemas/*.json` (and `src-tauri/Cargo.toml` on `tauri build`); restore them with `git checkout` (#91). Give each lane its own `CARGO_TARGET_DIR`; on this PC a full `cargo test` link once failed with Windows error 1455 (paging file) and passed with `-j 2`.
- **A new public-domain recording** is ignored by `.gitignore` and refused by the guard. Adding one means the same path in three places (`ALLOWED` in `media_guard.rs`, the `media` job's list in `ci.yml`, the `.gitignore` comment) plus `git add -f`; the sync test turns a missed one red.
- **Content and safety (unchanged):** never open, copy, hash or upload the .mp4 or anything under `fixtures/private/`; never read `%APPDATA%\Presenter`; never press keys in Presenter; no microphone; never the real identifier `com.voxxa.app`; stop every process you start, by PID. `test:gui:e2e` can leave headless Edge processes behind (verifier B stopped two by PID); check after each run.
- **Q9V items Q10 may touch:** Q9V-02 (required check) and Q9V-03 (an `if:` on the `media` job). The others can wait.

---

## From: Q9 → To: Q10 (Windows signing, winget, updater hardening)
**Date:** 2026-11-19
**Phase:** Q9 (Phase 4, Call for help, runbooks, quick-start, ADR-018) is done on `q9/base`: head **89f4ccb**, plus the state commit "state(implementation): Q9 Call for help, runbooks, quick-start". Both verifiers passed in round 3 at 89f4ccb. Both exit criteria are met: the bundle is produced, and the links resolve. Next: CI, PR and merge by the orchestrator (pre-approved above), filing the Q9 tickets and closing #102; then Q10.

### What Q9 did
- **Process:** two lanes from 7797556. `q9/10x` (8 commits, ADR first) was merged as the comparator's base (cd28ab5). `q9/ponytail` (6 commits, 8f87206) stays local for reference. Then came four ponytail grafts, the 10x review's fix, and fixes from two verifier rounds that did not pass, one commit each, each with a test that fails without it. Record: `reviews/2026-11-19-ponytail-q9.md`.
- **Built:**
  - **#102 fixed first** (2c28c15, with its test): the listening loop logs `Heard N words`, never the text, so no recognised text reaches the diagnostics buffer or report.
  - **Call for help** (`src-tauri/src/help/mod.rs`, `src/simple/help.js`): a review screen lists every file of the bundle (the diagnostic report, the redacted `voxxa.*` settings, and the last 2 minutes of words from the newest `voxxad` session log, behind a checkbox that is on by default). Save writes one zip to the Desktop; unchecked, the words file is absent. Then the zip's folder opens, and an e-mail draft with no recipient (`SUPPORT_EMAIL = ""`, ticket Q9-01) names the file. Four new commands, no existing command changed, no capability change, no new crate, no network call. `VOXXA_HELP_DIR` sandboxes it for QA and opens nothing.
  - **"What to do" links** on red rows 1, 5 and 6 and on grey row 3 ("model missing" is not red) open the runbook on GitHub through `help::runbook_url`.
  - **Docs:** `docs/quick-start.md` (one printed page, kept so by a test; it says there is no download page yet) and `docs/runbooks/{no-audio-device,presentation-app-not-reachable,model-missing}.md`.
- **Records:** ADR-018 (ADR-017 §§ 1 and 2 amended); `decisions/{sde,qa,security}/church-autoadvance.md` ("Q9 final") and `decisions/architect/_index.md`; `reviews/2026-11-19-q9-tickets.md`: 16 open (1 medium, 15 low). Q9 fixes #102.

### The orchestrator's Q9 close-out (before Q10)
1. Push `q9/base` with the `gh` credential helper, open a draft PR against `main`, and get every CI job green. Q9 has not run in CI yet.
2. Merge. Then check that the three runbook URLs open on GitHub (they return 404 until `main` has the files).
3. Create the label `q9`, file the 16 tickets, close #102 with 2c28c15, and add the comments to #53, #49, #99, #116, #109, #34, #8 and #121 (tickets file, last section).
4. Four headless Edge root processes with `%TEMP%\voxxa-verifya3-*` profiles were still running at the end of Q9 (PIDs 14608, 14632, 14872, 18796; started 1:16-1:18 PM). No Q9 agent started them. Stop them by PID if nobody claims them.

### Q10's task (prompt pack, `specs/2026-11-11-prompts-10x-team.md`, Stage C)
**Q10 — Windows signing, winget, updater hardening (M), Phase 4 + 6, per ADR-008.** In `build.yml`: keep the May branch's Trusted Signing scaffold, add the SignPath Foundation action as the preferred path (it skips when the secrets are absent), NSIS `perUser`, the WebView2 bootstrapper, a `winget` manifest template and a release step. In the updater client, refuse manifests with a lower version than the one installed. Set a strict CSP. Unit test the version check. Exit: a tag build produces signed NSIS and MSI installers when the secrets exist; CSP is no longer null.

### What Q10 must know
**Ask the user for Q10's decisions before the run, as for Q3–Q9.** Q10 collides with earlier decisions and with the budget:
- **Tags and releases.** Q7's user decision was "no tag, no GitHub release". Q10's exit criterion is a tag build. Ask whether Q10 may push a tag (and which, for example a `v0.x.y-rc` pre-release), or whether the proof is a `workflow_dispatch` run that builds without publishing. `build.yml` today publishes a full release (`releaseDraft: false`, `prerelease: false`) on any `v*` tag.
- **Signing secrets.** None exist yet. The founder checklist still says "apply to SignPath" and nothing records Azure Trusted Signing secrets. So Q10 can prove only "skips cleanly without secrets" plus unsigned installers, unless the user provides secrets. Ask which path is real today.
- **The updater.** The plugin is initialised (`src-tauri/src/lib.rs:121`, `tauri-plugin-updater` 2.14.1 in `Cargo.lock`), but nothing calls it, the webview has no updater permission (`src-tauri/capabilities/default.json`: `core:event:default` only; `tests/capabilities.rs`), and there is no `createUpdaterArtifacts` (Q2-28, #10). The cut-list ADR-012 (still a user decision) names "signing scaffolds, unused plugins". Ask whether Q10 wires the updater or only hardens the check.
- **The version check is already the plugin's default.** In `tauri-plugin-updater` 2.14.1, `src/updater.rs:712-714` (registry copy) updates only when `release.version > current_version`, unless a custom comparator is set. So a lower version is refused today. Q10's unit test should pin that (for example a small, tested comparator function that also defines equal versions and pre-release ordering), so a later custom comparator cannot allow a downgrade by accident.
- **Budget: the backend row is nearly full.** "Everything else in src-tauri/src" is **2,494 / 2,500 (6 left)**. Any Rust code for the updater outside an existing row will not fit. Ask before writing code: remove lines elsewhere, or a budget decision (the user prefers a new recorded row over raising "everything else"; Q9 allowed neither). Tests in `src-tauri/tests/` are not counted.
- **Pre-approvals** (push, PR, CI fixes, merge, tickets, stopping own processes): ask for them in one batch, up front.

**`build.yml` today** (`.github/workflows/build.yml`, 230 lines):
- Triggers: `push: tags: v*` and `workflow_dispatch`. Matrix: `macos-latest`, `macos-13` (retired runners; QA record, 2026-10-24, finding 2), `ubuntu-22.04`, `windows-latest`.
- `tauri-apps/tauri-action@v0`, `dtolnay/rust-toolchain@stable` and `swatinem/rust-cache@v2` are pinned by tag in the job that holds the signing secrets (Q2-28, #10). `ci.yml` pins Rust 1.99.1; `build.yml` does not.
- Trusted Signing (the May branch scaffold to keep): `trusted-signing-cli` 0.11.0 is installed only when `AZURE_CLIENT_ID` is set, and a `--config` override injects `signCommand` (`build.yml:109-140`). `tauri.conf.json` has no `signCommand`, `certificateThumbprint: null`, and `timestampUrl` over plain http.
- There is no SignPath step, no NSIS `installMode`, no `webviewInstallMode`, and no `winget` manifest.
- The GPG `.asc` files are made after the release is published and only uploaded as workflow artifacts (Q2-28). The upload glob `bundle/**/*.nsis` matches nothing; the NSIS installer is `bundle/nsis/*-setup.exe`, which the `*.exe` glob catches.
- Editing workflow files needs a push with the `workflow` scope: use `git -c credential.helper= -c 'credential.helper=!gh auth git-credential' push`.

**The CSP** (`src-tauri/tauri.conf.json:49` `"csp": null`; `:14` `"withGlobalTauri": true`; Q2-26, #8):
- No inline scripts: `index.html:375-376` and `stage.html:37` load module scripts. Stylesheets are local. `index.html` has one inline `style=` attribute. `src/main.js`, `src/stage.js` and `src/simple/*.js` use `window.__TAURI__`.
- Q2-26 proposes `default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; connect-src ipc: http://ipc.localhost`. Check the stage window too.
- Call for help renders file content with `textContent`; its one `innerHTML` is a static string (`src/simple/help.js:38`).
- **The mocked browser does not see the CSP.** `tests/gui/e2e.mjs` serves `dist/` itself and injects the mock with a `<script src>` (`:42`). Either send the same CSP header from the test server, so a CSP breakage fails the e2e, or rely on the guarded real app. Re-run the guarded real app with the CSP either way.

**Installers.**
- They must carry the GUI only; `voxxad` and `voxxa-eval` stay out (Q7; `tests/packaging.rs`; Q7-06, #100). Q7 proved it by reading the generated `installer.nsi`.
- Release builds compile whisper.cpp without optimisation on Windows (Q4-01, #59). A signed release would ship that.
- **Do not install or run a produced installer on this PC** without a user decision: it installs and starts the app with the real identifier `com.voxxa.app`, which the safety rules forbid. Inspect the generated `installer.nsi` and the MSI tables instead, or ask.
- The `voxxa.exe` in `C:/dev/q3-10x-target/release/` is verifier A's scratch build (identifier `com.voxxa.qa9va`, CDP port 9354). Never ship or sign it; build releases in CI or in a clean target folder.

**Docs that change when a download exists.**
- `docs/quick-start.md:3-5` says there is no download page yet, and the test `the_quick_start_names_no_download_page_while_there_is_no_release` (`src-tauri/tests/help_bundle.rs:500`) enforces it. Change both together once a pre-release exists.
- `README.md:42` and `docs/index.html:232-266` still point to the releases page (Q9-10). GitHub Pages is off (user decision).
- `docs/signing-policy.md` and `SIGNING.md` still need reconciling (Q1 A15 follow-up). SignPath requires the public signing policy page.

**Q9 items Q10 may touch:** Q9-03 (move from the shell plugin's deprecated `open` to `tauri-plugin-opener`, a new crate, so it needs `cargo-deny` and a user decision); a strict CSP must keep Call for help's dialog working (check it in the guarded real app).

### Numbers at the finish (Q9)
Replay, exit 0: 21 of 21 hits, precision and recall 1.00, 0 false and 0 double advances, median 2,310 ms, p95 4,870 ms (#84), unchanged since Q6. `baseline.json` unchanged. Tests: 221 passed with `--features cli`, 1 ignored. GUI: `test:gui` 7/7, e2e 160/160.

| Row | Used / cap | Left |
|---|---|---|
| everything else in src-tauri/src | 2,494 / 2,500 | **6** |
| conductor (`aligner/`, without `evidence/`) | 1,173 / 1,200 | 27 |
| conductor evidence (`aligner/evidence/`) | 215 / 500 | 285 |
| recognizer | 432 / 700 | 268 |
| voxxad (`src/cli/voxxad/`) | 552 / 600 | 48 |
| voxxa-eval (`src/cli/voxxa-eval/`) | 469 / 600 | 131 |
| presenters | 1,149 / 2,000 | 851 |
| importers + planning_center | 1,386 / 1,600 | 214 |
| GUI (without `src/simple/`) | 2,780 / 3,000 | 220 |
| Simple mode (`src/simple/*`) | 436 / 600 | 164 |

Tests in `src-tauri/tests/`, `tests/gui/` and `scripts/qa/` are not counted.

**Other things to keep.**
- Safety rules for unattended runs: no microphone, no keystroke dispatch, never the real identifier `com.voxxa.app`, `--shadow`/`--replay`/`--presenter none` only for `voxxad`, never the real Desktop, never the mail program or Explorer, and stop every process you start, by PID.
- GUI testing, two ways (Q8's method): (a) the mocked browser, `npm run build && npm run test:gui:e2e -- <screenshot dir>`; (b) the guarded real app, a scratch build with its own identifier and `additionalBrowserArgs` with `--remote-debugging-port`, built with `npm run tauri build -- --no-bundle --config <scratch config outside git>`, then `scripts/qa/simple-mode-real-app.mjs` and, with `VOXXA_HELP_DIR` set to a scratch folder, `scripts/qa/call-for-help-real-app.mjs`. Afterwards delete `%LOCALAPPDATA%\com.voxxa.qa<name>` and restore the LF-only rewrites of `src-tauri/Cargo.toml` and `gen/schemas/*.json` (#91).
- Build and test the tools with `--features cli`. A plain `cargo test` fails 10 tests on purpose. Never print setlist or recognised text.
- The shared `CARGO_TARGET_DIR` (`C:/dev/q3-10x-target`): two `cargo test` runs in it at once can crash rustc. Give each lane its own target.
- Write files with the Edit or Write tools, not Bash heredocs (#56). Check `git grep -lIP '[\x00\x0b]'` before every commit.
- CI: keep Linux on `ubuntu-22.04` (the `unshare -rn` comment in `ci.yml`), and keep the Windows socket test's per-child time window.

### Still open from earlier steps
- **Founder checklist** before any pilot leaves shadow mode: QA report Q2 §10 (M1–M14), the security checklist, Q3-07 (#51), the usability script on the founder's machine (Q8-01, #109), now with Call for help and "What to do" (Q9-07, Q9-12), and **setting the support e-mail address (Q9-01)**. **M15 Keystroke with WorshipTools Presenter** on LHMEZPC01 (R0's M8): with Presenter open, step 2 names it; keys are held while Presenter is not in front; F10 once the unblank is confirmed; Presenter not run as administrator (Windows drops Voxxa's keys silently). **M16 Song change:** Voxxa asks to put the next song live and presses nothing across songs. If the light goes green but Presenter shows the wrong item, Voxxa did not see you put the song live: put it live, then click the slide being sung in Presenter (not the song chip: for the song Voxxa already asked for, the chip presses Left back to slide 1, and Left on Presenter's first slide is untested). Also check what Left on a song's first slide does in Presenter. M17 is the next bullet. Test card T is done (`reviews/2026-11-20-worshiptools-founder-test.md`).
- **M17 New Life, local eval only** (Q9V; ADR-019 §§ 7-9; R0's record, `reviews/2026-11-20-ponytail-r0.md:219`, calls this check "M16": it is M17). On LHMEZPC01, in Windows PowerShell, from the repository root. The full text is `fixtures/README.md` "Making the New Life fixture", steps 1-9. A report, not a merge gate. **Steps 2-4 are done** (2026-11-23, at your request; ADR-019 amendment of that date) in the local worktree `C:\dev\voxxa-private` (branch `local/private-eval`, never pushed): `audio.wav` from your New Life lyric video (2:00), `setlist.json` from the song's 18 slides in Presenter. **Left for you:** step 5 (tap the truth while the 2-minute WAV plays), then steps 6-8, from that folder. New Life is the church's own song (Lighthouse Worship): CCLI Song # "none"; the church's CCLI number 2919521 is already in row 1.
  1. **Once.** `winget install Gyan.FFmpeg`, then open a new PowerShell window. Download Whisper Tiny in Voxxa (Advanced, then Models, then Tiny). Build the tools: `cargo build --manifest-path src-tauri/Cargo.toml --features cli --bin voxxad --bin voxxa-eval`
  2. **The folder:** `New-Item -ItemType Directory -Force fixtures\private\new-life`
  3. **The audio** (replace `<path>` with the full path of your .mp4, keep the quotes): `ffmpeg -i "<path>" -vn -ar 16000 -ac 1 -c:a pcm_s16le fixtures\private\new-life\audio.wav`
  4. **The setlist.** In Notepad, write the slides as Presenter shows them in this recording, in order, one empty line between slides, from the church's own licensed text; save as `fixtures\private\new-life\slides.txt` (UTF-8). Then paste the snippet from README step 4 into PowerShell. It prints only "setlist.json: N slides" and writes `setlist.json` without a byte-order mark (`Set-Content -Encoding utf8` would add one, and the eval then stops with "not a setlist (Syntax error at line 1, column 1)").
  5. **The truth:** `powershell -ExecutionPolicy Bypass -File fixtures\tools\tap-truth.ps1 -Wav fixtures\private\new-life\audio.wav`. Press Enter when the first slide's words begin (slide 0) and at each next slide; a number + Enter marks that slide (also after a blank, to show the same slide again); `b` + Enter a blank; `q` + Enter stops. It refuses to overwrite an existing `truth.jsonl`. Or write it by hand (README step 5).
  6. **Check the setup** on a public fixture: `cargo run --manifest-path src-tauri/Cargo.toml --features cli --bin voxxa-eval -- --full fixtures/recordings/prayer-then-reprise`. A table means the model and the tools work; a `FAIL total` line after it is normal. "voxxad failed with exit code: 1" here means the model is missing or the tools are not built.
  7. **Run the eval:** `cargo run --manifest-path src-tauri/Cargo.toml --features cli --bin voxxa-eval -- --full fixtures/private/new-life`. It prints numbers only. `FAIL` lines and exit code 1 are expected (no baseline row for a private fixture). If step 6 worked but this stops with "voxxad failed with exit code: 1", `audio.wav` is missing or not 16 kHz mono 16-bit: redo step 3. Never add `--record`, and never run `voxxad` on this audio by hand.
  8. **Record it:** `git rev-parse --short HEAD` for the commit. Fill the pending cells of row 1 of `fixtures/LICENSED_RUNS.md`: the date, the commit and the `new-life` row's numbers only (slides, gotos, hits, missed, precision, recall, median, p95, false, double, blank and unblank ms). No words, no paths under AppData, no hashes.
  9. **Before you commit:** `git status --short` must list nothing under `fixtures/private/` and no audio or video file; `cargo test --manifest-path src-tauri/Cargo.toml --test media_guard` must pass. Then `git add fixtures/LICENSED_RUNS.md` and commit only that file, through a PR like any other change. Never `git add -f` anything in `fixtures/private/`. `Remove-Item -Recurse fixtures\private\new-life` deletes the fixture when you no longer need it (the .mp4 is not in it).
- **Founder:** apply to SignPath; open Partner Center. #73 is closed as "local eval only" (ADR-019 § 9). Tap the New Life truth and run its eval (M17 above; the audio and setlist are made). Confirm the F10 unblank key (T3; ticket Q9W-02). Make "media guard" a required status check on `main` (Q9V-02).
- **User decisions:** the cut-list ADR-012; Q6-01 (#83) and Q6-10 (#89).
- **Q1 A15 follow-ups:** Q7-12 (#106); re-run the Windows verification on the merged head (`handoff-local-windows.md`); reconcile `docs/signing-policy.md` with `SIGNING.md`.

**Process rule (user, 2026-11-11):** every prompt (Q2…Q27) also runs independently through ponytail. Run code steps (Q10 is one) as two lanes plus cross-review, and verification and close steps as `ponytail` + `ponytail-audit`. Record the result in `reviews/YYYY-MM-DD-ponytail-<Q>.md`.

---

## Handoff History
- 2026-11-20 Q9W → Q9V: Q9W found WorshipTools Presenter by its process, guarded its keys and stopped stepping across songs (ADR-019 §§ 1-4 and 10); merged into `main` as 84a3966 (PR #144); 12 tickets filed as #145-#156; #117 closed; the full section, with Q9V's task and what Q9V had to know, is in this file's git history (be4819b).
- 2026-11-19 Q8 → Q9: Q8 built Simple mode in front of the existing panels (ADR-017: the mode switch, the status light with one sentence, the first-run flow, and a Call for help placeholder); the merge blocker Q8-10 was fixed in 88aa63b and fce3e58 and verified; merged into `main` as a3967e8 (PR #108); 15 tickets filed as #109–#123; the full section, with the GUI testing method, the Call for help notes for Q9 and the budget table, is in this file's git history (89f4ccb).
- 2026-11-19 Q7 → Q8: Q7 closed Increment 1′ (a determinism test on 3 OSes, a zero-sockets test for `--replay`, the report-only full eval, the tools out of the installers, the CLI release notes, the Companion guide and the runbook); merged into `main` as c448139 (PR #95); 12 tickets filed as #96–#107; #47 and #75 closed; the full section, with the eval commands, the budget table and the CI notes, is in this file's git history (ee431ce).
- 2026-11-19 Q6 → Q7: Q6 built ADR-009's minimum (the evidence tracker, ADR-016) on six synthetic fixtures (false advances 1 → 0, p95 11.23 → 4.87 s), with the user's one-time baseline exception for the jump row; merged into `main` as a60d1ce (PR #82); 12 tickets filed as #83–#94; #76 and #62 closed; the full section, with the eval commands, the determinism trap and the re-recording rules, is in this file's git history (6386f6b).
- 2026-11-13 Q5 → Q6: Q5 added `voxxa-eval` (replay in CI, `--full` with Whisper Tiny locally), two stand-in fixtures, `fixtures/baseline.json` and a CI eval step (ADR-015); merged into `main` as 69f7d27 (PR #72) after the orchestrator closed Q5-08, Q5-09 and Q5-10's M52/M55 with tests (5858872) and proved the gate in real CI (run 43469439549); 9 tickets filed as issues #73–#81; the full section, with the eval commands, the first numbers and the Q5-04 reproduction, is in this file's git history (4edd7f6).
- 2026-11-13 Q4 → Q5: Q4 put Whisper behind ADR-004's `Recognizer` trait as a 5 s window decoded every 1 s, with a stitcher and a session-log `window` line (ADR-014); code head a4bae0e, merged into `main` as 571cf0d (PR #58); 13 tickets filed as issues #59–#71; #46 and #52 closed; the full section, with the schema, cadence, replay and alignment notes for the eval, is in this file's git history (c10e963).
- 2026-11-12 Q3 → Q4: Q3 added the headless `voxxad` CLI and moved the listening loop into `voxxa_lib::listen::run` (ADR-013; schema v1, byte-identical replay); code head 912b978, merged into `main` as ec6d85e (PR #45); 12 tickets filed as issues #46–#57; Q3-02 (#46) handed to Q4; the full section is in this file's git history (a4bae0e).
- 2026-11-12 Q2 → Q3 (handed to the user for ADR-012, then Q3): Q2 verified the merged base in simulate mode and fixed every crash, data-loss, freeze and wrong-slide defect found, with tests (code head 713f40f after the ponytail-audit re-run, merged into `main` as 7bd84a1); 43 tickets filed as issues #2–#44; the full section, with the unattended-run needs and the cut-list evidence, is in this file's git history (f2bcea1).
- 2026-11-11 Q2: QA + security verification in simulate mode; 18 fix-now items + verifier finding Q2-44 fixed with tests; two fix lanes (10x, ponytail) → 10x base with ponytail grafts; 34 tickets; ADR-011 stage window compliant by default; cut-list evidence for ADR-012.
- 2026-11-11 Q1: May branch merged per ADR-010 + A1–A15; two lanes (10x, ponytail) → 10x base with ponytail grafts; all local gates green; build.yml YAML fixed.
- 2026-11-11 Q0 gate: the user accepted ADR-010 as amended; ADR-011; cuts deferred until after Q2; Q2 widened to cover wrong-slide defects.
- 2026-11-11 Q0: claims verified, ADR-010 path table + amendments A1–A15, ponytail lane; stopped at the user gate.
- 2026-11-11 re-planning: May branch discovered; ADR-010 Proposed; prompt pack regenerated for /10x-Team:10x-team.
- 2026-10-24 init → Discovery: `.10x/` created.
- 2026-10-24 Discovery → Strategy: codebase scanned; critical finding: model never loaded, VAD never initialized.
- 2026-10-24 Strategy → Design → Planning: specs + ADR-001…009 + prompt plan written in one pass after five-angle research.
