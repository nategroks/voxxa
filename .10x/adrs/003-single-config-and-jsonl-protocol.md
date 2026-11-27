# ADR-003: One `voxxa.toml`, one JSON-lines protocol, no database

**Status:** Accepted
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

## Amendment 1 (Accepted, FA-65, 2026-11-27; written in V7, no code): schema v1 gains one line type, `trace`

**Status: Accepted** (founder FA-65, option (a), 2026-11-27; drafted in the prompt pack's V7 task 6 and recorded by V7's 10x lane, Staff Engineer). This ADR carries the text; ADR-001 amendment 1 cross-references it. Code references are at `0190515`. **No code in V7**: the first step that emits a kind writes the variant, the test and its row of the table below.

**Decision.** The JSON-lines schema v1 (ADR-013; `src-tauri/src/cli/voxxad/events.rs`) gains exactly one line type, `trace`: `{"v":1,"t":<ms>,"type":"trace","kind":"<kind>", …}`.
1. **`kind` is required** and is one of `follow`, `verify`, `screen`, `watchdog`, `operator`, `level`, `cue`. A new kind needs a new row in the table below, through an amendment here; nothing else.
2. **Numbers only.** The fields of every kind are integers, floats, booleans and codes as small integers (a reason is a code, never a word; a source is a code). The only string values on a `trace` line are `type` and `kind`. No slide text, no heard word, no title, no path, no name. A kind that carries text fails the serialisation test (point 5).
3. **Replay skips it like `window`.** `replay` (`events.rs:194-243`) parses every line as a `Line` and feeds the conductor only `tick`, `speech` and `hypothesis` (`:224-232`); `window` and, from its first emitter, `trace` fall through `_ => continue`. So `voxxad --replay` reproduces the recorded actions byte for byte with or without trace lines, and `replay_reproduces_the_recorded_actions_byte_for_byte` (`src-tauri/tests/voxxad_events.rs:242`) stays the proof. Because every line is parsed, `Kind::Trace` must exist in `events.rs` before any build writes a trace line (a log with an unknown type stops replay at that line number, `:197-200`); the emitting step adds the variant and the line in the same commit.
4. **Session log only.** Like `tick`, `speech` and `window`, a `trace` line is not written to stdout (`on_stdout`, `events.rs:103-105`): the stdout protocol stays `status`, `hypothesis`, `action`, `error` (ADR-001). A screen that needs the same numbers gets them as a Tauri event or an HTTP route, as V11 (the reason line) and V13a (`/state`) already plan. A kind that must reach stdout amends this table first.
5. **The serialisation test.** `every_event_type_serialises_as_documented_and_round_trips` (`voxxad_events.rs:61`) gains one `trace` sample per kind with every field set, and a new test `a_trace_line_carries_numbers_only` walks the serialised JSON of every kind's sample and fails on any string value other than `type` and `kind`. The step that emits the first kind writes both (V3b, `level`); each later kind adds its sample to both.
6. **No top-level `follow`, `cue` or read-back line type is ever added.** Those are kinds of `trace`. Line types after this amendment: `status`, `hypothesis`, `action`, `error`, `tick`, `speech`, `window`, `trace` = 8, which reaches ADR-001's review trigger ("more than 8") without passing it; a ninth type reopens ADR-001.

**The kinds.** The field list of a kind is written by the step that emits it, inside this table, before that step merges; until then the cell says who writes it. Every field is a number per point 2.

| `kind` | Emitted by | Fields (numbers only) | Written by | Stdout |
|---|---|---|---|---|
| `level` | the input chain (ADR-030 amendment 1): once per second while listening, and the same numbers on the session start line | fields: written by V3b task 6 (its prompt names peak and RMS dBFS, floor, hum share at 60/180/300/420 Hz, `degraded` and its reason codes) | V3b | no |
| `verify` | the FreeShow driver after a move it made (ADR-032: fire-then-verify, ids only) | fields: written by V8b | V8b | no |
| `screen` | a position source on a settled position or a re-anchor (ADR-036: `screen` hashes first, `keys` hint second, `api` read-back third; FA-77) | fields: written by V8c, which merges before V8b; the prompt pack's V7 task 6 named V8b when V8c did not yet exist, and V8b's FreeShow read-back is now ADR-036's first `api` source, so V8b adds its fields in this same row if it needs more | V8c (V8b adds) | no |
| `watchdog` | the readiness watchdog on each hold-reason change (ADR-034) | fields: written by V13s (its prompt names `t`, `reason_code`, `value`, `action`) | V13s | no |
| `operator` | dispatch on HOLD, STOP FOLLOWING and resume (ADR-025 § 7; the V13a prompt writes the line on each flag change) | fields: written by V13a (its prompt names `t`, `kind_code`, `source` as a code: key, http, screen); the pack's V7 task 6 named V13s, which runs after V13a and confirms the row | V13a (V13s confirms) | no |
| `follow` | the follower, one line per decision (ADR-031) | fields: written by V11 (its prompt names `part`, `ptr`, `occ`, `word`, `m`, `A`, `d`, `r`, `gate`, `why` as a code, `arm`, `brk`) | V11 | no |
| `cue` | the trigger engine (ADR-024 § 5: ids and scores only) | fields: written by V13b (the layout engine, deferred behind the first autonomous Sunday, R17) | V13b | no |

**Shape in code (Staff Engineer, V7; for the emitting steps).** One `Trace` enum beside `Event` in `src-tauri/src/listen.rs` (`Event` is at `listen.rs:34`), `#[serde(tag = "kind", rename_all = "snake_case")]`, one variant per kind; each variant's payload is a struct owned by the emitting module and flattened (`input::Level`, `position::Screen`, …), so the fields live in that module's budget row and the compiler sees their types. `Event::Trace(Trace)` is the library-side event. In `events.rs`, `Kind::Trace` holds the flattened `Trace`, and the recorder maps `Event::Trace` to it (`Recorder::lines`, `:159`); `on_stdout` and `replay` need no new arm. Field types: `u16`/`u32`/`i32`/`f32`/`bool`; codes as `u8`. No `String`, no `Value`.

**Where the lines go (ADR-028 amendment 1 § 5 and § 7).** `listen.rs` (wiring row): about 4 lines once for the enum and `Event::Trace`, then 1 line per kind, inside the emitting step's wiring share (V3b's ceiling ≤ 15 includes the first). `events.rs` (voxxad row): ≤ 4 lines once (`Kind::Trace`, the recorder arm), V3b's share. Payload structs: the emitting module's row. Tests: uncounted.

**Docs.** `docs/releases/voxxad-cli-preview.md` and the MIT protocol reference (ADR-005) gain the `trace` line when the first kind merges (V3b), not in V7.
