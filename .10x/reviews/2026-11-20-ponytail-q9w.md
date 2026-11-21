# Ponytail lane — Q9W (2026-11-20)

Independent pass, as the user asked (process rule, 2026-11-11). Q9W is a code step (Phase 2 check, then Phase 4: WorshipTools Presenter found by process, keys guarded, no step across songs; ADR-019). It ran like this:
- **Two build lanes** started from b154a55 (`main` after the Q9 merge and R0) in worktrees that could not see each other's work: the **10x lane** (`q9w/10x`, `C:/dev/q9w-10x`) and the **ponytail lane** (`q9w/ponytail`, `C:/dev/q9w-pony`).
- **Cross-review** of each lane, then a **comparator**, a **finisher** on `q9w/base` (`C:/dev/q9w-base`) and **three verifier rounds**.

**Outcome:** base = **10x lane**, merged as aaca033, with the 10x review's must-fix and nit, four grafts from the ponytail lane, and the fixes from two verifier rounds that did not pass, one commit each, each with a test that fails without it. **In round 3, at b0de71b, both verifiers passed.** Every Q9W exit criterion is met (§ 8). `q9w/10x` (84acaeb) and `q9w/ponytail` (ee4d431) stay local for reference. Nothing is pushed yet, and CI has not run Q9W.

**Did the real first-run step 2 name the running Presenter? Yes.** On LHMEZPC01, with WorshipTools Presenter running, five separate guarded scratch builds (both lanes, the finisher and both verifiers) opened the real set-up guide, and step 2 named WorshipTools Presenter every time. The real `discover_presenters` returned the process hit, and the real driver reported `can_blank: false` after step 2 connected it (§ 7).

No lyrics and no real recognised words are quoted here. Tests and mocks use placeholder titles ("Song A", "Song B", "Bravo Hymn"). No agent pressed a key in Presenter, focused it, read its windows or its data folder, or stopped any of its 8 processes.

## 1. The task, in short
Pack block Q9W (`specs/2026-11-11-prompts-10x-team.md:71-83`), per ADR-019 and the founder's decisions (handoff, "User decisions for Q9W/Q9V"):
1. Find Presenter by its process on Windows (name `Presenter`, or a path ending `\Presenter\Presenter.exe`), Win32 FFI in the style of `presenters/keystroke.rs`, no new crate. Discovery returns it as `kind: Keystroke`, profile `WorshipToolsPresenter`, source "process", no host or port.
2. With this profile, send a key only when the front window belongs to Presenter. Otherwise send nothing and return "Bring WorshipTools Presenter to the front." The light shows it amber, word for word.
3. Never press Right past a song's last slide. On a song change, press nothing, show amber 'Put "<song title>" live in Presenter.' and record the new song's first slide. Steps inside a song and the operator's raw Next/Prev are unchanged.
4. Remove "." from the profile. `can_blank` comes from the key map (false here). Skip the conductor's blank (no red error at a prayer); the operator's Blank returns "not supported"; the light's blank sentence says Voxxa can't clear Presenter's screen.
5. Reconnect a saved keystroke form whose profile is not universal at every start.
6. First-run step 2 names WorshipTools Presenter in plain words.
7. Docs: a runbook section and INTEGRATIONS Tier 3, MIDI deferred.

Budget (D5): the process check, guard and key map in the presenters row; discovery in **one** new row; the dispatch rules in the 6 lines left in "everything else in src-tauri/src". No existing cap may be raised.

## 2. The two lanes

