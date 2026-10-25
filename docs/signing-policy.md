# Code signing policy

> **Draft for the founder's review.** Not yet live. SignPath Foundation requires this
> page to be reachable from the project home page, and the words "Code signing policy"
> to appear as a header or link on both the home page and the download/release page.

Free code signing provided by [SignPath.io](https://about.signpath.io/), certificate by
[SignPath Foundation](https://signpath.org/).

## What is signed

Only Windows installers and executables built by this repository's GitHub Actions
workflow (`.github/workflows/build.yml`) from source in
[nategroks/voxxa](https://github.com/nategroks/voxxa). Nothing built on a personal
machine is ever signed. Each signed file carries product name **Voxxa** and the version
of the release it belongs to.

Microsoft Store packages are signed by Microsoft during Store certification, not under
this policy. macOS builds are signed with an Apple Developer ID, also not under this
policy.

## When something is signed

1. A release is cut by pushing a tag `vMAJOR.MINOR.PATCH` that points at a commit on
   `main`. Tags on any other branch are not signed.
2. `main` is protected: changes arrive only through pull requests with passing CI
   (fmt, clippy, tests, licence and advisory checks).
3. The tag triggers the build workflow, which submits the unsigned artefacts to
   SignPath.
4. An approver reviews the signing request in SignPath and approves it by hand. Every
   release needs a fresh manual approval; there is no automatic signing.

## Team roles

| Role | Who | What they may do |
|---|---|---|
| Authors (committers) | [@nategroks](https://github.com/nategroks) | Merge to `main` |
| Reviewers | [@nategroks](https://github.com/nategroks) | Review pull requests from anyone who is not a committer |
| Approvers | [@nategroks](https://github.com/nategroks) | Approve each signing request in SignPath |

Membership: [repository collaborators](https://github.com/nategroks/voxxa/settings/access).
<!-- TODO founder: replace with a public link (e.g. a GitHub team page) if SignPath asks for one. -->

Pull requests from outside contributors, including code drafted with AI assistants,
are reviewed by a reviewer before merging. All team members use multi-factor
authentication on GitHub and on SignPath.

## Keys

The signing certificate's private key is held by SignPath Foundation in a hardware
security module. No member of this project has access to it or a copy of it. The
separate Tauri updater key (which lets the app verify its own updates) is stored only as
an encrypted GitHub Actions secret and is never committed.

## Privacy

This program will not transfer any information to other networked systems unless
specifically requested by the user or the person installing or operating it. In
particular:

- Audio is processed on the computer running Voxxa and is never uploaded.
- Speech-recognition models are downloaded from
  [Hugging Face](https://huggingface.co/ggerganov/whisper.cpp) only when the user picks a
  model in the Models tab. Hugging Face's [privacy policy](https://huggingface.co/privacy)
  applies to that download.
- Update checks fetch `latest.json` from GitHub Releases. GitHub's
  [privacy statement](https://docs.github.com/site-policy/privacy-policies/github-general-privacy-statement)
  applies. <!-- TODO cloud/founder: SignPath requires an option to disable this; confirm a setting exists or add one before applying. -->

## Uninstalling

Use **Settings → Apps → Installed apps → Voxxa → Uninstall**, or run `uninstall.exe`
in the install folder. Settings are kept in `%APPDATA%\com.voxxa.app` and can be deleted
by hand.
