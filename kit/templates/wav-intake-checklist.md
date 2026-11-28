# Recording intake checklist (any .wav, .mp3, .m4a or video)

Turns any recording into something voxxa can be measured on. The output is a table of numbers. Nobody,
person or agent, prints the words heard or the lyrics.

Needs `voxxa-eval` and `voxxad` from the voxxa-tools zip (plan step V5c; see `kit/README.md` "Tools and when they
arrive"). Until V5c ships, only the voxxa founder's development booth can run § 9. Tap-truth (§ 6) needs only
a checkout of the voxxa repository.

Recording: ______________  Received: __________  Retention date (delete the audio by): __________

Every command below runs in PowerShell **from `<church_root>`**. The one exception is tap-truth (§ 6): it is a
script in the voxxa repository checkout on the booth PC, not in `<church_root>`, so it is called by its full path.
Start with:
```powershell
Set-Location "<church_root>"      # a new church: Set-Location "$env:LOCALAPPDATA\voxxa-church"
```
`<date>-<id>` names this recording, for example `2026-11-29-r1`.

## 1. Is it allowed?
- [ ] Consent from everyone whose voice is on it (singers, pastor, readers), on the kit's form
      `templates/recording-consent.template.md`, filed in `consent\`, with the "Audio recordings kept until" box
      ticked; a service recording also needs the "Services" box
- [ ] The songs on it are in `songs.csv` with a licence; note the licence classes: ______________
- [ ] Talk and prayer: will be replaced with silence before the recording becomes a test fixture

## 2. Look at the source (numbers only)
```powershell
ffprobe -v error -select_streams a:0 -show_entries stream=sample_rate,channels -of csv=p=0 "inbox\<file>"
ffprobe -v error -show_entries format=duration -of csv=p=0 "inbox\<file>"
```
- [ ] Sample rate and channel count written down (e.g. `48000,8`): ______  Duration (s): ______
- [ ] More than 2 channels (a desk multitrack): ask the sound tech which channel carries the **vocal or main
      mix**. ffmpeg counts channels from 0, so desk channel 4 is `c3`. Channel used: `c___`

## 3. Convert to 16 kHz mono
**Desk multitrack:** take the one channel. Never `-ac 1` here: it mixes every channel, band and talkback
included.
```powershell
New-Item -ItemType Directory -Force recordings\<date>-<id> | Out-Null
ffmpeg -hide_banner -loglevel error -i "inbox\<file>" -vn -af "pan=mono|c0=c<N>" -ar 16000 -c:a pcm_s16le recordings\<date>-<id>\audio.wav
```
**Stereo or mono mix** (a video, a phone, the desk's 2-track main mix): fold to mono.
```powershell
New-Item -ItemType Directory -Force recordings\<date>-<id> | Out-Null
ffmpeg -hide_banner -loglevel error -i "inbox\<file>" -vn -ar 16000 -ac 1 -c:a pcm_s16le recordings\<date>-<id>\audio.wav
```
- [ ] `recordings\<date>-<id>\audio.wav` is 16 kHz, mono, 16-bit (the only format voxxa reads); a 3-minute file
      is about 5.7 MB
- [ ] `recordings\<date>-<id>\source.md` written: where it came from, consent reference, retention date, the
      talk span times (no lyrics)
- [ ] If kept as a fixture: talk spans silenced. Run the convert command again with `-y` (overwrite) and one
      `volume=enable='between(t,START,END)':volume=0` per span (times in seconds) in the `-af` value: after the
      pan, joined by commas, e.g. `-af "pan=mono|c0=c3,volume=enable='between(t,600,742)':volume=0"`; for a
      stereo mix, add `-af "volume=enable='between(t,600,742)':volume=0"`

## 4. The setlist for this recording
- [ ] `recordings\<date>-<id>\setlist.json` built from the songs' `.txt` files in the order sung, with the
      BOM-free PowerShell snippet in voxxa's `fixtures/README.md` (it prints only "setlist.json: N slides").
      For a service, copy the day's compiled `services\<date>-<slot>.setlist.json` instead

## 5. intake.json (numbers and licence classes only)
- [ ] Copy `kit\templates\intake.template.json` to `recordings\<date>-<id>\intake.json` and fill it: duration,
      sample rates, channels, channel used (`c<N>` or `mix`), talk spans muted (count), each song's position,
      slide count and licence class, the consent file names, the retention date. No titles, no words

## 6. Truth (only if this recording will be scored)
- [ ] Tap-lag calibrated once on a text-to-speech fixture (the tapper's average lag is subtracted)
- [ ] Run tap-truth **from the voxxa repository checkout** on the booth PC (`<repo>` is its folder). It writes
      `truth.jsonl` **next to the wav it is given** (here `recordings\<date>-<id>\`) and stops if one is already there:
```powershell
powershell -ExecutionPolicy Bypass -File "<repo>\fixtures\tools\tap-truth.ps1" -Wav "<church_root>\recordings\<date>-<id>\audio.wav"
```
- [ ] Enter when each slide's first word begins; a number + Enter for a repeated slide; `b` for a blank; `n` for
      the next song; `q` to stop
- [ ] Optional truth v2 rows, typed by the tech lead after tapping: the talk spans from `source.md` as
      `{"t_ms":…,"event":"talk_start"}` and `{"t_ms":…,"event":"talk_end"}`, and `{"t_ms":…,"event":"song_end"}`
      rows (format: `kit\templates\truth-v2.example.jsonl`). Today's voxxa-eval skips event rows; `last_ms` comes
      later from a review pass

## 7. File the fixture (only a recording with truth)
- [ ] New fixtures go to `fixtures\tune\`. Only the one-time hand choice in § 8 moves fixtures to
      `fixtures\heldout\`; after the freeze every new recording stays in `tune`
- [ ] Add any truth v2 rows (§ 6) **before** this step: the hash below must not change afterwards
- [ ] Copy the four files into the fixture folder:
```powershell
$dst = "fixtures\tune\<date>-<id>"
New-Item -ItemType Directory -Force $dst | Out-Null
Copy-Item recordings\<date>-<id>\audio.wav, recordings\<date>-<id>\setlist.json, recordings\<date>-<id>\truth.jsonl, recordings\<date>-<id>\intake.json $dst
```
- [ ] Add one row to `fixtures\manifest.json` (from `kit\templates\fixtures-manifest.template.json` the first
      time): id, split, songs, truth rows, `truth_sha256`, date added. The hash, printed with the folder name only:
```powershell
Get-ChildItem fixtures\tune, fixtures\heldout -Directory | ForEach-Object { "{0} {1}" -f $_.Name, (Get-FileHash -Algorithm SHA256 (Join-Path $_.FullName "truth.jsonl")).Hash.ToLower() }
```
- [ ] Delete `recordings\<date>-<id>\audio.wav` once the fixture copy exists (one copy of the audio is enough;
      `source.md` stays)

## 8. Freeze the held-out set (once, before any tuning)
- [ ] Do this before the first tuning run (for the founder's corpus: before plan step V6), and never again
- [ ] Choose by hand the fixtures that hold the held-out songs: at least 4 songs, stratified: at least one slow and one fast song, at
      least one with an instrumental gap of 6 s or more, at least one sung in a different order from the plan
      (a repeat or a skip). Write down why each was chosen (numbers and song positions only)
- [ ] Move those fixture folders (`Move-Item fixtures\tune\<date>-<id> fixtures\heldout\`) and set their `split`
      to `heldout` in `manifest.json`; every other fixture stays in `fixtures\tune\`
- [ ] Run the hash command above and record each held-out `truth_sha256` in `manifest.json`; set
      `heldout_frozen_at` to today and `look_count` to 0
- [ ] After the freeze: never re-tap, edit, add or move a held-out fixture. A held-out fixture found to be wrong
      may move to `tune` (never the other way), with a note in `manifest.json`
- [ ] Each held-out run (at most one per milestone): run the hash command first and compare with the manifest; a
      changed hash stops the run. Then add a `heldout_looks` row (date, milestone, voxxa commit) and raise
      `look_count` by 1. Only the aggregate numbers are reported

## 9. Measure (numbers only)
```powershell
voxxa-eval --full --model small "<church_root>\fixtures\tune\<date>-<id>"
```
- [ ] One numbers-only row added to `reports\LICENSED_RUNS.md` (licence class, credit line, numbers; no words, no paths)
- [ ] Without truth: run `voxxad --stdin-audio recordings\<date>-<id>\audio.wav --setlist recordings\<date>-<id>\setlist.json --shadow --model small --log-dir <a temp folder>`
      and keep only the counts of moves and blanks

## 10. Clean up
- [ ] The original in `inbox\` deleted once `recordings\<date>-<id>\audio.wav` exists
- [ ] The audio deleted on the retention date (in `fixtures\` and `recordings\`); the numbers stay
