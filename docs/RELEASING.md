# Releasing Voxxa (maintainers)

How a release is built, checked, published and rolled back. The decisions are in
`.10x/adrs/020-operator-initiated-updates-csp-windows-release.md` (§ 6) and ADR-008. The
workflow is `.github/workflows/build.yml`; winget is `.github/workflows/winget.yml`.

**The rule:** a tag only ever makes a **draft**. Nobody outside the repository sees a draft.
A person publishes it by hand after checking it.

## 1. Secrets and variables (once; all optional)

Settings → Secrets and variables → Actions. Each one lights up one step; without it the step
says so in the log and the build still succeeds.

| What | Secrets | Variables | Without it |
|---|---|---|---|
| Windows signing, SignPath Foundation (preferred) | `SIGNPATH_API_TOKEN`, `SIGNPATH_ORGANIZATION_ID` | `SIGNPATH_PROJECT_SLUG`, `SIGNPATH_SIGNING_POLICY_SLUG` | Azure if configured, else unsigned (warning) |
| Windows signing, Azure Trusted Signing (fallback) | `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`, `AZURE_TENANT_ID`, `AZURE_CODE_SIGNING_ENDPOINT`, `AZURE_CODE_SIGNING_ACCOUNT`, `AZURE_CODE_SIGNING_CERTIFICATE_PROFILE` | | unsigned (warning) |
| Updates (all platforms) | `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | | no `.sig` files and no `latest.json`; installed copies cannot update to this release |
| winget | `WINGET_TOKEN` (classic token, `public_repo`) | | the filled manifest stays an artifact |
| macOS, Linux | `APPLE_*` (see `SIGNING.md`), `GPG_PRIVATE_KEY`, `GPG_PASSPHRASE` | | unsigned macOS / no `.asc` |

SignPath also needs, in SignPath itself: the project, a signing policy with manual approval,
and `packaging/signpath/artifact-configuration.xml` as the project's **default** artifact
configuration. All four SignPath values must be set, or SignPath is not used; likewise all
six Azure values, or Azure is not used. The log warns which values are missing.

## 2. Cut a release

1. The version lives in `src-tauri/tauri.conf.json` (`version`). Change it through a pull
   request (keep `src-tauri/Cargo.toml` and `package.json` the same). A tag must match it:
   `v0.1.0` or `v0.1.0-rc.1` both build 0.1.0. The workflow refuses any other tag.
2. After the pull request is merged, tag the merge commit on `main` and push the tag:
   `git tag v0.1.0-rc.1 <sha on main>` then `git push origin v0.1.0-rc.1`.
   (Or run "Build and Release" by hand on `main` with the tag as input.) A commit that is
   not on `main` is refused, so nothing else can be signed.
3. Watch the run. Jobs: **Draft release** (creates the draft; pre-release when the tag has a
   `-`), **Build - macOS (Apple Silicon)**, **Build - macOS (Intel)**, **Build - Linux**,
   **Build - Windows**, then **Updater manifest (latest.json)**.
   With SignPath configured, the Windows job waits up to one hour for you to **approve the
   signing request in SignPath**.
4. A failed job: fix forward and re-run it. Re-running reuses the same draft (assets are
   replaced). A published release is never rebuilt: the workflow stops instead.

## 3. Check the draft before publishing

- **Assets.** Windows: `Voxxa_<v>_x64-setup.exe` and `Voxxa_<v>_x64_en-US.msi`; macOS: two
  `.dmg` and two `.app.tar.gz`; Linux: `.AppImage` and `.deb`. With the updater key: a `.sig`
  next to each updater bundle and `latest.json`. With GPG: `.asc` files.
- **Windows signature.** The Windows job prints `Get-AuthenticodeSignature` for both
  installers; with a signing path it fails unless both are `Valid`.
- **winget.** Download the `voxxa-winget-manifest` artifact and read the three files
  (version, installer URL, SHA-256).
- **Try it on a test PC**, not the booth PC: install the NSIS installer (no administrator
  prompt), start Voxxa, and open Advanced → Settings → Updates. Do not install anything on
  LHMEZPC01 without deciding to.

## 4. Publish

- **A release candidate** (`-rc.N`) can stay a draft, or be published as a pre-release for
  testers. Either way the in-app updater never offers it: the endpoint is GitHub's
  `releases/latest`, which skips drafts and pre-releases.
- **An rc installs as the final version number.** `v0.1.0-rc.1` builds 0.1.0 installers (§ 2),
  so a copy installed from an rc reports 0.1.0, and the in-app updater never moves it to the
  final 0.1.0: equal is no update (ADR-020 § 2). A tester must install the final installer by
  hand over it. **Never put an rc on a booth PC.** If an rc ever reaches churches anyway,
  ship the final release under the next patch number (0.1.1), so those copies are offered it.
- **A full release:** Edit the draft → Publish. From then on, **Check for updates** in
  installed copies offers it (only to older versions). Publishing triggers `winget.yml`,
  which fills the manifest from the published installer and, with `WINGET_TOKEN`, opens a
  pull request on `microsoft/winget-pkgs`.

## 5. Roll back

- **Before publishing:** delete the draft (`gh release delete <tag> --yes`) and the tag
  (`git push --delete origin <tag>`, then `git tag -d <tag>`). No user saw anything.
- **After publishing a bad full release:** turn it back into a draft
  (`gh release edit <tag> --draft=true`) so `releases/latest` points at the previous release again and no more copies update to it.
  Copies that already updated stay on it: Voxxa refuses to install an older or equal version
  (ADR-020 § 2), so the fix is a new, higher version. Close or ask to revert the winget pull
  request if it is still open.

## 6. Keys (custody)

- **Updater key** (minisign, key ID `84F2CD98C608E9C9`, public half in
  `tauri.conf.json` → `plugins.updater.pubkey`). The private key and its password live
  offline with two holders (ADR-008) and in the `TAURI_SIGNING_PRIVATE_KEY` secret, nowhere
  else; never in the repository, a chat or an issue.
  - **Lost:** installed copies can no longer be updated through the app. Generate a new pair,
    put the new public key in `tauri.conf.json`, and ask churches to install the next
    release by hand once.
  - **Leaked:** delete the secret, generate a new pair, and ship the new public key in a
    release signed with the old key as soon as possible; after that, releases are signed
    with the new key only.
  - Agents never generate, read or handle this key.
- **Windows code signing:** SignPath Foundation keeps the certificate's key in its HSM;
  nobody on the project has it (`docs/signing-policy.md`). Azure Trusted Signing keeps it
  in Azure; the app registration's client secret is the thing to rotate.
- **winget token:** a classic token with `public_repo` only, from the account whose fork of
  `winget-pkgs` receives the pull requests. Give it an expiry and renew it.
