# Contributing to Voxxa

Voxxa is GPL-3.0-or-later. Every commit must carry a `Signed-off-by: Your Name <you@example.com>` line (`git commit -s`), which certifies the Developer Certificate of Origin in `DCO`. Keep changes small and complete: the smallest diff that fully solves the issue, with one test for any new branch, loop or parser. Read `.10x/handoff.md` and the relevant ADR in `.10x/adrs/` before starting; architectural changes need a new ADR, not an edit to an accepted one.

## Code layout

- `src-tauri/`: Rust backend: audio capture (cpal), VAD, Whisper (whisper-rs), the conductor (`aligner/`), presenter drivers, setlist importers.
- `src/`, `index.html`, `stage.html`: the frontend, vanilla HTML/CSS/JS with Tauri IPC and no runtime npm dependencies.
- Speech models download on first launch into the user data directory (for example `~/.local/share/voxxa/models/` on Linux).

### Presenter drivers

Slide control is abstracted behind the `PresentationController` trait in
`src-tauri/src/presenters/`. Each concrete driver implements the same
async surface (`next_slide`, `prev_slide`, `blank`, `goto_slide`,
`current_state`) and declares what it supports through `Capabilities`.
The drivers are:

- `keystroke`: universal arrow-key sender, with per-app key profiles
  (ProPresenter, EasyWorship, OpenLP, MediaShout, SongShow Plus,
  PowerPoint, Keynote, VideoPsalm, WorshipTools Presenter, Proclaim,
  FreeShow). Works against any app that has focus.
- `propresenter7_rest`: official HTTP REST API for ProPresenter 7.9+
  (default port 1025, configurable in ProPresenter → Settings → Network).
- `propresenter7_ws`: WebSocket remote for ProPresenter 7.0–7.8, and ProPresenter 6.
- `freeshow`, `openlp_v2`, `opensong`: the network APIs of those apps.

Pick and configure the active driver in **Settings → Presentation App**.
Integration details for each app are in `docs/INTEGRATIONS.md`.
