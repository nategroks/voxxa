# Fixture sources

No audio is committed in `fixtures/`. A fixture's `audio.wav` is either rebuilt from its
`audio.json` recipe (committed WAVs joined with silences, by `voxxa-eval --full`)
or, for a real recording, kept on Drive in
[Voxxa/fixtures](https://drive.google.com/drive/folders/1yT-xiTOLxz67BeebUFdQYtgxqpIHpOXf)
(private to the founder; share per person).

| Fixture | Song(s) | Audio | Recorded | Licence |
|---|---|---|---|---|
| `recordings/prayer-then-reprise` | Amazing Grace v1, v2, v2 again; a spoken prayer | **Stand-in.** `audio.json` joins `src-tauri/tests/fixtures/voxxad/amazing-grace-v1.wav`, `amazing-grace-v2.wav` and `prayer.wav` | Synthetic speech: the hymn text read by the Windows SAPI voice; the prayer written for Q2 | Hymn text public domain (John Newton, 1779); prayer and synthetic audio written for Voxxa |
| `recordings/prayer-without-pause` | Amazing Grace v1, v2; the prayer | **Stand-in**, same WAVs | Same | Same |
| `recordings/prayer-before-first-song` | The prayer; Amazing Grace v1; the prayer; v2 | **Stand-in**, same WAVs (Q5-04's reproduction, added in Q6) | Same | Same |
| `recordings/refrain-it-is-well` | It Is Well with My Soul v1, refrain, v2, refrain, v3, refrain | **Synthetic (TTS).** `audio.json` joins `src-tauri/tests/fixtures/voxxad/tts-it-is-well-*-zira.wav` | Synthetic speech, Q6 (2026-11-13): `scripts/fixtures/make_tts_fixture.ps1`, System.Speech voice "Microsoft Zira Desktop", 16 kHz mono 16-bit, offline | Hymn text public domain (Horatio Spafford, 1873); synthetic audio made for Voxxa |
| `recordings/jump-be-thou-my-vision` | Be Thou My Vision v1, v3, v4, v1 again | **Synthetic (TTS).** `tts-be-thou-my-vision-*-david.wav` | Same script, voice "Microsoft David Desktop" | Hymn text public domain (Irish, tr. Mary Byrne 1905, versified by Eleanor Hull 1912); synthetic audio made for Voxxa |
| `recordings/prayer-between-songs` | Holy, Holy, Holy v1, v2; a spoken prayer; Come, Thou Fount of Every Blessing v1, v2 | **Synthetic (TTS).** `tts-holy-holy-holy-*-david.wav`, `tts-prayer-zira.wav`, `tts-come-thou-fount-*-zira.wav` | Same script, voices David (Holy, Holy, Holy) and Zira (prayer, Come Thou Fount) | Hymn texts public domain (Reginald Heber, 1826; Robert Robinson, 1758); prayer written for Voxxa (Q6); synthetic audio made for Voxxa |

The three **Synthetic (TTS)** fixtures (Q6, user decision 2026-11-13) are
generated, not recorded: their WAVs and truth times come from
`scripts/fixtures/make_tts_fixture.ps1` (rule in `README.md`, "Synthetic TTS
fixtures"). They add a repeated chorus, jumps and a prayer between two songs.

**These are stand-ins, not sung recordings.** They are spoken, synthetic
audio with no music, so they test the pipeline and the scoring, not how well
Voxxa follows a congregation singing. Ticket Q5-01 (#73) planned
public-domain hymn recordings to replace them in CI. The founder closed it on
2026-11-20 as "local eval only" (ADR-019 § 9): CI keeps these stand-ins, and
real sung audio (a licensed song) is evaluated only on the booth PC from
`fixtures/private/`, never on Drive (README, "Private fixtures"). A
public-domain recording can still be added here later the same way: a
`recordings/<name>/` folder, its `audio.wav` on Drive, a row here, and its
thresholds in `baseline.json`.
