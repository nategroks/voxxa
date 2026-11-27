# ADR-031: Parts-map follower: a song is a map of parts, and the plan is a hint

**Status:** Proposed (2026-11-26; amendment 1 drafted the same day, round 2) · **Date:** 2026-11-26 · **Feature:** church-autoadvance · **Reversibility:** Easy (follower config, default off; songs without part labels keep ADR-022's plan-order follower) · **Owner:** nategroks · **Author:** 10x-Team, from the first live Presenter test (live4) and the parts lab (takes T1, T2, P1, F1–F4)

Amends ADR-022 § 2 (a back move only to a section start), § 4 (dead-reckoning) and § 6 (logging). Amends ADR-030 § 1 (FIRE) and § 8 (the live helper). The founder chose this direction (option A, "a map of parts that can go to any part at any time") on 2026-11-26.

## Context
- **Live test (2026-11-26, "live4"):** the founder sang It Is Well (public domain) into the vocal mic. The lab helper (ADR-030 § 8) followed with ADR-022's plan-order follower plus ADR-030's onset trigger and pressed Right in WorshipTools Presenter: 16 presses, in order. The founder's report: "no lag honestly, had the opposite problem where it moved prematurely and it seems to have a hard time going back to the Chorus. Maybe we should think about this differently, as different parts of the song will often be referenced again when flowing in worship."
- **Why it moved early (replay of live4's own session log; truth read from the heard words, about ±1 s):** 11 of 16 changes right, 5 more than 1 s early. Two were dead-reckoning steps in mid-verse while the singer held long notes (3.4 s and 4.2 s early). Two were onset fires after a chorus, on a repeated "with my soul" before the next verse began (2.7 s and 5.0 s early). One was inside the chorus, between its two near-identical slides. The plan-order follower assumes the next line comes at a set pace, and that the phrase after a chorus is the next verse.
- **Why it cannot go back:** ADR-022's prototype walks one list, the planned order. An extra chorus, a skipped verse, a chorus sung first or a repeated last line breaks it. Four "flow" takes were made by re-cutting T1 (the founder's solo take, truth tapped) into other part orders, with the plan unchanged (lab `flows.py`, 30 ms crossfades at the quietest moment before each part). Changes right with the plan-order follower: F1 extra chorus 9 of 14, F2 skipped verse 4 of 8, F3 chorus first 0 of 10, F4 repeated last line 9 of 12.
- **The setlist already has parts.** `sections` (kind plus slides) exist in the setlist model (ADR-016) and the `.txt` importer fills them (ADR-022 § 7, plan V5). It Is Well: Verse 1–4 and Chorus, two slides each.
- **WorshipTools Presenter's keys** (its documented shortcuts): next and previous slide, next and previous service item, F10–F12 clear. It has no key for a section or a slide number. Going back to the chorus means several Left presses, and each slide in between appears on the screen.

## Decision
1. **The song model.** A song is a set of parts (the setlist's `sections`). The planned order (the arrangement) is a hint: a sequence of part instances. A part is *chorus-like* if its kind is chorus, refrain, bridge, tag, pre-chorus or ending, or if it appears at least twice in the plan. The follower's position is (slide, word), not an occurrence in the plan. It reports a slide id; the presentation driver chooses how to get there (§ 7).
2. **Words decide.** Each Whisper window is aligned (ADR-022's local word alignment, pace penalty and search width) along candidate paths from the current position:
   - the rest of the current part;
   - the rest of the current part, then the first slide of each part. Handicaps, in matched-word units: planned next part 0, a chorus-like part −0.5, the same part again −1.0, any other part −1.5;
   - the rest of the current slide, then the same slide again (a repeated line), except on a slide that repeats its own closing words (ADR-030's ambiguous slides).

   A part is always entered at its first slide. A window that only fits a line shared by several parts (for example "it is well with my soul") fits the current part as well as any branch, so it never moves the slide by itself.
3. **Gates for leaving the plan** (each a named test):

| Gate | Rule | Failure it fixes (lab) | Test |
|---|---|---|---|
| Shared lines | a move off the plan must fit the words better than anything in the slide on screen, by 1.5 matched words | Whisper "heard" a duplicate "with my soul, with my soul" during a held note and jumped back to the chorus's first slide | `shared_line_never_moves_alone` |
| Two windows | a move off the plan needs 2 consecutive windows that agree | the same, on the next window | `off_plan_needs_two_windows` |
| Look-alike slides | two slides whose words are ≥ 70% the same (It Is Well's two chorus slides differ by an echo) are look-alikes. Words never jump between look-alikes; the order decides. Words may skip a slide to land on a look-alike of the slide on screen only after time to sing the words in between at 1.5 words/s | T1 bounced between the chorus slides; on P1 the follower skipped to a later repeat of the same line | `no_jump_between_look_alike_slides`, `no_skip_to_look_alike_too_soon` |
| Self-repeating slide | on a slide that repeats its own closing words, neither "this slide again" nor "this part again" is a candidate | T2's refrain sent the position back to its start until verse 2 was out of reach | `no_repeat_branch_on_self_repeating_slide` |
| Lost | 10 s with voice and no anchor: the whole current part and every part start come into reach. After 20 s, the whole song, preferring later slides | P1 jumped back to an earlier repeat of a line | `lost_searches_forward_first` |

4. **Timing: when a slide changes before its words are heard** (amends ADR-030 § 1):
   - *Inside a part:* the onset fires the next slide, as in ADR-030.
   - *Verse to chorus* (the current part is not chorus-like, and the planned next part is): the onset fires the chorus's first slide. This step is reliable in worship.
   - *Any other part end* (after a chorus, a bridge, a tag): the next part is not known, so words decide. **Elimination:** if, 2 s after the next phrase starts, the singing goes on and the words fit neither the part just sung (< 1.5) nor any part's start (< 3), the plan's next part is shown. Whisper often mishears a verse's first line; on T1 it took 10 s to hear verse 2 clearly. Test: `elimination_shows_planned_part`.
   - *Review:* for 8 s after any guess (the first-slide onset, an onset at a part end, elimination), the first slide of every part except the part just left stays in reach, so words can correct the guess. A correction does not use up the guessed part in the plan. Test: `guess_corrected_within_review`.
   - *Option:* `boundary = plan` fires the onset at every part end, for a leader who keeps the plan. It is off by default. Practice memory may turn it on per leader and song later (ADR-023), from measured habits.
5. **Dead-reckoning never moves the slide in a song with parts** (amends ADR-022 § 4). Pace still sets the search width and the look-alike skip check. Words and onsets move slides; time alone does not.
6. **A song without part labels** (one part) keeps ADR-022's plan-order follower and ADR-030's trigger, unchanged. On P1 (a full band, no labels) that follower got 11 of 11 and the parts follower 8 of 11. Labels come from the importer, a service-plan arrangement or the operator. Test: `unlabelled_song_uses_plan_order`.
7. **Moves in the presentation app.** The next slide in the presentation's order is one Right. Any other move (back to a chorus, a skipped verse, a part out of order) needs a jump, and Presenter has no jump key. The driver choice is a founder decision (FA-44). Until it is made:
   - the lab helper presses only Right (ADR-030 § 8), at most 3 steps, when the target is ahead in the presentation's order;
   - for any other move it logs the target ("would go to Chorus, slide 1"), sends no key, and stops pressing keys for the rest of that song (the operator takes over); dry runs score every move;
   - a backward move is never sent as a burst of Left presses without FA-44's approval.
8. **Logging:** the `follow` line's `why` gains `jump` (left the plan on words), `elim` (elimination) and `lost`, plus `part` and `ptr` (the plan position). Numbers only.
9. **Evaluation:**
   - the flow stress set (re-cuts of tapped takes into other part orders, plan unchanged) joins V3b's takes;
   - every run reports **wrong-slide seconds**, the time a slide other than the truth slide is on screen, from the first truth row to 10 s after the last. It counts early, late and wrong slides in one audience-facing number;
   - ADR-022's held-out rule applies; the thresholds above were set on these takes.

**Results (lab, Small, 6 s encoder, the live decoder settings, 1 s passes; live4 replayed from its session log against a truth read from the heard words):**

| Take | Changes | Plan-order follower: right · wrong moves · > 1 s early · wrong-slide s | Parts map: right · wrong · early · wrong-slide s |
|---|---|---|---|
| T1 (planned order) | 11 | 10 · 2 · 1 · 5.5 | 11 · 1 · 0 · 10.7 |
| T2 (TTS, planned order) | 6 | 6 · 0 · 0 · 7.5 | 6 · 0 · 0 · 7.5 |
| F1 extra chorus | 14 | 9 · 3 · 2 · 49.0 | 12 · 0 · 0 · 21.8 |
| F2 skipped verse | 8 | 4 · 5 · 1 · 63.0 | 8 · 1 · 0 · 13.7 |
| F3 chorus first | 10 | 0 · 13 · 2 · 140.7 | 10 · 1 · 0 · 25.1 |
| F4 last line repeated | 12 | 9 · 3 · 3 · 37.6 | 12 · 0 · 0 · 15.2 |
| live4 (live test) | 16 | 11 · 5 · 5 · 24.1 | 15 · 1 · 1 · 17.6 |
| **Total** | **77** | **49 · 31 · 14 · 327** | **74 · 4 · 1 · 112** |
| P1 (band, no labels) | 11 | 11 · 0 · 0 · 45.9 | 8 · 1 · 0 · 66.0 (§ 6: keeps plan order) |

**The cost:** when the leader keeps the plan, the verse after a chorus appears 1–3 s after its first word (T1: 2.2 and 1.3 s; live4: 2.8, 2.6 and 2.6 s), not on the breath. And a slide whose last words Whisper missed now waits for the next line's words instead of a pace guess (T1 verse 2, slide 2: 3.7 s late; live4 verse 1, slide 2: 2.6 s late, where dead-reckoning had moved it 3.4 s early). That is why T1's wrong-slide time rose from 5.5 s to 10.7 s. `boundary = plan` puts this back for plan-keeping leaders, but leaves the plan badly: F1 13 · 4 wrong, F4 11 · 4 wrong, live4 13 · 3 wrong.

## Alternatives Considered
| Alternative | Pros | Cons | Why Not |
|---|---|---|---|
| B: the plan plus common detours (chorus again, bridge, tag) | simpler | still fails a skipped verse or a chorus first (F2, F3); the general map was no harder to build | the founder chose A |
| C: the plan runs itself and detours are suggested to the operator | safest for volunteers | not hands-free; the operator must watch | may sit on top of A as an operator setting "ask before leaving the plan" (Run screen, V14) |
| Onset at every part end (`boundary = plan`) | fastest when the plan is kept | wrong moves on every detour (F1 4, F4 4, live4 3) | an option, off by default |
| Keep dead-reckoning | P1 11 of 11 | 2 of live4's early moves | kept only for songs without labels (§ 6) |
| A full position model (an HMM or Viterbi over every word) | principled | hard to explain in a numbers-only log line; gates are harder to state | revisit if the corpus shows the branch model's limits |
| Wait for words at every part end, without elimination | never guesses | verse 2 on T1 came 11 s late (Whisper misheard its first line) | elimination with review |

## Consequences
### Positive
- voxxa can follow flowing worship: an extra chorus, a skipped verse, a chorus first, a repeated line.
- Far fewer wrong slides on screen: 31 → 4 wrong moves and 327 → 112 wrong-slide seconds over the 7 sung takes.
- No slide moves on pace alone; every move is explainable from the log line (`why`, `part`, `ptr`).
### Negative
- After a chorus, the next verse comes 1–3 s late when the plan is kept; a slide whose last words Whisper missed waits for words (no pace guess).
- Needs part labels; unlabelled songs keep the old follower and its dead-reckoning.
- Leaving the plan needs a presentation app that can jump, or Left presses that flash the slides in between (FA-44).
- More gates in the follower row (ADR-028 budget: V11 measures the port).
### Risks
- Tuned on one hymn with a highly repetitive chorus (T1 and its re-cuts F1–F4, live4, T2) and one band song. The re-cuts share T1's singer and room. Mitigation: the held-out corpus (V6), new founder takes with real detours (V3d), and the thresholds named as config.
- Elimination picks the planned part when the leader skips one and Whisper mishears both. The review corrects it (F2: verse 3 shown 10 s late, 1 wrong move).
- A spoken aside after a chorus ("let's sing that again") can trigger elimination. The review and the two-window gate correct it; the ADR-022 talk rules still apply.

## Dependencies
- Amends: ADR-022 § 2, § 4, § 6; ADR-030 § 1, § 8.
- Depends on: ADR-016 (sections), ADR-022 § 7 (the importer fills sections), ADR-030 (the onset detector), ADR-019 § 2 (front-window guard), ADR-013 (replay).
- Constrains: ADR-023 (practice memory learns detour habits and may set `boundary`), ADR-024 (service-plan arrangements give the plan), FA-44 (the jump driver; the FreeShow wire format W1).
- Plan steps: V3d (new), V3b, V11, V14.

## Amendment 1 (draft, 2026-11-26, round 2 review; `round2/50-adjudication.md` R3, R12, R14, C5, C8, C9)

Status of this amendment: Proposed with the ADR (FA-43, FA-63). Nothing above is removed; these points correct how the
ADR's evidence is read and add one optional switch.

1. **live4 is not evidence for or against jumps.** live4 ran the plan-order follower, the founder sang the planned
   order, and Left was never attempted (C9). Its three early moves after the replay were onset fires, each after a
   break by construction, and were in-plan moves, not jumps (C5). The parts map has never run live.
2. **live4's word-driven delays are optimistic.** The replay stamps word moves with voxxad's audio time; `live.py`
   stamped them at read time, about one pass later. Word-driven delays in the live4 row are optimistic by about
   0.5–1.2 s; onset moves are unaffected (R14, measured on the one comparable move). Every offline-vs-live comparison
   carries this bias until `live.py` uses voxxad's `t`. FA-45 (tap live4's truth) comes before this ADR cites live4
   again.
3. **Break gate: an option, measured before it is believed (R3, FA-63).** A lab switch in plan V3d task 3: a jump is
   accepted on one agreeing window when its alignment starts within about 1 s after an onset gap ≥ 300 ms, and on
   two windows otherwise. It is measured before/after on F1–F4, the real detour takes and the spoken-aside take, never
   on live4. In V11 it ships as `PartsConfig.break_gate`, **on by default** (FA-63 as answered 2026-11-27, a deviation from
   this draft); the before/after table still goes in the record, and a measured loss on the detour takes turns the
   default back off by a dated amendment with the numbers. (Amendment 4: measured; on by default only together with
   `head_min_words` = 2.)
4. **`boundary = plan` is not a per-leader fix (C8).** Measured worse even on T1, where the plan was kept (9/11 right,
   3 wrong, 2 early, 24.2 wrong-slide s, against 11/11, 1, 0, 10.7 by default; over the 7 sung takes 68/19/9/223
   against 74/4/1/112). Practice memory never turns it on (FA-64); learned handicaps come only after ≥ 3 confirmed
   runs per leader.
5. **Lab must-fixes before V3d's live test (R14), each with a test:**
   - the planned-next part is entered from mid-part on one window and relabelled `jump` after the gates
     (`parts.py:195, 277-279, 304-323`). Gate it like any other leave-the-plan move unless the current part's end was
     heard. Also stop the relabelled jump from rewinding the plan pointer inside a review window;
   - elimination has no talk gate (a spoken sentence after a chorus shows the next verse). `elim` goes behind a lab
     flag that the live test may turn off, and a spoken-aside take joins V3d task 4;
   - `live.py` stamps windows at read time, not voxxad's audio time (`live.py:310-312`); it is to use voxxad's `t`;
   - the mic queue is unbounded (`live.py:123`) and a stall is invisible live; show the queue depth in the status
     line;
   - pytest for the lab (`test_parts.py`, `test_onset.py`), written from the ADR texts;
   - also: a two-singer take, the OLTW arm (plan V3f), and timing columns within 0.5/1.0/1.5 s per take.
6. **Port shape (R12).** V11 ports one module, `aligner/follower/parts.rs`, with a `dr_show` flag for one-part songs
   (replacing § 6's "keep ADR-022's plan-order follower" as separate code). It replaces the `aligner/evidence/`
   accumulator under one config switch rather than stacking a third follower. Its own budget rows (onset 750, split
   from the follower row; the 250 estimate × 3 under FA-54 as answered 2026-11-27) come from the ADR-028 amendment in V7 (FA-54).
7. **Logging (R13, FA-65).** § 8's `why` values travel in the one new `trace` line type (with a `kind` field), not a
   new `follow` line.
8. **Jumps (FA-44, FA-47, ADR-032).** § 7's driver choice is answered: FreeShow (Tier A, jump plus read-back) is the
   autonomy target; on Presenter (Tier B) off-plan moves are suggestions, and Left bursts are refused in code.

## Review trigger
V3d's founder takes with real detours fail its bars; a wrong slide shown by a jump, elimination or review correction on a live test; the corpus shows that leaders keep the plan (then `boundary = plan` by default for them); FA-44's driver changes what a jump costs.

## Amendment 2 (draft, 2026-11-26, founder answer FA-43)

**Decision recorded.** The founder approved ADR-031 as the direction for V3d (option a, waiting for words after a chorus by
default), with amendment 1 (the break gate as an option, default `None`), and added one requirement: the framework traces
song parts from metadata and applies the parts-map follower on **both Linux and Windows booth PCs**.

**Parts from metadata (already the design; stated here).** Parts come from the setlist's `sections` (kind plus slides),
filled by the `.txt` importer (ADR-016, ADR-022 § 7, plan V5), from FreeShow groups on import (plan V17) and from practice
files (plan V12). No lyrics leave the private folders; the follower sees slide ids and part labels.

**Operating systems.** The follower, the parts map and `voxxad` are Rust and OS-neutral and already build on the three-OS CI
matrix. The Tier A FreeShow driver (ADR-032) is HTTP and OS-neutral. The Tier B keystroke driver (WorshipTools Presenter)
is Windows-only, and plan Q26 disables keystrokes under Wayland; so a Linux booth is Tier A only. The kit's "Windows booth
only" line (plan Q24/Q26) is reworded in V17 once FA-73 is answered. Recorded as **FA-73** in `founder-action-items.md`
with the recommendation: Linux is Tier A only, verified by one Linux smoke run of `voxxad` and the follower tests before
V11 closes; no new row.

**FA-73 answered 2026-11-27: (c), Linux after Easter.** Nothing Linux-specific before the FreeShow switch; the three-OS CI
stays as it is; the Linux smoke run leaves V11; the requirement is deferred, not dropped.

**Dependencies added.** FA-73; plan V11 (the driver interface names Tier A as the only autonomy path on Linux); V17 (kit
wording).

## Amendment 3 (draft, 2026-11-27, founder answer FA-61: a manual move re-anchors, never demotes)

**Gap closed.** This ADR never mentioned manual moves and the lab parts map has no entry point for one; the plan's V14
rule was "demote the song". The founder's answer is the opposite: voxxa keeps going from wherever the volunteer put
the slide, and the badge shows that voxxa is live. Spec § 4.3's row "Operator wins: any operator move re-anchors the
follower there" was already the stated intent.

**What is added.**
1. **`reanchor(slide, t)`** on the parts map (lab `parts.py`, then `aligner/follower/parts.rs` in V11): set the
   position to (slide, word 0); re-derive the plan pointer with the existing `_enter` rules (planned next → ptr+1;
   chorus-like → keep; otherwise forward search, then backward); end any review window; clear `boundary`; send
   nothing for `manual_grace_ms` (2,000 ms, a first guess, measured on V3d's detour takes).
2. **Evidence before `t` is dropped.** Grace alone cannot stop a repeat of the wrong move: the words that drove the
   jump are still in the alignment window after 2 s, and with CONFIRM = 2 the follower could jump straight back to
   where the operator just corrected it from. So the leave-plan margin is measured only on words heard after the
   correction.
3. **Tier B.** On Presenter a manual move is seen only through voxxa's own hotkeys (spec § 4.7). A press on
   Presenter's own keyboard leaves the follower one or more slides off and nothing corrects it; since suggest-only
   sends no keys there, the cost is a wrong suggestion card, not a wrong screen. ADR-034 § 25 carries the same line.
4. **Lab tests (join the R14 pytest list, V3d task 3):** `reanchor_blocks_repeat_of_the_corrected_jump` (a scripted
   wrong jump, a re-anchor to the true slide, the same words replayed → no second jump inside the grace or after
   it) and `reanchor_onto_repeated_chorus_picks_next_planned_instance` (a re-anchor onto a repeated chorus picks the
   next planned instance, not the first). V11 ports both.
5. The old rule survives as `demote_on_manual_move: false`, a switch for a church that wants it.

**Dependencies added.** FA-61 (answered); V3d task 3 (the hook and tests in the lab, once the founder says go on the
lab edits); V11 task 1 (the port); V14 task 1c (the UI rule); ADR-034 § 25; spec § 4.3 and § 4.7.

**Lab status and a measured correction (2026-11-27, `reviews/2026-11-27-lab-r14-reanchor-lag.md`).** The hook, the R14
fixes and 18 pytests are done in the private lab. Two details for the port: (a) windows carry no word times, so the lab
approximates "drop words stamped before t" with a one-window-span cut (what a later window shares with the last
window before the move is cut from its front); the port uses word times if the recognizer gives them, else the same
cut; (b) a `words` move inside a guessed part ends the review (the guess is confirmed), otherwise leaving the part
early rewound the plan pointer (plan V1 C V2 C: pointer 1 instead of 3). **Measured correction:** the live read lag is
2,148 ms median (p95 2,183) against the 1,000 ms the offline replays assume, so every offline number in this ADR is
optimistic by about 1.15 s; at `--lag-ms 2150` the parts map's wrong-slide seconds over the six sung takes go 94.0 →
160.4 (the plan-order follower: 322.4). The bars do not move; the lag itself becomes V3b task 8 (the breakdown row).

## Amendment 4 (draft, 2026-11-27, lab result for FA-63: the break gate ships on only with the head-word rule)

**Measured (lab, `reviews/2026-11-27-lab-v3d-break-gate.md`).** Seven sung takes, wrong / early > 1 s / wrong-slide seconds. At
the offline lag of 1,000 ms: base 4 / 0 / 161.0; gate alone 4 / 0 / 152.0; gate + head rule 4 / 0 / 153.0. At the measured
live lag of 2,150 ms: base 6 / 1 / 235.8; **gate alone 8 / 1 / 229.8 (a loss: T1 wrong 2 → 4)**; head rule alone 4 / 0 /
223.3; **gate + head rule 4 / 0 / 214.3 (a win)**. Root cause of the loss: an alignment running past a chorus end picked up one
stray word of the next verse's head and entered it 8.6 s early; the gate then sped up the back-and-forth.

**Decision.**
1. **Head-word rule.** `PartsConfig.head_min_words` (default **2**): on a branch path, a candidate inside a new part needs at
   least this many matched words in a head-only alignment before the part is entered. A slide moves on words of that slide.
   This is a correctness fix in its own right and stands whether or not the gate is on. 3 measured identical to 2.
2. **Break gate on by default, only together with rule 1.** FA-63's answer ("build it on by default") stands; its flip-back
   clause is satisfied because the combination wins at both lags, while the gate alone is recorded as a loss at the live lag
   and never ships without rule 1. Gate parameters from the lab: grace 1,000 ms, gap ≥ 300 ms, the jumped-to words' start
   bracketed at 1.5–3.5 words/s when word times are absent (the port uses word times if the recognizer gives them).
3. **Both lags in every table.** V3d's before/after tables, and V11's acceptance, report `--lag-ms 1000` and `2150`; a
   change that wins at 1,000 and loses at 2,150 is a loss.
4. **Tests for the port** (from the lab's 21): `break_gate_accepts_a_jump_on_one_window_after_a_break`,
   `break_gate_ignores_a_break_long_before_the_new_words`, `a_new_part_is_not_entered_on_one_stray_word`.
5. **OLTW arm deferred** until the founder's FA-45 solo takes give fair reference pairs (F1–F4 are re-cuts of T1); R5 and
   ADR-033 unchanged.

**Dependencies added.** V3d task 3 (done in the lab); V11 task 1 and 2b (`head_min_words`, both lags); spec § 4.3.
