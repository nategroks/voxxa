# voxxa: what an agent on this repository must know

Short sections only; the reasons live in `.10x/`. Added by the round-3 Records PR (2026-11-27); later steps add the
sections `.10x/specs/2026-11-26-skills-and-context-layout.md` § 3.1 names (FreeShow driver facts in V3e/V8b, Church
kit work rules in V5/V17, GUI testing in V10/V14).

## Read first
1. `.10x/handoff.md` (the safety block and the budget row table), then `.10x/status.md`.
2. `.10x/specs/2026-11-26-skills-and-context-layout.md`: where every file goes, the lane scan, what each step reads.
3. The prompt you are running: `.10x/specs/2026-11-26-prompts-round-3.md` (V-steps, round 3) over
   `.10x/specs/2026-11-24-church-autoadvance-plan.md`; founder decisions in `.10x/specs/2026-11-24-founder-action-items.md`.
4. The ADRs the prompt names (`.10x/adrs/`); amendments are dated sections at the end of each ADR.

## Sunday safety lines
- Never open the mic unattended. Listening starts only when a person arms Songs.
- Never press keys in WorshipTools Presenter or FreeShow; never send a FreeShow action to a real install from a lane.
- Never start the installed app (`com.voxxa.app`) or touch `%LOCALAPPDATA%\Voxxa\`; GUI work uses the guarded scratch build.
- Never read, print, quote or echo lyrics or heard words. Slides are indices; parts are labels (V1, C, B); the private
  song is "P1"; leaders are L1/L2; takes are T1, live1..live4, F1..F4; "the booth PC" and "the pilot church", never names.
- Never open the private folders (`C:\dev\voxxa-church-private\`, `C:\dev\voxxa-private\`, `fixtures/private/`) beyond the
  file a prompt names; never a recording, `.wav` or `.mp4`.
- Never run the health check, the hotkeys or the watchdog against the real app or a real slides app; use the canary
  fixture (the committed public-domain TTS take) and fake drivers.
- No remote desktop on the booth PC Saturday 18:00 to Sunday 14:00 (FA-67); nothing runs on the booth on a service day.
- Stop your own processes by PID before you finish. The "media guard" check stays required on `main`.
- Numbers only in anything that leaves the PC: counts, indices, milliseconds, yes/no, key names.
- Run the lane scan (`.10x/specs/2026-11-26-skills-and-context-layout.md` § 4.4) on staged files before every commit.
  Allowed matches: `127.0.0.1`, `<church_root>`, `<lab>`, the three lab file names when the prompt names them.

## Budget rows
- Caps only go down. A new module gets a new recorded row through an ADR; never raise a cap; stop and report at 95 %.
- V7 (2026-11-27) landed the wiring row (`dispatch.rs`, `listen.rs`, `http_api/`, `commands/`: 1,288 / 1,438, 150 left,
  spent only through the ceilings in ADR-028 amendment 1 § 5) and lowered "everything else in `src-tauri/src`" to
  900 / 904: its 4 lines are reserved for the `lib.rs` `mod` lines of `input`, `position`, `readiness`, `practice`. Row table: `.10x/handoff.md`.
- Rows V7 added (ADR-028 amendment 1; the founder's "× 3" rule, FA-54): `onset` 750 (estimate 250), `input chain` 600
  (200), `health check (src/health/)` 450 (150), `position` 900 (300, ADR-036), all at 0; `vad/` moved into the recognizer
  row (557 / 825, headroom unchanged); `align` 1,200 only on an R5 pass. The review trigger fires when a tripled row passes its estimate.
- The earlier rows keep their caps (follower ≤ 500, practice 300, layout 300, Run 350, kit tools 300); a step whose
  measured code cannot fit stops at its Phase 2 check and asks for 3 × the measured number for that row (FA-74).
- V11 ports one module, `aligner/follower/parts.rs`, replacing `aligner/evidence/` under one switch (R12).

## Plugins
- KEEP on every lane: `10x-Team`, the ponytail plugin, `superpowers`, `writing-skills`, `supermatt` (dependency
  vetting), `backend-design` (security reviewer); the built-ins `security-review` and `workflow-authoring` stay on.
- `ui-consistency` is KEEP, but its hooks run only in the V14 lane (and the V10 mock review).
- OFF on every unattended Q/V lane: `beads-superpowers`, `iceberg`, `agentic-council`, `desktop-commander`, and every
  connector that reaches church data or an outside service (Planning Center, Google Drive, Gmail, Slack, Notion,
  Canva, Wix).
- If a KEEP plugin is missing when a lane starts, stop and say so; do not improvise.

## JSONL and records
- Schema v1 gains one line type, `trace`, with a required `kind` (`follow`, `verify`, `screen`, `watchdog`, `operator`,
  `level`, `cue`) and numbers-only fields per kind; replay skips it like `window`; a `kind` that carries text fails the
  serialisation test (FA-65, ADR-001/003 amendment written in V7). No `follow`, `cue` or read-back line type is ever added.
- Every number in a record is marked (measured), (cited), (inferred) or (opinion), with who measured it and when.
- Records use `file:line` references, take ids and part labels; the private song is "P1"; leaders are L1/L2.
- Lab results are pasted as tables of numbers into `.10x/reviews/`; never a heard word, never a frame, never a path
  under a user profile.
