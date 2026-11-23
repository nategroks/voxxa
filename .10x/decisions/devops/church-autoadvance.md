# DevOps — church-autoadvance

## 2026-11-19 — Q7 final (`q7/base` @ 6386f6b; both verifiers passed)
- **Both verifiers passed at 6386f6b.** At the head, `npx tauri build --debug --bundles nsis` gives an `installer.nsi` that names neither tool, with fresh `voxxad.exe` and `voxxa-eval.exe` in the target folder; the `--features cli` control names both. `build.yml` passes no feature. `ci.yml` lints and tests with `--features cli`.
- **Correction:** the entry below said full-eval "runs once on the PR that adds it". It runs on **every push** to that PR, because the paths filter looks at the whole PR diff; `cancel-in-progress` stops the older run. After the merge it runs only by hand (`workflow_dispatch`).
- **Release path (decided, not yet done; for the orchestrator):** push `q7/base`, open a draft PR, get every job green (full-eval may warn), merge, then run full-eval once by hand on `main` and record its three tables and times in the QA report. No tag and no GitHub release (user decision 3).
- **Open, for the first CI run:** the Node 20 check on the older pins (below); record the per-OS run time.

## 2026-11-19 — Q7 verifier fixes (aa16e90)
- **`ci.yml`:** the `cargo test` step also sets `VOXXA_REQUIRE_NETNS: ${{ runner.os == 'Linux' && '1' || '' }}`. The matrix comment no longer says to read the `traced with` line in the log. That line shows only with `--nocapture`, and a missing `unshare -rn` now fails the job instead.
- **Installer exclusion re-proved at aa16e90:** the NSIS script names neither tool, and the `--features cli` control names both. The head installer was rebuilt last.

## 2026-11-19 — Q7 finish on `q7/base`
- **`Cargo.toml`:** the four `[[test]] required-features = ["cli"]` blocks are deleted (c9271f0). CI already runs `cargo test --features cli`, so CI does not change. A plain `cargo test` now fails the CLI tests with "run: cargo test --features cli" instead of skipping them. The `[[bin]]` comment now reads "CI passes it; run `cargo test --features cli`" (171ad76).
- **Exclusion re-proved at the head.** `npx tauri build --debug --bundles nsis` produced an `installer.nsi` whose "Copy external binaries" section is empty, and the file names neither tool. Fresh `voxxad.exe` and `voxxa-eval.exe` sat in the target folder during the build. Control: with `--features cli`, both tools are listed. The head installer was rebuilt last (14,245,540 bytes).
- **`ci.yml`:** a comment pins the reason for `ubuntu-22.04` (5fa4c76).
- **`full-eval.yml`:** the timing comment now gives the measured times: 38 min in Q7 on a shared PC, and about 12 min on an idle PC in Q6 (911d56c). `timeout-minutes` stays 180.
- **Open, for the first CI run:** check the full-eval run for Node 20 deprecation warnings on checkout v4.2.2, rust-cache v2.7.8, cache v4.2.3, setup-node v4.4.0 and upload-artifact v4.6.2. Bump any pin that warns, keeping it SHA-pinned. Record the per-OS run time.

## 2026-11-19 — Q7: the CLI stays out of the installers; full eval in its own workflow; release notes as a file

