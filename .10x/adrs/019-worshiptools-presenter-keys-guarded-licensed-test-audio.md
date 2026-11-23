# ADR-019: WorshipTools Presenter is found by process, gets keys only when it is in front, and never gets a Right past a song's end; licensed test audio stays on the booth PC

**Status:** Accepted 2026-11-20. Based on the founder's R0 answers (D1, D2, D4, D5, D6), test card T, the two addenda in `specs/2026-11-20-prompts-worshiptools-revision.md`, and the R0 ponytail audit (`reviews/2026-11-20-ponytail-r0.md`). Amends ADR-006 and ADR-015 (private fixtures). ADR-006's review trigger fired **by founder decision**: the only pilot runs a Tier 3 app; there are no support tickets yet. Supersedes nothing.

## Context
- **The pilot church runs WorshipTools Presenter** on the booth PC LHMEZPC01. First-run step 2 says "No presentation app found…" (`src/simple/firstrun.js:107`). That is the code working as written: discovery probes only ProPresenter 1025 (and mDNS), FreeShow 5506, OpenLP 4316 and OpenSong 8082 (`src-tauri/src/discovery/mod.rs:48-53`, `:159`).
- **Presenter has no local API** (ADR-006:6, research A4). Its Remote and Stage Display apps go through WorshipTools' cloud. It accepts MIDI In, but only with loopMIDI or rtpMIDI installed. It accepts keyboard shortcuts.
- **Presenter's documented shortcuts** (spec F3): Right, Space and Page Down go to the next slide; Left and Page Up go back; Down/Up and numpad +/− change the service item; F10 clears slide content, F11 the background media, F12 everything. No blank toggle is documented.
- **Today's profile is wrong.** `KeystrokeProfile::WorshipToolsPresenter` shares the default map: Right, Left and `.` to blank (`src-tauri/src/presenters/keystroke.rs:83-88`). The only focus check is Voxxa's own window (`own_window_focused`, `keystroke.rs:42`, called at `commands/mod.rs:284`). ADR-006:11 says keys go only to the app in front, but nothing checks that.
- **Test card T (founder, LHMEZPC01, 2026-11-20):**
  - **T1.** The process is `Presenter` (`C:\Program Files\Presenter\Presenter.exe`, window title "WorshipTools Presenter").
  - **T2.** Right on a song's last slide moves to the **next service item**.
  - **T3.** F10 clears the lyrics from the main screen. How to bring them back was not tested.
  - **T4.** `.` does nothing.
  - **T5.** Not applicable: no video plays in services.
  - **T6.** The "See a Victory" .mp4 is test audio only; the founder supplies its path in Q9V.
- **Keystroke dispatch walks the flat setlist** (`src-tauri/src/dispatch.rs:141-160`): moving from song A's last slide to song B's first presses one Right. In Presenter (T2) that Right is "next service item", which is song B only if Presenter's service order matches the setlist exactly, with nothing in between.
- **Budget** (`scripts/loc_budget.py`, 2026-11-20): presenters 1149/2000, everything else in `src-tauri/src` 2494/2500 (6 left), conductor 1173/1200, Simple mode 436/600.

## Decision
1. **Found by process.** On Windows, discovery checks the running processes for the name recorded in T1 (`Presenter`; the check also accepts the executable path ending `\Presenter\Presenter.exe`). It uses the Win32 FFI style already in `keystroke.rs`, with no new crate; other OSes return nothing. A hit is a found app: `kind: Keystroke`, profile `WorshipToolsPresenter`, source "process" (no host or port). The code lives in the presenters row.
2. **Keys only when Presenter is in front** (D2).
   - For the `WorshipToolsPresenter` profile, a key is sent only when the foreground window belongs to Presenter's process (`foreground_pid()`). Otherwise nothing is sent.
   - The guard lives inside `KeystrokeDriver` (presenters row), as a pure helper with a unit test, like `own_window_focused`. The own-window guard stays.
   - The status light shows the guard's sentence **amber and word for word**: "Bring WorshipTools Presenter to the front." It is not the red "<app> didn't take the last slide change: …" row (`src/simple/logic.js:16-19`).
   - The guard covers this profile only. ADR-006's "only when the target app is foreground" still does not hold for the other keystroke profiles.
