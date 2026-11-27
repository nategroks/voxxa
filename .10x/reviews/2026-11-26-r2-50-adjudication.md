# Round 2 adjudication: autonomous slide following (2026-11-26)

Orchestrator's rulings over the research report (`reports/Voxxa autonomous slide following research.md`) and the
seven reviews in this folder: 41 CTO, 42 Architect + Senior Engineer, 43 SRE, 44 UI designer, 45 Consultant,
46 ponytail lane (independent), 47 Implementer (the session that built the lab and ran live1..live4).
Every ruling names who said what. Where a reviewer measured something and another inferred the opposite, the
measurement wins. Nothing here is committed or approved yet: the founder decides the FA items in § 5.

## 1. What stands from the report

- **Direction (all seven agree):** the output path, not recognition, is the ceiling today. FreeShow is the right
  long-term live-output target (GPL, Windows, documented action API with slide-index selection and `get_output`
  read-back). The parts map (ADR-031) is the right follower. Learning happens offline in rehearsal review, never on
  Sunday. The volunteer becomes a "minder" with a health check, hold and stop controls, rehearsal gating and a
  one-page card.
- **Metrics:** the report's mapping of voxxa-eval columns to mir_eval and Matchmaker conventions is accepted as
  documentation (no renaming of columns, per CTO F8). The proposed gates stand as proposals, tightened in § 3.

## 2. What the report got wrong (corrected)

