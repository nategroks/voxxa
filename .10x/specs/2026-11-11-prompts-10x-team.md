# Voxxa — Claude Code Prompt Pack for `/10x-Team:10x-team` (regenerated 2026-11-11)

Replaces §7 of `2026-10-24-delivery-plan-and-prompts.md`. Reason: discovery of `origin/claude/affectionate-lamport-8al8B` (ADR-010), which already holds most of what Increments 1–3 planned to build. Increment 0 is done and CI-verified on PR #1.

## How to use

Each prompt is the first message of a fresh Claude Code session in the repo root, with the ponytail plugin installed (`full`). Every prompt opens with `/10x-Team:10x-team` so the team protocol applies: read `.10x/handoff.md` and `.10x/status.md` first, work only inside the named phase, write the named state files before finishing, and stop at the gate the prompt names. The numbering (Q0…Q27) is new so it cannot be confused with the October P-numbers.

Sizes and token bands are unchanged from the October plan (S: 5–10 turns, 25–50k output, $3–8 at list; M: 15–30 turns, 75–150k, $10–25; L: 30–60 turns, 150–300k, $25–50; Opus 5.5, cache reads dominate total tokens; 1.5× rework on L).

| | Count | Output tokens | API list cost |
|---|---|---|---|
| S | 9 | 0.2–0.45M | $27–72 |
| M | 14 | 1.05–2.1M | $140–350 |
| L | 5 | 0.75–1.5M | $125–250 |
| **Pack (28 prompts)** | | **≈ 2.0–4.1M** | **≈ $290–670; ≈ $450–1,000 with rework** |

Per day: part-time 1 prompt ≈ 75–150k output tokens; full-time 2–3 ≈ 150–400k. Calendar: ≈ 18 weeks part-time, ≈ 10 weeks full-time; the pilot (shadow mode from Q14) and signing paperwork still gate the middle of the plan, not tokens.

Phase map per increment: implementation prompts are Phase 4 (SDE + Senior + DBA); each increment ends with one **close** prompt that runs Phase 5 (QA + Security → `.10x/reviews/`) and Phase 6 (DevOps + SRE → release notes, runbooks, `decisions/devops`, `decisions/sre`). The 10x-team gates are mandatory: no code before the ADR exists, no "done" without the QA and security files.

---

## Stage A — Reconcile (Phases 0–2, then 4–5)

**Q0 — Discovery of the May branch and the reconciliation decision** (M)
> /10x-Team:10x-team
> Phase 0–2 (Discovery → Strategy → Design), roles: CTO, Architect, Staff Engineer. Read `.10x/handoff.md`, `.10x/status.md`, `.10x/decisions/architect/_index.md` (the [DISCOVERED 2026-11-11] section) and `.10x/adrs/010-adopt-phase-1-3-branch-as-code-base.md`. Run the Discovery Protocol on `origin/claude/affectionate-lamport-8al8B` (`git worktree add ../lamport origin/claude/affectionate-lamport-8al8B`): confirm or correct every [DISCOVERED] statement by reading the code, not the docs; add anything missed to the role indexes with the tag. Then review ADR-010 as the Architect: is the conflict rule in Decision 1 complete (list every path that differs between the two branches with `git diff --name-status origin/claude/trusting-newton-v71ebu origin/claude/affectionate-lamport-8al8B` and assign each a side)? Append that table to ADR-010. Present the findings to the user in the Discovery format ("Here's what I found… Does this look right?") and STOP at the user-review gate: do not merge, do not write code. State files: role indexes, `status.md` (phase: Discovery of May branch complete), `handoff.md` (to SDE for Q1 once ADR-010 is Accepted). Do not propose new features. Exit: ADR-010 carries the full path table and the user has approved or amended it.

**Q1 — Merge into one branch and meet the CI bar** (L)
> /10x-Team:10x-team
> Phase 4 (Implementation), roles: SDE + Staff Engineer + DBA (dependency hygiene). Precondition: ADR-010 Accepted. On `claude/trusting-newton-v71ebu`, `git merge --no-ff origin/claude/affectionate-lamport-8al8B` and resolve every conflict by ADR-010's path table; never rewrite history. Then: remove `tauri-plugin-store` (the GUI uses localStorage; grep proves it) and the `text_insert` module (replaced by `presenters/keystroke`); keep the `macos-private-api` feature; combine `build.yml` (their signing scaffolds, our release body); make `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo deny check licenses advisories bans` and `scripts/loc_budget.py` pass. For `cargo deny`: compatible updates first (`cargo update -p <crate>`), then minimal major bumps for `zip`, `quick-xml`, `rusqlite`; move `reqwest` to `rustls-tls` to drop OpenSSL advisories; if `pdf-extract`/`lopdf` has no fixed release, gate the PDF importer behind a Cargo feature that is off by default and say so in `docs/SETLIST_FORMATS.md` rather than ignoring the advisory. Re-base `scripts/loc_budget.py` to ADR-010's per-module budgets. Run the four integration tests and all unit tests. Push; PR #1 updates itself. State: `decisions/sde/church-autoadvance.md` (merge record, every dependency change and why, debt), `decisions/dba/church-autoadvance.md` (SQLite use in the EasyWorship importer: bundled vs system, licence), `status.md`, `handoff.md`. Do not add features, do not touch the conductor's logic, do not delete GUI panels. Exit: all five CI jobs green on the merged head; `README.md` still describes the church product; `git grep -i dictat` clean outside `.10x`.