2a. **The profile survives a restart.** Today `reconnectPresenter()` skips every keystroke form (`src/simple/firstrun.js:77`), and the app starts on the `Universal` profile (`src-tauri/src/lib.rs:80-87`). That would switch the guard, the song-change rule and the key map off after the first restart. A saved keystroke form whose profile is not `universal` is reconnected at startup, with a GUI test.
3. **Never a Right past a song's last slide** (T2). The conductor itself never steps past a song's end: the tracker stays inside the song and a manual step is clamped. The extra Rights come from **song changes**, because keystroke dispatch presses one key per slide of setlist distance (`dispatch.rs:141-160`). So the rule lives in dispatch:
   - The driver tells dispatch, through a per-driver flag set by the profile, not to step across songs. Dispatch sees only `kind()` and `capabilities()` today.
   - Dispatch tracks `sent.song` in the stepping branch too. It is set only in the goto branch today (`:140`).
   - On a song change it presses nothing, shows amber "Put \"<song title>\" live in Presenter.", and records **the new song's first slide** as where the screen is: putting a song live in Presenter shows its first slide.
   - Within one song, steps work as today. The operator's raw Next/Prev (`manual_step` falling back to `dispatch::step`) is the operator's own choice and is unchanged.
   - A test with a fake driver proves that crossing a song boundary sends no key and that stepping inside a song still does.
   - **Limit:** Voxxa is blind. If the operator does not put the new song live, the next Right inside it still goes to Presenter's next item. The amber sentence is the only protection.
4. **Key map and blanking** (T3, T4, D4).
   - Remove `.` from this profile. **F10 is the candidate blank key**, not used yet. `unblank()` presses the blank key again (`keystroke.rs:202-205`), and how to bring the lyrics back after F10 is untested.
   - `can_blank` comes from the key map, not the fixed `Capabilities::KEYSTROKE` (`keystroke.rs:147-149`). This profile reports `can_blank: false`.
   - Dispatch skips a conductor blank when `can_blank` is false, so prayers raise no red error. An operator blank returns "not supported".
   - When `can_blank` is false, the light's blank sentence says Voxxa can't clear Presenter's screen.
   - Tests: the exact key map ({Right, Left, no blank}), and that it never includes F11, F12, Up, Down, numpad + or numpad −. Other profiles don't change.
5. **MIDI is deferred** (D2). It is the named next step if keys prove unreliable at the pilot.
6. **No video songs.** A1 was wrong: no video plays during services. D3 and the video-song work are dropped.
7. **Licensed test audio** (addendum 1, A2).
   - The "See a Victory" .mp4 stays in Presenter's media folder. Agents never copy, upload, move or open it.
   - The founder converts it on LHMEZPC01 to `fixtures/private/see-a-victory/audio.wav` (16 kHz mono) with ffmpeg. Nothing made from it leaves that PC, Google Drive included.
   - `fixtures/LICENSED_RUNS.md` (committed) records each local run: the licence record and the summary numbers only. Row 1 is See a Victory, Elevation Worship, CCLI Song # 7129060; the church's CCLI License # is pending from the founder.
   - The fixture must be scorable. `voxxa-eval` needs `truth.jsonl` (slide-change times) next to `audio.wav` and `setlist.json` (`cli/voxxa-eval/main.rs:80-81`), and ffmpeg makes audio only. Q9V gives the founder a way to make `truth.jsonl` and a BOM-free `setlist.json` (the txt importer writes no file; PowerShell 5.1's `-Encoding utf8` adds a BOM the JSON reader rejects).
   - `voxxa-eval --full` must not print recognised words. Today `voxxad`'s stderr (Whisper's log), its whole stdout on failure, and a bad line are all printed (`main.rs:141,146`, `score.rs:305`). Errors name the exit code or the line number only.
