# voxxa: strategy decision and reviewed build plan for autonomous slide following (2026-11-26)

Writer's note. This file turns the research report (`reports/Voxxa autonomous slide following research.md`), the
seven round-2 reviews (`round2/41..47`) and the adjudication (`round2/50-adjudication.md`) into one founder-facing
decision and one build plan. The adjudication's rulings, step names and FA numbers are used verbatim. Costs are in
`out/plan.md` section 2 units (S $3-8, M $10-25, L $25-50 one lane; a ponytail lane roughly doubles; verifiers +20%).
Estimates are marked [est.]; anything added beyond the sources is marked (inferred). Numbers only: the private song
is P1, the pilot church is "the church", singers are aliases (L1, L2). No lyrics, no heard words.

---

## Part 1. Decision memo

### The decision

**Go on FreeShow as voxxa's first-class live-output target, now, as engineering work. Do not switch the church's
Sunday output yet.** The church switch is a separate decision (FA-48), taken at the end of January 2027 on measured
numbers from three dark Sundays. If taken, it happens per service, whole service, over four double-build Sundays
between Jan 31 and Mar 21, 2027, with Presenter kept for a month.

Everything else in the report stands in direction and is corrected in detail. The parts map (ADR-031) is the
follower. Learning happens offline, in rehearsal review, never on Sunday. The volunteer becomes a minder with a
health check, HOLD, STOP FOLLOWING and a one-page card. Recognition is not the ceiling today; the output path is.

### How voxxa gets there

**Autonomous slide changes.** voxxa already decides the right slide most of the time on the lab takes (74 of 77
changes right over seven takes, measured). It cannot show that decision, because Presenter accepts only Right and
Left and never says where it is. FreeShow has a documented jump (`index_select_slide` with show, layout and index)
and a read-back (`get_output`). The shipped FreeShow driver has never worked (C1: fields nested under `data`, so
the index never arrives). The fix is a small driver rewrite (V8b) after one founder-run probe (V3e) and one budget
step (V7). Every jump becomes fire-then-verify. A song that has passed its rehearsal gate runs autonomously; the rest
run as suggestions. On Presenter (Tier B) voxxa only steps forward in plan and suggests the rest.

**Self-training from practice sessions.** `voxxad --learn` becomes the parts map run non-causally over a rehearsal
log (V12). It yields the order sung, dwell, gaps and detour counts, which is all ADR-023 stores. The founder, or the
minder on rota, confirms per song with one key (`--review`). No neural aligner is needed for this. None is built
before V3f's lab numbers and V6's held-out numbers exist and the GPU is measured with both apps open (R5).

**A trained team, nobody at the slide PC every week.** Two gates, not one (C13, FA-52). After two scored shadow
Sundays, voxxa moves slides with a minder at the PC. After four consecutive green Sundays, at least 300 scored
occurrences, zero early moves beyond the line, zero spurious presses and two volunteers passing the card, the
minder leaves the PC and the worship leader holds a HOLD button at the stage. The role card v0 ships as docs in
week 1 (C15); the training programme and kit are V18. Nothing in the safety machinery exists in code today (no
watchdog, no hold, no panic), so V13s builds it before any song runs autonomously.

### Whether and how to move to FreeShow

Two decisions, separated (R1):

1. **Engineering: go now.** FreeShow becomes voxxa's Tier A target (jump plus read-back; autonomy-eligible).
   Presenter stays the Tier B universal fallback (keys only; assisted with suggestion cards; Left bursts refused in
   code). Cost: V3e (S, <= $20) now; V8b (M, <= $60) on a V3e go (FA-49).
2. **Church: staged, decided end of January (FA-48).** Stages 0 to 5 (Part 2.6). FreeShow is installed beside
   Presenter with no output assigned (stage 1), driven by voxxa in shadow on weekdays, then on three dark Sundays
   while Presenter stays live. If read-back agrees with the operator's slide on >= 99% of polls with 0
   `position_uncertain`, the library is re-imported with `songcheck` green, the worship leader has checked the stage
   display and one volunteer is trained on FreeShow's keys, the Sunday output switches: per service, whole service
   (C12), four double-build Sundays, a Thursday 18:00 rollback rule, Presenter kept a month.

Why not "one rehearsed song at a time": two apps cannot own the projector in one service (C12). Why not wait for
WorshipTools: the API request thread has been open since 2024-07 with no staff reply; FA-53 spends 20 minutes on the
MIDI dialog as Presenter's last chance. Why not Holyrics: only on a V3e no-go (R17).

### What the research settled

- The output path, not recognition, is the first blocker (all seven reviewers agree).
- FreeShow is the right long-term target: GPL, Windows, documented action API, slide-index selection, `get_output`.
- The parts map is the shape the score-following literature converged on; learning offline is the state of the art.
- The minder model (hold on uncertainty, graded controls, a card, a monthly drill) is right.
- The metric mapping to mir_eval and Matchmaker conventions is accepted as documentation; no column is renamed.

### What the review corrected (C1..C17, short form)

| # | Report said | Ruling |
|---|---|---|
| C1 | W1 (body nesting) is unproven | Settled from source: fields are top-level; the driver's `goto_slide` has never worked |
| C2 | Index base unknown | 1-based on the wire, 0-based in read-back; always send `showId` and `layoutId` |
| C3 | `id_select_group` can jump to the chorus | It is a global group key, "next match, wrapping"; `index_select_slide` is the only deterministic jump |
| C4 | Local socket is a safety property | FreeShow binds all interfaces with an optional password; the driver sends no credential |
| C5 | A break gate would have held live4's early moves | Wrong: all three were onset fires after a break; the parts map already removed them |
| C6 | Parts map reaches 80% within 300 ms | That was the plan-order follower on T1; the parts map is 54-71% within 1 s per take; the gap is speed at part ends |
| C7 | 814 MB is what Whisper leaves | 814 MB was the GPU baseline before Whisper loaded; Whisper's share is unmeasured; the GPU drives the projector |
| C8 | `boundary = plan` fixes the post-chorus lag | Measured worse (68/19/9/223 vs 74/4/1/112 over 7 takes); refused |
| C9 | live4 shows Presenter's ceiling | live4 ran the plan-order follower in planned order; the parts map has never run live |
| C10 | Whisper-timestamp error motivates an aligner | voxxa never uses Whisper timestamps (ADR-030) |
| C11 | Whisper as witness only | Words stay the fallback mover (onsets found 2 of 11 starts on P1) |
| C12 | Switch one rehearsed song at a time | Rejected: per service, whole service, double-build Sundays, rollback rule |
| C13 | Four shadow Sundays before autonomy | Two gates: 2 scored shadow Sundays (minder at PC); a later no-minder gate |
| C14 | QLab-style Escape as panic | Rejected (ADR-025 section 7); F15 HOLD, F20 STOP FOLLOWING, Companion button |
| C15 | Minder, health check, kill key last | Moved up: none of it exists in code; card v0 in week 1 |
| C16 | Input chain fourth | Moved to second; hardware first (FA-38 b, FA-39) |
| C17 | Rank 1 is cheap | Preconditions: V7 first ("everything else" has 1 line left); voxxa cannot change songs on FreeShow today |

