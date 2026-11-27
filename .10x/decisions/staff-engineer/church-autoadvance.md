# Staff Engineer — church-autoadvance

Date: 2026-10-24 · Status: Proposed

## Patterns to follow
- Ponytail ladder on every change; `shortcut:` comments for known limits; one small test per new branch/loop/parser.
- Errors: `anyhow` inside engine code, plain-English `String` across IPC and in JSON `error` events with a stable `code`.
- Traits with capability flags (`Recognizer`, `Adapter`); config-selected implementations; no plugin loader.
- Fixtures over mocks: recorded HTTP/WS traffic for adapters; WAV + truth for the tracker; `--replay` for every bug.
- Protocol and config are append-only (`"v":1`, new TOML keys with defaults).
- Keep the hand-rolled LCS (no crate has `partial_ratio`, research E8) but apply it to word tokens.

## Cross-cutting
- Logging: `log` + `env_logger` stay; every decision logged with scores to `session-*.jsonl`.
- Secrets: OS keychain via `keyring`; never in TOML.
- CI: fmt, clippy -D warnings, test, deny, tokei budgets, eval gate on fixtures.
- Delete now: `mcp`, `tauri-plugin-store`, GUI sliders, `overlay.html` (after P2.1).

## Known defects to fix first (P1.1)
`load_model` never called; `VadEngine::initialize` never called; `next_slide_manual` pushes a sentinel word; `model_loaded` hard-coded; scores always 0.0; `save_settings` no-op; CSP null.

## V7 (2026-11-27): the `trace` line's shape (ADR-003 amendment 1, FA-65) and the licence inventory (ADR-005 amendment 1, FA-50)

Lane `v7/10x` off `0190515`; no code in V7. Code references at 0190515.

**The `trace` line (ADR-003 carries the text; ADR-001 amendment 1 is the cross-reference).**
- One line type, `trace`, with a required `kind` ∈ {`follow`, `verify`, `screen`, `watchdog`, `operator`, `level`, `cue`} and numbers-only fields: integers, floats, booleans, codes as small integers. The only strings on the line are `type` and `kind`. Line types become 8 (`status`, `hypothesis`, `action`, `error`, `tick`, `speech`, `window`, `trace`), which reaches ADR-001's "more than 8" trigger without passing it; a ninth type reopens ADR-001; `follow`, `cue` and read-back are kinds, never line types.
- **Session-log only** (my call, from "replay skips it like `window`"): `on_stdout` (`events.rs:103-105`) stays false for it, so the stdout protocol of ADR-001 is unchanged for Companion-style consumers; screens get the same numbers as Tauri events or HTTP routes (V11's reason line, V13a's `/state`). A kind that must reach stdout amends the table first.
- **Replay** (`events.rs:194-243`) parses every line as a `Line` and feeds the conductor only `tick`, `speech`, `hypothesis` (`:224-232`); `trace` falls through `_ => continue` like `window`, so byte-identical replay holds with no new arm. Because every line is parsed, `Kind::Trace` must exist before any build writes the line (`:197-200` names an unknown type's line and stops); the first emitter adds the variant and the line in one commit.
- **Shape:** one `Trace` enum beside `Event` in `listen.rs` (`Event` at `listen.rs:34`), `#[serde(tag = "kind", rename_all = "snake_case")]`, one variant per kind, each payload a struct owned by the emitting module and flattened (`input::Level`, `position::Screen`, …), so the fields live in that module's budget row and the compiler sees the types (`u16`/`u32`/`i32`/`f32`/`bool`, codes `u8`; no `String`, no `Value`). `Event::Trace(Trace)`; in `events.rs`, `Kind::Trace` holds the flattened `Trace` and `Recorder::lines` (`:159`) maps it. Lines: `listen.rs` about 4 once + 1 per kind (wiring row, inside the emitting step's share; V3b's ≤ 15 includes the first); `events.rs` ≤ 4 once (voxxad row, V3b's share).
- **The test:** `every_event_type_serialises_as_documented_and_round_trips` (`src-tauri/tests/voxxad_events.rs:61`) gains a sample per kind; a new `a_trace_line_carries_numbers_only` walks each sample's JSON and fails on any string value other than `type` and `kind`. V3b writes both with `level`; every later kind adds its sample.
- **Owners of the field lists** (the table's "written by" cells): `level` V3b; `verify` V8b; `screen` **V8c** (the pack's task 6 said V8b before V8c existed; V8c merges first and V8b's FreeShow read-back is ADR-036's first `api` source, so V8b adds to the same row); `watchdog` V13s; `operator` **V13a** (the V13a prompt writes the line; the pack's task 6 said V13s, which runs later and confirms); `follow` V11; `cue` V13b (deferred, ADR-024 § 5). Two reassignments, both recorded in the table.
- **Docs:** `docs/releases/voxxad-cli-preview.md` and the MIT protocol reference gain the line with V3b, not in V7 (a documented line no build writes would mislead an integrator).

**The licence inventory (ADR-005 amendment 1).**
- Inventory, not a gate (founder FA-50): every model, weight file and aligner with name, version, licence, where it runs (lab-only or product), the step that added it and a (measured)/(cited) status. Permissive preferred (MIT, Apache-2.0, BSD, CC-BY, CC0); non-permissive flagged; nothing blocked now; FA-76 (voxxa's own licence) parked, with the recorded caution that a ministry-use clause is source-available and cannot sit over a copyleft dependency.
- Rows at V7: Whisper `ggml-tiny.bin` and `ggml-small.bin` (the two in use), plus `ggml-base.bin` and `ggml-medium.bin` (in the model list, `transcription/mod.rs:28-33`, source `:39`), MIT per whisper.cpp; **Silero VAD v5** added by me as a product model weight (bundled ONNX via `voice_activity_detector` 0.2, `Cargo.toml:71-72`, `vad/mod.rs:3-4,21`), MIT. All "(cited: verify in V3f)"; V3f marks them (measured) after reading each upstream licence file. Placeholder rows for the lab aligners (V3f) and the kit tools' Python dependencies (V5). No row flagged.
- `cargo deny` keeps gating crates; the table covers what `cargo deny` cannot see.
