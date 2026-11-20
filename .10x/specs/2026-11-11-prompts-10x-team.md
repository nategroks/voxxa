# Voxxa — Claude Code Prompt Pack for `/10x-Team:10x-team` (regenerated 2026-11-11)

Replaces §7 of `2026-10-24-delivery-plan-and-prompts.md`. Reason: discovery of `origin/claude/affectionate-lamport-8al8B` (ADR-010), which already holds most of what Increments 1–3 planned to build. Increment 0 is done and CI-verified on PR #1.

## How to use

Each prompt is the first message of a fresh Claude Code session in the repo root, with the ponytail plugin installed (`full`). **User rule (2026-11-11):** run every prompt independently through ponytail as well: a separate ponytail lane that does not see the 10x output (`ponytail` full, plus `ponytail-review` for code steps or `ponytail-audit` for discovery and close steps). Record it in `reviews/YYYY-MM-DD-ponytail-<Q>.md` and adjudicate disagreements. **ADR numbering:** ADR-011 is taken (operator previews); prompts that say "write ADR-011" use the next free number. Every prompt opens with `/10x-Team:10x-team` so the team protocol applies: read `.10x/handoff.md` and `.10x/status.md` first, work only inside the named phase, write the named state files before finishing, and stop at the gate the prompt names. The numbering (Q0…Q27) is new so it cannot be confused with the October P-numbers.

Sizes and token bands are unchanged from the October plan (S: 5–10 turns, 25–50k output, $3–8 at list; M: 15–30 turns, 75–150k, $10–25; L: 30–60 turns, 150–300k, $25–50; Opus 5.5, cache reads dominate total tokens; 1.5× rework on L).

| | Count | Output tokens | API list cost |
|---|---|---|---|
| S | 9 | 0.2–0.45M | $27–72 |
| M | 16 | 1.2–2.4M | $160–400 |
| L | 5 | 0.75–1.5M | $125–250 |
| **Pack (30 prompts)** | | **≈ 2.15–4.35M** | **≈ $310–720; ≈ $480–1,070 with rework** |
| of which added 2026-11-20 | Q9W, Q9V (M each) | 0.15–0.3M | $20–50 |

Per day: part-time 1 prompt ≈ 75–150k output tokens; full-time 2–3 ≈ 150–400k. Calendar: ≈ 18 weeks part-time, ≈ 10 weeks full-time; the pilot (shadow mode from Q14) and signing paperwork still gate the middle of the plan, not tokens.

Phase map per increment: implementation prompts are Phase 4 (SDE + Senior + DBA); each increment ends with one **close** prompt that runs Phase 5 (QA + Security → `.10x/reviews/`) and Phase 6 (DevOps + SRE → release notes, runbooks, `decisions/devops`, `decisions/sre`). The 10x-team gates are mandatory: no code before the ADR exists, no "done" without the QA and security files.

---

## Stage A — Reconcile (Phases 0–2, then 4–5)

**Q0 — Discovery of the May branch and the reconciliation decision** (M)
> /10x-Team:10x-team
> Phase 0–2 (Discovery → Strategy → Design), roles: CTO, Architect, Staff Engineer. Read `.10x/handoff.md`, `.10x/status.md`, `.10x/decisions/architect/_index.md` (the [DISCOVERED 2026-11-11] section) and `.10x/adrs/010-adopt-phase-1-3-branch-as-code-base.md`. Run the Discovery Protocol on `origin/claude/affectionate-lamport-8al8B` (`git worktree add ../lamport origin/claude/affectionate-lamport-8al8B`): confirm or correct every [DISCOVERED] statement by reading the code, not the docs; add anything missed to the role indexes with the tag. Then review ADR-010 as the Architect: is the conflict rule in Decision 1 complete (list every path that differs between the two branches with `git diff --name-status origin/claude/trusting-newton-v71ebu origin/claude/affectionate-lamport-8al8B` and assign each a side)? Append that table to ADR-010. Present the findings to the user in the Discovery format ("Here's what I found… Does this look right?") and STOP at the user-review gate: do not merge, do not write code. State files: role indexes, `status.md` (phase: Discovery of May branch complete), `handoff.md` (to SDE for Q1 once ADR-010 is Accepted). Do not propose new features. Exit: ADR-010 carries the full path table and the user has approved or amended it.

