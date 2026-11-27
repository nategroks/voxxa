# 47 — Implementer review of "Ship FreeShow Jumps Before Smarter Listening"

Reviewer: the engineering session that wrote `lab\predict.py`, `lab\parts.py` and `lab\live.py`, ran live1 to live4
with the founder, and wrote ADR-030 and ADR-031. Date 2026-11-26. Slide indices only; no lyric text. Evidence
tags: (measured) = my own runs or the code at `C:\dev\r2-review` (a5f7e2e); (cited) = the report's source;
(inferred) = reasoning from those; (opinion).

**Verdict in one paragraph.**
- **I agree with the direction.** The output path is the ceiling. FreeShow is the right long-term target. The
  parts map is the right follower. Learning belongs in rehearsal review, not on Sunday.
- **I disagree with how cheap rank 1 looks.** Today's FreeShow driver cannot run a service on its own (§ 1, S1).
- **The report misreads three measurements** that shape its proposals:
  - the 80% within 300 ms belongs to the plan-order follower on one take, not the parts map;
  - live4's early moves were onset fires after a breath, so a break gate would not have held them;
  - 814 MB is the GPU baseline before Whisper loads, not Whisper's share.
- **Re-ordered:** a short FreeShow and VRAM probe first; the parts map's real detour takes in parallel; the input
  chain moved up; the frame-level aligner left as a lab probe after the VRAM numbers.

## 1. Findings, ranked by severity

**S1 (high): rank 1 is an M–L build, not "ten minutes", and today's FreeShow path stops at every song change.**
- `presenters/freeshow.rs` (measured, at a5f7e2e):
  - `goto_slide` sends `{"action":"index_select_slide","data":{"index":N}}`, with no `showId` or `layoutId` (`:163-167`);
  - `can_query_state: false` (`:91`), and `current_state` is not implemented;
  - the module comment says the HTTP API "requires no authentication" (`:5`), while `kit/templates/bindings/freeshow.json` says bearer auth.
- `dispatch.rs` (measured): for a goto driver, a new song first calls `presentation_is` (`:136-143`). Without
  `can_query_state` that returns only the operator flag (`:66-70`). So every song change the conductor makes on
  FreeShow blanks the output and asks the operator to select the song. Autonomy across songs needs
  `current_state` (show id → name) before anything else.
- **The follower reports a slide id; FreeShow's `index_select_slide` takes a layout position** (inferred from the
  API's `layoutId` parameter). A FreeShow layout repeats groups, so "go to the chorus" is a choice among layout
  positions. The driver needs `get_show`/layout data to map slide id → nearest layout position, or must use
  `id_select_group` with its "next matching group" semantics (cited: FreeShow MIDI docs). Neither exists in the
  driver.
- **Fire-then-verify** needs a `get_output` read-back that keeps ids only (the kit template already says so). It
  must never use `get_output_slide_text`: that returns words and breaks the no-words rule (S10).

**S2 (high): the parts map is not at "80% within 300 ms"; it is at 54–71% within 1 s.**
- The conclusion's "onset-triggered parts map reaches 80% on its labelled takes" is wrong. 80% within ±300 ms
  was T1 alone, with the plan-order `Predictive` follower plus the onset trigger (ADR-030) (measured).
- The parts map's share of changes after the first within ±1 s (measured, `evalparts.py`): T1 0.60, T2 0.60,
  F1 0.54, F2 0.71, F3 0.56, F4 0.55, live4 0.67. Median |delay|: T1 0.24 s, live4 0.78 s, F3 and F4 0.66 s.
- The late tail is the known cost, at part ends after a chorus and on slides whose closing words Whisper missed.
- The proposed promotion gate (≥ 95% within 1.5 s, 0 early beyond 0.3 s) would keep every song in suggest-only
  today (inferred). That is a fine gate, but the plan must say the gap is speed at part ends, not recognition
  accuracy.