8. **The repo refuses media.**
   - The root `.gitignore` ignores `*.mp4 *.mov *.m4v *.mkv *.webm *.wav *.flac *.mp3 *.m4a` and `fixtures/private/`.
   - In Q9V, a test in `src-tauri/tests/` and a `ci.yml` step fail on any tracked path with one of those extensions or under `fixtures/private/`, naming the path. Both read `git ls-files -z` (quoted non-ASCII paths would slip past) and lower-case the path before matching (`X.WAV`).
   - A new synthetic test WAV is now ignored too: add it to the allowlist and `git add -f` it, or its fixture passes locally and fails in CI (`fixtures/README.md` says so).
   - Exactly the 15 public-domain recordings already tracked in `src-tauri/tests/fixtures/voxxad/` are allowed, listed by name (founder decision). History is not rewritten.
9. **#73 closes as "local eval only"** (founder). A CCLI song in `fixtures/private/` can never run in CI. CI keeps the public-domain fixtures it has, and real sung audio is evaluated locally on LHMEZPC01.
10. **Budget** (D5). The process check, the foreground guard and the key map go in the presenters row. Discovery (a `Process` source, the call, a profile field) plus the dispatch rules need about 15 lines, against the 6 left in "everything else". So Q9W uses the **one** new budget row, **"discovery (src/discovery/)"** (about 183 lines today), recorded in `scripts/loc_budget.py` with its cap. The dispatch changes then fit the 6 lines, or Q9W stops and reports. No existing cap is raised.
11. **Order** (D1). Q9W and Q9V run before Q10.

## Alternatives
| Alternative | Why not |
|---|---|
| MIDI driver now | Needs loopMIDI or rtpMIDI installed and configured on the booth PC, plus a new dependency. Keys with a foreground guard serve the pilot first. |
| Keep stepping across songs with Right | T2: one Right at a song's end jumps to the next service item, which may be a reading or announcement. A wrong item goes up on the screen in front of the congregation. |
| F10 as the blank key now | How to bring the lyrics back is unknown (T3). A blank Voxxa can't undo is worse than no blank. |
| Commit licensed audio in Git LFS or a private repo | A CCLI licence covers projection and local use, not redistribution. The media never leaves the booth PC. |
| Purge the 15 tracked wavs | They are public domain and the replay tests need them; rewriting history breaks clones and PR refs. |

## Consequences
- With WorshipTools, Voxxa tracks across songs but never moves Presenter between service items. The operator puts each song live, which is one action per song.
- Blanking with WorshipTools is off until the founder confirms the unblank key.
- `fixtures/private/` and every media file are kept out of git by two checks. A new public-domain recording needs an explicit allowlist entry.

## Review trigger
- The founder confirms an unblank key for F10: switch `can_blank` on in a follow-up.
- Keys fail at the pilot more than once a service (focus lost, wrong item): reopen MIDI (Decision 5).
- WorshipTools ships a local API: move Presenter to Tier 1 or 2 under ADR-006.

## Amendment (Q9W Phase 2 check, 2026-11-20)
The Architect and the Staff Engineer re-read every code reference above at `b154a55`. Each still matches: `firstrun.js:77,107`, `discovery/mod.rs:48-53,159`, `keystroke.rs:42,83-88,147-149,202-205`, `commands/mod.rs:284`, `logic.js:16-19`, `lib.rs:80-87`, `dispatch.rs:140,141-160`, and the budget figures. Two amendments:
1. **§ 10, the numbers.** `src/discovery/` is 183 code lines at `b154a55` (tokei). The new row "discovery (src/discovery/)" is capped at **250**, which leaves 67 lines for the process source, the profile field and the call. "Everything else in src-tauri/src" stops counting those 183 lines, so its cap goes **down from 2500 to 2317**. Its headroom stays the 6 lines this section names: moving discovery out does not hand it 183 free lines. No cap is raised.
2. **§ 2, off Windows.** The process check runs only on Windows (`foreground_pid()` is 0 elsewhere). Off Windows the guard passes, as the own-window guard does there (`keystroke.rs:32-36`). The pilot runs Windows.

