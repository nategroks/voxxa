# Contributing to Voxxa

Voxxa is GPL-3.0-or-later. Every commit must carry a `Signed-off-by: Your Name <you@example.com>` line (`git commit -s`), which certifies the Developer Certificate of Origin in `DCO`. Keep changes small and complete: the smallest diff that fully solves the issue, with one test for any new branch, loop or parser. Read `.10x/handoff.md` and the relevant ADR in `.10x/adrs/` before starting; architectural changes need a new ADR, not an edit to an accepted one.

## Presenter drivers

Slide control is abstracted behind the `PresentationController` trait in
`src-tauri/src/presenters/`. Each concrete driver implements the same
async surface (`next_slide`, `prev_slide`, `blank`, `goto_slide`,
`current_state`). Drivers:

- `keystroke` — universal arrow-key sender, with per-app key profiles
  (ProPresenter, EasyWorship, OpenLP, MediaShout, SongShow Plus,
  PowerPoint, Keynote, VideoPsalm, WorshipTools Presenter, Proclaim,
  FreeShow). Works against any app that has focus.
- `propresenter7_rest` — official HTTP REST API for ProPresenter 7.9+
  (default port 1025, configurable in ProPresenter → Settings → Network).
- `propresenter7_ws`, `freeshow`, `openlp_v2`, `opensong` — see each module's doc comment.

Pick and configure the active driver in **Settings → Presentation App**.