| # | Report said | Correction | Source |
|---|---|---|---|
| C1 | W1 (body nesting) is "unproven, first ten minutes" | **Settled from FreeShow source:** actions read fields at the body's top level (`src/frontend/components/actions/api.ts:448-462`, `src/electron/utils/api.ts:141-151`); voxxa's driver nests under `data` (`freeshow.rs:47-49`), so `goto_slide` has always been a silent no-op and FreeShow still answers 204. | 42 (measured), 41, 46, 47 |
| C2 | Index base unknown | **1-based on the wire, 0-based in read-back** (`api.ts:453-454` decrements; `apiGet.ts:37-40` returns the internal index). The kit binding's `{app_slide}` would land one slide early; use a 1-based placeholder. Without `showId`/`layoutId` the jump targets the editor's *active* show, not the output. | 42 (measured) |
| C3 | `id_select_group` may jump to "the chorus" | Its `id` is a **global group key** (`verse`, `chorus`, `bridge`, ...), matched against `slide.globalGroup`, playing the next matching slide after the current one, wrapping. It cannot address verse 3. **`index_select_slide` with explicit show and layout is the only deterministic jump.** | 42 (measured) |
| C4 | "local socket" is a safety property | FreeShow's REST/WS/OSC **bind all interfaces with an optional password** (`api.ts:38-65,121,198`, `cors: *`); the shipped driver sends **no** credential (0 matches for Authorization/bearer). | 42, 43, 46, 47 (measured) |
| C5 | A break-gated jump cost "would have held" live4's three remaining early moves | **Wrong.** All three were onset fires, and an onset requires >= 200 ms below floor by construction, so each followed a break; they were in-plan moves, not jumps. The parts map already removed them (5 -> 1 early move in replay) by waiting for words after a chorus. | 47 (measured), 42, 46 |
| C6 | "The onset-triggered parts map reaches 80% within 300 ms" | 80% within ±300 ms was **T1 with the plan-order follower** plus the onset trigger. The parts map's share within ±1 s is **54-71%** per take (T1 0.60, live4 0.67); median delay T1 0.24 s, live4 0.78 s. The gap is speed at part ends, not recognition. | 47 (measured) |
| C7 | 814 MB is "what Whisper leaves" | 814 MB was the **GPU baseline with Presenter open, before Whisper loaded** (870 MiB today with nothing running). Whisper Small's share on this Vulkan build is unmeasured. The GTX 960 also drives the projector. | 47 (measured) |
| C8 | `boundary = plan` for plan-keeping leaders recovers the post-chorus lag | **Measured worse**, even on T1 where the plan was kept: 9/11 right, 3 wrong, 2 early, 24.2 wrong-slide s vs 11/11, 1, 0, 10.7 default; over 7 takes 68/19/9/223 vs 74/4/1/112. After a chorus the next breath is often a repeat or an ad-lib whatever the plan. The CTO's hope (41 § 3) that practice memory fixes the lag "for free" this way is refused. | 47 (measured) |
| C9 | "live4 could not return to a chorus" shows Presenter's ceiling | live4 ran the **plan-order** follower; the founder sang the planned order; Left was never attempted by design. Presenter's ceiling is real (shortcuts page, binding profile) but live4 is not its evidence. The parts map has never run live. | 47 (measured) |
| C10 | Whisper-timestamp error (0.82 s vs 0.071 s) motivates an aligner | voxxa never uses Whisper timestamps; it times with onsets (ADR-030). The citation motivates nothing voxxa does. | 46, 42 |
| C11 | "Whisper as witness only" | Taken literally, slides on a band mix would stop moving (onsets found 2 of 11 starts on P1). Words stay the fallback mover. | 47 (measured) |
| C12 | "Switch the live output one rehearsed song at a time" | **Rejected.** Two apps cannot own the projector within one service. Switch **per service**, after double-build Sundays, whole service (not only the worship set), with a rollback rule. | 45, 46, 41 |
| C13 | Four consecutive shadow Sundays before autonomy | **Two gates, not one.** ADR-022 § 3's 2 scored shadow Sundays flips songs mode *with a minder at the PC* (unchanged). A new, later gate removes the person: >= 4 consecutive green Sundays, >= 300 scored occurrences, 0 early beyond the line, 0 spurious presses, two volunteers passed the card. | 41, 43, 45 |
| C14 | QLab-style reserved Escape as panic | **Rejected.** ADR-025 § 7 refuses Esc; voxxa is never the front window; Escape's effect in Presenter is undocumented. The pattern that transfers is "unremappable and graded", via F15/F20 and a Companion button. | 44, 43 |
| C15 | Step 6 (minder, health check, kill key) last | **Moved up.** Nothing of it exists in code today (no watchdog, no hold or panic: 43 F1-F2 measured), and the first shadow Sunday with a volunteer needs it. Role card v0 ships in week 1 as docs. | 43, 44, 45 |
| C16 | Step 4 (input chain, vocal-only feed) fourth | **Moved to second.** Three of four live attempts failed on setup; the hardware fixes (FA-38 b, FA-39) cost no code. | 42, 45, 47, 46 |
| C17 | Rank 1 is cheap | Rank 1 has **unstated preconditions**: the dispatch changes (verify hook, `position_uncertain`, song select) live in "everything else", which has **1 line left**; V7 (wiring row, FA-10) must land first. Also: voxxa **cannot change songs on FreeShow today** (`can_query_state: false` -> dispatch blanks and asks the operator, pinned by `tests/dispatch.rs:289-305`). | 42, 46, 47 (measured) |

## 3. Rulings on the contested proposals

- **R1 Decision order vs work order (41).** The report's ranking is the *decision* order. The *work* order is the
  plan's, with a FreeShow track added in parallel: V7 (budget row) and the V3e probe immediately, the driver (V8b)
  after V7/V8-lite, while V3/V4/V3b/V3d continue in the lab. Two decisions are separated: **engineering** (FreeShow
  becomes voxxa's first-class Tier A target: go now) and **church** (switching the Sunday output: staged, decided end
  of January on measured dark-Sunday numbers, FA-48).
- **R2 Staging ("dark FreeShow", 41 § 2.3, merged with 43 § 5).** Stage 0 shadow on Presenter (today). Stage 1
  FreeShow installed with **no output assigned**, voxxa drives it in shadow on weekdays, then during Sundays while
  Presenter stays live; read-back is logged. Stage 2 FreeShow live, a volunteer operates it, voxxa shadow scored
  from read-back (double-build Sundays; Thursday 18:00 rollback rule; Presenter maintained four weeks). Stage 3 one
  promoted song autonomous. Stage 4 all promoted songs, volunteer as minder at the PC. Stage 5 no volunteer in the
  booth; the leader holds a Companion/phone HOLD. Rollback triggers per 43 § 5.
