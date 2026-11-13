# Fixtures

Recordings with ground truth, used to score auto-advance with `voxxa-eval`
(ADR-015), and a Planning Center sample for the importer.

```
fixtures/
  baseline.json              voxxa-eval thresholds; CI fails when a metric is worse
  recordings/<name>/         one eval fixture (one service, one "set"):
    setlist.json               the setlist voxxad reads (the one session.jsonl was recorded with)
    truth.jsonl                what the operator showed, and when
    session.jsonl              voxxad session log (Whisper Tiny, file speed, --shadow): the CI input
    audio.json                 stand-ins only: recipe that rebuilds audio.wav from committed WAVs
    audio.wav                  real recordings only; never committed (Drive, see SOURCES.md)
  tools/record-session.ps1   capture desk audio + slide-change truth
  setlists/pd-hymns.json     v1 setlist for the public-domain hymns (not the voxxad format)
  pco/plan-sample.json       anonymised Planning Center plan (no people)
  SOURCES.md                 where each fixture's audio comes from
```

Today `recordings/` holds two **stand-ins** built from synthetic speech
(see `SOURCES.md`). They prove the pipeline, not the product: real sung
recordings replace them (ticket Q5-01).

## Running the eval

Run from the repository root. `voxxa-eval` finds `voxxad` next to itself
(Cargo builds both into `target/<profile>/`), or takes `--voxxad <path>`.

```sh
cargo build --manifest-path src-tauri/Cargo.toml --bin voxxad --bin voxxa-eval
# Replay (what CI runs): no model, no audio.
src-tauri/target/debug/voxxa-eval --baseline fixtures/baseline.json
# Full: voxxad --stdin-audio <audio> --shadow with the real model.
src-tauri/target/debug/voxxa-eval --full --model tiny
# Full, and save each run's session log as the fixture's session.jsonl.
src-tauri/target/debug/voxxa-eval --full --model tiny --record
# Only some fixtures, or another folder.
src-tauri/target/debug/voxxa-eval --fixtures fixtures/recordings standin-paused-service
```

- **Replay** runs `voxxad --replay <fixture>/session.jsonl --setlist <fixture>/setlist.json`.
  It re-runs today's conductor on the recorded recogniser output, so a
  conductor change (Q6) is measured by replay alone, on every OS. A debug
  `voxxad` replays both stand-ins in under a minute.
- **Full** runs `voxxad --stdin-audio <wav> --setlist <fixture>/setlist.json --shadow --log-dir .`
  in a fresh temporary folder, so no session log (with its `window` lines
  of transcript text) is added to the roaming `%APPDATA%\voxxa\sessions`
  (Q3-05). It uses `<fixture>/audio.wav` if present, else it builds the
  WAV from `audio.json` in that folder. Use a **debug** build on Windows:
  a release build compiles whisper.cpp unoptimised (Q4-01). In debug a
  45 s fixture takes about two minutes.
- A **recogniser change** (window, hop, stitcher, model) changes what the
  conductor hears: re-record with `--full --model tiny --record`, check the
  table, and give the reason in the commit (ADR-014 Decision 6).
- `voxxa-eval` only ever passes `--replay` or `--shadow` to `voxxad`: it
  never builds a presenter and never opens a microphone.
- `--write-audio` writes each fixture's `audio.wav` from its `audio.json`
  (git-ignored), for listening.
- Exit status: **0** pass, **1** a metric is worse than `--baseline`,
  **2** an error (bad truth line, missing `session.jsonl`, unknown
  baseline key, `voxxad` failed).

## truth.jsonl

One JSON object per line, `t_ms` never decreasing. `t_ms` is milliseconds
from the first sample of the fixture's WAV: the same timeline `voxxad`
uses at file speed, where an action's `t` is audio time.

