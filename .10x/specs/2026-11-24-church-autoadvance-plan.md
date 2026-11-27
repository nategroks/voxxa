# voxxa — prompt pack addendum: the V steps (follower, practice memory, master layout, kit)

Date 2026-11-24. Adds steps to `.10x/specs/2026-11-11-prompts-10x-team.md` (Q0–Q27; Q11 next). Design:
`spec.md` (to be committed as `.10x/specs/2026-11-24-follower-layout-kit.md` once approved); ADR drafts 021–030
(`adrs/`, to be committed as Proposed and accepted one by one). Base `origin/main` @ `866b51e`.

**Round 3 (draft, 2026-11-26; nothing approved yet).** The autonomy review (`round2/50-adjudication.md`, rulings C1–C17 and R1–R17; build plan `round2/51-strategy-and-build-plan.md`) changes this plan: a FreeShow track (V3e, F5, V8b, F7), Sunday readiness and minder controls (V13a, V13s, V14, V18, F6), a church-side migration kit (V17), a lab aligner measurement (V3f), and modified V2, V3, V3b, V3c (conditional), V3d, V5 (split V5a/V5b), V7, V8 (V8-lite), V10, V11, V12, V14 and V15. Dropped: V6b. Deferred behind the first autonomous Sunday: V13b (layout engine), V5b, V5c, V16, V8 steps 2–4. The round-3 prompts are in `plan-round3.md` (a copy of `round2/52-prompts-round-3.md`); where a step below carries a **Round 3** line, that line and the round-3 prompt win over the older text. Founder items FA-46 … FA-72 are in `founder-action-items.md` § A2; ADR-032 (FreeShow driver, Tier A/B), ADR-034 (Sunday readiness) and ADR-035 (live-output migration) are drafted Proposed; ADR-033 is reserved for an aligner row.

## How to use

- Each prompt is the first message of a fresh Claude Code session in the repo root, opened with
  `/10x-Team:10x-team`, exactly as the Q pack (`.10x/specs/2026-11-11-prompts-10x-team.md:7`): read
  `.10x/handoff.md` and `.10x/status.md` first, work only inside the named phase, write the named state files,
  stop at the named gate.
- **Ponytail lane, every step (user rule 2026-11-11):** run the same prompt independently with `ponytail` (level
  `full`), plus `ponytail-review` for code steps or `ponytail-audit` for measurement and close steps, in a lane
  that does not see the 10x output. Code steps run as two lanes (`vN/10x`, `vN/ponytail`) off `main`, then a
  comparator, cross-reviews and two verifiers (safety lens, feasibility lens). Measurement and close steps run as
  10x + `ponytail-audit`. Record it in `.10x/reviews/YYYY-MM-DD-ponytail-vN.md` and adjudicate.
- **ID scheme:** `V0…V18`, `V3b`–`V3f`, `V5a`–`V5c`, `V8-lite`, `V8b`, `V13a`, `V13s` are agent steps (round 3 adds `V3e`, `V3f`, `V5a`, `V5b`, `V8-lite`, `V8b`, `V13a`, `V13s`, `V17`, `V18` and founder steps `F5`–`F7`); `F1…F4` are founder-only steps. `FA-nn` refers to
  `founder-action-items.md`. Sizes and bands as the Q pack (`:9`): S $3–8, M $10–25, L $25–50 at list price,
  ×1.5 rework on L; a ponytail lane roughly doubles a code step.
- **Standing safety block (applies to every prompt below, quoted by reference as "Safety: standard"):** no
  microphone; no keystroke dispatch; never the real identifier `com.voxxa.app`; `voxxad` only with
  `--shadow`, `--replay` or `--presenter none`; never read, print or copy lyrics, slide text, session-log words,
  `setlist.json`, `truth.jsonl`, `heard_*.jsonl` or audio under any private folder; examples and tests use
  made-up words or public-domain text; numbers-only output; never install a built installer; never touch the
  booth PC's installed voxxa; no GitHub write outside the founder's pre-approved batch; stop every process you
  start, by PID; private things are named by alias ("pilot church", "private song P1", "booth PC").
- **Standing budget rule:** never raise a cap; new code goes in the rows of ADR-028; if a change would exceed its
  stated share, stop and report.

---

## 1. Sequencing against Q11–Q27