**S3 (high): the break flag would not have held live4's early moves.**
- The report says the two post-chorus fires and the look-alike move "happened mid-phrase, where a break-gated
  cost would have held". They were the opposite. All three were onset fires, and an onset requires ≥ 200 ms
  below floor + 5 dB first, so each followed a break by construction (measured: `predict.py` `OnsetStream`; replay
  of live4's session log).
- They were also in-plan moves (next slide, next planned part), not jumps, so a jump prior does not touch them.
- The parts map already removed them: after a chorus it waits for words, so the replay has 1 move more than 1 s
  early instead of 5 (measured). Where a break flag can help is § 3.1.

**S4 (medium-high): VRAM is unmeasured with Whisper loaded, and the GPU drives the projector.**
- 814 MB was the baseline with Presenter open, measured to show Presenter was not why Whisper was slow. Today, with
  no voxxad running, `nvidia-smi` read 870 of 2048 MiB (measured).
- Whisper Small's own share on this Vulkan build was never logged (I don't know it).
- The founder said the GTX 960 also drives the projector. FreeShow's output window plus Whisper plus an aligner on
  2 GB is a service risk the report does not name (§ 6).

**S5 (medium): what live4 shows.**
- live4 ran the plan-order follower (`live.py` → `predict.Predictive`), and the founder sang the planned order
  (measured, from the heard words).
- `live.py` never sends Left (ADR-030 § 8), so no back move was attempted. "Could not return to a chorus" was the
  follower's fixed list, not a Presenter refusal. The parts map has never run live.
- Presenter's ceiling is real (shortcuts page; binding profile), but live4 is not evidence of it.

**S6 (medium): a per-leader prior that turns on "fire at every part end" does not recover the post-chorus lag.**
- The report's step 5 sets `boundary = plan` for leaders who keep the plan. Measured on the final code:
  `boundary = plan` is worse even on T1, where the plan was kept.

| | Right | Wrong | > 1 s early | Wrong-slide s |
|---|---|---|---|---|
| T1, `boundary = plan` | 9/11 | 3 | 2 | 24.2 |
| T1, default | 11/11 | 1 | 0 | 10.7 |
| 7 sung takes, `boundary = plan` | 68 | 19 | 9 | 223 |
| 7 sung takes, default | 74 | 4 | 1 | 112 |

- Cause (measured, traces): after a chorus, the next breath is often a repeat of the closing phrase or an ad-lib,
  whatever the plan. Whether a leader keeps the plan is not the variable.
- Firing on the breath only from verse into chorus is worth it: against "words at every part end", it saves 34 of
  146 wrong-slide seconds with no added wrong move (measured).

**S7 (medium): the port does not fit today's follower budget row.**
- ADR-028's follower row is 1.3 × the measured port, ≤ 500.
- The lab sizes, in raw lines including docstrings and scoring (measured): `parts.py` 395 Python lines; the onset
  detector and arm/fire rules ≈ 150 of `predict.py`'s 366; the plan-order follower kept for unlabelled songs
  (`follower.py` 144).
- A Rust port of all three with the named tests is likely 600–900 code lines (inferred). It needs new recorded
  rows (onset, parts), not a raised cap.
- `vad/` (189 lines) counts in "everything else", whose headroom ADR-028 keeps at 1 line (`scripts/loc_budget.py:88`).
  The energy gate therefore needs a row move (into the recognizer row, cap 700) in the same PR.
- "A few lines inside the existing budget" is true only in the lab.

