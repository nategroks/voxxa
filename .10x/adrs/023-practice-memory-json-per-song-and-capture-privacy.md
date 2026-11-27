# ADR-023: Practice memory (the "listener DB") is one JSON file per song; capture is private by default

**Status:** Proposed (2026-11-24) · **Date:** 2026-11-24 · **Feature:** church-autoadvance · **Reversibility:** Easy (files; a later store can import them) · **Owner:** nategroks · **Author:** 10x-Team synthesiser, from drafts 10, 12, 14 and reviews 20, 21

Keeps ADR-003 (no database). Amends ADR-018's default for "include words" in the help bundle.

## Context
- The founder's "listener DB" means practice-session memory: per song, what voxxa learns in a rehearsal (order actually sung, pace, pauses, what Whisper tends to hear per slide), used live in the service (brief § 2).
- ADR-003: durable data is files; "No SQLite, no embedded database"; review trigger "any file exceeds ~10 MB or two machines share state" (`.10x/adrs/003-single-config-and-jsonl-protocol.md:16-17`, `:41`). A practice file with ≤ 20 heard tokens per slide is < 25 KB [est.]; one booth PC per church.
- `voxxad` can already listen to a device in shadow mode and log every hop (`src-tauri/src/cli/voxxad/cli.rs:31-41`). The GUI writes no session log (#53; ADR-018 line 10).
- Session logs hold every transcript, prayers included, in roaming `%APPDATA%`, which a domain profile may copy to a server (`fixtures/README.md:277-282`; `src-tauri/src/help/mod.rs:181` uses `dirs::data_dir()`).
- The help bundle's "include the last 2 minutes of words" box is ticked by default (`src/simple/help.js:38`).
- A setlist fingerprint already exists (FNV-1a, `src-tauri/src/cli/voxxad/events.rs:131-138`).

## Decision
1. **One file per song:** `<church_root>\practice\<song_fp>.json`, `song_fp` = the existing FNV-1a fingerprint applied to one song (a lyric edit gives a new key, so stale data is never used). Schema v1 (spec § 4.4): `v`, `song_fp`, `title`, `recog` {model, window_ms, hop_ms, audio_ctx, backend}, `runs[]` {date, kind practice|service, status proposed|confirmed|discarded, order, dwell_ms, gaps_ms, outro_ms, words_per_s, confidence}, `plan` {order, dwell_ms, max_gap_ms, outro_ms, words_per_s}, `heard` {slide → ≤ 20 tokens}. Keep the last 3 confirmed runs; times are medians; writes are atomic (temp + rename).
2. **Only confirmed runs** set `plan` or any learned threshold. `heard` is valid only while `recog` matches.
3. **Capture v1 via `voxxad`** (no new capture code): `voxxad --device "<desk feed>" --shadow --setlist <church_root>\services\<date>-<slot>.setlist.json --log-dir <church_root>\sessions`. A church gets `voxxad` from the `voxxa-tools-<version>-windows-x64.zip` release asset (plan V5c, ADR-029 § 7); until V5c ships, capture runs only on the founder's development booth from a local build. GUI capture and a "Learn from this rehearsal" button come with Q14 (which also delivers #53).
4. **Learn offline, then confirm (one CLI form everywhere):**
   - `voxxad --learn <session.jsonl> --setlist <file> --out <church_root>\practice` runs a hindsight alignment with the moves stay / next / back to a section start / back to the song start / skip one, so a restarted song needs no operator input. It prints section labels (from the importer's `sections`, ADR-022 § 7), counts and confidence only, and writes runs as `proposed`.
   - `voxxad --confirm <song_fp> --practice-dir <church_root>\practice` confirms the newest proposed run after the operator agrees.
   - The learner's logic lives in the practice row (`src-tauri/src/practice/`, cap 300); the voxxad row holds only the parsing of these four flags and the dispatch into the practice module (≤ 20 lines, ADR-028 § 3).
5. **Live use:** the follower takes `plan.order` as the expected path, `dwell_ms` as the pace prior, `max_gap_ms + 2 s` as the gap hold, `heard` as extra match tokens. Missing, stale or low-confidence files → cold rules.
6. **Capture privacy:**
   - session logs default to `%LOCALAPPDATA%\com.voxxa.app\sessions\`, Tauri's app-local-data dir (not roaming): app data, kept 14 days (rotation unchanged), and may be removed with the app; what the NSIS uninstaller's "delete app data" option removes is **[UNVERIFIED]**. Rehearsal captures for `--learn` still go to `<church_root>\sessions` through `--log-dir` (§ 3);
   - every export (corpus fixture, practice file, help bundle) drops words heard outside songs; practice files keep only per-slide tokens;
   - the help bundle's words box defaults to **unticked** (founder item FA-19);
   - the full logs themselves keep talk text, because replay of the default conductor needs it (ADR-013); this is the one accepted limit.
7. **Location:** `church_root` defaults to `%LOCALAPPDATA%\voxxa-church\` for a new church (one church per PC). It is a sibling of the install folder `%LOCALAPPDATA%\Voxxa\` (NTFS names are case-insensitive, so that is also `%LOCALAPPDATA%\voxxa\`) and of Tauri's app-local-data folder `%LOCALAPPDATA%\com.voxxa.app\`; install, update, uninstall and the V16 rename never touch it. On the founder's development booth `church_root` is `C:\dev\voxxa-church-private\pilot\`; the kit's fake second church is `C:\dev\voxxa-church-private\fake-church-2\`; shared private tools stay in `C:\dev\voxxa-church-private\lab\` (ADR-029 § 2). Never a repo working tree, never roaming, never a cloud-synced folder, never inside the install folder or the app-data folder.

## Alternatives Considered
| Alternative | Pros | Cons | Why Not |
|---|---|---|---|
| One SQLite file per church for library + practice (draft 12) | queries, transactions, one backup file | schema, migrations, a store module (~900 lines), amends ADR-003 for < 100 KB of data; the size argument misread ADR-003's per-file trigger | not needed at one church, one PC; revisit at > ~500 songs or two machines |
| Practice data inside `setlist.json` | one file | changes the replay fingerprint and the setlist format | separate concerns |
| Live learning during the service | no rehearsal needed | learns from mistakes in front of the congregation | offline hindsight learning has no live risk |
| Remove talk text from the logs themselves (review S-36 as written) | stronger privacy | breaks byte-identical replay of the default conductor, which reacts to unmatched speech | strip at export instead; logs stay local and rotate |

## Consequences
### Positive
- No new storage layer to secure or migrate; files are readable by support.
- Rehearsal learning works the day `--learn` lands, with today's capture path.
- Prayer words stop leaving the booth through help bundles by default.
### Negative
- No cross-song queries (e.g. "which songs are warm") without reading a folder; fine at this scale.
- GUI capture waits for Q14.
### Risks
- A messy rehearsal proposes a wrong order: only confirmed runs count.
- `heard` tokens are lyric-derived: private; never logged, bundled or printed.
- Moving existing session logs between folders could fail on a locked file **[UNVERIFIED]**: the move is best-effort and logged.

## Dependencies
- Depends on: ADR-003, ADR-013, ADR-018, ADR-022.
- Constrains: ADR-029 (private folder layout).
- Plan steps: V5c (tools zip, so a church can run `voxxad`), V9 (log privacy), V12 (practice memory), Q14 (GUI capture).

## Review trigger
> ~500 songs per church; two machines need the same memory; any practice file > 1 MB; a church asks for a cross-song report the folder cannot serve.
