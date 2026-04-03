# Voxxa

Local-first voice dictation desktop app powered by Whisper. Runs entirely on-device with no cloud APIs.

## Features

- **Local Whisper transcription** via whisper.cpp — your audio never leaves your machine
- **Voice Activity Detection** — energy-based VAD gates audio so only speech hits the model
- **System tray app** — lives in your tray, activate with a global hotkey
- **Floating overlay** — see live transcription in a minimal always-on-top widget
- **Auto text insertion** — transcribed text is typed directly into the focused app
- **Multiple Whisper models** — from Tiny (75MB) to Distil-Large-v3 (756MB)
- **MCP integration** (planned) — voice-to-action via Model Context Protocol servers

## Architecture

Built with **Tauri 2.0** (Rust backend + web frontend):

- `src-tauri/` — Rust backend: audio capture (cpal), VAD, Whisper (whisper-rs), text insertion (enigo)
- `src/` — Frontend: vanilla HTML/CSS/JS with Tauri IPC
- Models download on first launch, stored in `~/.local/share/voxxa/models/`

## Prerequisites

- Rust 1.77+
- Node.js 18+
- System dependencies (Ubuntu/Debian):
  ```bash
  sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libasound2-dev
  ```

## Development

```bash
# Install frontend deps
npm install

# Run in dev mode (hot-reload frontend + Rust backend)
npx tauri dev

# Build for production
npx tauri build
```

## Legacy

The original Python slide-advancer code is preserved in `voxxa/` and `main.py`.
