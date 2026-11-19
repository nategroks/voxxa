# Fixtures

Recordings with ground truth, used to score auto-advance with `voxxa-eval`
(Q5, ADR-015), and Planning Center import samples (P3.5).

```
fixtures/
  baseline.json              the eval gate's thresholds (CI fails below them)
  recordings/<name>/         one fixture per folder:
    setlist.json             the setlist voxxad follows (--setlist format)
    truth.jsonl              what the operator did, and when
    session.jsonl            a recorded voxxad session log (Whisper Tiny)
    audio.json               recipe that rebuilds audio.wav from committed WAVs
    audio.wav                the audio (never committed; fixtures/.gitignore)
  tools/record-session.ps1   capture desk audio + slide-change truth
  setlists/pd-hymns.json     the two public-domain hymns, older v1 schema
                             (voxxad does not load it; Q1 A15 follow-up)
  pco/plan-sample.json       anonymised Planning Center plan (no people)
  SOURCES.md                 where each fixture's audio comes from
```

`fixtures/` commits no audio: a fixture is JSON only. The WAVs an
`audio.json` joins are committed test audio in `src-tauri/tests/fixtures/voxxad/`
(the Q2 WAVs, and Q6's `tts-*.wav` made by
`scripts/fixtures/make_tts_fixture.ps1`; see "Synthetic TTS fixtures" below).

## Running the eval

From the repository root (Windows, macOS or Linux):

```sh
cargo build --manifest-path src-tauri/Cargo.toml --features cli --bin voxxad --bin voxxa-eval
cargo run --manifest-path src-tauri/Cargo.toml --features cli --bin voxxa-eval            # replay (CI)
cargo run --manifest-path src-tauri/Cargo.toml --features cli --bin voxxa-eval -- --full  # Whisper Tiny
```

Both tools need the `cli` feature (off by default, so the installers leave
them out; Q7). The tests that run them need it too:
`cargo test --manifest-path src-tauri/Cargo.toml --features cli`, as CI runs it.
CI's separate "Full eval" workflow (`.github/workflows/full-eval.yml`, run
by hand) runs `--full` with a checksum-pinned Whisper Tiny on Linux, macOS
and Windows and only reports the table; see `docs/runbooks/eval-regression.md`.

`voxxa-eval` runs the `voxxad` binary built beside it on every folder in
`fixtures/recordings` (or the folders given as arguments), reads the
`action` lines it prints, scores them against `truth.jsonl`, prints one table
row per fixture plus a total, and checks each row against `baseline.json`.

- **Replay (default, CI):** `voxxad --replay session.jsonl --setlist setlist.json`.
  No model, no audio, the same result on every OS. It re-runs today's
  conductor on the recorded recogniser output, so a conductor-logic change
  is measured by replay alone. But replay rebuilds the conductor from the
  `SmartConfig` stored in each log's start line (`info.config`), not from
  today's defaults: a change to a `SmartConfig` default is not seen by
  replay (or CI) until the logs are re-recorded, or their start-line config
  is edited (ticket Q5-07, #78). Fields missing from a log's start-line
  config take today's defaults, but since Q7 all six logs store the whole
  `SmartConfig`, the ADR-016 `evidence` block included. So a change to any
  `SmartConfig` or `EvidenceConfig` default moves no replay until the logs
  are re-recorded: such a PR must re-record them (below).
- **Determinism (CI on three OSes, `src-tauri/tests/replay_determinism.rs`):**
  every committed log must replay to its own `action` lines, byte for byte,
  and two replays of it must agree. So a conductor PR that changes actions
  must re-record the affected logs with `voxxa-eval --full --record
  fixtures/recordings/<name>`, and its commit must say why; their recogniser
  lines stay the same. `docs/runbooks/eval-regression.md` tells such a
  re-record from a real regression.
- **Full (`--full`, local):** `voxxad --stdin-audio audio.wav --setlist setlist.json
  --shadow --model tiny`, at file speed. When the fixture has an `audio.json`,
  `audio.wav` is rebuilt from it first. Needs `ggml-tiny.bin` (download it in
  the app). A debug build takes about 2 s per second of audio; a release
  build on Windows is much slower (Q4-01). Whisper's own log (the words it
  hears) goes to stderr; keep only `voxxa-eval`'s lines with
  `2> >(grep '^voxxa-eval' >&2)` (bash), as `full-eval.yml` does.