```json
{"t_ms":0,"event":"start","device":"Line In (UMC204HD)","song":"Amazing Grace","audio_offset_ms":1166}
{"t_ms":1620,"song":0,"slide":0}
{"t_ms":15620,"song":"Amazing Grace","slide":1}
{"t_ms":28900,"kind":"blank"}
{"t_ms":212400,"event":"stop"}
```

- A **slide row** says the operator showed `slide` (0-based within the
  song) of `song` (the 0-based index in `setlist.json`, or its title,
  case-insensitive; the recording kit writes titles).
- A **blank row** (`"kind":"blank"`) says the operator cleared the screen,
  e.g. when a prayer began.
- The operator's **unblank** is the first slide row after a blank row;
  there is no separate kind.
- Lines with an `event` key are metadata and are skipped.
- Before the first row the screen shows nothing (not scored).
- An unknown `kind`, a song or slide not in the setlist, or `t_ms` going
  back is an error naming the line.

## Metrics

Exact definitions (ADR-015 Decision 6). `E` = 750 ms tolerance.

| Term | Definition |
|---|---|
| Window | Truth row *i* owns `[t_i − E, t_{i+1} − E)`; the last row owns the rest. Before the first row's window, the expected screen is "nothing shown". |
| Voxxa's screen | Starts blank. `goto` shows that slide, `blank` blanks, `unblank` shows the last `goto` slide again. |
| Match | The first action in row *i*'s window that leaves the screen as row *i* says. If the screen already shows it when the window opens, the row is matched with no latency sample. |
| Double advance | A `goto` or `unblank` that leaves the screen unchanged (a goto to the slide already shown). Dispatch would drop it, but it is a regression of Q3-08. A `blank` on a blank screen is not scored. |
| False advance | Any other action that shows a slide: a wrong slide, an extra change after the match, lyrics during a blank row, a slide before the first row, or the right slide so late that the next row's window has opened. |
| False blank | Any other action that blanks the screen (e.g. the silence rule blanking a song part-way). |
| Missed slide / blank | A row with no match. |
| Precision | Slide rows matched by an action ÷ (those + false advances); 1.0 if Voxxa changed no slide. |
| Recall | Matched slide rows ÷ slide rows. |
| Latency | Action `t` − row `t_ms` for matched slide rows (can be down to −750). **Median** and **p95** by nearest rank: sort, take the value at rank ⌈p·n/100⌉. |
| Blank latency | Matching `blank` `t` − blank row `t_ms`; the table shows the worst. |
| Unblank latency | For a slide row *u* straight after a blank row *b*: the first action in `[t_b − E, end of u's window)` that turns a blank screen into a slide, minus `t_u`. Negative means lyrics came back during the prayer. Worst shown; none found is a missed unblank (`miss`). |
| False advances per set | False advances in one fixture (one fixture is one service). |

Precision, recall and latency are computed per fixture and pooled in the
`total` row. The table is plain text, one row per fixture plus `total`:

```
fixture                 slides   prec  recall  lat_med  lat_p95  false  double  missed  blanks  blank_ms  unblank_ms  false_blank
standin-paused-service       2  1.000   1.000     7380     7380      0       0       0     1/1      6100           -            0
```

Word-level timing is not possible yet: word times are not logged (Q4-13),
so latency is measured from the action line.

## baseline.json

Thresholds, not last numbers. Keys are a metric name plus `_min` or
`_max`; `per_set` is checked against every fixture row, `total` against
the total row. Metric names: `precision`, `recall`, `latency_median_ms`,
`latency_p95_ms`, `false_advances`, `double_advances`, `missed_slides`,
`missed_blanks`, `false_blanks`, `blank_latency_ms`, `unblank_latency_ms`,
`missed_unblanks`. An unknown key is an error (a typo must not switch a
check off). A latency with no sample is not checked; `recall` and the
`missed_*` thresholds cover that case.

**Setting:** counts and ratios at the worst value measured; latencies at
the worst measured rounded up to the next 500 ms.

