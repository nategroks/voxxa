# Adjudication: the 10x lanes vs the ponytail lane, and every review finding

Date 2026-11-24. Inputs: drafts 10 (Sr Eng A, listening), 11 (Sr Eng B, integrations), 12 (Sr Eng C, platform),
13 (UI consultant), 14 (independent ponytail lane, which saw no 10x draft), and the adversarial reviews 20
(safety/security) and 21 (feasibility/accuracy/scope). Base `origin/main` @ `866b51e`. Repo citations were
spot-checked on that commit for this adjudication: `score.rs:49-51`, `transcription/mod.rs:179-204`,
`dispatch.rs:148-178`, `aligner/mod.rs:84-93`, `:352-364`, `listen.rs:51-59`, `:157-167`,
`voxxa-eval/main.rs:186-190`, `build.yml:38-45`, `:160-168`, `:350-356`, `lib.rs:117-122`,
`http_api/mod.rs:76-90`, `keystroke.rs:59-63`, `events.rs:131-138`, `importers/txt.rs:1-9`, `cli.rs:31-41`,
`help/mod.rs:179-182`, `src/simple/help.js:36-39`, ADR-003, ADR-007, ADR-011, `fixtures/README.md:262-292`,
`software-strategy.md:24-32`, `handoff.md:78-124`, `status.md:302-306`, `loc_budget.py:1-10`, `:76-100`,
`ci.yml:140-165`, `.gitignore:18-45`. All held.

## 1. Method

1. Where the 10x drafts and the ponytail lane agree, the plan adopts the shared position unless a review refutes it.
2. Where they disagree, the ruling goes to the position with measured evidence; failing that, to the smaller
   change that still meets the founder's request, plus any safety control a review shows is missing.
3. A review finding is accepted when it cites code or arithmetic that holds on re-check; it is rejected (in
   whole or part) only with a stated reason. The founder's explicit asks (brief § 1–2) override "cut" advice.

## 2. Agreements (10x lanes and ponytail)

| # | Shared position | Where it lands |
|---|---|---|
| 1 | Fix the release-build flags before any live number counts | V0, ADR-021 |
| 2 | One-song results are signal, not proof; ≥ 10 songs, song-disjoint held-out set, one frozen setting | spec § 2.3, V6 |
| 3 | The follower ships behind an ADR and a gate | ADR-022, V11 |
| 4 | whisper-rs 0.13.2 already has `vulkan`; Vulkan over CUDA; a GPU build may not start without the loader | ADR-021 |
| 5 | `voxxa-eval --model` passthrough (`main.rs:188`) | V2 |
| 6 | Delete the dead Python prototype | V1, ADR-027 |
| 7 | No lyrics in layout or service files; voxxa sends identifiers and navigation only (ADR-011) | ADR-024 |
| 8 | Practice data per song, keyed by a lyric fingerprint, local (not roaming), private | ADR-023 |
| 9 | Amend ADR-007; CCLI optional; credit shown by the presentation app | ADR-026 |
| 10 | Kit public under `kit/`, church data private; nothing church-specific in the repo | ADR-029 |
| 11 | On WorshipTools, graphics and scripture are operator hints | ADR-024 |
| 12 | New code in new recorded budget rows; never raise a cap | ADR-028 |
| 13 | The operator always wins; numbers-only logs | ADR-022, ADR-024 |

## 3. Disagreements and rulings