**Q1 — Merge into one branch and meet the CI bar** (L)
> /10x-Team:10x-team
> Phase 4 (Implementation), roles: SDE + Staff Engineer + DBA (dependency hygiene). Precondition: ADR-010 Accepted. On `claude/trusting-newton-v71ebu`, `git merge --no-ff origin/claude/affectionate-lamport-8al8B` and resolve every path by ADR-010 Appendix A **as amended by A1–A15** (Q0, 2026-11-11). In particular, `Cargo.lock` starts from ours with no bare `cargo update` (A1); resolve hunks, never whole files (A3); A9/A10/A11 licence and content fixes go in the merge PR. Never rewrite history. Then: remove `tauri-plugin-store` (the GUI uses localStorage; grep proves it) and the `text_insert` module (replaced by `presenters/keystroke`); keep the `macos-private-api` feature; combine `build.yml` (their signing scaffolds, our release body); make `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo deny check licenses advisories bans` and `scripts/loc_budget.py` pass. For `cargo deny`: compatible updates first (`cargo update -p <crate>`), then minimal major bumps for `zip`, `quick-xml`, `rusqlite`; move `reqwest` to `rustls-tls` to drop OpenSSL advisories; if `pdf-extract`/`lopdf` has no fixed release, gate the PDF importer behind a Cargo feature that is off by default and say so in `docs/SETLIST_FORMATS.md` rather than ignoring the advisory. Re-base `scripts/loc_budget.py` to ADR-010's per-module budgets. Run the four integration tests and all unit tests. Push; PR #1 updates itself. State: `decisions/sde/church-autoadvance.md` (merge record, every dependency change and why, debt), `decisions/dba/church-autoadvance.md` (SQLite use in the EasyWorship importer: bundled vs system, licence), `status.md`, `handoff.md`. Do not add features, do not touch the conductor's logic, do not delete GUI panels. Exit: all five CI jobs green on the merged head; `README.md` still describes the church product; `git grep -i dictat` clean outside `.10x`.

**Q2 — Verification of the merged base** (L; widened at the Q0 gate, 2026-11-11)
> /10x-Team:10x-team
> Phase 5 (Verification), roles: QA Engineer + Security Engineer. Read `docs/REVIEW.md` (the May branch's own punchlist) and `.10x/specs/2026-10-24-software-strategy.md` § Security and privacy. QA: build the app, load `examples/*` and `setlist.example.json`, exercise every Tauri command through the GUI once, run the HTTP API with and without a bearer token, unplug the audio device mid-listen; record every defect with repro steps in `.10x/reviews/2026-MM-DD-qa-report.md`; fix crashes, data-loss defects **and the four wrong-slide defects confirmed in Q0** now (ponytail: smallest change): `reviews/2026-11-11-ponytail-q0.md` §4 items 1–4 (unmatched speech re-shows lyrics; toggle unblank; keystrokes into Voxxa's own window; setlist-global Goto index). Each fix gets a scripted conductor or dispatcher test. List the rest as tickets. Also check that the stage window cannot default to an audience-facing output (ADR-011), and gather the evidence for the post-Q2 cut-list ADR (HTTP API, mDNS, Hymnary, PDF, stage display, Flatpak, signing scaffolds). Security: STRIDE-lite on the two boundaries plus the new ones this code adds (HTTP API on 127.0.0.1, Hymnary and Planning Center outbound calls, mDNS, EasyWorship SQLite import of an untrusted file, pptx/PDF parsers on untrusted files); verify the outbound-request counter actually counts every `reqwest` client; confirm tokens are not logged by the ring-buffer logger; write `.10x/reviews/2026-MM-DD-security-review.md`. State: `decisions/qa/church-autoadvance.md`, `decisions/security/church-autoadvance.md`, `status.md` (release readiness), `handoff.md`. Exit: both reviews committed; no open crash, data-loss or wrong-slide (audit 1–4) defect; security findings triaged as fix-now (done) or ticket.