- **R3 Break gate (report step 2).** Not a plan step. A **lab switch inside V3d task 3**: a jump is accepted on one
  agreeing window when its alignment starts within ~1 s after an onset gap >= 300 ms, two windows otherwise (47
  § 3.1); measured before/after on F1-F4, live4 and the real detour takes. Ships in V11 only as
  `PartsConfig.break_gate` default `None` with the before/after table in the record. live4 is never cited for it.
- **R4 Per-leader resume prior.** Folded into V12; applies only after >= 3 confirmed runs for that leader
  (leader alias L1/L2); never wired to `boundary = plan` (C8). Hand-set handicaps until then.
- **R5 Aligner, OLTW, CB-Whisper (report step 3).** **No product code, no Python sidecar (ADR-003), no new runtime
  before V6's held-out numbers.** One cheap, founder-run **lab measurement step V3f** is allowed after V3e's VRAM
  numbers exist: OLTW within a part against a prior take (no model, CPU), and a CRNN/CTC alignment of the second
  after each onset against candidate first lines; report tight_300, boundary delay, early moves, VRAM and ms per
  update on the GTX 960. ADR-033 and an `align` row only on a pass against a bar set in V3f's prompt **and** after
  V6. Model weights must be permissively licensed (FA-50); MMS/wav2vec2 CC-BY-NC weights are lab-only. CPU streaming
  recognisers: experiment only, not in the plan.
- **R6 Input chain.** Hardware first (FA-38 b USB interface with balanced input, FA-39 vocal-only aux), then software [superseded 2026-11-26 by the founder's answers: hardware deferred; software carries a degraded full-mix mode, ADR-030 amendment 1]
  in V3b task 6: 100 Hz high-pass, energy gate on the 300-4000 Hz band OR'd with Silero (config, default off),
  level and hum numbers on the start line and in the health check; `vad/` moves to the recognizer row (ADR-028
  amendment, same PR); windows do **not** follow vocal activity in v1 (replay determinism, 47 § 3.5).
- **R7 V3c (Base + prompt).** Conditional: run only if the booth release build shows Small > 1.0 s/pass. If run:
  a spoken-form tail prompt of the current slide's remaining line (<= 40 tokens), **never the planned next part at a
  part end** (biases against detours), and detour detection on F1-F4 in its acceptance (47 § 3.6).
- **R8 Minder controls.** Two live controls: **HOLD** (F15, toggle; pause following, keep listening and logging) and
  **STOP FOLLOWING** (F20; dispatch off for the rest of the service, listening and logging continue, defined
  post-state: current slide held, nothing sent), plus **Back one** (F14) on the pedal/Companion page; HTTP routes
  `/pause`, `/resume`, `/panic`, `/state` (V13a). Off stays in the mode switch. No Escape. FreeShow's output lock is
  the external panic path; voxxa's API stays on 127.0.0.1. Companion/Stream Deck is a **docs recipe** (colour from
  `/state`), not voxxa code (41 F7 vs 43/44: both satisfied).
- **R9 Stage cue.** voxxa's own stage strip in three states (following / holding / you're driving); the **heard-words
  line is removed** (44 F2 measured: `stage.html:31`, `src/stage.js:52-55`; ADR-011). Ticket filed now, fixed in V14.
  FA-29 (stage output never in the stream) is confirmed before any stage display is turned on. voxxa may not write
  FreeShow stage text (ADR-025 § 7).
- **R10 Badge and demotion.** Badge words "voxxa moves it / voxxa suggests / You move it". Any manual move demotes the
  song to suggest for the rest of the song; on Presenter this is observable only through voxxa's own hotkeys (43 F10),
  one more reason the no-minder gate is FreeShow-only (Tier A).
