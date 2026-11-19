# DevOps — church-autoadvance

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
  - **Triggers:** `workflow_dispatch`, plus `pull_request` with `paths: [.github/workflows/full-eval.yml]`, so it runs once on the PR that adds it.
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