**S8 (medium): the product already has section priors; the report treats the parts map as new.**
- `aligner/evidence/` (`mod.rs` 225, `sections.rs` 76; ADR-009's minimum) already scores repeat-aware moves.
- live4's session log shows `prior_next 0.9`, `prior_chorus 0.8`, `prior_section 0.6`, `prior_other 0.4`,
  `min_dwell_ms 1500` (measured, config block only).
- The V11 port should replace or absorb that accumulator under one config switch, not stack a third follower on
  the conductor (opinion).

**S9 (low-medium): metric mapping.**
- "Right changes" in the lab tables is voxxa-eval recall: a move counts if it shows the truth slide before the next
  change (1 s grace). It is not "within 1.5 s"; the timing columns are separate.
- Wrong-slide seconds runs from the first truth row to 10 s after the last, in 50 ms steps. It is close to
  (1 − PCS) × duration, but not equal to it.

**S10 (low): never read words back.**
- `get_output_slide_text` and `get_output_group_name` return text. Read-back must use `get_output` ids (show,
  layout, index) and group ids, never text. Logs stay numbers-only.

## 2. The six ranked steps against the code, the numbers and the budgets

| Rank | Step | Feasible as written? | Effort (my estimate) | Gain on the 7 takes and live4 | Blocker |
|---|---|---|---|---|---|
| 1 | FreeShow jump driver with read-back | Yes, after S1's four fixes | M driver + M dispatch + a founder booth probe (≈ 1 h) | Makes the detours executable: F1, F2 and F3 each need at least one move Right-only cannot make (a back move, or > 3 steps forward) (inferred from the part orders). No change to the follower's numbers | W1 nesting, index base, layout positions, auth, VRAM beside the projector |
| 2 | Finish the parts map plus a break-gated prior | Yes; the break part is small | Parts map V3d as planned (M); break flag S (lab ≈ 20 lines) | Break flag: 0 of live4's remaining errors; on F1–F4 maybe 0.5–1 s faster detour detection, if 1 window is allowed after a break (opinion; not run) | Real detour takes; budget rows (S7) |
| 3 | Whisper for which; onset plus a frame aligner for when | Partly: ADR-030 already splits which and when; the aligner is unproven on this GPU | Probe S; CRNN port L plus a new dependency (ONNX runtime) and row; OLTW lab M | Within-part late moves only: T1 slide 5 (+3.7 s), live4 slide 1 (+2.6 s), T1 the V1 → C move (+2.6 s); a guess of −10 to −20 of 112 wrong-slide seconds (opinion). Part ends are not helped | VRAM (S4); rehearsal references for OLTW |
| 4 | Product input chain and a vocal-only feed | Yes | Input chain M (gain, high-pass, limiter, check); energy gate S plus an ADR-028 row move | On the gained lab feed ≈ 0: live4 had 216 passes in about 4 min, nearly every hop (measured). Its value is robustness: live1 and live3 failed on setup | Replay determinism: VAD changes alter logs, so config default off |
| 5 | Listener DB as hindsight alignment | Yes (ADR-023 shape) | V12 L as planned; per-leader statistics S once data exists | 0 now: one leader, 0 real detours recorded | Consent, data volume |
| 6 | Minder role, kill key, rehearsal gating, card | Yes | M (UI plus kit) | Not measurable on takes | It must land before any unattended run |

## 3. The specific proposals

1. **Break flag from the onset detector with a grace window.**
   - The signal exists: `OnsetStream.push` returns `gap_ms`, and `parts.py` already uses it for elimination
     (`ELIM_GAP_MS` 300).
   - Use it for genuine detours: allow a jump with 1 agreeing window when its alignment starts within about 1 s
     after a break, and keep 2 windows mid-phrase. Whisper writes its phantom repeats during held notes, which is
     mid-phrase (measured: the T1 traces).
   - Effort S: about 20 lines, then a 5-minute `evalparts.py` run.
   - Expected gain: faster jumps on F1–F3 (the F2 jump came 7 s after elimination, F3's correction at about 3 s),
     wrong moves 4 → 3–4. Not the live4 errors (S3).

2. **A per-leader resume prior from detours.** Sound as a prior over the handicaps (planned next 0, chorus −0.5,
   same −1.0, other −1.5). It needs V12 data: dozens of part ends per leader before it beats hand values (opinion).
   It must not be wired to `boundary = plan` (S6). Effort S in the practice row; gain unknown until data exists.

3. **Whisper as witness only.** Already the architecture for timing (ADR-030 words arm, onsets fire). Taken
   literally (words never move a slide), slides on a band mix would stop moving, because onsets find only 2 of 11
   starts on P1 (measured). Keep words as the fallback mover.

4. **CRNN, MMS_FA or Matchmaker OLTW beside Whisper.**
   - **CRNN** (LyricsAlignment-MTL): a PyTorch model; in voxxa it means an ONNX runtime crate and DLL, a new ADR,
     a new row and installer growth. L, after a VRAM probe. GPU fit unknown (S4).
   - **MMS_FA** (wav2vec2 class, about 1.2 GB fp32; inferred): CPU only on this booth. CPU headroom with Whisper on
     the GPU is unmeasured. Offline only (rehearsal labelling, step 5): there it is a good fit and needs no VRAM.
   - **Matchmaker OLTW:** CPU-cheap (cited: 0.07 ms per step). It needs a reference capture of the same song, and
     within a part only.
   - The first test I would run: T1 as the reference for live4 (same singer and song, different day), and score
     the within-part changes. M in the lab.
   - Order: the OLTW lab arm, then a VRAM probe, then decide on a CRNN.

5. **An energy gate replacing or OR-ing Silero.**
   - `vad/mod.rs` already has an RMS fallback (`ENERGY_THRESHOLD` 0.005, used only when Silero fails).
   - OR-ing a 300–4000 Hz band-energy gate after the input chain is S in the lab: a local voxxad build, then re-run
     the caches on T1 and live4.
   - In the product it needs the ADR-028 row move (S7) and must be off by default (ADR-013 replay).
   - Expected gain on gained takes ≈ 0 (passes were already near every hop); it is insurance against level errors.
   - **Do not make windows follow vocal activity in v1.** That changes the recognizer's hop and stitcher, and with
     it every committed replay.

6. **A short tail prompt for V3c.** Agree: the current slide's remaining line, ≤ 40 tokens, at the prompt's tail.
   - **Never prompt the planned next part at a part end.** A prompt of the planned part biases Whisper toward the
     plan and hides detours, the very thing the parts map relies on (inferred).
   - V3c must add detour detection on F1–F4 (right changes and jump delay) to its acceptance.

7. **FreeShow fire-then-verify.**
   - Yes, with `get_output` ids, a 150–300 ms verify, one retry, then hold and amber (inferred values, to measure in
     the probe).
   - This replaces ADR-025 § 2's per-key position counting for this driver.
   - The front-window guard no longer applies to HTTP moves. Replace it with a localhost-only bind and auth check
     (S1 found the driver assumes no auth).
   - Effort M in `presenters/` (row cap 2000; current headroom unknown without tokei) plus dispatch's song check (S1).

## 4. The one experiment I would run first

**A FreeShow booth probe, about 1 hour, founder at the booth, no service.**
- Install FreeShow beside Presenter; Presenter stays the live output. Build It Is Well as a FreeShow show: groups
  V1–V4 and C, layout V1 C V2 C V3 C V4 C. It is public domain.
- Run a numbers-only script of about 10 HTTP calls:
  - `index_select_slide`, body at top level versus under `data` (W1);
  - the index base;
  - whether the index is a layout position when groups repeat;
  - `id_select_group` by id versus by name, and its "next matching group" behaviour from inside C;
  - `get_output` (keep ids only), with the response time for verify;
  - auth on versus off, and the bind address.
- With Whisper Small loaded in voxxad `--shadow` and FreeShow's output on the projector display, read `nvidia-smi`.

Why first:
- it decides rank 1 and the VRAM budget for rank 3;
- it needs no audio, lyrics or new code;
- every follower gain stays capped by Right-only until a jump exists.

Second, in the lab and cheap: the break-gated 1-window jump on the 7 takes (§ 3.1).

## 5. What the report should have asked me

1. Which follower ran live4? (The plan-order one; the parts map has never run live.)
2. What is the 814 MB? (Baseline before Whisper; Whisper's share is unmeasured.)
3. What are the parts map's timing shares, not only right and wrong? (54–71% within 1 s.)
4. What do the 4 remaining wrong moves look like?
   - T1: an elimination guess, corrected.
   - F2: elimination picked the planned verse while a different verse was sung.
   - F3: the first-slide guess, corrected.
   - live4: one change inside the chorus between look-alike slides, against a truth read from the heard words.
5. Does `boundary = plan` help a plan-keeping leader? (No; S6.)
6. Is the VAD the bottleneck once gain is applied? (No: 216 passes in about 4 min live.)
7. How big is the port, against the follower row? (S7.)
8. What can the product's FreeShow driver do today? (S1.)
9. How was live4's truth made? (Read from the heard words, ±1 s; FA-45 taps it.)
10. Were the thresholds tuned on the evaluated takes? (Yes, all of them. F1–F4 share T1's singer and room.)

## 6. Closing items

### 6.1 Verdict on the ranked path
- **Agree** with step 1's target, step 2, step 5 and step 6.
- **Change the order:**
  - 1a is a FreeShow and VRAM probe (S, first);
  - 2, the parts map's real detour takes and port plan (V3d), runs in parallel because it does not depend on the
    output app;
  - step 4, the input chain, moves to second: live1 and live3 failed on setup, and it is cheap;
  - 1b is the FreeShow driver build (M–L) after the probe;
  - step 3 becomes a lab probe: OLTW first, a CRNN only after the VRAM numbers;
  - steps 5 and 6 as written, with the kill key landing before any unattended run.
- **Reject:** wiring a per-leader prior to `boundary = plan` (S6), and Whisper-as-witness taken literally (§ 3.3).

### 6.2 Changes to plan.md
- **Add V3e — FreeShow probe and driver read-back (M).**
  - Founder probe (§ 4), then `freeshow.rs`: W1 body, showId and layoutId, slide id → layout-position mapping,
    `current_state` from `get_output` ids, fire-then-verify with one retry then hold, auth and a localhost bind.
  - `dispatch.rs`: the song check uses read-back; a contract test against a mock HTTP server.
  - Exit: contract tests green; a supervised probe repeats C with one call and verifies it by id; VRAM with
    Whisper, FreeShow and the projector measured and under the FA-47 line.
- **Modify V3d.** Add the break-gated 1-window jump arm and the OLTW-within-part arm (T1 as live4's reference) to
  task 3. Report within 0.5, 1.0 and 1.5 s per take beside right/wrong/early. The port plan must state its line
  count, and propose new ADR-028 rows (onset, parts) if the measured port exceeds the follower row.
  - Exit: as written, plus both arms scored and the budget rows proposed.
- **Modify V3b task 6.** The input-chain ticket adds the energy gate OR Silero (config, default off) and moves
  `vad/` into the recognizer row (an ADR-028 amendment). Exit: a replay of committed logs is byte-identical with
  the gate off.
- **Modify V3c.** Add detour detection on F1–F4 to the acceptance. The prompt never contains a part the follower
  has not entered.
- **Add V3f — aligner feasibility probe (S, lab, measure only).** CRNN (ONNX) on the GPU beside Whisper, and
  MMS_FA on the CPU for offline rehearsal labelling: VRAM, CPU and time per second of audio. No product code.
  - Exit: a numbers table, and a go or no-go ADR for an aligner row.

### 6.3 Prompts for the steps I would own

**V3e — FreeShow probe and driver read-back** (M)
- **Size** M · **Phase** 2 (evidence) + 4 · **Owner** Sr Eng 2 (presenters) + Security Engineer · **Depends on** FA-46, FA-47; V0 merged · **Unblocks** jumps for ADR-031 on FreeShow; the V11 jump interface
- **Founder decisions needed:** FA-46 (install FreeShow for a probe), FA-47 (the VRAM line). **Pre-approvals needed:** push/PR for the driver and test PR after CI (standard batch only if the founder adds V3e to it).

> /10x-Team:10x-team
> Phase 2 (evidence), then Phase 4, roles: Senior Engineer + SDE + Security Engineer + SRE (VRAM). Read ADR-019, ADR-024 § 5, ADR-025, ADR-031 § 7, `src-tauri/src/presenters/freeshow.rs`, `src-tauri/src/dispatch.rs:60-175` and `kit/templates/bindings/freeshow.json`. Two lanes (`v3e/10x`, `v3e/ponytail`); comparator; record `reviews/YYYY-MM-DD-ponytail-v3e.md`.
> 1. **Probe checklist** for the founder (you never run it; numbers only). FreeShow beside Presenter; Presenter stays live. A public-domain test show with groups V1–V4 and C, layout V1 C V2 C V3 C V4 C. About 10 HTTP calls from a PEP 723 script you write under `scripts/lab/freeshow_probe.py`: W1 (`data` versus top level), index base, layout position versus slide, `id_select_group` id versus name and its repeat behaviour, `get_output` shape (ids only; the script drops every other field before printing), auth on and off, bind address, and the latency of each call. Then `nvidia-smi` with voxxad `--shadow` Small loaded and FreeShow's output on the projector display.
> 2. **Driver**, from the probe's answers: the body shape; `showId`/`layoutId`; a slide-id → layout-position map (nearest occurrence after the current one, else before); `current_state` from `get_output` ids and `get_show` name for the song check; fire-then-verify (verify by id within the measured latency, one retry, then `position_uncertain` and amber); never call `get_output_slide_text` or `get_output_group_name`; bearer token from settings; refuse a non-loopback host unless the operator set it.
> 3. **Tests:** a mock FreeShow HTTP server in tests (no network): body shape, mapping with repeated groups, verify mismatch → hold, auth header, no text getters called (a test asserts the action list).
> Budget: presenters row (cap 2000); the dispatch change in the wiring row; report both counts. Safety: standard; the probe script runs only on the founder's booth, and never touches Presenter or its files.
> State files: `decisions/{sde,security,sre}/church-autoadvance.md`, `status.md`, `handoff.md`, `reviews/YYYY-MM-DD-v3e-freeshow.md` (numbers only). Exit: CI green incl. the media guard; the probe table is filled; a supervised probe repeats C by one call and verifies it by id; VRAM under the FA-47 line, or the step stops and reports.

**V3d amendment — the break-gated jump and OLTW arms** (adds to V3d task 3; S + M)

> Add to V3d task 3: (a) a lab switch in `parts.py`: a jump is accepted on 1 agreeing window when its alignment starts within `BREAK_GRACE_MS` (default 1000) after an onset gap ≥ 300 ms, and on 2 windows otherwise; (b) a lab arm that runs Matchmaker OLTW within the current part, against a rehearsal capture of the same song (live4 against T1), and moves a within-part slide at the reference's predicted change time only when the follower has armed it. The founder runs both; paste per take: right, wrong, early beyond 0.3 s and 1 s, within 0.5/1.0/1.5 s, and wrong-slide seconds, for both arms and the default. Safety: standard; OLTW runs on recorded takes only.

### 6.4 Skills and context files
- **Public repo `.10x/`:**
  - `adrs/030`, `031` and the V3e ADR (driver read-back and verify);
  - `reviews/*-v3e-freeshow.md`;
  - `decisions/sde/church-autoadvance.md` with the lab-to-port line counts;
  - `specs/2026-11-11-prompts-10x-team.md` (format).
- **Public `kit/`:**
  - `templates/bindings/freeshow.json` updated from the probe (auth, body, read-back ids only);
  - `templates/presenter-test-card.md` gets FreeShow rows;
  - `prompts/04-bindings.md` gets the FreeShow probe;
  - a minder card template (step 6): `templates/minder-card.md`.
- **`.claude/skills/`:** a `voxxa-freeshow-probe` skill (checklist plus numbers-only script rules), reusing
  `kit/skills/voxxa-layout-author` for the who-moves-it table.
- **Private church folder:** the FreeShow test show, probe outputs, rehearsal captures for OLTW references, and the
  lab files (`parts.py`, `flows.py`, `evalparts.py`, `predict.py`, `live.py`). Never in the repo.
- **Research notes:** this round's report and notes, read-only context for V3e/V3f; cite by URL, never copy text
  into the kit.

### 6.5 Founder decisions (proposed numbers; renumber at adjudication)
- **FA-46 — Install FreeShow on the booth PC for a probe, not as the live output.** Recommend yes: one weekday hour;
  Presenter stays live.
- **FA-47 — A VRAM line for the projector GPU.** Recommend: total VRAM ≤ 1.6 of 2.0 GB with everything open, and
  no new GPU model without a measured number under it. Why: the GTX 960 drives the projector.
- **FA-48 — The early-move line.** Recommend gating at 0.3 s in rehearsal and reporting 1.0 s live, as the report
  proposes; keep ADR-030's ≤ 0.4% budget.
- **FA-49 — FreeShow API exposure.** Recommend loopback only, auth on, and the token in voxxa settings, never in
  logs or help bundles.

### 6.6 Risks the report missed
1. **Projector starvation:** FreeShow output plus Whisper plus an aligner on the 2 GB GPU that drives the projector
   (S4).
2. **Song changes on FreeShow blank and ask the operator** until `current_state` exists (S1).
3. **Text read-back** (`get_output_slide_text`) would put words into logs and help bundles; ids only (S10).
4. **An open control port:** the driver assumes no auth. Any local process, or the LAN if FreeShow binds widely,
   could drive the projector (S1, inferred).
5. **Budget overflow at port time** (S7), and a third follower beside the evidence accumulator (S8).
6. **Overfitting:** every parts-map threshold was tuned on the same 7 takes; F1–F4 share T1's singer and room;
   live4's truth is ±1 s.
7. **Prompt bias against detours** in V3c (§ 3.6).
8. **Replay determinism:** VAD or windowing changes alter committed logs (ADR-013); config default off.
9. **Two presentation apps during a migration:** volunteer confusion, plus media in two libraries. The
   church's licensed videos must never be copied by agents.
10. **RDP:** the founder cannot run the minder role over RDP during a service (the console is locked while RDP is
    connected), so the minder must sit at the booth or use the stage-reachable control.
