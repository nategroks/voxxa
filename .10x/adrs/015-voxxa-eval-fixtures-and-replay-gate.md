# ADR-015: `voxxa-eval` scores `voxxad` against operator truth, with a replay-mode CI gate

**Status:** Accepted 2026-11-13 (the user's pre-approved Q5 decisions: stand-in fixtures built from the committed public-domain WAVs, a CI gate that replays committed hypothesis logs, one new 600-line "voxxa-eval" budget row, no GUI or conductor change, no sockets) · **Date:** 2026-11-13 · **Feature:** church-autoadvance · **Reversibility:** Easy for the thresholds and the tolerance (data and one constant); moderate for the fixture layout and the `truth.jsonl` shape (recordings made with the kit will use them) · **Owner:** nategroks · **Author:** 10x-Team (Principal Architect + Staff Engineer). The code is the ponytail lane's (`q5/ponytail`, merged into `q5/base`); this text is the 10x lane's ADR with every Decision rewritten to match that code.

## Context
Q5 adds `voxxa-eval`: a fixture is `audio.wav` + `setlist.json` + `truth.jsonl` (`{t_ms, song, slide}` from the operator). It runs `voxxad --stdin-audio --shadow`, aligns the actions to truth and reports precision, recall, median and p95 latency, false advances per set, double advances, and blank/unblank timing. CI must run it on committed fixtures and fail below `fixtures/baseline.json`.

What the accepted ADRs already settle:

| ADR | Settles | Leaves open |
|---|---|---|
| ADR-013 | `voxxad` at file speed uses a sample clock, so `t` is ms of audio from the start of the WAV; event schema v1; `--replay` re-runs the conductor on a session log without a model and is byte-identical on action lines; a new binary gets its own budget row. | How actions are scored; what truth says; how CI gets actions without a model. |
| ADR-014 | The conductor input is the stitched `hypothesis` line; `window` lines are log-only; replay skips them. A recogniser change needs logs re-recorded. | Same. |
| ADR-010 D3 | Budgets per module, "budgets only go down". | No row for an eval tool. |

Forces:
- **CI has no Whisper model** and runs on Ubuntu, Windows and macOS; the gate must be deterministic everywhere.
- **No founder recordings exist yet** (user decision (1)). The only committed audio is three public-domain WAVs read by the Windows SAPI voice: Amazing Grace verse 1 (12.0 s), verse 2 (10.3 s) and a synthetic prayer (8.1 s), plus `pd-hymns-setlist.json`. No new audio may be committed.
- **The recording kit** (`fixtures/tools/record-session.ps1`, from the Windows session) already writes `recordings/<name>/audio.wav` + `truth.jsonl` with `{t_ms, song: "<title>", slide}` and `event` metadata lines. It has no blank key, and it writes no row for the first slide of the first song (that song is only on the `start` event line).
- **The eval must not reimplement the conductor** (prompt facts).
- **Budget:** "everything else" has 214 lines left; the conductor 12; `voxxad` 52.

## Decision
1. **A third binary, `src-tauri/src/bin/voxxa-eval/`, that runs `voxxad` as a subprocess.** It never links the conductor, the recogniser or the replay code; it reads `voxxad`'s stdout (schema v1) and scores the `action` lines (`main.rs` runs and reports, `score.rs` is the pure scoring, `audio.rs` the recipe). `voxxad` is located next to the `voxxa-eval` executable (`current_exe()`'s folder plus `voxxad` and the OS's executable suffix; Cargo builds both into the same `target/<profile>/`). There is no `--voxxad` flag; a missing `voxxad` is an error (exit 2) that says to run `cargo build --bins`.
   - *Why a subprocess and not the library:* `voxxad`'s replay (`bin/voxxad/events.rs`) and its WAV reader (`bin/voxxad/main.rs`) live in the `voxxad` binary, not in `voxxa_lib`; calling "the same library entry" would mean copying that glue (clock, recorder, sink) into a second place, which is the reimplementation the prompt forbids, or moving it into the library (the "everything else" row has 214 lines left). The subprocess scores exactly the command a support engineer or the founder runs, and its cost (one process per fixture) is nothing next to a Whisper run.
2. **Two modes.**
   - **Replay (the default; CI):** `voxxad --replay <fixture>/session.jsonl --setlist <fixture>/setlist.json`. No model, no audio, no presenter. Deterministic on every OS (ADR-013).
   - **`--full` (local):** `voxxad --stdin-audio <fixture>/audio.wav --setlist <fixture>/setlist.json --shadow --model tiny --log-dir .`, with its working folder set to a fresh temporary folder. So no session log is added to the roaming `%APPDATA%` (Q3-05), and the start line's `log` path is relative (`.\session-<time>.jsonl` on Windows) and names no user folder. When the fixture has an `audio.json` (point 4), `audio.wav` is first rebuilt from it into the fixture folder, where `fixtures/.gitignore` (`*.wav`) keeps it out of git. `--record` (only with `--full`) copies the run's session log as is to `<fixture>/session.jsonl`; that is how the committed logs are re-recorded. There is no `--write-audio` and no `--model` flag: full mode always uses Whisper Tiny, the model the committed logs were recorded with. Only `--shadow` or `--replay` is ever passed: the eval never builds a presenter.
3. **Fixture layout:** `fixtures/recordings/<name>/` (the kit's folder). **Every folder under `fixtures/recordings` is a fixture** (sorted by name); fixture folders can also be given as positional arguments. There is no `--fixtures` flag. Each folder holds:
   - `setlist.json`: the setlist `voxxad --setlist` reads (`{"setlist":[{title, slides:[{id,text}]}]}`), exactly the one the log was recorded with (replay checks the FNV-1a fingerprint).
   - `truth.jsonl`: the operator's screen (point 5).
   - `session.jsonl`: the recorded `voxxad` session log (Whisper Tiny, file speed, `--shadow`). Required in replay mode; a missing file is an error (exit 2), not a skip.
   - `audio.json` (stand-ins) or `audio.wav` (real recordings, never committed; Drive per `SOURCES.md`).
   The two committed fixtures are `prayer-then-reprise` and `prayer-without-pause` (point 10).
4. **Audio recipe, `audio.json`** (stand-ins only): a JSON list of parts, each `{"silence_ms": <n>}` or `{"wav": "<path relative to the fixture folder>"}`, e.g. `[{"silence_ms":2000},{"wav":"../../../src-tauri/tests/fixtures/voxxad/amazing-grace-v1.wav"}, …]`. Every `wav` part must be 16 kHz mono 16-bit PCM (else an error); parts are concatenated sample for sample and written as one 16 kHz mono 16-bit PCM WAV. The output is a pure function of committed bytes, so it is identical on every machine.
5. **`truth.jsonl` shape** (one object per line, `t_ms` non-decreasing, ms from the first sample of the same WAV, the timeline `voxxad` uses at file speed):
   - slide shown: `{"t_ms":2100,"song":0,"slide":0}`; `song` is the 0-based index in `setlist.json` or its title (the kit writes titles; matched ignoring ASCII case), `slide` the 0-based slide within the song;
   - screen blanked: `{"t_ms":30400,"blank":true}`; `{"blank":false}`, a blank row with a `song` or `slide`, and a slide row without both are rejected;
   - lines with an `event` key (the kit's start/stop lines) are skipped; blank lines are skipped;
   - a song or slide outside `setlist.json`, an unknown title, bad JSON or `t_ms` going back is an error naming the line (exit 2).
   There is no unblank row: the operator's unblank is the first slide row after a blank row (to show the same slide again, repeat its row).
6. **Metrics** (pure code in `bin/voxxa-eval/score.rs`, tested by hand-computed cases in `tests/voxxa_eval.rs`):
   - **Tolerance:** `EARLY_MS = 1000` (operators press about 200-400 ms late, so an action may come up to 1 s before the operator's row).
   - **Spans:** truth row *i* owns the actions with `t` in `[t_i − 1000, t_{i+1} − 1000)`; the last row owns everything from `t_i − 1000` on. So a late action matches until 1 s before the operator's next change. An action before `t_0 − 1000` belongs to no row.
   - **Match:** each row is matched by at most one action: the first `goto` (slide row) or `blank` (blank row) in its span that shows the same thing (same song and slide). There is no "already showing" rule: a row whose slide is already on screen still needs its own `goto`.
   - **Voxxa's screen** starts blank with nothing shown (the conductor starts blank). A `goto` shows its slide; a `blank` blanks; an `unblank` ends the blank.
   - **Double advance:** an unmatched `goto` to the slide already shown with no blank since, or an unmatched `blank` on a blank screen (including before anything was shown). The screen would not change (dispatch drops it), but any double is a regression (Q3-08).
   - **False advance:** any other unmatched `goto` or `blank`: a wrong slide, a goto outside its row's span, or a blank during singing.
   - **Explicit `unblank` actions** are never false or double advances; they count only for unblank timing. This is a known gap: an unblank that re-shows lyrics in the middle of a prayer counts only as an early unblank. The conductor emits `unblank` rarely and neither committed log has one.
   - **Precision** = hits ÷ all `goto` actions, so doubles and false gotos count against it (1.00 with no gotos). **Recall** = hits ÷ truth slide rows (1.00 with none). **Missed** = slide rows − hits.
   - **Latency** = action `t` − the matched slide row's `t_ms` (can be down to −1000). **Median** and **p95** by nearest rank: sort the n samples, take the value at rank ⌈p·n/100⌉ (at least 1); `-` when there are no hits.
   - **Blank** (per truth blank row): the matched `blank`'s `t` − the row's `t_ms`, or *miss* when no blank matched it. **Unblank** (per truth blank row that a slide row follows): the first action after Voxxa's matched blank that is not a blank (a `goto` or an `unblank`), minus that slide row's `t_ms`; *miss* when Voxxa did not blank or never showed anything again. Both report the worst per row, with *miss* counted as infinity; `-` when there is nothing to time.
   - **False advances per set** = false advances in one fixture (one fixture is one set/service).
   - **Total row:** counts are summed (its false/set is the sum over all sets); ratios, percentiles and worst timings are computed over all fixtures' samples together.
7. **Output:** one plain-text table on stdout after a `mode: replay` or `mode: full (Whisper Tiny)` line, one row per fixture and a `total` row, 13 columns: `fixture`, `slides`, `gotos`, `hits`, `missed`, `precision`, `recall`, `median ms`, `p95 ms`, `false/set`, `double`, `blank ms`, `unblank ms`. Ratios print with two decimals, times in whole ms, `-` for nothing to measure and `miss` for a missed blank or unblank. Then one `FAIL` line per gate failure, or `OK: no metric is worse than <baseline>`.
8. **`fixtures/baseline.json` holds thresholds**, not last numbers, per row: `{"<fixture>|total": {"<metric>": <worst accepted value>}}`. Metric names: `precision`, `recall` (minimums), `median_ms`, `p95_ms`, `false_advances`, `double_advances`, `blank_ms`, `unblank_ms` (maximums). A metric left out of a row is not gated.
   - **Failures (exit 1):** a fixture or `total` row with no entry in the baseline; an unknown metric name or a non-number value (a typo must not switch a check off); a value of `-` or `miss` against a threshold set for that metric; any value worse than its threshold. Each prints `FAIL <row>: <metric> is <v>, baseline needs <=|>= <limit>` (or names the missing row or the unknown metric). **Errors (exit 2):** a usage error, a missing or bad fixture file, a bad truth or action line, `voxxad` failing. **Exit 0** otherwise. `--baseline <file>` checks against another file (default `fixtures/baseline.json`).
   - **Setting thresholds:** replay is exact, so counts and ratios are set to today's value (a new false advance, double advance or missed slide fails); each latency is the next multiple of 500 ms strictly above the measured worst (1,500 → 2,000; 2,900 → 3,000), headroom for hop jitter when a log is re-recorded. **Updating:** tighten in the same commit as a change that improves a number (a ratchet). Loosen only with the reason in the commit message; a recogniser change re-records `session.jsonl` with `--full --record` first, with the reason in the commit (ADR-014 Decision 6). A new fixture needs its row and an updated `total`.
9. **CI:** a step in the existing `check` job (Ubuntu, Windows, macOS), after `cargo test`, with `working-directory: src-tauri` and `shell: bash`: `cargo build --bin voxxad --bin voxxa-eval && cargo run --bin voxxa-eval`. Replay mode and `fixtures/baseline.json` are the defaults, so the step needs no model, no audio and no device. `cargo test` also enforces the gate (`replay_of_the_committed_fixtures_passes_the_baseline`: exit 0) and tests it (`a_broken_threshold_fails_the_gate_and_names_the_metric`: exit 1 naming `p95_ms`); the step stays so the table appears in the CI log.
10. **Stand-in fixtures** (user decision (1)), built only from the committed WAVs. Truth is at each part's start plus 100 ms (the measured speech onset is 120-130 ms into each WAV), set from the recipe offsets before any run:
    - `prayer-then-reprise`: 2 s silence, verse 1, 2 s, verse 2, 4 s, the prayer, 2 s, verse 2 again, 4 s. Setlist order Amazing Grace, Be Thou My Vision, It Is Well. Truth by index: slide 0/0 at 2,100; 0/1 at 16,100; blank at 30,400; 0/1 at 40,500. Measures a blank after a pause (the silence rule), the unblank on the reprise, and a late advance into verse 2 (Q4-04).
    - `prayer-without-pause`: 1 s, verse 1, 1 s, verse 2, the prayer straight after, 6 s. Setlist reordered (Amazing Grace is song 1). Truth by title, as the kit writes it: slide 0 at 1,100; slide 1 at 14,100; blank at 24,400. Measures a blank during a prayer with no pause (Q4-03).
    `fixtures/SOURCES.md` labels them as stand-ins (synthetic speech, not sung recordings); ticket Q5-01 replaces them with the founder's two public-domain hymn recordings.
11. **Budget (recorded deviation from ADR-010 Decision 3, user decision (3)):** `scripts/loc_budget.py` gets one new row, "voxxa-eval (src/bin/voxxa-eval*)", capped at **600** tokei code lines and excluded from "everything else". Measured at Q5 close: 458. No other budget changes. Tests in `src-tauri/tests/` are not counted.
12. **Out of scope:** any GUI change, any conductor change, sockets, new crates, resampling (`--stdin-audio` still takes 16 kHz mono only; the README gives the `ffmpeg -ar 16000 -ac 1` step, Q3-10), `--language` (Whisper auto-detects; not needed for English stand-ins), word-level latency (word times are not logged, Q4-13).

## Alternatives Considered
| Alternative | Pros | Cons | Why Not |
|---|---|---|---|
| Call the library in-process (a `voxxa_lib` entry for replay and file runs) | No subprocess; one binary | Replay and the WAV reader live in the `voxxad` binary; moving them costs "everything else" lines (214 left) or duplicating them violates "do not reimplement" | The subprocess scores exactly what users run |
| `cargo xtask` crate instead of a bin | Keeps the tool out of the app package | A second crate and workspace; the budget row the user chose names `src/bin/voxxa-eval*` | User decision (3) |
| CI runs the full mode with a downloaded model | Measures the recogniser too | 75 MB download per OS per run, minutes of inference, Whisper output not guaranteed identical across OS/CPU | User decision (2): replay only |
| Commit the stand-in `audio.wav` files | Simpler full mode | New audio in git (user decision (1) forbids it); duplicates committed bytes | A recipe rebuilds them exactly |
| Recipe as a shell/Python script | No Rust | Another language and runtime in CI paths; harder on Windows | `voxxa-eval` already reads JSON |
| Match each truth row by "the first goto to that slide at or after `t_ms`" (the handoff's first sketch) | Simple | An early advance never matches; a goto long after the operator moved on still counts as correct | Spans bounded by the next row score lateness honestly |
| Truth with `"slide": null` for a blank | One fewer key | Collides with "slide missing" errors | An explicit `"blank": true` |
| Rewrite the log's absolute path to `session.jsonl` after a full run (the first ponytail draft) | Stable file name in the log | If the rewrite misses, the user's profile path lands in a committed file | A temporary working folder and `--log-dir .` make the path relative by construction |
| Baseline as last measured numbers, fail on any change | Exact | Any harmless improvement or re-record fails CI | Thresholds with a ratchet rule |
| Raise "everything else" instead of a new row | One row | Hides the tool's size; ADR-010 says budgets only go down | User decision (3) |

## Consequences
### Positive
- Every conductor-logic change (Q6) is measured on every OS in CI, without a model, against operator truth.
- A real recording from the kit becomes a fixture by adding `setlist.json`, a first slide row, any blank rows, and `session.jsonl` from `--full --record`.
- Q4-03 (blank timing), Q4-04 (late advances) and Q3-08 regressions (double advances) have numbers.
### Negative
- The stand-ins are spoken by a synthetic voice, not sung; their numbers say little about a congregation. They prove the pipeline, not the product.
- Replay measures the conductor on recorded recogniser output only; a recogniser change needs a local re-record (Whisper Tiny, about a minute per fixture in debug).
- **Replay uses the `SmartConfig` stored in each log's start line** (`info.config`), not today's defaults. A conductor-logic change is measured by replay, but a change to a `SmartConfig` default is invisible to replay and CI until the logs are re-recorded (or their start-line config is edited). Ticket Q5-07 proposes a `--current-config` option for Q6.
- Explicit `unblank` actions are not scored as false or double advances (Decision 6); an unblank mid-prayer looks like a good, early unblank.
- Debug replay takes about 27 s for both fixtures per OS (ticket Q5-05).
- `--full` needs a debug build on Windows for speed (Q4-01).
- A third binary: like `voxxad` (Q3-03), `tauri build` may bundle it; the shipping guard is still open (ticket Q5-03).
### Risks
- **Threshold drift:** someone loosens a threshold to pass. Mitigation: the update rule above; thresholds sit in one reviewed file.
- **Truth error:** hand-written truth can be wrong. Mitigation: stand-in truth is computed from recipe offsets and measured onsets; the eval rejects rows outside the setlist and time going back.
- **Windows-recorded logs replayed on Linux/macOS:** already done in CI by `session-pd-hymns.jsonl` (ADR-013); CRLF checkouts are handled by line splitting.
- **Missing shapes:** neither fixture has an opening prayer before the first song; that shape exposes a real defect (Q5-04) and should become a third fixture when Q6 fixes it.

## Dependencies
- Depends on ADR-013 (schema v1, sample clock, replay, `--shadow`), ADR-014 (hypothesis cadence; re-record rule), ADR-010 D3 (budget rows).
- Constrains Q6 (conductor tuning is judged by `voxxa-eval` against `fixtures/baseline.json`), the founder's recordings (Q5-01), and any future `--log-keys` (it should write this `truth.jsonl` shape).

## Review trigger
Re-open when real recordings replace the stand-ins (the tolerance and thresholds may need new values), when word times are logged (Q4-13: word-level latency), when `SmartConfig` defaults start changing (Q5-07), or if `voxxa-eval` needs more than 600 lines.
