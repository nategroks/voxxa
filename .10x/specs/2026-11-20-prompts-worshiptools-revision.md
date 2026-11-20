# Voxxa — prompt revisions: WorshipTools Presenter and "See a Victory" (2026-11-20)

Run these before Q10. **R0** revises the prompt pack and the records and writes no code. It inserts **Q9W** and **Q9V** (text below) after Q9 in `specs/2026-11-11-prompts-10x-team.md` and applies amendments **M1–M8**. Every prompt also runs through ponytail (user rule, 2026-11-11).

---

## Facts (founder, 2026-11-20, booth PC LHMEZPC01, local build 0.1.0 from `voxxa-newton`)

- **F1.** The Lighthouse runs **WorshipTools Presenter** on the booth PC. With Presenter open, first-run step 2 of 4 says "No presentation app found. Voxxa will press the arrow keys instead…".
- **F2.** That is the code working as written. `discovery/mod.rs` probes only API apps: ProPresenter on 1025 (and mDNS), FreeShow on 5506, OpenLP on 4316 and OpenSong on 8082. Presenter has no local API (ADR-006, research A4). Its Remote and Stage Display apps go through WorshipTools' cloud. It accepts MIDI In, which needs rtpMIDI or loopMIDI on Windows, and it accepts keyboard shortcuts.
- **F3.** Presenter's documented shortcuts (WorshipTools docs, "Keyboard shortcuts", universal group):
  - Right / Space / Page Down: next slide.
  - Left / Page Up: previous slide.
  - Down / numpad +: next service item.
  - Up / numpad −: previous service item.
  - F10: clear slide content. F11: clear background media. F12: clear all.
  - No blank toggle is documented.
- **F4.** `KeystrokeProfile::WorshipToolsPresenter` sends Right and Left, and `.` to blank. `.` is not a documented Presenter shortcut. The driver checks only that Voxxa itself is not focused (`own_window_focused`). It never checks that Presenter is in front, although ADR-006 says it should.
- **F5.** "See a Victory" (Elevation Worship, CCLI-licensed) is in Presenter with its lyrics imported, and Presenter can play an .mp4 for it.
- **F6.** Presenter cannot export songs (WorshipTools forum). Voxxa can get these lyrics only from Planning Center or from a text file the church makes on its own machine (ADR-007).

## Assumptions (strike any that are wrong before running R0)

- **A1.** The .mp4 plays inside Presenter for "See a Victory". It is either the SongSelect lyric video, with the words in the video so no slide changes while it plays, or a motion background under lyric slides.
- **A2.** On the booth PC, the same .mp4 can be turned into 16 kHz WAV test audio for a **private** eval fixture.
- **A3.** The Lighthouse is the first pilot, so WorshipTools support comes before Q10 (signing).

## Founder test card T (about 10 minutes, on LHMEZPC01, not during a service)

Agents never press keys in Presenter (unattended safety rule: no keystroke dispatch). These checks are yours. Paste the results into the R0 session. R0 records them in `reviews/2026-11-20-worshiptools-founder-test.md`.

- **T1. Process name.** Run this in PowerShell with Presenter open, then copy the output:
  ```powershell
  Get-Process | Where-Object { $_.Path -like '*WorshipTools*' -or $_.MainWindowTitle -like '*Presenter*' } |
    Select-Object Name, Id, Path, MainWindowTitle | Format-List
  ```
- **T2. End of song.** In a test service, put "See a Victory" live and click Presenter. On its last slide, press **Right**. Does it move to the next service item? Answer yes or no.
- **T3. Clearing lyrics.** On a lyric slide, press **F10**. Do the lyrics clear while the background or video keeps playing? Then press Right and Left. Which key brings the lyric back, and does it bring back the same slide?
- **T4. The `.` key.** Press **`.`** (period). Does anything happen?
- **T5. The video.** With the .mp4 live, press **Right**, then **Space**. Does either skip, stop or pause the video? Is the .mp4 a lyric video or a background (A1)?
- **T6. The file.** Where is the .mp4 on disk? Give its full path. Don't move it.

---

## R0 — Revise the prompt pack and the records (S, planning only, no code)