### What is deliberately not being built now (R17, FA-72)

Dropped: V6b; the Holyrics driver (only on a V3e no-go); Companion as voxxa code (a docs recipe instead); metric
renaming. Deferred behind the first autonomous Sunday: V13b (layout, service plan, cue engine), V5b/V5c (kit tools
zip, church #2), V16 (rename and icon), V8 steps 2-4, any product aligner or reference tracker, the CPU streaming
recogniser (experiment only). Refused outright: wiring a per-leader prior to `boundary = plan` (C8); a Python
sidecar in the product (ADR-003); CC-BY-NC model weights anywhere but the lab (FA-50); Escape as a voxxa key (C14).

---

## Part 2. The build plan

### 2.1 Dependency graph

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

### 2.2 Ordered step list (canonical names from adjudication section 4)

Lanes: "2 lanes" = 10x + ponytail + comparator + two verifiers (build steps); "10x + audit" = 10x + ponytail-audit
(measurement and close steps). Agent cost: one lane / with ponytail and verifiers [est.]. Founder time [est.].

| # | Step | Size | Lanes | Agent cost [est.] | Founder [est.] | Depends on | Exit (short) |
|---|---|---|---|---|---|---|---|
| 1 | **V7** Wiring budget row: ADR-028 amendment (onset 750, input chain 600, health 450: the estimates × 3, FA-54 answered 2026-11-27; `vad/` -> recognizer; `align` 1,200 conditional), `trace` line (ADR-001/003, FA-65 answered 2026-11-27) | S | 2 lanes | $3-8 / $7-19 | 10 min (approve) | FA-10, FA-54, FA-65 | budgets green; 150 free in wiring; rows recorded |
| 2 | **F5** Founder: FreeShow stable beside Presenter: install, API password, loopback firewall, unique-layout test show, numbers | founder | - | $0 | 1 h | FA-46, FA-51, FA-55 | version, VRAM with both apps, pasted |
| 3 | **V3e** FreeShow probe (founder-run, numbers only) + VRAM/CPU with both apps and a video playing + PD-hymn import counts + Presenter MIDI check (FA-53) + one non-projected live parts-map run | S | 10x + audit | $7-19 | 2 h | F5 | go / no-go / Holyrics-fallback table with numbers; FA-48 sheet filled |
| 4 | **V3 / V4** Decoder settings + pass-time p95 amber / GPU proof | S+S | 2 lanes | $6-16 / $14-38 | 1 h booth run | V0 | Small <= 1.0 s/pass on the booth release build; p95 on every session-end line |
| 5 | **V5a** `.txt` importer sections now | S | 2 lanes | $3-8 / $7-19 | 0 | - | three named tests; FreeShow groups get labels |
| 6 | **V2** Eval v2: signed error, `early_300`, `early_1000`, `tight_300`, `wrong_slide_s`, `--service-report`, `--truth-from-screen` (R11) | S/M | 2 lanes | $3-25 / $7-55 | 0 | - | eight baseline names unchanged; fits 112 lines or stops and reports |
| 7 | **V3b** Onset trigger (task 6 = input chain per R6; task 5 may target non-projected FreeShow after V3e) | M | 2 lanes | $10-25 / $24-60 | 2 x 30 min | V3; V3e for task 5 | replay byte-identical with the gate off; level and hum on the start line |
| 8 | **V3d** Parts map: R3 break-gate arm, OLTW arm, R14 lab fixes, detour + spoken-aside + two-singer takes, timing columns, port line counts and row proposal | M | 2 lanes | $10-25 / $24-60 | 3 x 45 min takes | FA-43, FA-45, FA-63 | per-take table incl. within 0.5/1.0/1.5 s; pytest for the lab; row proposal |
| 9 | **V3c** Base + tail prompt (conditional, R7) | S | 2 lanes | $0, or $3-8 / $7-19 | 30 min | booth Small > 1.0 s/pass | never prompts the planned next part; detour detection on F1-F4 |
| 10 | **V8-lite** ADR-025 per-key position only | S | 2 lanes | $3-8 / $7-19 | 0 | V7 | `position_uncertain` exists |
| 11 | **V8b** FreeShow driver v2: top-level body, 1-based index, showId/layoutId from `get_output`, `select_presentation` via `name_select_show`, `current_state` ids only, fire-then-verify (one re-read, latency from V3e), `position_uncertain`, bearer header, refuse open API, never-call list for text getters, `--watch` screen log, TcpListener stub, the named tests listed in 52 (10), ADR-032, kit binding + test card rows | M | 2 lanes | $10-25 / $24-60 | 30 min contract check | V7, V8-lite, V3e go | 8 tests on 3 OSes; replay byte-identical; a chorus repeated by one call on dark FreeShow, confirmed by `get_output` |
| 12 | **V3f** Aligner/OLTW lab measurement only (R5) | S/M | 10x + audit | $3-25 / $7-55 | 2 h | V3e numbers | `tight_300`, boundary delay, early, VRAM, ms/update on the GTX 960; verdict against the bar in its prompt |
| 13 | **F1 -> V6** Corpus and held-out gate | founder + M | 2 lanes | $10-25 / $24-60 | 3 Sundays recording; <= 45 min tapping each | V2, F1 | held-out numbers on the live configuration |
| 14 | **V13a** Hotkeys F14/F15/F20 + routes `/pause /resume /panic /state` | S | 2 lanes | $3-8 / $7-19 | 0 | V7 | `panic_stops_dispatch_keeps_listening`; `--shadow` sends nothing |
| 15 | **V10** Run-screen mock incl. minder states and health sentences (R16) | S | 2 lanes | $3-8 / $7-19 | 45 min walkthrough | - | every amber/red sentence approved word for word (FA-62) |
| 16 | **V11** Follower port as one module (R12), Tier A/B jump interface, `break_gate: None`, `trace` line, replay byte-identical | L | 2 lanes | $37-75 / $70-150 | 1 h equivalence checks | V7, V8b, V3d, V6 | identical goto lists vs the prototype on T1, T2, F1-F4, live4; break-gate before/after table; default `None` |
| 17 | **V13s** Sunday readiness: health check page (6 rows), off-thread watchdog with heartbeats (4 signals), hold/panic post-state, service manifest frozen at last rehearsal, per-service numbers-only report, runbooks (sunday-window, watchdog-tripped, panic-pressed), RDP check (`SM_REMOTESESSION`), device pinned by name | M | 2 lanes | $10-25 / $24-60 | 30 min | V13a, V2 | named watchdog tests; `--check-ready` prints every item with a number; panic tested with fake drivers |
| 18 | **V14** Run screen and health check (minder UI): badge, NOW/NEXT firm/provisional, amber hold sentence, HOLD/Back one/STOP, stage strip (heard words removed), demote rule, a11y contrast fixes | M | 2 lanes | $10-25 / $24-60 | 30 min | V10, V13a, V13s | sentences match the approved list; zero CSP violations; Run <= 350, health <= 150 |
| 19 | **V12** Practice memory v1: data model (parts order, detours, boundary lag, leader alias), `status` {not_rehearsed, proposed, promoted, demoted}, `--learn` = parts map run non-causally, `--review` keyboard flow, `--confirm`, >= 3 runs before learned handicaps; no neural aligner | M | 2 lanes | $10-25 / $24-60 | 5 rehearsal pairs reviewed | V11 | `replay_with_practice_is_byte_identical`; a demotion survives restart |
| 20 | **V17** FreeShow migration kit and library re-import, church side | M | 2 lanes | $10-25 / $24-60 | 3-4 h over 6 weeks (inferred) | V3e go, FA-48, FA-68 | kit files pass `kit_check`; inventory counts signed; first ten songs re-imported with counts equal to `songs.csv` |
| 21 | **V18** Minder training programme and kit (role card v0 Presenter / v1 FreeShow, ten-minute session, monthly drill with fault cards, rota, worship-leader card, congregation notice, prompts 05/06/07 edits) | M | 2 lanes | $10-25 / $24-60 | 2 x 15 min sessions | V10 words | two volunteers pass the v0 card on a public-fixture drill; rota has 3 names for 8 Sundays |
| 22 | **V15** Close: read-back as shadow truth, runbook set, acceptance in occurrences, minder card passed by two volunteers, founder <= 45 min per scored Sunday | S/M | 10x + audit | $3-25 / $7-55 | 45 min per scored Sunday | all above | fake-church dry run of the check from the kit alone |
| 23 | **F6** Founder: monthly fault drill + one manual song a month | founder | - | $0 | 10 min/month | V18 | counts recorded |
| 24 | **F7** Founder: dark Sundays (Jan 10/17/24), 15 min each; FA-48 decision end of January | founder | - | $0 | 3 x 15 min | F5, V8b | read-back agreement and `position_uncertain` counts per Sunday |
| - | **Q11** unchanged, on Presenter; **Q12 / Q13** absorb FreeShow `get_output` ids-only read-back / the V8b stub as the FreeShow fixture | - | as pack | as pack | as pack | V8b | - |

Also carried from the plan unchanged: V0 (merged), V1 (uv), V9 (log privacy), all S.

### 2.3 Calendar, Nov 27, 2026 to Easter, Mar 28, 2027 (Sundays are the clock)

Bounds: Christmas booth freeze Dec 18 to Jan 3 (no release or dev build on the booth PC, FA-28; agent work that
needs no booth continues); dark Sundays labelled Jan 10/17/24; FA-48 decision end of January; migration window to
Mar 21; Easter Mar 28, 2027. Calendar note: the reviews first wrote Jan 11, 18 and 25, which are Mondays in 2027; the Sundays are
Jan 10, 17 and 24, and every date below uses the Sundays.

| Week | Sunday | Agent steps | Founder and church |
|---|---|---|---|
| Nov 27 - Dec 3 | Nov 29 | V7; V3, V4; V5a; V2; V10; V18 card v0 as docs (C15); ADR-032 drafted Proposed | Decision batch "this week" (Part 3); FA-38/FA-39 answered: no hardware purchase, the current feed is the supported degraded condition; tap live4 (FA-45, 5 min); Nov 29 rehearsal recording (F1) if consent is in |
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

Slip rule (45 section 8, risk 5): a slip of three weeks pushes the migration past Easter into the summer. If FA-48
is not a go by Jan 31, the switch is re-decided after Easter and the Tier B path (suggest-only on Presenter, minder
at the PC) runs through Lent.

### 2.4 Budget-row changes (ADR-028 amendment, in V7; FA-10, FA-54)

**FA-54 answered 2026-11-27** with the rule "take the number you need and multiply times 3": the three new rows below carry the estimate and the tripled cap; whether the rule extends to the follower, practice, layout, Run and kit-tools rows is FA-74 (`out/founder-action-items.md`).

| Row | Change | Cap | Why |
|---|---|---|---|
| wiring (`dispatch.rs` changes, hotkeys, routes) | new recorded row (FA-10) | 150 | "everything else" has 1 line left; V8b's dispatch changes, V13a and V3b task 6 all land here |
| `onset` (`src-tauri/src/aligner/onset/`) | new, split from the follower row | 750 (estimate 250 × 3) | the detector and arm/fire rules are signal processing, not following; the port of parts + onset is 600-900 Rust lines [est., 47 S7] |
| `input chain` | new | 600 (estimate ~200 × 3) | high-pass, energy gate, level and hum numbers |
| `health check (src/health/)` | new | 450 (estimate 150 × 3) | a new module gets a new row; Run 350 is tight |
| `vad/` | move -> recognizer row | recognizer 700 unchanged | frees "everything else"; same PR as the energy gate |
| `align` | conditional only per R5 | 1,200 (estimate 400 × 3, if earned) | only after V3f passes its bar and V6 reports |
| follower | unchanged | 500 | V11 ports one module `aligner/follower/parts.rs` with `dr_show` for one-part songs, replacing the `aligner/evidence/` accumulator under one config switch |
| presenters | unchanged | 2,000 (per file 400) | V8b fits (1,374 used; `freeshow.rs` small) |
| voxxa-eval | unchanged | 600 (112 left) | R11: `--model` passthrough -> V3, truth-v2 rows -> V6, one shared signed-delay helper; stop and report if it does not fit |
| V13s watchdog and manifest | wiring row, or a `readiness` row proposed at V13s's plan (inferred; the adjudication records only the rows above) | 900 if approved (estimate 300 × 3) | FA-54's rule: a new module gets a new recorded row at 3 × its estimate; never raise a cap |

### 2.5 ADRs to write or amend

| ADR | Action | Content | When |
|---|---|---|---|
| ADR-032 | **new** | FreeShow jump driver (facts from 42 section 1: top-level body, 1-based wire / 0-based read-back, `showId`/`layoutId` required, `index_select_slide` the only deterministic jump, no error channel so read-back is mandatory, optional password on all interfaces) and the Tier policy (FA-47: Tier A jump + read-back is autonomy-eligible; Tier B keys-only is assisted; Left bursts refused in code); open-API refusal (FA-55); unique layouts in v1 (FA-56); pin rule (FA-51) | Proposed this week; Accepted in V8b |
| ADR-033 | **new, conditional** | aligner / OLTW in the product, `align` row 400 | only after V3f passes its bar **and** V6 reports; permissive weights only (FA-50) |
| ADR-001/003 | amend | one `trace` line type with `kind` (follow, cue, verify) instead of three new types; replay-skipped like `window` (FA-65, answered 2026-11-27) | V7 |
| ADR-005 | amend | the dependency licence rule covers model weights: MIT, Apache, BSD, CC-BY, CC0 only; CC-BY-NC lab-only (FA-50) | before V3f |
| ADR-022 | amend | section 3 unchanged (2 scored shadow Sundays -> songs mode with a minder); add the no-minder gate (FA-52) and the four watchdog signals off-thread (V13s) | V13s |
| ADR-023 | amend | section 4 data model: `parts_order`, `detours`, `boundary_lag_ms`, `leader` alias, `status` {not_rehearsed, proposed, promoted, demoted}, `demote_reason`; handicaps per leader only after >= 3 confirmed runs (FA-64) | V12 |
| ADR-024 | amend | one more typed placeholder `{item.project_item_index1}`; song select is a driver method; hotkeys F14/F15/F20 and routes `/pause /resume /panic /state`; badge words (FA-61) | V8b, V13a |
| ADR-025 | amend | fire-then-verify replaces per-key counting for Tier A; the front-window guard does not apply to HTTP moves; no Escape (confirmed); voxxa's API stays 127.0.0.1; FreeShow's output lock is the external panic path (FA-58) | V8b, V13a |
| ADR-028 | amend | the rows in 2.4 at 3 × the estimates (FA-54, answered 2026-11-27; amendment 1 drafted in `out/adrs/028`) | V7 |
| ADR-030 | amend | the detector's break event (`gap_ms >= 200`) is exposed to the follower as an input; floor warm-up is 1 s (R14) | V3b |
| ADR-031 | amend | section 3 gains `break_gate` as an option, default `None`, before/after measured, live4 never cited (FA-63); section 7 names FreeShow `index_select_slide` as the jump and keeps Right-only on Presenter; R14 lab must-fixes recorded | V3d, V11 |

### 2.6 Rollout stages 0 to 5 (R2 merged with 43 section 5)

| Stage | What runs | Entry criteria | Rollback trigger -> action |
|---|---|---|---|
| 0 Shadow on Presenter (today) | voxxa `--shadow`; scoring by tapping; Q11 volunteer test on Presenter | consent for service audio | - |
| 1 FreeShow installed, **no output assigned**; Presenter live | voxxa drives FreeShow in shadow on weekdays, then on the three dark Sundays; read-back logged; a volunteer runs Presenter | V3e go; F5 done (password set, loopback firewall, auto-update off, stable version pinned); VRAM measured with both apps open and a video playing | FreeShow crashes Presenter, or VRAM > 1,800 MB, or projector stutter -> close FreeShow for that service |
| 2 FreeShow live, a volunteer operates it, voxxa shadow scored from read-back (double-build Sundays) | the whole service built in both apps; FreeShow on the projector; Presenter on cold standby with the same service; `get_output` polled at 2 Hz as operator-timed truth | FA-48 adopted: >= 2 dark Sundays with read-back agreeing with voxxa's commanded slide on >= 99% of polls and 0 `position_uncertain`; library re-imported, `songcheck` green, groups match labels; worship leader checked the stage display; one volunteer trained on FreeShow keys; repertoire counts signed (FA-68); announcements, scripture and media rebuilt | a FreeShow-caused visible failure (black projector, wrong song, lost stream) -> rollback declared by Thursday 18:00, Presenter next Sunday; Presenter maintained four weeks |
| 3 One promoted song autonomous; volunteer moves the rest and holds HOLD/STOP | voxxa moves that song; badge "voxxa moves it" on it, "voxxa suggests" or "You move it" elsewhere | gate 1 passed (2 scored shadow Sundays at stage 2); that song's promotion pass (2.7); V13s health check, watchdog and panic merged; V14 merged; card v0 passed by the minder on duty | S1 or S2 breach on that song -> automatic demotion to suggest; two breaches -> back to stage 2 for 2 services |
| 4 All promoted songs autonomous; volunteer as minder at the PC | minder runs the health check and holds the keys; unpromoted songs stay suggest | 2 services at stage 3 clean on the autonomous song; card passed by 2 volunteers (V18) | any panic -> stage 3 next service; any song's S1/S2 breach -> that song demoted |
| 5 No volunteer in the booth; the leader holds a Companion/phone HOLD | health check run by a person within 30 min of the service; Companion page colour from `/state` | gate 2 passed (FA-52): 4 consecutive green Sundays, >= 300 scored occurrences, 0 early beyond 1.0 s, 0 spurious presses, two volunteers passed the card, one fault drill run | any S1, S2 or S5 breach -> a volunteer returns to the PC for 2 services |

### 2.7 Acceptance gates (FA-52) and SLOs (43 section 2), stated so a tool can compute them

Inputs: the numbers-only `service-<date>-<slot>.json` written by `voxxa-eval --service-report` (V2, V13s), with
truth from tapping (rehearsals, Presenter services) or from FreeShow read-back (`--truth-from-screen`, operator-timed,
Correct bar only, FA-66). Let `e = t_press - t_true` in ms, per matched change; `occurrences` = scored sung slide
occurrences.

**Promotion pass per song** (needed before a song may be autonomous): on the latest confirmed rehearsal or scored
shadow run, `false_advances == 0` AND `count(e < -300) == 0` on rehearsal truth AND `recall_1500 >= 0.95` AND
`count(why == lost) == 0` AND S6 met. The result is written to the practice file as `status: promoted`.

**Gate 1 (minder at the PC, ADR-022 section 3 unchanged):** 2 consecutive scored shadow Sundays with the Correct
bar met on every scored song.

**Gate 2 (no minder):** over the last 4 consecutive services, each with S1-S3 and S5 met on every song, the last two
clean on every song: `sum(occurrences) >= 300` AND `sum(false_advances) == 0` AND `sum(count(e < -1000)) == 0` AND
`minders_passed_card >= 2` AND `fault_drills_run >= 1`. Early line: 300 ms on rehearsal truth, 1,000 ms live;
thresholds tighten, never loosen.

| # | SLO | Target (per service unless stated) | Computed from | Alert |
|---|---|---|---|---|
| S1 | Spurious presses | `false_advances == 0` | `score.rs` (exists) | any -> demote the song; incident write-up |
| S2 | Early moves | `count(e < -1000) == 0` live; report `early_300 = count(e < -300)` | signed error, decoupled from the 1 s match window | any beyond 1,000 ms -> demote; `early_300 > 2` in a song -> review before the next service |
| S3 | Wrong-slide seconds | `wrong_slide_s <= 10` per service; `<= 3` per song | `wrong_slide_s` | service > 10 s -> the service does not count toward promotion |
| S4 | Recall within 1.5 s | `recall_1500 >= 0.95` per song and per service | `recall`, `median_ms`, `p95_ms`, `tight_300` | song < 0.90 -> "rehearse again" |
| S5 | Follower availability | `armed_s / sung_s >= 0.99` (armed = health green, no watchdog hold, no panic) | `trace` (follow), watchdog and operator lines | < 0.99 or any panic -> incident write-up |
| S6 | Pass time while singing | `median(pass_s) <= 1.0` and `p95(pass_s) <= 1.5` | per-hop s/pass line | p95 > 1.5 s for 30 s -> watchdog hold, amber "slow"; canary replay > 1.5x the manifest's -> red |
| S7 | Audio health | `dropouts == 0`; `share(peak in [-20, -6] dBFS over sung seconds) >= 0.95`; `floor <= peak - 30 dB` | 1 Hz `MicLevel` summary | out of band 30 s -> amber; sound without words 30 s -> watchdog hold |
| S8 | Read-back agreement (FreeShow) | `acks_within_750ms / moves == 1.0` | dispatch `t_sent`, `t_ack`, read-back (id, layout, index) | any disagreement -> `position_uncertain`, hold |
| S9 | Decision to command | `p95(t_sent - t_decision) <= 100 ms` | `t_decision`, `t_sent` | report only |
| S10 | Overrides and hold time | `overrides_per_song < 2`; `hold_s` trending down over services; `panics == 0` | operator lines (hold, resume, panic, F13/F14, accept, dismiss) | `hold_s > 60` in a service or any panic -> review; >= 2 overrides in a song -> demote |

`trend.csv` gets one row per service (date, songs, occurrences, S1-S10, n_hold, hold_s, n_panic, n_next, n_back,
n_accept, n_dismiss, songs_demoted). Demotion triggers (written to the practice file, surviving restart): live
`e < -1000`, any spurious press, a watchdog hold inside the song, >= 2 overrides, `lost` twice, or any manual move
(FA-61, for the rest of the song).

### 2.8 Risk register (top 15; merged from the seven reviews)

| # | Risk | Owner | Mitigation (step) |
|---|---|---|---|
| 1 | **GPU contention with the projector**: the GTX 960 (2 GB) drives the projector, Whisper and now FreeShow (Electron, GPU-composited); 870 MiB used with nothing running; Whisper's share unmeasured | founder (V3e), agents (V13s) | V3e measures VRAM and CPU with both apps and a video playing; health check red above 1,800 MB; no aligner without a measured number; fallback: close Presenter after the switch, or CPU at V3's settings |
| 2 | **Open API**: FreeShow's REST/WS/OSC bind all interfaces, password optional, `cors: *`; the shipped driver sends no credential | founder (F5), agents (V8b) | password set; 5505/5506 firewalled to loopback; voxxa refuses an API that answers without a password unless `require_password` (FA-55); bearer header only; token never logged or bundled; readiness item "answers only on loopback" |
| 3 | **Budget wall**: "everything else" has 1 line left; voxxa-eval 112 left; the follower port is 600-900 Rust lines against a 500 cap | founder (FA-10), EM | V7 first (this week); the rows in 2.4; R11's fit for voxxa-eval; stop and report at 95% of any row; never raise a cap |
| 4 | **Founder hours** bound the schedule (consent, recording, tapping, every booth step) | founder | read-back as Correct truth removes most tapping (FA-66); tapping one Sunday in four; talk-span notes live, consented volunteer tapper, partial scoring (FA-71); cap 2 h/week written in `kit/README.md`; never the Sunday minder from month two (FA-69) |
| 5 | **n = 1 song**: every parts-map threshold was tuned on one singer, one room, one hymn and its re-cuts; live4's truth is +/- 1 s | founder (F1, FA-45), agents (V3d, V6) | tap live4 before ADR-031 cites it; >= 3 real detour takes, a spoken-aside take, a two-singer take (V3d task 4); V6 held-out gate before V11 flips anything; break gate default `None` |
| 6 | **Two apps on one projector**: a mid-service swap is a black screen; the stream capture path may be Presenter-specific | founder, tech lead, stream person (V17) | per-service switch, whole service (C12); double-build Sundays; Thursday 18:00 rollback rule; `stream_capture` and `stage_display` fields in the church profile; back-to-Presenter runbook |
| 7 | **RDP steals the console** (a locked console drops keys and may redirect audio); the founder cannot mind over RDP | founder (FA-67), agents (V13s) | no RDP Sat 18:00 - Sun 14:00; `SM_REMOTESESSION` check red; Windows Update active hours cover services, auto-restart off; voxxa may auto-start on service days (FA-67 as answered 2026-11-27), Songs armed by a person, the mic never opened by itself |
| 8 | **Stage text in the stream**: the stage window prints live heard words (`src/stage.js:52-55`); next-slide text captured into the stream leaves the room | founder (FA-29), agents (V14) | FA-29 confirmed before any stage display is turned on; heard-words line removed (ticket now, fixed in V14); three-state strip only; voxxa never writes FreeShow stage text (ADR-025 section 7) |
| 9 | **Licensing of weights**: MMS weights are CC-BY-NC; an aligner's checkpoints may be unlicensed; `deny.toml` allows no NC licence | founder (FA-50), agents (V3f) | permissive weights only in product or kit; CC-BY-NC lab-only; ADR-005 amended to cover weights; ADR-033 only after V3f and V6 |
| 10 | **FreeShow self-update**: weekly betas and an Electron auto-updater can change ports or API behaviour mid-week | founder (FA-51), agents (V13s) | stable channel only, auto-update off, version pinned in `church-profile.json` and checked by the health check; upgrades on a weekday; the V8b stub is the regression net |
| 11 | **Repertoire shrink**: no bulk export from WorshipTools; only licensed sources are re-imported; a small church's rotation is mostly CCLI | worship leader, pastor (FA-68) | inventory counts (licensed / none / legacy CCLI) signed before any migration weekend; first ten songs = most-sung licensed songs; songs without a source stay on Presenter during the transition or are dropped by the leader's decision |
| 12 | **Worship leader's reaction to the post-chorus wait** (the next verse appears 1-3 s after its first word) | founder, worship leader (V18) | tell the leader first (worship-leader card); the break-gate arm and timing columns in V3d measure the gap; `boundary = plan` is refused (C8); per-leader handicaps only after >= 3 confirmed runs (FA-64) |
| 13 | **Slide volunteers losing a role**: the tech team shrinks before the machine is ready | founder, pastor (V18, FA-69) | the minder role is framed as an upgrade for the existing slide team; >= 3 minders from that team; a veteran trains the next; one manual song a month keeps the skill |
| 14 | **Replay determinism**: VAD, windowing or read-back feeding the conductor would change every committed replay | agents (V3b, V8b, V11) | every new behaviour behind config, default off (ADR-022); windows do not follow vocal activity in v1; read-back affects dispatch (hold), not the conductor, in v1; `replay byte-identical` is an exit criterion on V3b, V8b, V11, V12 |
| 15 | **Tapping labour**: a scored Sunday costs about 2 h of tapping today; four shadow Sundays is four afternoons | founder (FA-66, FA-71) | FreeShow read-back at 2 Hz as operator-timed truth for Correct; tapping one Sunday in four for Tight; `--learn` confirm-or-nudge replaces tapping for learning; founder <= 45 min per scored Sunday |

Also tracked, below the top 15 (from 42, 43, 46, 47): no error channel on a failed FreeShow select (read-back is
the only signal); a 10 s renderer stall can time out the 750 ms read-back and hold voxxa; many-to-one layout
mapping (unique layouts in v1, FA-56); the active-show ambiguity of a bare `index_select_slide`; the default-device
fallback picking a webcam mic; talk after a chorus triggering elimination (R14); a second singer on the feed; three
copies of the service order (PCO, FreeShow, voxxa) until Q12; FreeShow's PCO sync importing lyrics from a licensed
source before the library moves (PCO connection off during the trial); incidental recording of the congregation
(counsel; FA-70 notice).

---

## Part 3. The founder's decision batch

Each item: the decision, then the recommendation (from adjudication section 5). Already-open items keep their
numbers from `out/founder-action-items.md`.

### This week (by Dec 3; unblocks V7, V3d, F5 and the hardware)

**Answers received 2026-11-26 (batch 1):** FA-38 deferred and FA-39 not required (both deviate from R6: the full mix on the current feed is a supported degraded condition; `out/founder-action-items.md` answers table, ADR-030 amendment 1); FA-40 … FA-49 approved as recommended, FA-43 with a Linux + Windows requirement (FA-73); FA-10 approved later the same day, option (a). **Answers received 2026-11-27 (batch 2):** FA-54 yes with caps at 3 × the estimates (onset 750, input chain 600, health check 450; FA-74 opened for the rule's scope); FA-65 yes, option (a); V7 runs in full. **Answers received 2026-11-27 (batch 3):** FA-29 yes (all Presenter output is a YouTube lower third via Blackmagic: FA-75 opened for the FreeShow path); FA-50 use whatever licensing is required now (ADR-005 amendment = inventory; own licence parked as FA-76); FA-52 yes; FA-55 yes; FA-56 reframed as a per-song JSON layout file; FA-57 keep the 500 ms fade; FA-58 yes with a hotkey-collision check; FA-59 not now; FA-60 proceed; FA-61 no demotion, re-anchor instead; FA-62 … FA-64 asked again in plain words. Still open: FA-51, FA-53, FA-62 … FA-64, FA-66 … FA-75.

| FA | Decision | Recommendation |
|---|---|---|
| FA-10 | The budget wall: approve one 150-line increase as a recorded "wiring" row | **approve this week**; 1 line left blocks every product step on the goal's path · **answered 2026-11-26: approved, option (a)** |
| FA-43 | Approve ADR-031 (Proposed): the parts-map follower | **approve as recommended, with the R3 amendment** (break gate as an option, default `None`) |
| FA-44 | How voxxa goes to any slide in the presentation app | **answered as (d) FreeShow trial via V3e and (e) suggest-only on Presenter until then**; skip (a)-(c), none gives read-back |
| FA-45 | Tap the truth for live4 | **yes, 5 minutes, before ADR-031 cites it** |
| FA-38 (b) | USB interface with a balanced input (hum fix) | **buy now** (~$100-150 [est.] together with FA-39); deletes gain and hum code · **answered 2026-11-26: deferred; gain and hum code stay, degraded mode is supported** |
| FA-39 | A vocal-only aux feed from the console | **buy / request now**; the only credible fix for the band mix · **answered 2026-11-26: not required; must work with music present, with reduced performance** |
| FA-29 | Is the booth desktop duplicated to the projector or captured for the stream? | **confirm the stage output is never captured into the stream before any stage display is used** |
| FA-46 | Approve the V3e spike and installing FreeShow **stable** beside Presenter, weekday before Dec 18; Presenter stays live | yes |
| FA-49 | Budget gate: V3e now (<= $20); V8b (<= $60) on a V3e go; nothing for an aligner, tracker or new model before V6 reports and the GPU is measured with both apps open | yes |
| FA-54 | New ADR-028 rows: onset 250 (split from follower), input chain ~200, health check 150; `vad/` -> recognizer row; `align` 400 only per R5 | yes · **answered 2026-11-27: yes, caps × 3 (750 / 600 / 450)** |
| FA-65 | JSONL: one `trace` line type with `kind` (ADR-001/003 amendment) | (a) · **answered yes, 2026-11-27** |
| FA-63 | Break gate: lab switch in V3d, `PartsConfig.break_gate` default `None` in V11, before/after measured; live4 is not its evidence | yes · **answered 2026-11-27: build it on by default (deviation; V3d still measures before/after)** |
| FA-72 | Approve the drops and deferrals in R17 | yes · **answered 2026-11-27 per line: all drops approved (Holyrics only ever as an ADR-036 bindings file, on need); deferrals as recommended except V16, which runs now** |

### Before Dec 18 (unblocks F5/V3e details, V8b, V10's words, V13a, V18 card v0, the Christmas booth policy)

| FA | Decision | Recommendation |
|---|---|---|
| FA-47 | Two-tier app policy: Tier A (jump + read-back) is autonomy-eligible; Tier B (keys-only) is assisted with suggestion cards; Left bursts refused in code | yes |
| FA-50 | Model and aligner licences: only permissive weights (MIT, Apache, BSD, CC-BY, CC0) in the product or kit tools; CC-BY-NC lab-only; amend ADR-005's dependency rule to cover model weights | yes · **answered 2026-11-27: use whatever licensing is required for now; own MIT-like ministry licence later (FA-76); ADR-005 amendment = inventory, not a gate** |
| FA-51 | Pin FreeShow: stable channel, auto-update off, version in `church-profile.json`, checked by the health check; upgrades on a weekday | yes · **answered yes, 2026-11-27** |
| FA-52 | Two gates: 2 scored shadow Sundays -> songs mode with a minder at the PC (ADR-022 unchanged); no-minder gate = 4 consecutive green Sundays, >= 300 scored occurrences, 0 early beyond the line, 0 spurious presses, two volunteers passed the card; early line 0.3 s on rehearsal truth, 1.0 s live; tighten only | yes · **answered yes, 2026-11-27: extensive listening period; never interfere with the volunteer** |
| FA-53 | Presenter MIDI mapping check (20 min, inside V3e): does any command name a slide or section? A "no" closes the "wait for WorshipTools" option | yes · **answered 2026-11-27: skip it** |
| FA-55 | FreeShow API posture: password set, 5505/5506 firewalled to loopback, voxxa refuses an API that answers without a password unless `require_password` is set to false, bearer header only, token never logged or bundled | (a) refuse by default · **answered yes, 2026-11-27; steps in `out/founder-freeshow-setup.md`** |
| FA-56 | Unique layouts (every slide once) for followed songs in v1; a per-song occurrence resolver later with Q12 | yes · **answered 2026-11-27: the per-song JSON layout file is the object of record (ADR-032 Decision 13 as amended); unique layouts stay the v1 rule** |
| FA-57 | Text transition 0 ms for the followed output style (FreeShow default is a 500 ms fade) | measure in F5, then yes · **answered 2026-11-27: keep the 500 ms default (reading time)** |
| FA-58 | Controls: F15 HOLD (toggle), F20 STOP FOLLOWING (no confirm; dispatch off, listening continues), F14 Back one; routes `/pause /resume /panic /state`; FreeShow output lock as external panic; voxxa API stays 127.0.0.1; no Escape; Off only in the mode switch | yes · **answered yes, 2026-11-27, on condition of no hotkey collisions (FS14 check)** |
| FA-59 | Alarm/veto surface when nobody is at the PC: a Companion/Stream Deck page (or a phone) at the stage, colour from `/state`, calling `/pause`; docs recipe only; hardware optional ($0-80 [est.]) | (a) · **answered 2026-11-27: not now (deferred)** |
| FA-60 | Stage cue: voxxa's own three-state strip; heard-words line removed (ticket now, fix in V14) | yes · **answered yes, 2026-11-27** |
| FA-61 | Badge words "voxxa moves it / voxxa suggests / You move it"; any manual move demotes the song for the rest of the song | yes · **answered 2026-11-27: no demotion; voxxa re-anchors and keeps following; the badge shows live mode** |
| FA-62 | Health-check sentences approved word for word on the V10 mock; "Ready, with a note" may start Songs; only a red slides-app row blocks | yes · **answered 2026-11-27: approve when I see the mock (V10)** |
| FA-64 | Per-leader handicaps only after >= 3 confirmed runs; leader alias L1/L2 in practice files; never `boundary = plan` | yes · **answered yes, 2026-11-27** |
| FA-66 | Read-back as shadow truth (operator-timed) for the Correct bar; tapping one Sunday in four for Tight | yes · **answered yes, 2026-11-27** |
| FA-67 | Booth PC policy on service days: no RDP Sat 18:00 - Sun 14:00; Windows Update active hours cover services, auto-restart off; voxxa started and armed by a person, never auto-start; health check within 30 min of the service | yes (in force for the Christmas services) · **answered 2026-11-27: yes, and voxxa may auto-start (Songs still armed by a person)** |
| FA-69 | Who minds: >= 3 minders from the existing slide team; the founder is never the Sunday minder from month two; a veteran (3 months) trains the next | yes · **answered 2026-11-27: fewer minders is fine** |
| FA-70 | Words: no "AI" from the stage or in the bulletin; one-paragraph notice; the pastor decides wording | yes · **answered 2026-11-27: no notice at all** |
| FA-71 | Scoring labour: talk-span notes live, talk muted, a consented volunteer taps songs only, partial scoring counts; founder <= 45 min per scored Sunday, <= 2 h/week total | yes · **answered 2026-11-27: 90 min per scored Sunday, 4 h per week** |

### End of January (on the dark Sundays' numbers)

| FA | Decision | Recommendation |
|---|---|---|
| FA-48 | Gate and shape for switching the Sunday output: V3e go; >= 2 dark Sundays with read-back agreeing with voxxa's commanded slide on >= 99% of polls and 0 `position_uncertain`; library re-imported, `songcheck` green, groups match labels; worship leader checked the stage display; one volunteer trained on FreeShow keys; **per-service switch, whole service, 4 double-build Sundays, Thursday 18:00 rollback rule, Presenter kept a month**; window Jan 10 - Mar 21, 2027; decide end of January | adopt |
| FA-68 | Repertoire sign-off: the worship leader signs the inventory counts (licensed source / none / legacy CCLI) before any migration weekend; first ten re-imported songs = most-sung licensed songs | yes (counts prepared in V17 during January; signed before the first double-build weekend) · **answered 2026-11-27: later (decide at the end of January)** |

### Pre-approvals to ask for in one batch

For every step in Part 2.2, under the standing safety block (never open the mic unattended; never press keys in
Presenter; never start the installed app; never read or print lyrics or heard words; never touch private folders;
media guard stays required; numbers only in anything that leaves the PC):

1. **push** to lane branches (`vN/10x`, `vN/ponytail`) off `main`;
2. **draft PR** per step, with the comparator and verifier records in `.10x/reviews/`;
3. **CI fixes** on the lane branches without a new ask;
4. **merge when green**, including the **media guard** check, on all three OSes; no tag, no release, no installer;
5. **tickets** as GitHub issues, numbers-only: W1's resolution, the stage heard-words line (`src/stage.js:52-55`),
   the Q13 FreeShow fixture, the FreeShow test-card rows;
6. **stopping agents' own processes** by PID (never the founder's, never Presenter, never FreeShow);
7. **one FreeShow install on the booth PC by the founder** (F5): stable release from the official page, SHA-256
   checked, auto-update off, API password set, 5505/5506 firewalled to loopback, Planning Center connection off,
   no output assigned, on a weekday before Dec 18; nothing on the booth PC between Dec 18 and Jan 3.

---

## Part 4. Cost summary

| Item | Before | After | Note |
|---|---|---|---|
| Plan total, agent cost, with ponytail lanes and verifiers | ~$315-780 (plan section 2, 22 steps) | **~$350-875** | adjudication section 6, from the CTO's estimate |
| FreeShow track increment (V3e, V8b, break-flag case in V3d, read-back cross-check in V15, dark Sundays) | - | **+$37-95 agent; 6-8 founder hours over six weeks** | 41 section 4 |
| New steps beyond the FreeShow track (V3f, V13s, V17, V18, V13a split, V5a split) | - | roughly offset by the drops and deferrals (V6b dropped; V13b, V5b/V5c, V16, V8 steps 2-4 deferred; V3c conditional) | (inferred) reconciliation below |
| Founder hours | ~2 h tapping per scored Sunday; every booth step | **capped at 2 h/week** (FA-71): Thursday 30 min, Sunday 0 min after month one, Monday <= 45 min; never the Sunday minder from month two | read-back as Correct truth is the main saving |
| Cash | - | $0 now for FA-38 (b) and FA-39 (deferred 2026-11-26; ~$100-150 [est.] if bought later); $0-80 [est.] optional Companion hardware (FA-59); FreeShow $0 | - |
| Aligner (ADR-033), if ever | - | L lab + new row, $50-100 [est.], 2 h founder; **not before V3f passes and V6 reports** | 41 section 4 |

Reconciliation [est.] (inferred): the step list in 2.2 has 12 S steps (V0, V1, V3, V4, V5a, V7, V3e, V8-lite, V9,
V10, V13a, V15), 11 M steps (V2, V3b, V3d, V6, V8b, V3f, V12, V13s, V14, V17, V18) and 1 L (V11), plus V3c
conditional. One lane: $36-96 + $110-275 + $37-75 (with rework) = ~$183-446; with ponytail lanes and verifiers
~$340-830; with V3c ~$345-845. That sits inside the canonical ~$350-875 band, so the headline stands.

**Budget gate (FA-49):** V3e now (<= $20); V8b (<= $60) on a V3e go; nothing for an aligner, tracker or new model
before V6 reports held-out numbers and the GPU is measured with both apps open. The process rule stands: any step
over $50 needs a yes; 1.5x the estimate stops the step.

The limits are the founder's gates, Sundays (recordings, dark Sundays, double-build Sundays) and the live
installs, not tokens.
