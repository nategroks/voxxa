# ADR-001: Split Voxxa into a headless core (`voxxad`) and a thin optional GUI

**Status:** Proposed
**Date:** 2026-10-24
**Feature:** church-autoadvance
**Reversibility:** Costly (process boundary and stdio protocol become public surface)
**Owner:** nategroks
**Author:** 10x-Team (Architect + Staff Engineer)

## Context
Today everything runs inside one Tauri process: audio, VAD, Whisper, alignment and key injection live behind Tauri IPC commands, and the only way to exercise the pipeline is to click the GUI. The business model is "free binary, paid support": support cost is driven by how fast a volunteer's problem can be reproduced and fixed. The product brief asks for the Unix philosophy and a suckless footprint, and for integration with Bitfocus Companion / Stream Deck style tooling that churches already use.

## Decision
- `voxxad` is a headless Rust binary: it reads audio (device or `--stdin-audio` WAV), runs VAD → ASR → aligner → supervisor, and emits one JSON object per line on stdout (`advance`, `goto`, `show_ref`, `status`, `transcript`). It reads one config file, `voxxa.toml`.
- Adapters are small modules in the same binary selected by `[adapter]` in the config, each ≤ ~300 LOC, each with a replay contract test. (A separate process per adapter is allowed later if a dependency forces it; not now, ponytail rung 1.)
- `voxxa-gui` (the existing Tauri window) becomes a thin client that spawns `voxxad`, renders its status stream, and writes `voxxa.toml`. It never contains alignment or recognition logic.
- Everything a volunteer can do in the GUI can be done from the CLI; the CLI is the product, the GUI is the convenience.

## Alternatives Considered
| Alternative | Pros | Cons | Why Not |
|---|---|---|---|
| Keep the monolithic Tauri app | No change; fewer moving parts today | Untestable without a GUI and a microphone; support cannot reproduce issues from a log; Companion/Stream Deck integration impossible | Fails QA rank 5 (maintainability) and the Unix brief |
| Full daemon with HTTP/WebSocket API | Remote control from phones/Companion out of the box | Adds a server, auth, ports, firewall prompts on Windows; more surface for a volunteer to misconfigure | Rung 1: nobody asked for remote control yet. JSON-lines over stdio gives 90% of the value; an HTTP shim can wrap it later |
| Library crate consumed by the GUI (no process boundary) | Simplest code sharing | Still needs the GUI to run; stdio replay testing is harder | Chosen split gives the same crate reuse plus a process boundary for free |

## Consequences
### Positive
- `voxxad --stdin-audio service.wav --setlist today.json` makes every bug reproducible from a support bundle, in CI, with no microphone.
- The evaluation harness (`voxxa-eval`) is just `voxxad` plus a scorer.
- Companion/Stream Deck/AutoHotkey users get integration for free.
### Negative
- Two binaries to sign and ship (GUI spawns the core); install size grows by a few MB.
- A stdio protocol is a compatibility promise; versioned with a `"v":1` field from day one.
### Risks
- Process spawning on Windows with WebView2 sandboxing: mitigated by Tauri's shell plugin (already a dependency) and a fallback to in-process mode behind a config flag if spawning is blocked.

## Dependencies
- Depends on: nothing new; uses crates already in `Cargo.toml` (tokio, serde_json, cpal, whisper-rs, enigo, tauri-plugin-shell).
- Constrains: ADR-003 (config/protocol shape); every adapter ADR.

## Review trigger
Re-open if the stdio protocol needs more than 8 event types, or if a church asks for phone/remote control (then add an HTTP shim, do not fold logic back into the GUI).
