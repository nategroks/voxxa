# Presentation-app test card (one per app and version)

A person runs this at the booth PC with the presentation app open and **no service running**. voxxa is not
listening. Agents never press keys in the presentation app; they only read the answers.

App: ______________  Version: ______________  Date: __________  Tester: ______________

| # | Test | How | Answer |
|---|---|---|---|
| T1 | voxxa finds the app | voxxa first-run step 2, or Advanced → presentation app | found / not found |
| T2 | Right on a song's last slide | put a song live on its last slide, press Right | next song's blank / next song's first slide / nothing / other: ____ |
| T3 | Blank key | which key clears only the lyrics (not the background or media)? | key: ____ |
| T4 | Unblank key | after T3, which key brings the lyrics back? | key: ____ |
| T5 | Right when the next item is a graphic | put the last slide of a song live, next item a graphic, press Right | ____ |
| T6 | Next service item key | is there a key for "next item in the service list"? may voxxa use it? | key: ____  allowed: yes / no |
| T7 | Function keys used by the app | does the app react to F6, F7, F8, F9, F13–F19? | list: ____ |
| T8 | Confirm the `goto.keys` chord | press the binding profile's `goto.keys` chord by hand: WorshipTools Presenter, Right Right from a song's last slide when the next song is the next item in the list; the universal profile, Home on a song's later slide | WorshipTools: next song's first slide / other: ____  universal: the song's first slide / other: ____ |
| T9 | Window names | the main window's title, and the title of one edit dialog (so voxxa sends keys only to the main window) | main: ____  dialog: ____ |
| T10 | Administrator | is the app run as administrator? (it must not be: Windows silently drops voxxa's keys) | yes / no |
| T11 | Credit line | does the app have a footer or copyright field per song for the CC credit line? | yes (where: ____) / no → use a `Credit:` last slide |
| T12 | API (network apps only) | does the app's remote/API page show enabled, port, password? | port: ____ (never write the password here) |

The binding profiles cite these row numbers (for example "test card T4" for the unblank key and "test card T8"
for a `goto.keys` chord); a `keys` step is used only after its row confirms it on this app and version.

Keep this card in `<church_root>\` as `presenter-test-card-<app>-<date>.md`. voxxa's team needs only the answers,
never screenshots of song text.

---

## FreeShow rows (FS1–FS20)

Only for a church that runs FreeShow or is moving to it. A church that stays on a keys-only app skips this
section.

Rules for every FreeShow row:

- A person runs every step on a weekday with no service or rehearsal. Agents never install, start or click
  anything in FreeShow or Presenter; they only read the answers.
- The church's current app stays the live output. FreeShow has no output assigned, except on the **booth monitor**
  for FS8, FS11 and FS12: never on the projector or the stream.
- FS3–FS7 use only the test show built from the kit's public-domain hymn (a unique layout, every slide once); FS9
  uses only the kit's three public-domain hymn files. Never the church's own songs.
- The FreeShow API password is **never** written on this card or in any file, chat or paste. voxxa's FreeShow probe
  (`scripts/lab/freeshow_probe.py`, built in step V3e) reads it from the environment variable `VOXXA_FS_KEY`.
- Answers are numbers, yes / no or one of the listed choices. Never paste a reply, a show or layout id, a show
  name, slide words or a screenshot.

### When FreeShow is first installed (FS1–FS12)

| # | What to do | What to record |
|---|---|---|
| FS1 | Install FreeShow from its official release page, stable channel only, never a beta; check the checksum if the page publishes one; turn auto-update off (FA-51) | version: ____  release date: ____  auto-update off: yes / no |
| FS2 | Send one request without the password to port 5505 and one to port 5506; check that Windows Defender Firewall has inbound rules for TCP 5505 and 5506 that allow `127.0.0.1` only; send one request to this PC's own LAN address (F5) | 401 when no password is sent, 5505: yes / no, 5506: yes / no  loopback firewall rule exists: yes / no  LAN-address request refused: yes / no |
| FS3 | Probe: `index_select_slide` with its fields at the top level of the body (shape A), then nested under `data` (shape B) (W1) | shape A moves the slide: yes / no  shape B moves the slide: yes / no |
| FS4 | Probe: `index_select_slide` with `index: 1`, then `index: 0`, then `index: 2`, with `get_output` after each (C2) | `index: 1` shows the first slide: yes / no  `index: 0` shows nothing: yes / no  `slide.index` after `index: 2`: ____ (expected 1) |
| FS5 | Probe: ten moves, each followed by `get_output`; compare `slide.index` with the slide FreeShow shows; time each read-back | `slide.id`, `slide.layout`, `slide.index` in the reply: yes / no (if no: the key names the probe printed, names only)  matches the screen: __ of 10  verify latency median: ____ ms  p95: ____ ms |
| FS6 | Probe: from inside the chorus, `id_select_group` with `chorus`; then with `verse 1`; `get_output` after each (C3) | `chorus` lands on the next chorus: yes / no  `verse 1` moves nothing: yes / no |
| FS7 | Probe: repeat the chorus with **one** `index_select_slide` call naming the show and the layout; `get_output` after | chorus repeated by one call: yes / no  `get_output` ids confirm it: yes / no |
| FS8 | Presenter open; FreeShow outputting to the booth monitor with a video playing; `voxxad --shadow --model small` on a public-domain fixture. Then the same with FreeShow closed, as the baseline | with FreeShow: dedicated GPU memory ____ MiB  GPU ____ %  CPU ____ %  Whisper pass time median ____ s  p95 ____ s  baseline: ____ MiB  ____ %  ____ %  median ____ s  p95 ____ s |
| FS9 | Import the kit's three public-domain hymn `.txt` files into FreeShow (FA-56) | slides / groups / `presenter_slides` in `songs.csv`: song 1 __ / __ / __  song 2 __ / __ / __  song 3 __ / __ / __  group names match the labels `v1 c v2 …`: yes / no  each layout lists every slide once: yes / no |
| FS10 | Skipped: the founder dropped the Presenter MIDI mapping check (FA-53, answered 2026-11-27). Leave this row blank | not needed |
| FS11 | FreeShow outputting to the booth monitor for 10 minutes with a video playing while `voxxad --shadow` runs | visible stutter: yes / no  dropped hops (session-end line): ____ |
| FS12 | Open the output style the followed songs use; read its text transition. The 500 ms default stays (FA-57) | text transition: ____ ms (expected 500, a fade) |

### After voxxa's FreeShow driver ships (FS13–FS15, step V8b)

| # | What to do | What to record |
|---|---|---|
| FS13 | In FreeShow, Settings → Connections: check the API password is set; send one request without it | password set: yes / no  401 without it: yes / no |
| FS14 | In Windows Defender Firewall, check the inbound rules for TCP 5505 and 5506 | rules allow `127.0.0.1` only: yes / no |
| FS15 | For each song voxxa will follow in FreeShow: check the layout lists every slide once, and FreeShow's slide count against `presenter_slides` in `songs.csv` (FA-56) | songs checked: __  unique layout: __ of __  slide count equals `presenter_slides`: __ of __ |

### Before the first Sunday with FreeShow live (FS16–FS20, step V17)

| # | What to do | What to record |
|---|---|---|
| FS16 | Open FreeShow's stage display on a phone on the booth's network. voxxa's own strip is the worship leader's cue; voxxa writes no FreeShow stage text (FA-59, FA-60) | opens on a phone: yes / no |
| FS17 | Find how the projector and the stream get FreeShow's output; confirm the stage output is never in the stream (FA-29) | HDMI split / capture of a window / other  stage output never in the stream: yes / no |
| FS18 | Check the FreeShow version is pinned and auto-update is off (FA-51) | auto-update off: yes / no  version equals `freeshow_version` in `church-profile.json`: yes / no |
| FS19 | Check that the followed output style's text transition is still FreeShow's 500 ms default (FA-57) | 500 ms default: yes / no |
| FS20 | Find out, from FreeShow's documentation, whether its Planning Center import carries song words or only the service order. Never connect the church's Planning Center account to find out; FreeShow's Planning Center connection stays off until this is answered and the song library is clean (ADR-026) | carries song words: yes / no |

The FreeShow binding profile (`bindings/freeshow.json`) cites these rows; each of its http steps runs only after
its rows and its contract test pass. Keep the FreeShow answers in `<church_root>\` as
`presenter-test-card-freeshow-<date>.md`.
