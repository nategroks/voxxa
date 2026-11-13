# ADR-015: `voxxa-eval` scores `voxxad` against operator truth, with a replay-mode CI gate

**Status:** Accepted 2026-11-13 (the user's pre-approved Q5 decisions: stand-in fixtures built from the committed public-domain WAVs, a CI gate that replays committed hypothesis logs, one new 600-line "voxxa-eval" budget row, no GUI or conductor change, no sockets) · **Date:** 2026-11-13 · **Feature:** church-autoadvance · **Reversibility:** Easy for the thresholds and the tolerance (data and one constant); moderate for the fixture layout and `truth.jsonl` shape (recordings made with the kit will use it) · **Owner:** nategroks · **Author:** 10x-Team (Principal Architect + Staff Engineer)

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
- **The recording kit** (`fixtures/tools/record-session.ps1`, from the Windows session) already writes `recordings/<name>/audio.wav` + `truth.jsonl` with `{t_ms, song: "<title>", slide}` and `event` metadata lines. It has no blank key, and it writes no row for the first slide of the first song.
- **The eval must not reimplement the conductor** (prompt facts).
- **Budget:** "everything else" has 214 lines left; the conductor 12; `voxxad` 52.

## Decision
1. **A third binary, `src-tauri/src/bin/voxxa-eval/`, that runs `voxxad` as a subprocess.** It never links the conductor, the recogniser or the replay code; it reads `voxxad`'s stdout (schema v1) and scores the `action` lines. `voxxad` is found next to `voxxa-eval` (`current_exe()`'s folder; Cargo builds both into the same `target/<profile>/`) or given with `--voxxad <path>`.
   - *Why a subprocess and not the library:* `voxxad`'s replay (`bin/voxxad/events.rs`) and its WAV reader live in the `voxxad` binary, not in `voxxa_lib`; calling "the same library entry" would mean copying that glue (clock, recorder, sink) into a second place, which is the reimplementation the prompt forbids, or moving it into the library (the "everything else" row has 214 lines left). The subprocess scores exactly the command a support engineer or the founder runs, and its cost (one process per fixture) is nothing next to a Whisper run.
2. **Two modes.**
   - **`--replay` (the default; CI):** `voxxad --replay <fixture>/session.jsonl --setlist <fixture>/setlist.json`. No model, no audio, no presenter. Deterministic on every OS (ADR-013).
   - **`--full` (local):** `voxxad --stdin-audio <wav> --setlist <fixture>/setlist.json --shadow --log-dir .`, run in a fresh temporary folder (so no session log is added to the roaming `%APPDATA%`, Q3-05, and the log's `info.log` path is relative, with no user name in it). `--model <name>` is passed through (default: `voxxad`'s, the smallest downloaded model). `--record` copies the run's session log to `<fixture>/session.jsonl`, which is how the committed logs are re-recorded. The WAV is `<fixture>/audio.wav` if present, else it is built from `<fixture>/audio.json` (point 4) into the temporary folder. Only `--shadow` is ever passed: the eval never builds a presenter.