| | 10x lane (`q9w/10x` @ 84acaeb) | ponytail lane (`q9w/ponytail` @ ee4d431) |
|---|---|---|
| Commits on b154a55 | 7: fdd5f85 (ADR-019 amendment first), cbb8816 (budget row), 71f3f65 (presenters, discovery), f386b71 (dispatch), 7602872 (GUI), c089b09 (docs), 84acaeb (records) | 3: 82bdf71 (backend), 6395b00 (GUI), ee4d431 (docs) |
| Size | 21 files, +818 / −39 (much of it tests and docs) | 14 files, +510 / −42 |
| Finding Presenter | `K32EnumProcesses`, then `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)` and `QueryFullProcessImageNameW` per process: the full path, read-only | a Toolhelp32 snapshot: the exe name only, no process handle opened |
| Discovery result | `host: ""`, `port: 0`, `keystroke_profile`, `DiscoverySource::Process` | `host` and `port` became `Option`, plus `keystroke_profile` and `Process` (changes the shape of `discover_presenters`) |
| The guard | pure `keys_allowed(profile, front_exe)`, called in `tap()` | pure `presenter_in_front(fg_pid, processes)`, called in `tap()`; a new snapshot per key |
| Song-change flag | `stops_at_song_end()`, a default-false trait method; the sentence lives in dispatch | `song_change_ask(title) -> Option<String>`, a default-None trait method that carries the sentence |
| Operator Blank | refused with `Unsupported("blank")` before `set_blank`, so "not supported" wins over the focus error | goes through `set_blank`, which runs the own-window check first, so the operator can get the focus error instead |
| First-run step 2 | connects at once when Presenter is the only app found; a pick next to other finds | a button only; a Next with no pick leaves the universal profile |
| The asks' toasts | the loop's `presenter-error` toast is info when it matches `AMBER_ASKS` | still a red error toast, in Simple mode too |
| Budget | lowered "everything else" 2500 → 2317 so its headroom stays 6; used all 6 (2317 / 2317); discovery 210 / 250; ADR-019 amended in the repo | left "everything else" at 2500 (2316 / 2500, about 178 lines of new room); discovery 227 / 250; amendment text only in its report |
| Tests | `cargo test --features cli` 233 passed, 1 ignored (`tests/worshiptools.rs` 7, 5 new keystroke unit tests); `test:gui` 9/9; e2e 176/176 | lib 60, `tests/dispatch.rs` 14; `test:gui` 9/9; e2e 174/174 three times (one earlier run 173/1 under CPU load, the failing check not identified) |
| Guarded real app | `com.voxxa.qa9wx`, CDP 9361, PIDs 11220 and 14648 (first run, then restart) | `com.voxxa.qa9wp`, CDP 9362, PID 14736; driver 7/7 |
| Records | ADR-019 amendment, SDE, Security and QA records | none, by its instructions |

Both lanes put the rules at the same places and with the same logic. On a song change, dispatch presses nothing, sets `(sent.global, sent.song)` to the new song's first slide and returns the ask as `Err(Other)`. `sent.song` is now recorded for stepping drivers. `can_blank` comes from `keymap().blank`. The conductor's blank is skipped and the operator's is refused. The guard sits in `KeystrokeDriver::tap` before enigo is created. The restart reconnect skips only the universal form. Both lanes passed every gate, the CI eval step was unchanged (21/21 hits, precision and recall 1.00, median 2,310 ms, p95 4,870 ms), and no test could press a real key.

**Safety, both lanes.** The microphone was never opened. Before the first launch, both read the startup path: `lib.rs` only calls `AudioEngine::new()`, and capture opens only in `start_listening_with_state` (`src-tauri/src/commands/mod.rs:229`). Each scratch binary was checked for its QA identifier and for the absence of `com.voxxa.app`. The real apps were never used for Listen, the 20-second test, Next/Previous/Blank, the HTTP API or the stage window. Each lane stopped its app by PID and deleted its `%LOCALAPPDATA%` scratch folder. Presenter's 8 processes and the user's own installed Voxxa (PID 10228) were left alone.

## 3. Cross-reviews

### Review of the 10x lane (b154a55..84acaeb)
**Verdict: fix the must-fix first; the nit is optional.**
- **Must-fix 1. The stage and the main window stay on the old song at every song change.** `commands::dispatch` sent `slide-advanced` only when `drive()` returned Ok. On the WorshipTools song-change ask, `drive()` had already moved `sent` to the new song but returned the ask as an error, so the `?` skipped the event. The stage window kept the last song's slide and "next" text, and the main window's song strip kept the old song, for the whole first slide of the new song. A song-chip click seemed to do nothing. No test checked the event on the ask path.
- **Nit 2.** The operator's own song chip and Prev/Next showed the two asks as red "Failed to jump: …" and "Next slide failed: …" toasts.
- **Not counted as findings** (accepted decisions): a multi-key step that stops halfway (R0 item 18), any `Presenter.exe` treated as Presenter, an administrator-run Presenter dropping keys, the guard passing off Windows, and the song chip asking instead of pressing keys.
- **Strengths named:** the pure guard with a thin FFI wrapper; an unreadable front window is refused; the song rule fits the 6 lines and handles a mid-song landing; the key-map tests cover all 12 profiles and the keys that must never be sent; the fake driver takes its capabilities from the real driver, so no test presses a key; step 2 connects Presenter when it is the only find.
- Re-run by the reviewer: lib 61, dispatch 12, worshiptools 7, `test:gui` 9/9, budget OK.

