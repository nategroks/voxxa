# Release notes: `voxxad` and `voxxa-eval`, developer preview (Increment 1′)

2026-11-19. Not tagged and not published as a GitHub release. **The two
tools are not in the Voxxa installers**: build them from source (below).
They are for developers, testers and integrators, not for a Sunday
operator yet.

## What they are

- **`voxxad`** is Voxxa without a window. It runs the app's own listening
  loop and conductor (ADR-013) on a setlist and a WAV file or an input
  device, prints one JSON object per line on stdout (schema v1: `status`,
  `hypothesis`, `action`, `error`), and writes a session log that
  `--replay` re-runs without a model or audio.
- **`voxxa-eval`** scores `voxxad` against what an operator did, on the
  fixtures in `fixtures/recordings` (ADR-015; `fixtures/README.md`).

What Increment 1′ added: the CLI and its replay (Q3), Whisper behind a
5 s window decoded every 1 s (Q4), the eval and its CI gate (Q5), and the
position tracker of ADR-016 (Q6). On the six fixtures: recall 1.00, 0 false
advances, p95 latency 4.87 s (the budget is 2.5 s: not met). These are
synthetic spoken fixtures, not singing, and the latency is stream time at
file speed, not wall clock.

## Build from source

You need Rust 1.90 or newer (CI uses 1.99.1), Node 22 with npm (the library
embeds the app's built front end), CMake and a C/C++ compiler (whisper.cpp
is compiled), and on Linux the packages listed in `README.md`.

```sh
git clone https://github.com/nategroks/voxxa && cd voxxa
npm ci && npm run build
cargo build --manifest-path src-tauri/Cargo.toml --features cli --bin voxxad --bin voxxa-eval
```

The binaries are in `src-tauri/target/debug/`. Without `--features cli`
neither tool is built; that is what keeps them out of `tauri build`. All
numbers above come from debug builds; on Windows a release build compiles
whisper.cpp without optimisation (Q4-01, #59).

`voxxad` needs a Whisper model except for `--replay`: download one in the
Voxxa app, or place `ggml-tiny.bin` (SHA-256 `be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21`)
in `%APPDATA%\voxxa\models` (Windows), `~/Library/Application Support/voxxa/models`
(macOS) or `~/.local/share/voxxa/models` (Linux).

## First runs, in this order

1. Replay, no model and no audio:
   `voxxad --setlist fixtures/recordings/refrain-it-is-well/setlist.json --replay fixtures/recordings/refrain-it-is-well/session.jsonl`
2. A recording, in shadow mode (computes and logs actions, sends nothing):
   `voxxad --setlist setlist.json --stdin-audio service.wav --shadow`.
   The WAV must be 16 kHz mono, 16-bit PCM or 32-bit float. Convert others
   with `ffmpeg -i in.wav -ar 16000 -ac 1 -c:a pcm_s16le out.wav`.
3. The booth microphone, still in shadow mode: `voxxad --list-devices`,
   then `voxxad --setlist setlist.json --device default --shadow`. Run a
   whole service like this and read the log before going further.
4. Only then a presenter. Prefer a network driver
   (`--presenter propresenter7_rest|freeshow|openlp_v2|opensong --host … --port …`).

Stop with Ctrl+C; `voxxad` then writes a `stop` line. Exit status 0, or 1
after one `error` line. `voxxa-eval`: 0 no metric worse than
`fixtures/baseline.json`, 1 a metric worse, 2 an error.

## Warnings

- **Keystroke:** `--presenter keystroke` types arrow keys into whichever
  window has focus. **Focus the presentation app.** If you start `voxxad`
  from a terminal, the terminal has focus and gets the keys. `voxxad` prints
  this warning but has no focus guard (Q3-04, #48).
- **One conductor at a time:** do not run `voxxad` with a presenter while
  the Voxxa app is listening; both would change slides.
- **Not signed:** the binaries you build are unsigned; Windows SmartScreen
  and macOS Gatekeeper may warn.

## Privacy

- **Session logs hold text of everything the microphone heard**: songs,
  prayers, announcements, names said aloud (the `hypothesis` and `window`
  lines; no audio). Every run except `--replay` writes one to
  `<data dir>/voxxa/sessions/`. Logs older than 14 days are deleted when
  `voxxad` next starts, not before.
- On Windows `<data dir>` is the **roaming** `%APPDATA%`: on a church
  domain with roaming profiles, the logs are copied to the server. Use
  `--log-dir` with a local folder (for example `%LOCALAPPDATA%\voxxa\sessions`)
  and delete logs you do not need (Q3-05, #49).
- Network: `--replay` opens no socket (a Linux CI test checks this);
  `--shadow` builds no presenter (tested). A presenter driver connects only
  to the host you name, and only without `--shadow`. `voxxad` never
  downloads anything and sends no telemetry.

## Known limits

- p95 latency 4.87 s at file speed; live runs are slower (Q4-02 #60,
  Q6-03 #84). Lyrics can stay up for 5-7 s into a prayer (Q4-03 #61).
- The fixtures are synthetic speech (Q5-01 #73). `--device` has not yet
  run on a real booth PC (Q3-07 #51). No `--language` (Q3-10 #54).
- Driving `voxxad` from a Stream Deck: `docs/INTEGRATIONS.md`, "Driving
  voxxad from Companion". When the eval goes red: `docs/runbooks/eval-regression.md`.
