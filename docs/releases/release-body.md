## Voxxa — voice-following slide advancer for churches

Voxxa listens to the room and tells your presentation software when to change the lyric slide. It never projects lyrics and never uploads audio.

| Platform | File |
|----------|------|
| Windows (no administrator rights needed) | `_x64-setup.exe` |
| Windows (IT-managed PCs, per machine) | `_x64_en-US.msi` |
| macOS (Apple Silicon) | `_aarch64.dmg` |
| macOS (Intel) | `_x64.dmg` |
| Linux | `.AppImage` / `.deb` |

### First service
1. Install and open Voxxa; follow the first-run steps (audio device, presentation software, speech model).
2. Load today's setlist and choose **Songs**.
3. Your hands stay free; the operator can always take over.

### Updates
Voxxa never checks for updates by itself. In **Advanced → Settings → Updates**, **Check for updates** asks GitHub once; Voxxa installs a newer version only after you confirm, and never while it is listening. Updates are signed, and an older or equal version is never installed.

> Audio never leaves the computer. No telemetry; network only when the operator starts a model download, a Planning Center import, a Hymnary.org lookup or an update check.

Windows builds are signed when the project's signing service is set up; see the [code signing policy](https://github.com/nategroks/voxxa/blob/main/docs/signing-policy.md). Until then the installers are unsigned and Windows SmartScreen warns before running them.