### Review of the ponytail lane (b154a55..ee4d431)
**Verdict: approve with four should-fixes; nothing blocks a merge.**
1. **A multi-key step can stop halfway.** The guard is checked per key, so a focus change in the 60 ms gap stops the step partway, and `sent.global` stays where it was although keys went out. At the song's end Voxxa would then press one Right too many: the next service item, which ADR-019 § 3 forbids.
2. **The shared blank rule changes ProPresenter 7 and 6 over WebSocket** (`can_blank` false): no red error at a prayer, and the light's can't-clear sentence. An improvement, but not listed as a deviation, and only a JS test pins it.
3. **Moving discovery out of "everything else" frees about 178 lines** under the unchanged 2500 cap, against ADR-019 § 10's intent.
4. **Both asks also raise a red error toast**, in Simple mode too, at every song change.
- **Nits:** 5, the song chip could count as the operator's confirmation; 6, with Voxxa's own window in front the volunteer sees the red own-window row, not the amber sentence; 7, if the ask is missed, the next Right shows Presenter's next item and the light turns green; 8, Toolhelp gives the bare exe name, so any `Presenter.exe` matches.
- Re-run by the reviewer: lib 60, dispatch 14, `simple_logic` 9/9, e2e 174/174, budget OK.

## 4. The comparator's ruling
**Base: the 10x lane at 84acaeb.** The logic is the same in both lanes, so the choice came down to safety, ADR fit, tests and budget, where 10x is better on every point that differs:
1. **Safety and ADR fit.** Step 2 connects Presenter at once when it is the only find; in ponytail a Next with no pick leaves the universal profile, with no guard and no song rule, the gap ADR-019 §§ 2-3 closes. 10x's asks are info toasts, not red. 10x refuses the operator's Blank as "not supported" before the focus check.
2. **Tests.** A dedicated `tests/worshiptools.rs`, including a `gui_sink`-level test that a prayer emits no `presenter-error`; key-map tests over all 12 profiles; a real-app restart.
3. **Budget.** 10x kept § 10's intent by lowering "everything else" to 2317 and recorded the 250 cap in a real ADR amendment.
4. **Simplicity:** about equal. Ponytail is smaller, its `Option` host/port is cleaner, and its `song_change_ask` is neat.

The 10x must-fix also applied to ponytail (its review listed it as a nit), so it was not a reason to choose ponytail.

**Grafts from ponytail:** (1) a Rust test pinning the shared blank rule for a non-keystroke driver with `can_blank` false, and the ProPresenter WS change listed as a deviation; (2) an e2e check that no error toast follows either ask, and that a delivered slide turns the light green; (3) the missed-ask symptom in the runbook and in M16 (nit 7); (4) optional, the red own-window row explained in the runbook (nit 6).

**Disagreements, and how they were settled:**
1. **Toolhelp32 against `OpenProcess`.** Ponytail's snapshot opens no handle; 10x opens a read-only `PROCESS_QUERY_LIMITED_INFORMATION` handle on each process at discovery and on the front process at each key. Nothing is sent to Presenter and its state does not change, and the full path lets 10x match the spec's path literally. Swapping would cost 35-50 lines in a 364 / 400 file for no gain. **Left to the founder:** if "Presenter's process must stay untouched" means no handle at all, this becomes a follow-up (ticket Q9W-06).
2. **The halfway multi-key step** (ponytail should-fix 1). A real drift risk with WorshipTools, but R0 item 18 adopted it as left as is, and "everything else" had no lines to spare. Not in this merge (ticket Q9W-01).
3. **The song chip as confirmation** (ponytail nit 5). Rejected: the chip selects the song in Voxxa; it does not put it live in Presenter. Asking stays correct, and the must-fix makes the strip follow the click.
4. **`Option` host/port.** Cleaner, but it changes the `discover_presenters` shape; not worth the churn.
5. **Budget tension.** 10x lowered the cap, so its own must-fix had no room. That is the honest cost of keeping § 10's intent; ponytail's 2316 / 2500 fits only because the intent was dropped.