- **R11 Eval (V2).** Add signed error, `early_300`, `early_1000` (decoupled from the 1 s match window, `score.rs:10`),
  `tight_300`, `wrong_slide_s`, `--service-report` (numbers-only `service-<date>.json` + `trend.csv`) and
  `--truth-from-screen` (FreeShow read-back as operator-timed truth for the **Correct** bar only; it never gates
  Tight because operator presses lag ~0.5 s). The eight baseline names stay. Fit the voxxa-eval row (112 left): move
  `--model` passthrough to V3, truth-v2 event rows to V6, one shared signed-delay helper; stop and report if it does
  not fit. Never raise a cap.
- **R12 Budget rows (ADR-028 amendment in V7).** New: `onset` (250, split from the follower row), `input chain`
  (~200), `health check (src/health/)` (150) [answered 2026-11-27: caps set at 3 × these estimates, 750 / 600 / 450; FA-74 asks whether the rule extends to the earlier rows]. Move: `vad/` -> recognizer row. Conditional: `align` (400) only per R5.
  The port of parts.py (395) + onset (~150 of predict.py) + one-part fallback is estimated 600-900 Rust lines (47 S7):
  V11 ports **one module** `aligner/follower/parts.rs` with a `dr_show` flag for one-part songs (46 lean 15), replacing
  the `aligner/evidence/` accumulator under one config switch rather than stacking a third follower (47 S8).
- **R13 JSONL protocol.** One new `trace` line type with a `kind` field (ADR-001/003 amendment) instead of `follow`,
  `cue` and read-back types (42 F7).
- **R14 Lab must-fixes before V3d's live test (46 § 3; confirmed by 47 against the code on 2026-11-26: a, b, c, e, f
  confirmed, d partly: the unbounded queue is the mic queue, not stdin, and live4 shows no stall). Two effects 46
  missed: a relabelled jump inside a review window also rewinds the plan pointer; and the live4 replay stamps word
  moves with voxxad's audio time while live.py stamps them at read time, so the replay's delay rows for word-driven
  moves are optimistic by about 0.5-1.2 s (onset moves unaffected). Every offline-vs-live comparison carries that
  bias until live.py uses voxxad's `t`.** The fixes: planned-next part entered from
  mid-part on one window then relabelled `jump` after the gates (`parts.py:195,277-279,304-323`); elimination has no
  talk gate (spoken sentence after a chorus -> next verse shown): `elim` behind a lab flag the live test may turn off,
  and a spoken-aside take added to V3d task 4; `live.py` stamps windows at read time not audio time (`:311-312`);
  stdin queue depth in the status line; floor warm-up 1 s in code vs 0.5 s in ADR-030 § 1 (ADR says 1 s); pytest for
  the lab (`test_parts.py`, `test_onset.py`) written from ADR text. Also: two-singer take; OLTW arm; timing columns
  within 0.5/1.0/1.5 s per take.