**Updating:**
1. A change improves a number: tighten the threshold in the same commit (a ratchet).
2. A change makes a number worse on purpose: loosen it only with the
   reason in the commit message and a line under "Baseline history" below.
3. New or re-recorded fixtures: run `--full --model tiny --record`, then
   replay with `--baseline` and set any new values by the rule above.

CI runs the replay gate in the `check` job on Ubuntu, Windows and macOS
(`.github/workflows/ci.yml`, step "voxxa-eval (replay gate)").

### Baseline history
- 2026-11-13 (Q5): first thresholds on the two stand-ins. Measured:
  precision 1.000, recall 0.750 (verse 1 after the opening prayer is never
  shown), latency median and p95 7,380 ms (Q6's budget is p95 ≤ 2.5 s),
  worst blank 6,100 ms after the prayer starts, unblank 6,710 ms, no false
  or double advances and no false blanks.

## Recording a real fixture

1. Install ffmpeg once: `winget install Gyan.FFmpeg`
2. Find the input name: `.\fixtures\tools\record-session.ps1 -List`
   Use the desk feed or the USB audio interface, not the PC microphone.
3. Record:
   ```powershell
   .\fixtures\tools\record-session.ps1 -Device "<name from step 2>" `
       -Name amazing-grace-2026-11-01 -Song "Amazing Grace"
   ```
4. Watch the slides. Press **Enter** the moment each slide changes.
   `n` + Enter starts the next song (you type its title), `q` + Enter stops.
5. Make it an eval fixture in `recordings/<name>/`:
   - write `setlist.json` in the format `voxxad --setlist` reads
     (`{"setlist":[{"title":…,"slides":[{"id":0,"text":…}]}]}`, as in
     `recordings/standin-paused-service/setlist.json`), with the slides
     as they were shown. `setlists/pd-hymns.json` is the older v1 schema
     and does not load in `voxxad` (Q1 A15 follow-up);
   - add the first slide row by hand: the kit records only changes, so
     the slide on screen when recording started has no row
     (`{"t_ms":0,"song":"Amazing Grace","slide":0}` if it was already up);
   - add a `{"t_ms":…,"kind":"blank"}` row for each blank (the kit has no
     blank key yet, ticket Q5-02);
   - record the session log: `voxxa-eval --full --model tiny --record <name>`.
6. Commit only the JSON files. Put `audio.wav` on Drive and add a row to
   `SOURCES.md`.

Audio is 16 kHz, mono, 16-bit PCM WAV, about 1.9 MB per minute. `voxxad
--stdin-audio` takes only that format (Q3-10): convert anything else first,
`ffmpeg -i in.wav -ar 16000 -ac 1 -c:a pcm_s16le audio.wav`. Whisper
detects the language itself; there is no `--language` flag.

DirectShow takes about 1-1.6 s to open the device before ffmpeg records
anything. On stop, the script measures that gap (`audio_offset_ms` on the
start line) and shifts every `t_ms` so 0 is the first audio sample. The
remaining error is human reaction time (~200-400 ms late), which is why
the eval accepts an action up to 750 ms before the operator.

## Rules

- Public-domain songs only for committed fixtures (e.g. "Amazing Grace",
  "Be Thou My Vision"). No CCLI-licensed lyrics in git.
- No audio in git: `fixtures/` holds only JSON, this README, `SOURCES.md`
  and the kit. Stand-in audio is rebuilt from `audio.json`; real audio
  goes to Google Drive `Voxxa/fixtures/`, linked in `SOURCES.md`.
- `session.jsonl` holds every transcript (`hypothesis` and `window`
  lines). Commit it only for public-domain fixtures. Session logs that
  `voxxad` writes by default sit in the roaming `%APPDATA%\voxxa\sessions`
  for 14 days; on a domain network with roaming profiles they are copied
  to the server (ADR-013).
- Pilot-church recordings stay on the church's machine and one encrypted drive
  (research plan §6), are never committed or uploaded, and are deleted once
  reduced to lyrics-only timing data.
