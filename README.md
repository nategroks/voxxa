# Voxxa

Voxxa listens to the room and advances the lyric slides for you. It runs beside the presentation software your church already uses, follows the worship leader through repeats and bridges, and never projects lyrics: the projected screen stays with your presentation software.

**Free, open source, and local.** Audio never leaves the computer. There is no account and no telemetry, and no network traffic unless the operator starts it: downloading a speech model, importing a plan from Planning Center, or searching Hymnary.org for a public-domain hymn.

## What it does

- Listens to the sound desk feed or a room mic and matches what is sung against the lyrics already loaded in your presentation software.
- Tells the presentation software when to move: next slide, back a slide, or jump to the chorus.
- Blanks the projected output when the music stops and brings the slide back when the singing resumes.
- Loads the setlist from a Voxxa JSON file, from files exported by common worship software (see `docs/SETLIST_FORMATS.md`), or from a Planning Center Services plan.
- Shows the operator, on the booth computer only, which song it hears and a preview of the current and next slide.

On the roadmap, not in the code yet:

- *Songs + Scripture* mode: hear a spoken reference ("Romans 8, verse 28"), confirm it from the first words read, and ask the presentation software to show the verse.
- A one-screen Simple mode for volunteers: a three-position switch (Off / Songs / Songs + Scripture), a status light, and a Call for help button.

## What it does not do

- It never projects lyrics or Bible text. Your presentation software does that, under your church's licences. The operator previews on the booth computer are not sent to any output.
- It never uploads audio or transcripts. Speech recognition runs on the computer in the booth.
- It never replaces your presentation software or reports to CCLI for you.

## Works with

| Presentation software | Integration |
|---|---|
| ProPresenter 7.9 and later | Native (network API): drives and blanks slides, reads the current slide |
| ProPresenter 6, and 7.0 to 7.8 | Native (WebSocket remote): drives slides; no blank |
| FreeShow | Native (network API): drives and blanks slides |
| OpenLP 2.x | Native (HTTP API): moves and blanks slides, reads the current slide |
| OpenSong | Native (network API): drives and blanks slides |
| EasyWorship, MediaShout, WorshipTools Presenter, others | Keystrokes (Voxxa sends the next/previous key while the app has focus) |

Reading the current slide from FreeShow, and showing scripture through FreeShow and OpenLP, are on the roadmap in `.10x/specs/`. Driver details: `docs/INTEGRATIONS.md`.

## Platforms

Windows first, then macOS, then Linux (AppImage). Download from the [releases page](https://github.com/nategroks/voxxa/releases).

## Support

The software is free. Churches that want a person to call on Sunday morning can buy **Sunday Support** or a one-time **Setup & Calibration**. Details will be published with the first Windows release.

## Development

Built with Rust and Tauri 2. See `.10x/` for the architecture decisions, delivery plan and research behind the project, and `CONTRIBUTING.md` for the code layout and the presenter-driver interface.

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
