# Integrating hardware and automation with Voxxa

Voxxa exposes an opt-in HTTP API on `127.0.0.1` (see **Settings →
Local HTTP API**). With it on, anything that can fire an HTTP request
can drive Voxxa: Stream Decks, Bitfocus Companion, foot pedals,
QLab cues, shell scripts, MIDI-to-HTTP bridges, the works.

The API is intentionally tiny — six endpoints — so a 20-line
integration is realistic.

## Endpoint reference

Base URL: `http://127.0.0.1:<port>` (default `7575`, configurable).

| Method | Path | Purpose |
|---|---|---|
| GET | `/api/v1/state` | Returns the conductor snapshot |
| POST | `/api/v1/next` | Manual next slide |
| POST | `/api/v1/prev` | Manual previous slide |
| POST | `/api/v1/blank` | Blank audience output |
| POST | `/api/v1/listen/start` | Start auto-advance |
| POST | `/api/v1/listen/stop` | Stop auto-advance |

Every request needs `Authorization: Bearer <token>`. Type a token of at
least 16 characters in the **Bearer token** field in Settings, or leave it
blank and Voxxa generates one when you turn the API on and shows it there.
Requests from a web browser are refused (see Security notes).

### `GET /api/v1/state`

```json
{
  "is_running": true,
  "current_slide": 4,
  "total_slides": 27,
  "song_title": "Amazing Grace",
  "machine_state": "Singing",
  "is_blank": false
}
```

`machine_state` is one of `Listening`, `Singing`, `InterVerseSilence`,
or `BlankHold` (see §4.3 of the project plan).

### Write endpoints

All write endpoints return `204 No Content` on success, `401` on a missing
or wrong token, `403` for a browser request, or `500` with a JSON
`{"error": "..."}` body on failure.

## Stream Deck (Elgato)

Two paths depending on how deep you want to go.

### Path A — built-in "Website" action

The simplest. No plugin needed.

1. Drag the **System → Website** action onto a key.
2. **URL**: `http://127.0.0.1:7575/api/v1/next` (or whichever endpoint
   you want on that key).
3. **Access in Background**: ON.
4. **HTTP method**: switch to **POST** in the action's properties.
5. **Headers**: add `Authorization: Bearer <your-token>`.

Repeat for Prev, Blank, Start/Stop. The downside: no visual feedback —
the key just fires.

### Path B — Bitfocus Companion (stream-deck.com)

For visual feedback (current song title, slide counter, BLANK indicator)
use Companion as the middleware. Companion has a Generic HTTP module
that does GET + POST against arbitrary URLs and can store the JSON
response into variables.

Setup sketch:

```text
1. Add a Connection → Generic HTTP, set Base URL = http://127.0.0.1:7575
2. Create a feedback that polls /api/v1/state every 500 ms and stores:
     internal:voxxa_song    = data.song_title
     internal:voxxa_state   = data.machine_state
     internal:voxxa_blank   = data.is_blank
3. Add a Button → Action → HTTP POST to /api/v1/next
4. Add a Button → Feedback → "Background red when $(voxxa:voxxa_blank) = true"
```

Once that's wired, you get a real Voxxa control surface with live
status feedback on every key.

## Foot pedals

Most USB foot pedals show up to the OS as either a keyboard (sending
PgUp / PgDn / Space / arrows) or a HID device. Two options:

1. **Keyboard-emulating pedal**: just use Voxxa's built-in keyboard
   shortcuts (`←`, `→`, `B`, `S`, `Space`). Map the pedal to the keys
   you want via the pedal's own configurator. Voxxa must have main-
   window focus.
2. **HID pedal + bridge script**: more flexible because it doesn't
   require Voxxa's window focus. Write a tiny bridge that listens for
   pedal events and POSTs to the HTTP API.

Example bridge in Python with `hidapi`:

```python
import hid, requests

VOXXA = "http://127.0.0.1:7575"
TOKEN = "paste-the-token-from-voxxa-settings"
HEADERS = {"Authorization": f"Bearer {TOKEN}"}

# Replace with your pedal's vendor/product IDs (lsusb / Device Manager).
VENDOR_ID = 0x05F3
PRODUCT_ID = 0x00FF

dev = hid.device()
dev.open(VENDOR_ID, PRODUCT_ID)
print("Pedal connected.")
while True:
    report = dev.read(8, timeout_ms=1000)
    if not report:
        continue
    # Tailor this to your pedal's encoding — most are 1 byte: left, center, right.
    pressed = report[0]
    if pressed == 1:
        requests.post(f"{VOXXA}/api/v1/prev", headers=HEADERS)
    elif pressed == 2:
        requests.post(f"{VOXXA}/api/v1/blank", headers=HEADERS)
    elif pressed == 4:
        requests.post(f"{VOXXA}/api/v1/next", headers=HEADERS)
```