| Q step | Change | Why |
|---|---|---|
| Q11 Close Increment 2′ | **unchanged; runs after V0** (or tests with a build that has the #59 fix, `.10x/handoff.md:92`). Not extended with new usability tasks. | a volunteer test on an unoptimised build feels slow; new screens do not exist yet |
| Q12 Read-back + setlist import | **absorbs** draft 11's slice 4 (`goto.item` for API apps, drift check) and #185 "Sync from Presenter" (read-only copy; moved from Q11, founder item FA-37); also fills the service plan's `app` refs once V13 exists | one place for read-back; Q11 stays the volunteer test |
| Q13 Recorded-traffic fixtures | **absorbs** the wire-format audit W1–W5 (FreeShow body, OpenLP v2 paths/auth, OpenSong paths, ProPresenter WS clear actions, PP REST version path); its contract tests are what enables binding-profile steps per app | needs live installs either way (FA-22) |
| Q14 Shadow mode + corpus export | **amended, S → M:** keeps the "Shadow (log only)" toggle, adds the GUI session record (#53) and a "Learn from this rehearsal" button wrapping `voxxad --learn`; corpus export mutes talk spans and needs every recorded speaker's consent | practice capture v1 runs through `voxxad` in V12; the GUI path follows |
| Q15 Close Increment 3′ | unchanged | — |
| Q16–Q17 Bible bundle, spoken references | unchanged | — |
| Q18 Scripture in the conductor | consumes `scripture` items from the service plan and the follower's song-active signal; **scripture stays suggest-only** until the exposure bar (ADR-024) | review S-20 |
| Q19 Close Increment 4′ | unchanged | — |
| Q20–Q21 sherpa-onnx, bake-off | **absorbs** draft 10's sherpa probe (E10); the bake-off reports the two "95%" numbers | no duplicate probe |
| Q22 Store, channels, support | **`usage.csv` superseded** by ADR-026 (service files are the usage record); add a credits export from `songs.csv` to the support artefacts | the church left CCLI |
| Q23 Close | unchanged | — |
| Q24 macOS, Q26 Linux | the kit is "Windows booth only" until these land; global hotkeys under Wayland are not supported (Q26 already disables keystrokes) | — |
| Q25, Q27 | Q27's 1.0 needs the Correct bar on ≥ 300 occurrences and a published On-time number | spec § 2.3 |
| **Round 3 (draft)** | Q11 unchanged, on Presenter. Q12 absorbs FreeShow `get_output` ids-only read-back; Q13 uses the V8b stub as the FreeShow fixture (adjudication § 4) | R17 |
| **New** | V0–V16, V3b, V3c, V3d and V5c below; V3b (onset trigger, ADR-030) and V3c (Whisper prompt A/B) right after V3, in the lab, before V6 fixes the live configuration; V3d (parts-map follower, ADR-031) after V3b, before V11; V5c (CLI tools zip) before church #2; V16 (lowercase rename and circle-logo icon) before the founder publishes v0.1.0 | brief § 7 |

**Superseded or cut from the drafts** (see `adjudication.md`): SQLite store and `voxxa-library` CLI; in-app library
editor (founder item FA-14); bindings executor for API apps beyond driver methods; cue engine auto mode; CLI steps;
torch/Demucs/wav2vec2 lab; whisper-rs 0.16 bump; GPU CI matrix; Prepare/Rehearse/Review screens; usability round 2.

## 2. Calendar and cost [est.]

Round 3 calendar (draft, from `round2/51` § 2.3), Nov 27, 2026 to Easter, Mar 28, 2027. Bounds: Christmas booth freeze Dec 18 – Jan 3 (FA-28); dark Sundays Jan 10/17/24, 2027; FA-48 decision end of January; migration window to Mar 21; Easter Mar 28, 2027.

| Week | Sunday | Agent steps | Founder and church |
|---|---|---|---|
| Nov 27 - Dec 3 | Nov 29 | V7; V3, V4; V5a; V2; V10; V18 card v0 as docs (C15); ADR-032 drafted Proposed | Decision batch "this week" (`round2/51` Part 3); FA-38/FA-39 answered: no hardware purchase, the current feed is the supported degraded condition; tap live4 (FA-45, 5 min); Nov 29 rehearsal recording (F1) if consent is in |
| Dec 4 - 10 | Dec 6 | V3b, V3d start; V8-lite; Q11 (after V0); V13a | F5 on a weekday (1 h); V3e checklist run (2 h) incl. FA-53 MIDI check; Dec 6 recording |
| Dec 11 - 17 | Dec 13 | V3e review and go/no-go; V8b starts on a go; V3d continues; V6 starts at >= 8 songs; V17/V18 kit files start (no booth) | Dec 13 recording, tapping <= 45 min; last booth day Dec 17; FA batch "before Dec 18" closed |
| Dec 18 - Jan 3 | Dec 20, 27, Jan 3 | **Booth freeze.** V8b finishes; V2; V13a; V13s; V17 and V18 kit text; V12 design; ADR amendments; no booth, no dark run | Christmas services on Presenter as today; FA-67 policy in force (no RDP Sat 18:00 - Sun 14:00) |
| Jan 4 - 10 | Jan 10  | V3f lab (needs V3e numbers); V6 gate at >= 10 songs; V11 starts (needs V6, V8b, V3d); V14 starts | **Dark Sunday 1**: FreeShow installed, no output; voxxa drives it in shadow; read-back logged; 15 min |
| Jan 10 - 17 | Jan 17  | V11; V14; V13s closes | **Dark Sunday 2**; stage 1 rollback triggers watched (VRAM > 1,800 MB) |
| Jan 17 - 24 | Jan 24  | V11 closes; V12 starts; V17 inventory counts drafted | **Dark Sunday 3**; repertoire inventory to the worship leader (FA-68) |
| Jan 24 - 31 | Jan 31 | V12; V15 prep; ADR-033 only if V3f and V6 both passed | **FA-48 decision** on three Sundays' numbers; if go: slide team's one-hour FreeShow basics session; first ten licensed songs re-imported; last Presenter-only Sunday |
| Feb 1 - 28 | Feb 7, 14, 21, 28 | V15; Q12/Q13 absorb the FreeShow fixture | **Double-build Sundays 1-4** (stage 2): FreeShow live, a volunteer at its keys, voxxa shadow scored from read-back; Feb 7 and 14 are the two scored shadow Sundays if green (gate 1); Thursday 18:00 rollback rule each week; earliest stage 3 (one promoted song, minder at PC) Feb 21 [est.] |
| Mar 1 - 21 | Mar 7, 14, 21 | close-out; V18 v1 card | FreeShow only from Mar 7 (Presenter kept installed and maintained for a month after Feb 7); stage 3 -> stage 4 earliest Mar 7 after two clean stage-3 services [est.]; Mar 21 last window Sunday |
| Mar 22 - 28 | **Mar 28 Easter** | none on the booth | Frozen configuration; the stage reached by Mar 21 is held; no rollout change on Easter (inferred) |
| after Easter | - | - | Stage 5 (no volunteer in the booth) needs 4 consecutive green stage-4 Sundays and >= 300 scored occurrences: not before late April 2027 [est.] |

Slip rule (round2/45 § 8, risk 5): a slip of three weeks pushes the migration past Easter into the summer. If FA-48
is not a go by Jan 31, the switch is re-decided after Easter and the Tier B path (suggest-only on Presenter, minder
at the PC) runs through Lent.

| Size | Steps (round 3) | One lane | With ponytail lanes and verifiers |
|---|---|---|---|
| S | V0, V1, V3, V4, V5a, V7, V3e, V8-lite, V9, V10, V13a, V15 (12) | $36–96 | |
| M | V2, V3b, V3d, V6, V8b, V3f, V12, V13s, V14, V17, V18 (11) | $110–275 | |
| L | V11 (1) | $37–75 (with rework) | |
| **Total** | 24 steps, plus V3c conditional | **≈ $183–446** | **≈ $340–830 (≈ $345–845 with V3c); headline ≈ $350–875** |

Dropped: V6b (FA-72 approved 2026-11-27; the Holyrics driver only ever as an ADR-036 bindings file on need). Deferred: V13b, V5b, V5c, V8 steps 2–4; V16 runs now (FA-72). The FreeShow track adds ≈ $37–95 agent cost and 6–8 founder hours over six weeks. Founder time is capped at 90 min per scored Sunday and 4 h a week (FA-71, 2026-11-27). Cash: ≈ $100–150 [est.] for FA-38 (b) and FA-39; $0–80 [est.] optional Companion hardware (FA-59); FreeShow $0. Budget gate (FA-49): V3e ≤ $20 now; V8b ≤ $60 on a V3e go; nothing for an aligner, tracker or new model before V6 reports and the GPU is measured with both apps open. Any step over $50 needs a yes; 1.5× the estimate stops the step.

Round-2 calendar and sizes (superseded by the table above; kept for the record):

| Week (2026) | Agent steps | Founder |
|---|---|---|
| 1 (Nov 24–30) | V0, V1, V2, V5 | FA-01…FA-09; FA-38…FA-42 (onset lab); FA-43…FA-45 (parts map, jumps); a first supervised Presenter test with the lab helper once FA-40 and FA-41 are answered; Sun Nov 29: first rehearsal recording (F1) if consent is in |
| 2 (Dec 1–7) | V3, V5c, V7, V10; Q11 (after V0) | booth timing run (V3); Vulkan SDK (FA-04); FA-31 (tools zip); Sun Dec 6 recording |
| 3 (Dec 8–14) | V4, V8, V9; V3b and V3c (after V3); V3d (after V3b and V5); V6 starts at ≥ 8 songs | Sun Dec 13 recording; truth tapping (~2 h); F4 on a weekday before the freeze |
| 4 (Dec 15–21) | V6 gate at ≥ 10 songs; V6b | approve ADR-022 with V6 numbers |
| Dec 18 – Jan 3 | **booth freeze** (no release or dev build on the booth PC; FA-28) | Christmas services |
| 5–8 (Jan 4–31) | V11, V12, V13, V14, V15; V16 before v0.1.0 publish | WorshipTools test card 2 (F3); 2 shadow Sundays |

| Size | Steps | One lane | With ponytail lane |
|---|---|---|---|
| S | V0, V1, V3, V3c, V5, V5c, V7, V9, V10, V15, V16 (11) | $33–88 | ≈ $55–150 |
| M | V2, V3b, V3d, V4, V6, V6b, V8, V12, V13, V14 (10) | $100–250 | ≈ $190–480 |
| L | V11 (1) | $37–75 (with rework) | ≈ $70–150 |
| **Total** | 22 steps | **≈ $170–415** | **≈ $315–780** |
| of which the first wave (V0–V6, not counting V3b, V3c, V3d or V5c) | 7 steps | ≈ $45–110 | ≈ $85–210 |

The limits are the founder's gates, Sundays (recordings, shadow runs) and live installs, not tokens (review 21 § 2.1).

## 3. Dependency graph

Round 3 (draft, `round2/51` § 2.1):

```
FA-10, FA-54, FA-65 -> V7 wiring row + ADR-028 rows + trace line (S, this week) --+-> V8-lite (S) ----+
                                                                                  |                   |
FA-46, FA-51, FA-55 -> F5 founder: FreeShow stable beside Presenter -> V3e probe (S) -> go? ----------+-> V8b FreeShow driver v2 (M) -> ADR-032
                                                 |                      |  numbers                    |           |
                                                 |                      +-> V3f aligner lab (S/M) ----+--- (ADR-033 only after V3f AND V6)
V0 -> V3 decoder (S) -> V4 GPU proof (S) -> V3b onset trigger (M; task 6 = input chain; task 5 -> dark FreeShow)
                              |                  |
V2 eval v2 (S/M) -------------+------------------+--------------------------------------------------+
F1 corpus (founder, Sundays) -+-> V6 held-out gate (M) <---- V3d parts map (M; FA-43, FA-45, FA-63)  |
                                       |                                                            |
V7 + V8b + V3d + V6 ------------------> V11 follower port, one module (L) -> V12 practice memory (M) |
V7 -> V13a hotkeys + routes (S) --+-> V13s Sunday readiness (M) <- V2 ------------------------------+
V10 run-screen mock (S) ----------+-> V14 minder UI (M) <- V13a, V13s
V3e go + FA-48 + FA-68 -> V17 migration kit, church side (M)
V10 words -> V18 minder training programme (M; card v0 as docs in week 1) -> F6 monthly drill
F5 + V8b -> F7 dark Sundays (Jan 10/17/24) -> FA-48 decision (end of January)
all above -> V15 close (S/M)
Q11 unchanged (Presenter) . Q12/Q13 absorb get_output ids-only read-back and the V8b stub
```

Round 2 (superseded):

```
V0 #59 ──► Q11 ──────────────────────────────────────────────────────────────┐
V0 ──► V3 CPU decoder ──► V4 GPU proof (full if V3 misses) ──┐                │
V1 uv ──► (private lab under uv) ─────────────────────────────┤                │
V2 eval v2 ───────────────────────────────────────────────────┼─► V6 follower eval ─► V6b prediction (opt.)
F1 corpus (founder, 3 Sundays) ───────────────────────────────┘        │
V5 kit + licences + .txt sections                                      ▼
V7 budget row ─► V8 dispatch safety ─► V9 log privacy ──────────► V11 follower port (L)
V10 Run mock (UI) ─────────────────────────────────────────────┐       │
F3 test card 2 ───────────────────────────────────────────────┤       ▼
                                                               ├─► V13 layout v1 ─► V14 Run screen ─► V15 close
                                                     V12 practice memory ◄──┘ (after V11)        V16 rename + icon (FA-24, FA-36)
V0 + V5 ──► V5c tools zip (FA-31) ──► church #2 (practice capture, .wav intake, layout check)
V3 + V2 ──► V3b onset trigger (gates, takes, F4 live test) ──► V11 onset module (config, default off)
V3 + V3b ──► V3c Whisper prompt A/B ──► V6 live configuration (model, prompt on or off)
V3b + V5 ──► V3d parts map (gates, flow set, detour takes, live test; FA-44 driver) ──► V11 parts map (config, default off)
```

---

## 4. The steps

### V0 — Release-build optimisation flags (#59)

- **Size** S · **Phase** 4 + 6 · **Owner** Sr Eng 3 (platform/data/tooling) · **Depends on** ADR-021 § 1 Accepted; FA-01 pre-approvals · **Unblocks** Q11, V3, every live number
- **Acceptance:** Windows-job-only env lines; new cache keys; CMakeCache flag check that fails without the lines; `release_config.rs` pins; local recipe documented; founder booth check: release build, Tiny, 30 s encoder, private song P1: compute ≤ 150 s (was 461 s).

> /10x-Team:10x-team
> Phase 4 + 6, roles: Staff Engineer + DevOps Engineer + QA Engineer. Precondition: ADR-021 Accepted (at least § 1) and the founder's pre-approval batch (push, PR, CI fixes, merge when green, the #59 comment). Read `.10x/handoff.md`, `.10x/status.md`, `.10x/specs/2026-11-24-follower-layout-kit.md` § 4.1 and ADR-021. Two lanes (`v0/10x`, `v0/ponytail`) off `main`, then the comparator, cross-reviews and verifiers, as in Q9W; record the ponytail lane in `reviews/YYYY-MM-DD-ponytail-v0.md`.
> 1. In `.github/workflows/build.yml`, Windows job only, add to `env:` `CMAKE_C_FLAGS_RELEASE: -MD -O2 -Ob2 -DNDEBUG` and `CMAKE_CXX_FLAGS_RELEASE: -MD -O2 -Ob2 -DNDEBUG`. In `ci.yml` (and `full-eval.yml` if it builds on Windows), a step with `if: runner.os == 'Windows'` appends both to `$GITHUB_ENV` before any cargo command. Never at workflow level and never in a `.cargo/config.toml`: `-Ob2` breaks gcc and clang.
> 2. Change every rust-cache key that can hold a Windows whisper-rs-sys build (today the fixed `release-portable-ggml` at `build.yml:165` and `:354`, and the Windows keys in `ci.yml` and `full-eval.yml`) to a new string: whisper-rs-sys reruns its build only on `VULKAN_SDK`, `HIP_PATH` and `AMDGPU_TARGETS`, so a cached unoptimised build would survive.
> 3. After the Windows release build, a bash step finds the newest `whisper-rs-sys-*/out/build/CMakeCache.txt` under the release target and fails with a `::error::` unless both `CMAKE_C_FLAGS_RELEASE:` and `CMAKE_CXX_FLAGS_RELEASE:` contain `O2`. Calibrate before merging: ask the founder to diff those two lines between a broken and a fixed local build folder and paste only the flag strings; if they do not differ, assert on the `<Optimization>` element of the generated ggml/whisper `.vcxproj` instead.
> 4. Extend `src-tauri/tests/release_config.rs`: it fails if either env line is missing from the Windows job, appears at workflow level, or a cache key reverts. Update the test and the workflow in the same commit (`.10x/handoff.md:99`).
> 5. Add `scripts/dev/msvc-release-flags.ps1` (sets both variables for the current PowerShell session, prints them) and the local recipe in `fixtures/README.md` and `docs/RELEASING.md`: run `cargo clean --manifest-path src-tauri/Cargo.toml -p whisper-rs-sys --release` once, then build in a shell that ran the script.
> 6. Draft the #59 comment (cause: the `cmake` crate 0.1.58 overwrites the MSVC release flags; fix; numbers) in `reviews/YYYY-MM-DD-v0-tickets.md` for the orchestrator to post under the founder's batch. No new ticket: #59 exists.
> Budget: no Rust change. Safety: standard; never build or run anything on the booth PC.
> State files: `decisions/{devops,qa}/church-autoadvance.md`, `status.md`, `handoff.md`, the tickets file. Exit: CI green on all three OSes; the flag check passes on windows-latest and fails on a scratch branch without the env lines (record the run IDs); a founder checklist item: on the booth PC, a release `voxxad` built with the script runs private song P1 with Tiny and the 30 s encoder in ≤ 150 s of compute (was 461 s), numbers only.

### V1 — uv for Python tooling; delete the Python prototype

- **Size** S · **Phase** 4 · **Owner** Sr Eng 3 · **Depends on** ADR-027 Accepted; FA-03 (uv installed on the booth); FA-09 (delete approved) · **Unblocks** the private lab under uv (V6), kit tools (V5)
- **Acceptance:** PEP 723 headers on the 6 stdlib scripts; `uv run scripts/loc_budget.py` prints the same rows as `python3 scripts/loc_budget.py`; prototype deleted; `.venv/` ignored; INTEGRATIONS bridges on `urllib` + PEP 723; CI unchanged; a private-lab `pyproject.toml` template in `kit/templates/lab/`; the booth's uv version recorded as the minimum in `kit/README.md`.

> /10x-Team:10x-team
> Phase 4, roles: Staff Engineer + SDE + DBA (dependency hygiene). Precondition: ADR-027 Accepted; the founder installed uv (`winget install --id astral-sh.uv -e`, `uv python install 3.12`) and approved deleting the prototype. Two lanes (`v1/10x`, `v1/ponytail`) off `main`; record `reviews/YYYY-MM-DD-ponytail-v1.md`.
> 1. Add a PEP 723 header (`# /// script`, `requires-python = ">=3.12"`, `dependencies = []`, `# ///`) to `scripts/loc_budget.py`, `scripts/qa/mock_pro7.py`, `scripts/release/{test_release_scripts,updater_manifest,winget_manifest}.py` and `src-tauri/tests/fixtures/pdf/make_pdfs.py`. Behaviour unchanged; they still run with plain `python3` (CI keeps `ci.yml:149`, `:153`).
> 2. Delete `main.py`, `voxxa/` and `requirements.txt` (the May prototype; nothing runs it, `.10x/decisions/cto/_index.md:8`). Add `.venv/` and `**/.venv/` to `.gitignore`.
> 3. `docs/INTEGRATIONS.md`: rewrite the foot-pedal and MIDI bridge examples with `urllib.request` and a PEP 723 header naming their real dependencies (`hidapi`; `mido` + `python-rtmidi`), run as `uv run bridge.py`.
> 4. `CONTRIBUTING.md` and `fixtures/README.md`: "install uv; run tools with `uv run`".
> 5. Add `kit/templates/lab/pyproject.template.toml` for a church's private lab (no build system, `requires-python = ">=3.12,<3.14"`, `[tool.uv] exclude-newer`, a `dev` group with pytest) and a README paragraph: the founder copies it into the shared lab `C:\dev\voxxa-church-private\lab\` (any other church: `<church_root>\lab\`), runs `uv sync`, and adds headers to private scripts only after the lyric scan (V5) reports them clean. Agents never open private scripts.
> 6. Record the minimum uv version (spec § 4.12 item 5): the founder pastes the booth's `uv --version`; this PR writes "uv ≥ <that version>" into `kit/README.md` (creating the file with that line if V5 has not merged yet; V5 commits the rest of the kit around it).
> Budget: none (no Rust). Safety: standard. Do not add uv to CI; do not add a root `pyproject.toml`. SHA-pinning `ci.yml` actions is a separate S ticket: file it in the tickets file.
> State files: `decisions/{sde,dba}/church-autoadvance.md`, `status.md`, `handoff.md`, `reviews/YYYY-MM-DD-v1-tickets.md`. Exit: `uv run scripts/loc_budget.py` and `python3 scripts/loc_budget.py` print identical rows (diff empty) on Windows and Ubuntu; `git ls-files '*.py'` lists no file importing a non-stdlib package without a PEP 723 header; CI green; a founder checklist item confirms `python --version` on the booth still prints 3.9 and `uv run --python 3.12 python --version` prints 3.12.

### V2 — Evaluation v2: the two "95%" numbers

**Round 3 (draft, 2026-11-26):** R11: add signed error, `early_300`, `early_1000` (decoupled from the 1 s match window), `tight_300`, `wrong_slide_s`, `--service-report` (numbers-only `service-<date>.json` and `trend.csv`) and `--truth-from-screen` (FreeShow read-back as operator-timed truth for the Correct bar only, never Tight); the eight baseline names stay; fit the 112 lines left in the voxxa-eval row (`--model` passthrough moves to V3, truth-v2 event rows to V6, one shared signed-delay helper) or stop and report; never raise a cap. Prompt: `plan-round3.md` § 2 where it has a block for this step.

- **Size** M · **Phase** 4 · **Owner** Sr Eng 1 (listening) · **Depends on** ADR-022 § 1 (definition) Proposed; FA-05 · **Unblocks** V6, V11
- **Acceptance:** `--model` passthrough; new columns; `--actions` input; the 8 baseline names and `baseline.json` unchanged; numbers-only; ≤ 100 lines in the voxxa-eval row.

> /10x-Team:10x-team
> Phase 4, roles: Senior Engineer + SDE + QA Engineer. Precondition: the founder approved the two-number definition (spec § 2.2; FA-05). Read ADR-015, ADR-022 and `src-tauri/src/cli/voxxa-eval/{main,score}.rs`. Two lanes (`v2/10x`, `v2/ponytail`); record `reviews/YYYY-MM-DD-ponytail-v2.md`.
> 1. `--model <name>` is passed through to `voxxad` in `--full` (Tiny is hard-coded at `main.rs:188`); default stays `tiny`.
> 2. New report columns, computed from the existing hits and truth, without changing the 8 names in `score.rs:116-125` or `fixtures/baseline.json`: `on_time_1500` (share of non-song-start hits with latency ≤ 1,500 ms), `song_start_median_ms` and `song_start_p95_ms` (first slide of each song), `wrong_per_10_songs` (gotos to a slide that is neither the current nor the next truth row, plus skips of ≥ 2, scaled per 10 songs), `hidden_mid_song` (blanks > 1 s inside a song where the truth has none).
> 3. `--actions <file.jsonl>`: score an external list of action lines (the `voxxad` stdout format) against a fixture's `truth.jsonl`, so the private follower prototype is scored by the same scorer. Errors name the file and line number only.
> 4. Optional truth v2 keys (`last_ms`) and event rows (`talk_start`, `talk_end`, `song_end`) are accepted and ignored by the current metrics; a unit test proves an old truth file scores identically.
> 5. Unit tests with made-up fixtures (synthetic truth and action lists) for every new column, including a song start, a skip, a wrong move and a mid-song blank.
> Budget: voxxa-eval row 488/600 (`.10x/handoff.md:118`); ≤ 100 lines, else stop and report. Safety: standard; output stays numbers-only (ADR-019 § 7).
> State files: `decisions/{sde,qa}/church-autoadvance.md`, `status.md`, `handoff.md`. Exit: replay eval on the committed fixtures prints the same 8 numbers as before plus the new columns; `baseline.json` untouched; CI green.

### V3 — CPU decoder diagnosis and state reuse

**Round 3 (draft, 2026-11-26):** decoder settings plus a pass-time p95 amber on every session-end line; takes V2's `--model` passthrough (R11). Exit adds: Small ≤ 1.0 s/pass on the booth release build decides whether V3c runs. Prompt: `plan-round3.md` § 2 where it has a block for this step.

- **Size** S · **Phase** 2 check + 4 · **Owner** Sr Eng 1 · **Depends on** V0 merged; ADR-021 § 2 Accepted; one founder booth timing run (no service that day) · **Unblocks** V4 scope, V3b, V3c, V6 live configuration
- **Acceptance:** settings behind configuration (defaults unchanged); per-pass encode/decode/fallback numbers logged; booth s/pass reported for Tiny and Small at the 6 s encoder; decision rule applied; follower on cached P1 windows still 11/11, 0 wrong under the new decoding (founder runs, private); the sung-audio slowdown is diagnosed on a founder sung take, not only the TTS fixture (Small 2.57 s/pass on sung audio vs 0.85 s on TTS, GTX 960 Vulkan dev build; Presenter is not the cause, VRAM 814 of 2048 MB) with `temperature_inc 0` and `max_tokens 48` (lab result with the local-only `VOXXA_TEMP_INC` and `VOXXA_MAX_TOKENS` switches, voxxa-private, never pushed: 2.57 → 0.94 s/pass with `temperature_inc 0`, 0.53 s/pass with also `max_tokens 48`, follower unchanged; V3 confirms on the release build with `reuse_state`).

> /10x-Team:10x-team
> Phase 2 check, then Phase 4, roles: Senior Engineer + SDE. Precondition: V0 merged; ADR-021 Accepted. Read `src-tauri/src/transcription/mod.rs:139-204` and `recognizer.rs`. Two lanes (`v3/10x`, `v3/ponytail`); record `reviews/YYYY-MM-DD-ponytail-v3.md`.
> 1. Add recognizer settings, read from configuration and logged on the session start line, with today's behaviour as the default: `reuse_state` (one `WhisperState` per recognizer instead of one per window, `mod.rs:179-181`), `temperature_inc` (default whisper.cpp's), `max_tokens`, `n_threads`, `audio_ctx` (per model; the local-only `VOXXA_AUDIO_CTX`, `VOXXA_TEMP_INC` and `VOXXA_MAX_TOKENS` switches are voxxa-private and never pushed: write the settings fresh).
> 2. Log per pass, numbers only: wall-clock ms for the encode and decode phases if whisper-rs 0.13 exposes whisper.cpp's timings (check; otherwise total ms per pass) and the fallback count if available. Mark anything not exposed as not measured.
> 3. A `voxxad --bench <wav>` mode is not needed: use `voxxad --stdin-audio <public PD fixture> --shadow --model <m>` and the logged numbers.
> 4. Prepare a founder checklist for one booth run on a release build with the V0 flags: Tiny and Small, 6 s encoder, with (a) defaults, (b) `temperature_inc 0`, `max_tokens 48`, `reuse_state`, `n_threads` = physical cores. The founder pastes s/pass and fallback counts only. They also re-run the private follower prototype on windows cached with setting (b) and reports goto counts and wrong moves only.
> 5. Write the decision into ADR-021's record: Small ≤ 1.0 s/pass → CPU at a 1 s hop; ≤ 1.5 s → V6 also tests a 1.5 s hop; otherwise V4 runs in full.
> Budget: recognizer row 432/700; ≤ 60 lines. A recognizer default change would force re-recorded fixture logs, so defaults stay as they are in this step. Safety: standard.
> State files: `decisions/sde/church-autoadvance.md`, `status.md`, `handoff.md`, `reviews/YYYY-MM-DD-v3-decoder.md` (numbers only). Exit: tests cover the new settings' parsing and defaults; replay unchanged; the decision is recorded with the founder's numbers.

### V3b — Onset trigger: lab prototype, gates and supervised live tests

**Round 3 (draft, 2026-11-26):** task 6 is the input chain per R6: **FA-38 and FA-39 answered 2026-11-26: hardware deferred; the full mix on the current unbalanced, low-level feed is a supported, degraded condition** (R6's "hardware first" is withdrawn), so software carries it: a 100 Hz high-pass, an energy gate on the 300–4000 Hz band OR'd with Silero (config, default off), level and hum on the start line and in the health check, `vad/` moved to the recognizer row in the same PR (ADR-028 amendment), windows do not follow vocal activity in v1. Task 5 may target a non-projected FreeShow after V3e. Exit adds: replay byte-identical with the gate off. Prompt: `plan-round3.md` § 2 where it has a block for this step.

- **Size** M · **Phase** 2 (evidence) + 4 · **Owner** Sr Eng 1 (listening) + QA Engineer · **Depends on** V3 (passes near 1 s: at 2.6 s passes the trigger barely helps); V2 merged (`--actions`); V1 (lab under uv); ADR-030 Proposed (`adrs/030-phrase-onset-trigger-arm-on-words-fire-on-onset.md`); FA-40, FA-41; FA-38 and FA-39 answered (deferred; band and full-mix takes run on the current feed as the required degraded-mode row) · **Unblocks** F4; ADR-030 acceptance; the onset module in V11 (behind config, default off); a measured "Tight on time"
- **Acceptance:** the gate cases pass in the lab (founder-run, numbers only); `voxxa-eval` reports Tight on time; every founder take is scored words only and with the trigger at live-fidelity pass times (1 s and the booth's measured s/pass); on solo-vocal takes at 1 s passes, proposed bars from T1 and T2: median |delay| ≤ 0.5 s, Tight on time ≥ 0.6, 0 wrong slides, no onset fire earlier than −300 ms; on band and full-mix takes on the current feed (the required degraded-mode row, FA-39), recall and wrong moves no worse than words only, 0 onset fires on band energy, and the trigger disarms by itself while the level line flags the feed `degraded`; F4 done (dry run, then keys) with 0 wrong slides; the port plan and the input-chain ticket written; kit prompts check level, hum, channel and feed.

> /10x-Team:10x-team
> Phase 2 (evidence), then Phase 4, roles: Senior Engineer + QA Engineer + SDE + Security Engineer (the key-press guard). Precondition: V3's decision is recorded; V2 merged; ADR-030 Proposed; the founder answered FA-40 and FA-41. Read ADR-019 § 2, ADR-022, ADR-030 and spec § 2.2. The lab prototype exists in the shared lab `C:\dev\voxxa-church-private\lab\`: `predict.py` (the follower plus the onset trigger, offline) and `live.py` (the live helper). Agents never open the lab source, the takes or any cached window (V6 rule): **the founder runs every lab command from a checklist you write and pastes numbers only.** The code tasks (1, 2, 6, 7) run as two lanes (`v3b/10x`, `v3b/ponytail`); the measurement tasks (3–5) run as 10x + `ponytail-audit`; record `reviews/YYYY-MM-DD-ponytail-v3b.md`.
> Lab baseline (Small, 6 s encoder, 1 s passes, results arriving 1 s after their audio, overlapping passes dropped; median |delay| of every slide after the first, words only → with the trigger): T1 (the founder's solo take of It Is Well, 11 changes) 1.90 s → 0.18 s, within ±300 ms 10% → 80%, 7 onset fires, no wrong slide; T2 (the public TTS fixture, 6 changes) 2.12 s → 0.04 s, 20% → 60%, no early fire; P1 (band) 4.30 s → 4.01 s, 11 of 11 both ways. At 2.6 s passes the gain mostly disappears on the solo takes (T1 3.28 s → 2.66 s; T2 3.84 s → 3.72 s); P1 6.40 s → 4.36 s and 9/11 → 10/11.
> 1. **Gate cases (public, synthetic).** Add `scripts/lab/onset_gate_cases.py` (PEP 723, stdlib only, fixed seed, `--self-test`). It writes synthetic WAVs and JSON case files with made-up words into a folder named on the command line, never into the repo, each with its expected result. Detector cases (300–4000 Hz band energy, 10 ms frames; the floor follows quiet down at once and rises at most 2 dB/s; onset = above floor + 10 dB for 60 ms after ≥ 200 ms below floor + 5 dB): `onset_after_breath` (one onset within 30 ms), `no_onset_without_breath`, `hum_alone_never_onsets` (60, 180, 300 and 420 Hz at −60 dBFS), `onset_floor_starts_from_real_audio` (leading digital silence), `floor_rises_at_most_2_db_per_s`. Decision cases (a made-up setlist, a heard-word timeline at a given pass time, an onset list; the five failure-mode names are ADR-030's): `fire_on_onset_after_arm`, `onset_arms_only_on_last_two_words`, `no_onset_on_slide_with_repeated_closing_words` (the slide's final 3 words occur earlier in it more than 4 words away), `onset_move_keeps_word_search_width`, `no_dr_through_quiet` (a 10 s pause), `first_slide_onset_needs_cue_and_quiet` (song cued on its leading blank, ≥ 2 s of quiet), `no_onset_moves_on_words`, `at_most_3_steps_never_left`. No committed audio (media guard).
> 2. **Tight on time.** Add the `tight_300` column to `voxxa-eval` (spec measure "Tight on time": share of non-song-start occurrences shown within ±300 ms of the first sung word), with a unit test on a made-up truth and action list. The 8 names and `baseline.json` stay unchanged.
> 3. **Founder checklist: the gates in the lab.** The founder pastes `uv run predict.py --help` (flag names only); the checklist runs every task-1 case through the prototype; the founder pastes pass or fail per case name. A case the prototype cannot read is "not run", never guessed. Any fail: stop and report the case name.
> 4. **Founder checklist: score every take.** For T1, T2, P1 and each new founder take (public-domain songs first; truth tapped with `fixtures/tools/tap-truth.ps1` after the reaction-lag calibration, 524 ms last time): run the prototype words only and with the trigger, at live-fidelity pass times for 1 s and for the booth's s/pass from V3; score both action lists with `voxxa-eval --actions`. Paste per take alias: median |delay|, Tight on time, recall, wrong slides, early moves and how early, onset fires vs word fires, and the detector's hits within ±300 ms of slide starts. Band takes go in their own rows: they are the **degraded-mode row** (FA-39, required, not optional), and they also report the share of time the level line flags `degraded` and any onset fire on band energy (expected 0).
> 5. **Founder checklist: supervised live Presenter test (F4).** (a) A weekday with no service or rehearsal. The first run may use today's feed and vocal mic, as the lab did (the helper's 100 Hz high-pass and +12 dB); every run uses this feed until FA-38 changes (deferred 2026-11-26); its floor, level and hum share are recorded as the degraded-mode baseline. (b) Level and hum check: Line In, the voice channel selected, 100 Hz high-pass, +12 dB; paste the floor and the singing level in dBFS. (c) Presenter shows the FA-41 test song on its leading blank. (d) `live.py --dry-run` for one full song: it logs the moves it would make and presses nothing. (e) Then the same song with keys: `live.py` presses Right only while a Presenter.exe window is in front (the ADR-019 § 2 guard), never Left, at most 3 steps per move; the founder stays at the keyboard, stops it with Ctrl+C at the first wrong move and fixes the slide by hand; `--minutes` caps every run. First-slide mode (the first sung phrase after ≥ 2 s of quiet shows the first lyric slide) is for these supervised tests only: a cough after silence also triggers it. (f) Record the feed during the run, tap truth and score it as in task 4. The helper runs a local-only voxxad build with streaming stdin (`--stdin-audio - --realtime`, `--shadow`; voxxa-private, never pushed) and logs numbers only.
> 6. **Port plan and input chain.** Write `reviews/YYYY-MM-DD-v3b-port-plan.md` for V11: the detector and the arm and fire rules as `src-tauri/src/aligner/follower/onset.rs`, behind `FollowerConfig.onset: Option<OnsetConfig>` with `#[serde(default)]` = `None` (default off; replay byte-identical); the task-1 cases become Rust tests with the same names; an onset fire passes every ADR-022 gate (fire-time re-check, watchdog, operator wins) and the existing dispatch path; a size estimate inside the follower row (ADR-028). Add one line to `.10x/handoff.md` so the V11 session reads ADR-030 and this file. File the input-chain ticket from § 5 (FA-02 batch) for what voxxa lacks today: channel select, a 100 Hz high-pass and gain or AGC before the Silero VAD (raw singing at about −34 dBFS gave 2 Whisper passes in 200 s; the same take +12 dB gave 152), and a level and hum check at setup.
> 7. **Kit.** `kit/templates/church-setup-checklist.md` (§ 8, audio feed), `kit/prompts/05-practice-session.md`, `06-live-readiness.md` and `kit/templates/service-day-checklist.md` check the input level, hum, the voice channel and whether the feed is vocal-only or a full mix (both supported; the mix is the degraded mode); `07-review.md` reports Tight on time and onset fires. Made-up words only.
> Budget: voxxa-eval row ≤ 20 lines; no other Rust (the port is V11's). Safety: standard for agents (no microphone, no keys, no lab source). Only the founder runs the helper that presses keys, under task 5's rules.
> State files: `decisions/{sde,qa,security}/church-autoadvance.md`, `status.md`, `handoff.md`, `reviews/YYYY-MM-DD-v3b-onset.md` (numbers only), the port plan, tickets file. Exit: CI green; every gate case passes in the lab; the take table is filled; F4 is done with 0 wrong slides, or the failing case and take are named; ADR-030 can be accepted or amended.

### V3c — Whisper prompt from the follower (bias A/B)

**Round 3 (draft, 2026-11-26):** **conditional** (R7): runs only if the booth release build shows Small > 1.0 s/pass. If run: a spoken-form tail prompt of the current slide's remaining line (≤ 40 tokens), never the planned next part at a part end (it biases against detours), and detour detection on F1–F4 in its acceptance. Prompt: `plan-round3.md` § 2 where it has a block for this step.

- **Size** S · **Phase** 2 (evidence) · **Owner** Sr Eng 1 (listening) · **Depends on** V3 (decoder settings fixed for every arm); V3b tasks 3–4 (the takes and their scoring); FA-42 · **Unblocks** the V6 live configuration (Base with the prompt, or Small); an ADR-004 amendment; the prompt source in V11 (behind config, default off)
- **Acceptance (recorded takes only, never the held-out set; live-fidelity pass times):** an A/B table per take alias: recall, wrong slides, median |delay|, Tight on time, phantom words, moves caused by a phantom word, s/pass median and p95; Base with the prompt is chosen only if it is no worse than Small unbiased on recall and wrong slides on every take, causes no move from a phantom word, and runs ≤ 1.0 s/pass on the booth; otherwise Small stays and the prompt stays off.

> /10x-Team:10x-team
> Phase 2 (evidence), roles: Senior Engineer + QA Engineer. Runs as 10x + `ponytail-audit`; record `reviews/YYYY-MM-DD-ponytail-v3c.md`. Precondition: V3's decision recorded; V3b's take table exists; the founder approved FA-42. Read ADR-004 (E6: why `--bias` is off by default), ADR-021, ADR-022 and ADR-030. Same founder-runs-it rule as V6: agents never open the lab source, the takes, cached windows or any prompt text.
> Context: `voxxad --bias` passes slide text as Whisper's `initial_prompt`. It is off by default because it may make Whisper write words nobody sang. Today the text is the conductor's current and next slide, so it follows the conductor, which lags the singer: 2 of 11 changes on T1. On the booth's GTX 960 (Vulkan dev build), on sung audio, Base runs 1.13 s/pass and Small 2.57 s/pass; unbiased, the follower got 5 of 11 changes on T1 with Base and 9 of 11 with Small.
> 1. Write the founder checklist for a lab run in which the prompt comes from the follower's position: the current slide's remaining line plus the next slide's first line, ≤ 40 tokens, rebuilt every pass. The lab hands it to Whisper through a local-only hook (voxxa-private, never pushed).
> 2. Guards, all on in every prompted arm: decode only while the VAD hears voice; heard words may not outrun the onset and voice timeline (a match that puts the singer further ahead than the voiced time since the last anchor allows is dropped); count phantom words (words matched while the VAD hears no voice) per pass.
> 3. Arms, on T1, T2, P1 and V3b's new takes, at each arm's measured s/pass and at 1 s, each words only and with the onset trigger: (A) Small unbiased; (B) Base with the follower prompt; controls (C) Base unbiased and (D) Small with the follower prompt. Score every action list with `voxxa-eval --actions`.
> 4. Write `reviews/YYYY-MM-DD-v3c-bias-ab.md`: the acceptance table per arm and take alias, and the verdict. If (B) passes, draft the ADR-004 amendment (prompt source = the follower's position, the guards, off by default) and a port note for V11 (`FollowerConfig.prompt`, default `None`). If it fails, say which guard or take fails; never tune on the held-out set.
> Budget: none (lab only; no Rust). Safety: standard; the prompt is lyric text, so it is never logged, printed or pasted; numbers only.
> State files: `status.md`, `handoff.md`, the review file, an addendum to ADR-021's record (the live model choice). Exit: the founder has the A/B table and a verdict; V6's live configuration names its model and whether the prompt is on.

### V3d — Parts-map follower: gates, flow set, detour takes and a live test

**Round 3 (draft, 2026-11-26):** adds the R3 break-gate arm (lab switch, FA-63; live4 never cited), the OLTW-within-part arm, the R14 lab must-fixes (planned-next entry from mid-part gated and no pointer rewind inside a review; `elim` behind a lab flag with a spoken-aside take; `live.py` uses voxxad's `t`; mic-queue depth in the status line; pytest `test_parts.py` and `test_onset.py` from the ADR texts), detour, spoken-aside and two-singer takes, timing columns within 0.5/1.0/1.5 s, and the port's line counts with a row proposal (R12). Depends on FA-43, FA-45, FA-63. Task 5's live test follows FA-44's answer: non-projected FreeShow (V3e) or Right-only on Presenter. Prompt: `plan-round3.md` § 2 where it has a block for this step.

- **Size** M · **Phase** 2 (evidence) + 4 · **Owner** Sr Eng 1 (listening) + QA Engineer · **Depends on** V3b (the onset trigger and its take table); V5 (sections from the `.txt` importer and kit song files); ADR-031 Proposed (`adrs/031-parts-map-follower-plan-is-a-hint.md`); FA-43; FA-44 before any live test that leaves the plan · **Unblocks** ADR-031 acceptance; the parts map in V11 (behind config, default off); a live test of flowing worship
- **Acceptance:** ADR-031's gate cases pass in the lab (founder-run, numbers only); `voxxa-eval` reports wrong-slide seconds and scores by slide id; on every planned-order take and every flow-set take, the parts map has no fewer right changes than the plan-order follower, fewer wrong moves and wrong-slide seconds in total, and no move earlier than −1 s on solo-vocal takes; at least 3 founder takes with real detours (sung, not re-cut: an extra chorus, a skipped verse, a repeated last line) are scored; a supervised live test with FA-44's driver shows 0 wrong slides (or, before FA-44, Right-only mode with the operator taking over at the first jump); the port plan is written.

> /10x-Team:10x-team
> Phase 2 (evidence), then Phase 4, roles: Senior Engineer + QA Engineer + SDE + Security Engineer (the driver and key guard). Precondition: V3b's take table exists; V5 merged (sections); ADR-031 Proposed; the founder answered FA-43 (and FA-44 before task 5). Read ADR-016, ADR-019 § 2, ADR-022, ADR-030 and ADR-031. The lab prototype exists in the shared lab `C:\dev\voxxa-church-private\lab\`: `parts.py` (the parts-map follower), `flows.py` (re-cuts a tapped take into other part orders) and `evalparts.py` (plan-order follower vs parts map, numbers only). Agents never open the lab source, the takes or any cached window (V6 rule): **the founder runs every lab command from a checklist you write and pastes numbers only.** The code tasks (1, 2, 6) run as two lanes (`v3d/10x`, `v3d/ponytail`); the measurement tasks (3–5) run as 10x + `ponytail-audit`; record `reviews/YYYY-MM-DD-ponytail-v3d.md`.
> Lab baseline (Small, 6 s encoder, live decoder settings, 1 s passes), plan-order follower → parts map, over T1, T2, F1–F4 (re-cuts of T1: extra chorus, skipped verse, chorus first, last line repeated) and live4 (the first live test): right changes 49 → 74 of 77, wrong moves 31 → 4, early by more than 1 s 14 → 1, wrong-slide seconds 327 → 112. P1 (band, no labels) stays on the plan-order follower (11 of 11).
> 1. **Gate cases (public, synthetic).** Add `scripts/lab/parts_gate_cases.py` (PEP 723, stdlib only, fixed seed, `--self-test`): made-up songs with parts, heard-word timelines and onset lists, written to a folder named on the command line, never into the repo, each with its expected moves. ADR-031's names: `shared_line_never_moves_alone`, `off_plan_needs_two_windows`, `no_jump_between_look_alike_slides`, `no_skip_to_look_alike_too_soon`, `no_repeat_branch_on_self_repeating_slide`, `lost_searches_forward_first`, `elimination_shows_planned_part`, `guess_corrected_within_review`, `part_entered_at_first_slide`, `onset_fires_verse_to_chorus_only`, `no_dr_move_in_song_with_parts`, `unlabelled_song_uses_plan_order`. No committed audio.
> 2. **Wrong-slide seconds.** Add the `wrong_slide_s` column to `voxxa-eval` (spec § 2.2) and score moves by slide id, so a take that leaves the plan scores; unit tests on a made-up truth and action list. The 8 names and `baseline.json` stay unchanged.
> 3. **Founder checklist: the gates and the flow set.** The founder runs every task-1 case through `parts.py` and pastes pass or fail per name; then re-cuts each tapped solo take with `flows.py` into the four orders, runs `evalparts.py` on all takes and pastes the per-take numbers (right, wrong, early, late, wrong-slide seconds) for both followers.
> 4. **Founder checklist: takes with real detours.** The founder sings (or records at a rehearsal, with consent) at least 3 public-domain songs with detours chosen on the spot: an extra chorus, a skipped verse, the last line repeated, a return to verse 1. Truth tapped after the reaction-lag calibration; scored as in task 3.
> 5. **Founder checklist: supervised live test of flowing worship.** As V3b task 5, with the driver FA-44 chose. Before FA-44: Right-only mode (the helper sends Right when the target is ahead, at most 3 steps; for any other move it logs the target, sends nothing and stops pressing keys for that song; the founder takes over). The founder sings with at least 2 detours; Ctrl+C at the first wrong move; record, tap and score.
> 6. **Port plan.** Write `reviews/YYYY-MM-DD-v3d-port-plan.md` for V11: the parts map as `src-tauri/src/aligner/follower/parts.rs` behind `FollowerConfig.parts: Option<PartsConfig>` with `#[serde(default)]` = `None`; the task-1 cases as Rust tests with the same names; the `follow` line's `why` values `jump`, `elim` and `lost`, with `part` and `ptr`; the driver interface for a jump (FA-44); a size estimate inside the follower row (ADR-028).
> Budget: voxxa-eval row ≤ 25 lines; no other Rust (the port is V11's). Safety: standard for agents (no microphone, no keys, no lab source). Only the founder runs the helper that presses keys, under task 5's rules.
> State files: `decisions/{sde,qa,security}/church-autoadvance.md`, `status.md`, `handoff.md`, `reviews/YYYY-MM-DD-v3d-parts.md` (numbers only), the port plan. Exit: CI green; every gate case passes in the lab; the flow-set and detour-take tables are filled; the live test is done with 0 wrong slides, or the failing case and take are named; ADR-031 can be accepted or amended.

### V4 — GPU speed proof (Vulkan, booth-only dev build)

**Round 3 (draft, 2026-11-26):** sized S in round 3 (GPU proof); unchanged otherwise. Prompt: `plan-round3.md` § 2 where it has a block for this step.

- **Size** M (S if V3 reaches ≤ 1.0 s/pass: then only steps 1, 3 and the VRAM/stutter numbers) · **Phase** 2 check + 4 · **Owner** Sr Eng 1 + Sr Eng 3 · **Depends on** FA-04 (Vulkan SDK installed), V0, V3 · **Unblocks** FA-23 (hardware spec), V6 live configuration
- **Acceptance:** opt-in feature; Small 6 s ≤ 0.8 s/pass (p95 ≤ 1.0 s) with a reused state; VRAM ≤ 1.2 GB; no visible Presenter stutter; no-loader start test recorded; no installer change.

> /10x-Team:10x-team
> Phase 2 check, then Phase 4, roles: Senior Engineer + DevOps Engineer + SRE. Precondition: the founder installed the Vulkan SDK on the booth PC and pasted `vulkaninfo --summary`'s device line; V0 and V3 merged; ADR-021 Accepted. Two lanes (`v4/10x`, `v4/ponytail`); record `reviews/YYYY-MM-DD-ponytail-v4.md`.
> 1. `src-tauri/Cargo.toml` `[features]`: `gpu-vulkan = ["whisper-rs/vulkan"]` on the locked whisper-rs 0.13.2. Never in `default`. No whisper-rs bump.
> 2. Create the context with `use_gpu` set explicitly when the feature is on; log backend and device name on the session start line; if GPU init fails, fall back to CPU and say so in diagnostics.
> 3. Founder checklist for the booth (agents never build there): build `voxxad --features cli,gpu-vulkan` with the V0 flags script in a separate target folder, on a day with no service and no timing run; then Small and (if it loads) a quantised Medium, 6 s encoder, reused state: s/pass median and p95, VRAM peak from Task Manager, and whether Presenter visibly stutters during a 10-minute run on the projector output. Numbers only.
> 4. No-loader test: the founder (or a test PC, never the booth's installed voxxa) starts the GPU `voxxad --help` in Windows Sandbox. Record "starts / does not start". If it does not, the GPU build stays a separate dev build (ADR-021 § 3).
> 5. Record the result and the hardware-class table (G: Vulkan GPU ≥ 2 GB; C6: 6–8 core AVX2 CPU; C4: older 4-core) in ADR-021 and `docs/`; mark untested classes as such.
> Budget: recognizer row; ≤ 40 lines. CI: no GPU job in this step. Safety: standard.
> State files: `decisions/{sde,devops,sre}/church-autoadvance.md`, `status.md`, `handoff.md`, `reviews/YYYY-MM-DD-v4-gpu.md`. Exit: CI proves the default build is unchanged (the feature needs the SDK, which runners do not have, so no GPU job runs in CI); the booth numbers meet or miss the bar, and the next step (ship Small on GPU as a dev build, or stay on CPU) is recorded in ADR-021.

### V5 — Kit skeleton, licence register, songcheck, privacy guards

**Round 3 (draft, 2026-11-26):** **split:** V5a (S, now) is the `.txt` importer's sections (three named tests; FreeShow groups get labels, which V8b and V17 need); V5b (M, kit tools and guards) is **deferred** behind the first autonomous Sunday (R17). Prompt: `plan-round3.md` § 2 where it has a block for this step.

- **Size** S · **Phase** 4 · **Owner** Sr Eng 3 + engineer team · **Depends on** ADR-026, ADR-027, ADR-029 Accepted; FA-12 (scrub decision), FA-13 (licence policy) · **Unblocks** F1 intake, church #2
- **Acceptance:** `kit/` from `out/kit/`; `songcheck.py` and `kit_check.py` (stdlib, PEP 723, self-tests); `presenter_slides` required on keys-only apps; the `published_year` rule; media guard and `.gitignore` extended; CODEOWNERS; `fixtures_carry_an_allowed_licence`; the `.txt` importer fills `sections` and marks credit slides (`txt_section_labels_fill_sections`, `credit_slide_kind_detected`, `old_setlists_replay_unchanged`); the scrub PR (if FA-12 says scrub) merged before any deny-file gate; kit tools row 300; a fake second church passes.

> /10x-Team:10x-team
> Phase 4, roles: Staff Engineer + SDE + Senior Engineer (importer) + Security Engineer + Product Manager (plain-language review of the kit). Precondition: ADR-026, ADR-027 and ADR-029 Accepted; the founder answered FA-12 and FA-13. Read the kit draft (copied into the session as `kit-draft/`). Two lanes (`v5/10x`, `v5/ponytail`); record `reviews/YYYY-MM-DD-ponytail-v5.md`.
> 1. Commit `kit/` from the draft: `README.md`, `private-folder.md`, `templates/`, `prompts/01–08`, `skills/*/SKILL.md`. Lowercase "voxxa" in every new file. Only made-up words and public-domain titles. Use `writing-skills:plain-language` on the README and checklists.
> 2. `kit/tools/songcheck.py`: PEP 723, stdlib, `--self-test`. Checks a church folder: every `songs\*.txt` has exactly one `songs.csv` row and every row a file; the licence code is in `licence-kinds.csv`; required fields per ADR-026 § 4, including `published_year` when `pd_basis` is `us_published_95_years` (public domain in the US when `published_year <= current_year - 96`, computed from today's date on every run, so the rule never goes stale; the other bases are `author_died_70_years`, `traditional`, `dedicated_cc0`); `presenter_slides` is **required** on keys-only apps (ADR-026 § 4; WorshipTools Presenter is keys-only) and must equal the `.txt` slide count (blank line or `---` splits, the importer's rules), and on other apps it is checked when set; `church-profile.json` rules (streams → ND red, SA/NC amber; `ccli.active` false → CCLI red, UNKNOWN amber). Prints file and field names only, never song text; exit 1 on a red problem.
> 3. `kit/tools/kit_check.py`: PEP 723, stdlib, `--self-test`. Refuses lyric keys (`text`, `lyrics`, `slides`, `words`) and a `run` key in kit JSON; media files; `C:\Users\` paths; strings from `--deny-file <private file>`; and with `--lyrics-src <dir>` fails on any 4-word sequence of the `.txt` files there found in the candidate files (hashed, printing file:line only).
> 4. Guards: extend `.gitignore`, `src-tauri/tests/media_guard.rs` and the `media` CI job to refuse `church-profile.json`, `layout.json`, `*.service.json`, `songs.csv`, `practice/`, `sessions/`, `session-*.jsonl`, `truth.jsonl` and `*.sqlite3*` outside `kit/templates/` (files there are named `*.template.*`) and the existing public fixtures. Add `src-tauri/tests/licensing.rs`'s sibling `fixtures_carry_an_allowed_licence`: every committed lyric fixture has a `LICENCE.json` with code `PD` or `CC0`.
> 5. Add `.github/CODEOWNERS`: the founder owns `kit/prompts/**`, `kit/skills/**`, `kit/templates/bindings/**`. The `budgets` job runs `python3 kit/tools/kit_check.py --kit kit/` (no uv in CI).
> 6. If FA-12 says "scrub": one scrub PR replaces the strings below with aliases. It merges **before** the ADR-029 deny-file scan becomes a gate over `.10x/` and `fixtures/` (ADR-029 § 3, V15); until then that scan would fail on any PR touching those folders. Lines are `path:line` on `866b51e`, checked for this plan; re-check them on the PR's base. Never quote a replaced string in the PR, its commits or the review file. Code and test lines keep their meaning (made-up values).
>    - **Church identifiers:** ADR-019's private-song row (`.10x/adrs/019-worshiptools-presenter-keys-guarded-licensed-test-audio.md:112-114`: licence number, band as artist); `fixtures/LICENSED_RUNS.md`'s row 1 (`:11-13`) and its template columns → `alias`, `licence_class`, numbers (ADR-026 § 10; credit lines, titles and authors go only to the private `<church_root>\reports\LICENSED_RUNS.md`); the prompt pack's church name (`.10x/specs/2026-11-11-prompts-10x-team.md:117`).
>    - **The private song's title or slug → "private song P1" (slug `p1`), 36 lines:** `.10x/handoff.md:134`, `:136-139`, `:141-144`; `.10x/status.md:4`, `:342`; `.10x/reviews/2026-11-23-ponytail-q10.md:4`; `.10x/adrs/019-worshiptools-presenter-keys-guarded-licensed-test-audio.md:110`, `:112-113`; `fixtures/README.md:294`, `:296`, `:298`, `:306`, `:310`, `:313`, `:315`, `:320`, `:328`, `:332`, `:344`, `:346-347`, `:352`; `fixtures/LICENSED_RUNS.md:5`, `:9`, `:13`; `fixtures/tools/tap-truth.ps1:20` (the `.EXAMPLE` path); `src-tauri/tests/media_guard.rs:180-182` (example paths under `fixtures/private/`; the guard refuses that whole folder, so the slug `p1` keeps the test's meaning).
>    - **The booth PC's host name → "booth PC", 64 lines in 21 files** (the plan listed only `fixtures/README.md:288`; the rest were found while checking this list): `.10x/adrs/019-worshiptools-presenter-keys-guarded-licensed-test-audio.md:6`, `:10`, `:45`, `:54`, `:117`; `.10x/decisions/architect/_index.md:115`; `.10x/decisions/devops/church-autoadvance.md:147`, `:176`; `.10x/decisions/product-manager/church-autoadvance.md:35`; `.10x/decisions/qa/church-autoadvance.md:389`, `:495`, `:530`; `.10x/handoff.md:24`, `:29`, `:73`, `:79`, `:83`, `:133-134`; `.10x/reviews/2026-11-20-ponytail-q9v.md:143`; `.10x/reviews/2026-11-20-ponytail-q9w.md:9`; `.10x/reviews/2026-11-20-ponytail-r0.md:53`, `:70`, `:72`, `:89`, `:164`, `:188`, `:192`, `:206`, `:209`, `:212`, `:264`, `:271`, `:292`; `.10x/reviews/2026-11-20-q9v-tickets.md:46`; `.10x/reviews/2026-11-20-q9w-tickets.md:23`; `.10x/reviews/2026-11-20-worshiptools-founder-test.md:3`; `.10x/reviews/2026-11-23-ponytail-q10.md:166`; `.10x/reviews/2026-11-23-q10-tickets.md:74`; `.10x/specs/2026-11-11-prompts-10x-team.md:82`, `:92`, `:94`, `:103`, `:117`; `.10x/specs/2026-11-20-prompts-worshiptools-revision.md:7`, `:28`, `:110`, `:142`, `:156`, `:159`, `:177`, `:187`, `:198`; `.10x/status.md:313`, `:340`, `:342`, `:405`; `docs/RELEASING.md:56`; `fixtures/LICENSED_RUNS.md:13`; `fixtures/README.md:288`, `:294`, `:296`, `:352`; `src-tauri/tests/worshiptools.rs:245`.
>    - **Before the PR merges,** the founder runs `kit_check.py --deny-file <private list>` on the booth PC over `.10x/`, `docs/`, `fixtures/` and `src-tauri/tests/` on the PR branch and pastes counts and `file:line` only; anything it finds (the church name elsewhere, licence numbers, other song titles) joins the same PR.
>    No history rewrite unless FA-12 says (b). With FA-12 (c), the deny-file scan runs on `kit/` only and never gates `.10x/` or `fixtures/`.
> 7. Add the "kit tools (kit/tools/)" Python row, cap 300, to `scripts/loc_budget.py`.
> 8. Validate: at `C:\dev\voxxa-church-private\fake-church-2\` (never inside `pilot\`), build a fake second church from the kit alone (copy the templates, including `fixtures-manifest.template.json`, `intake.template.json`, `truth-v2.example.jsonl` and `recording-consent.template.md`, and fill made-up values); `songcheck` and `kit_check` pass; deleting a CC row's `credit_line` fails with a file-and-field line and no song text; so does emptying `presenter_slides` in a row while the profile names a keys-only app.
> 9. **Section labels and credit slides** (the one Rust change in V5; spec § 4.5). In `src-tauri/src/importers/txt.rs`, keep each section label line (`looks_like_section_label`) as a `sections` entry instead of writing `sections: Vec::new()` (`txt.rs:69`): the label plus the slides it covers, starting at its first slide index (the existing `Section { kind, slides }`, `src-tauri/src/aligner/evidence/sections.rs:25-28`). The label stays off the slide text, as today; a label with no slide after it is dropped; the result must pass the existing section validator (`sections.rs:30-31`). Map labels to `SectionKind` as ADR-016's parser does [UNVERIFIED: whether `SectionKind` has a value for every label the importer accepts; an unmapped label is dropped, never guessed]. A slide whose first line starts with `Credit:` gets `kind: "credit"`, a new optional field on `Slide` (`src-tauri/src/aligner/mod.rs:18-21`) with `#[serde(default)]`, not written when absent (like `sections`, `:27-31`), so old setlists and session logs replay byte for byte. Tests (made-up words only): `txt_section_labels_fill_sections`, `credit_slide_kind_detected`, `old_setlists_replay_unchanged` (every committed setlist and session log replays byte-identical).
> Budget: Python rows as above; Rust only for step 9: ≤ 40 lines in the importers row (1,386/1,600) and ≤ 3 lines in the conductor row for the `kind` field (1,173/1,200; V11's ≤ 20 lines of wiring still fit). Safety: standard; the lyric scan and the deny-file scan against a real church run only on the booth PC, by the founder.
> State files: `decisions/{sde,security}/church-autoadvance.md`, `status.md`, `handoff.md`, `reviews/YYYY-MM-DD-v5-tickets.md`. Exit: CI green including the new guards; a planted `layout.json` outside `kit/templates/` turns the media job red in a scratch run; the fake church passes.

### V5c — voxxa-tools zip: the CLI tools for churches

**Round 3 (draft, 2026-11-26):** **deferred** behind the first autonomous Sunday (R17, FA-72); needed for church #2. Prompt: `plan-round3.md` § 2 where it has a block for this step.

- **Size** S · **Phase** 4 + 6 · **Owner** Sr Eng 3 (platform) + DevOps · **Depends on** V0 merged (the tools must carry the release flags); FA-31 option (a); the Windows signing path (FA-08) · **Unblocks** church #2's practice capture, `.wav` intake and `--check-layout`; ADR-029's kit acceptance
- **Acceptance:** the Windows release job attaches `voxxa-tools-<version>-windows-x64.zip` (signed `voxxad.exe` and `voxxa-eval.exe`, built with `--features cli` and the V0 flags) and its SHA-256 file to the draft release; the GUI installers, updater signatures and `latest.json` are unchanged; `release_config.rs` pins all of it; `voxxad --version` exists; the fake church runs `voxxad --version` and `voxxa-eval --help` from the unzipped tools with no repo checkout.

> /10x-Team:10x-team
> Phase 4 + 6, roles: DevOps Engineer + Staff Engineer + Security Engineer. Precondition: V0 merged; the founder chose FA-31 (a). Read the `windows` job of `.github/workflows/build.yml` (`:328-558`), `src-tauri/Cargo.toml:21-35` and `:121-122` (the Q7 decision: the installers carry voxxa only, and stay that way), ADR-020 § 6 and `packaging/signpath/artifact-configuration.xml`. Two lanes (`v5c/10x`, `v5c/ponytail`); record `reviews/YYYY-MM-DD-ponytail-v5c.md`.
> 1. In the `windows` job, after "Build the installers" (`build.yml:444`), build the tools with the job's env (so the V0 flags apply): `cargo build --release --features cli --bin voxxad --bin voxxa-eval --manifest-path src-tauri/Cargo.toml`. The installer build stays without `--features cli`, so the installers do not change.
> 2. Sign `voxxad.exe` and `voxxa-eval.exe` on the same path as the installers (SignPath when configured, else Azure Trusted Signing: one publisher identity, ADR-020 § 6), and extend "Check the Authenticode signatures" (`build.yml:504`) to both. If SignPath cannot sign them in the same request, stop and report [UNVERIFIED].
> 3. Zip the two signed exes with `LICENSE` and a short `README.txt` (what each tool does; "never run a test build on a church booth PC"; the kit link) as `voxxa-tools-<version>-windows-x64.zip`, write `voxxa-tools-<version>-windows-x64.zip.sha256`, and upload both with the installers (`build.yml:531-535`). The zip gets no `.sig`, so `scripts/release/updater_manifest.py` never lists it and `latest.json` is unchanged; check that `winget_manifest.py` still lists only the installers [UNVERIFIED].
> 4. `voxxad --version` prints the crate version (≤ 3 lines in the voxxad row, 552/600: flag parsing only; ADR-028 § 3). `voxxa-eval --help` already exists (`src-tauri/src/cli/voxxa-eval/main.rs:49`).
> 5. `src-tauri/tests/release_config.rs`: the tools build step has `--features cli` and runs in the `windows` job's env; the installer build step has no `--features cli`; the zip's name; no `.sig` and no updater entry for the zip. Update test and workflow in the same commit (`.10x/handoff.md:99`).
> 6. Kit: the kit README's tools table and the church-setup checklist get a step: download the zip and its `.sha256` from the release page, check it with `Get-FileHash -Algorithm SHA256`, unzip it outside `<church_root>` (e.g. `%LOCALAPPDATA%\voxxa-tools\`), add that folder to PATH, and replace it whenever the app is updated (same version). Until a release carries the zip, the kit README says plainly that practice capture, `.wav` intake and `--check-layout` run only on the founder's development booth from a local build.
> 7. ADR-029's acceptance test gains: the fake church at `C:\dev\voxxa-church-private\fake-church-2\` runs `voxxad --version` and `voxxa-eval --help` from the unzipped tools, with no repo checkout on the PATH.
> Budget: voxxad row ≤ 3 lines (its share of the 48 left; ADR-028 § 3); no other Rust. Safety: standard; no tag, no release and no upload outside the founder's pre-approved batch.
> State files: `decisions/{devops,security}/church-autoadvance.md`, `status.md`, `handoff.md`. Exit: CI green with the new `release_config.rs` pins; the next rc draft the founder approves (FA-08) carries the zip and its SHA-256 file, and the founder runs the two commands from it on a test PC (never the booth PC's installed voxxa).

### F1 — Founder: the corpus (about 3 Sundays)

- **Size** founder · **Owner** founder + worship team · **Depends on** FA-06 (consent, `kit/templates/recording-consent.template.md`), V1 (lab under uv) · **Unblocks** V6
- **Acceptance:** ≥ 10 real sung songs, ≥ 120 slide changes, ≥ 2 worship leaders, ≥ 3 songs with an instrumental gap ≥ 6 s, desk feed; prefer PD/CC0 songs (December carols can become public fixtures with performer consent); talk spans muted at intake; truth tapped with `fixtures/tools/tap-truth.ps1` after a one-time tap-lag calibration on a TTS fixture; **held-out set of ≥ 4 songs chosen by hand once, stratified, before any tuning**, frozen by `sha256(truth.jsonl)` in `<church_root>\fixtures\manifest.json` (template `kit/templates/fixtures-manifest.template.json`; each fixture gets an `intake.json` from `kit/templates/intake.template.json`); each recording has a retention date. On the founder's booth `<church_root>` is `C:\dev\voxxa-church-private\pilot\`. Runbook: `kit/prompts/08-wav-intake.md` and `kit/templates/wav-intake-checklist.md`.

### V6 — Multi-song follower evaluation at one frozen setting (the gate)

- **Size** M · **Phase** 2 (evidence for the ADR) · **Owner** Sr Eng 1 · **Depends on** V2, F1 (≥ 8 performances to start, ≥ 10 songs to gate), V3/V4 (live configuration) · **Unblocks** ADR-022 acceptance, V11
- **Acceptance (held-out, live configuration, live-fidelity hop dropping):** recall ≥ 0.95; wrong moves ≤ 1 per 10 songs; 0 hidden mid-song with the cold gap hold modelled; 0 shown during talk; a neighbour configuration ≤ +1 wrong move per 10 songs; reported, not gated: `on_time_1500`, song-start median/p95, DR availability under the ADR-022 gate; the prototype's code size measured.

> /10x-Team:10x-team
> Phase 2 (evidence), roles: Senior Engineer + QA Engineer + Staff Engineer. Runs as 10x + `ponytail-audit`. Precondition: V2 merged; the founder's corpus manifest exists with a frozen held-out split; the live configuration is decided (V3/V4). Agents never open the private fixtures, the prototype source or any cached window: **the founder runs every command on the booth PC from a checklist you write, and pastes numbers only.**
> 1. Write the founder checklist: in the shared lab `C:\dev\voxxa-church-private\lab\` (uv project), run the prototype at one setting chosen on the tune set (the flat-region rule: all ±1-step neighbours within 1 point of recall and with no more wrong moves) on (a) the live configuration and (b) one neighbour configuration (encoder context ± one step), with the hops the live keep-up gate would skip at the measured s/pass dropped (`listen.rs:159-166`), and with the ADR-022 gates modelled: the one hide rule (cold 10 s), DR only with ≥ 2 on-score tokens in the last 6 s and anchor ≤ 8 s, song start on 2 agreeing windows. Output: an action list per fixture, scored with `voxxa-eval --actions` (V2).
> 2. Tune-set runs first; then exactly one held-out run, logged in `manifest.json`'s look count. The tuner sees only the held-out aggregate.
> 3. Ask the founder for the prototype's line count (`tokei` numbers only) to size ADR-028's follower row.
> 4. Write `reviews/YYYY-MM-DD-v6-follower-eval.md`: the table (recall, wrong per 10 songs, hidden mid-song, shown during talk, on-time share, song-start median/p95, DR fires available vs used), tune vs held-out, both configurations, the frozen setting's parameter values, and the verdict against the bar. No titles, no words.
> 5. If the bar fails: say which gate or which songs (by alias) fail, and propose the next experiment (pitch gate, heard tokens, recognizer config) for the founder; do not re-tune on the held-out set.
> Budget: none. Safety: standard.
> State files: `status.md`, `handoff.md`, the review file, an addendum to ADR-022's record. Exit: the founder has the numbers and a pass/fail verdict; ADR-022 can be accepted or amended.

### V6b — Timeliness experiment: prediction from practice (optional)

**Round 3 (draft, 2026-11-26):** **dropped** (R17, FA-72): prediction from practice is not pursued; the onset trigger and the parts map carry timing. Prompt: `plan-round3.md` § 2 where it has a block for this step.

- **Size** M · **Phase** 2 · **Owner** Sr Eng 1 · **Depends on** V6 · **Unblocks** an ADR-022 amendment enabling prediction
- **Acceptance (tune set only):** on-time share +20 points vs V6 with no added wrong moves; early hides ≤ 0.4% per prediction, firing at the 99.6th percentile of residuals.

> /10x-Team:10x-team
> Phase 2, roles: Senior Engineer + QA Engineer. 10x + `ponytail-audit`. Precondition: V6 done. Same founder-runs-it rule as V6. Design the founder checklist for an offline prediction run on the **tune set only**: per occurrence, predicted fire time = anchor-backed first-word time + the slide's practice duration scaled by the live tempo ratio, fired at the 99.6th percentile of residuals from another performance of the same song (never the same one); prediction only after 2 confirmed moves in a song; cancelled on gate close, Pause or operator action. Report on-time share, early hides per prediction, added wrong moves, song by alias. Verdict against the bar; if it passes, draft an ADR-022 amendment for the founder. Exit: review file `reviews/YYYY-MM-DD-v6b-prediction.md`, numbers only.

### V7 — Budget row split ("wiring")

**Round 3 (draft, 2026-11-26):** this week. ADR-028 amendment: new rows onset 750 (split from the follower row), input chain 600, health check (`src/health/`) 450 (the estimates 250 / 200 / 150 tripled under the founder's FA-54 rule, answered 2026-11-27; FA-74 asks whether the rule extends to the follower, practice, layout, Run and kit-tools rows); `vad/` moves to the recognizer row; `align` 400 only per R5; plus the one new `trace` JSONL line type with a `kind` field (ADR-001/003 amendment, FA-65). Depends on FA-10, FA-54, FA-65 (all answered by 2026-11-27). Exit: budgets green; 150 free in wiring; rows recorded with estimate and cap.

**Founder direction (2026-11-27): re-anchor for every presentation software on the market is the spearhead.** V7 also adds the `position` row (estimate 300, cap 900) once FA-77 is answered, and a new step **V8c** (ADR-036: one position interface; read-back adapters as data; output-frame hashes for every app) runs right after V7, before the FreeShow driver. Prompt: `plan-round3.md` § 2 V8c. Prompt: `plan-round3.md` § 2 where it has a block for this step.

- **Size** S · **Phase** 2 + 4 · **Owner** Engineering Manager + Sr Eng 3 · **Depends on** ADR-028 Accepted (FA-10 approved 2026-11-26, option (a)) · **Unblocks** V8, V9, V11, V13, V14
- **Acceptance:** new "wiring" row = measured size of `dispatch.rs`, `listen.rs`, `http_api/`, `commands/` + 150; "everything else" lowered by the measured size; budgets job green; numbers recorded in ADR-028.

> /10x-Team:10x-team
> Phase 2 check, then Phase 4, roles: Engineering Manager + Staff Engineer. Precondition: ADR-028 Accepted with the founder's explicit approval of the one 150-line increase. 10x lane + ponytail lane; record `reviews/YYYY-MM-DD-ponytail-v7.md`.
> 1. Measure with `tokei` the code lines of `src-tauri/src/dispatch.rs`, `src-tauri/src/listen.rs`, `src-tauri/src/http_api/` and `src-tauri/src/commands/` on `main`.
> 2. In `scripts/loc_budget.py`, add the row "wiring (dispatch.rs, listen.rs, http_api/, commands/)" with cap = measured + 150, subtract it from `rest` (`loc_budget.py:81`), and lower the "everything else" cap by exactly the measured amount, with a comment citing ADR-028. No file moves.
> 3. Record the measured numbers and the new caps in ADR-028 and `.10x/handoff.md`'s row table.
> Budget: this is the budget change. Safety: standard.
> State files: `decisions/{engineering-manager,staff-engineer}/church-autoadvance.md`, `status.md`, `handoff.md`. Exit: the budgets job is green; "everything else" still shows 1 line left; the wiring row shows 150 left.

### V8 — Dispatch and keystroke safety

**Round 3 (draft, 2026-11-26):** **reduced to V8-lite** (S): ADR-025 per-key position only (`position_uncertain` exists); steps 2–4 deferred (R17). Depends on V7. Prompt: `plan-round3.md` § 2 where it has a block for this step.

- **Size** M · **Phase** 4 · **Owner** Sr Eng 2 (integrations/layout) · **Depends on** V7; ADR-025 Accepted · **Unblocks** V11, V13
- **Acceptance:** tests `partial_key_sequence_updates_sent_per_key_and_flags_uncertain`, `every_keystroke_profile_requires_a_target`, `voxxad_keystroke_requires_target_exe`, `elevated_target_is_refused`; amber/red sentences in `statusLight` with GUI tests; replay unchanged.

> /10x-Team:10x-team
> Phase 4, roles: SDE + Senior Engineer + Security Engineer. Precondition: V7 merged; ADR-025 Accepted. Read `src-tauri/src/dispatch.rs:99-180`, `src-tauri/src/presenters/{mod,keystroke}.rs`, `src/simple/logic.js`. Two lanes (`v8/10x`, `v8/ponytail`); record `reviews/YYYY-MM-DD-ponytail-v8.md`.
> 1. **Per-key position:** in the stepping loop (`dispatch.rs:163-172`), update `sent.global` by ±1 after each key that succeeds; if a key fails, set a `position_uncertain` flag, stop, and return the error. While the flag is set, conductor moves are not dispatched; an operator confirm (an existing manual step or a new "position confirmed" command) clears it. Test with a fake driver that fails on the 3rd of 4 keys: `sent.global = start + 2`, flag set, the next automatic Goto sends nothing.
> 2. **Every keystroke profile needs a target:** generalise `keys_allowed` (`keystroke.rs:59-63`) from "WorshipTools only" to "the profile's target process": WorshipTools keeps `Presenter.exe`; other named profiles take their executable from a table (mark each name [UNVERIFIED] until a test card confirms it); the universal profile records the front process when Songs starts (operator clicks the presentation window; amber until then). Keys go only to the target; otherwise nothing is sent and the amber sentence "Bring <app> to the front." shows.
> 3. `voxxad --presenter keystroke` refuses to start without `--target-exe <name>` (update `docs/INTEGRATIONS.md:260-266`).
> 4. **Elevated target:** on connect and every 60 s, check whether the target process is elevated while voxxa is not (Win32 FFI in the existing style, no new crate); if so, red: "Presenter runs as administrator; voxxa's keys can't reach it." If the check needs more than about 40 lines, stop and report.
> 5. GUI tests for the two new `statusLight` sentences, word for word.
> Budget: wiring row (V7) and presenters row (`keystroke.rs` is 396/400: put new logic in `presenters/mod.rs` or a new file in the presenters row); voxxad row ≤ 5 lines for `--target-exe` parsing and the refusal (ADR-028 § 3), the target check itself in the presenters row. Safety: standard; test only with fake drivers and pure helpers.
> State files: `decisions/{sde,security,qa}/church-autoadvance.md`, `status.md`, `handoff.md`, tickets file. Exit: the four tests pass on all OSes; replay byte-identical; CI green.

### V9 — Session-log and help-bundle privacy

- **Size** S · **Phase** 4 · **Owner** Sr Eng 3 + UI consultant · **Depends on** V7; ADR-023 Accepted; FA-19 · **Unblocks** V12, Q14
- **Acceptance:** session logs default to `%LOCALAPPDATA%\com.voxxa.app\sessions\` (Tauri's app-local-data folder; never under the install folder `%LOCALAPPDATA%\Voxxa\` or `<church_root>`) with a best-effort one-time move; the 14-day rotation unchanged; help bundle words box unticked by default; tests `session_logs_in_localappdata`, `help_bundle_words_default_off`.

> /10x-Team:10x-team
> Phase 4, roles: SDE + Security Engineer + UI consultant (copy). Precondition: V7 merged; ADR-023 Accepted; the founder answered FA-19. Two lanes; record `reviews/YYYY-MM-DD-ponytail-v9.md`.
> 1. `voxxad`'s default log directory (`src-tauri/src/cli/voxxad/main.rs:313-317`) and the help bundle's session source (`src-tauri/src/help/mod.rs:181`) move from `dirs::data_dir()` + `voxxa` (roaming `%APPDATA%\voxxa\`) to `dirs::data_local_dir()` + `com.voxxa.app`, i.e. `%LOCALAPPDATA%\com.voxxa.app\sessions\`, the folder Tauri uses as app-local data for the identifier. Not `%LOCALAPPDATA%\voxxa\`: NTFS names are case-insensitive, so that is the install folder `%LOCALAPPDATA%\Voxxa\`; the test asserts the path is outside it (case-insensitive). On first start, move existing `session-*.jsonl` files best-effort (a locked file stays and is logged by count). `--log-dir` still overrides. Check and record what the NSIS uninstaller's "delete app data" option removes [UNVERIFIED].
> 2. `src/simple/help.js:38`: the "Include the last 2 minutes of words" checkbox starts unticked; the review screen still shows what would be included.
> 3. Update `fixtures/README.md:277-282`, `docs/INTEGRATIONS.md` and ADR-018's record.
> Budget: wiring row / everything else as measured in V7; voxxad row: no net lines (its three-line default becomes one call to a shared session-folder helper in the wiring row, which also does the one-time move; ADR-028 § 3 lists no voxxad share for V9); Simple mode row ≤ 5 lines. Safety: standard; tests use a scratch data dir (`VOXXA_HELP_DIR`).
> State files: `decisions/{sde,security}/church-autoadvance.md`, `status.md`, `handoff.md`. Exit: the two tests pass; replay unchanged; the help bundle test still redacts secrets.

### V10 — Run screen clickable mock (UI round R1)

**Round 3 (draft, 2026-11-26):** re-scoped (R16): the mock shows the minder states (following / holding / you're driving) and the health-check page, so every amber and red sentence is approved word for word with the mock in front of the founder (FA-62). Prompt: `plan-round3.md` § 2 where it has a block for this step.

- **Size** S · **Phase** 3 (design) · **Owner** UI consultant · **Depends on** none · **Unblocks** V14
- **Acceptance:** a mock in the existing GUI test harness showing Run, Pause, Not sure, Lost, the keys-only Resume card (no default button), the suggestion card and the off-by-one keys; contrast numbers for the two new tokens; founder walkthrough notes.

> /10x-Team:10x-team
> Phase 3 (design), roles: Product Manager + UI consultant (use `ui-consistency:verifying`). 10x + ponytail lane. Build a clickable mock of spec § 4.7 inside `tests/gui/` using the mocked backend (`tests/gui/mock-tauri.js`) with scripted events (moves with reasons, Not sure, Lost, Pause, a keys-only Resume, a suggestion card between items); no Rust, no change to shipped GUI files. Measure contrast for `--accent-strong` and `--field-border` (WCAG 2.2 AA). Write a 30-minute walkthrough script for the founder (no volunteers yet) and record his notes in `reviews/YYYY-MM-DD-v10-run-mock.md`. Safety: standard. Exit: the mock runs in the harness; the founder's decisions FA-17 (override hardware) and FA-21 (first line on Run) are asked with the mock in front of him.

### V11 — Follower port

**Round 3 (draft, 2026-11-26):** one module `aligner/follower/parts.rs` with a `dr_show` flag for one-part songs, replacing the `aligner/evidence/` accumulator under one config switch (R12); the Tier A/B jump interface (FA-47, ADR-032); `PartsConfig.break_gate` default `None` with the before/after table; the `trace` line; replay byte-identical. Depends on V7, V8b, V3d, V6. Exit adds: identical goto lists against the prototype on T1, T2, F1–F4 and live4. Prompt: `plan-round3.md` § 2 where it has a block for this step.

- **Size** L · **Phase** 4 · **Owner** Sr Eng 1 · **Depends on** V6 pass; V7; V8; ADR-022 Accepted (with V6 numbers); FA-05; for the onset module, V3b's port plan and ADR-030 Accepted; for the parts map, V3d's port plan and ADR-031 Accepted · **Unblocks** V12, V13, V14
- **Acceptance:** `Option<FollowerConfig>` default `None`; replay byte-identical with `None`; private equivalence (identical goto lists on cached windows, founder-run); public fixtures no worse with `Some`; held-out V6 bar holds in Rust on the live configuration; the ADR-022 gate tests; synthetic stress set: 0 DR fires in ≥ 3 h talk over pad and ≥ 1 h hum over pad; numbers-only `follow` line; row per ADR-028. If ADR-030 is accepted: the onset module from V3b's port plan (`FollowerConfig.onset`, default `None`), ADR-030's named tests, and early hides ≤ 0.4% per onset fire on the tune corpus and the stress set (founder-run on the corpus; numbers only). If ADR-031 is accepted: the parts map from V3d's port plan (`FollowerConfig.parts`, default `None`), ADR-031's named tests, the flow set no worse than V3d's lab numbers, and songs without part labels on the plan-order follower.

> /10x-Team:10x-team
> Phase 4, roles: Senior Engineer + SDE + QA Engineer + Security Engineer. Precondition: V6 passed; ADR-022 Accepted; V7 and V8 merged. Read ADR-009, ADR-013, ADR-016, ADR-022 and `src-tauri/src/aligner/`. Two lanes (`v11/10x`, `v11/ponytail`), comparator, cross-reviews, verifiers; record `reviews/YYYY-MM-DD-ponytail-v11.md`.
> 1. Port the follower as specified in ADR-022 § 2 from the founder's written description and the V6 parameter values (agents never open the private prototype): occurrences from the setlist (repeats as copies, section-start jump edges), banded local alignment of each window's words, margin rule, pace learning, one dead-reckoning step per anchor. Module `src-tauri/src/aligner/follower/` (or inside `evidence/` if ≤ 250 lines), behind `SmartConfig.follower: Option<FollowerConfig>` with `#[serde(default)]` = `None`. Conductor wiring ≤ 20 lines.
> 2. Implement every ADR-022 § 4 gate with a named test: margin, lost (keep slide, amber), one hide rule (cold 10 s), song start (2 windows; suggestion during a spoken item when the layout exists), DR limits (≤ 8 s anchor, ≥ 2 on-score tokens in 6 s, never across songs or on a first slide), fire-time re-check and cancellation (`scheduled_move_cancelled_on_gate_close_pause_off_operator`), watchdog (`no_move_when_newest_window_older_than_8s`), operator re-anchor, credit slides, no early WorshipTools song-end Right (`worshiptools_no_song_end_right_on_last_song`). Add `spoken_next_song_quote_does_not_start_song` (a TTS reading of public-domain hymn lines over silence) and `cues_disarmed_through_mid_song_instrumental` if the layout exists, else defer it to V13.
> 3. Build the public synthetic stress set (deterministic; public-domain speech and CC0 pads mixed with ffmpeg; a 10 s instrumental gap; an unplanned chorus repeat; ±10% tempo copies of public-domain songs); commit hypothesis logs only (media guard). Gate: 0 DR fires and 0 shown-during-talk over ≥ 3 h talk-over-pad and ≥ 1 h hum-over-pad.
> 4. A numbers-only `follow` session-log line; an event for the GUI with `why` and a certainty value (consumed by V14).
> 5. Founder checklist: the private equivalence run (Rust port vs prototype on cached windows: identical goto lists) and the held-out re-run on the live configuration. The equivalence test lives only in the private lab.
> Budget: per ADR-028 (follower row cap = 1.3 × measured port, ≤ 500). Safety: standard.
> State files: `decisions/{sde,qa,security}/church-autoadvance.md`, `status.md`, `handoff.md`, tickets file, `reviews/YYYY-MM-DD-v11-before-after.md`. Exit: replay byte-identical with `None`; public fixtures no metric worse with `Some`; the gate tests and the stress gate pass; the founder's private equivalence and held-out numbers meet the V6 bar. The default stays `None`; flipping it is a separate PR after 2 shadow Sundays (FA-05).

### V12 — Practice memory ("listener DB") v1

**Round 3 (draft, 2026-11-26):** practice memory v1: data model (parts order, detours, boundary lag, leader alias L1/L2), `status` {not_rehearsed, proposed, promoted, demoted}, `--learn` = the parts map run non-causally, a `--review` keyboard flow, `--confirm`, ≥ 3 confirmed runs before learned handicaps (R4, FA-64), never `boundary = plan`; no neural aligner. Exit adds: `replay_with_practice_is_byte_identical`; a demotion survives a restart. Prompt: `plan-round3.md` § 2 where it has a block for this step.

- **Size** M · **Phase** 4 · **Owner** Sr Eng 1 (learning) + Sr Eng 3 (file IO) · **Depends on** V11; V9; V5 step 9 (section labels); ADR-023 Accepted · **Unblocks** Q14 GUI capture
- **Acceptance:** `voxxad --learn <session.jsonl> --setlist <file> --out <church_root>\practice` and `voxxad --confirm <song_fp> --practice-dir <church_root>\practice`; schema tests; atomic writes; per-song fingerprint reuse; stale on lyric change; never in a help bundle; founder: ≥ 5 rehearsal → service pairs, learned order = truth order in ≥ 90% of songs; held-out with practice better on recall and median latency, worse on none.

> /10x-Team:10x-team
> Phase 4, roles: Senior Engineer + SDE + DBA (file format). Precondition: V11 merged; ADR-023 Accepted. Two lanes; record `reviews/YYYY-MM-DD-ponytail-v12.md`.
> 1. `src-tauri/src/practice/`: the ADR-023 schema v1 (serde), load/merge/save with atomic writes (temp + rename), the merge rule (last 3 confirmed runs; medians; `plan` from the newest confirmed run with confidence ≥ 0.6), staleness by `recog`.
> 2. Move the FNV-1a fingerprint (`src-tauri/src/cli/voxxad/events.rs:131-138`) to the library and apply it per song; do not write a second hash.
> 3. `voxxad --learn <session.jsonl> --setlist <file> --out <church_root>\practice`: hindsight alignment of the logged words per song (moves stay / next / back to a section start / back to the song start / skip one, so a restarted song needs no operator input); writes runs as `proposed`; prints per song only section labels (the setlist's `sections`, filled by V5 step 9), part count, unsure count and confidence. `voxxad --confirm <song_fp> --practice-dir <church_root>\practice` marks that song's newest proposed run confirmed. These two forms are the only ones; the kit's checklist, prompt 05 and the voxxa-practice-session skill use them word for word.
> 4. The follower reads `plan.order`, `dwell_ms`, `max_gap_ms` (+ 2 s gap hold) and `heard` when present and valid; otherwise cold rules.
> 5. Tests with made-up songs: round trip; a changed slide text gives a new fingerprint and the old file is ignored; an unconfirmed run never sets `plan`; a song restarted from its first slide is learned with no operator input; `--confirm` without `--practice-dir` is refused with one sentence; the help bundle never includes `practice/`; no CLI output contains a word from a sentinel song.
> 6. Founder checklist (`kit/prompts/05-practice-session.md`): capture with `voxxad --device … --shadow --setlist <church_root>\services\<date>-<slot>.setlist.json --log-dir <church_root>\sessions` during rehearsal; learn; confirm; then the service in shadow; numbers only. On the founder's booth `<church_root>` is `C:\dev\voxxa-church-private\pilot\`; at other churches the commands come from the V5c tools zip.
> Budget: practice row 300 (the learner, merge rule and file IO live in `src-tauri/src/practice/`); voxxad row ≤ 20 lines: `--learn`/`--confirm`/`--out`/`--practice-dir` parsing and dispatch into the practice module (ADR-028 § 3). Safety: standard.
> State files: `decisions/{sde,dba,qa}/church-autoadvance.md`, `status.md`, `handoff.md`. Exit: tests pass; replay unchanged; the founder's ≥ 5 pairs meet the bars (may complete after merge as a checklist item, as Q8's usability run did).

### V13 — Master layout v1: validator, compiler, suggest-only cues, hotkeys, HTTP routes

**Round 3 (draft, 2026-11-26):** **split:** V13a (hotkeys F14/F15/F20 and routes `/pause /resume /panic /state`, S) and V13s (Sunday readiness, M) are new steps below; the rest of V13 (layout, service plan, cue engine) becomes **V13b, deferred** behind the first autonomous Sunday (R17). Prompt: `plan-round3.md` § 2 where it has a block for this step.

- **Size** M · **Phase** 4 · **Owner** Sr Eng 2 (+ UI consultant for copy) · **Depends on** ADR-024, ADR-025 Accepted; V7; V8; V11; F3 (test card); FA-17 (override hardware) · **Unblocks** V14, Q12 item refs, Q18
- **Acceptance:** schemas for layout, service and bindings; `voxxad --check-layout` with ≥ 25 refusal tests; compile byte-identical for an existing fixture; cue matcher suggest-only; arming and media disarm; 3 cues / 3 decoys replay; hotkeys with modifier wait; 4 HTTP routes; WorshipTools graphics and scripture as hints; `--shadow` logs cues and sends nothing.

> /10x-Team:10x-team
> Phase 4, roles: SDE + Senior Engineer + Security Engineer + UI consultant. Precondition: ADR-024 and ADR-025 Accepted; V7, V8, V11 merged; the founder's WorshipTools test card 2 results (F3) and FA-17 answer are in `reviews/`. Read `kit/templates/{layout,service}.template.json` and `kit/templates/bindings/*.json`. Two lanes (`v13/10x`, `v13/ponytail`); record `reviews/YYYY-MM-DD-ponytail-v13.md`.
> 1. `src-tauri/src/layout/`: serde types for `layout.json` (v1), `*.service.json` (v1) and binding profiles (v1); the binding profiles from `kit/templates/bindings/` compiled in with `include_str!`; JSON Schemas for editors in `kit/templates/schema/`.
> 2. `voxxad --check-layout <file>`: every ADR-024 § 3 rule, each with a refusal test (≥ 25), including `layout_refuses_run_key_and_shell_programs`, `service_file_cannot_raise_modes`, `on_start_refuses_clear_all`, `service_cannot_call_non_callable_action`, `apptoken_rejects_braces_and_dot_segments`, a `lyrics` key, a BOM. Errors are one plain sentence with a line number. The `via` rule (ADR-024 § 2): a graphic entry `graphics.<id>.<profile>` whose `via` does not exist in that profile is refused; a graphic with no entry for the active profile falls back to `hint` and prints a warning, not an error; a `service_callable` composite with a step written in `layout.json` of kind `http` or `cli` is refused (abstract actions inside it are allowed). Each has a test.
> 3. Compile a service to today's `setlist.json` (songs only) plus an item timeline; a test proves a service made from an existing public fixture compiles byte-identical to its `setlist.json`; replay unchanged. For keys-only apps, check `service_position` continuity and name the item in between (amber).
> 4. Trigger engine v1 per ADR-024 § 5: item pointer; cue matcher (ordered, ≤ 1 inserted word, 6 s, one edit for tokens ≥ 5 letters, cooldowns); arming only in listed spoken items with no song anchor for ≥ 30 s; disarm on media/announcement; suggest only; priority and 5 s operator hold; numbers-only `cue` log lines. Tests: `cues_disarmed_through_mid_song_instrumental`, `media_item_disarms_audio_triggers`, and a replayed service (made-up words) with 3 cue phrases and 3 decoys (in a song, in a sermon sentence) → 3/3 suggestions within 1.5 s, 0 decoys.
> 5. Hotkeys (ADR-024 § 6) through the already-loaded global-shortcut plugin (`lib.rs:119`): registered while Songs is on; F13–F24 defaults (pedal or Stream Deck); keyboard F-keys only for apps other than WorshipTools (its shortcuts page documents F6–F9), after a conflict check; never Ctrl+Alt+arrows; `hotkey_waits_for_modifier_release`; no hotkey stops listening or turns voxxa Off; a failed registration shows one sentence and Run continues.
> 6. HTTP: `POST /api/v1/pause`, `/resume`, `/accept`, `/dismiss` with the existing token and origin rules (`http_api/mod.rs:76-90`); document them in `docs/INTEGRATIONS.md` with PowerShell `Invoke-RestMethod`, curl and Companion examples.
> 7. Execution: song moves through existing driver methods. Graphic and scripture actions resolve per profile (ADR-024 § 2): a missing entry for the active profile becomes a `hint`; a key that exists runs only after its contract test passes, and until then the driver turns it into a hint (ADR-024 § 5). No profile has a passed contract test yet, so in v1 all graphic and scripture actions are operator hints.
> Budget: layout row 300 (validator, compiler and trigger engine in `src-tauri/src/layout/`); wiring row (hotkeys, HTTP routes); voxxad row ≤ 10 lines: `--check-layout` parsing and dispatch into the layout module (ADR-028 § 3). Safety: standard; never the real Presenter; fake drivers only.
> State files: `decisions/{sde,security,qa}/church-autoadvance.md`, `status.md`, `handoff.md`, tickets file. Exit: all tests pass on all OSes; `--shadow` with a layout logs cue suggestions and dispatches nothing; CI green.

### V14 — Run screen (U1 subset)

**Founder answers (2026-11-27):** FA-61 as answered: a manual move re-anchors the follower, no demotion, badge stays "voxxa moves it"; FA-60: proceed; FA-59: no Companion page; FA-52: shadow mode shows one line and nothing else. Details in `plan-round3.md` V14 and `founder-action-items.md` batch 3.

**Round 3 (draft, 2026-11-26):** re-scoped to the minder UI: badge words "voxxa moves it / voxxa suggests / You move it", NOW/NEXT firm and provisional, the amber hold sentence, HOLD / Back one / STOP, the stage strip in three states with the heard-words line removed (R9, FA-60), the re-anchor rule (a manual move re-anchors the follower, no demotion, FA-61 as answered 2026-11-27), shadow mode's single line (FA-52), the a11y contrast fixes. Depends on V10, V13a, V13s. Exit: sentences match the approved list; zero CSP violations; Run ≤ 350, health ≤ 450 (estimate 150) lines. Prompt: `plan-round3.md` § 2 where it has a block for this step.

- **Size** M · **Phase** 4 · **Owner** UI consultant + SDE · **Depends on** V10 (founder walkthrough), V11, V13, FA-21 · **Unblocks** V15
- **Acceptance:** Pause/Resume (F15 from a pedal, Space in voxxa's window, the HTTP route), reason line, Sure/Not sure/Lost, Off confirm while singing heard, keys-only Resume card with no default, off-by-one keys, accessibility fixes; GUI logic and e2e tests; `ui-consistency:verifying`; Run row ≤ 350.

> /10x-Team:10x-team
> Phase 4, roles: SDE + UI consultant + QA Engineer. Precondition: V10 walkthrough recorded; V11 and V13 merged; FA-21 answered. Read ADR-017, ADR-024 and `src/simple/*`. Two lanes; record `reviews/YYYY-MM-DD-ponytail-v14.md`; run `ui-consistency:verifying`.
> 1. Simple mode stays the default view (ADR-017). Add, in a new `src/run/` (subtracted from the GUI row like Simple mode): Pause/Resume (F15 from a pedal or Stream Deck, Space when voxxa's window is in front, and the HTTP route; Space starts and stops listening today, `src/main.js:1583-1586`, and becomes Pause/Resume on Run, with a test); a one-sentence reason per move from the follower's `why` (table in spec § 4.7); Sure / Not sure / Lost from the certainty value (proposed thresholds 0.80 / 0.50, tuned in shadow); an Off confirm with "Keep following" focused while singing was heard in the last 5 minutes; on keys-only apps a Resume card that asks "Presenter shows slide __ of __ (not counting its blank)?" with no default button; "screen is ahead / behind" via F18/F19. If FA-21 says yes: the first line of the current and next slide (operator only; amend ADR-017).
> 2. Accessibility: `--accent-strong` (white text ≥ 4.5:1), `--field-border` (≥ 3:1), one `:focus-visible` ring, 44 px live targets, a symbol in each status dot, numeric countdown under reduced motion.
> 3. Tests: pure-logic tests like `tests/gui/simple_logic.test.mjs`; e2e in `tests/gui/e2e.mjs` with zero CSP violations.
> Budget: Run row 350 (new, ADR-028); Simple mode ≤ 60 lines. Safety: standard; GUI only through the mocked browser and a guarded scratch build (own identifier, CDP, no mic).
> State files: `decisions/{sde,qa}/church-autoadvance.md`, `status.md`, `handoff.md`. Exit: tests green; the founder's usability script item updated (checklist).

### V15 — Close the V increment

**Round 3 (draft, 2026-11-26):** close: read-back as shadow truth (FA-66), the runbook set, acceptance counted in occurrences, the minder card passed by two volunteers, founder ≤ 45 min per scored Sunday; the two gates of FA-52 (2 scored shadow Sundays for songs mode with a minder; the later no-minder gate). Prompt: `plan-round3.md` § 2 where it has a block for this step.

- **Size** S · **Phase** 5 + 6 · **Owner** Engineering Manager (QA, Security, DevOps, SRE roles) · **Depends on** V0–V14 and V5c; the scrub PR (V5 step 6) merged · **Unblocks** shadow Sundays, the default flip
- **Acceptance:** every review-20 § 5 safety test that applies to built features exists and passes (the CLI/HTTP-step tests are listed as deferred with their features); privacy scan of `kit/`, `.10x/`, `fixtures/` clean; fake-church kit validation, including the V5c tools zip; the shadow-Sunday scoring procedure written as a founder checklist; release notes; SRE runbook "voxxa is not sure / position uncertain"; QA and security review files.

> /10x-Team:10x-team
> Phase 5 + 6, roles: QA Engineer + Security Engineer + DevOps Engineer + SRE. Close step: 10x + `ponytail` + `ponytail-audit`, not two build lanes. Run the built-in code review before the close merge, and `security-review`.
> QA: list the safety tests from the spec § 7 item 4 and show each passes; run the replay eval and the stress set; repeat the fake second church at `C:\dev\voxxa-church-private\fake-church-2\` from the kit alone, including `voxxad --version` and `voxxa-eval --help` from the V5c tools zip with no repo checkout. Security: STRIDE-lite on the new boundaries (layout and service files from outside, hotkeys, the 4 HTTP routes, practice files, session-log location, the tools zip); run `kit_check.py` on `kit/`, `.10x/` and `fixtures/` (the founder runs the lyric scan and the deny-file on the booth PC and pastes counts and `file:line` only). The deny-file scan becomes a gate over `.10x/` and `fixtures/` only after the V5 step 6 scrub PR has merged; with FA-12 (c) it stays on `kit/` only. DevOps: release notes; no tag unless pre-approved. SRE: runbook `docs/runbooks/voxxa-not-sure.md` (Not sure, Lost, position uncertain, Resume card) linked from the status rows.
> Founder checklist, shadow Sundays: 2 consecutive shadow Sundays with the follower on (`--shadow` or the GUI toggle when Q14 lands) meeting the Correct bar before the default-flip PR. In shadow mode on WorshipTools voxxa cannot see the operator's moves (no read-back, ADR-019), so write the scoring as a checklist for each Sunday: (1) record the desk feed to a `.wav` during the shadow service; this needs FA-06 option (b), consent for service audio, with talk spans muted after the replay is scored, before the recording is kept; (2) tap truth afterwards with `<repo>\fixtures\tools\tap-truth.ps1` (it writes `truth.jsonl` next to the wav it is given); (3) replay the recording offline on the booth PC with `voxxad --stdin-audio <wav> --shadow --setlist <church_root>\services\<date>-<slot>.setlist.json --realtime`, the same build and configuration as the live run (`--realtime` feeds the audio at live speed, so the keep-up gate drops the hops it would drop live, spec § 2.3); (4) score the replay's actions with `voxxa-eval --actions` against the tapped truth; (5) use the live shadow session log from `voxxad --device … --shadow` only as a cross-check, because its clock is not aligned to the recording. Operator notes are context, never the score. The founder pastes the numbers only.
> State files: `.10x/reviews/YYYY-MM-DD-qa-report.md`, `…-security-review.md`, `decisions/{qa,security,devops,sre}/church-autoadvance.md`, `status.md`, `handoff.md`. Exit: no open wrong-slide or privacy defect; findings triaged as fix-now or ticket.

### V16 — Lowercase "voxxa" rename and the circle-logo icon

**Round 3 (draft, 2026-11-26):** **deferred** behind the first autonomous Sunday (R17, FA-72). Prompt: `plan-round3.md` § 2 where it has a block for this step.

- **Size** S · **Phase** 4 + 6 · **Owner** Sr Eng 3 · **Depends on** FA-24 (timing), FA-36 (icon source and small variant), Q11 · **Unblocks** the first full v0.1.0 publish
- **Acceptance:** `productName` and window titles lowercase; the impact on the exe name, install folder, updater artefact names, winget identifier `nategroks.Voxxa` and the data directory identifier `com.voxxa.app` listed and tested on a test PC (never the booth PC); the church folder `%LOCALAPPDATA%\voxxa-church\` is untouched by install, update, uninstall and the rename; docs and website text lowercase; the icon set generated from the circle logo with `tauri icon`, with the simplified 16 px and 32 px variant.

> /10x-Team:10x-team
> Phase 4 + 6, roles: DevOps Engineer + SDE + SRE. Precondition: the founder chose the timing (FA-24). Two lanes; record `reviews/YYYY-MM-DD-ponytail-v16.md`. Change `productName` and window titles to lowercase "voxxa" (`src-tauri/tauri.conf.json`); list every place the name flows into (exe, NSIS install folder and uninstall entry, Start menu, MSI, updater artefacts and `latest.json` names, `scripts/release/*`, `winget.yml`, `tests/release_config.rs`), and whether the identifier `com.voxxa.app` (data directory) changes (it must not). On Windows NTFS a case-only folder change does not move `%LOCALAPPDATA%\Voxxa\`: check what NSIS does with an existing install [UNVERIFIED]. The church folder `%LOCALAPPDATA%\voxxa-church\` is a sibling of the install folder and of the app-data folder `%LOCALAPPDATA%\com.voxxa.app\`: on the test PC, put a made-up church folder there and show that install over the old version, update, uninstall and the rename leave it byte for byte. Test only with a scratch identifier build on a test PC. Update docs and website text. **Icon (FA-36):** from the founder's `voxxa_logo_circle.png` (teal contour lines in a white and teal ring, no text; a square PNG of at least 1024 px), run `npx tauri icon <file>` to regenerate `src-tauri/icons/` (7 files on `866b51e`); then replace `32x32.png`, the 16 px and 32 px images inside `icon.ico`, and `tray.png` (which `tauri icon` does not make) with the simplified variant the founder approved, because the contour lines blur at small sizes. Check the exe, installer, tray and taskbar icons on a test PC. Exit: release config tests updated in the same commit; CI green; a founder checklist for an install-over-old-version test on a test PC, which also confirms the new icon at 16, 32 and 256 px.

### F2 — Founder: booth runs (rolling)

V0's ≤ 150 s check; V3's decoder run; V3b's gate cases and take scoring; V3c's A/B runs; V4's GPU build and no-loader test; V6/V11 private runs; V12's rehearsal pairs. Each has a checklist written by the step; numbers only.

### F3 — Founder: WorshipTools test card 2 (before V13)

The founder runs `kit/templates/presenter-test-card.md` on the booth PC for the rows still open on WorshipTools Presenter, using the card's numbers: T4: which key restores the lyrics after F10? T5: Right on a song's last slide when the next Presenter item is a graphic: what shows? T6: may voxxa ever send Down (next service item)? (ADR-019 forbids it today.) T7: function keys: Presenter's use of F6–F9 is documented on its shortcuts page; confirm on the card, and whether it reacts to F13–F19. T8: confirm the `goto.keys` chord (Right Right from a song's last slide into the next song's blank and first slide). T9: window class/title of Presenter's main window and of an edit dialog (S-11). T10: Presenter not run as administrator. Record in `reviews/YYYY-MM-DD-worshiptools-test-card-2.md`.

### F4 — Founder: supervised live Presenter test (V3b)

**Round 3 (draft, 2026-11-26):** the parts map's first live run moves to V3e (non-projected FreeShow); Presenter live tests stay Right-only with suggest-only off-plan moves (FA-44 answer (e)). Prompt: `plan-round3.md` § 2 where it has a block for this step.

The founder runs V3b's live checklist (task 5) on the booth PC on a weekday with no service or rehearsal. The first run may use today's feed and vocal mic, conditioned by the helper as in the lab (100 Hz high-pass, +12 dB); FA-38 and FA-39 are deferred (2026-11-26), so every run uses this feed, and its level and hum numbers are recorded as the degraded-mode baseline. Before every run: Presenter shows the test song (FA-41) on its leading blank, and the level and hum check numbers are pasted. First `live.py --dry-run` for one full song, then the same song with keys. The lab helper presses Right only while a Presenter.exe window is in front (ADR-019 § 2), never Left, at most 3 steps per move. The founder stays at the keyboard and stops it with Ctrl+C at the first wrong move; `--minutes` caps every run. The helper runs a local-only voxxad build with streaming stdin (`--stdin-audio - --realtime`, `--shadow`; voxxa-private, never pushed). The feed is recorded, truth tapped and scored as in V3b task 4. Record moves, onset fires, word fires, wrong slides, early moves and Tight on time in `reviews/YYYY-MM-DD-f4-live-presenter.md`; numbers only.

---

### Round 3 new steps (draft, 2026-11-26)

Full prompts in `plan-round3.md` § 2 (copy of `round2/52-prompts-round-3.md`); sizes, costs and exits from `round2/51` § 2.2.

#### V3e — FreeShow probe, VRAM with both apps, PD-hymn import counts, Presenter MIDI check, one non-projected live parts-map run

- **Size** S · **Lanes** 10x + audit · **Cost** $7–19 · **Founder** 2 h · **When** a weekday before Dec 18 · **Depends on** F5, FA-46 · **Unblocks** V8b (on a go), V3f (VRAM numbers), FA-48's sheet, V3b task 5, V3d's live run
- **Acceptance:** a go / no-go / Holyrics-fallback table with numbers (founder-run, numbers only): wire facts C1–C3 confirmed on the installed version, read-back latency, VRAM and CPU with Presenter, FreeShow and voxxad running and a video playing, PD-hymn import counts, the Presenter MIDI answer (FA-53), one non-projected live run of the parts map; FA-48's sheet filled.

#### F5 — Founder: FreeShow stable beside Presenter

Install the stable channel, auto-update off (FA-51), set the API password and firewall 5505/5506 to loopback (FA-55), build a unique-layout test show (FA-56), measure the text transition (FA-57), paste the version and VRAM with both apps. About 1 h. Depends on FA-46, FA-51, FA-55.

#### V8b — FreeShow driver v2 (ADR-032)

- **Size** M · **Lanes** 2 · **Cost** $10–25 / $24–60 (≤ $60, FA-49) · **Founder** 30 min contract check · **Depends on** V7, V8-lite, a V3e go · **Unblocks** V11's Tier A interface, F7, Q12/Q13
- **Acceptance:** top-level body; 1-based index; `showId`/`layoutId` from `get_output`; song select via `name_select_show`; `current_state` ids only; fire-then-verify (one re-read, latency from V3e); `position_uncertain`; bearer header; refuse an open API unless `require_password` is set to false (default true, FA-55); `get_show`/`get_shows` only through an id-and-name parser that discards the body; never-call list for the text getters; `--watch` screen log; a TcpListener stub; the ten named tests of `plan-round3.md` V8b on 3 OSes; replay byte-identical; a chorus repeated by one call on dark FreeShow, confirmed by `get_output`; kit binding and test-card rows FS1–FS20.

#### V3f — Aligner and OLTW lab measurement only (R5)

- **Size** S/M · **Lanes** 10x + audit · **Cost** $3–25 / $7–55 · **Founder** 2 h · **Depends on** V3e numbers · **Unblocks** ADR-033 only if V3f **and** V6 pass
- **Acceptance:** OLTW within a part against a prior take (CPU, no model) and a CRNN/CTC alignment of the second after each onset against candidate first lines; `tight_300`, boundary delay, early moves, VRAM and ms per update on the GTX 960; a verdict against the bar in its prompt. No product code, no Python sidecar (ADR-003), permissive weights only (FA-50); CC-BY-NC weights lab-only.

#### V13a — Hotkeys F14/F15/F20 and routes `/pause /resume /panic /state`

- **Size** S · **Lanes** 2 · **Cost** $3–8 / $7–19 · **Depends on** V7 · **Unblocks** V13s, V14
- **Acceptance:** HOLD F15 (toggle: pause following, keep listening and logging), STOP FOLLOWING F20 (dispatch off for the rest of the service; current slide held, nothing sent), Back one F14; routes on 127.0.0.1; no Escape; Off stays in the mode switch (R8, FA-58); `panic_stops_dispatch_keeps_listening`; `--shadow` sends nothing.

#### V13s — Sunday readiness (ADR-034)

- **Size** M · **Lanes** 2 · **Cost** $10–25 / $24–60 · **Founder** 30 min · **Depends on** V13a, V2 · **Unblocks** V14, V15, the first shadow Sunday with a minder
- **Acceptance:** health check page (6 rows); an off-thread watchdog with heartbeats (4 signals); hold and panic post-states; the service manifest frozen at the last rehearsal; a numbers-only per-service report; runbooks (sunday-window, watchdog-tripped, panic-pressed); RDP check (`SM_REMOTESESSION`); device pinned by name; named watchdog tests; `--check-ready` prints every item with a number; panic tested with fake drivers.

#### V17 — FreeShow migration kit and library re-import, church side (ADR-035)

- **Size** M · **Lanes** 2 · **Cost** $10–25 / $24–60 · **Founder** 3–4 h over 6 weeks [est.] · **Depends on** a V3e go, FA-48, FA-68 · **Window** after the Christmas freeze, Jan 10 – Mar 21, 2027
- **Acceptance:** kit files pass `kit_check`; the repertoire inventory counts are signed by the worship leader; the first ten (most-sung licensed) songs re-imported with counts equal to `songs.csv`; per-service switch, whole service, 4 double-build Sundays, Thursday 18:00 rollback rule, Presenter kept a month (FA-48).

#### V18 — Minder training programme and kit

- **Size** M · **Lanes** 2 · **Cost** $10–25 / $24–60 · **Founder** 2 × 15 min · **Depends on** V10's words · **Note** role card v0 ships as docs in week 1 (C15)
- **Acceptance:** role card v0 (Presenter) and v1 (FreeShow), the ten-minute session, a monthly drill with fault cards, a rota (3 names for 8 Sundays, FA-69), a worship-leader card, a congregation notice (FA-70), prompts 05/06/07 edits; two volunteers pass the v0 card on a public-fixture drill.

#### F6 — Founder: monthly fault drill and one manual song a month

10 minutes a month after V18; counts recorded.

#### F7 — Founder: dark Sundays (Jan 10, 17, 24, 2027) and the FA-48 decision

15 minutes each: FreeShow installed with no output, voxxa drives it in shadow, read-back logged; read-back agreement and `position_uncertain` counts per Sunday; FA-48 decided at the end of January. Depends on F5, V8b.

## 5. Tickets to file (one founder batch, FA-02)

| Ticket | Content | Step |
|---|---|---|
| #59 comment | cause (cmake crate 0.1.58 MSVC release flags), fix, numbers | V0 |
| new: partial key sequence leaves voxxa N slides off | `dispatch.rs:163-175`; S-10 | V8 |
| new: universal keystroke profile types into any window | `keystroke.rs:59-63`, `INTEGRATIONS.md:260-266`; S-09 | V8 |
| new: W1 FreeShow body shape; W2 OpenLP v2 paths/auth; W3 OpenSong paths/auth | draft 11 § 0.1, [UNVERIFIED live] | Q13 |
| new: pin `ci.yml` and `pages.yml` actions by SHA | draft 12 § 1.5 | separate S |
| new: help bundle words default on | `src/simple/help.js:38`; S-37 | V9 |
| new: session logs in roaming `%APPDATA%` | `help/mod.rs:181`; S-36 | V9 |
| new: church identifiers and song titles in public files | S-42; the `path:line` list in V5 step 6 (only if FA-12 says scrub) | V5 |
| new: release the CLI tools as a signed `voxxa-tools-<version>-windows-x64.zip` | spec § 4.9; `Cargo.toml:21-35`, `:121-122`; FA-31 | V5c |
| new: updater refuses to install between Saturday 18:00 and Sunday 14:00 local time | spec § 4.12 item 1; updater row 98/150 (only if FA-32 is (a) or (b)) | before church #2 |
| new: pin Whisper model downloads by SHA-256; a model change goes through the eval gate | spec § 4.12 item 3; no hash check in `src-tauri/src` today (only if FA-32 is (a)) | before church #2 |
| new: input chain: channel select, a 100 Hz high-pass and gain or AGC before the Silero VAD; a level and hum check at setup | spec § 4.1; ADR-030 § 5; raw singing at about −34 dBFS gave 2 Whisper passes in 200 s, +12 dB gave 152 (only if FA-40 is (a) or (b)) | V3b |
