# Fixture sources

No audio is committed under `fixtures/`. Each eval fixture's audio is either
rebuilt from committed files (stand-ins) or kept on Drive (real recordings),
in [Voxxa/fixtures](https://drive.google.com/drive/folders/1yT-xiTOLxz67BeebUFdQYtgxqpIHpOXf)
(private to the founder; share per person).

## Stand-ins (synthetic speech, not sung recordings)

The two fixtures in `recordings/` are **stand-ins**. There are no founder
recordings yet, so they are built from three short WAVs already committed
for the `voxxad` tests (`src-tauri/tests/fixtures/voxxad/`):

| WAV | Content | Length | Licence |
|---|---|---|---|
| `amazing-grace-v1.wav` | Amazing Grace, verse 1 (John Newton, 1779), read by the Windows SAPI voice | 12.0 s | public-domain text; synthetic speech |
| `amazing-grace-v2.wav` | Amazing Grace, verse 2, same voice | 10.3 s | public-domain text; synthetic speech |
| `prayer.wav` | a short prayer written for Q2, same voice | 8.1 s | written for this project |

They are spoken, not sung, by one synthetic voice in a quiet room, so their
numbers say little about a congregation, a band or a hall. They prove the
eval pipeline and give the CI gate something to hold. **Real recordings of
the founder's two public-domain hymns replace them** (ticket Q5-01); the
thresholds in `baseline.json` are then set again.

| Fixture | Built from (`audio.json`) | Length | Truth |
|---|---|---|---|
| `standin-paused-service` | 1.5 s silence, verse 1, 2 s, verse 2, 3 s, prayer, 4 s | 40.9 s | slide 0/0 at 1,620 ms; slide 0/1 at 15,620; blank at 28,900 |
| `standin-prayer-between` | 1 s silence, prayer, 2 s, verse 1, prayer (no pause), 1 s, verse 2, 3 s | 45.5 s | nothing during the opening prayer; slide 0/0 at 11,200 ms; blank at 23,200; slide 0/1 at 32,290 |

- **Rebuilding the audio:** `voxxa-eval --write-audio` (or `--full`, which
  builds it in a temporary folder) concatenates the parts sample for
  sample into a 16 kHz mono 16-bit WAV. The output depends only on the
  committed WAVs, so it is the same on every machine (checked by
  `src-tauri/tests/eval.rs`).
- **How the truth was set:** as an operator would, a slide row when a
  verse starts and a blank row when the prayer starts. Each time is the
  segment's start in the recipe plus the first sample above 2 % of full
  scale in that WAV (121 ms into verse 1 and the prayer, 130 ms into
  verse 2), rounded down to 10 ms. It was set before any run and not
  tuned to the results.
- **Setlist:** `setlist.json` is a copy of
  `src-tauri/tests/fixtures/voxxad/pd-hymns-setlist.json` (Amazing Grace,
  Be Thou My Vision, It Is Well with My Soul; all public domain).
- **Session logs:** `session.jsonl` was recorded on 2026-11-13 with
  `voxxa-eval --full --model tiny --record` (Whisper Tiny, debug build,
  Windows 11, file speed, `--shadow`).

## Real recordings

| Fixture | Song(s) | Recorded | Device | Drive link | Licence |
|---|---|---|---|---|---|
| _none yet (Q5-01)_ | | | | | |
