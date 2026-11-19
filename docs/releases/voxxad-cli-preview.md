# Release notes: `voxxad` and `voxxa-eval`, developer preview (Increment 1′)

**Date:** 2026-11-19 (Q7, closing Increment 1′: Q3-Q6). **Version:** the crate version, 0.1.0.
**How it is distributed:** source only. There is **no installer, no tag and no GitHub release**
for these tools. The church installers (`tauri build`) carry the Voxxa app only. The two tools
need the off-by-default `cli` Cargo feature, so `tauri build` never builds them.

## What they are

- **`voxxad`** is Voxxa without a window (ADR-013). It runs the GUI's listening loop and
  conductor on a WAV file, on an audio input device, or on a recorded session log (`--replay`).
  It prints one JSON object per line (event schema v1: `status`, `hypothesis`, `action`,
  `error`) and writes everything, recogniser windows included, to a session log.
- **`voxxa-eval`** scores `voxxad` against an operator's ground truth (ADR-015). It covers
  precision, recall, median and p95 latency, false and double advances, and blank and unblank
  timing. It checks the scores against `fixtures/baseline.json`.

## What changed in Increment 1′

- **Q3 (ADR-013):** the headless `voxxad` CLI. The listening loop moved into the library
  (`voxxa_lib::listen::run`), so the app and `voxxad` share it. `--replay` is
  byte-identical, and `--shadow` computes and logs actions without sending any.
- **Q4 (ADR-014):** the recogniser sits behind a trait, as a 5 s window decoded every 1 s
  with a stitcher. The session log gains `window` lines. `--bias` is opt-in.
- **Q5 (ADR-015):** `voxxa-eval`, the fixture format (`fixtures/README.md`) and a CI gate
  that replays the committed logs on Linux, macOS and Windows.
- **Q6 (ADR-016):** a position tracker for repeats and jumps. It uses optional
  `sections[].kind` in the setlist, repeat-aware priors, word-token scoring and an evidence
  accumulator. On the six synthetic fixtures (replay): 21 of 21 slides, 0 false advances,
  median 2.31 s, p95 4.87 s. The 2.5 s p95 budget is not met yet (#84).
- **Q7:** the tools are left out of the installers (see below). A determinism test replays
  every committed log twice on each OS. A security test proves `--replay` opens no socket
  (Linux and Windows). A report-only CI workflow runs the full eval with a checksum-pinned
  Whisper Tiny on all three OSes.

## Build from source

Prerequisites: Rust (CI uses 1.99.1; the minimum is 1.90), Node 22, CMake and a C/C++
toolchain for whisper.cpp, and LLVM/libclang for its bindings. On Windows use LLVM 22 or
older: LLVM 23.1.4 produced empty bindings in a local test (QA record, 2026-10-24). On
Ubuntu/Debian, also install the packages in the README's "Development" section.

From the repository root:

```sh
npm ci && npm run build                     # the library embeds the GUI's dist/
cargo build --manifest-path src-tauri/Cargo.toml --features cli --bin voxxad --bin voxxa-eval
```

The binaries are in `src-tauri/target/debug/` (`voxxad.exe` and `voxxa-eval.exe` on
Windows). Add `--release` for an optimised build, but **not on Windows yet**. There a release
build compiles whisper.cpp without optimisation and decodes much more slowly than a debug
build (Q4-01, #59).

`voxxa-eval` runs the `voxxad` that sits in the same folder, so build both together.

## Speech model

`voxxad` never downloads anything. Install a model in the Voxxa app (Models tab), or put the
file in `<data dir>/voxxa/models/`:

| OS | Folder |
|---|---|
| Windows | `%APPDATA%\voxxa\models\` |
| macOS | `~/Library/Application Support/voxxa/models/` |
| Linux | `~/.local/share/voxxa/models/` (or `$XDG_DATA_HOME/voxxa/models/`) |

Whisper Tiny comes from
`https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin` (77,691,713 bytes).
Check its SHA-256 before use:
`be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21`. `--model tiny|base|…`
picks one; by default `voxxad` uses the smallest model it finds.

## Running it safely

```sh
voxxad --help
voxxad --list-devices
voxxad --setlist today.json --stdin-audio service.wav --shadow                       # a file, nothing sent
voxxad --setlist today.json --device default --shadow --log-dir <a folder of its own>  # live, nothing sent
voxxad --setlist today.json --replay <session.jsonl>                                  # re-run a log
```

- **Start with `--shadow`.** It logs every action it would take and sends none. The default
  presenter is `none`, which also sends nothing.
- **Keystroke presenter: focus the presentation app.** `--presenter keystroke` types
  Next/Previous into whatever window has focus, including the terminal that started
  `voxxad`, where a keystroke can run a command. `voxxad` has no window, so it cannot guard
  focus (Q3-04, #48). Prefer a network presenter (`propresenter7_rest`, `freeshow`,
  `openlp_v2`, `opensong`, with `--host` and `--port`).
- Do not drive the same presentation software from `voxxad` and the Voxxa app at once.
- **Audio files** must be 16 kHz mono WAV (16-bit PCM or 32-bit float). `voxxad` does not
  resample (Q3-10, #54). Convert other files first:
  `ffmpeg -i in.wav -ar 16000 -ac 1 -c:a pcm_s16le out.wav`.
- Stop a live run with Ctrl+C. It writes a final `stop` line.
- To drive `voxxad` from Bitfocus Companion, see `docs/INTEGRATIONS.md`, "Driving `voxxad`
  from Bitfocus Companion".

## Privacy

- **Session logs** hold every transcript, prayers and spoken names included. They are kept
  for 14 days in `<data dir>/voxxa/sessions/`. On Windows that is the **roaming**
  `%APPDATA%`, which a domain-joined profile may copy to a server (Q3-05, #49). Voxxa never
  uploads them. To keep them on the computer, use `--log-dir`, for example
  `--log-dir "%LOCALAPPDATA%\voxxa\sessions"`. Give it a folder of its own: `voxxad` deletes
  `session-*.jsonl` files older than 14 days there. Commit only logs of public-domain or
  synthetic audio.
- **Network:** `--replay` and `--shadow` open no socket. CI checks `--replay` on Linux
  (strace, networking disabled) and Windows (ETW). A local Windows check covered a full
  `--shadow` run with Whisper. Without `--shadow`, a network presenter talks to the host and
  port you give it, and that is the only network use. There is no telemetry.

## Evaluating

```sh
cargo run --manifest-path src-tauri/Cargo.toml --features cli --bin voxxa-eval                        # replay, as CI
cargo run --manifest-path src-tauri/Cargo.toml --features cli --bin voxxa-eval -- --full 2>/dev/null  # Whisper Tiny
```

Whisper prints the words it hears to stderr, so `2>/dev/null` keeps them out of your
terminal. Exit codes: 0 means no metric is worse than the baseline, 1 means one is, 2 means
an error. When the eval goes red, see `docs/runbooks/eval-regression.md`.

## Known limits

- All six fixtures are synthetic speech, not singing (#73). The numbers catch regressions;
  they do not predict accuracy in a church.
- Latency is stream time at file speed. Live, a hop costs more than 1 s on the reference PC
  (#60), and p95 is above budget (#84).
- Lyrics stay up for the first 5-7 s of a prayer with no pause before it (#61).
- Small output gaps: a shadow run's start line shows `"presenter": null`, and early errors go
  to stdout only (#55).
- Neither tool is signed, and neither runs as a service. `voxxad` has no remote control:
  start and stop it as a process.