## 5. The finisher (`q9w/base`, one commit each)
`q9w/10x` merged with `--no-ff` as aaca033, then:
- **705605a, must-fix 1.** `commands::dispatch` keeps `drive()`'s result and sends `slide-advanced` when it is Ok **or** the song or slide in `sent` moved, then returns the error. It compares `(sent.global, sent.song)`, not the whole `Sent`, because a goto driver's "select …" path may blank first, which must not show a new slide. Merging the two blank arms to make room was tried; rustfmt spread it over 7 lines (2321 / 2317), so it was not made, and the call site got shorter instead (2316 / 2317).
- **7e5473d, graft 1.** `tests/dispatch.rs:376` pins the shared blank rule for ProPresenter 7 and 6 over WebSocket; the deviation is in the SDE record.
- **71ab2b5, nit 2 and graft 2.** `src/main.js:560` `stepFailed` shows an ADR-019 ask bare, as info, from the song chip and Prev/Next; anything else keeps the red prefixed toast. The e2e checks that no `.toast-error` follows each ask (main.js's class is `toast-error`, not the graft's `.toast.error`).
- **38f043d, grafts 3 and 4,** with graft 3 corrected: "put it live, then use the song chip" is wrong here. Once Voxxa has asked for a song, its chip is no song change, so dispatch steps back with Left from the slide it recorded, and what Left does on Presenter's first slide is untested. The runbook and M16 now say: put the song live and click the slide being sung; don't use the chip for that song. M16 adds checking Left on a first slide.
- **492407a, ADR-019 base-merge amendment** (final numbers, the GUI follows the song change, § 4 is shared, the asks are info) and the records 1927856.

Checks at 1927856: `cargo test --features cli` 235 passed, 1 ignored; `test:gui` 9/9; e2e 185/185; budget everything else 2316 / 2317, discovery 210 / 250; the guarded real app (`com.voxxa.qa9wf`, CDP 9363, PIDs 9912 and 14300): step 2 named Presenter, and the restart kept the profile.

## 6. Verifier rounds

**Round 1, on 1927856: did not pass.** The findings, each fixed with a test that fails without it (every mutant re-run against its new test):
- **Docs (e47b2c0).** INTEGRATIONS said Auto-detect connects Presenter; it only lists it, and the operator clicks it and presses Connect. The runbook steps now name a found Presenter, and step 5 says the guide connects at once when Presenter is the only find.
- **The guard was untested at its call site (8022bfd, 97279a8).** Deleting the guard call, flipping `cfg!(windows)` or ignoring the guard's result passed every test. `tap()` now asks the pure `guard()` with the front window's executable from an injectable lookup and sends through an injectable sender (enigo by default). `the_driver_asks_the_guard_before_every_key` (Windows only) connects WorshipTools with explorer.exe in front (next and prev return `FRONT_MSG` and record no key) and with Presenter in front (Right and Left recorded). Every test driver now records keys instead of pressing them. 8022bfd took `keystroke.rs` to 432 / 400; 97279a8 brought it back to 396 / 400 by moving the name cases to `tests/worshiptools.rs`.
- **A ProPresenter look-alike (8022bfd, 63b03d6, 97279a8).** The mutant `ends_with("presenter.exe")` survived; `ProPresenter.exe` and "ProPresenter" joined the miss list.
- **The process hit could be dropped from the merge (63b03d6).** `discovery::merge(local, mdns, process)` is now pure, and `discovery_keeps_the_process_hit_beside_the_network_hits` kills `let _ = process;`.
- **The GUI tell (4fb1346).** `dispatch::tell_gui(ok, before, after)` is the pure rule `commands::dispatch` uses. `the_song_change_ask_tells_the_gui_and_the_guards_refusal_does_not` kills "ok alone" and "compare the whole `Sent`". To fit (2315 / 2317), `drive` records the position in one assignment and `presentation_is` returns its name check as one match arm, with the same behaviour.
- Records 006372c (ADR-019 verifier-fix amendment). Gates: 236 passed, e2e 185/185. The guarded real app (`qa9wf`, PIDs 7232 and 12656) re-ran step 2 and the restart.

**Round 2, on 006372c: did not pass.** Two survivors:
- **Advanced → Auto-detect (d538a4b).** Mutant `gui-advanced-profile-not-set` (`src/main.js:1268` replaced by `void 0;`): without the profile line, Connect would send universal keys (no guard, steps across songs, blanks with "."). A new e2e check in section 9 starts the selects on ProPresenter and universal, runs Auto-detect with the process hit, clicks the row, and checks the selects, the row's meta ("process", no host:port) and the `connect_presenter` arguments. The mutant now fails 2 checks.
- **Discovery wiring (7a8a457).** Mutant `disc-not-wired` (`let _ = process; merge(local, mdns, None)`). `discover_all(timeout)` is now `discover_with(timeout, process_paths)`, and `discovery_runs_the_process_strategy` runs the whole discovery with a 50 ms budget and a fake process list: one process hit with Presenter in the list, none without. Discovery is 223 / 250.
- Records b0de71b (ADR-019 verifier round 2 amendment). Gates: 237 passed, e2e 188/188. The guarded real app (`qa9wf`, PIDs 14924 and 13732) also ran Advanced → Auto-detect against the real process list (§ 7).
- **One line stays untested:** `discover_all` passing the real `process_paths`. The real-app runs and M15 cover it (ticket Q9W-11).