## Stage B — Increment 1′: measure it (Phases 2–6)

**Q3 — `voxxad` CLI over the existing library** (M)
> /10x-Team:10x-team
> Phase 2 check then Phase 4: confirm ADR-001 (as adjusted by ADR-010) covers this; if not, write ADR-011 first. Add a second binary `voxxad` that uses `voxxa_lib` (`Conductor`, `VadEngine`, `TranscriptionEngine`, `make_controller`) headlessly: flags `--setlist <json>`, `--stdin-audio <wav>` (16 kHz mono; file speed unless `--realtime`), `--device <name>`, `--presenter keystroke|propresenter7_rest|freeshow|openlp_v2|opensong|none`, `--shadow` (compute and log actions, dispatch nothing), `--replay <session.jsonl>`, `--list-devices`. Stdout: one JSON object per line with `"v":1` and `"t"` ms: `status`, `hypothesis`, `action` (the conductor's `Action`), `error`. Write every event to `session-YYYYMMDD-HHMM.jsonl` in the data dir, rotated after 14 days; `--replay` must reproduce identical `action` lines without running the recognizer. Reuse the audio loop by extracting it from `commands::start_listening` into the library (one function used by both GUI and CLI). State: `decisions/sde/…`, `status.md`, `handoff.md`. Do not add sockets, do not change the GUI. Exit: `voxxad --stdin-audio x.wav --setlist s.json --shadow` prints actions; replay is byte-identical on the action lines; `cargo test` covers the event serialisation.

**Q4 — Recognizer trait and streaming hop** (M)
> /10x-Team:10x-team
> Phase 4, per ADR-004. Introduce `trait Recognizer { fn feed(&mut self, pcm16k: &[f32]) -> Vec<Hypothesis>; fn bias(&mut self, text: &str); }` and implement it for the existing `TranscriptionEngine` (whisper-rs) with a rolling window (`window_ms` 5000, `hop_ms` 1000), `single_segment`, token timestamps, and `initial_prompt` from `bias()` behind a `bias_enabled` flag. Replace the `accumulated.len() >= 16000 * 5` batch in the shared audio loop with the hop loop; feed the conductor's `on_transcript` with each hypothesis. Keep the conductor API unchanged. Add a test that feeds a 10 s WAV fixture and asserts monotonically increasing timestamps. Do not add sherpa-onnx yet. Exit: hypotheses every second in the session log; conductor tests still pass.

**Q5 — `voxxa-eval` and the first fixtures** (M)
> /10x-Team:10x-team
> Phase 4. Add `voxxa-eval` (bin or xtask): a fixture is `audio.wav` + `setlist.json` + `truth.jsonl` (`{t_ms, song, slide}` from the operator). It runs `voxxad --stdin-audio --shadow`, aligns actions to truth, and reports precision, recall, median/p95 latency, false advances per set, double advances, and blank/unblank timing. Add `fixtures/README.md` describing recording (the Windows session's kit if it exists), a CI job that runs eval on committed fixtures and fails below `fixtures/baseline.json`. The founder supplies two public-domain hymn recordings; commit only JSON and a `SOURCES.md`. State files as usual. Exit: eval table prints; CI gate proven on a deliberately broken threshold.

**Q6 — Conductor against the budget: decide, then the minimum** (L, conditional)
> /10x-Team:10x-team
> Phase 2 → 4. Run `voxxa-eval` on every fixture. If false advances ≤ 1 per 10-song set and p95 latency ≤ 2.5 s, write ADR-011 "ADR-009 deferred: conductor meets the budget" with the numbers and stop. Otherwise implement ADR-009's minimum: section labels in `setlist.json` v1 (`sections[].kind`), repeat-aware transitions with priors from `SmartConfig`, word-token scoring on the last 12 recognized words, an evidence accumulator with decay and `min_dwell_ms`, keeping `MachineState`, blanking and `manual_step` untouched. Scripted tests: repeated chorus, bridge jump, dropped words, operator override. Re-run eval and record before/after in `.10x/reviews/`. Exit: a decision file and numbers either way; eval not worse on any fixture.

**Q7 — Close Increment 1′** (S)
> /10x-Team:10x-team
> Phase 5 + 6. QA: run the full fixture eval and the replay determinism test on all three OSes via CI; Security: a test that runs `voxxad --replay` with networking disabled and asserts zero sockets opened; DevOps: release notes for the CLI, `docs/INTEGRATIONS.md` section for driving `voxxad` from Companion; SRE: runbook "eval shows regression". Write `.10x/reviews/…-qa-report.md`, `…-security-review.md`, `decisions/{qa,security,devops,sre}/church-autoadvance.md`, `status.md`, `handoff.md`.

## Stage C — Increment 2′: volunteer-ready Windows (Phases 4–6)

**Q8 — Simple mode in front of the existing panels** (M)
> /10x-Team:10x-team
> Phase 4, per ADR-010 Decision 4 and `software-strategy.md` § Simple mode. Make the default view: the three-position mode switch (Off / Songs / Songs + Scripture, the last disabled until Increment 4′), the status light with one plain sentence (grey/amber/green/red mapped from `MachineState`, listening state and presenter connection), current song and slide, and a Call for help button. Move every existing panel (Settings, Models, confidence bars, presenter connection, imports, HTTP API, stage display) behind one "Advanced" link; delete nothing. The welcome overlay becomes the first-run flow: device → auto-detect presenter → model download → 20-second test. Keep the GUI under ADR-010's budget and dependency-free. Do not change backend commands. Exit: a volunteer sees three controls; everything else is one click away; usability script from the research plan passes on the founder's machine.

**Q9 — Call for help, runbooks, quick-start** (S)
> /10x-Team:10x-team
> Phase 4. Wire Call for help to `generate_diagnostic_report` plus an export of the operator's localStorage state and the last 2 minutes of hypotheses from the session log, zipped to the Desktop, shown to the user before writing, then opened in the support e-mail draft. Write `docs/runbooks/{no-audio-device,presentation-app-not-reachable,model-missing}.md` and link each from the matching red status. Write `docs/quick-start.md` (one page, Windows). No telemetry. Exit: bundle produced; links resolve.

**Q9W — WorshipTools Presenter: find it, guard the keys, never step past a song (M)** (inserted by R0, 2026-11-20; `specs/2026-11-20-prompts-worshiptools-revision.md`)
> /10x-Team:10x-team
> Phase 2 check, then Phase 4, per ADR-019 and ADR-006. Precondition: ADR-019 Accepted; test card T in `reviews/2026-11-20-worshiptools-founder-test.md`. Two lanes (`q9w/10x`, `q9w/ponytail`) off `main`, then the comparator, cross-reviews and verifiers, as in Q9. Record it in `reviews/YYYY-MM-DD-ponytail-q9w.md`.
> 1. **Find Presenter by process.** A Windows-only check for a running process named `Presenter` (T1; also accept an executable path ending `\Presenter\Presenter.exe`), in the Win32 FFI style already in `presenters/keystroke.rs`, with no new crate; other OSes return nothing. Discovery returns it as a found app: `kind: Keystroke`, profile `WorshipToolsPresenter`, source "process" (no host or port).
> 2. **Keys only when Presenter is in front.** Inside `KeystrokeDriver`, for this profile, send a key only when the foreground window belongs to Presenter's process (reuse `foreground_pid()`); otherwise send nothing and return the sentence "Bring WorshipTools Presenter to the front." A pure helper with a unit test, like `own_window_focused`; the own-window guard stays. `statusLight` (`src/simple/logic.js`) shows this sentence and step 3's **amber, word for word**, not as the red "didn't take the last slide change" row, with a GUI test.
> 3. **Never a Right past a song's last slide** (T2; ADR-019 § 3). The extra Rights come from song changes in keystroke dispatch (`dispatch.rs:141-160`), so the rule lives there: a per-driver flag that the profile sets; `sent.song` tracked in the stepping branch; on a song change, no key, amber "Put "<song title>" live in Presenter.", and the new song's first slide recorded as where the screen is. Steps inside one song and the operator's raw Next/Prev are unchanged. Test with a fake driver: crossing a song boundary sends no key; stepping inside a song still does.
> 4. **Key map and blanking** (T3, T4; ADR-019 § 4). Remove `.` from this profile; F10 is the candidate, not used until the founder confirms the unblank key. `can_blank` comes from the key map (this profile: false); dispatch skips a conductor blank when it is false (no red error at a prayer); an operator blank returns "not supported"; the light's blank sentence says Voxxa can't clear Presenter's screen. Tests: the exact key map ({Right, Left, no blank}) and that it never includes F11, F12, Up, Down, numpad + or numpad −. Other profiles don't change.
> 5. **The profile survives a restart** (ADR-019 § 2a). `reconnectPresenter()` (`src/simple/firstrun.js:77`) reconnects a saved keystroke form whose profile is not `universal`, with a GUI test. Otherwise the app starts on `Universal` (`lib.rs:80-87`) and the guard is off after the first restart.
> 6. **First-run step 2** (Simple mode row). When Presenter is found, the step says in plain words that Voxxa found WorshipTools Presenter, that Presenter has no remote control so Voxxa presses its keys, and that Presenter must stay in front during the service. "No presentation app found" stays for the case where nothing is found.
> 7. **Docs.** `docs/runbooks/presentation-app-not-reachable.md`: a WorshipTools section (keep Presenter in front; the two amber sentences; F10 clears lyrics, F11 and F12 clear media, so Voxxa never sends them; Presenter run as administrator silently drops Voxxa's keys). `docs/INTEGRATIONS.md`: WorshipTools is Tier 3 (keys), MIDI deferred per ADR-019.
> Budget (D5, ADR-019 § 10): the process check, guard and key map in the presenters row; discovery in the **one** new row "discovery (src/discovery/)", added to `scripts/loc_budget.py` with its cap; the dispatch rules in the 6 lines left in "everything else", or stop and report. Never raise an existing cap.
> Safety: no keystroke dispatch, no microphone, never the real identifier `com.voxxa.app`. Test with pure helpers and fake presenters, plus both GUI methods (mocked browser and guarded scratch build). Agents never press keys in Presenter: the live check on LHMEZPC01 is founder checklist items M15 and M16.
> State files: `decisions/{sde,qa,security}/church-autoadvance.md`, `status.md`, `handoff.md`, Q9W tickets. Exit: unit tests cover discovery-by-process (fake process list), the foreground guard, the no-step-across-songs rule, `can_blank` false with no red error at a blank, and the exact key map; a GUI test covers the amber sentences and the restart reconnect; all gates pass; M15 and M16 are in the founder checklist.

**Q9V — Private "See a Victory" fixture and the media guard (M)** (inserted by R0, 2026-11-20)
> /10x-Team:10x-team
> Phase 4 + 5, per ADR-019 § 7–9 and ADR-015. Precondition: Q9W merged. Two lanes plus review, as in Q9W. Record it in `reviews/YYYY-MM-DD-ponytail-q9v.md`. (Video songs were dropped: no video plays in services.)
> 1. **The repo refuses media.** A test in `src-tauri/tests/` reads `git ls-files -z`, lower-cases each path, and fails if any ends in `.mp4 .mov .m4v .mkv .webm .wav .flac .mp3 .m4a` or sits under `fixtures/private/`, naming the path. It allows exactly the 15 public-domain recordings in `src-tauri/tests/fixtures/voxxad/`, listed by name. Test the pure check with a fake list (an accented folder name, an upper-case extension). Add the same check as a `ci.yml` step, so a PR that adds media goes red even if the test is skipped.
> 2. **`voxxa-eval --full` prints numbers only.** Today it prints `voxxad`'s stderr (Whisper's log of the words heard), its whole stdout on failure, and a bad line (`cli/voxxa-eval/main.rs:141,146`, `score.rs:305`). In `--full`, stderr goes to null and errors name the exit code or the line number. A test proves a failing run prints no hypothesis text. (A private fixture path already works, `main.rs:54`.)
> 3. **The private fixture: founder steps in `fixtures/README.md` § Private fixtures.** One folder, `fixtures/private/see-a-victory/`. Agents never open, print, copy or move its contents or the .mp4.
>    - Audio (the founder supplies the .mp4 path): `ffmpeg -i "<path>" -vn -ar 16000 -ac 1 -c:a pcm_s16le fixtures\private\see-a-victory\audio.wav`.
>    - `setlist.json` from the church's own licensed text, on LHMEZPC01 only. The txt importer writes no file, so give a BOM-free PowerShell 5.1 snippet (or a tiny `voxxad` subcommand if it fits the budget); `-Encoding utf8` adds a BOM the reader rejects.
>    - `truth.jsonl` (slide-change times): `voxxa-eval` needs it (`main.rs:80-81`). Give the founder a way to make it for the ffmpeg-made WAV, for example the recording kit's key-tap timing while the WAV plays, or written by hand.
> 4. **Founder-run eval.** A checklist item: run `voxxa-eval --full fixtures/private/see-a-victory` on LHMEZPC01 and add one row to `fixtures/LICENSED_RUNS.md` with the summary numbers only.
> State files as in Q9W. Exit: the media guard (test and CI step) passes and catches a planted file in a scratch run; `--full` prints numbers only; `git log --all -- fixtures/private` is empty; the docs give the founder every command.

**Q10 — Windows signing, winget, updater hardening** (M)
> /10x-Team:10x-team
> Phase 4 + 6, per ADR-008. Runs after Q9W and Q9V (ADR-019, D1). In `build.yml`: keep the May branch's Trusted Signing scaffold, add the SignPath Foundation action as the preferred path (skips when secrets are absent), NSIS `perUser`, WebView2 bootstrapper, a `winget` manifest template and release step. In the updater client refuse manifests with a lower version than installed. Set a strict CSP. Unit test the version check. Exit: tag build produces signed NSIS and MSI when secrets exist; CSP no longer null.

**Q11 — Close Increment 2′** (S)
> /10x-Team:10x-team
> Phase 5 + 6. QA: run the five-volunteer first-run test from `specs/2026-10-24-user-research-plan.md` §2 on LHMEZPC01 with WorshipTools Presenter, including recovery from "Bring WorshipTools Presenter to the front" and "Put "<song>" live in Presenter" (ADR-019); Security: review the support bundle for over-collection; DevOps: first signed pre-release; SRE: on-call runbook for the Sunday window. Reviews and decision files as in Q7.

## Stage D — Increment 3′: read-back, fixtures, pilots (Phases 4–6)

**Q12 — Presenter read-back and in-app setlist import** (L)
> /10x-Team:10x-team
> Phase 4, per ADR-006. Extend `PresentationController` with `current_slide_text() -> Result<Option<(String, Option<String>)>>` (current, next) and `setlist() -> Result<Option<Setlist>>`, capability-flagged. Implement for ProPresenter 7.9+ REST (`/v1/status/slide`, `/v1/playlist/current` + `/v1/presentation/{uuid}`), FreeShow (`get_output_slide_text`, `get_projects`/`get_show`), OpenLP v2 (`/api/v2/controller/live-items`, `/api/v2/service/items`). Add `Conductor::resync(song, slide)` and call it when `current_state`/text disagrees with the conductor twice in a row; the operator always wins. GUI: "Load today's setlist from <app>" in the loader. WorshipTools has no read-back, so Voxxa stays blind for it: for that app the loader offers "Load from file" and "Planning Center" only, and no driver is written (ADR-019). Extend the parity contract test to the new capabilities. Exit: all three drivers import a setlist from a live install (the founder provides access; the pilot runs WorshipTools Presenter, which has no read-back, ADR-019); resync demonstrated.

**Q13 — Recorded-traffic contract fixtures** (M)
> /10x-Team:10x-team
> Phase 4 + 5. Build a small in-process HTTP/WS stub that replays request/response fixtures from `src-tauri/tests/fixtures/<driver>/`; add `voxxad --record-adapter` to capture them once against real installs; commit fixtures for ProPresenter REST, FreeShow and OpenLP; make each driver's tests run against the stub in CI. WorshipTools has no traffic to record: add a key-map contract test per keystroke profile instead (ADR-019). Exit: three drivers covered; CI runs them on all OSes.

**Q14 — Shadow mode for pilots and corpus export** (S)
> /10x-Team:10x-team
> Phase 4. Add a "Shadow (log only)" toggle in Advanced that maps to `--shadow`; add `voxxad --export-corpus <dir>` that writes the consented session audio (only when recording was explicitly enabled by the operator), setlist and operator key log into the fixture format. Document the pilot protocol link in `docs/pilot.md`. The pilot is WorshipTools Presenter on LHMEZPC01: `--export-corpus` writes licensed songs only under `fixtures/private/`, and nothing from them leaves the machine (ADR-019). Exit: The Lighthouse, Midlothian can run shadow mode for a Sunday; the resulting fixture stays on LHMEZPC01, and only `fixtures/LICENSED_RUNS.md` rows and summary numbers are committed (ADR-019).

**Q15 — Close Increment 3′** (S)
> /10x-Team:10x-team
> Phase 5 + 6. QA on the three live installs; security review of read-back (tokens in keychain, host binding, identity check via version endpoints before any command); DevOps release; SRE runbook "app reports a different slide than Voxxa". Reviews and decision files.

## Stage E — Increment 4′: scripture (Phases 2, 4–6)

**Q16 — Public-domain Bible bundle** (M) — as October P4.1, under ADR-007; confirm ADR-007 before coding.
**Q17 — Spoken scripture reference grammar** (M) — as October P4.2 (200-case test table; `bibleref` only if it already handles spoken numbers).
**Q18 — Scripture detection in the conductor and `show_ref` per driver** (L) — for WorshipTools, `show_ref` is an operator hint, as for keystroke (ADR-019); as October P4.3 + P4.4, but inside `Conductor` (new `MachineState::Reading` with chapter graph; back to songs on silence > 20 s or a song match); FreeShow `start_scripture`, OpenLP bibles search + live, ProPresenter and keystroke: operator hint. Two spoken-passage fixtures; eval reports reference precision.
**Q19 — Close Increment 4′** (S) — Phase 5 + 6 as above.

## Stage F — Increment 5′: bake-off, Store, support (Phases 4–6)

**Q20 — sherpa-onnx recognizer** (L) — as October P5.1 behind a Cargo feature; `bias()` via hotwords if exposed.
**Q21 — Bake-off and default engine** (M) — as October P5.2; report to `.10x/reviews/`; update ADR-004 status.
**Q22 — Microsoft Store packaging, release channels, support artefacts** (M) — October P5.3 + P5.4 combined: MSIX from the same build, `community`/`stable` channels, `docs/support/*`, `usage.csv`.
**Q23 — Close Increment 5′** (S).

## Stage G — Increment 6′: macOS (Phases 4–6)

**Q24 — macOS permissions, audio, notarization, Homebrew** (M) — October P6.1 + P6.2; the Apple API-key notarization staging already in `build.yml` is the starting point.
**Q25 — Close Increment 6′** (S) — October P6.3 plus reviews.

## Stage H — Increment 7′: Linux and 1.0 (Phases 4–6)

**Q26 — AppImage with zsync, Flatpak verified, Wayland handling** (M) — ADR-008 plus the existing Flatpak manifest and `docs/LINUX.md`; keystroke driver disabled under Wayland with the documented message.
**Q27 — Release 1.0 and upstream docs** (S) — October P7.3 plus the Phase 6 sign-off in `status.md`.

## What to do this week

1. Run **Q0** and approve or amend ADR-010.
2. Run **Q1**, then **Q2**. Merge PR #1 into `main` once Q2 signs off, so later prompts can use `workflow_dispatch`.
3. Record the two hymn fixtures (needed by Q5).
