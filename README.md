# Voxxa

Voxxa listens to the room and advances the lyric slides for you. It runs beside the presentation software your church already uses, follows the worship leader through repeats and bridges, and never shows a word on screen itself.

**Free, open source, and local.** Audio never leaves the computer. There is no account, no cloud, and no telemetry.

## What it does

- Listens to the sound desk feed or a room mic and matches what is sung against the lyrics already loaded in your presentation software.
- Tells the presentation software when to move: next slide, back a slide, or jump to the chorus.
- In *Songs + Scripture* mode, hears a spoken reference ("Romans 8, verse 28"), confirms it from the first words read, and asks the presentation software to show the verse.
- Shows one screen to the operator: a three-position switch (Off / Songs / Songs + Scripture), a status light, and a Call for help button.

## What it does not do

- It never displays lyrics or Bible text. Your presentation software does that, under your church's licences.
- It never uploads audio or transcripts. Speech recognition runs on the computer in the booth.
- It never replaces your presentation software or reports to CCLI for you.

## Works with

| Presentation software | Integration |
|---|---|
| ProPresenter 7.9 and later | Native (network API): drives slides and reads the current slide |
| FreeShow | Native (REST/WebSocket API): drives slides, reads the current slide, shows scripture |
| OpenLP | Native (HTTP API): drives slides, reads the current slide, shows scripture |
| EasyWorship, MediaShout, WorshipTools Presenter, others | Keystrokes (Voxxa sends the next/previous key while the app has focus) |

Native adapters arrive in the order above; see the roadmap in `.10x/specs/`.

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