**Round 3, at b0de71b: both verifiers passed.**
- **Verifier A** (exit criteria, real app). Every gate (`tests/worshiptools.rs` 10/10, e2e 188/188). 8 extra tests of its own on a `git archive` copy, never committed, using the real `KeystrokeDriver` with a fake front window and a recording sender: name variants and look-alikes; 7 non-Presenter front windows, each refused with the exact sentence and no key; through the real `dispatch::drive` and `step`, `sent` unchanged and no GUI tell on a refusal; forward, backward and chip song changes; a whole-setlist walk pressing only inside songs; the universal profile still crossing songs; the 11 other profiles equal to b154a55's table. Read-only, the real `process_paths()` found Presenter's 8 processes, and the real front window (PickerHost.exe) got the exact sentence. Guarded real app `com.voxxa.qa9wa` (CDP 9364, PIDs 284 and 5092): step 2 named Presenter; the restart kept the profile. Two notes, neither a failure: INTEGRATIONS says the guide "connects" Presenter, but it does so on its own only when Presenter is the only find (ticket Q9W-12); and M16 now means the song-change check, while the R0 record (`2026-11-20-ponytail-r0.md:219`) had M16 as Q9V's private-fixture eval (handoff, "From: Q9W → To: Q9V").
- **Verifier B** (scope, safety, mutants; detached worktree `C:/dev/q9w-verify`, since removed). Every gate (the first `cargo test` failed only to link, with Windows error 1455, the paging file; the rerun with `-j 2` passed). No manifest, lock file, `tauri.conf.json` or vite config changed; no new crate; `dist/` holds no mock or QA string; one new budget row and one lowered cap. **35 Rust mutants and 18 GUI mutants, all killed**: the guard (removed, inverted, cfg-disabled, always Ok, all profiles, unreadable window allowed), the song rule (removed, flag false, detected slide recorded, song tracking dropped, operator skipped), the key map (".", F10, Down, Up, F12, F11 put back), `can_blank` true, the blank rule, discovery matching and wiring, other profiles' maps, the messages and `tell_gui`, and the GUI's amber rows, `AMBER_ASKS`, `canBlank`, reconnect, step 2 and `useDiscovered`. Guarded real app `com.voxxa.qa9wb` (CDP 9365, PID 6120): step 2 named Presenter. B stopped its headless Edge processes by PID.

## 7. Where each check ran

| Check | Mocked browser (`tests/gui/e2e.mjs` + `mock-tauri.js`, headless Edge; `simple_logic.test.mjs` in Node) | Rust tests (fake presenters, fake process lists, recording sender) | Guarded real app (scratch build, own identifier, CDP) |
|---|---|---|---|
| Presenter found by process | step 2 and Advanced Auto-detect with a mocked hit | `presenter_is_found_by_its_process`, `discovery_keeps_the_process_hit_beside_the_network_hits`, `discovery_runs_the_process_strategy` (`tests/worshiptools.rs:121,164,210`) | **yes**: the real `discover_presenters` returned `{kind: keystroke, keystroke_profile: worship_tools_presenter, source: process}` (ponytail); Advanced → Auto-detect listed the real "WorshipTools Presenter" row (finisher, round 2) |
| Step 2 names Presenter in plain words | yes (section 9) | | **yes, five times**: 10x (`qa9wx`), ponytail (`qa9wp`, its own wording), finisher (`qa9wf`, after each fix round), verifier A (`qa9wa`), verifier B (`qa9wb`) |
| The foreground guard and its sentence | the amber light and info toast (screenshot 28) | `worshiptools_keys_go_only_to_presenter_in_front` and `the_driver_asks_the_guard_before_every_key` (`keystroke.rs:422,397`) | read-only: verifier A's own test got the exact sentence for the real front window; **no key was ever sent** |
| No step across songs | the amber ask, the song strip and stage following it, no red toast | `worshiptools_never_steps_across_a_song`, `the_song_change_ask_tells_the_gui_and_the_guards_refusal_does_not`, `the_operators_raw_next_and_prev_are_unchanged`, `other_keystroke_profiles_still_step_across_songs` (`tests/worshiptools.rs:244,281,330,344`) | **never** (no dispatch, by rule) |
| `can_blank` false, no red error at a prayer | the can't-clear row | `a_blank_worshiptools_cannot_do_is_skipped_or_refused`, `the_light_gets_no_red_error_at_a_prayer_and_the_amber_ask_at_a_song_change` (`:354,375`); ProPresenter WS `tests/dispatch.rs:376` | `get_presenter_info` reported `can_blank: false` after step 2 |
| The exact key map | | `the_worshiptools_key_map_is_right_left_and_no_blank`, `other_profiles_keep_their_keys`, `can_blank_and_the_song_end_stop_come_from_the_profile` (`keystroke.rs:441,470,488`) | |
| Restart reconnect | reload: WorshipTools reconnected, universal not | | **yes**: after a stop by PID and a relaunch, `can_blank` was false at once (10x, finisher, verifier A) |
| Amber sentences word for word | `simple_logic.test.mjs` (titles with brackets; look-alikes stay red) and e2e | | |
| Listen, the 20-second test, Next/Prev/Blank, stage, HTTP API | yes (mock) | | **never** |

