# 08 — Recording intake: any .wav or video becomes numbers

| | |
|---|---|
| Run with | skill `voxxa-wav-intake` |
| Who | the tech lead runs the commands and taps the truth; the agent prepares commands and reads the numbers |
| Time [est.] | 20 minutes plus real-time tapping |
| Cost band | S |
| Inputs | a file in `<church_root>\inbox\`; `kit/templates/wav-intake-checklist.md`; `songs.csv`; signed consent forms (`kit/templates/recording-consent.template.md`) in `consent\`; `voxxa-eval` from the voxxa-tools zip (plan step V5c; until then only the voxxa founder's development booth); a checkout of the voxxa repository for tap-truth |
| Outputs | `recordings\<date>-<id>\audio.wav`, `source.md` and `intake.json`; optionally a fixture under `fixtures\tune\` or `fixtures\heldout\` with its row in `fixtures\manifest.json`; a numbers-only row in `reports\LICENSED_RUNS.md` |
| Acceptance | `audio.wav` is 16 kHz mono 16-bit; the evaluation prints a table of numbers; no word, lyric or path under a user profile appears in any output |
| Stops at | missing consent; any step that would need the agent to listen to, open or print the audio, the setlist, the truth or a session log |

## Prompt

> You are helping a church turn a recording into a voxxa test, with numbers as the only output. Read
> `kit/templates/wav-intake-checklist.md`, `kit/private-folder.md` and `<church_root>\songs.csv` (licence columns
> only).
>
> Rules: you never open, play, read or print the audio, `setlist.json`, `truth.jsonl`, any session log or any lyric.
> You give commands; the tech lead runs them and pastes back only file sizes, slide counts, exit codes and the
> table of numbers. Never send church data or recordings anywhere. Never move or copy a licensed recording off
> this PC.
>
> 1. Ask what the recording is (a rehearsal, a service, a song video), who is on it, and whether a signed consent
>    form (`kit/templates/recording-consent.template.md`) is in `consent\` for every voice, with the box for this
>    kind of recording ticked. No consent → stop.
> 2. Give the two `ffprobe` commands from the checklist; ask only for the sample rate, the channel count and the
>    duration. If the source has more than 2 channels (a desk multitrack), ask which channel carries the vocal or
>    main mix and use the checklist's `pan=mono|c0=c<N>` command (never `-ac 1`, which mixes in the band and
>    talkback); for a stereo or mono mix use the `-ac 1` command. Ask for the file size of
>    `recordings\<date>-<id>\audio.wav` only. Write
>    `source.md` with the tech lead: origin, consent reference, retention date, talk span times, licence classes of
>    the songs (from `songs.csv`).
> 3. If the recording will become a fixture: list the talk and prayer spans with the tech lead (start and end times
>    only) and give the ffmpeg command that replaces each span with silence.
> 4. Setlist: the tech lead builds `recordings\<date>-<id>\setlist.json` from the songs' `.txt` files in the order
>    sung with the BOM-free PowerShell snippet in voxxa's `fixtures/README.md` (for a service, a copy of the day's
>    `services\<date>-<slot>.setlist.json`); ask only for the printed slide count.
> 5. `intake.json`: the tech lead copies `kit/templates/intake.template.json` to
>    `recordings\<date>-<id>\intake.json` (next to `recordings\<date>-<id>\audio.wav`) and fills it with numbers and
>    licence classes only; help field by field.
> 6. Truth (only if it will be scored): tap-lag calibration once; then tap-truth by its full path in the voxxa
>    repository checkout, `<repo>\fixtures\tools\tap-truth.ps1 -Wav <church_root>\recordings\<date>-<id>\audio.wav`.
>    It writes `truth.jsonl` next to the wav it is given. Optional truth v2 rows follow
>    `kit/templates/truth-v2.example.jsonl`.
> 7. File the fixture (only with truth): copy `audio.wav`, `setlist.json`, `truth.jsonl` and `intake.json` into
>    `<church_root>\fixtures\tune\<date>-<id>\` and add its row and truth hash to `fixtures\manifest.json` (from
>    `kit/templates/fixtures-manifest.template.json`). Only the one-time held-out freeze in the checklist (§ 8)
>    puts fixtures in `fixtures\heldout\`: chosen by hand, before any tuning, and never changed afterwards.
> 8. Measure: `voxxa-eval --full --model <model> "<fixture folder>"`; read back the table. The tech lead adds one
>    numbers-only row to `reports\LICENSED_RUNS.md` (date, voxxa commit, licence class, credit line, the numbers);
>    you never open files under `reports\`.
> 9. Clean up: the original in `inbox\` is deleted once `recordings\<date>-<id>\audio.wav` exists; the
>    `recordings\` copy of the audio is
>    deleted once the fixture copy exists; all audio is deleted on the retention date.
>
> Finish with: the table of numbers and the retention date. Nothing else.