## Amendment (Q9W base merge, 2026-11-20)
Recorded when `q9w/10x` was merged into `q9w/base` with the review fixes. Nothing above is reversed.
1. **§ 10, final numbers** (`python scripts/loc_budget.py` at the end of Q9W). The rows and caps are the ones amendment 1 set, in `scripts/loc_budget.py:83-84`: "discovery (src/discovery/)" **210 / 250**; "everything else in src-tauri/src" **2316 / 2317**; presenters 1342 / 2000 (keystroke.rs 364 / 400); GUI 2784 / 3000; Simple mode 450 / 600. No cap was raised; the 2500 cap was not restored.
2. **§ 3, the GUI follows the song change.** The ask returns as the dispatch error, but dispatch has already recorded the new song's first slide. `commands::dispatch` (`src-tauri/src/commands/mod.rs:308-333`) therefore sends `slide-advanced` when `drive` succeeds **or** moves the song or slide in `sent`, then returns the error. The stage, the song strip and Simple mode's song line follow the new song while the light shows the ask. The guard's refusal and the goto drivers' "select …" error move nothing and send nothing. The comparison leaves out `sent.blank`: the "select …" path may blank first.
3. **§ 3, the limit, for the volunteer.** When the ask is missed, the light goes green on the next slide while Presenter shows the wrong item. The runbook tells the volunteer to put the song live and click the slide being sung. It says not to use the song chip for that song: once Voxxa has asked for a song, its chip is no song change, so dispatch steps back to slide 1 with Left. What Left does on Presenter's first slide is untested (M16).
4. **§ 4 is shared.** The blank rule lives in `dispatch::drive` for every driver with `can_blank` false, not only this profile. ProPresenter 7 and 6 over WebSocket (`presenters/propresenter7_ws.rs:135`) change with it: the conductor's blank at a prayer is skipped (no red error, the light's blank rows say Voxxa can't clear the screen), and the operator's Blank is still "not supported". Pinned in `src-tauri/tests/dispatch.rs:376`.
5. **The operator's asks are info.** In Advanced, the song chip and Next/Previous show the two ADR-019 sentences bare, as info toasts, like the listening loop's `presenter-error` (`src/main.js:560`, `stepFailed`).

## Amendment (Q9W verifier fixes, 2026-11-20)
Nothing above is reversed. The verifier's surviving mutants are now caught, and the code has moved to make that possible:
1. **§ 2, the guard at its call site.** `KeystrokeDriver::tap()` asks the pure `guard(profile, front_exe)` and sends through an injectable sender (enigo by default). The front window lookup is injectable too. Tests use a fake front window and a recording sender, so no test can reach enigo.
2. **§ 1 and § 3 helpers.** `discovery::merge(local, mdns, process)` is the pure union that `discover_all` returns. `dispatch::tell_gui(ok, before, after)` is the pure rule from amendment 2 of the base merge, and `commands::dispatch` calls it.
3. **§ 10, numbers now** (`python scripts/loc_budget.py`): discovery **217 / 250**; everything else in src-tauri/src **2315 / 2317**; presenters 1374 / 2000 (keystroke.rs **396 / 400**); GUI 2784 / 3000; Simple mode 450 / 600. No cap was raised. To fit `tell_gui`, two pieces of `dispatch.rs` were written in fewer lines with the same behaviour. The `is_presenter_exe` cases moved from `keystroke.rs` to `tests/worshiptools.rs`, where they run through `presenter_process`.

## Amendment (Q9W verifier round 2, 2026-11-20)
Nothing above is reversed. Two paths that no test reached are now tested:
1. **§ 1, Advanced → Auto-detect.** `useDiscovered` (`src/main.js:1266-1269`) sets the key profile from the process hit. An e2e check in `tests/gui/e2e.mjs` section 9 starts the selects on ProPresenter and universal, runs Auto-detect with the process hit, and clicks the WorshipTools row. It checks that the row shows no host:port, that the selects read keystroke and `worship_tools_presenter`, and that Connect sends that profile. Without the profile line, Connect would send universal keys, which have no guard, step across songs and blank with ".".
2. **§ 1, the process strategy is wired in.** `discover_all(timeout)` is now `discover_with(timeout, process_paths)`. `discover_with` takes the process list as a `fn() -> Vec<String>` (`src-tauri/src/discovery/mod.rs:64-77`). `discovery_runs_the_process_strategy` in `tests/worshiptools.rs` runs the whole discovery (network probes included) with a 50 ms budget and a fake process list. It expects one process hit when Presenter is in the list and none when it is not. The one line that is still untested is `discover_all` passing the real `process_paths`. The real-app demo and founder check M15 cover it.
3. **§ 10, numbers now:** discovery **223 / 250**. The other rows are unchanged from the amendment above. No cap was raised.