| # | Topic | 10x position | Ponytail (14) position | Ruling | Reason |
|---|---|---|---|---|---|
| 1 | "Listener DB" storage | C: one SQLite file (library + practice), amends ADR-003 | JSON file per song | **Ponytail** | ADR-003:17 forbids a DB and its trigger is per file (`:41`); files are < 25 KB; C's 10 MB argument misread the trigger (review 21). C's licence semantics kept as data (`licence-kinds.csv`). Founder confirms (FA-11). |
| 2 | Lyric entry | UI U3 + C C8: in-app editor and library (1,050–1,700 GUI lines) | `.txt` + `songs.csv` + existing loader and editor | **Ponytail now; founder decides the editor** | The strategy ruled out lyric editing (`software-strategy.md:30`); one source text pasted into both apps avoids slide-count drift. Flagged as a deviation from "in voxxa" (FA-14). |
| 3 | Master layout | B: 3 files, 7 profiles, executor, cue engine, TOFU, CLI allowlist (~1,200 lines, 9 slices) | one `layout.json` (≤ 200 lines) with `say`, `keys`, `run` | **Hybrid** | B's file split (church / service / bindings) and security rules, at P's size: execution limited to existing driver methods and hints in v1; app steps enabled per live contract test; no `run` (S-01). |
| 4 | Follower scope | A: new 700-line row, time-based prediction, pitch gate, heard signatures, `Input::Window` | port the prototype as measured into `evidence/`, margin gate, DR gated | **Split** | P on order (port what was measured); A on `Option<FollowerConfig>` for byte-identical replay, the gap hold shipping with the port, off-score words as talk evidence, reporting timeliness separately. Pitch gate, signatures beyond 20 tokens and prediction deferred until an experiment shows a gain. Size unknown: the row is set from V6's measurement. |
| 5 | GPU target | A: Small, 4 s window, `audio_ctx` 200, 0.5 s hop, p95 ≤ 350 ms | Small, 6 s, `audio_ctx` 300, ≤ 0.8 s/pass, 1 s hop | **Ponytail** | Only the 6 s encoder has evidence; shrinking the encoder broke Tiny (brief § 6); A's 0.15–0.3 s is a FLOP estimate. |
| 6 | whisper-rs bump | C: 0.16 before GPU; A: with GPU | none | **Ponytail** | 0.13.2 has `vulkan`; 3 breaking minors; forces fixture re-records; API delta unverified. |
| 7 | Release-flag fix mechanism | C: CMake `FORCE` include + repo `.cargo/config.toml`; A: same in `src-tauri/.cargo/` | env vars in the Windows job + `release_config.rs` + add the cause to #59 | **Ponytail + C's guards** | Env is the one verified fix; `FORCE` unverified; A's folder is wrong. Added from C: new cache keys (`build.yml:165`, `:354`), a CMakeCache check, the local `cargo clean` recipe. A's and C's "new ticket" rejected: #59 exists. |
| 8 | uv | C: `.python-version`, setup-uv in CI, public lab uv project with torch | booth install, PEP 723, no CI change | **Ponytail + C's 3.12 pin and private lab project** | Every tracked script is stdlib; uv in CI buys nothing. C's SHA-pinning is a separate ticket. |
| 9 | "95%" | A: SA within `[l(k−1), t_w + 1.0 s]` | the strategy's numbers (recall ≥ 0.95, p50 ≤ 1.2 s, p95 ≤ 2.5 s) | **Neither alone: two numbers** | A's shape is right, its song-start edge impossible; P's "recall" is late-tolerant (`score.rs:49-51`). Correct now, On time after V6b (FA-05). |
| 10 | Kit size | C: process/, onboarding/, library/, 11 prompts + 5 role prompts, 4 skills, 2 tools, manifest, semver | 8 templates, 6 prompts, 1 skill, 1 tool | **Middle** | The founder asked for prompt sets and skill sets, so 8 prompts and 5 thin skills over one shared safety block; validated on a fake second church; process stays in `.10x/`; no role prompts generalised from staging (S-45); tagged kit instead of a manifest generator. |
| 11 | Practice capture | A: practice mode with a per-song confirm card, room calibration, signatures; C: SQLite | shadow `voxxad` + offline `voxxad --learn`; Q14's toggle moves here | **Ponytail + A's confirm** | Offline learning has no live risk; capture works today only through `voxxad` (the GUI writes no session log, ADR-018:10), so the GUI toggle and #53 stay in an amended Q14. Confirm is `--confirm` now, a GUI card later. |
| 12 | Override hotkeys | UI: global bare F6–F9 + focus hand-back spike | Ctrl+Alt chords; bare keys refused | **Neither** | F13–F24 for pedals/Stream Deck (no keyboard has them) + keyboard F-keys only after a conflict check; never Ctrl+Alt+arrows; modifier-release wait (S-13). Founder chooses hardware (FA-17). |
| 13 | CLI commands | B: allowlist, hash, approval, off by default, slice 7 | `run` argv from the layout, no shell, no approval | **Neither in v1** | `run` is Critical (S-01); B's model has gaps (S-03, S-05). Not built in v1; ADR-025 records the full rule set for later. The inbound CLI surface is the existing HTTP API. |
| 14 | Cue matcher | B: ordered phrase, suggest, 120 s cooldown | every word anywhere in the last 12 words, auto, 20 s | **B + arming fix** | Bag-of-words fires on sermon sentences (S-17); arming by item pointer, not conductor state (S-15). |
| 15 | Cold-song hide rule | A: hold 10 s; B: blank 5 s after the last slide | keep 3 s/5 s until a practice file exists | **One rule (S-27), A's 10 s cold minimum** | The measured failure was the mid-song hide; review 21 § 2.3 item 3. |
| 16 | CC BY-ND | C: warn on streams; UI: note only | allow outright | **C / S-30** | Music synced with video is an adaptation under CC 4.0 [UNVERIFIED legal]; ND red when the church streams. |
| 17 | Prototype location | A: move into public `tools/voxxa-lab/`, equivalence test public | private lab only | **Ponytail** | Hidden lyric risk in cached windows and docstrings (S-43); equivalence test stays private. |
| 18 | `LICENSED_RUNS.md` row with church data | C: keep values | move to the private folder | **Move and scrub (FA-12)** | S-42. |
| 19 | WorshipTools song end | A: one Right to the next song's blank after 3 s | — | **Not in v1** | Tail choruses and the last song of a set (S-26); #184's song change stays. |
| 20 | Song detection | A: lower bar for the planned next song; whole-library match prompt | unchanged | **Neither: 2 windows, no lowered bar, suggestion during spoken items** | A spoken quote of the next song must not start it (S-16); the pitch gate is deferred, so the control is temporal plus the layout's item pointer, gated by a TTS test. |
| 21 | `usage.csv` | C: `song_usage` table and `usage --csv` | drop it | **Ponytail** | The church is leaving CCLI; dated service files are the usage record; a credits export goes to Q22. |
| 22 | Dependency policy | C: monthly window, 7-day cooldown, weekly deny cron, six refresh PRs | no upgrades for their own sake | **Ponytail now** (revised in § 8, G05: a monthly agent-run window if FA-32 (a)) | Only the SHA-pin ticket now; C's policy is a later process ADR when a refresh is due. |
| 23 | Q11 sequencing | A, B, C ignore Q11; UI extends it with new tasks | V0 → Q11 → the rest | **Ponytail** | Q11 is next and needs #59 (`handoff.md:92`); new screens do not exist yet. |
| 24 | Global-shortcut plugin | — | use it for hotkeys, or delete it | **Use it in V13** | `lib.rs:119`. |
| 25 | Draft-truth lab (torch, Demucs, wav2vec2) | A: yes, with a review player | no | **Deferred** | 2–3 GB, unproven on singing; tapping first (review 21 § 4.5). |
| 26 | Wire-format audit W1–W5 | B: first ticket, before bindings | not covered | **B, as tickets now, fixed in Q13** | Needs live installs; independent of the layout. |