- **R15 Church-side steps (45).** New V17 (FreeShow migration kit and library re-import, church side) and V18 (minder
  training programme and kit update; absorbs the UI review's "V5d minder kit"). Window: after the Christmas freeze
  (Dec 18 - Jan 3), start Jan 10, 2027, last double-build Sunday before Easter (Mar 28, 2027). Repertoire inventory
  signed by the worship leader before any migration weekend; first ten songs = most-sung licensed songs.
- **R16 V10 mock.** Kept (S), re-scoped: the mock shows the minder states and the health-check page so the founder
  approves every amber/red sentence with the mock in front of him (44); not dropped (46).
- **R17 Drops and deferrals** [FA-72 answered 2026-11-27: drops approved; V16 runs now; the rest deferred as written]**.** Dropped: V6b; the report's Holyrics driver (only on a V3e no-go); Companion as code;
  metric renaming. Deferred behind the first autonomous Sunday: V13b (layout/service-plan/cue engine; V13a hotkeys and
  routes stays), V5b/V5c (kit tools zip, needed for church #2), V16 (rename/icon), V8 steps 2-4 (V8-lite keeps per-key
  position only), any product aligner/tracker.

## 4. Canonical step list for round 3 (names are final; writers use these)

| Step | Title | Size | Status | Depends on |
|---|---|---|---|---|
| V7 | Wiring budget row (ADR-028 amendment: onset, input chain, health rows; vad/ move; trace line) | S | existing, **this week** | FA-10, FA-54, FA-65 |
| V3e | FreeShow probe (founder-run, numbers only) + VRAM/CPU with both apps and a video playing + PD-hymn import counts + Presenter MIDI check + one non-projected live parts-map run | S | **new, weekday before Dec 18** | FA-46, F5 |
| F5 | Founder: FreeShow stable beside Presenter: install, API password, loopback firewall, unique-layout test show, numbers | founder | new | FA-46, FA-51, FA-55 |
| V3 / V4 | Decoder settings + pass-time p95 amber / GPU proof | S+S | existing | — |
| V3b | Onset trigger (task 6 = input chain per R6; task 5 may target non-projected FreeShow after V3e) | M | existing, modified | V3e for task 5 |
| V3d | Parts map (R3 break-gate arm, OLTW arm, R14 lab fixes, detour + spoken-aside + two-singer takes, timing columns, port line counts and row proposal) | M | existing, modified | FA-43, FA-45, FA-63 |
| V3c | Base + tail prompt | S | **conditional** (R7) | booth Small > 1.0 s/pass |
| V5a / V5b | `.txt` importer sections now / kit tools and guards later | S / M | split | — |
| V2 | Eval v2 columns + service report + truth-from-screen (R11) | S/M | existing, modified | — |
| V8-lite | ADR-025 per-key position only | S | reduced | V7 |
| V8b | FreeShow driver v2: top-level body, 1-based index, showId/layoutId from `get_output`, `select_presentation` via `name_select_show`, `current_state` ids only, fire-then-verify (one re-read, latency from V3e), `position_uncertain`, bearer header, refuse open API, never-call list for text getters, `--watch` screen log, TcpListener stub, the named tests listed in 52 (10), ADR-032, kit binding + test card rows | M | **new** | V7, V8-lite, V3e go |
| V3f | Aligner/OLTW lab measurement only (R5) | S/M | **new, lab** | V3e numbers |
| F1 -> V6 | Corpus and held-out gate | founder + M | existing | — |
| V11 | Follower port as one module (R12), Tier A/B jump interface, `break_gate: None`, `trace` line, replay byte-identical | L | existing, modified | V7, V8b, V3d, V6 |
| V13a | Hotkeys F14/F15/F20 + routes `/pause /resume /panic /state` | S | split from V13 | V7 |
| V13s | Sunday readiness: health check page (6 rows), off-thread watchdog with heartbeats (4 signals), hold/panic post-state, service manifest frozen at last rehearsal, per-service numbers-only report, runbooks (sunday-window, watchdog-tripped, panic-pressed), RDP check (`SM_REMOTESESSION`), device pinned by name | M | **new** | V13a, V2 |
| V10 | Run-screen mock incl. minder states and health sentences (R16) | S | existing, modified | — |
| V14 | Run screen and health check (minder UI): badge, NOW/NEXT firm/provisional, amber hold sentence, HOLD/Back one/STOP, stage strip (heard words removed), demote rule, a11y contrast fixes | M | existing, re-scoped | V10, V13a, V13s |
| V12 | Practice memory v1: data model (parts order, detours, boundary lag, leader alias), `status` {not_rehearsed, proposed, promoted, demoted}, `--learn` = parts map run non-causally, `--review` keyboard flow, `--confirm`, >= 3 runs before learned handicaps; **no neural aligner** | M | existing, modified | V11 |
| V17 | FreeShow migration kit and library re-import, church side | M | **new** | V3e go, FA-48, FA-68 |
| V18 | Minder training programme and kit (role card v0 Presenter / v1 FreeShow, ten-minute session, monthly drill with fault cards, rota, worship-leader card, congregation notice, prompts 05/06/07 edits) | M | **new**; card v0 as docs in week 1 | V10 words |
| V15 | Close: read-back as shadow truth, runbook set, acceptance in occurrences, minder card passed by two volunteers, founder <= 45 min per scored Sunday | S/M | existing, modified | all above |
| F6 | Founder: monthly fault drill + one manual song a month | founder | new | V18 |
| F7 | Founder: dark Sundays (Jan 10, 17, 24), 15 min each; FA-48 decision end of January | founder | new | F5, V8b |
| Q11 | unchanged, on Presenter | — | existing | — |
| Q12 / Q13 | absorb FreeShow `get_output` ids-only read-back / V8b stub as the FreeShow fixture | — | existing, modified | V8b |

Dropped: V6b. Deferred: V13b (layout engine), V5c, V16, V8 steps 2-4. Conditional: V3c.

ADR numbering for this round (fixed after the writers' pass): **ADR-032** FreeShow driver and the Tier A/B policy (V8b);
**ADR-033** reserved for an aligner row, written only after V3f and V6 both pass (R5); **ADR-034** Sunday readiness
(V13s); **ADR-035** live-output migration (V17; Proposed until FA-48 is decided). Amendments: 001/003 (`trace` line),
005 (model weights), 022 (two gates), 023 (practice memory data model and status), 024, 025 (hold/stop semantics),
028 (rows), 030 (floor warm-up 1 s), 031 (R3 break-gate option).

Naming rulings after the integration pass (2026-11-26, afternoon): one test card, `kit/templates/presenter-test-card.md`,
with the FreeShow rows appended as FS1-FS20 (no separate FreeShow card); the 1-based binding placeholder is
`{app_slide1}`; the API-posture setting is `require_password` (default true; voxxa refuses an API that answers
without a password unless it is set to false); V8b's named tests are the ten listed in 52. Two open questions settled:
(a) getters whose response may carry slide text (`get_show`, `get_shows`) may be called only through a parser that
keeps id and name and discards the body before anything is logged or stored, with a test asserting the raw body is
never written; `get_output_slide_text` and `get_output_group_name` stay on the never-call list; the V3e probe records
which getters carry text. (b) On a dark Sunday Presenter has no read-back, so FA-48's 99% line measures the
**driver**: FreeShow's read-back against voxxa's own commanded slide over all polls, with 0 `position_uncertain`.
The follower's correctness against the singer is scored as in stage 0 (shadow plus tapping) until stage 2, where the
volunteer's FreeShow moves become operator-timed truth (FA-66).

## 5. Founder decisions (canonical numbering; FA-01..FA-45 exist in out/founder-action-items.md)

Already open and now urgent: **FA-10** (wiring row) -> approved 2026-11-26, option (a); **FA-44** -> answered as (d) FreeShow trial
via V3e and (e) suggest-only on Presenter until then; **FA-45** -> tap live4's truth (5 min) before ADR-031 cites it;
**FA-38 (b), FA-39** -> answered 2026-11-26 as deferred and not required (a deviation from R6: the full mix on the current unbalanced, low-level feed is a supported, degraded condition); **FA-54, FA-65** -> answered 2026-11-27 (FA-54 yes with the rows at 3 × the estimates, a deviation upward from R12 that opened FA-74; FA-65 yes, option (a)); **FA-29** -> answered 2026-11-27: yes, all Presenter output is streamed as a YouTube lower third via Blackmagic (FA-75 opened); **FA-50, FA-52, FA-55 … FA-61** -> answered 2026-11-27 (deviations: FA-50 no licence gate now, FA-57 keep the 500 ms fade, FA-59 deferred, FA-61 re-anchor instead of demote); **FA-62 … FA-64** -> asked again in plain words; the original ask was: confirm the
stage output is never captured into the stream before any stage display is used; **FA-43** -> approve ADR-031 as
recommended, with the R3 amendment.

| FA | Decision | Recommendation |
|---|---|---|
| FA-46 | Approve the V3e spike and installing FreeShow **stable** beside Presenter, weekday before Dec 18; Presenter stays live | yes |
| FA-47 | Two-tier app policy: Tier A (jump + read-back) is autonomy-eligible; Tier B (keys-only) is assisted with suggestion cards; Left bursts refused in code | yes |
| FA-48 | Gate and shape for switching the Sunday output: V3e go; >= 2 dark Sundays with read-back agreeing with voxxa's commanded slide on >= 99% of polls and 0 `position_uncertain`; library re-imported, songcheck green, groups match labels; worship leader checked the stage display; one volunteer trained on FreeShow keys; **per-service switch, whole service, 4 double-build Sundays, Thursday 18:00 rollback rule, Presenter kept a month**; window Jan 10 - Mar 21, 2027; decide end of January | adopt |
| FA-49 | Budget gate: V3e now (<= $20); V8b (<= $60) on a V3e go; nothing for an aligner, tracker or new model before V6 reports and the GPU is measured with both apps open | yes |
| FA-50 | Model and aligner licences: only permissive weights (MIT, Apache, BSD, CC-BY, CC0) in the product or kit tools; CC-BY-NC lab-only; amend ADR-005's dependency rule to cover model weights | yes · **answered 2026-11-27: use whatever licensing is required for now; own MIT-like ministry licence later (FA-76); ADR-005 amendment = inventory, not a gate** |
| FA-51 | Pin FreeShow: stable channel, auto-update off, version in `church-profile.json`, checked by the health check; upgrades on a weekday | yes · **answered yes, 2026-11-27** |
| FA-52 | Two gates: 2 scored shadow Sundays -> songs mode with a minder at the PC (ADR-022 unchanged); no-minder gate = 4 consecutive green Sundays, >= 300 scored occurrences, 0 early beyond the line, 0 spurious presses, two volunteers passed the card; early line 0.3 s on rehearsal truth, 1.0 s live; tighten only | yes · **answered yes, 2026-11-27: extensive listening period; never interfere with the volunteer** |
| FA-53 | Presenter MIDI mapping check (20 min, inside V3e): does any command name a slide or section? A "no" closes the "wait for WorshipTools" option | yes · **answered 2026-11-27: skip it** |
| FA-54 | New ADR-028 rows: onset 250 (split from follower), input chain ~200, health check 150; `vad/` -> recognizer row; `align` 400 only per R5 | yes · **answered 2026-11-27: yes, caps × 3 (750 / 600 / 450)** |
| FA-55 | FreeShow API posture: password set, 5505/5506 firewalled to loopback, voxxa refuses an API that answers without a password unless `require_password` is set to false, bearer header only, token never logged or bundled | (a) refuse by default · **answered yes, 2026-11-27; steps in `out/founder-freeshow-setup.md`** |
| FA-56 | Unique layouts (every slide once) for followed songs in v1; a per-song occurrence resolver later with Q12 | yes · **answered 2026-11-27: the per-song JSON layout file is the object of record (ADR-032 Decision 13 as amended); unique layouts stay the v1 rule** |
| FA-57 | Text transition 0 ms for the followed output style (FreeShow default is a 500 ms fade) | measure in F5, then yes · **answered 2026-11-27: keep the 500 ms default (reading time)** |
| FA-58 | Controls: F15 HOLD (toggle), F20 STOP FOLLOWING (no confirm; dispatch off, listening continues), F14 Back one; routes `/pause /resume /panic /state`; FreeShow output lock as external panic; voxxa API stays 127.0.0.1; no Escape; Off only in the mode switch | yes · **answered yes, 2026-11-27, on condition of no hotkey collisions (FS14 check)** |
| FA-59 | Alarm/veto surface when nobody is at the PC: a Companion/Stream Deck page (or a phone) at the stage, colour from `/state`, calling `/pause`; docs recipe only; hardware optional ($0-80 est.) | (a) · **answered 2026-11-27: not now (deferred)** |
| FA-60 | Stage cue: voxxa's own three-state strip; heard-words line removed (ticket now, fix in V14) | yes · **answered yes, 2026-11-27** |
| FA-61 | Badge words "voxxa moves it / voxxa suggests / You move it"; any manual move demotes the song for the rest of the song | yes · **answered 2026-11-27: no demotion; voxxa re-anchors and keeps following; the badge shows live mode** |
| FA-62 | Health-check sentences approved word for word on the V10 mock; "Ready, with a note" may start Songs; only a red slides-app row blocks | yes · **answered 2026-11-27: approve when I see the mock (V10)** |
| FA-63 | Break gate: lab switch in V3d, `PartsConfig.break_gate` default `None` in V11, before/after measured; live4 is not its evidence | yes · **answered 2026-11-27: build it on by default (deviation; V3d still measures before/after)** |
| FA-64 | Per-leader handicaps only after >= 3 confirmed runs; leader alias L1/L2 in practice files; never `boundary = plan` | yes · **answered yes, 2026-11-27** |
| FA-65 | JSONL: one `trace` line type with `kind` (ADR-001/003 amendment) | (a) · **answered yes, 2026-11-27** |
| FA-66 | Read-back as shadow truth (operator-timed) for the Correct bar; tapping one Sunday in four for Tight | yes · **answered yes, 2026-11-27** |
| FA-67 | Booth PC policy on service days: no RDP Sat 18:00 - Sun 14:00; Windows Update active hours cover services, auto-restart off; voxxa started and armed by a person, never auto-start; health check within 30 min of the service | yes · **answered 2026-11-27: yes, and voxxa may auto-start (Songs still armed by a person)** |
| FA-68 | Repertoire sign-off: the worship leader signs the inventory counts (licensed source / none / legacy CCLI) before any migration weekend; first ten re-imported songs = most-sung licensed songs | yes · **answered 2026-11-27: later (end of January)** |
| FA-69 | Who minds: >= 3 minders from the existing slide team; the founder is never the Sunday minder from month two; a veteran (3 months) trains the next | yes · **answered 2026-11-27: fewer minders is fine** |
| FA-70 | Words: no "AI" from the stage or in the bulletin; one-paragraph notice; the pastor decides wording | yes · **answered 2026-11-27: no notice at all** |
| FA-71 | Scoring labour: talk-span notes live, talk muted, a consented volunteer taps songs only, partial scoring counts; founder <= 45 min per scored Sunday, <= 2 h/week total | yes · **answered 2026-11-27: 90 min per scored Sunday, 4 h per week** |
| FA-72 | Approve the drops and deferrals in R17 | yes · **answered 2026-11-27 per line: all drops approved (Holyrics only ever as an ADR-036 bindings file, on need); deferrals as recommended except V16, which runs now** |

## 6. Process notes for the writers

- Prompts follow `.10x/specs/2026-11-11-prompts-10x-team.md` exactly (title, size, phase and roles, task list, budget
  shares, founder decisions needed, pre-approvals, safety lines, state files, exit criteria), two lanes (10x + ponytail)
  and a comparator for build steps; ponytail + ponytail-audit for close and discovery steps; adversarial verifiers.
- Safety lines on every step: never open the mic unattended; never press keys in Presenter; never start the installed
  app; never read or print lyrics or heard words; never touch private folders; stop own processes by PID; media guard
  stays required; numbers only in anything that leaves the PC.
- Costs in plan.md § 2 units (S $3-8, M $10-25, L $25-50; ponytail x2; verifiers +20%). CTO's added-cost estimate for
  the FreeShow track: ~$37-95 agent cost and 6-8 founder hours; the plan total moves from ~$315-780 to ~$350-875.
- Calendar bounds: Christmas booth freeze Dec 18 - Jan 3; dark Sundays Jan 10/17/24; FA-48 decision end of January;
  migration window to Mar 21; Easter Mar 28, 2027.