## Amendment (Q9V, 2026-11-20)
Nothing above is reversed. §§ 7 and 8 as built:
1. **§ 8, how the guard matches.** The extension and `fixtures/private/` checks use the lower-cased path; `fixtures/private/` matches at the root or deeper (`*/fixtures/private/*`). The allowlist compares the path **exactly as tracked**, so `PRAYER.WAV` beside the 15, or `prayer.wav` in another folder, is refused. The test (`src-tauri/tests/media_guard.rs`) also fails when an allowlisted file is no longer tracked, and checks that ci.yml lists the same 15 paths and nine extensions.
2. **§ 8, the CI step has its own job.** The step is the `media` job in `ci.yml` (`ubuntu-latest`, bash, like the deny and budgets jobs), not a step of the three-OS `check` job: it needs bash 4 or later (`${path,,}`, `declare -A`), which macOS runners lack and every Ubuntu image has, and it then does not wait on the Rust build. It is not pinned to ubuntu-22.04, so it does not break when GitHub retires that image. A PR that adds media fails that job even if the test is skipped.
3. **§ 7, numbers only.** In `--full`, `voxxad`'s stderr goes to null and a failure names its exit status only; an error line on `voxxad`'s stdout is named by its line number. In both modes, a bad `voxxad` line or truth line is named by its number, never quoted, and a bad `setlist.json` by line and column (serde quotes a wrong value, which may be lyrics). Replay (the default, CI) still shows `voxxad`'s stderr and, on failure, its stdout: public-domain text. Proved by `src-tauri/tests/eval_numbers_only.rs` with a fake `voxxad` and placeholder words.
4. **§ 7, the founder's files.** `setlist.json` comes from a PowerShell 5.1 snippet in `fixtures/README.md` that writes without a BOM (no `voxxad` subcommand; the voxxad row is untouched). `truth.jsonl` comes from `fixtures/tools/tap-truth.ps1`, which plays the WAV and writes a row per Enter (the first Enter is slide 0; `b` is a blank), or by hand. A private fixture has no `baseline.json` row, so its `--full` run exits 1 with `FAIL` lines: it is a report (M17), not a gate.
5. **§ 10, numbers:** voxxa-eval (`src/cli/voxxa-eval/`) **488 / 600**; everything else in src-tauri/src 2315 / 2317 and voxxad 552 / 600, unchanged. No cap was raised.

## Amendment (founder, 2026-11-23): the private test song is New Life
Nothing above is reversed except the song, and the rule on who converts the media.
1. **§ 7, the song.** The founder replaced "See a Victory" with **New Life**: "that is much better option as it is original and you can follow vocals clearly". Presenter's local data (read from a copy, no text printed) lists it with artist **Lighthouse Worship** and an empty CCLI field: it is the church's own song, not CCLI-listed. It stays private like a licensed song: the media guard, `fixtures/private/`, numbers only.
2. **§ 7, who converts.** The founder pointed the orchestrator at the .mp4 (a 2:00 lyric video in the founder's Downloads folder) and said the lyrics are in Presenter. On 2026-11-23 the orchestrator made `fixtures/private/new-life/audio.wav` with the README's ffmpeg command (the .mp4 was read in place, not copied, moved or uploaded) and `setlist.json` (the song's 18 slides in Presenter's order, written without a BOM, from a scratch copy of Presenter's IndexedDB that was then deleted; no slide text was printed or logged). Both live in a local worktree that is never pushed. `truth.jsonl` needs someone listening: the founder taps it with `fixtures/tools/tap-truth.ps1`.
3. **The licence record.** The church's CCLI number is **2919521** (the founder calls it the CCLI account number), now in `fixtures/LICENSED_RUNS.md` row 1. See a Victory (CCLI Song # 7129060) was never run and is no longer row 1.