**Q2 — Verification of the merged base** (M)
> /10x-Team:10x-team
> Phase 5 (Verification), roles: QA Engineer + Security Engineer. Read `docs/REVIEW.md` (the May branch's own punchlist) and `.10x/specs/2026-10-24-software-strategy.md` § Security and privacy. QA: build the app, load `examples/*` and `setlist.example.json`, exercise every Tauri command through the GUI once, run the HTTP API with and without a bearer token, unplug the audio device mid-listen; record every defect with repro steps in `.10x/reviews/2026-MM-DD-qa-report.md`; fix only crashes and data-loss defects now (ponytail: smallest change), list the rest as tickets. Security: STRIDE-lite on the two boundaries plus the new ones this code adds (HTTP API on 127.0.0.1, Hymnary and Planning Center outbound calls, mDNS, EasyWorship SQLite import of an untrusted file, pptx/PDF parsers on untrusted files); verify the outbound-request counter actually counts every `reqwest` client; confirm tokens are not logged by the ring-buffer logger; write `.10x/reviews/2026-MM-DD-security-review.md`. State: `decisions/qa/church-autoadvance.md`, `decisions/security/church-autoadvance.md`, `status.md` (release readiness), `handoff.md`. Exit: both reviews committed; no open crash or data-loss defect; security findings triaged as fix-now (done) or ticket.

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

**Q10 — Windows signing, winget, updater hardening** (M)
> /10x-Team:10x-team
> Phase 4 + 6, per ADR-008. In `build.yml`: keep the May branch's Trusted Signing scaffold, add the SignPath Foundation action as the preferred path (skips when secrets are absent), NSIS `perUser`, WebView2 bootstrapper, a `winget` manifest template and release step. In the updater client refuse manifests with a lower version than installed. Set a strict CSP. Unit test the version check. Exit: tag build produces signed NSIS and MSI when secrets exist; CSP no longer null.

**Q11 — Close Increment 2′** (S)
> /10x-Team:10x-team
> Phase 5 + 6. QA: run the five-volunteer first-run test from `specs/2026-10-24-user-research-plan.md` §2 on a Windows laptop (coordinate with the Windows session via `.10x/handoff-local-windows.md`); Security: review the support bundle for over-collection; DevOps: first signed pre-release; SRE: on-call runbook for the Sunday window. Reviews and decision files as in Q7.

## Stage D — Increment 3′: read-back, fixtures, pilots (Phases 4–6)

**Q12 — Presenter read-back and in-app setlist import** (L)
> /10x-Team:10x-team
> Phase 4, per ADR-006. Extend `PresentationController` with `current_slide_text() -> Result<Option<(String, Option<String>)>>` (current, next) and `setlist() -> Result<Option<Setlist>>`, capability-flagged. Implement for ProPresenter 7.9+ REST (`/v1/status/slide`, `/v1/playlist/current` + `/v1/presentation/{uuid}`), FreeShow (`get_output_slide_text`, `get_projects`/`get_show`), OpenLP v2 (`/api/v2/controller/live-items`, `/api/v2/service/items`). Add `Conductor::resync(song, slide)` and call it when `current_state`/text disagrees with the conductor twice in a row; the operator always wins. GUI: "Load today's setlist from <app>" in the loader. Extend the parity contract test to the new capabilities. Exit: all three drivers import a setlist from a live install (the founder or the pilot church provides access); resync demonstrated.

**Q13 — Recorded-traffic contract fixtures** (M)
> /10x-Team:10x-team
> Phase 4 + 5. Build a small in-process HTTP/WS stub that replays request/response fixtures from `src-tauri/tests/fixtures/<driver>/`; add `voxxad --record-adapter` to capture them once against real installs; commit fixtures for ProPresenter REST, FreeShow and OpenLP; make each driver's tests run against the stub in CI. Exit: three drivers covered; CI runs them on all OSes.

**Q14 — Shadow mode for pilots and corpus export** (S)
> /10x-Team:10x-team
> Phase 4. Add a "Shadow (log only)" toggle in Advanced that maps to `--shadow`; add `voxxad --export-corpus <dir>` that writes the consented session audio (only when recording was explicitly enabled by the operator), setlist and operator key log into the fixture format. Document the pilot protocol link in `docs/pilot.md`. Exit: The Lighthouse, Midlothian can run shadow mode for a Sunday and hand back a fixture.

**Q15 — Close Increment 3′** (S)
> /10x-Team:10x-team
> Phase 5 + 6. QA on the three live installs; security review of read-back (tokens in keychain, host binding, identity check via version endpoints before any command); DevOps release; SRE runbook "app reports a different slide than Voxxa". Reviews and decision files.

## Stage E — Increment 4′: scripture (Phases 2, 4–6)

**Q16 — Public-domain Bible bundle** (M) — as October P4.1, under ADR-007; confirm ADR-007 before coding.
**Q17 — Spoken scripture reference grammar** (M) — as October P4.2 (200-case test table; `bibleref` only if it already handles spoken numbers).
**Q18 — Scripture detection in the conductor and `show_ref` per driver** (L) — as October P4.3 + P4.4, but inside `Conductor` (new `MachineState::Reading` with chapter graph; back to songs on silence > 20 s or a song match); FreeShow `start_scripture`, OpenLP bibles search + live, ProPresenter and keystroke: operator hint. Two spoken-passage fixtures; eval reports reference precision.
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