### Decisions
- **`voxxad` and `voxxa-eval` do not ship (user decision 2026-11-18; Q3-03 #47, Q5-03 #75).** They moved from `src-tauri/src/bin/` to `src-tauri/src/cli/`, with explicit `[[bin]]` entries that carry `required-features = ["cli"]`. The `cli` feature is off by default. `tauri build` (locally, or `build.yml`'s tauri-action) passes no feature, so the installers carry `voxxa` only. CI lints with and without the feature, and runs `cargo test --features cli` and the eval step with it. Commit b9692c5.
- **Why both a move and a feature:** tauri-cli 2.10.1 picks the binaries to bundle in `get_binaries` (`src/interface/rust.rs`). It takes every `[[bin]]` whose required features are enabled, **and every entry of `src/bin/` it has not already seen**, whatever that entry's features. A feature alone would not have removed a tool that lives in `src/bin/`. The same list feeds every bundle type (NSIS, MSI, deb, AppImage, `.app`).
- **A defect that was already there:** at 46cb6d1 the NSIS installer carried `voxxa-eval.exe`, so Q5-03 was real. `voxxad` was left out only by accident. tauri-cli adds `src/bin` entries with no source path, and `Path::ends_with("")` is true for any path, so the second `src/bin` folder in the listing is skipped. Which one comes second depends on the OS's directory order (NTFS lists `voxxa-eval` first). On Linux or macOS the installers could have carried `voxxad` instead.
- **Regression guard:** `src-tauri/tests/packaging.rs` runs without the feature. It checks that both tools need `cli` and live under `src/cli/`, that `cli` is not a default feature, and that `src/bin/` does not exist.
- **Full eval, report only (user decision):** `.github/workflows/full-eval.yml`.
  - **Triggers:** `workflow_dispatch`, plus `pull_request` with `paths: [.github/workflows/full-eval.yml]`, so it runs on the PR that adds it (on every push to that PR; see "Q7 final").
  - **Runners:** ubuntu-22.04, windows-latest and macos-latest, with `fail-fast: false` and a 180-minute timeout.
  - **Model:** it downloads `ggml-tiny.bin` from the app's own URL (`transcription/mod.rs`, `download_url`) into each OS's model folder and caches it (key = OS + SHA-256). It always re-checks the pinned SHA-256 `be07e048…1b21`. That hash equals the Hugging Face LFS pointer's oid (77,691,713 bytes) and the reference PC's copy.
  - **Run:** a debug build (release on Windows is slower, Q4-01), then `voxxa-eval --full`. Exit 0 or 1 passes. Exit 1 means a metric differs: the job prints a `::warning::` and the table, and never fails for it. Any other exit fails the job.
  - **Output:** stderr is dropped except the `voxxa-eval:` lines, so recognised words stay out of the CI log. The table goes to the job summary and to an artifact (`full-eval-<os>`).
  - **Hardening:** `permissions: contents: read`, `persist-credentials: false`, and every action pinned to a commit SHA.
- **CI hardening:** `ci.yml` gained `permissions: contents: read`, and installs `strace` for the socket test. Its actions still use tags (ticket Q7-03).
- **Release notes are a file (user decision: no tag, no GitHub release):** `docs/releases/voxxad-cli-preview.md`. It covers:
  - what changed in Q3-Q7;
  - how to build from source (`--features cli`; debug on Windows);
  - the model and its SHA-256;
  - safe first runs (`--shadow`) and the keystroke warning;
  - 16 kHz conversion with ffmpeg;
  - the privacy of session logs in the roaming `%APPDATA%` (`--log-dir`), and the network statement;
  - known limits.
- **Companion:** `docs/INTEGRATIONS.md`, "Driving `voxxad` from Bitfocus Companion". It gives start and stop scripts (a single-instance check, `--shadow` first, `pkill -INT` or `taskkill /F`), the keystroke warning "focus the presentation app", and the privacy and network notes. It names no Companion feature beyond "an action that runs a local program", and tells the reader to check their Companion version.

### Proof of the exclusion (Windows, `npx tauri build --debug --bundles nsis`, tauri-cli 2.10.1)
The table shows the "Copy external binaries" section of the generated `installer.nsi`. Every run had a stale `voxxad.exe` and `voxxa-eval.exe` in the target folder, so the bundler had files to copy if it wanted them.

| Tree | Lines under "Copy external binaries" |
|---|---|
| 46cb6d1 (before) | `File /a "/oname=voxxa-eval.exe" …\debug\voxxa-eval.exe` |
| b9692c5 (after) | none |
| b9692c5 with `--features cli` (control) | `voxxa-eval.exe` and `voxxad.exe` |

The installer built (`Voxxa_0.1.0_x64-setup.exe`, 14,244,652 bytes after the change). MSI, deb, AppImage and `.app` were not built locally; they take their binary list from the same `get_binaries`.

### Not done
- No tag, no release and no signing (user decision). `build.yml` is unchanged, and it now builds no tool.
- The full-eval workflow has not run on GitHub yet. It runs on the Q7 PR (paths filter), and that first run on each OS is also its test.

## 2026-10-24 — Signing and Store prep (local Windows session, handoff task 5)

### Built
- `docs/signing-policy.md` draft, written against SignPath Foundation's published terms
  (signpath.org/terms, read 2026-10-24): attribution line, Authors/Reviewers/Approvers
  table, tag-on-`main`-only release rule, manual approval per release, key custody,
  privacy statement in SignPath's required wording, uninstall instructions.

### Gaps before applying to SignPath Foundation
1. **Release must already exist.** Terms: the project must be "already released in the form
   to be signed". No GitHub Release exists yet; cut an unsigned `v0.x` preview first.
2. **Home page link.** "Code signing policy" must appear on the home page and the
   download/release page. `docs/index.html` (cloud-owned) needs a footer link, and GitHub
   Pages is not enabled on the repo (`has_pages: false`), so there is no live home page yet.
3. **Network opt-out.** Terms require software that contacts other systems to let the user
   disable it. The updater checks GitHub on its own; confirm an off switch exists or add
   one (cloud owns `src-tauri/src/`). Model downloads are user-initiated, so they are fine.
4. **File metadata.** SignPath enforces product name = project name and a version
   identical in each build: `productName: "Voxxa"`, `version: "0.1.0"` in
   `tauri.conf.json` already fit; keep Cargo, npm and Tauri versions in lockstep.
5. **MFA** on GitHub and SignPath for every member: founder to confirm on GitHub.
6. **Single-person team.** One person holds all three roles. The terms allow this, but
   an approver other than the author would be stronger if a second maintainer joins.

### Partner Center: individual vs company account (needs founder decision)
Microsoft's sign-up page (learn.microsoft.com/windows/apps/get-started/sign-up, updated
2026-07-17) says an **Individual** account is for distribution "not in relation to their
business, trade, or profession". ADR-008 plans a paid (~$19.99) listing and the handoff
records a US LLC, which matches the **Company** account description instead. Changing
Individual → Company "is not supported", and ADR-008 says publisher identity is costly
to reverse. Both account types are now free.

