---
name: voxxa-wav-intake
description: Turn any recording a church or the voxxa founder provides (.wav, .mp3, .m4a or video) into a voxxa test - convert to 16 kHz mono, silence talk spans, build the setlist, tap the truth, and run voxxa-eval - with a table of numbers as the only output. Use when a recording arrives in a church's inbox folder.
---

# voxxa recording intake

Runs `kit/prompts/08-wav-intake.md`.

## Safety rules (every step)

- Never open, play, read or print the audio, `setlist.json`, `truth.jsonl`, a session log or any lyric. The
  person runs every command and pastes back only file sizes, slide counts, exit codes and the numbers table.
- Consent from every voice on the recording (singers, pastor, readers), on the kit's form
  `kit/templates/recording-consent.template.md` and filed in `consent\`, is on file before anything else; without
  it, stop.
- `voxxa-eval` comes from the voxxa-tools zip (plan step V5c); until it ships, measuring runs only on the voxxa
  founder's development booth.
- Talk and prayer spans are replaced with silence before a recording becomes a test fixture; only the span times
  are kept.
- Never move, copy or upload a recording off the booth PC; never commit anything from `<church_root>`.
- Never send church data to any external service or connector.

## Steps

1. Confirm consent and the licence classes of the songs on it (from `songs.csv`).
2. Check the source with `ffprobe` (sample rate, channels, duration only). Convert with ffmpeg to
   `recordings\<date>-<id>\audio.wav` (16 kHz, mono, 16-bit PCM): one desk channel with `-af "pan=mono|c0=c<N>"`
   for a multitrack (never `-ac 1`, which mixes in band and talkback), `-ac 1` for a stereo or mono mix. Write
   `source.md` (origin, consent reference, retention date, talk span times).
3. If it becomes a fixture: silence the talk spans.
4. Setlist with the BOM-free PowerShell snippet (only the slide count is read back).
5. `intake.json` from `kit/templates/intake.template.json`: numbers and licence classes only.
6. Truth by tapping, after a one-time tap-lag calibration: `<repo>\fixtures\tools\tap-truth.ps1` from the voxxa
   repository checkout, which writes `truth.jsonl` next to the wav it is given.
7. File the fixture: copy `audio.wav`, `setlist.json`, `truth.jsonl` and `intake.json` from
   `recordings\<date>-<id>\` into `<church_root>\fixtures\tune\<date>-<id>\`; add its row and truth hash to `fixtures\manifest.json`. Only the
   checklist's one-time held-out freeze (§ 8) puts fixtures in `fixtures\heldout\` (chosen by hand, before
   tuning, never changed).
8. `voxxa-eval --full --model <model> <fixture>`; the person adds one numbers-only row to
   `reports\LICENSED_RUNS.md` (you never open files under `reports\`).
9. Delete the inbox original and the `recordings\` copy of the audio once the fixture exists; note the retention
   date for the audio.

## Output contract

The numbers table (recall, precision, median and p95 delay, wrong moves, blanks; and, when voxxa-eval has them,
on-time share and song-start delay), the fixture's set (tune or held-out) and the retention date. No words.
