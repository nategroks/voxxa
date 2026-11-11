# Voxxa

Voxxa listens to the room and advances the lyric slides for you. It runs beside the presentation software your church already uses, follows the worship leader through repeats and bridges, and never projects lyrics itself: the projected display stays with your presentation software.

**Free, open source, and local.** Audio never leaves the computer. There is no account and no telemetry, and Voxxa makes no network traffic unless the operator starts it: a speech-model download, a Planning Center import or a Hymnary.org lookup.

## What it does

- Listens to the sound desk feed or a room mic and matches what is sung against the lyrics already loaded in your presentation software.
- Tells the presentation software when to move: next slide, back a slide, or jump to the chorus.
- *Roadmap:* in *Songs + Scripture* mode, hear a spoken reference ("Romans 8, verse 28"), confirm it from the first words read, and ask the presentation software to show the verse.
- *Roadmap:* one simple screen for the operator: a three-position switch (Off / Songs / Songs + Scripture), a status light, and a Call for help button. Today the operator window shows the full set of panels, including a preview of the current and next slide.

## What it does not do

- It never projects lyrics or Bible text. Your presentation software does that, under your church's licences. Previews appear only on the operator's own screen.
- It never uploads audio or transcripts. Speech recognition runs on the computer in the booth.
- It never replaces your presentation software or reports to CCLI for you.

## Works with

| Presentation software | Integration |
|---|---|
| ProPresenter 7.9 and later | Native (REST API): drives slides and reads the current slide |
| ProPresenter 6 and 7.0–7.8 | Native (WebSocket API): drives slides |
| FreeShow | Native (API): drives and blanks slides |
| OpenLP | Native (HTTP API): blanks and reads the current slide |
| OpenSong | Native (API): drives and blanks slides |
| EasyWorship, MediaShout, WorshipTools Presenter, others | Keystrokes (Voxxa sends the next/previous key while the app has focus) |

Scripture display is on the roadmap in `.10x/specs/`. Developer notes on the drivers are in `CONTRIBUTING.md`.

## Platforms

Windows first, then macOS, then Linux (AppImage). Download from the [releases page](https://github.com/nategroks/voxxa/releases).

## Support

The software is free. Churches that want a person to call on Sunday morning can buy **Sunday Support** or a one-time **Setup & Calibration**. Details will be published with the first Windows release.

## Development

Built with Rust and Tauri 2. See `.10x/` for the architecture decisions, delivery plan and research behind the project.

```bash
npm install
npx tauri dev       # run with hot reload
npx tauri build     # build installers
```

Linux build dependencies (Ubuntu/Debian):

```bash
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libasound2-dev libxdo-dev
```

## Licence

GPL-3.0-or-later. See `LICENSE`. Contributions are accepted with a Developer Certificate of Origin sign-off; see `CONTRIBUTING.md`.