Recommendation: wait for the LLC, then open a **Company** account.
**Decision (founder, 2026-10-25): Company account, opened after the LLC is formed.**
Do not open an Individual account in the meantime.

| | Individual | Company (LLC) |
|---|---|---|
| Entry point | storedeveloper.microsoft.com → "Get started for free" → Individual developer | same → Company account |
| Sign-in | Personal Microsoft account only | Personal MSA or Entra ID work account |
| Identity | Government-issued photo ID + selfie, captured on a phone in good light, originals only; name should match the ID | Mandatory due diligence, then business verification + employment verification |
| Business proof | — | D-U-N-S number (fast, automated) **or** articles of organization / state registration / tax filing (manual review, 2–5 business days) |
| Email | Any | **Work email on the company's domain**; Gmail is refused, and a mismatched domain needs a domain-ownership record or invoice |
| Appeals | — | Up to three per verification type |
| Publisher name | Your own name | The LLC's legal name |

Founder checklist for a Company account:
1. LLC formed, with its articles of organization (or state registration).
2. A free D-U-N-S number for the LLC (dnb.com; can take days to weeks).
3. A domain for Voxxa and a mailbox on it (e.g. `founder@<domain>`).
4. Payout and tax profile in Partner Center before any paid listing: bank account
   and a US tax form for the LLC (W-9 with the LLC's EIN). Not verified this session;
   check Partner Center's payout page when the account exists.

Nothing has been submitted to SignPath or Microsoft.

### Skipped / risk
- I did not apply to SignPath or open any account; both need the founder.
- Azure Trusted Signing (the ADR-008 fallback) not researched this session.
- Payout/tax details are from general knowledge, not a primary source read today.

## 2026-11-23 — Q10 (10x lane, `q10/10x`): draft releases, signing paths, winget, release runbook
Decisions: ADR-020 § 6 (amends ADR-008). Runbook: `docs/RELEASING.md`. Founder decisions: stop at a draft; signing secrets later (everything must work without them); the orchestrator may push, open the PR, fix CI, merge, make **one** rc tag that creates a draft, and file tickets.

**`build.yml` (rewritten):**
- **release** job (`:46-117`): validates the tag (`vX.Y.Z` or `vX.Y.Z-…`), requires its base version to equal `tauri.conf.json`'s, requires an existing tag to point at the built commit and the commit to be on `main` (`:80`), then creates a **draft** with `docs/releases/release-body.md` (`:111`), `--prerelease` when the tag has a `-` (`:112`); re-runs reuse the draft; a published release stops the run (`:106`).
- **build** matrix (macOS arm64 on `macos-latest`, macOS x86_64 on **`macos-15-intel`** (`:130`, replaces the retired `macos-13`), Linux on `ubuntu-22.04`): tauri-action into the draft by `releaseId`, `includeUpdaterJson: false` (`:251`); `createUpdaterArtifacts` injected through a `--config` file only when `TAURI_SIGNING_PRIVATE_KEY` exists (`:216-230`); Linux `.asc` files now go into the draft too. Upload globs without the dead `*.nsis`.
- **windows** job: Tauri CLI build (NSIS `currentUser` + MSI) → collect exactly one `*-setup.exe` and one `.msi` (`:415`) → **SignPath** when all four SignPath values exist (`:435`, `@v3`, waits up to 1 h for the manual approval) → else **Azure Trusted Signing** via an injected `signCommand` (`:373`) → else unsigned with a `::warning::` → `Get-AuthenticodeSignature` (`:467`, must be `Valid` when a path ran) → `tauri signer sign` over the final bytes when the key exists (`:483`) → `gh release upload` → winget manifest filled from the final installer (`:499`) as the `voxxa-winget-manifest` artifact.
- **updater-manifest** job: skips cleanly without the key; otherwise downloads the draft's `.sig` files and writes `latest.json` once (`scripts/release/updater_manifest.py`, `:539`).
- Everywhere: Rust 1.99.1 as `ci.yml`; `CMAKE_PROJECT_INCLUDE` (portable ggml, `:43`; #142); `permissions: contents: write, actions: read`; concurrency per tag.
- **`winget.yml`** (new): on `release: published` (or by hand), skips drafts, pre-releases and non-`vX.Y.Z` tags; fills the manifest from the published installer; submits with `wingetcreate` v1.9.14.0 (SHA-256 pinned) only when `WINGET_TOKEN` exists.
- **`ci.yml`:** one step added to the `budgets` job (release script unit tests). The `media` job is unchanged (key, `run: |` block, no `continue-on-error`, no `if:`), and `media_guard.rs` now also fails on an `if:` (Q9V-03).

**Secrets and variables for the founder** (all optional; each lights up one step):
- SignPath (preferred): secrets `SIGNPATH_API_TOKEN`, `SIGNPATH_ORGANIZATION_ID`; variables `SIGNPATH_PROJECT_SLUG`, `SIGNPATH_SIGNING_POLICY_SLUG`; in SignPath, `packaging/signpath/artifact-configuration.xml` as the project's default artifact configuration and a manual-approval policy.
- Azure Trusted Signing (fallback): `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`, `AZURE_TENANT_ID`, `AZURE_CODE_SIGNING_ENDPOINT`, `AZURE_CODE_SIGNING_ACCOUNT`, `AZURE_CODE_SIGNING_CERTIFICATE_PROFILE`.
- Updater: `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (the key of pubkey 84F2CD98C608E9C9, or a new pair with the pubkey replaced).
- winget: `WINGET_TOKEN` (classic, `public_repo`). Optional, unchanged: `APPLE_*`, `GPG_PRIVATE_KEY`, `GPG_PASSPHRASE`.
- Also founder: make "media guard" a required check (Q9V-02); confirm the winget identity `nategroks.Voxxa` / publisher `nategroks` (`packaging/winget/`).

**SRE / Phase 6:** release runbook `docs/RELEASING.md`: cut (version PR, tag on `main`), watch (job list, SignPath approval within 1 h), check the draft (asset list, Authenticode lines, winget artifact, a test PC — never the booth PC), publish (rc never offered by the updater: `releases/latest` skips drafts and pre-releases), **rollback = delete the draft and its tag** before publishing; after publishing, `gh release edit <tag> --draft=true` so `latest` points back, and fix forward with a higher version (no downgrades). Key custody § 6.

**Validated locally:** actionlint 1.7.7 on every workflow (with `macos-15-intel` declared as a label, which that actionlint predates; GitHub's runner-images README lists it as a standard x64 label); PyYAML parse of every workflow and winget template; `bash -n` on all 31 bash blocks; the signing-path block with four secret combinations, the Azure override block (SignCommand args list) and the tag check (`v0.1.0-rc.1`, `v0.1.0` pass; `v0.2.0`, `0.1.0`, an injected `;` refused); `scripts/release` 10/10 unit tests; `tests/release_config.rs` pins the draft, the order SignPath/Azure → Authenticode check → minisign → upload, the skip conditions and the matrix.

**What remains for the orchestrator's tag run** (one rc tag, pre-approved), cost: GitHub-hosted runners are free for this public repository (macOS included), about 30-45 min wall time:
1. Merge first; then tag `v0.1.0-rc.1` on the merge commit of `main` (`tauri.conf.json` is 0.1.0). Any other version is refused.
2. Expect: one **draft pre-release** `v0.1.0-rc.1`; Windows job log "No Windows signing secrets … unsigned" and "TAURI_SIGNING_PRIVATE_KEY not set"; Authenticode lines `NotSigned`; assets `Voxxa_0.1.0_x64-setup.exe`, `Voxxa_0.1.0_x64_en-US.msi`, two `.dmg`, an `.AppImage`, a `.deb` (no `.sig`, no `latest.json`); updater-manifest prints the skip line; the `voxxa-winget-manifest` artifact holds three filled files.
3. Unproven until then: tauri-action's `releaseId` upload into our draft; `macos-15-intel` and the portable ggml on both macOS runners; `gh release upload` from Windows bash; the NSIS installer really installs per-user (read the generated `installer.nsi`, or try on a test PC — not LHMEZPC01); SignPath (`@v3` inputs, the artifact configuration's wildcards), Azure, the updater signatures and `latest.json` until their secrets exist; `winget.yml` until a full release is published; the in-app update from one published release to the next.
4. Do not publish the draft (founder decision); the founder publishes the real release.

## 2026-11-23 — Q10 base (`q10/base`): grafts and review fixes in the release workflows
- **Pinned:** every `uses:` in `build.yml` and `winget.yml` is a full commit SHA with a `# version` comment: tauri-action `84b9d35…` (v0.6.2, what `@v0` points at), SignPath `a2c5b60…` (v3.2, what `@v3` points at; released 2026-11-20), rust-toolchain `683b588…` and rust-cache `d1527c8…` (as `full-eval.yml`), checkout `11d5960…` (v4.4.0), setup-node `49933ea…` (v4.4.0), upload-artifact `ea165f8…` (v4.6.2), each resolved with `git ls-remote` and checked as a commit with the read-only API. `release_config.rs` fails on an unpinned `uses:` (Q2-28 closed for these two workflows; `ci.yml`, `deploy` and the cargo-deny action are unchanged).
- **Azure** needs all six `AZURE_*` values; a partial SignPath or Azure setup warns with the missing names and falls through to `signing=none`.
- **latest.json** job: gated on `needs.windows.outputs.updater == 'yes'`; the key is not in its env; `updater_manifest.py` exits 1 with `::error::` unless `windows-x86_64`, `darwin-aarch64`, `darwin-x86_64` and `linux-x86_64` all have an entry.
- **Tag run, changed expectations** (otherwise as above): the Windows log says "::warning::No Windows signing path is fully configured: the installers are unsigned and SmartScreen will warn." and "TAURI_SIGNING_PRIVATE_KEY not set: no updater signatures."; the updater-manifest job runs only its "No updater key" step. Also unproven until the tag run: the pinned SHAs resolve on GitHub's runners (they are real commits, checked), and tauri-action v0.6.2's macOS updater asset names match `updater_manifest.py` (now a hard failure, only once the key exists).
- **Secrets and variables to add:** unchanged from the list above; Azure is used only when all six secrets are set.

## 2026-11-23 — Q10 base (`q10/base`): verifier fix, macOS without the Apple secrets (94d5ac5)
- **Gap (verifier, case i, plausible):** the tauri-action step set `APPLE_SIGNING_IDENTITY`, `APPLE_API_ISSUER`, `APPLE_API_KEY` and `APPLE_TEAM_ID` from secrets even when they were missing, so the Tauri CLI got them as empty strings. The CLI reads `APPLE_SIGNING_IDENTITY` whenever the variable exists and it replaces the ad-hoc `"-"` from `tauri.conf.json`, so an empty identity could reach codesign and fail both macOS jobs.
- **Fix:** a macOS-only step "Apple signing values" (`shell: bash`) writes each of the four to `GITHUB_ENV` only when it is not empty, and logs "<NAME> not set: left out of the build." otherwise; the tauri-action step has no `APPLE_*` entry. GitHub masks the secret values in the log. `release_config.rs::macos_builds_unsigned_when_the_apple_secrets_are_missing` failed on the old block and passes on the new one. The block was run locally in bash with all four empty (nothing written to the env file) and with three fake values and one empty (only those three written). actionlint 1.7.7 (with `macos-15-intel` declared) and PyYAML clean on every workflow.
- **Tag run, changed expectations:** each macOS job log shows four "… not set: left out of the build." lines in "Apple signing values", and both macOS jobs build ad-hoc signed and not notarized, uploading their `.dmg` into the draft. This is still unproven until the orchestrator's rc tag run; if a macOS job fails at codesign anyway, the cause is elsewhere (this env gap is closed).

## 2026-11-23 — Q10 final (`q10/base` @ f424c74): what the tag run must show, and the founder's secrets
**DevOps position: ready to merge once CI is green, then one rc tag.** Both verifiers passed in round 2. Verifier A traced every secret combination through `build.yml` (`reviews/2026-11-23-ponytail-q10.md` § 8); verifier B's three surviving workflow mutants are Q10-08.

**The orchestrator's tag run** (pre-approved: one rc tag that creates a draft). The full list, merged from the sections above:
1. Push `q10/base` (it edits workflows, so use the `workflow` scope), open the PR, get every CI job green (the new "Release script tests" step in `budgets` runs on GitHub for the first time), merge.
2. Tag **`v0.1.0-rc.1`** on `main`'s merge commit and push the tag. `tauri.conf.json` is 0.1.0; any other version, a tag on another commit, or a commit not on `main` is refused by the release job.
3. Expect, and record with the run ID:
   - release job: "Draft release v0.1.0-rc.1 of version 0.1.0 (pre-release: true) from <sha>"; one **draft pre-release**;
   - Windows: "::warning::No Windows signing path is fully configured: the installers are unsigned and SmartScreen will warn.", "TAURI_SIGNING_PRIVATE_KEY not set: no updater signatures.", Authenticode `NotSigned` for both installers, exactly one `*-setup.exe` and one `.msi` collected;
   - macOS (both runners): four "… not set: left out of the build." lines, "APPLE_CERTIFICATE not set: the macOS build is not signed.", ad-hoc signed and not notarized, `.dmg` uploaded; `macos-15-intel` accepted;
   - Linux: "GPG_PRIVATE_KEY not set — Linux artifacts will not be signed.", `.AppImage` and `.deb` uploaded;
   - assets: `Voxxa_0.1.0_x64-setup.exe`, `Voxxa_0.1.0_x64_en-US.msi`, two `.dmg`, an `.AppImage`, a `.deb`; **no** `.sig`, `.asc` or `latest.json`;
   - updater-manifest: only its "No updater key" step runs;
   - artifact `voxxa-winget-manifest` with three files and no `{{` placeholder left.
4. Also check: the pinned SHAs resolve on the runners; tauri-action uploads into our draft by `releaseId`; the portable ggml build works on both macOS runners; `gh release upload` works from the Windows job; per-user NSIS (read the generated `installer.nsi` from a clean local `tauri build --bundles nsis` without installing, or try the draft's installer on a test PC, never LHMEZPC01); only `voxxa.exe` is in the installers (#100).
5. **Do not publish the draft.** If the run fails, fix forward on a branch (CI fixes are pre-approved); deleting the draft and the tag is the rollback (`RELEASING.md` § 5). A second rc tag needs the founder's approval: only one is pre-approved.

**Unproven until the founder's secrets exist:** SignPath (the `@v3` inputs, the artifact configuration's wildcards, the manual approval within 3600 s), Azure Trusted Signing, the updater signatures and `latest.json` (including tauri-action's macOS updater asset names, which `updater_manifest.py` requires), notarization. `winget.yml` waits for a published full release; the in-app update needs two published releases.

**Secrets and variables the founder adds** (Settings → Secrets and variables → Actions; all optional; each lights up one step; `docs/RELEASING.md` § 1):
- **SignPath Foundation (preferred), all four or none:** secrets `SIGNPATH_API_TOKEN`, `SIGNPATH_ORGANIZATION_ID`; variables `SIGNPATH_PROJECT_SLUG`, `SIGNPATH_SIGNING_POLICY_SLUG`. In SignPath: `packaging/signpath/artifact-configuration.xml` as the project's default artifact configuration, and a signing policy with manual approval.
- **Azure Trusted Signing (fallback, used only without a full SignPath setup), all six or none:** secrets `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`, `AZURE_TENANT_ID`, `AZURE_CODE_SIGNING_ENDPOINT`, `AZURE_CODE_SIGNING_ACCOUNT`, `AZURE_CODE_SIGNING_CERTIFICATE_PROFILE`.
- **Updater:** secrets `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`, matching pubkey **84F2CD98C608E9C9** in `tauri.conf.json`. Confirm you hold that key and password, or generate a new pair yourself (`npx tauri signer generate`, offline) and replace the pubkey in a PR before the first published release. Agents never handle the key.
- **winget:** secret `WINGET_TOKEN` (a classic personal access token with `public_repo`). Confirm the identity `nategroks.Voxxa` / publisher `nategroks` before the first submission (hard to change later).
- **Unchanged, optional:** the `APPLE_*` secrets in `SIGNING.md`; `GPG_PRIVATE_KEY`, `GPG_PASSPHRASE`.
- **Branch protection:** make "media guard" a required status check on `main` (#159).

**Before the founder publishes v0.1.0:** fix #59 (the Windows installers carry an unoptimised whisper.cpp); publish the draft by hand (`RELEASING.md` § 4); then update `README.md`, `docs/index.html` and `docs/quick-start.md` with its test (#134). An rc never reaches a booth PC (`RELEASING.md` § 4).

## 2026-11-23 — Q10 tag run (`v0.1.0-rc.1`): a draft pre-release from `main`
**First run (45258000510, tag on 4dc17e6): green, but incomplete.** The draft job created the release and then read its id back from the releases list, which had not caught up: the job output `id` was empty. tauri-action (macOS and Linux) logged "No releaseId or tagName provided, skipping all uploads"; Windows uploads by tag and worked. Fixed forward in PR #180 (86c140e): the job asks again for up to 30 s and fails if the id is not a number; `release_config.rs` pins it (a mutant without the check fails). The draft and the tag were deleted and the same tag re-cut on 86c140e (RELEASING.md § 5); not a second rc.

**Second run (45268876214, tag on 86c140e): passed.** The Apple Silicon job was re-run once because GitHub had no arm64 runner ("not acquired by Runner … capacity constraints"); nothing of ours failed.
- Release job: "Draft release v0.1.0-rc.1 of version 0.1.0 (pre-release: true) from 86c140e…"; the release is a **draft pre-release**, target 86c140e.
- Windows: "No Windows signing path is fully configured…" warning and "TAURI_SIGNING_PRIVATE_KEY not set: no updater signatures."; Authenticode `NotSigned` for both installers (first run's log; same steps).
- macOS (both runners): four "… not set: left out of the build." lines and "APPLE_CERTIFICATE not set: the macOS build is not signed."; `macos-15-intel` accepted; the portable ggml include is in every release job's env.
- Linux: "GPG_PRIVATE_KEY not set — Linux artifacts will not be signed."
- Assets: `Voxxa_0.1.0_x64-setup.exe`, `Voxxa_0.1.0_x64_en-US.msi`, `Voxxa_0.1.0_aarch64.dmg`, `Voxxa_0.1.0_x64.dmg`, `Voxxa_0.1.0_amd64.AppImage`, `Voxxa_0.1.0_amd64.deb`; also `Voxxa-0.1.0-1.x86_64.rpm` and `Voxxa_{aarch64,x64}.app.tar.gz`, which tauri-action uploads by default (harmless; drop them in a later release-config change if unwanted). **No** `.sig`, `.asc` or `latest.json`.
- Updater manifest: only "No updater key" ran.
- `voxxa-winget-manifest` artifact: three files, no `{{` left, `Scope: user`, `InstallerSha256` equal to the uploaded `-setup.exe`'s GitHub digest.
- Per-user NSIS and #100: a local `tauri build --bundles nsis` under the scratch identifier `com.voxxa.qansis` (nothing installed; build folder deleted) generated `installer.nsi` with `INSTALLMODE "currentUser"` (`RequestExecutionLevel user`), `INSTALLWEBVIEW2MODE "embedBootstrapper"`, and one `File` line, `voxxa.exe`, with no resources or external binaries.

Still unproven until the founder's secrets exist: SignPath, Azure, notarization, the updater signatures and `latest.json`. The draft stays unpublished; it is for checking only and never goes on a booth PC.