> /10x-Team:10x-team
> Phase 2 (Strategy → Design) for the planning records only. Roles: Product Manager, Architect, QA. **No code changes.** Read `.10x/handoff.md`, `.10x/status.md`, this file (`.10x/specs/2026-11-20-prompts-worshiptools-revision.md`), ADR-006, ADR-007, ADR-011 and ADR-015, `src-tauri/src/discovery/mod.rs`, `src-tauri/src/presenters/keystroke.rs`, `src/simple/firstrun.js` and `fixtures/README.md`.
>
> 1. **Verify F1–F6 in the code and docs.** Mark each one confirmed or corrected, with the file and line. Don't search the web again; cite this file for the WorshipTools facts.
> 2. **Ask the founder one batch of questions** (AskUserQuestion; options with a recommendation first), then STOP until the answers come back:
>    - **D1 Order:** run Q9W and Q9V before Q10 (recommended), or after Q11.
>    - **D2 MIDI:** keep it deferred and ship keys with a guard that checks Presenter is in front (recommended; MIDI needs loopMIDI or rtpMIDI installed on the booth PC), or build a MIDI driver now.
>    - **D3 Video songs:** a setlist song marked as video gets no keys. Voxxa keeps listening and tracking, and the operator moves Presenter on (recommended). Or treat it like any other song.
>    - **D4 Blank key for Presenter:** F10 if T3 shows it clears only the lyrics (recommended), or no blank at all. Never F11 or F12: they stop the background video.
>    - **D5 Budget:** detection code goes in the presenters row (851 lines left). The discovery call site and any dispatch change must fit in the 6 lines left in "everything else in src-tauri/src"; if they can't, stop and report the shortfall. Or allow one new recorded row.
>    - **D6 Pre-approvals:** push the planning branch and open a draft PR. For Q9W and Q9V: push, draft PR, CI fixes, merge when green, file tickets, stop processes the agents started (by PID).
>    - Confirm or strike A1–A3, and ask for the T1–T6 results if they aren't in the session yet.
> 3. **ADR-019** "WorshipTools Presenter: found by process, keys only when Presenter is in front; video songs; MIDI deferred". The next free number is 019; ADR-012 stays reserved for the cut list. Record F1–F6, the answers to D1–D5, the T results, and that ADR-006's review trigger has fired: the pilot church's app is Tier 3. Accept it per the founder's answers.
> 4. **The prompt pack** `specs/2026-11-11-prompts-10x-team.md`:
>    - Insert **Q9W** and **Q9V** after Q9, verbatim from this file except for edits the answers require.
>    - Apply **M1–M8**.
>    - Add two M rows to the cost table.
>    - Don't renumber Q10–Q27.
> 5. **Private fixtures:**
>    - Add `fixtures/private/` to `fixtures/.gitignore`. Prove it with `git check-ignore -v fixtures/private/x/setlist.json`.
>    - Add a section "Private fixtures (licensed songs)" to `fixtures/README.md` with the content rules below.
> 6. **Records:**
>    - `handoff.md`: a new top section "User decisions for Q9W/Q9V (2026-11-20)", and the founder checklist with M8 applied.
>    - `status.md`: the plan change.
>    - `decisions/product-manager/church-autoadvance.md`: the pilot runs WorshipTools Presenter.
>    - `decisions/architect/_index.md`: F1–F6 tagged [STATED 2026-11-20].
>    - `reviews/2026-11-20-worshiptools-founder-test.md`: T1–T6, as given.
> 7. **Ponytail:** a `ponytail` + `ponytail-audit` lane that does not see your output. Record it in `reviews/2026-11-20-ponytail-r0.md` and adjudicate any disagreements.
>
> Branch `plan/worshiptools` off current `main`. Commit with the Write and Edit tools, not heredocs. Run `git grep -lIP '[\x00\x0b]'` before every commit. Push and open a draft PR only if D6 allows it.
> **Exit:** ADR-019 Accepted; the pack holds Q9W and Q9V with M1–M8 applied; `fixtures/private/` is ignored; no song text appears in the diff.

---