**Not checked anywhere, by rule** (founder checklist M15 and M16): a real key reaching Presenter, the guard against a real foreground switch, Presenter run as administrator, the F10 unblank, and what Left does on Presenter's first slide.

## 8. Exit criteria
- Unit tests cover discovery by process with a fake process list, the foreground guard, the no-step-across-songs rule, `can_blank` false with no red error at a blank, and the exact key map: **met** (§ 7).
- A GUI test covers the amber sentences and the restart reconnect: **met** (`simple_logic.test.mjs`, e2e section 9).
- All gates pass: **met** at b0de71b (finisher and both verifiers): npm build, fmt, clippy with and without `--features cli`, `cargo test --features cli` 237 passed, 0 failed, 1 ignored, cargo-deny, the budget, the `dictat` grep, the CI eval step (21/21, "OK: no metric is worse"), `test:gui` 9/9, e2e 188/188; the control-byte grep matches nothing.
- M15 and M16 are in the founder checklist: **met** (`handoff.md`, "Still open from earlier steps").

**Final budget** (`python scripts/loc_budget.py` at b0de71b): discovery (new row) **223 / 250**; everything else in `src-tauri/src` **2315 / 2317** (cap lowered from 2500 when discovery moved out); presenters 1149 → 1374 / 2000 (`keystroke.rs` 396 / 400); GUI 2784 / 3000; Simple mode 436 → 450 / 600. No cap raised, no new crate, no runtime npm dependency.

## 9. What is open
- **Next for the orchestrator** (pre-approved, D6): push `q9w/base`, open a draft PR against `main`, get every CI job green (Q9W has not run in CI; the Linux and macOS jobs will see the off-Windows stubs for the first time), merge, file the tickets (`2026-11-20-q9w-tickets.md`, label `q9w`), close #117, and add the comments listed there. Then Q9V.
- **Tickets:** 12 open (2 medium, 10 low), including "Founder: confirm the F10 unblank key (T3)".
- **Housekeeping:** the scratch binaries carry QA identifiers; never ship them: `C:/dev/q3-10x-target/release/voxxa.exe` (last built by a Q9W agent with `com.voxxa.qa9wf` or `qa9wa`), the copies `voxxa-qa9wx.exe` and `voxxa-qa9wa.exe` in the session scratchpad, and the ponytail and verifier B builds in `C:/dev/q3-pony-target` (`qa9wp`, `qa9wb`). Every build rewrote the line endings of `src-tauri/Cargo.toml` and `src-tauri/gen/schemas/*.json`; each agent restored them with `git checkout` (#91). No Q9W process is left running.

## 10. Lessons for the next step
- **An error can still move state.** The song-change ask returns an error after it has recorded a new position, and the GUI has to follow that position. Decide what the GUI is told from the state change, not from Ok alone.
- **Test the guard where it is called.** A pure helper with a perfect test still let three call-site mutants through, until the driver took an injectable front-window lookup and sender.
- **Lowering a cap keeps an ADR's intent.** Moving code into a new row must not hand the old row free room; the cost is that later fixes have to find their own lines.
- **One real run per wiring line.** The last untested line (`discover_all` with the real process list) is covered only by the guarded real app; keep that run in every re-take of the exit demo.