## QLab (Mac show-control)

QLab's **Network** cue can hit any HTTP endpoint. Add a Network cue with:

- **Type**: HTTP
- **Method**: POST
- **URL**: `http://127.0.0.1:7575/api/v1/blank`

Sequence Voxxa's blank into your QLab cue list at the right downbeat
and the audience-facing output stays clean across set changes without
manual operator intervention.

## MIDI

There's no direct MIDI surface in Voxxa, but a 20-line Python bridge
with `mido` does the job:

```python
import mido, requests

VOXXA = "http://127.0.0.1:7575"
TOKEN = "paste-the-token-from-voxxa-settings"
HEADERS = {"Authorization": f"Bearer {TOKEN}"}

# CC -> action map. Customise to your controller.
MAP = {
    20: "next",
    21: "prev",
    22: "blank",
    23: "listen/start",
    24: "listen/stop",
}

with mido.open_input() as port:
    print("Listening for MIDI...")
    for msg in port:
        if msg.type == "control_change" and msg.value > 0:
            action = MAP.get(msg.control)
            if action:
                requests.post(f"{VOXXA}/api/v1/{action}", headers=HEADERS)
```

## Shell automation

Manual cron-like triggers, or one-off cues from any terminal:

```bash
TOKEN=paste-the-token-from-voxxa-settings

# Status snapshot
curl -s -H "Authorization: Bearer $TOKEN" http://127.0.0.1:7575/api/v1/state | jq .

# Next slide
curl -X POST -H "Authorization: Bearer $TOKEN" http://127.0.0.1:7575/api/v1/next

# Blank
curl -X POST -H "Authorization: Bearer $TOKEN" http://127.0.0.1:7575/api/v1/blank
```

## Driving `voxxad` from Bitfocus Companion

`voxxad` is Voxxa without a window: the same listening loop and conductor,
started from a command line. It prints one JSON line per event and writes a
session log (ADR-013). It is a **developer preview**. It is not in the
installers; build it from source (`docs/releases/voxxad-cli-preview.md`).
For a booth that runs the Voxxa app, use the app's HTTP API with Companion
instead (Path B above).

`voxxad` has **no HTTP API and no remote control**. Companion can only start
it as a program and stop it by ending the process. Both need Companion to
run on the same computer as `voxxad`, and an action that runs a local
program or script. Some Companion versions have one (it may be called
"Run shell path (local)"); **check your Companion version** and its
modules. This guide names no other Companion feature.

### 1. Shadow first

Run every new setup in shadow mode for at least one rehearsal.
`--shadow` computes and logs every action and sends nothing to the
presentation software. Start script for Windows (`start-voxxad.cmd`):

```bat
@echo off
rem One voxxad at a time: a second one would drive the same screen.
tasklist /FI "IMAGENAME eq voxxad.exe" | find /I "voxxad.exe" >nul && exit /b 0
start "voxxad" /min "C:\Voxxa\voxxad.exe" --setlist "C:\Voxxa\today.json" ^
  --device default --shadow --log-dir "%LOCALAPPDATA%\voxxa\sessions"
```

On macOS or Linux (`start-voxxad.sh`):

```sh
#!/bin/sh
pgrep -x voxxad >/dev/null && exit 0
nohup /opt/voxxa/voxxad --setlist "$HOME/voxxa/today.json" \
  --device default --shadow >/dev/null 2>&1 &
```

- `--device` takes a name from `voxxad --list-devices`, or `default`. Use
  the desk feed or a USB audio interface, not the laptop microphone.
- The setlist is the `voxxad --setlist` JSON format (see
  `fixtures/recordings/*/setlist.json`).
- After the rehearsal, open the newest `session-*.jsonl` and compare its
  `action` lines with what the operator did.

### 2. Stop

- **macOS and Linux:** `pkill -INT -x voxxad`. `voxxad` stops as on Ctrl+C
  and writes a final `stop` line (`"reason":"stopped"`).
- **Windows:** `taskkill /IM voxxad.exe /F`. Windows has no command that
  sends Ctrl+C to another console program, so this ends the process at
  once. The session log keeps every line written so far (each line is
  written when it happens); only the final `stop` line is missing.

Put the start script on one Companion button and the stop command on
another. Do not run `voxxad` and the Voxxa app on the same presentation
software at the same time.

### 3. Driving the presentation software

Replace `--shadow` with a presenter only after a clean shadow rehearsal:

- **Prefer a network presenter:** `--presenter propresenter7_rest`,
  `freeshow`, `openlp_v2` or `opensong`, with `--host` and `--port`.
  These talk to the presentation software over its own API, whatever
  window has focus.