## Q9W — WorshipTools Presenter: find it, guard the keys, fix the key map (M)

*R0 inserts this after Q9.*

> /10x-Team:10x-team
> Phase 2 check, then Phase 4, per ADR-019 and ADR-006. Precondition: ADR-019 Accepted, and the T results in `reviews/2026-11-20-worshiptools-founder-test.md`. Two lanes (`q9w/10x`, `q9w/ponytail`) off `main`, then the comparator, cross-reviews and verifiers, as in Q9. Record it in `reviews/YYYY-MM-DD-ponytail-q9w.md`.
>
> 1. **Find Presenter by process.**
>    - Add a Windows-only check for a running process, matching the name or names recorded in T1. Use the Win32 FFI style already in `presenters/keystroke.rs`, with no new crate. Other OSes return nothing.
>    - Put it in the presenters row. Make discovery return it as a found app with `kind: Keystroke`, profile `WorshipToolsPresenter`, and a source that says it was found by process (no host or port).
>    - Keep within the D5 budget. If the call site doesn't fit, stop and report.
> 2. **Guard: keys only when Presenter is in front.**
>    - For the `WorshipToolsPresenter` profile, send a key only when the foreground window belongs to Presenter's process. Reuse `foreground_pid()`.
>    - Otherwise send nothing and return an error that the status light shows amber with one sentence: "Bring WorshipTools Presenter to the front."
>    - Put the decision in a pure helper with a unit test, like `own_window_focused`. The existing own-window guard stays.
> 3. **Key map.**
>    - Set the `WorshipToolsPresenter` blank key to F10 if D4 chose it. Make unblank match what T3 found, or make the profile report `can_blank: false` if T3 found no safe way back.
>    - Add a test that this profile's keys never include F11, F12, Up, Down, numpad + or numpad −. Those keys clear the media or change the service item.
>    - Other profiles don't change.
> 4. **First-run step 2** (Simple mode row):
>    - When Presenter is found, the step says, in plain words, that Voxxa found WorshipTools Presenter, that Presenter has no remote control so Voxxa presses its keys, and that Presenter must stay in front during the service.
>    - "No presentation app found" stays for the case where nothing is found.
> 5. **Docs:**
>    - `docs/runbooks/presentation-app-not-reachable.md`: add a WorshipTools section (Presenter in front; what the amber sentence means; F10 and F11 differ).
>    - `docs/INTEGRATIONS.md`: WorshipTools is Tier 3 (keys), and MIDI is deferred per ADR-019.
>
> **Safety rules from the Q9 handoff:**
> - No keystroke dispatch, no microphone, never the real identifier `com.voxxa.app`.
> - Test with pure helpers and fake presenters, plus both GUI methods (the mocked browser and the guarded scratch build).
> - The live check on LHMEZPC01 is a founder step. Write it as M-items in the founder checklist.
>
> State files: `decisions/{sde,qa,security}/church-autoadvance.md`, `status.md`, `handoff.md`, and the Q9W tickets.
> **Exit:**
> - Unit tests cover discovery-by-process (with a fake process list), the foreground guard and the forbidden-keys rule.
> - All gates pass.
> - The founder checklist holds: "with Presenter open, step 2 names it" and "keys are held while Presenter is not in front".

---

## Q9V — Video songs, and a private "See a Victory" fixture (M)

*R0 inserts this after Q9W.*

