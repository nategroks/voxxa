# Current Handoff

## Round 3 strategy adopted (2026-11-27): read this before any V-step

- **Read first:** root `CLAUDE.md` (safety lines, budget rows, plugins, records), then `specs/2026-11-26-skills-and-context-layout.md`
  (where files go, the lane scan § 4.4, what each step reads § 5), then the step's block in `specs/2026-11-26-prompts-round-3.md`
  over `specs/2026-11-24-church-autoadvance-plan.md`. Founder decisions FA-01 … FA-77 and their effects: `specs/2026-11-24-founder-action-items.md`
  (batches of 2026-11-26/27 at the top). Every FA is answered; FA-76 is parked.
- **Order:** V7 (full scope) → V8c (ADR-036 position sources; task 0 = booth capture probe before any build) → F5 (founder) → V3e → V8b → … as the
  prompt pack's sequencing table. V7 adds the `position` row (estimate 300, cap 900) with the FA-54 rows.
  **V7 merged** (2026-11-27): `scripts/loc_budget.py` has the wiring row (1,288 / 1,438), `vad/` in the recognizer row (557 / 825), "everything else" 900 / 904, and `onset` 750, `input chain` 600, `position` 900, `health check` 450 at 0; ADR-028 is Accepted with amendment 1 (every row, estimate, cap, owner, step and the shares); ADR-001/003 amendment 1 records the `trace` line (FA-65); ADR-005 amendment 1 the licence inventory (FA-50). The row table below is current. Next: V8c (its Phase 2 check reads the wiring row's "left" and its ≤ 20 ceiling from ADR-028 amendment 1 § 5).
- **Staging → repo file map** (the records cite their staging names): `out/spec.md` → `specs/2026-11-24-church-autoadvance-spec.md`;
  `out/plan.md` → `specs/2026-11-24-church-autoadvance-plan.md`; `out/process.md` → `specs/2026-11-24-process.md`;
  `out/founder-action-items.md` → `specs/2026-11-24-founder-action-items.md`; `plan-round3.md` = `round2/52` → `specs/2026-11-26-prompts-round-3.md`;
  `round2/51` → `specs/2026-11-26-strategy-and-build-plan.md`; `round2/53` → `specs/2026-11-26-skills-and-context-layout.md`;
  `round2/40 … 47, 50` → `reviews/2026-11-26-r2-<nn>-<role>.md`; `out/adjudication.md` (round 1) → `reviews/2026-11-24-r1-adjudication.md`;
  the research report → `specs/2026-11-26-research-autonomous-following.md`, its notes → `specs/research-notes-2026-11-26/`;
  `out/adrs/022 … 036` → `adrs/` (033 is reserved for an aligner row, written only on an R5 pass); `out/kit/` stays in staging until V5 (kit-v0.1.0).
- **ADR status:** 022–036 are Proposed; their dated amendment sections are drafts that the named step accepts with its numbers.
  ADR-036 is at revision 1 (the lab's hash probe: 64-bit whole-frame hashes collide on lyric slides; 256-bit text-region hashes separate).
- **Lab (private, never committed), done 2026-11-27:** the R14 must-fixes, the `reanchor(slide, t)` hook with 18 pytests (ADR-031 amendment 3)
  and the capture probe for V8c task 0 (`reviews/2026-11-27-lab-r14-reanchor-lag.md`); lanes receive edit lists and numbers only. **Measured:** the
  live read lag is 2,148 ms median against the 1,000 ms replays assume; V3b task 8 breaks it down; offline follower numbers are optimistic by ≈ 1.15 s.
- **Pre-existing names in this file and `status.md`** (the pilot church and the test song in older sections) stay per FA-12 (c); new text uses
  "the pilot church" and "P1". The lane scan runs on staged files only.


## User decisions for Q10 (2026-11-23)
- **Stop at a DRAFT release.** Tag builds create drafts only; the founder publishes the real release by hand, soon after. Agents never publish a draft.
- **Signing secrets:** the founder is still working on them. Build so that everything works without them and lights up when they are added; list exactly which secrets and variables to add (`docs/RELEASING.md` § 1; `status.md`, founder items).
- **Windows signing:** keep the May branch's Azure Trusted Signing scaffold (runs only when the `AZURE_*` secrets exist); add SignPath Foundation as the **preferred** path, running only when its secrets exist and skipping cleanly otherwise. NSIS per-user, the WebView2 bootstrapper (pick and justify), `timestampUrl` over https if supported. A winget manifest template filled by a release step as an artifact, submitted to winget-pkgs only when a token exists. Pre-release for tags with a `-`. Fix the retired `macos-13` runner and the dead `*.nsis` glob. `createUpdaterArtifacts` only when `TAURI_SIGNING_PRIVATE_KEY` exists; the build succeeds without it.
- **The updater: "act in the best interest" → option B, operator-initiated only.** A "Check for updates" control in Advanced → Settings (not Simple mode's main screen). Never automatic, never at startup; refused while listening with a clear sentence and no network call. The GUI calls Voxxa's own Rust commands; the webview gets **no** updater plugin permission (`capabilities/default.json` stays least-privilege). A small pure, unit-tested comparator: update only when the remote version is strictly greater under semver (equal → no update; lower → refused; pre-release ordering per semver; malformed → refused). Installing asks first and says Voxxa will restart. Signatures are verified with the existing pubkey; **the founder confirms he holds the matching private key or generates a new pair himself** (agents never generate or handle keys). The privacy promise names the update check as a network use.
- **Strict CSP** in `tauri.conf.json` (no longer null), zero violations shown in the guarded real app, e2e green; prefer removing inline styles to allowing `'unsafe-inline'`.
- **Budget:** no cost cap. Never raise a line cap. The updater's Rust goes in **one** new row "updater (src/updater/)" in `scripts/loc_budget.py`, recorded in a new ADR-020 (the D5 precedent, ADR-019 § 10). Wire-up lines elsewhere fit the 2 spare lines of "everything else" or are offset.
- **Keep the ci.yml `media` job intact** (key, `run: |` block, no `continue-on-error`, no `if:`); Q9V-03 may be fixed if cheap.
- **Pre-approved, for the orchestrator only:** push, PR, CI fixes, merge, **one rc tag that creates a draft release**, tickets.
- **Safety (unchanged and added):** no microphone; never press keys in or touch WorshipTools Presenter; never start the GUI with `com.voxxa.app`; never touch the installed Voxxa (`%LOCALAPPDATA%\Voxxa\voxxa.exe`) or its data; never install a built installer; never generate, read or print signing keys or secrets; no network calls from tests; **never click "Check for updates" in a real app**; stop every process you start, by PID.

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

## V0 (#59), 2026-11-25: Windows release builds compile whisper.cpp with -O2
Plan step V0 of the 2026-11-24 strategy draft (not yet committed), with the orchestrator's rulings: V0 never runs `build.yml` (tag only, always a draft, a new rc needs the founder), so the check is a script with a test on every PR; the env goes only where whisper-rs-sys is built with `--release` on Windows. ADR-021 § 1 is Accepted (founder, FA-01, 2026-11-25); §§ 2-3 are Proposed. Two lanes ran off 866b51e (`v0/10x`, `v0/ponytail`). The final branch `fix/59-msvc-release-flags` takes the ponytail lane as its base and grafts parts of the 10x lane; `reviews/2026-11-25-ponytail-v0.md` has the ponytail lane's record and the adjudication.
- **Built:** `build.yml`'s Windows job sets `CMAKE_C_FLAGS_RELEASE` and `CMAKE_CXX_FLAGS_RELEASE` to `-MD -O2 -Ob2 -DNDEBUG` (that job only: `-Ob2` breaks gcc and clang); its rust-cache key is `release-portable-ggml-o2`; "Check that whisper.cpp was built with -O2" runs `scripts/ci/check-msvc-release-flags.sh` right after "Build the installers" and fails with `::error::` when no whisper-rs-sys `CMakeCache.txt` exists or a flag line lacks `-O2`. `ci.yml` tests the script on every PR, in the `budgets` job and in the `check` job's Windows leg (the same CRLF checkout and Git Bash as the release job): 12 cases, from fixed, broken and half-broken caches to stale, foreign and missing ones. `release_config.rs` pins the lines, the key and the step, and refuses the two variables in every other workflow and in a `.cargo/config(.toml)`.
- **Not changed, on purpose:** `ci.yml` and `full-eval.yml` build debug only, and a debug build already compiles whisper.cpp with `/O2` (the crate's override lands on the RelWithDebInfo line, which the Visual Studio generator does not use). The macOS/Linux cache key `build.yml:165` stays: rust-cache keys by job id and OS, so it never holds an MSVC build. Reasons in `decisions/devops/church-autoadvance.md` ("V0").
- **Open:** (1) `build.yml` runs only for a tag, so the check has not run on GitHub. The next founder-approved rc draft's Windows job must pass it and print two flag lines with `-O2`; record the run ID (`status.md`, Pending). (2) The founder's booth timing, ≤ 150 s (`status.md`, Pending).
- **Local release builds** (founder): in Windows PowerShell, `Set-ExecutionPolicy -Scope Process Bypass`, then dot-source `scripts/dev/msvc-release-flags.ps1` in the window you build from, and run `cargo clean --manifest-path src-tauri/Cargo.toml -p whisper-rs-sys --release` once (`docs/RELEASING.md` § 7). `powershell -File` would lose the variables.
- **Later steps must know:** the `v0.1.0-rc.1` draft predates V0, so its Windows installers carry the unoptimised whisper.cpp; a volunteer test with an installed build (Q11) needs a new rc built after the V0 merge, and a new rc needs the founder. A workflow that adds a Windows `--release` build of whisper-rs-sys needs the two variables in a Windows-only step and a new cache key, with `release_config.rs` changed in the same commit (`the_msvc_release_flags_are_set_nowhere_else` fails until then).
- **PR body:** "Fixes #59", with the cause section from `reviews/2026-11-25-v0-tickets.md`. Ticket candidates V0-01 to V0-03 are in the same file.

## From: Q10 → To: Q11 (close Increment 2′)
**Date:** 2026-11-23
**Phase:** Q10 (Phase 4 + 6: Windows signing, winget, updater hardening, per ADR-008; new ADR-020) is done on `q10/base`: head **f424c74**, plus the state commit "state(implementation): Q10 signing paths, draft releases, operator-initiated updater, strict CSP". Both verifiers passed in round 2 at f424c74. Every exit criterion that can be proved on this PC is met; the tag build itself is proved by the orchestrator's rc tag run below. Next: the orchestrator's close-out (CI, PR, merge, tickets, then **one** rc tag, `v0.1.0-rc.1`, which must produce a **draft** release), then Q11. **Start Q11 from `main` after the Q10 merge and the tag run.**

### What Q10 did
- **Process:** two lanes from 68adf2d. `q10/10x` (9 commits, ADR-020 before any code) was merged as the comparator's base (b5d8df0). `q10/ponytail` (3 commits, 810c203) stays local for reference. Then three grafts and a tickets graft from the ponytail lane and its review, the 10x review's three should-fixes (one corrected) and one nit, and four fixes from verifier round 1, one commit each. Record: `reviews/2026-11-23-ponytail-q10.md`.
- **Built:**
  - **Updater, option B** (`src-tauri/src/updater/mod.rs`, its own budget row): **Check for updates** in Advanced → Settings only; refused while listening before any request and again after the download; a pure semver comparator (strictly greater) set as the plugin's default and on the instance; install only the version the operator confirmed, with "Voxxa will close, install the update and start again."; no webview updater permission; endpoint `releases/latest`, so drafts and pre-releases are never offered. Tests check the rules where they are enforced (`tests/updater.rs`, `tests/capabilities.rs`).
  - **Strict CSP** (`tauri.conf.json:49`), no `unsafe-*`; the three inline styles removed; a Vite-only `devCsp`; the e2e serves the same header.
  - **Release pipeline:** `build.yml` makes a **draft** for every tag (pre-release for a `-` tag) after checking the tag, its version, its commit and `main`; Windows NSIS per-user (`currentUser`) and MSI, WebView2 `embedBootstrapper`; signing SignPath (all four values) → else Azure (all six) → else unsigned with a warning; Authenticode checked; updater signatures over the final bytes; `latest.json` written once, only with the key; macOS on `macos-latest` and `macos-15-intel`, each `APPLE_*` exported only when set; the portable ggml build (#142); every action pinned by SHA. `winget.yml` submits a **published** full release only when `WINGET_TOKEN` exists. Runbook: `docs/RELEASING.md`.
- **Records:** ADR-020 and an ADR-008 amendment; `decisions/{sde,qa,security,devops}/church-autoadvance.md` ("Q10 final") and `decisions/architect/_index.md`; `reviews/2026-11-23-q10-tickets.md`: 10 open (2 medium, 8 low). Q10 fixes #8, #10, #142, #148 and #160.

### The orchestrator's Q10 close-out (before Q11)
1. Push `q10/base` with the `gh` credential helper and the `workflow` scope (`git -c credential.helper= -c 'credential.helper=!gh auth git-credential' push`; it edits `build.yml` and `ci.yml` and adds `winget.yml`). Open a PR against `main` and get every CI job green. Q10 has not run in CI: the `budgets` job's new "Release script tests" step, `release_config.rs`, `updater.rs` and the new e2e checks run on GitHub for the first time.
2. Merge.
3. Create the label `q10`, file the 10 tickets, close #8, #10, #142, #148 and #160 with the commits named in the tickets file, and add the comments to #159, #59, #100 and #134 (tickets file, last sections).
4. **The tag run (pre-approved: one rc tag that creates a draft).** Tag **`v0.1.0-rc.1`** on `main`'s merge commit and push the tag. `tauri.conf.json` is 0.1.0; the release job refuses any other version, a tag on another commit, or a commit not on `main`. Check, and record the run ID in the DevOps record and `status.md`:
   - the release job prints "Draft release v0.1.0-rc.1 of version 0.1.0 (pre-release: true) from <sha>", and the release is a **draft pre-release**;
   - Windows: "::warning::No Windows signing path is fully configured: the installers are unsigned and SmartScreen will warn." and "TAURI_SIGNING_PRIVATE_KEY not set: no updater signatures."; Authenticode `NotSigned` for both installers; exactly one `*-setup.exe` and one `.msi` collected; `gh release upload` works;
   - macOS, both runners: four "… not set: left out of the build." lines in "Apple signing values" and "APPLE_CERTIFICATE not set: the macOS build is not signed."; both build ad-hoc signed and not notarized and upload a `.dmg`; `macos-15-intel` is accepted and the portable ggml build works on both;
   - Linux: "GPG_PRIVATE_KEY not set — Linux artifacts will not be signed."; `.AppImage` and `.deb` uploaded;
   - assets: `Voxxa_0.1.0_x64-setup.exe`, `Voxxa_0.1.0_x64_en-US.msi`, two `.dmg`, an `.AppImage`, a `.deb`; **no** `.sig`, `.asc` or `latest.json` (the names say 0.1.0, not rc.1: the MSI needs a numeric version);
   - the updater-manifest job runs only its "No updater key" step;
   - the `voxxa-winget-manifest` artifact holds three filled files with no `{{` placeholder left;
   - the pinned action SHAs resolve, and tauri-action uploads into our draft by `releaseId`;
   - per-user NSIS: read the generated `installer.nsi` (a clean local `npm run tauri build -- --bundles nsis` in its own target folder, nothing installed), or try the draft's installer on a test PC, **never LHMEZPC01**; and only `voxxa.exe` is inside (#100).
   **Do not publish the draft.** If the run fails, fix forward on a branch (CI fixes are pre-approved) and re-run the workflow for the same tag; if the tag itself must move, delete the draft and the tag (`RELEASING.md` § 5). A second rc tag needs the founder's approval.
5. Still unproven after the tag run, until the founder's secrets exist: SignPath (the `@v3` inputs, the artifact configuration's wildcards, the approval within 3600 s), Azure, notarization, the updater signatures and `latest.json` (including tauri-action's macOS updater asset names, which `updater_manifest.py` requires). `winget.yml` waits for a published full release, and the in-app update needs two published releases.
6. Housekeeping: after the merge, the worktrees `C:/dev/q10-10x`, `C:/dev/q10-pony` and `C:/dev/q10-base` can go (keep the branches local for reference). `C:/dev/q3-10x-target` and `C:/dev/q3-pony-target` hold Q10 QA builds with scratch identifiers (`qa10x`, `qa10f`, `qa10a`, `qa10p`): never ship or sign them.

### Q11's task (prompt pack, `specs/2026-11-11-prompts-10x-team.md:101-103`, with R0's M2)
**Q11 — Close Increment 2′ (S), Phase 5 + 6.** QA: run the five-volunteer first-run test from `specs/2026-10-24-user-research-plan.md` §2 on LHMEZPC01 with WorshipTools Presenter, including recovery from "Bring WorshipTools Presenter to the front" and "Put "<song>" live in Presenter" (ADR-019; R0's M2 removed "one video song"). Security: review the support bundle for over-collection. DevOps: first signed pre-release. SRE: on-call runbook for the Sunday window. Reviews and decision files as in Q7. Q11 is a verification and close step, so the process rule asks for `ponytail` + `ponytail-audit` beside it, not two build lanes.

### What Q11 must know
**Ask the founder for Q11's decisions before the run, in one batch:**
- **Which build the volunteers use, and where.** `docs/RELEASING.md` § 4 says an rc never goes on a booth PC, and LHMEZPC01 is the booth PC. Installing any build there replaces the installed Voxxa (`%LOCALAPPDATA%\Voxxa\voxxa.exe`, real identifier `com.voxxa.app`), which agents may not touch. Options: the founder publishes v0.1.0 first and installs it himself; or he accepts the rc on LHMEZPC01 for the test and installs the final by hand afterwards (the updater will not move an rc to the same X.Y.Z); or the test runs on another PC. Agents never install.
- **"First signed pre-release" needs secrets.** If no SignPath or Azure secrets exist by Q11, a new tag can only give another unsigned draft. Ask whether the founder has the secrets, and whether Q11 may make one more rc tag (only one was pre-approved in Q10).
- **The updater key:** whether the founder holds the private key for pubkey 84F2CD98C608E9C9 or will replace it, before anything is published.
- **The volunteers are people.** The founder runs the sessions; agents prepare the script, the observation sheet and the record, and never press keys in Presenter or start the app with `com.voxxa.app`.
- Pre-approvals (push, PR, CI fixes, merge, tags, tickets), in one batch, up front.

**Before the volunteer test:**
- **Q10-02 (medium):** in the ponytail lane's real-app run, the first-run flow advanced to step 2 by itself, chose an audio device and ran discovery. A first run that advances on its own would spoil a first-run test. Reproduce it (clean scratch identifier, CDP attached before the window shows, every click and key logged) or rule it out before volunteers sit down.
- **Q10-01 (medium):** Settings logs `ReferenceError: settings is not defined`, and the threshold and margin labels never follow their sliders. Fix it if volunteers will open Advanced.
- **#59 (Q4-01, medium):** the Windows installers built by `build.yml` compile whisper.cpp unoptimised (about 6.5 s per 1 s hop on the reference PC). A volunteer test that listens with an installed release build will feel slow. Fix it first, or test with a build that has the fix. **V0 fixes it** (section above); an installer has the fix only once built from a commit with V0 merged, and the next rc proves it.

**Security (support bundle):** the bundle is Q9's Call for help (ADR-018: the diagnostic report, the redacted `voxxa.*` settings, the last 2 minutes of words behind a checkbox that is on by default). Q10 added update checks and downloads to the outbound counter (`net_stats`); check whether the report names them and whether anything there is more than support needs. The redaction rules are in ADR-018.

**SRE (Sunday window):** the runbook should say that Voxxa never checks for updates by itself and refuses while listening, so a service cannot be interrupted by an update; that updates are installed between services by pressing **Check for updates**; and how to roll back a bad release (`RELEASING.md` §§ 5-6: unpublish, fix forward with a higher version; there are no downgrades). Link the existing runbooks in `docs/runbooks/`.

**Things to keep from Q10:**
- `tests/release_config.rs` pins `build.yml` and `winget.yml`: the draft rule, the signing order, the skip conditions, the matrix, the SHA pins and the macOS env rule (since V0 also the #59 flags, the Windows cache key and the flag check, and no copy of the flags in another workflow or a cargo config). A workflow change updates the test in the same commit. A new action needs a full SHA and a `# version` comment.
- The updater's source-text tests (`tests/updater.rs`) read the guard as rustfmt lays it out; a deliberate reformat must update the constant there.
- `capabilities/` must hold only `default.json` (`core:event:default`); `tests/capabilities.rs` refuses anything else.
- Budget (as of V7, 2026-11-27; the row table below): "everything else in `src-tauri/src`" is 900 / 904, **4 left**, and those 4 are the `lib.rs` `mod` lines for `input`, `position`, `readiness` and `practice`; any other change to `lib.rs`, `audio/`, `diagnostics/`, `help/`, `hymnary/`, `net_stats/` or `tray/` needs lines removed first. The wiring row (`dispatch.rs`, `listen.rs`, `http_api/`, `commands/`) has **150 left**, spent only through the ceilings in ADR-028 amendment 1 § 5 (they sum to 170, so a step's Phase 2 check reads what is left first). The updater row has 52 left.
- Never click **Check for updates** in a real app; the guarded real app (scratch identifier, CDP port, `--no-bundle`, `VOXXA_HELP_DIR` a scratch folder) is the only way agents run the GUI. `scripts/qa/csp-real-app.mjs` re-checks the CSP.
- Keep the ci.yml `media` job intact; `media_guard.rs` now also refuses an `if:` on it. "Media guard" is a required check since 2026-11-24 (ruleset `main`, id 27988429; #159 closed).

### Numbers at the finish (Q10)
Replay, exit 0: 21 of 21 hits, precision and recall 1.00, 0 false and 0 double advances, median 2,310 ms, p95 4,870 ms (#84), unchanged since Q6. `baseline.json` unchanged. Tests: `cargo test --features cli` 266 passed, 1 ignored; `scripts/release` 11/11; GUI `test:gui` 18/18, e2e 209/209 with zero CSP violations.

**Budget rows after V7** (measured 2026-11-27, tokei 15.0.0, `python scripts/loc_budget.py` at 0190515 + the V7 change; ADR-028 Accepted with amendment 1, which has every row's estimate, owner, step and the reason for its headroom). Caps only go down; a new module gets a new row through an ADR; stop and report at 95 %, and a tripled row reports when it passes its estimate.

| Row | Used / cap | Left |
|---|---|---|
| everything else in src-tauri/src (`lib.rs`, `main.rs`, `audio/`, `diagnostics/`, `help/`, `hymnary/`, `net_stats/`, `tray/`) | 900 / 904 | **4** (the `lib.rs` `mod` lines for `input`, `position`, `readiness`, `practice`) |
| wiring (`dispatch.rs`, `listen.rs`, `http_api/`, `commands/`; new in V7, FA-10: measured 1,288 + 150) | 1,288 / 1,438 | **150** (ceilings below) |
| updater (`src/updater/`) | 98 / 150 | 52 |
| discovery (`src/discovery/`) | 223 / 250 | 27 |
| conductor (`aligner/`, without `evidence/` and `onset/`) | 1,173 / 1,200 | 27 (V5a 3 · V11 20 + `pub mod onset;` 1) |
| conductor evidence (`aligner/evidence/`) | 215 / 500 | 285 |
| onset (`aligner/onset/`; new in V7, estimate 250 × 3, FA-54; V11) | 0 / 750 | 750 |
| recognizer (`transcription/` + `vad/`; `vad/` 125 moved in V7, headroom unchanged) | 557 / 825 | 268 (V3 60 · V4 40 · V3b gate 40) |
| input chain (`src-tauri/src/input/`; new in V7, estimate 200 × 3, FA-54; V3b task 6) | 0 / 600 | 600 |
| position (`src-tauri/src/position/`; new in V7, estimate 300 × 3, ADR-036 Decision 10, FA-77; V8c) | 0 / 900 | 900 |
| voxxad (`src/cli/voxxad/`) | 552 / 600 | 48 (V12 20 · V8b 10 · V13s 8 · V3b `Kind::Trace` 4) |
| voxxa-eval (`src/cli/voxxa-eval/`) | 488 / 600 | 112 (V2 105) |
| presenters (`keystroke.rs` 396 / 400) | 1,396 / 2,000 | 604 (V8b 150 · V8c 30) |
| importers + planning_center | 1,386 / 1,600 | 214 (V5a 40) |
| GUI (without `src/simple/` and `src/health/`) | 2,860 / 3,000 | 140 (V14 net +15) |
| Simple mode (`src/simple/*`) | 450 / 600 | 150 (V9 5 · V14 60 · V13s 10) |
| health check (`src/health/`; new in V7, estimate 150 × 3, FA-54; V13s) | 0 / 450 | 450 |

Wiring-row ceilings (the prompt pack's "How to use" table plus the two steps it predates; ADR-028 amendment 1 § 5). They sum to 170 against 150 left, so they are ceilings, not allocations: the row is spent in merge order, and a step starts only if its ceiling fits what `loc_budget.py` shows left, or records a smaller share with the logic moved into its own row behind one call line.

| Step | Wiring lines | Ceiling |
|---|---|---|
| V8-lite | `position_uncertain` flag, setter, per-key clear | ≤ 20 |
| V8c | `PositionSource` polled by dispatch, `reanchor` call, config field | ≤ 20 |
| V8b | `select_presentation` first; consume the verify result | ≤ 30 |
| V3b task 6 | input config field, the chain call, the `level` emission, `Event::Trace` + `Trace` enum | ≤ 15 |
| V9 | session-folder helper and the one-time move | ≤ 10 |
| V13a | `holding` / `following_stopped` flags, four routes, hotkey call sites, `operator` trace | ≤ 40 |
| V13s | heartbeat publish calls | ≤ 15 |
| V11 | Tier A/B dispatch of `Action::Goto`, follower hand-off | ≤ 20 |

Named, not added: `align` 1,200 (400 × 3) only on an R5 pass (ADR-033 then); `readiness` 900 (300 × 3) only on the founder's yes at V13s (ADR-034 § 11); no row for `scripts/lab/` (EM, V7: lab tooling, uncounted like `scripts/qa/`). Tests in `src-tauri/tests/`, `tests/gui/`, `scripts/qa/`, `scripts/release/` and `scripts/lab/` are not counted.

**Other things to keep.**
- Safety rules for unattended runs: no microphone, no keystroke dispatch, never the real identifier `com.voxxa.app`, `--shadow`/`--replay`/`--presenter none` only for `voxxad`, never the real Desktop, never the mail program or Explorer, never install a built installer, never handle a key or secret, and stop every process you start, by PID. `test:gui:e2e` can leave headless Edge processes behind; check after each run.
- Build and test the tools with `--features cli`. Give each lane its own `CARGO_TARGET_DIR`; on this PC a full `cargo test` link once failed with Windows error 1455 (paging file) and passed with `-j 2`. Builds rewrite the line endings of `src-tauri/gen/schemas/*.json` and `src-tauri/Cargo.toml`; restore them with `git checkout` (#91).
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
- **Founder:** apply to SignPath; open Partner Center. #73 is closed as "local eval only" (ADR-019 § 9). Tap the New Life truth and run its eval (M17 above; the audio and setlist are made). Confirm the F10 unblank key (T3; ticket Q9W-02).
- **Done since (2026-11-24):** the founder made "media guard" a required check on `main` through the ruleset `main` (id 27988429); #159 is closed.
- **Founder (Q10):** add the release secrets and variables when ready (SignPath: secrets `SIGNPATH_API_TOKEN`, `SIGNPATH_ORGANIZATION_ID`, variables `SIGNPATH_PROJECT_SLUG`, `SIGNPATH_SIGNING_POLICY_SLUG`, and the artifact configuration and a manual-approval policy in SignPath; Azure fallback: the six `AZURE_*` secrets; updater: `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`; winget: `WINGET_TOKEN`); confirm you hold the private key for pubkey 84F2CD98C608E9C9 or generate a new pair yourself and replace the pubkey; confirm the winget identity `nategroks.Voxxa`; publish the draft when ready (`docs/RELEASING.md` §§ 3-4; #59 first). The full list is in `status.md` and `RELEASING.md` § 1.
- **User decisions:** the cut-list ADR-012; Q6-01 (#83) and Q6-10 (#89); Q10-06 (keep or drop ADR-008's "stable" channel).
- **Q1 A15 follow-ups:** Q7-12 (#106); re-run the Windows verification on the merged head (`handoff-local-windows.md`); `docs/signing-policy.md` and `SIGNING.md` now describe the same pipeline (Q10); read them through once more when SignPath answers.

**Process rule (user, 2026-11-11):** every prompt (Q2…Q27) also runs independently through ponytail. Run code steps as two lanes plus cross-review, and verification and close steps (Q11 is one) as `ponytail` + `ponytail-audit`. Record the result in `reviews/YYYY-MM-DD-ponytail-<Q>.md`.

---

## Handoff History
- 2026-11-21 Q9V → Q10 and 2026-11-19 Q9 → Q10: Q9V added the media guard and a numbers-only `voxxa-eval --full` (ADR-019 §§ 7-9); merged into `main` as 5bbac72 (PR #157); 10 tickets filed as #158-#167; #105 closed. The two sections that briefed Q10 (Q10's task; tags and releases, signing secrets, the updater, the budget, the CSP, the installers, the docs that change with a download) are in this file's git history (f424c74).
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
