# ADR-003: One `voxxa.toml`, one JSON-lines protocol, no database

**Status:** Proposed
**Date:** 2026-10-24
**Feature:** church-autoadvance
**Reversibility:** Easy
**Owner:** nategroks
**Author:** 10x-Team (Architect + DBA)

## Context
The current app has no persisted settings (`save_settings` is a no-op), hard-coded thresholds, and sliders in the GUI that a volunteer should never touch during a service. Support staff need to read a church's configuration from a log bundle in seconds. The brief asks for suckless and for "high level button settings" for non-technical churches.

## Decision
- All configuration lives in one human-readable `voxxa.toml` in the OS config dir (`%APPDATA%\voxxa\`, `~/Library/Application Support/voxxa/`, `~/.config/voxxa/`). Every key has a commented default written on first run. The GUI exposes exactly: mode (`off | songs | songs+scripture`), audio device, presentation app, and the "Call for help" button. Everything else is in the file.
- Runtime state between core and GUI, and between core and any external controller, is newline-delimited JSON on stdio with a `"v"` field. No sockets by default.
- Durable data is files: `setlist.json` (canonical setlist cache), `bible/*.txt` (public-domain texts shipped with the binary), `usage.csv` (append-only song usage log the church can paste into CCLI reporting), `voxxa.log` + `session-YYYYMMDD.jsonl` (rotated, 14 days).
- No SQLite, no embedded database, no cloud sync.

## Alternatives Considered
| Alternative | Pros | Cons | Why Not |
|---|---|---|---|
| tauri-plugin-store JSON (already a dependency) | Zero new code | Opaque to support, GUI-owned, not shareable with `voxxad` | Core must read config without the GUI |
| SQLite for setlists, usage, logs | Queryable | A schema, migrations, a dependency, and nothing to query at this scale (one church, one machine, < 100 songs/week) | Ponytail rung 3: plain files do it |
| YAML / JSON config | Familiar | YAML has footguns; JSON has no comments, and comments are the documentation here | TOML has comments and a Rust-native parser (`toml` crate) |

## Consequences
### Positive
- A support bundle is `zip voxxa.toml voxxa.log session-*.jsonl setlist.json`; every problem is reproducible with `voxxad --replay`.
- Companion/AutoHotkey can drive and observe Voxxa with no SDK.
### Negative
- `tauri-plugin-store` becomes dead weight: remove it.
- Concurrent edits (GUI writes while user edits in Notepad) need a "reload on change" and last-writer-wins; acceptable for a single-user tool.
### Risks
- Volunteers editing TOML by hand will break it; mitigated by: GUI never requires it, strict parse errors shown in plain English with the line number, and `voxxad --check-config`.

## Dependencies
- Depends on: ADR-001.
- Constrains: adapter configuration shape (`[adapter.propresenter] host = …`).

## Review trigger
Re-open if any file exceeds ~10 MB or if two machines ever need to share state (then consider a sync file format, still not a database).