> /10x-Team:10x-team
> Phase 4 + 5, per ADR-019 (D3) and ADR-015. Precondition: Q9W merged. Two lanes plus review, as in Q9W. Record it in `reviews/YYYY-MM-DD-ponytail-q9v.md`.
>
> 1. **Video songs** (only if D3 chose them):
>    - Add an optional per-song field to setlist.json v1 that marks the song as video. Document it in `docs/SETLIST_FORMATS.md`.
>    - For a video song, the conductor keeps tracking, dispatch sends no key, and the status sentence says Voxxa is listening only during the video.
>    - Add scripted tests:
>      - No action is dispatched inside a video song.
>      - Tracking moves to the next song when singing matches it.
>      - The operator can still step manually.
>    - Keep the conductor API. Respect the conductor row (27 lines left) and D5. Stop and report if it doesn't fit.
> 2. **The private fixture: tooling and docs only.** The founder makes the fixture. Agents never open, print or copy its contents.
>    - Founder steps, written into `fixtures/README.md` § Private fixtures:
>      - Convert the .mp4: `ffmpeg -i "<T6 path>" -vn -ar 16000 -ac 1 -c:a pcm_s16le fixtures\private\see-a-victory-mp4\audio.wav`.
>      - Or record the desk feed while the .mp4 plays in Presenter: `powershell -ExecutionPolicy Bypass -File .\fixtures\tools\record-session.ps1 -Device "<name>" -Name see-a-victory-live -Song "See a Victory" -OutDir fixtures\private`.
>      - Make `setlist.json` from the church's own licensed text, on the booth PC only. One way: copy the lyrics from WorshipTools Planning into a `.txt` and import it with Voxxa's txt importer.
>    - Make `voxxa-eval --full` accept a private fixture path and print **numbers only**: no hypothesis text, no setlist text. CI never sees `fixtures/private/`.
>    - Add a test that fails when any path under `fixtures/private/` is tracked (`git ls-files fixtures/private` is empty).
> 3. **Founder-run eval:** a checklist item to run `voxxa-eval --full fixtures/private/<name>` on LHMEZPC01 and paste only the summary table into `reviews/`.
>
> State files as in Q9W.
> **Exit:**
> - The video-song tests pass, if D3 chose them.
> - The private-fixture guard test passes.
> - `git log --all -- fixtures/private` is empty.
> - The docs give the founder every command.

---

## Amendments R0 applies to the pack

