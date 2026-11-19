# ADR-017: Simple mode in front of the existing panels, and a first-run flow

**Status:** Accepted 2026-11-19 (the user's binding Q8 decisions of 2026-11-18: ADR number, one 600-line "Simple mode (src/simple/*)" budget row, two ways of GUI testing, the founder's usability run is not a merge gate, no backend command change, no panel deleted) · **Date:** 2026-11-19 · **Feature:** church-autoadvance · **Reversibility:** Easy (the Simple view is one directory and one `hidden` attribute; removing them restores the old window) · **Owner:** nategroks · **Author:** 10x-Team (Principal Architect + Staff Engineer), Q8 10x lane

Implements ADR-010 Decision 4 ("the one-screen spec becomes Simple mode") and the first-run part of `specs/2026-10-24-software-strategy.md` § "Simple mode — the whole UI for a volunteer". Supersedes nothing.

## Context
- The operator window opens on the Slides tab: a setlist loader with four import paths, slide previews, a mic meter, confidence bars, five buttons, and Models and Settings tabs with 17 settings groups. A volunteer who runs slides twice a month has to find "Start Listening" among them.
- The strategy wants one window with three things: a mode switch (`Off` · `Songs` · `Songs + Scripture`), a status light (grey · amber · green · red, one plain sentence), and `Call for help`. Everything else moves behind "Advanced". First run: mic → find the presentation app → download the model → "play 20 seconds of any song" → green.
- **Forces.** No backend command may change and none may be added (user decision 4), so the light is built only from what the GUI can already read: the `get_status` and `get_presenter_info` commands, and the `machine-state`, `listening-stopped`, `presenter-error`, `mic-level`, `transcription`, `download-progress` and `slide-advanced` events. No panel is deleted (ADR-010 Decision 4). The GUI row is 2,951 / 3,000 and must not grow; Simple mode gets its own row capped at 600 (user decision 1). No npm runtime dependency. Operator window only: no lyrics on an audience screen (ADR-011).
- **Privacy.** The diagnostics report holds every transcript the GUI heard (Q7-08, #102, medium). A one-click "Call for help" that shows it would put that in front of every volunteer.

## Decision

### 1. Layout of Simple mode (the default view)
One column, 420 px wide (the main window's size), top to bottom:
1. **Mode switch**: one control, three positions: `Off`, `Songs`, `Songs + Scripture`. The third is visible and disabled, with the tooltip "Scripture comes in a later update" (Increment 4′).
2. **Status light**: a coloured dot and one plain sentence (table below). Not a control.
3. **Now showing**: the current song and slide, "Song 2 of 3 · <title>" and "Slide 2 of 6", counted inside the song. Titles only, never slide text (ADR-011). "No song on screen yet" until the conductor picks one.
4. **Call for help**: a button.
5. **Advanced**: a link.

So a volunteer sees three controls (mode switch, Call for help, Advanced) and one indicator.

**Off and Songs** go through the existing commands: they click the existing listen button, so `start_listening` and `stop_listening` run through `main.js`'s own `toggleListening` with its error toasts, and the old panel stays in step. The switch shows `Songs` when `get_status.is_running` is true and `Off` otherwise (polled every second), so the tray, the Space key and the Advanced button keep it right.

**Call for help (Q8):** shows "Call for help comes in the next update. Until then, call your tech lead and read them the sentence next to the light." It calls no command. `generate_diagnostic_report` fits technically, but it would show every heard transcript to any volunteer (Q7-08). Q9 builds the help bundle after Q7-08 is fixed.

### 2. Status light: the mapping (pure function `statusLight` in `src/simple/logic.js`, tested row by row)
Inputs, all from existing signals:
- `running`: `get_status.is_running`.
- `state`, `isBlank`: the latest `machine-state` event (`listening` · `singing` · `inter_verse_silence` · `blank_hold`, plus `is_blank`); before any event, `get_status.machine_state` and `is_blank`.
- `setlist`: `get_status.total_slides > 0`. `model`: `get_status.model_loaded`.
- `presenter`: `get_presenter_info` (`connected`, `display_name`), polled every second.
- `presenterError`: set by a `presenter-error` event; cleared by `slide-advanced` (a goto that reached the presenter) and by starting again.
- `micLost`: set by `listening-stopped` with `disconnected: true`; cleared by starting again.
- `song`: the in-song position from `get_status.current_slide` and the loaded setlist.

First matching row wins:

| # | Condition | Light | Sentence |
|---|---|---|---|
| 1 | `micLost` | red | "Can't hear the mic. Check the cable, then choose Songs again." |
| 2 | not `running`, no setlist | grey | "Off. Load this week's songs in Advanced first." |
| 3 | not `running`, no model | grey | "Off. Download the voice model in Advanced first." |
| 4 | not `running` | grey | "Off. Choose Songs to start following." |
| 5 | `presenterError` | red | "<app> didn't take the last slide change. Check that it is open." |
| 6 | presenter not connected | red | "<app> not found. Voxxa can't change slides." |
| 7 | `state` = `blank_hold` | amber | "Hearing talking, not a song. The screen stays blank." |
| 8 | `state` = `singing` or `inter_verse_silence`, `isBlank` | amber | "The screen is blank. It comes back when the song does." |
| 9 | `state` = `singing` or `inter_verse_silence` | green | "Following <title>, slide <n> of <m>." ("Following the song." without a position) |
| 10 | otherwise (`listening`, or no state yet) | amber | "Listening. Waiting for a song from the list." |

`<app>` is the presenter's display name without its bracketed detail ("ProPresenter 7.9+"). The keystroke driver always reports `connected`, so rows 5-6 come only from network presenters and from refused sends.

**Not mapped (recorded):** the strategy's red "Lyrics don't match what I hear". No existing signal tells a wrong setlist from a prayer or a sermon: both are `blank_hold`, which is the conductor's designed behaviour, so row 7 is amber. Ticket for a time-based rule once the pilots show what "lost" looks like.

### 3. The Advanced move
- `index.html`'s existing `#app` (header, tabs, every panel) gets `hidden`; Simple mode is built by `src/simple/simple.js` in front of it. The "Advanced" link hides Simple and shows `#app`; a "Simple mode" link and a "Set-up guide" link, added to the existing header by `simple.js`, go back and re-open the first-run flow.
- Every existing panel is unchanged and one click away: Slides (setlist loader, Hymnary, Planning Center, recent setlists, previews, confidence bars, mic meter, Listen/Prev/Next/Blank/Stage/?), Models, Settings (presenter connection, auto-detect, device, model, language, sliders, smart blanking, privacy counter, HTTP API, diagnostics).
- The diagnostic modal moves out of `#app` (same lines) so it still opens while `#app` is hidden.
- `main.js` gains one line: `loadSetlist` dispatches a `voxxa-setlist` DOM event with the songs, so Simple mode can count slides inside a song. The welcome overlay (markup, CSS, JS) is deleted because the first-run flow replaces it; that frees about 180 GUI lines.
- The app always opens in Simple mode; it does not remember Advanced.

### 4. First-run flow (replaces the welcome overlay)
An overlay in `src/simple/firstrun.js`. States:

| State | What it does (existing commands only) | Next |
|---|---|---|
| `device` | `list_audio_devices`; the saved device (`voxxa.audioDevice`) or "Default" pre-selected; Next calls `select_audio_device` (stores the name; opens nothing) and saves the key `main.js` uses | `presenter` |
| `presenter` | runs `discover_presenters` (2.5 s) on entry; one button per app found; picking one calls `connect_presenter` and saves `voxxa.presenterForm` like Settings. None found: says so and that Voxxa will press the arrow keys, so the presentation app must stay in front (the default keystroke driver; the flow never connects it itself). "Search again" | `model` |
| `model` | `get_model_status`; a loaded model: "ready"; downloaded but not loaded: "Use it" (`load_model`); none: "Download Tiny (75MB)" (`download_model`, progress from `download-progress`) and saves `voxxa.lastModel` | `test` |
| `test` | needs a setlist and a model: "Choose song files" opens the existing file picker. "Start the 20-second test" starts listening through the listen button, counts `mic-level` and `transcription` events for 20 s, stops, and judges with the pure `testVerdict` | `done` on pass; "Try again" or "Finish anyway" on fail |
| `done` | sets `voxxa.firstLaunchDone` and closes | |

- **Skippable:** "Skip setup" on every step closes it and sets the same key. **Re-openable:** Advanced → "Set-up guide".
- **Remembered** in `localStorage` under the welcome overlay's own key, `voxxa.firstLaunchDone`, so an operator who dismissed the welcome is not asked again.
- **Tiny** is the default model because it is the one the eval measures (ADR-015, ADR-016). The app still checks no hash (#9): the GUI cannot hash a file the backend wrote without a new command.
- **The test verdict** (`testVerdict`): listening never started → fail; mic lost → fail; loudest `rms` < 0.01 → "I couldn't hear anything"; no words → "I hear sound but no words"; otherwise pass. Only counts are kept; no heard text is shown. The test is a real 20-second listening session, so a song from the list can move the presentation app; the step says so.

### 5. Budget (recorded deviation from ADR-010 Decision 3)
`scripts/loc_budget.py` gets the row **"Simple mode (src/simple/*)" capped at 600**, counting every JavaScript, HTML and CSS file under `src/simple/` and subtracting it from the GUI row, which stays at 3,000. This is a deviation from ADR-010 Decision 3, which set one GUI budget, made by the user's decision of 2026-11-18 instead of raising the GUI row. The GUI row must not grow in Q8; it shrinks because the welcome overlay goes. No other cap changes. `package.json` keeps no runtime dependency.

### 6. Testing
1. **Pure logic in CI:** `tests/gui/simple_logic.test.mjs` (Node's built-in `node:test`, no dependency) covers every row of the status table, row order, `testVerdict` and the in-song slide count. `ci.yml` runs it after the frontend build on all three OSes.
2. **Browser with a mocked backend (local):** `tests/gui/e2e.mjs` serves the built `dist/` with `tests/gui/mock-tauri.js` injected before the app's scripts (it is never in the bundle), drives headless Edge or Chrome over the DevTools protocol, scripts every command and event, checks the default view's controls, each Advanced panel one click away, the first-run flow including the 20-second test, and the status rows, and saves screenshots. Listen, the 20-second test and every audio path run only here.
3. **The real app, guarded (manual, never committed):** a scratch build with identifier `com.voxxa.qa10x` and WebView2 remote debugging through `additionalBrowserArgs`, driven over CDP. Never Listen, never the 20-second test, no keystroke presenter, no stage window, no HTTP API.
4. **The founder's usability run** of the research plan's script is a founder checklist item and a Q8 ticket, not a merge gate.

## Alternatives
| Alternative | Why not |
|---|---|
| A second page (`simple.html`) or a tab | A second page duplicates `main.js`'s state; a tab is still a panel among panels, not the default. |
| Rewrite `main.js` around Simple mode | Grows the GUI row and risks every Q2 fix in the panels; Decision 4 says the panels stay. |
| Add a `get_ui_status` command that returns the light | Forbidden (no backend change), and the mapping is GUI policy: pure JS is enough and testable. |
| "Advanced…" opens `voxxa.toml` in an editor (the strategy's text) | There is no `voxxa.toml` (ADR-010 Decision 2 deferred it); the panels are the settings today. |
| Call for help shows the diagnostic report now | Shows every transcript to every volunteer (Q7-08). |
| Map `blank_hold` to red ("Lyrics don't match") | A prayer or sermon would turn the light red every Sunday. |
| Playwright for the browser test | Not available offline; Node 22+ has `fetch` and `WebSocket`, which is all CDP needs. |

## Consequences
Good: a volunteer sees three controls and one sentence; every old panel is one click away and unchanged; the light is a pure, tested function of existing signals; the GUI row shrinks. Bad: songs are still loaded in Advanced (the presenter's own playlist is not read yet); one more polling loop (two cheap commands a second); Call for help is a placeholder until Q9; the browser test runs locally, not in CI. Risks: `main.js`'s own listening flag can drift from the backend when the HTTP API starts listening (existing); the switch follows the backend through polling, but the old Listen button may need two clicks then.

## Review trigger
Re-open when Q9 wires Call for help, when Increment 4′ enables `Songs + Scripture`, when the founder's usability run fails a task, or if the pilots show the amber `blank_hold` hides a real failure.
