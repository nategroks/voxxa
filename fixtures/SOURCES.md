# Fixture sources

No audio is committed. A fixture's `audio.wav` is either rebuilt from its
`audio.json` recipe (committed WAVs joined with silences, by `voxxa-eval --full`)
or, for a real recording, kept on Drive in
[Voxxa/fixtures](https://drive.google.com/drive/folders/1yT-xiTOLxz67BeebUFdQYtgxqpIHpOXf)
(private to the founder; share per person).

| Fixture | Song(s) | Audio | Recorded | Licence |
|---|---|---|---|---|
| `recordings/prayer-then-reprise` | Amazing Grace v1, v2, v2 again; a spoken prayer | **Stand-in.** `audio.json` joins `src-tauri/tests/fixtures/voxxad/amazing-grace-v1.wav`, `amazing-grace-v2.wav` and `prayer.wav` | Synthetic speech: the hymn text read by the Windows SAPI voice; the prayer written for Q2 | Hymn text public domain (John Newton, 1779); prayer and synthetic audio written for Voxxa |
| `recordings/prayer-without-pause` | Amazing Grace v1, v2; the prayer | **Stand-in**, same WAVs | Same | Same |

**These are stand-ins, not sung recordings.** They are spoken, synthetic
audio with no music, so they test the pipeline and the scoring, not how well
Voxxa follows a congregation singing. The founder's two public-domain hymn
recordings replace them when they exist (ticket Q5-01): add each as a new
`recordings/<name>/` folder with `audio.wav` kept on Drive (a row here with its
link, device and date), record its `session.jsonl` with
`voxxa-eval --full --record`, add its thresholds to `baseline.json`, and then
delete the stand-ins.