- **Re-recording (`--full --record`):** also saves the session log as the
  fixture's `session.jsonl`. `voxxad` runs in a temporary folder with
  `--log-dir .`, so the `log` path on the start line is relative
  (`.\session-<time>.jsonl` on Windows) and names no user folder.
  Re-record after any recogniser change (window, hop, stitcher, model) and say
  why in the commit; then check the table and `baseline.json`.
- Exit status: 0 when no metric is worse than the baseline, 1 when one is
  (each failure is printed as `FAIL <row>: <metric> is <value>, baseline needs …`),
  2 on an error (a missing file, a bad line, `voxxad` failing).
- `--baseline <file>` checks against another thresholds file.

## truth.jsonl

One JSON object per line, in time order. `t_ms` is milliseconds from the
first sample of `audio.wav`, the same timeline as `voxxad`'s `t` at file speed.

```json
{"t_ms":2100,"song":0,"slide":0}
{"t_ms":16100,"song":"Amazing Grace","slide":1}
{"t_ms":30400,"blank":true}
{"t_ms":40500,"song":0,"slide":1}
```

- `{"t_ms","song","slide"}`: the operator shows a slide. `song` is the 0-based
  index in `setlist.json` or its exact title; `slide` is the 0-based slide in
  that song (as in `voxxad`'s `goto` lines).
- `{"t_ms","blank":true}`: the operator blanks the screen. The slide row after
  a blank is the unblank (to show the same slide again, repeat its slide row).
- Lines with an `event` key (`record-session.ps1` metadata) are skipped.

## Metrics

Truth row *i* owns the actions with `t` in `[t_i - 1000, t_{i+1} - 1000)`
(the last row owns everything after `t_i - 1000`). So an action may be up
to 1 s early (operators press 200-400 ms late), and a late action matches
until 1 s before the operator's next change; from then on it belongs to the
next row. Every truth row is matched by at most one action: the first `goto`
(for a slide row) or `blank` (for a blank row) in its span that shows the
same thing. There is no "already showing" rule: a row whose slide was
already on screen still needs its own `goto`.

| Column | Definition |
|---|---|
| slides | truth slide rows |
| gotos | `goto` actions |
| hits | gotos that matched a slide row |
| missed | slide rows no goto matched |
| precision | hits / gotos (1.00 when there are no gotos) |
| recall | hits / slides (1.00 when there are no slide rows) |
| median ms, p95 ms | latency of the hits: action `t` minus truth `t_ms`, nearest rank (`-` when there are no hits) |
| false/set | false advances in the fixture (one fixture is one set): a goto or blank that matched no row and is not a double |
| double | a goto to the slide already shown with no blank since, or a blank while blank: the screen would not change (dispatch drops it); any is a regression (Q3-08) |
| blank ms | per truth blank: the matched blank's `t` minus `t_ms`; the worst is shown, `miss` if any truth blank had none, `-` if there are none |
| unblank ms | per truth blank followed by a slide row: the first `goto` or `unblank` after `voxxad`'s matched blank, minus that slide row's `t_ms`; worst shown, `miss` when `voxxad` did not blank or never showed anything again |

The total row sums the counts (so its false/set is the sum over all sets)
and computes the ratios, percentiles and worst timings over all fixtures'
actions together. Explicit `unblank` actions count only for unblank timing:
they are never counted as false or double advances, even when one re-shows
lyrics in the middle of a prayer. That is a known gap; the conductor emits
`unblank` rarely and neither committed log has one.
Word timing is not measured: word times are rough and not logged (Q4-13).

## baseline.json

`{"<fixture or total>": {"<metric>": <worst accepted value>}}`. Metric names:
`precision`, `recall` (minimums), `median_ms`, `p95_ms`, `false_advances`,
`double_advances`, `blank_ms`, `unblank_ms` (maximums). Every row needs an
entry: a new fixture without thresholds fails, and so does an unknown metric
name. Every metric that has a value in a row needs a threshold there; only a
metric shown as `-` (nothing to measure, such as `unblank_ms` when no slide
follows the blank) may be left out, so dropping a threshold fails instead of
switching its gate off. A `miss` or `-` fails any threshold set for that
metric.

These are thresholds, not the last numbers. They start at the first replay's
values: each latency (median, p95, blank, unblank) is the next multiple of
500 ms strictly above the measured worst (1,500 becomes 2,000; 2,900
becomes 3,000), headroom for hop jitter when a log is re-recorded; counts
and ratios are exact (a ratio with no short decimal form, such as 13/14, is
cut to three decimals: 0.928). Update them:

- **Tighten** in the same commit as a change that improves a metric (Q6 aims
  at p95 latency of at most 2,500 ms).
- **Loosen** only with the reason in the commit message (a re-recorded log,
  a new fixture shape), never to make an unexplained regression pass.
- Adding a fixture: run the eval, then add its row and update `total`.
  The rows of the fixtures already there are never loosened. `total` is a
  different measurement once the set changes, so it is set from the new
  set's numbers by the same rule; say each changed total threshold in the
  commit message (Q6 did this when it added the three TTS fixtures).

The gate checks the full set; running on a subset changes the total row.

## The current fixtures

All six are stand-ins built from committed public-domain WAVs (synthetic
speech, not singing; see `SOURCES.md`). Truth rows are at the first word of
each part: for the Q5 fixtures (and `prayer-before-first-song`) the start of
the part plus 100 ms; for the three Q6 TTS fixtures the synthesizer's own
first-word position (below), which is also 100 ms into each of their WAVs.

| Fixture | Shape | What it measures |
|---|---|---|
| `prayer-then-reprise` | 2 s silence, Amazing Grace v1, 2 s, v2, 4 s, a prayer, 2 s, v2 again, 4 s; setlist order AG, BTMV, IIW; truth uses indices | blank after a pause (silence rule), unblank on the reprise, a late advance into v2 (Q4-04) |
| `prayer-without-pause` | 1 s, v1, 1 s, v2, the prayer straight after, 6 s; setlist reordered (AG is song 1); truth uses titles, as the recorder writes them | blank during a prayer with no pause (Q4-03) |
| `refrain-it-is-well` | Zira; 2 s, It Is Well v1, 1 s, refrain, 1.5 s, v2, 1 s, refrain, 1.5 s, v3, 1 s, refrain, 4 s (73.3 s); setlist IIW (v1, refrain, v2, v3, v4), HHH, BTMV; IIW carries section labels (`sections[].kind`: verse, refrain, verse, verse, verse; ADR-016), so its log was re-recorded in Q6; truth slides 0, 1, 2, 1, 3, 1 | a repeated chorus: the refrain is one slide shown three times, so the operator goes back to it after each verse |
| `jump-be-thou-my-vision` | David; 2 s, Be Thou My Vision v1, 1.5 s, v3, 1.5 s, v4, 1.5 s, v1 again, 4 s (59.3 s); setlist BTMV, CTF, AG; truth slides 0, 2, 3, 0 | jumps: a skipped verse (v2) and a return to the first verse |
| `prayer-before-first-song` | 1 s, the prayer, 2 s, Amazing Grace v1, the prayer straight after, 1 s, v2, 3 s (Q5-04's reproduction from the Q5 10x lane, commit 30e8990); setlist AG, BTMV, IIW; truth slide 0 at 11,200, blank at 23,200, slide 1 at 32,290 | an opening prayer before the first song (Q5-04: verse 1 was never shown) and a prayer between two verses |
| `prayer-between-songs` | 2 s, Holy, Holy, Holy v1, 1.5 s, v2 (David), 1 s, a prayer (Zira), 2 s, Come Thou Fount v1, 1.5 s, v2 (Zira), 4 s (94.0 s); setlist HHH, CTF, IIW; truth song 0 slides 0, 1, blank, song 1 slides 0, 1 | a set: blank in a prayer with only a short breath before it, then the next song after the prayer (Q5-04) |

### Synthetic TTS fixtures

`scripts/fixtures/make_tts_fixture.ps1` (Windows PowerShell 5.1, offline
`System.Speech`, voices "Microsoft David Desktop" and "Microsoft Zira
Desktop") makes the three Q6 fixtures. It reads each slide's text from the
fixture's `setlist.json`, speaks it line by line (a 300 ms break between
lines) into `src-tauri/tests/fixtures/voxxad/tts-<song>-<part>-<voice>.wav`
at 16 kHz mono 16-bit (the output is a WAV file, never the speakers), and
writes the fixture's `audio.json` and `truth.jsonl`. A part repeated in a
fixture (the refrain, the return to v1) reuses its WAV. Re-run it with
`powershell -ExecutionPolicy Bypass -File scripts/fixtures/make_tts_fixture.ps1`
(add `-Fixture <name>` for one); on the same PC it writes the same bytes.

**Truth times come from the synthesizer, not from listening:**

- A slide row's `t_ms` is the part's offset in `audio.wav` (the silences and
  WAV data chunks before it, in samples, exactly as `voxxa-eval` joins them)
  plus the audio position of the part's first word: the earliest
  `SpeakProgress` event's `AudioPosition`. Rounded to the nearest ms.
- A blank row is at the first word of a prayer (the same rule). For a silence
  between songs with no prayer (a silence step marked `blank`), the row is
  where that silence starts: the end of the previous part's WAV.
- No other rows: the operator does nothing during the 4 s at the end.

The synthesizer's first-word position (100 ms into every Q6 WAV) agrees with
the WAVs' energy onset (100-130 ms above an amplitude of 300) to within 30 ms.
After running the script, record each fixture's `session.jsonl` with
`voxxa-eval --full --record fixtures/recordings/<name>` and add its
`baseline.json` row.

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
5. Fix up `truth.jsonl` by hand (ticket Q5-02):
   - add the first slide row: the kit records only changes and puts the
     first song only on its `start` event line, which the eval skips, so the
     slide on screen when recording started has no row. Without it every
     real recording scores a false advance on its first slide. Add
     `{"t_ms":<when it went up>,"song":"Amazing Grace","slide":0}`
     (`0` if it was already up);
   - add a `{"t_ms":…,"blank":true}` row for each blank: the kit has no
     blank key.
6. Write `recordings/<name>/setlist.json` in the format `voxxad --setlist`
   reads (`{"setlist":[{"title":…,"slides":[{"id":0,"text":…}]}]}`, as in
   `recordings/prayer-then-reprise/setlist.json`), with the slides as they
   were shown, in order: truth `slide` counts Enter presses. The kit's
   earlier step 5 said to copy songs from `setlists/pd-hymns.json`; that
   file uses the older v1 setlist schema, which `voxxad` does not load
   (Q1 A15 follow-up), so copy the text, not the file.
7. Run `voxxa-eval --full --record fixtures/recordings/<name>` to write its
   `session.jsonl`, then add its thresholds to `baseline.json`.

The script writes 16 kHz mono 16-bit PCM, the only format `voxxad` reads.
Convert any other recording first:
`ffmpeg -i in.wav -ar 16000 -ac 1 -c:a pcm_s16le audio.wav` (Q3-10).
About 1.9 MB per minute.

DirectShow takes about 1-1.6 s to open the device before ffmpeg records
anything. On stop, the script measures that gap (`audio_offset_ms` on the
start line) and shifts every `t_ms` so 0 is the first audio sample. The
remaining error is human reaction time (~200-400 ms late), which the 1 s
early tolerance above covers.

## Rules

- Public-domain songs only for committed fixtures (e.g. "Amazing Grace",
  "Be Thou My Vision"). No CCLI-licensed lyrics in git.
- No audio in git: `audio.wav` is ignored. Keep a real recording's audio on
  Google Drive `Voxxa/fixtures/` and record the link in `SOURCES.md`.
- Pilot-church recordings stay on the church's machine and one encrypted drive
  (research plan §6), are never committed or uploaded, and are deleted once
  reduced to lyrics-only timing data.
- A session log holds every transcript, prayers and spoken names included
  (with Q4's `window` lines, several times the text on stdout). Commit only
  logs of public-domain or synthetic audio. `voxxad`'s own logs go to
  `<data dir>/voxxa/sessions`, on Windows the roaming `%APPDATA%`, which a
  domain profile may copy to a server (`--log-dir` moves them);
  `voxxa-eval --full` sends them to a temporary folder instead.