3. **Fixture layout:** `fixtures/recordings/<name>/` (the kit's folder), each with:
   - `setlist.json`: the setlist `voxxad` reads (`{"setlist":[{title, slides:[{id,text}]}]}`), exactly the one the log was recorded with (replay checks the FNV-1a fingerprint).
   - `truth.jsonl`: the operator's screen (point 5).
   - `session.jsonl`: the recorded `voxxad` session log (Whisper Tiny, file speed, `--shadow`). Required in replay mode; a fixture without it is an error, not a skip.
   - `audio.json` (stand-ins) or `audio.wav` (real recordings, never committed; Drive per `SOURCES.md`).
   A folder is a fixture when it holds `truth.jsonl` and `setlist.json`. `voxxa-eval` takes `--fixtures <dir>` (default `fixtures/recordings`) and optional fixture names.
4. **Audio recipe, `audio.json`** (stand-ins only): `{"v":1,"parts":[{"silence_ms":1500},{"wav":"<path relative to this file>"},…]}`. Every `wav` part must be 16 kHz mono 16-bit PCM; parts are concatenated sample for sample and written as a 16 kHz mono 16-bit WAV. The output is a pure function of committed bytes, so it is identical on every machine. `voxxa-eval --write-audio` writes `<fixture>/audio.wav` for listening; `audio.wav` is git-ignored under `fixtures/recordings/`.
5. **`truth.jsonl` shape** (one object per line, `t_ms` non-decreasing, ms from the first sample of the same WAV, the timeline `voxxad` uses at file speed):
   - slide shown: `{"t_ms":1620,"song":0,"slide":0}`; `song` is the 0-based index in `setlist.json` or its title (the kit writes titles; case-insensitive), `slide` the 0-based slide within the song;
   - screen blanked: `{"t_ms":28900,"kind":"blank"}`;
   - lines with an `event` key (the kit's start/stop lines) are skipped;
   - anything else (an unknown `kind`, a slide out of range, an unknown title, `t_ms` going back) is an error naming the file and line.
   Before the first row the screen is blank (nothing shown); that state is not scored. There is no `unblank` kind: the operator's unblank is the first slide row after a blank row.
6. **Metrics** (pure code in `bin/voxxa-eval/metrics.rs`, tested by hand-computed cases):
   - **Tolerance** `E = 750 ms` (the kit's README: operator reaction time is 200-400 ms late, so score with ±750 ms).
   - **Windows:** truth row *i* owns `[t_i − E, t_{i+1} − E)`; the last row owns everything after it; the implicit "nothing shown" state owns `[0, t_0 − E)`.
   - **Voxxa's screen:** starts blank; `goto s` shows `s`; `blank` blanks; `unblank` shows the last slide sent by a `goto` (no change if none).
   - **Each action, in order,** is classified in the window it falls in (by its `t`), comparing the screen before and after it:
     - no change: a `goto` or `unblank` → **double advance** (a goto to the slide already shown; dispatch would drop it, so the screen is unaffected, but it is a defect: Q3-08's path is closed, so any repeat is a regression); a `blank` on a blank screen is not scored;
     - the screen now equals the window's row, and that row has no match yet → the row's **match**;
     - otherwise: **false advance** if the screen now shows a slide, **false blank** if it is now blank.
   - A row whose expected screen is already on screen when its window opens is matched with no latency sample (e.g. Voxxa never blanked, and the operator re-shows the same slide). Every other row without a match is a **missed slide** or a **missed blank**.
   - **Precision** = slide rows matched by a `goto`/`unblank` ÷ (those + false advances); 1.0 when Voxxa sent no slide change. **Recall** = matched slide rows ÷ slide rows. Both per fixture and pooled for the total.
   - **Latency** = action `t` − `t_ms` of the row it matched (slide rows only; can be down to −750). **Median** and **p95** by nearest rank: sort the n samples, take the value at rank ⌈p·n/100⌉. Per fixture and pooled.
   - **Blank latency** = the matching `blank`'s `t` − the blank row's `t_ms`; reported as the worst (largest) per fixture and in total.
   - **Unblank latency:** for a slide row *u* that directly follows a blank row *b*, the first action in `[t_b − E, end of u's window)` that turns a blank screen into a slide, minus `t_u` (negative when Voxxa re-showed lyrics during the prayer). Worst per fixture and in total; none found is a **missed unblank**.
   - **False advances per set** = false advances in one fixture (one fixture is one set/service).
7. **Output:** one plain-text table on stdout, one row per fixture and a total row: slide rows, precision, recall, median and p95 latency, false advances, double advances, missed slides, blanks matched, worst blank and unblank latency, false blanks. Then one line per gate failure.
8. **`fixtures/baseline.json` holds thresholds**, not last numbers:
   ```json
   {"v":1, "per_set":{"false_advances_max":0, ...}, "total":{"recall_min":1.0, "latency_p95_ms_max":12000, ...}}
   ```
   Keys are a metric name plus `_min` or `_max`; `per_set` is checked against every fixture row, `total` against the total row. An unknown key is a usage error (a typo must not switch a check off). A latency metric with no sample (nothing matched) is not checked; the `missed_*` and `recall` thresholds catch that case. With `--baseline <file>`, `voxxa-eval` exits **1** when any metric is worse than its threshold and prints `FAIL <row>: <metric> = <value>, worse than <min|max> <threshold>`; it exits **2** on a usage, fixture or `voxxad` error; **0** otherwise.
   - **Setting thresholds:** replay is exact, so counts are set to today's value (a new false advance, double advance or missed slide fails); latencies are today's worst rounded up to the next 500 ms; ratios are today's value. **Updating:** tighten in the same commit as a change that improves a number (a ratchet). Loosen only with the reason in the commit message and a line in `fixtures/README.md`'s history; a recogniser change re-records `session.jsonl` with `--full --record` first, with the reason in the commit (ADR-014 Decision 6).
9. **CI:** a step in the existing `check` job (Ubuntu, Windows, macOS), after `cargo test`, which has already built `voxxad` and `voxxa-eval` in debug: `cargo run --bin voxxa-eval -- --fixtures ../fixtures/recordings --baseline ../fixtures/baseline.json` from `src-tauri`. Replay mode is the default, so the step needs no model, no audio and no device. `cargo test` also runs the binary on the committed fixtures (exit 0) and on a broken scratch baseline (exit 1, failing metric named), so the gate itself is tested.
10. **Stand-in fixtures** (user decision (1)), both built only from the committed WAVs, with truth at each segment's start plus its speech onset (first sample above 2 % of full scale, 121-130 ms into each WAV), rounded down to 10 ms:
    - `standin-paused-service`: 1.5 s silence, verse 1, 2 s, verse 2, 3 s, the prayer, 4 s. Truth: slide 0/0, slide 0/1, blank at the prayer.
    - `standin-prayer-between`: 1 s, the prayer (nothing shown), 2 s, verse 1, the prayer straight after (no pause, Q4-03), 1 s, verse 2, 3 s. Truth: slide 0/0, blank, slide 0/1 (the unblank).
    `fixtures/SOURCES.md` labels them as stand-ins (synthetic speech, not sung recordings); ticket Q5-01 replaces them with the founder's two public-domain hymn recordings.
11. **Budget (recorded deviation from ADR-010 Decision 3, user decision (3)):** `scripts/loc_budget.py` gets one new row, "voxxa-eval (src/bin/voxxa-eval*)", capped at **600** tokei code lines and excluded from "everything else". No other budget changes. Tests in `src-tauri/tests/` are not counted.
12. **Out of scope:** any GUI change, any conductor change, sockets, new crates, resampling (`--stdin-audio` still takes 16 kHz mono only; the README gives the `ffmpeg -ar 16000 -ac 1` step, Q3-10), `--language` (Whisper auto-detects; not needed for English stand-ins), word-level latency (word times are not logged, Q4-13).

## Alternatives Considered
| Alternative | Pros | Cons | Why Not |
|---|---|---|---|
| Call the library in-process (a `voxxa_lib` entry for replay and file runs) | No subprocess; one binary | Replay and the WAV reader live in the `voxxad` binary; moving them costs "everything else" lines (214 left) or duplicating them violates "do not reimplement" | The subprocess scores exactly what users run |
| `cargo xtask` crate instead of a bin | Keeps the tool out of the app package | A second crate and workspace; the budget row the user chose names `src/bin/voxxa-eval*` | User decision (3) |
| CI runs the full mode with a downloaded model | Measures the recogniser too | 75 MB download per OS per run, minutes of inference, Whisper output not guaranteed identical across OS/CPU | User decision (2): replay only |
| Commit the stand-in `audio.wav` files | Simpler full mode | New audio in git (user decision (1) forbids it); duplicates committed bytes | A recipe rebuilds them exactly |
| Recipe as a shell/Python script | No Rust | Another language and runtime in CI paths; harder on Windows | `voxxa-eval` already reads JSON |
| Match each truth row by "the first goto to that slide at or after `t_ms`" (the handoff's first sketch) | Simple | An early advance never matches; a goto long after the operator moved on still counts as correct | Windows bounded by the next row score lateness honestly |
| Truth with `"slide": null` for a blank | One fewer key | Collides with "slide missing" errors; no room for future kinds | An explicit `kind` |
| Baseline as last measured numbers, fail on any change | Exact | Any harmless improvement or re-record fails CI | Thresholds with a ratchet rule |
| Raise "everything else" instead of a new row | One row | Hides the tool's size; ADR-010 says budgets only go down | User decision (3) |

## Consequences
### Positive
- Every conductor change (Q6) is measured on every OS in CI in seconds, without a model, against operator truth.
- A real recording from the kit becomes a fixture by adding `setlist.json`, a first slide row, any blank rows, and `session.jsonl` from `--full --record`.
- Q4-03 (blank timing), Q4-04 (late advances) and Q3-08 regressions (double advances) have numbers.
### Negative
- The stand-ins are spoken by a synthetic voice, not sung; their numbers say little about a congregation. They prove the pipeline, not the product.
- Replay measures the conductor on recorded recogniser output only; a recogniser change needs a local re-record (Whisper Tiny, about a minute per fixture in debug).
- `--full` needs a debug build on Windows for speed (Q4-01).
- A third binary: like `voxxad` (Q3-03), `tauri build` may bundle it; the shipping guard is still open (ticket Q5-03).
### Risks
- **Threshold drift:** someone loosens a threshold to pass. Mitigation: the update rule above; thresholds sit in one reviewed file.
- **Truth error:** hand-written truth can be wrong. Mitigation: stand-in truth is computed from segment offsets and measured onsets (method in `SOURCES.md`); the eval rejects rows outside the setlist and time going back.
- **Windows-recorded logs replayed on Linux/macOS:** already done in CI by `session-pd-hymns.jsonl` (ADR-013); CRLF checkouts are handled by line splitting.

## Dependencies
- Depends on ADR-013 (schema v1, sample clock, replay, `--shadow`), ADR-014 (hypothesis cadence; re-record rule), ADR-010 D3 (budget rows).
- Constrains Q6 (conductor tuning is judged by `voxxa-eval` against `fixtures/baseline.json`), the founder's recordings (Q5-01), and any future `--log-keys` (it should write this `truth.jsonl` shape).

## Review trigger
Re-open when real recordings replace the stand-ins (the tolerance and thresholds may need new values), when word times are logged (Q4-13: word-level latency), or if `voxxa-eval` needs more than 600 lines.