## 4. Review 21 (feasibility) findings: accepted and rejected

| Finding | Ruling | Applied in |
|---|---|---|
| § 0.1 / § 1: "95% on time" has no evidence; ≤ 45% within 1.5 s offline | **Accepted** | spec § 2.2, ADR-022 |
| § 0.2: 11/11 has a Wilson lower bound of 0.74; the 8-wrong swing is a robustness failure | **Accepted** | spec § 2.3 (neighbour configuration) |
| § 0.3 / § 1.2: song starts are late by construction; report separately | **Accepted** | spec § 2.2 |
| § 0.4 / § 4.1: CPU decoder settings and state reuse before the GPU | **Accepted, with one change:** the GPU proof stays in the first wave (the founder asked for it and the SDK install is pending), but its scope shrinks to S if V3 reaches ≤ 1.0 s/pass | V3, V4 |
| § 0.5 / § 2.1: about 42 new steps and +3,450/+2,050/+1,800 lines is too much | **Accepted** | 18 steps (19 after critic round 1 added V5c, § 8); ADR-028 (+1,250 Rust, +350 GUI, +300 Python in new caps; about 300 lines inside existing rows, every share listed in ADR-028 § 3 after § 9, R2) |
| § 0.6 / § 4.3: the "everything else" wall (1 line left) | **Accepted** | ADR-028, V7, FA-10 |
| § 0.7 / § 3: where P and the 10x drafts disagree, P is mostly right | **Accepted** with the exceptions in § 3 above (kit size, CLI, hotkeys, cue matcher, hide rule) | § 3 |
| § 0.8 / § 5: factual errors (#59 exists; ADR-003 trigger is per file; capture only via `voxxad`; stale budget counts; B's follower confidence describes today's conductor; `release_config.rs` exists; `.cargo` folder; hash split) | **Accepted**, all corrected | ADR-021, ADR-023, plan V0/V12 |
| § 0.9 / § 6: prove first, in order: #59 → CPU decoder → corpus → offline robustness → GPU only if needed → timeliness → ADR + port | **Accepted** (GPU as above) | plan § 3 |
| § 0.10 / § 7: cut or defer the SQLite store, in-app editor, bindings executor, torch lab, sherpa probe, whisper-rs bump, GPU CI matrix, full kit, Prepare/Review screens | **Accepted except:** the kit is not cut to one skill (founder asked for skill sets); R1 mock and a U1 subset are kept | spec § 9 |
| § 2.3 items 1–2: Q11 ignored; UI extends Q11 | **Accepted** | plan § 1 |
| § 2.3 item 3: gap hold must ship with the follower | **Accepted** | ADR-022 |
| § 2.3 item 4: no follower merge before the speed question is settled | **Accepted** | V11 depends on V3/V4/V6 |
| § 2.3 items 5–6: founder IDs and ADR numbers collide | **Accepted** | FA-01…FA-30 (FA-31…FA-37 appended in critic round 1, § 8, nothing renumbered); ADR-021…029 |
| § 2.3 item 7: rename before publishing; NTFS case-only change does not move folders | **Accepted** | V16, FA-24 |
| § 3 row 9: hand-chosen stratified held-out, not a hash split | **Accepted** | spec § 2.3 |
| § 3 row 12: override hardware is a founder decision; HTTP pedal path is proven | **Accepted**, with S-13's key rules | FA-17 |
| § 4.2: Vulkan build panics without `VULKAN_SDK`; load-time import; Maxwell fp16; driver horizon | **Accepted** | ADR-021 |
| § 4.4: Windows-only assumptions; never set `-Ob2` repo-wide | **Accepted** | kit README, ADR-021 |
| § 4.5: consent gates the corpus; draft-truth accuracy on singing unverified | **Accepted** | FA-06, deferred lab |
| § 1.1 row: live CPU Small skips ~2 of 3 hops, so offline windows overstate what the follower sees | **Accepted** | spec § 2.3 "live fidelity" |

Nothing in review 21 was rejected outright.

## 5. Review 20 (safety) findings: accepted and rejected

| ID | Sev. | Ruling | How applied |
|---|---|---|---|
| S-01 | Critical | **Accepted** | no `run` key; refused by the validator; CLI not built in v1 (ADR-024, ADR-025) |
| S-02 | High | **Accepted** | service files only lower modes; `on_start`/`on_end` allowlist; `service_callable` (ADR-024) |
| S-03 | Medium | **Accepted, built later** | trust store outside the church folder with HMAC, for when CLI/HTTP church steps exist (ADR-025 § 5); v1 has no church-editable steps |
| S-04 | Medium | **Accepted** | allowlists compiled into the binary; no church overrides in v1 (ADR-024 § 1) |
| S-05 | Medium | **Accepted, built later** | refused file types, two interpreters by full path, copy-then-hash-then-run (ADR-025 § 5) |
| S-06 | Low | **Accepted** | AppToken regex, no `.`/`..`, one-pass substitution (ADR-024 § 3) |
| S-07 | Low | **Accepted, for API apps** | non-loopback warning, IP pin, identity pin (ADR-025 § 6) |
| S-08 | Medium | **Accepted, for API apps** | blank-wrapped multi-step moves; position unknown on failure (ADR-025 § 6) |
| S-09 | High | **Accepted** | target for every keystroke profile; `--target-exe` (ADR-025 § 1, V8) |
| S-10 | High | **Accepted** (confirmed at `dispatch.rs:163-175`) | per-key position, `position_uncertain` (ADR-025 § 2, V8) |
| S-11 | Medium | **Accepted, after the test card** | window class/title check (ADR-025 § 1, F3) |
| S-12 | Medium | **Accepted** | elevation check with a 40-line stop rule (V8) |
| S-13 | Medium | **Accepted** | F13–F24, checked F-keys, never Ctrl+Alt+arrows, modifier wait, no hotkey for Off (ADR-024 § 6) |
| S-14 | Medium | **Accepted except the keyboard hook** | Resume card in Presenter's terms with no default; off-by-one keys; the arrow-counting hook is not planned (reviewer's own recommendation) |
| S-15 | High | **Accepted** | arming by item pointer and song-anchor age, never conductor state (ADR-024 § 5) |
| S-16 | High | **Partly accepted** | the risk is accepted; the control differs: the pitch-based singing gate is deferred (review 21), so song start needs 2 agreeing windows, no lowered bar for the next song, and is a suggestion during spoken items; the TTS test is a release gate; the pitch gate returns if the test fails (ADR-022) |
| S-17 | High | **Accepted** | B's ordered matcher (ADR-024 § 5) |
| S-18 | Medium | **Accepted** | exposure bars: cues ≥ 300 opportunities or ≥ 30 h; DR ≥ 3 h talk-over-pad + ≥ 1 h hum, using public synthetic audio and CC-licensed public sermons, never private services (V11) |
| S-19 | Medium | **Accepted** | media and announcement items disarm audio triggers |
| S-20 | Low | **Accepted** | scripture suggest-only in v1 (Q18) |
| S-21 | High | **Accepted** | prediction off in v1; 0.4% per prediction budget; 99.6th-percentile firing (ADR-022 § 5) |
| S-22 | High | **Accepted** | fire-time re-check and cancellation |
| S-23 | High | **Accepted** | watchdog: 8 s newest window, 2 hops lag; DR anchor ≤ 8 s |
| S-24 | Medium | **Accepted** | DR needs ≥ 2 on-score tokens in 6 s; V6 measures how much DR availability this costs |
| S-25 | Medium | **Accepted** | only confirmed runs set thresholds; the θ floor applies when a pitch gate exists |
| S-26 | Medium | **Accepted** | no early song-end Right in v1 |
| S-27 | Medium | **Accepted** | one hide rule owned by the follower |
| S-28 | Medium | **Accepted** | credit slide in both copies; kind `credit`; slide-count check on keys-only apps |
| S-29 | Medium | **Accepted** | PD/CC0 only in public fixtures until FA-13; `fixtures_carry_an_allowed_licence` |
| S-30 | High (legal) | **Accepted** | `streams` flag; ND red, SA/NC amber; counsel (FA-15) |
| S-31 | Medium | **Accepted as a checklist** | "credit shown in <app>?" per CC song on the service-day checklist; no GUI now |
| S-32 | Medium | **Accepted** | imported rights arrive unchecked (ADR-026 § 6) |
| S-33 | Medium | **Accepted** | `ccli.active` in the church profile |
| S-34 | Low | **Accepted in spirit** | no SQLite; `public_fixture_ok` is 1 only for PD and CC0 in `licence-kinds.csv` |
| S-35 | High | **Accepted** | rehearsals first; talk muted at intake; consent from every recorded speaker; retention date; BitLocker (FA-06, FA-20) |
| S-36 | Medium-High | **Partly accepted** | logs move to `%LOCALAPPDATA%\com.voxxa.app\sessions\` (local app data; § 9, R1); every export drops talk words. **Rejected:** removing talk text from the session log itself, because replay of the default conductor reacts to unmatched speech (`aligner/mod.rs:86`) and must stay byte-identical (ADR-013) [UNVERIFIED beyond reading the config: replay inputs were not traced line by line] |
| S-37 | Medium | **Accepted** | words box unticked by default (FA-19, V9) |
| S-38 | Medium | **Accepted as a checklist; detection is a spike** | FA-29; [UNVERIFIED that Tauri can detect duplicate mode] |
| S-39 | Low-Medium | **Accepted** | v1 intake writes no transcript; `voxxa-eval --full` already logs to a temporary folder (`fixtures/README.md:281-282`) |
| S-40 | Low | **Accepted** | `.ignore` at the private root; kit agent settings deny reads there; new churches default to `%LOCALAPPDATA%\voxxa-church\` (§ 9, R1) |
| S-41 | Low | **Accepted** | `note` fields say "not for personal details"; usability with outsiders deferred |
| S-42 | High (pre-existing) | **Accepted** | aliases in all records; scrub decision FA-12 |
| S-43 | High | **Accepted** | 4-gram lyric scanner over the church's `songs\*.txt` (no SQLite); prototype stays private; equivalence test private (ADR-029) |
| S-44 | Medium | **Accepted** | no git in the private folder (FA-20) |
| S-45 | Medium | **Partly accepted** | prompts written fresh (done in `out/kit/`); `C:\Users\` paths and the deny-file refused. **Rejected:** refusing the generic `C:\dev\voxxa-church-private` path, which names no church and which the founder asked the kit to describe |
| S-46 | Medium | **Accepted** | `.gitignore` and media guard extended (V5) |
| S-47 | Medium-High | **Accepted, manifest replaced** | CODEOWNERS, connector deny list, skills say "never send data out"; tagged kit releases instead of a manifest generator (git tags already pin content) |
| S-48 | Low | **Accepted** | any new screen renders untrusted text with `textContent`/`escapeHtml`; CSP unchanged (V14, later editor screens) |

## 6. Corrections to the drafts (for the record)

- Drafts 10 and 12 said to file a new ticket for the release defect: **#59 exists** (`.10x/handoff.md:92`).
- Draft 10 put the cargo config in `src-tauri/.cargo/`: Cargo reads config from the working directory upward;
  the env route makes the question moot.
- Draft 11 quoted conductor 1,188/1,200 and everything else 2,315/2,317: current values are **1,173** and
  **2,316** (`.10x/handoff.md:111`, `:114`).
- Draft 12 read ADR-003's "~10 MB" review trigger as a total; it is per file (`003:41`).
- Draft 14 said practice capture needs no new code: true only for `voxxad`; the GUI writes no session log.
- Draft 14's `requires-python >=3.11` vs draft 12's 3.12: **3.12** chosen.
- Draft 13's R2 extended Q11 with tasks for screens that do not exist: removed.
- ADR numbers: drafts 10, 11 and 14 all claimed ADR-021; numbering here is 021–029, assigned once.

## 7. What survives from each draft

| Draft | Kept | Deferred or cut |
|---|---|---|
| 10 (A) | metric shape, hold-out discipline, `Option<FollowerConfig>`, gap hold, DR conditions, decoder diagnosis, hardware classes, truth v2 keys, intake stages 0–1 and 8–9 | time-based prediction (V6b only), pitch gate, signatures beyond 20 tokens, room calibration, 4 s/0.5 s grid, sherpa probe (Q20), draft-truth lab, public prototype |
| 11 (B) | three-file split, abstract actions, binding-profile format and six profiles as data, refusals, typed templates, cue matcher, priorities, W1–W5 audit (as tickets), #185 and Q12 merge | executor for API steps, CLI steps, TOFU, keyring, new drivers, export to apps, 4-service promotion gate (replaced by exposure bars) |
| 12 (C) | uv details (3.12, private lab), flag guards and cache keys, licence semantics as data, credit/TASL rules, sync-clause warning, private-folder layout, kit tools | SQLite store, library CLI, dependency-refresh programme, GPU CI matrix, kit manifest and semver, role prompts |
| 13 (UI) | Run screen subset, reason table, certainty words, Off confirm, accessibility fixes, words-to-use list, Resume card (with S-14's change), R1 mock | Prepare/Rehearse/Review screens, song editor, plan editor, R2 round, bare F6–F9, $400 |
| 14 (ponytail) | ordering, #59 by env, uv minimalism, JSON practice files, `.txt` + `songs.csv`, `--learn`, `--model`, prototype delete, rename timing, kit skeleton | `run` actions, bag-of-words cues, Ctrl+Alt chords, keeping 3 s/5 s for cold songs, allowing BY-ND outright |

## 8. Critic round 1 (2026-11-24)

A critic read every output file against the brief and found 20 gaps (10 major, 10 minor) and no safety
violation. The orchestrator ruled on each one; the parallel fixers applied the rulings with shared names
(church_root paths, file names, CLI forms, FA numbers). Every gap is closed by the files listed. Files other than
`spec.md`, `plan.md`, `founder-action-items.md` and `adjudication.md` were changed by the other fixers under the
same rulings; the table lists the files each ruling assigns.

| Gap | Sev. | Ruling (one line) | Files changed (under `out/`) |
|---|---|---|---|
| G01 | major | voxxad and voxxa-eval ship as a separate signed release asset `voxxa-tools-<version>-windows-x64.zip` (new step V5c, after V0, before church #2); the installer is unchanged; until then practice capture, `.wav` intake and `--check-layout` run only on the founder's booth; the fake church runs `voxxad --version` and `voxxa-eval --help` from the zip; FA-31 | `spec.md` (§ 0, § 3, § 4.4, § 4.9, § 7.8, § 9, § 11), `plan.md` (V5c, § 1–3, § 5, V12, V15), `founder-action-items.md` (FA-31, FA-02, FA-08), `kit/README.md`, `adrs/029-…` |
| G02 | major | the `.txt` importer keeps section labels as `sections` entries and marks `Credit:` slides `kind: "credit"` (optional, `#[serde(default)]`, replay unchanged); tests `txt_section_labels_fill_sections`, `credit_slide_kind_detected`, `old_setlists_replay_unchanged`; in V5 | `spec.md` (§ 4.3, § 4.4, § 4.5), `plan.md` (V5 step 9, V12) |
| G03 | major | FA-12 and V5 step 6 cover song titles too, by `path:line` only; the scrub PR merges before the deny-file scan gates `.10x/` and `fixtures/` | `founder-action-items.md` (FA-12), `plan.md` (V5 step 6, V15, § 5), `spec.md` (§ 8) |
| G04 | major | the public `fixtures/LICENSED_RUNS.md` template keeps `alias, licence_class, numbers` only; credit lines, titles and authors stay in the private `<church_root>\reports\LICENSED_RUNS.md` | `adrs/026-…` (§ 10), `plan.md` (V5 step 6), `spec.md` (§ 4.5) |
| G05 | major | new spec § 4.12 "Keeping voxxa current" reads "latest updates" as five things (signed app updates with a weekend window, a monthly dependency window, SHA-256-pinned models, kit versions, minimum uv); FA-32 asks the founder | `spec.md` (§ 0, § 4.8, § 4.12), `plan.md` (V1 step 6, § 5 tickets), `founder-action-items.md` (FA-32, FA-30), `adjudication.md` (§ 3 #22) |
| G06 | major | three founder items for the deviations: FA-33 (command lines out, not in v1), FA-34 (scripture and key-word graphics suggest-only), FA-35 (outbound hotkey sequences, with a chord example in two binding profiles marked [UNVERIFIED]) | `founder-action-items.md`, `spec.md` (§ 0, § 4.6, § 9), `kit/templates/bindings/keystroke.universal.json`, `kit/templates/bindings/keystroke.worshiptools_presenter.json` |
| G07 | major | new templates for the fixtures manifest, per-fixture intake and truth v2; the wav-intake checklist creates `intake.json` and holds the held-out freeze steps | `kit/templates/fixtures-manifest.template.json`, `kit/templates/intake.template.json`, `kit/templates/truth-v2.example.jsonl`, `kit/templates/wav-intake-checklist.md`, `spec.md` (§ 4.2, § 4.9, § 7.8), `plan.md` (F1, V5 step 8) |
| G08 | major | new `recording-consent.template.md` (who, what, purpose, booth-only storage, talk muted, retention, withdrawal within 7 days, minors, signature; "not legal advice") | `kit/templates/recording-consent.template.md`, `kit/private-folder.md`, `founder-action-items.md` (FA-06), `spec.md` (§ 8) |
| G09 | major | shadow Sundays are scored from a desk recording (needs FA-06 option (b)), tapped truth, an offline `voxxad --stdin-audio <wav> --shadow` replay with live-fidelity hop dropping, and `voxxa-eval --actions`; the live log is a cross-check; operator notes are never the score | `spec.md` (§ 7.6), `plan.md` (V15), `kit/prompts/07-review.md`, `founder-action-items.md` (FA-06) |
| G10 | major | agent settings deny shell and PowerShell reads and copies under `<church_root>`, and say plainly that permission patterns are a seatbelt, not a lock | `kit/templates/agent-settings.example.md`, `spec.md` (§ 4.9) |
| G11 | minor | one practice CLI form everywhere: `voxxad --learn <session.jsonl> --setlist <file> --out <church_root>\practice` and `voxxad --confirm <song_fp> --practice-dir <church_root>\practice`; "back to the song start" is a learner move, so the "note restart times" step is dropped | `spec.md` (§ 4.4), `plan.md` (V12), `adrs/023-…` (§ 4), `kit/templates/practice-session-checklist.md`, `kit/prompts/05-practice-session.md`, `kit/skills/voxxa-practice-session/SKILL.md` |
| G12 | minor | ADR-026 wins: `presenter_slides` is required on keys-only apps (WorshipTools is one); the example rows are filled | `plan.md` (V5 steps 2 and 8), `spec.md` (§ 4.5), `kit/templates/songs.template.csv` |
| G13 | minor | one naming rule: profile action keys are `<abstract_action>` or `<abstract_action>.<via>`, where `via` is a name defined per profile and `hint` is implicit in every one; a layout graphic entry `graphics.<id>.<profile>` names a `via` that must exist in that profile, and a missing entry for the active profile falls back to `hint` with a `--check-layout` warning (made final in § 9, R3); `goto_item` → `goto.item`; the opensong placeholder is listed or replaced; `on_start`/`on_end` in the schema | `adrs/024-…` (§ 1–3), `kit/templates/layout.template.json`, `kit/templates/service.template.json`, `kit/templates/bindings/*.json`, `spec.md` (§ 4.6), `plan.md` (§ 1, Q12 row) |
| G14 | minor | `service_position` is per item in the service plan; ADR-024 § 4's wording fixed | `adrs/024-…` (§ 4) |
| G15 | minor | FreeShow read-back uses an index-only call if the API has one [UNVERIFIED]; otherwise the binding notes that the returned slide text is dropped in the driver and never logged, stored or bundled | `kit/templates/bindings/freeshow.json` |
| G16 | minor | shared paths: `%LOCALAPPDATA%\voxxa-church\` by default (changed in § 9, R1); `C:\dev\voxxa-church-private\pilot\` on the founder's booth; `C:\dev\voxxa-church-private\fake-church-2\` for the kit test; the lab in `C:\dev\voxxa-church-private\lab\`; `services\<date>-<slot>.service.json` compiles to `services\<date>-<slot>.setlist.json` | `spec.md` (§ 4.4, § 4.8, § 4.9, § 5, § 7.6, § 7.8), `plan.md` (V1, V5, V6, V12, V15, F1), `adrs/023-…` (§ 7), `adrs/029-…` (§ 2, § 6), `kit/private-folder.md`, the kit checklists and prompts that name setlists |
| G17 | minor | `pd_basis = us_published_95_years` plus a `published_year` column, public domain when `published_year <= current_year - 96` (songcheck computes it); other bases `author_died_70_years`, `traditional`, `dedicated_cc0` | `adrs/026-…` (§ 4), `spec.md` (§ 4.5), `plan.md` (V5 step 2), `kit/prompts/02-song-library.md`, `kit/templates/licence-kinds.csv`, `kit/templates/songs.template.csv` |
| G18 | minor | the wav-intake checklist copies audio, setlist and truth into `<church_root>\fixtures\{tune,heldout}\<date>-<id>\`, runs tap-truth from the repo path (it writes `truth.jsonl` next to the wav), and picks one desk channel with `-af "pan=mono\|c0=c<N>"` instead of `-ac 1` | `kit/templates/wav-intake-checklist.md` |
| G19 | minor | FA-36: the circle-logo icon set from `tauri icon` plus a simplified 16/32 px variant, in V16; FA-37: #185 moves from Q11 to Q12 | `founder-action-items.md` (FA-36, FA-37, FA-08), `plan.md` (§ 1, § 3, V16), `spec.md` (§ 3, § 9) |
| G20 | minor | backups are dated weekly copies (`robocopy "<church_root>" "E:\voxxa-backup\<yyyy-mm-dd>" /E /XD sessions inbox`), keep the last 8, test one restore each quarter, on a BitLocker drive; never `/MIR` | `kit/private-folder.md`, `spec.md` (§ 4.9), `founder-action-items.md` (FA-20) |

**Checked while fixing (on `866b51e`, line numbers only, no text printed):**
- G03: the critic's 36 lines in 8 files hold exactly. The booth PC's host name appears in 64 lines across 21
  files (`.10x/`, `docs/RELEASING.md`, `fixtures/`, `src-tauri/tests/worshiptools.rs`), not only
  `fixtures/README.md:288`. Plan V5 step 6 now lists them, and the founder runs the deny-file scan on the PR
  branch before it merges, so the church name and licence numbers (which agents cannot search for) are caught
  too.
- G02: the existing section type is `Section { kind: SectionKind, slides: Vec<usize> }`
  (`src-tauri/src/aligner/evidence/sections.rs:25-28`), so "label plus first slide index" is stored as the kind
  plus the slides from that index. Whether `SectionKind` covers every label the importer accepts is
  [UNVERIFIED]; an unmapped label is dropped, never guessed.
- G01: `voxxad` has no `--version` flag today (`src-tauri/src/cli/voxxad/cli.rs:30-44`); V5c adds it (≤ 3 lines in
  the voxxad row, § 9, R2).
  `voxxa-eval --help` exists (`src-tauri/src/cli/voxxa-eval/main.rs:49`).
- G05: no Rust code hashes model files today (no SHA-256 code in `src-tauri/src`), so model pinning is a new
  ticket. Updates are already operator-initiated and refused while listening (ADR-020 § 1); only the weekend
  window is new.
- G09: `voxxad --realtime` feeds a `.wav` at live speed (`src-tauri/src/cli/voxxad/main.rs:44`, `:249-255`), which
  is how the offline replay gets the live keep-up gate's hop dropping.

**Rulings that change earlier rulings:** § 3 #22 (dependency policy): a monthly agent-run window replaces "no
upgrades for their own sake" if the founder chooses FA-32 (a). § 3 #11 (practice capture): unchanged for the
founder's booth; other churches get `voxxad` from the V5c zip instead of a local build.

## 9. Critic round 2 (2026-11-24)

A re-checker read the round-1 fixes. Round 1 closed 19 of the 20 gaps; G13 was only partly fixed. It also found
2 major and 17 minor new issues. The orchestrator ruled on them as R1–R4 (R4 has 14 small fixes, a–n); the
parallel fixers applied them with the same names, paths and numbers. Repo facts behind R1 and R2 were checked on
`866b51e`: `productName` "Voxxa" (`src-tauri/tauri.conf.json:3`), `identifier` `com.voxxa.app` (`:5`), NSIS
`installMode` `currentUser` (`:77`), the voxxad row 552/600. Files are under `out/`; the table lists the files
each ruling assigns.

| Item | Ruling (one line) | Files |
|---|---|---|
| R1 (major) | the default `church_root` is `%LOCALAPPDATA%\voxxa-church\`, a sibling of the install folder `%LOCALAPPDATA%\Voxxa\` (the same as `%LOCALAPPDATA%\voxxa\` on NTFS) and of the app-data folder `%LOCALAPPDATA%\com.voxxa.app\`, which install, update, uninstall and the V16 rename never touch; session logs go to `%LOCALAPPDATA%\com.voxxa.app\sessions\` (app data, 14 days; what the uninstaller's "delete app data" option removes is [UNVERIFIED]); the trust store to `%LOCALAPPDATA%\com.voxxa.app\trust\`; both folders join the private folder's "never put it" list; the founder's `C:\dev\voxxa-church-private\{pilot,fake-church-2,lab}\` paths are unchanged | `spec.md` (§ 4.4, § 4.6, § 4.9, § 8), `plan.md` (V9, V16), `adjudication.md` (S-36, S-40, G16), `adrs/023-…` (§ 6, § 7), `adrs/025-…` (§ 5), `adrs/029-…` (§ 2), `kit/private-folder.md`, `kit/README.md`, `kit/prompts/01-intake.md`, `kit/skills/voxxa-church-setup/SKILL.md`, `kit/templates/agent-settings.example.md`, `kit/templates/church-setup-checklist.md` |
| R2 (major) | the voxxad row (48 lines left) holds only flag parsing and wiring: V5c `--version` ≤ 3, V8 `--target-exe` ≤ 5, V12 practice flags ≤ 20, V13 `--check-layout` ≤ 10 (≤ 38, 10 spare); the logic goes in the new practice and layout rows (300 each); the conductor takes ≤ 23 of its 27 (routing ≤ 20, `Slide.kind` ≤ 3) and the importers ≤ 40 | `adrs/028-…` (§ 3), `plan.md` (V5c, V8, V9, V12, V13), `spec.md` (§ 4.11), `adjudication.md` (§ 4, § 8 G01 note) |
| R3 (G13) | `via` is a name defined per binding profile; `hint` is implicit in every profile and never sends anything; a layout graphic entry is per profile (`graphics.<id>.<profile>`) and its `via` must exist in that profile; a missing entry for the active profile falls back to `hint` with a `--check-layout` warning; a key that exists runs only after its contract test passes; `service_callable` means no step written in `layout.json` of kind `http` or `cli` (abstract actions inside are allowed); prompt 03 step 2 uses `hint` when the active profile has no `show_graphic.<via>` key | `adrs/024-…` (§ 2, § 3), `spec.md` (§ 4.6), `plan.md` (V13 steps 2 and 7), `kit/prompts/03-layout.md` (steps 2 and 6), `adjudication.md` (§ 8 G13 line) |
| R4a | `keystroke.universal.json` adds `Home` to `allowed_keys`, so `goto.keys` passes its own allowlist | `kit/templates/bindings/keystroke.universal.json` |
| R4b | WorshipTools documents F6–F9 on its shortcuts page (confirm on the test card): FA-17 recommends (a) for the pilot, F13–F24 from a pedal or Stream Deck plus the HTTP API, keyboard F-keys only for other apps after the conflict check; Pause is F15 (pedal) or Space when voxxa's window is in front, built in V14; FA-18, F3 and spec § 11 say "documented, confirm on the card"; FA-02 is "one approval instead of eleven" | `founder-action-items.md` (FA-02, FA-17, FA-18), `spec.md` (§ 4.6, § 4.7, § 11), `plan.md` (V13 step 5, V14, F3) |
| R4c | the presenter test card's numbering is the reference (restore after F10 is T4); a new row T8 confirms the `goto.keys` chord (Home, or Right Right); the bindings cite "test card T8" | `kit/templates/presenter-test-card.md`, `kit/templates/bindings/keystroke.*.json`, `plan.md` (§ 2, § 3, V13, F3), `founder-action-items.md` (FA-18, FA-35), `spec.md` (§ 4.6) |
| R4d | the V plan costs ≈ $150–360, or ≈ $270–665 with the ponytail lane | `process.md` (§ 4) |
| R4e | one backup command everywhere: `robocopy "<church_root>" "E:\voxxa-backup\<yyyy-mm-dd>" /E /XD sessions inbox` | `spec.md` (§ 4.9), `founder-action-items.md` (FA-20), `adjudication.md` (§ 8 G20) |
| R4f | ADR-029's kit inventory lists the 5 new templates; its agent-settings paragraph describes the example as written (named-folder denies, shell patterns, "a seatbelt, not a lock"); the scrub count is "36 lines in 8 files (verified), list in plan V5 step 6" | `adrs/029-…` |
| R4g | the example agent settings also deny `consent\`, `reports\` and `licences\` (if present) for Read, Grep and the shell patterns | `kit/templates/agent-settings.example.md` |
| R4h | the consent template gets an "Audio recordings kept until ____" box, which the practice checklist's audio box refers to | `kit/templates/recording-consent.template.md` |
| R4i | arrangement labels: `v<n>` = the n-th Verse in file order, `c`/`c<n>` = Chorus (the first when no number), `b` = Bridge, `t` = Tag, `o<n>` = Other; lower case in files, upper case in printed output | `adrs/024-…` (§ 1), `spec.md` (§ 4.5), `kit/templates/service.template.json` (only if it already uses `_comment`-style keys) |
| R4j | the loader maps the kit id `keystroke.worshiptools_presenter` to the saved `keystroke_profile: "worship_tools_presenter"`; existing configs are not changed | `adrs/024-…`, `kit/templates/church-profile.template.json` (note field, if one exists) |
| R4k | CODEOWNERS covers `kit/prompts/**`, `kit/skills/**` and `kit/templates/bindings/**` in the spec too | `spec.md` (§ 4.9, § 8) |
| R4l | the tools step: download the zip and its `.sha256`, check it with `Get-FileHash -Algorithm SHA256`, unzip outside `<church_root>` (e.g. `%LOCALAPPDATA%\voxxa-tools\`), add that folder to PATH | `kit/templates/church-setup-checklist.md`, `kit/README.md`, `plan.md` (V5c step 6) |
| R4m | no founder-only numbers in `kit/`: "FA-06 option (b)" becomes "written consent that covers service audio (see recording-consent.template.md)"; "FA-35" in the binding `keys_note`s becomes "confirm on the presenter test card (T8) before use" | the `kit/` prompts, checklists and binding profiles that named them |
| R4n | the wav-intake checklist names `recordings\<date>-<id>\audio.wav` and `recordings\<date>-<id>\setlist.json` in full, relative to `<church_root>` | `kit/templates/wav-intake-checklist.md` |

**Checked while fixing (on `866b51e`, no text printed):** today both session-log defaults are roaming
`dirs::data_dir()` + `voxxa` (`src-tauri/src/cli/voxxad/main.rs:313-317`, `src-tauri/src/help/mod.rs:181`), so a
plain switch to `dirs::data_local_dir()` + `voxxa` would have put the logs inside the install folder; V9 now
targets `com.voxxa.app`. The 14-day rotation already exists (`main.rs:310-325`). Space starts and stops listening
today (`src/main.js:1583-1586`); V14 turns it into Pause/Resume on Run.