- **Keystroke (`--presenter keystroke`): focus the presentation app.**
  `voxxad` has no window, so it cannot check focus (ticket Q3-04, #48). It
  types Next/Previous into **whatever window has focus**: a terminal (where
  a keystroke can run a command), a document, or Companion's own window.
  Use it only when the presentation app stays focused for the whole
  service, and never start it from a terminal you keep using.
  `voxxad` prints a warning to stderr when it starts this way.

### Privacy and files

- **Session logs** hold every transcript, prayers and spoken names
  included, for 14 days. They default to `<data dir>/voxxa/sessions`. On
  Windows that is the roaming `%APPDATA%`, which a domain profile may copy
  to a server. `--log-dir "%LOCALAPPDATA%\voxxa\sessions"` keeps them on
  the computer (Q3-05, #49). Give `--log-dir` a folder of its own:
  `voxxad` deletes `session-*.jsonl` files older than 14 days there.
- **Network:** `--replay` opens no socket; `--shadow` makes no network connection (on Linux/macOS tokio's signal driver creates one local AF_UNIX socketpair). A network
  presenter is the only network traffic `voxxad` makes, and only without
  `--shadow`. `voxxad` never downloads a model: install one in the Voxxa
  app first, or see the release notes.
- **Audio files** (`--stdin-audio`) must be 16 kHz mono WAV. Convert
  others with `ffmpeg -i in.wav -ar 16000 -ac 1 -c:a pcm_s16le out.wav`
  (Q3-10, #54).

## WorshipTools Presenter: Tier 3 (keys); MIDI deferred

Voxxa drives presentation apps in tiers (ADR-006). ProPresenter 7.9+, FreeShow,
OpenLP and OpenSong are driven over their own local network APIs. Apps without
one are **Tier 3**: Voxxa presses their keys. WorshipTools Presenter is Tier 3.
It has no local API: its Remote and Stage Display apps go through WorshipTools'
cloud. ADR-019 records how Voxxa drives it:

- **Found by its process.** On Windows, the set-up guide finds a running
  `Presenter.exe` and connects the keystroke driver with the **WorshipTools
  Presenter** key profile. **Advanced > Settings > Auto-detect** lists it too:
  click it, then press **Connect**. There is no host or port. The profile
  is saved and reconnected at every start.
- **Keys only when Presenter is in front.** With this profile a key is sent only
  while the front window belongs to Presenter's process. Otherwise nothing is
  sent and the status light is amber: "Bring WorshipTools Presenter to the
  front." The other key profiles have no such check.
- **Never a Right past a song's last slide.** In Presenter that Right moves to
  the next service item. On a song change Voxxa presses nothing and the light
  asks the operator: "Put "`<song>`" live in Presenter." Inside a song, and
  for the operator's own Next and Previous, keys work as before.
- **Keys:** Right and Left only. No blank key yet: F10 clears the lyrics, but
  how to bring them back is unconfirmed, so the profile reports `can_blank:
  false`. Voxxa skips its own blanks and refuses the Blank button. It never sends
  F11, F12, Up, Down or the number pad's + and −.
- **Run Presenter normally, not as administrator.** Windows drops keys that an
  ordinary program sends to a program running as administrator, and Voxxa
  cannot tell.
- **MIDI is deferred** (ADR-019 § 5). Presenter accepts MIDI In, but only with
  loopMIDI or rtpMIDI installed. Voxxa sends no MIDI to presentation apps yet.
  MIDI is the named next step if keys prove unreliable at the pilot. (The
  [MIDI](#midi) section above is the other direction: a MIDI controller driving
  Voxxa through the HTTP API.)
- **`voxxad --presenter keystroke`** always uses the universal key profile, with
  no WorshipTools guard and no song rule. Don't use it with Presenter.

The operator steps are in the runbook
[Voxxa can't reach the presentation app](runbooks/presentation-app-not-reachable.md#worshiptools-presenter).

## Security notes

- The HTTP server binds **only** to `127.0.0.1`, and Voxxa has no setting
  to expose it on the LAN or the internet. Do not tunnel or port-forward
  it: anyone who reaches it can blank or move the audience screen and turn
  the microphone on.
- Every request needs the bearer token. Voxxa generates a random one when
  the field is blank. The token lives in memory only and is not saved, so
  after a restart turn the API on again and update your integration.
- Requests from web browsers are refused with `403`: any request with an
  `Origin` header, and any request whose `Host` is not `127.0.0.1:<port>`
  or `localhost:<port>`. A web page open on the booth machine therefore
  cannot drive Voxxa (cross-site requests, DNS rebinding). Stream Deck,
  Companion, scripts and `curl` send no `Origin` and are unaffected.