- **M1 · Q10:** scope unchanged. It runs after Q9W and Q9V if D1 says so.
- **M2 · Q11:** the five-volunteer first-run test runs on LHMEZPC01 with WorshipTools Presenter. It includes recovery from "Bring WorshipTools Presenter to the front" and one video song.
- **M3 · Q12:** WorshipTools has no read-back, so Voxxa stays blind for it. For that app, the loader offers "Load from file" and "Planning Center" only. No driver is written.
- **M4 · Q13:** WorshipTools has no traffic to record. Add a key-map contract test per keystroke profile instead.
- **M5 · Q14:** the pilot is WorshipTools on LHMEZPC01. `--export-corpus` writes licensed songs only under `fixtures/private/`, and nothing from them leaves the machine.
- **M6 · Q18:** for WorshipTools, `show_ref` is an operator hint, as for keystroke.
- **M7 · ADR-006:** its review trigger fired on 2026-11-20 (the pilot's app is Tier 3). Point to ADR-019 and keep MIDI as the named next step.
- **M8 · Founder checklist:** add "Keystroke with WorshipTools Presenter: in front, not in front, F10, video song" next to the PowerPoint/ProPresenter item. Add test card T as done once R0 records it.

## Content rules (every prompt above)

- Never write, echo, commit or upload the lyrics of "See a Victory" or any CCLI song, its .mp4, or audio made from either. This includes Google Drive: the Drive folder is for public-domain recordings only.
- Licensed fixtures live only in `fixtures/private/<name>/` on the booth PC, and git ignores that folder.
- Agents use placeholder words in tests, and print counts, never recognised text.
- Voxxa never projects lyrics (ADR-011). Presenter does the projecting.

---

## Addendum 1 (founder, 2026-11-20): licensed test audio. Record the licence, never commit the media.

Applies to R0 and Q9V.

1. **Record every run, never the media.** `fixtures/LICENSED_RUNS.md` (committed) has one row per local eval run on a licensed song, with these columns only: date | song title | artist | CCLI Song # | church CCLI License # | source (Presenter media library / SongSelect) | machine (LHMEZPC01) | voxxa commit | eval summary numbers.
   - First row: See a Victory | Elevation Worship | CCLI Song # | License #.
   - Never put lyrics, recognised words, file paths under AppData, or media hashes in this file.
   - `fixtures/README.md` § Private fixtures links to it and says: "Licensed songs are tested locally under the church's CCLI licence. Only the licence record and the numbers are committed; the media, audio and lyrics never are."
2. **The repo refuses media everywhere**, not only in `fixtures/private/`.
   - Root `.gitignore`: `*.mp4 *.mov *.m4v *.mkv *.webm *.wav *.flac *.mp3 *.m4a`, and `fixtures/private/`.
   - A test in `src-tauri/tests/` runs `git ls-files`. It fails if any tracked path ends in one of those extensions or sits under `fixtures/private/`, and names the offending path.
   - The same check is a step in `ci.yml`, so a PR that adds media goes red even if the test file is skipped.
   - If any such file is already tracked, stop and report it, and don't rewrite history without asking.
   - **Founder decision (2026-11-20):** 15 public-domain recordings are already tracked in `src-tauri/tests/fixtures/voxxad/` and the replay tests need them. Both the test and the CI step allow exactly those 15 paths, listed by name. Any other media path fails. History is not rewritten.
3. **Agents never copy, upload or move the .mp4.** It stays in Presenter's media folder. The founder converts it locally to `fixtures/private/see-a-victory/audio.wav` with ffmpeg. Nothing derived from it leaves LHMEZPC01, Google Drive included.
4. Record this as decided in ADR-019 and in the "User decisions for Q9W/Q9V (2026-11-20)" section of `handoff.md`.

**Split.** R0 changes no code. It writes `.gitignore`, `fixtures/LICENSED_RUNS.md` and `fixtures/README.md`. The test and the `ci.yml` step are Q9V work.

**CCLI numbers** (read-only lookup in Presenter's local data, 2026-11-20):
- See a Victory: CCLI Song # 7129060. Source: the `text_ccli` field on the song record named "See A Victory".
- The church's CCLI License # is not in Presenter's local data (`copyrightLicense` is empty). It is pending from the founder.

## Addendum 2 (founder, 2026-11-20): test card results and spec changes

**Test card T (LHMEZPC01):**
- **T1.** Process `Presenter`, path `C:\Program Files\Presenter\Presenter.exe`, window title "WorshipTools Presenter" (read-only process listing).
- **T2.** Right on a song's last slide moves Presenter to the **next service item**.
  - **Risk:** an extra Right at the end of a song skips into the next item, which may not be a song.
  - **Q9W:** the conductor must never send Right past a song's last slide for the WorshipToolsPresenter profile, and a test must prove it.
- **T3.** F10 clears the lyrics from the main screen. How to bring them back was **not tested**.
  - The WorshipToolsPresenter profile reports `can_blank: false` until the founder confirms an unblank key.
  - ADR-019 records F10 as the candidate blank key.
- **T4.** `.` does nothing in Presenter. Remove `.` as this profile's blank key.
- **T5.** Not applicable: the .mp4 is not used in services.
- **T6.** The .mp4 is only **test audio**, used in place of the public-domain hymn recordings (#73). The founder supplies its path in Q9V.

**Changes to this spec:**
- **A1 is wrong:** no video plays during services.
  - Drop **D3** (video songs) and **Q9V step 1**.
  - Q9V becomes "Private See a Victory fixture" only (steps 2 and 3).
- **A2 holds:** the .mp4 becomes 16 kHz WAV test audio, in `fixtures/private/` only.
- **#73:** a CCLI song in `fixtures/private/` can't run in CI. The founder chose to close #73 as **"local eval only"**. CI keeps the public-domain fixtures it already has.
- **M2:** remove "and one video song" from Q11.
- **M8:** drop "video song" from the founder checklist line.

**Founder answers to R0 step 2:**
- **D1:** before Q10.
- **D2:** keys with a guard that checks Presenter is in front; MIDI stays deferred.
- **D3:** dropped (see above).
- **D4:** F10 if T3 shows it clears only the lyrics. T3's unblank was not tested, so `can_blank: false` for now and F10 is the candidate.
- **D5:** one new recorded budget row is allowed.
- **D6:** all pre-approved. R0: push and open a draft PR. Q9W and Q9V: push, draft PR, CI fixes, merge when green, file tickets, stop processes the agents started.
- **A2, A3:** confirmed.
