# Fixtures

Real-world recordings with ground truth, used to test auto-advance (P1.6) and
Planning Center import (P3.5).

```
fixtures/
  tools/record-session.ps1   capture desk audio + slide-change truth
  recordings/<name>/         audio.wav (not committed if > 25 MB), truth.jsonl, setlist.json
  setlists/pd-hymns.json     v1 setlist for the two public-domain hymns
  pco/plan-sample.json       anonymised Planning Center plan (no people)
  SOURCES.md                 Google Drive links for audio kept out of git
```

## Recording a fixture

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
5. Copy the song(s) you sang from `setlists/pd-hymns.json` into
   `recordings/<name>/setlist.json` (v1 schema from P1.3) and edit the verses
   to match what was actually sung.

Audio is 16 kHz, mono, 16-bit PCM WAV, about 1.9 MB per minute, so a single
hymn fits under 25 MB but a full service will not.

## truth.jsonl

One JSON object per line. `t_ms` is milliseconds since ffmpeg was launched.
`slide` is the 0-based index within the song across all its sections: the
slide on screen when the song starts is 0, and the first Enter gives 1.
Lines with an `event` key are metadata and should be skipped by eval.

```json
{"t_ms":0,"event":"start","device":"Line In (UMC204HD)","song":"Amazing Grace","audio_offset_ms":1166}
{"t_ms":14250,"song":"Amazing Grace","slide":1}
{"t_ms":31870,"song":"Amazing Grace","slide":2}
{"t_ms":212400,"event":"stop"}
```

DirectShow takes about 1-1.6 s to open the device before ffmpeg records
anything. On stop, the script measures that gap (`audio_offset_ms` on the
start line) and shifts every `t_ms` so 0 is the first audio sample. The
remaining error is human reaction time (~200-400 ms late), so score
auto-advance with a tolerance window of at least ±750 ms, not exact matches.

## Rules

- Public-domain songs only for committed fixtures (e.g. "Amazing Grace",
  "Be Thou My Vision"). No CCLI-licensed lyrics in git.
- Audio over 25 MB goes to Google Drive `Voxxa/fixtures/`; record the link in
  `SOURCES.md` and commit only the JSON.
- Pilot-church recordings stay on the church's machine and one encrypted drive
  (research plan §6), are never committed or uploaded, and are deleted once
  reduced to lyrics-only timing data.

## Relation to P1.6

P1.6 plans `voxxad --log-keys` for operator timestamps. Until `voxxad` exists,
`tools/record-session.ps1` writes the same `{t_ms, song, slide}` lines that
`voxxa-eval` will read.
