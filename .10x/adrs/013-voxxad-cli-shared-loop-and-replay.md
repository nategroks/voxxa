# ADR-013: `voxxad` CLI on a shared listening loop, with a sample clock and a replayable session log

**Status:** Accepted 2026-11-12 (the user's pre-approved Q3 decisions: a separate 600-line budget row for `voxxad`, no sockets, no GUI change) · **Date:** 2026-11-12 · **Feature:** church-autoadvance · **Reversibility:** Costly for the wire format (`"v":1` is a promise to Companion users and to `voxxa-eval`), easy for everything else · **Owner:** nategroks · **Author:** 10x-Team (Principal Architect + Staff Engineer)

## Context
Q3 adds a second binary, `voxxad`, that runs Voxxa's pipeline headlessly. Phase 2 had to check whether the accepted ADRs already decide how.

| ADR (as adjusted by ADR-010) | What it settles for `voxxad` | What it leaves open |
|---|---|---|
| ADR-001 | A `voxxad` CLI over `voxxa_lib`, the GUI stays in-process (ADR-010 Decision 2). JSON lines on stdout with `"v":1`. Review trigger: more than 8 event types. | Its event names (`advance`, `goto`, `show_ref`, `transcript`) differ from Q3's (`status`, `hypothesis`, `action`, `error`). How the GUI and the CLI share the audio loop. How a run is made reproducible. |
| ADR-003 | Stdio JSON lines, no sockets by default, durable data as files, a rotated session log kept 14 days, "every problem is reproducible with `voxxad --replay`". | The log's contents and file name (`session-YYYYMMDD.jsonl` there, `session-YYYYMMDD-HHMM.jsonl` in Q3). What replay feeds the conductor. `voxxa.toml` stays deferred (ADR-010), so the CLI is configured by flags. |
| ADR-004 | The recogniser sits behind a seam so engines can be swapped. | Nothing about time: the conductor's methods take `now: Instant`, so a run at file speed and a replay need a clock that does not come from the wall. |
| ADR-010 Decision 3 | Budgets per module; "budgets can only go down". | A new binary has no row. "Everything else in src-tauri/src" is 2,483 of 2,500. |

**Verdict:** the direction is covered (ADR-001 and ADR-003 already promise this CLI and its replay), but four architectural choices are not: the shared loop, the clock, the event schema and the budget row. Those are decided here.

## Decision
1. **One loop, in the library.** The body of `commands::start_listening_with_state` moves to `voxxa_lib::listen::run`. It is the only code that turns audio into conductor calls. The GUI and `voxxad` each call it with three things of their own:
   - a **clock**, `FnMut(samples_received) -> Instant`. The GUI passes `Instant::now()`, so its timing is unchanged. `voxxad` passes a sample clock at file speed (`base + samples/16 ms`) and a wall clock rounded to whole milliseconds with `--realtime` or `--device`.
   - a **recogniser**, `FnMut(&[f32]) -> Result<String>`. Both pass `TranscriptionEngine::transcribe`. This is the ADR-004 seam in its smallest form, a closure, until a second engine exists.
   - an **event sink**, `FnMut(Instant, Event)`. The GUI's sink emits the same Tauri events, in the same order, as before and dispatches through `commands::dispatch`. The `voxxad` sink writes JSON lines and, outside shadow mode, dispatches through `dispatch::drive` (the one dispatch path, Q2).
2. **Every conductor input is an `Input`**: `Tick`, `Speech` or `Heard(text)`. The loop reports each input to the sink just before applying it through `Conductor::apply(&Input, now)`; replay applies logged inputs through the same function. The conductor reads time only as differences between the `now` values it is given (`state_since`, set from the wall clock in `Conductor::new`, is never read), so the same inputs at the same millisecond offsets give the same actions.
3. **Event schema v1.** One JSON object per line; `v` and `t` (milliseconds since the session started, by the session's clock) come first, then `type`:

   | `type` | Fields | stdout | session log |
   |---|---|---|---|
   | `status` | `state`: `start`, `listening`, `singing`, `inter_verse_silence`, `blank_hold` or `stop`; `blank` for conductor states; `info` for `start` (mode, shadow, presenter, model, setlist fingerprint, `SmartConfig`) and `stop` (reason) | yes | yes |
   | `hypothesis` | `text`: a recogniser transcript; also the conductor input `Heard` | yes | yes |
   | `action` | `action`: `goto` (`song`, `slide`, `global`, `title`), `blank` or `unblank` | yes | yes |
   | `error` | `message` | yes | yes |
   | `tick` | none: the 200 ms conductor tick | no | yes |
   | `speech` | none: VAD says speech | no | yes |

   Six types, under ADR-001's limit of 8. The `action` line leaves out the slide text: the setlist has it, and the line stays short for Companion. `speech` lines are coalesced: only the last one before the next `tick` or `hypothesis` is written, because `Conductor::on_speech` only records the time and nothing reads it in between (this invariant is noted on `on_speech`). A device delivers about 100 chunks a second; coalescing keeps an hour near 5 MB.
4. **Session log.** Every line, stdout or not, goes to `<data dir>/voxxa/sessions/session-YYYYMMDD-HHMM.jsonl` (local time; `-2`, `-3` … when the minute is taken; `--log-dir` overrides the folder). At start, `session-*.jsonl` files whose name date is more than 14 days old are deleted. Lines are written as they happen, so a killed process loses nothing.
5. **Replay.** `voxxad --replay <session.jsonl> --setlist <json>` reads the `start` line (its `SmartConfig` and setlist fingerprint; a different setlist is refused), builds a conductor, and applies every `tick`, `speech` and `hypothesis` line at `base + t`. It prints `hypothesis`, `action` and `status` lines; the `action` lines are byte-identical to the live run's. Replay never loads a model, opens audio or builds a presenter. A line that does not parse (a crash can leave a partial last line) or whose `v` is not 1 stops replay with an error naming the file and the line (`<path>: session line N: …`).
6. **Shadow.** `--shadow` never builds or connects a presenter: actions are computed, printed and logged, and nothing leaves the process. `--presenter none` (the default) does the same without the word.
7. **Budget (recorded deviation from ADR-010 Decision 3).** `scripts/loc_budget.py` gets a new row, "voxxad binary (`src-tauri/src/bin/voxxad.rs`, `src-tauri/src/bin/voxxad/**`)", capped at **600** tokei code lines and excluded from "everything else". This is a new budget, not a raise; no other budget changes. The extracted loop fits the existing "everything else" row (2,500); `Input` and `Conductor::apply` count in the conductor row (1,200). Tests go in `src-tauri/tests/` (not counted).
8. **Out of scope** (user decision): sockets, any GUI change, `voxxa.toml`. Flags added beyond the prompt, all optional: `--host`/`--port` (presenter address, needed to point at a mock), `--model` (default: the smallest downloaded Whisper model) and `--log-dir`. Flags that a mode would ignore are usage errors: `--replay` takes only `--setlist`, and `--realtime` needs `--stdin-audio`. At file speed the feeder queue holds at most 64 chunks (6.4 s), so a long WAV is not held in memory twice.

## Alternatives Considered
| Alternative | Pros | Cons | Why Not |
|---|---|---|---|
| ADR-001 as first written: the GUI spawns `voxxad` and reads its stdout | One process boundary, as planned in October | Rewrites the GUI's start/stop, dispatch and device-loss handling; the user ruled out GUI changes | ADR-010 already chose "GUI stays in-process" |
| A second copy of the loop in `voxxad` | No change to `commands/mod.rs` | Two loops drift; the CLI would no longer reproduce the GUI's behaviour, which is the point of a support replay | The prompt asks for one function |
| Replay by re-running the recogniser on recorded audio | No input log needed | Needs the model and the audio; Whisper output is not guaranteed stable across builds or threads | "without running the recognizer" |
| Log the conductor's state snapshots instead of its inputs | Replay is trivial | Proves nothing: replay would copy the actions instead of recomputing them | Determinism must be checked, not assumed |
| A mockable clock trait inside the conductor | Explicit | Touches every conductor method and its 30 tests | The conductor already takes `now`; only the loop needed a different source |
| Log every `speech` input uncoalesced | No reliance on `on_speech`'s body | About 100 lines a second from a device | Coalescing is exact today and the invariant is written down |
| Raise "everything else" from 2,500 to ~3,100 | One row | Hides the CLI's size in a shared bucket; ADR-010 says budgets only go down | User decision: a separate 600 row |
| `clap` / `hound` for flags and WAV | Less hand-written code | Two new crates through `cargo deny` for ~60 lines | Prefer no new crates |

## Consequences
### Positive
- `voxxad --stdin-audio x.wav --setlist s.json --shadow` reproduces a service from a recording with no microphone and no presentation app; Q4's `voxxa-eval` builds on it, with `t` in audio time.
- A support bundle's session log replays without the model, byte-identical on actions, so a wrong slide can be bisected against conductor changes.
- The GUI and the CLI cannot drift: they share the loop, `Conductor::apply` and `dispatch::drive`.
### Negative
- `voxxad` duplicates no logic, but it does hand-parse flags and WAV headers (16-bit PCM or 32-bit float, mono, 16 kHz only).
- The session log keeps full transcripts for 14 days, prayers and spoken names included (the diagnostics ring buffer keeps only recent ones, in memory). Voxxa never uploads it, but it is not guaranteed to stay on the machine: it is written under `dirs::data_dir()`, which on Windows is the roaming `%APPDATA%`, and on a domain-joined church network with roaming profiles that folder is copied to the server. We keep `data_dir` so the logs sit beside the models (`%APPDATA%\voxxa\models`), where the prompt asked for them ("in the data dir"). Considered alternative: `dirs::data_local_dir()` (`%LOCALAPPDATA%`, never roams), at the cost of a second Voxxa folder; `--log-dir` already moves them. Q5's privacy text must say this.
- File-speed runs are only as fast as Whisper, because the loop transcribes synchronously (Q0 finding, unchanged).
### Risks
- **Keystroke from a terminal.** `voxxad` has no window, so the GUI's own-window focus guard cannot apply; `--presenter keystroke` types into whatever window has focus, the terminal included. Mitigation: never the default, `--shadow` and `none` build no presenter, without `--shadow` `voxxad` prints one stderr line before it starts ("keystroke types into the focused window; focus the presentation app"; stderr, so it is not an event), and the README/INTEGRATIONS text (Q5) must say "focus the presentation app". Ticket for Q5.
- **Release packaging (not decided by either Q3 lane; nothing checked yet).** Cargo builds every `src/bin` target, and tauri-cli reads them as extra binaries, so `tauri build` will probably put `voxxad` in the installers too (MSI/NSIS, deb, AppImage, `.app`). That would add a second executable to macOS signing and notarisation and to the Flatpak manifest. Decision: Q3 does not ship `voxxad` to churches. The guard is `required-features = ["cli"]` on a `[[bin]] voxxad` entry (a feature that is off by default; CI and `cargo test` then pass `--features cli`), or a bundler exclusion if tauri-cli offers one; whichever is used, the shipping decision is Q5's (with Companion docs). Ticket Q3-T1: confirm with `tauri build --no-bundle` or in the CI build job whether `voxxad` reaches the bundle, then apply the guard or decide to ship and sign it.
- **Conductor changes that read time differently** (e.g. `on_speech` doing more than recording `now`) would break coalesced replay. Mitigation: the invariant comment on `on_speech`, and the replay test on a real recorded session in `src-tauri/tests/`.
- **Wire format drift.** Mitigation: serialisation round-trip tests pin every type; a breaking change needs `"v":2`.

## Dependencies
- Depends on: ADR-001 and ADR-003 as adjusted by ADR-010; ADR-004 (recogniser seam); the Q2 single dispatch path (`dispatch::drive`).
- Constrains: Q4 `voxxa-eval` (reads these lines), Q5 Companion docs, the "Shadow (log only)" toggle (maps to `--shadow`), `--record-adapter` and `--export-corpus` (new flags, same schema). A future in-GUI session log reuses the same sink and schema.

## Review trigger
Re-open if the protocol needs more than 8 event types, if a second recogniser lands (turn the closure into ADR-004's trait), or if the `voxxad` row needs more than 600 lines.
