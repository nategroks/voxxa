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

When the **Bearer token** field in Settings is non-empty, every request
needs `Authorization: Bearer <token>`. When empty, no auth is required.

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

All write endpoints return `204 No Content` on success, `401` on bad
token, or `500` with a JSON `{"error": "..."}` body on failure.

## Stream Deck (Elgato)

Two paths depending on how deep you want to go.

### Path A — built-in "Website" action

The simplest. No plugin needed.

1. Drag the **System → Website** action onto a key.
2. **URL**: `http://127.0.0.1:7575/api/v1/next` (or whichever endpoint
   you want on that key).
3. **Access in Background**: ON.
4. **HTTP method**: switch to **POST** in the action's properties.
5. **Headers**: if you set a bearer token, add
   `Authorization: Bearer <your-token>`.

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
TOKEN = ""  # set if you configured one
HEADERS = {"Authorization": f"Bearer {TOKEN}"} if TOKEN else {}

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
TOKEN = ""
HEADERS = {"Authorization": f"Bearer {TOKEN}"} if TOKEN else {}

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
# Status snapshot
curl -s http://127.0.0.1:7575/api/v1/state | jq .

# Next slide
curl -X POST http://127.0.0.1:7575/api/v1/next

# Blank (with token)
curl -X POST -H "Authorization: Bearer mytoken" \
  http://127.0.0.1:7575/api/v1/blank
```

## Security notes

- The HTTP server binds **only** to `127.0.0.1`. There is no way to
  expose it on the LAN through Voxxa's UI. If you want remote control,
  put Voxxa behind something like Tailscale Funnel and tunnel from
  there — never expose `0.0.0.0` directly.
- The Bearer token is in-memory only and not persisted across launches.
  Re-set it after restart if your integration needs it. (Persisting
  the token to disk is on the roadmap for a future OS-keychain
  integration.)
- The `/api/v1/listen/start` endpoint can be a service-stopping action
  in unfriendly hands; if you're piping commands through a shared
  Stream Deck profile or remote bridge, set a token even on 127.0.0.1.
