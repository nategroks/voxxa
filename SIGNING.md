# Signing & notarization

Voxxa's release builds in `.github/workflows/build.yml` are wired to sign and
notarize binaries on macOS and Windows **when the relevant secrets are
configured**. Without them, builds still succeed but produce unsigned
artifacts that trigger Gatekeeper/SmartScreen warnings.

This file documents the secrets each platform needs and the one-time setup
to obtain them.

## macOS — Developer ID + notarization

**Budget:** $99/yr (Apple Developer Program).

### One-time setup

1. Enroll at <https://developer.apple.com/programs/>. Individual enrollment
   typically clears in 24–48 hours; organizations need a D-U-N-S number and
   can take 1–4 weeks.
2. In the Apple Developer portal, create a **Developer ID Application**
   certificate. Download the `.cer`, install into Keychain Access, then export
   the certificate **with its private key** as a `.p12` file. Set a strong
   password.
3. In App Store Connect → Users and Access → **Integrations → App Store
   Connect API**, create a new key with **Developer** role. Download the
   `AuthKey_XXXXXX.p8` file (Apple lets you download it exactly once — save
   it). Note the Key ID and the Issuer ID.

### Repository secrets to set

| Secret | Value |
|---|---|
| `APPLE_CERTIFICATE` | Base64 of the `.p12` file: `base64 -i certificate.p12 \| pbcopy` |
| `APPLE_CERTIFICATE_PASSWORD` | Password you set when exporting the `.p12` |
| `KEYCHAIN_PASSWORD` | Any non-empty string; used to lock the temporary CI keychain |
| `APPLE_SIGNING_IDENTITY` | Exact identity string, e.g. `Developer ID Application: Your Name (TEAMID)` |
| `APPLE_TEAM_ID` | 10-character team ID from the developer portal |
| `APPLE_API_KEY` | Base64 of the `AuthKey_XXXXXX.p8` file: `base64 -i AuthKey_XXXXXX.p8 \| pbcopy` |
| `APPLE_API_KEY_ID` | The Key ID (e.g. `XXXXXX`) |
| `APPLE_API_ISSUER` | The Issuer ID (looks like a UUID) |

The workflow uses the App Store Connect API key path for notarization, which
is materially more reliable than the older `APPLE_ID` + app-specific-password
flow (see §5.2 of the project plan).

### Required Info.plist usage strings

Tauri 2 generates the Info.plist at build time. The following keys are
**not** currently emitted from `tauri.conf.json` and must be added manually
post-build (or via a custom `signCommand` wrapper). Until that integration
lands, document them in your release process:

```xml
<key>NSMicrophoneUsageDescription</key>
<string>Voxxa listens to room audio locally to detect songs. Audio never leaves your computer.</string>
<key>NSLocalNetworkUsageDescription</key>
<string>Voxxa connects to your presentation software on this network.</string>
<key>NSBonjourServices</key>
<array>
  <string>_pro7stagedsply._tcp.</string>
  <string>_pro6stagedsply._tcp.</string>
</array>
```

The `NSLocalNetworkUsageDescription` plus `NSBonjourServices` pair is what
unlocks the macOS Local Network permission prompt — without it, mDNS browse
and any `.local` connection silently fail on macOS 14+.

## Windows — Microsoft Artifact Signing

**Budget:** ~$120/yr (Artifact Signing Basic — $9.99/month, 5,000 sigs/month).

> **Eligibility:** Microsoft Artifact Signing (renamed from Azure Trusted
> Signing at April 2026 GA) is restricted to **US, Canadian, EU, or UK
> businesses**. Contributors outside those regions need an alternative
> HSM-backed path — SSL.com eSigner or DigiCert KeyLocker each cost
> ~$250–500/yr. The plan §5.1 has the trade-offs.

### One-time setup

1. Have an Entra ID (Azure AD) tenant. If not, sign up at portal.azure.com.
2. In the Azure portal, create an **Artifact Signing account** and a
   **Certificate Profile** under it. Note the endpoint URL, account name,
   and certificate profile name.
3. Register an Entra ID application and grant it the **Trusted Signing
   Certificate Profile Signer** role on the account.
4. Generate a client secret for the app registration. Note the Application
   (client) ID, the directory (tenant) ID, and the secret value.

### Repository secrets to set

| Secret | Value |
|---|---|
| `AZURE_CLIENT_ID` | App registration's Application (client) ID |
| `AZURE_CLIENT_SECRET` | App registration's client secret value |
| `AZURE_TENANT_ID` | Directory (tenant) ID |
| `AZURE_CODE_SIGNING_ENDPOINT` | e.g. `https://eus.codesigning.azure.net` |
| `AZURE_CODE_SIGNING_ACCOUNT` | Artifact Signing account name |
| `AZURE_CODE_SIGNING_CERTIFICATE_PROFILE` | Certificate profile name |

The workflow installs `trusted-signing-cli` only when `AZURE_CLIENT_ID` is
present, and injects a `bundle.windows.signCommand` Tauri config override
that invokes it. Builds without the secrets produce unsigned `.msi` /
`.exe` (still functional, but SmartScreen will warn).

### SmartScreen reputation

Even with signing, fresh certificates take time to build SmartScreen
reputation. Submit your first signed binary to the
[Microsoft File Submission portal](https://www.microsoft.com/en-us/wdsi/filesubmission)
to speed up trust. EV certificates (~$300–600/yr) bypass this entirely if
you're willing to absorb the cost.

## Tauri updater key pair

Required for the in-app update path even when binaries are unsigned.

```bash
npx tauri signer generate -w voxxa-updater.key
```

The command prints a public key — paste it into `tauri.conf.json` under
`plugins.updater.pubkey` (already done in this repo). The private key file
needs a password and goes into repository secrets:

| Secret | Value |
|---|---|
| `TAURI_SIGNING_PRIVATE_KEY` | Contents of `voxxa-updater.key` |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Password you set when generating |

Without these, the workflow won't be able to sign the updater manifest and
auto-updates will fail signature verification on clients.

## Linux

No code-signing convention. Voxxa's `.AppImage` and `.deb` are emitted
unsigned. If you want detached signatures for distribution authenticity,
GPG-sign the artifacts after the workflow finishes and publish the
public key on your release page.

## Total realistic cost

Per the project plan §5.4:
- Apple Developer Program: **$99/yr**
- Microsoft Artifact Signing: **~$120/yr**
- Domain/HTTPS update server: **$15/yr**
- **Total: ~$235/yr**

Swap Artifact Signing for an EV cert (immediate SmartScreen trust) and
the total climbs to ~$520–720/yr.
