# V3d port plan (for V11): the parts map as one module, `aligner/follower/parts.rs`

Reported by the lab session on 2026-11-27 from the private lab; the orchestrator records it. Numbers and names only:
take ids, lab identifiers and lab line numbers (private files at this date), no lab text. Every number is (measured)
unless marked [est.].

## 1. Result in brief

- **Follower row: about 860 Rust lines [est.], range 765–955.** The size factor was measured, not assumed: 10 lab
  functions (104 lab code lines) were written as Rust that compiles, formatted with rustfmt defaults (the repo has no
  `rustfmt.toml`). They count 246 tokei code lines, a factor of 2.37. The estimate uses 2.3 for logic, plus a
  structural count for config and trace types. The scratch file stays in the lab session and is never committed.
- **This is over the 500 ceiling.** As FA-74 (b) and the round-3 prompt say, the proposal is to ask for **3 × the
  measured port** at V11's Phase 2 check: about 2,580 at the estimate. **This collides with ADR-028 amendment 1
  § 6**: with that row, the core-engine sum in caps would be 1,200 + 500 + 2,580 + 300 = 4,580, against the 3,000
  bound that § 6 says "stays". This needs a founder line (§ 10).
- **Reuse, measured on the corpus:**
  - The conductor's LCS ratio (`simple_ratio`/`lcs_length`) makes the same decisions as the lab's difflib ratio at
    both thresholds:
    - 0 flips in 91,903 heard-word × slide-word pairs at 0.75;
    - 0 flips in 208 slide pairs at 0.7 (20 look-alike pairs either way).
  - `evidence::tokens` gives the lab's token lists on 26 of 26 distinct slides.
  - Making `simple_ratio` `pub(crate)` costs 0 lines.
- **Five findings for V11:**
  1. **`dr_show` has no lab implementation.** `DR_SHOW` is defined at `parts.py:38` and never read. So "a one-part
     song with `dr_show` reproduces the plan-order follower" is unverified. P1 run as one part without
     dead-reckoning scores 8/11, 66.0 s at lag 1,000 and 74.4 s at lag 2,150. The plan-order follower scores 11/11,
     45.9 s at lag 1,000 (ADR-031).
  2. **`scripts/loc_budget.py` has no follower row.** `conductor = total(["aligner/"]) - evidence - onset`, so
     `aligner/follower/` counts against the conductor row (27 left) until V11 adds the row.
  3. **`Conductor::apply(input, now)` carries no audio time.** `Input::Heard(String)` has text only, and `now` is
     the arrival time, so the lab's measured 2.15 s read lag would be built into the port (R14).
  4. **`SectionKind` is Verse, Chorus (alias refrain), Bridge, Tag, Other.** ADR-031 § 1's chorus-like list also
     has pre-chorus and ending, and the lab adds outro and vamp. Today those parse as `Other`, so they count as
     chorus-like only when they repeat in the plan.
  5. **There are two slide-id spaces.** The map matches words against unique slides. A driver moves to a
     presentation index, where a repeated chorus appears more than once. The lab works and scores in unique slides.
     The port emits the presentation index of the planned occurrence (`occ`, from `ptr`). The Rust `Song` has
     `slides` and `sections` but no plan order (§ 5).
- **Two conflicts in the V11 prompt wording:**
  - The test name `break_gate_off_by_default_changes_nothing` contradicts FA-63's on-by-default.
  - `BreakGate { grace_ms, delta }` has no `delta` in the lab. The gate lowers CONFIRM from 2 to 1; `M_OFF` and the
    look-alike rule are unchanged (§ 4).

## 2. Lab line counts (tokei 15, private lab, 2026-11-27)

| File | Code | Comments | Blanks | Role |
|---|---|---|---|---|
| `parts.py` | 427 | 16 | 26 | the parts map: ports |
| `predict.py` | 302 | 14 | 50 | the detector and ADR-030's arm rules (V3b's plan); its dead-reckoning step (`:253-259`) ports under `dr_show` |
| `follower.py` | 128 | 0 | 16 | the plan-order prototype: only `tok` and `sim` are used by `parts.py`, and both are replaced by Rust that already exists |
| `evalparts.py` | 100 | 1 | 11 | replay and scoring: does not port |
| `flows.py` | 91 | 0 | 13 | the re-cut takes: does not port |
| `live.py` | 335 | 8 | 43 | the lab helper: does not port |
| `test_parts.py` | 145 | 6 | 47 | 15 tests: 13 port (§ 6), 2 go to V2 |
| `test_onset.py` | 38 | 2 | 18 | 6 tests: V3b's plan |

`parts.py` has 380 code lines once docstrings are left out (an `ast` count; tokei counts docstrings as code). The split
below uses that count.

| Part of `parts.py` | Lines | Code |
|---|---|---|
| `sim`, `sw` | 47–67 | 18 |
| `Map` (`__init__`, `next_in_part`) | 70–120 | 48 |
| `Parts.__init__` | 124–140 | 17 |
| `expected_next`, `_enter`, `_prior`, `_show`, `reanchor`, `_after_cut` | 143–216 | 60 |
| `_paths`, `_eliminate`, `_here_any`, `_global` | 219–277 | 50 |
| `window` | 279–380 | 96 |
| `_after_break` | 383–394 | 8 |
| `onset` | 396–428 | 31 |
| `score_ids` | 432–469 | 36 |
| module level (imports, `CHORUS_KINDS`, `PRIOR`, `CFG`) | 28–44 | 15 |

## 3. Port estimate by component (Rust lines, tokei code)

| Component | Lab source | Lab code | Destination | Rust [est.] |
|---|---|---|---|---|
| Local word alignment and word match | `parts.py:47-67`, `follower.py:46-49` | 22 | `parts.rs` | 50 |
| Tokens, string ratio | `follower.py:19-20`; difflib | 2 | reuse `evidence::tokens` and `simple_ratio` (0 flips, § 1) | 0 |
| Map: parts, plan instances (rule at `:95-98`), chorus-like, look-alikes, self-repeating slides, line starts | `parts.py:70-120` without file and truth reading | 36 | `parts.rs` | 85 |
| State and `new` | `parts.py:124-140` | 17 | `parts.rs` | 40 |
| Plan hint, show, `reanchor`, cut, break check | `parts.py:143-216`, `:383-394` | 68 | `parts.rs` | 155 |
| Candidate paths, elimination, lost | `parts.py:219-277` | 50 | `parts.rs` | 115 |
| `window` | `parts.py:279-380` | 96 | `parts.rs` | 220 |
| `onset`: part-end rules (21) / ADR-030 arm check (10) | `parts.py:396-428` | 31 | `parts.rs` 50 / `aligner/onset/` 25 | 50 |
| `dr_show` step for one-part songs | `predict.py:253-259` plus the voice gate | 8 | `parts.rs` | 20 |
| `PartsConfig`, `BreakGate`, handicaps, `Default` | `parts.py:37-44` (30 keys, 6 handicaps) | 8 | `parts.rs` | 70 (one line per field plus its default) |
| `trace` payload `Follow` and `Why` codes | new | 0 | `parts.rs` | 40 |
| `follower/mod.rs`: `FollowerConfig`, re-exports | new | 0 | `mod.rs` | 15 |
| **Follower row** | | **338** | | **≈ 860** (765–955 at 2.0–2.6 ×) |
| `score_ids` and the timing columns | `parts.py:432-469` | 36 | V2, voxxa-eval (R11) | 0 here |

Tests go in `src-tauri/tests/parts_follower.rs`. Like `conductor_evidence.rs`, that file is outside every row, so
it is uncounted. Expect about 600 lines [est.].

## 4. Config: lab key → Rust field → default

`PartsConfig` (all `#[serde(default)]`; `SmartConfig.follower: Option<FollowerConfig>` is `None` by default):

| Lab key | Field | Default |
|---|---|---|
| `MIN`, `LAM`, `RATE`, `SLACK`, `START`, `GAP` | `min_score`, `pace_penalty`, `rate_wps`, `slack_words`, `start: (score, words)`, `gap` | 3.0, 0.1, 1.5, 4, (4.0, 4), 0.7 |
| `PRIOR`: NEXT, CHORUS, SAME, OTHER, LINE, START_OTHER | `handicaps` | 0, −0.5, −1.0, −1.5, −1.0, −1.0 |
| `BOUNDARY` | `boundary: Likely \| Plan` | Likely |
| `DR_SHOW` | `dr_show` (set by the conductor for a song without sections) | false |
| `LOST_MS`, `M_OFF`, `CONFIRM`, `LINE` | `lost_ms`, `off_plan_margin`, `confirm_windows`, `repeat_line` | 10,000, 1.5, 2, true |
| `ELIM`, `ELIM_MS`, `ELIM_MAX`, `ELIM_GAP_MS`, `REVIEW_MS` | `elim`, `elim_ms`, `elim_max`, `elim_gap_ms`, `review_ms` | true, 2,000, 1.5, 300, 8,000 |
| `MANUAL_GRACE_MS`, `CUT_SPAN_MS` | `manual_grace_ms`, `cut_span_ms` (only without word times) | 2,000, 6,000 |
| `BREAK_GATE`, `BREAK_GRACE_MS`, `BREAK_GAP_MS`, `BREAK_RATES` | `break_gate: Option<BreakGate { grace_ms, min_gap_ms, rate_lo, rate_hi }>` | `Some(1,000, 300, 1.5, 3.5)` |
| `HEAD_N` | `head_min_words` | 2 (amendment 4: the gate is on only together with this) |
| constants `parts.py:103`, `:109`; `follower.py:49` | `self_repeat_words`, `self_repeat_gap`, `look_alike_ratio`, `word_match_ratio` (+1.0 exact, ±0.6) | 3, 4, 0.7, 0.75 |
| (amendment 3 § 5) | `demote_on_manual_move` | false |

`OnsetConfig` (V3b's plan, the onset row): `TAIL` 1, `ARM_W` 1, `AMBIG` true, `FIRST` onset, `FIRST_GAP_MS` 2,000,
`FIRST_AFTER_MS` 3,000.

The break event: the onset module sends `break(gap_ms)` at ≥ 200 ms (V11 task 2b), and the gate keeps only
`gap_ms ≥ min_gap_ms` (300). **Proposed:** replace `delta` with `min_gap_ms`, or define it, because the lab gate has no
other parameter.

## 5. Interfaces

- **`PartsMap::new(song, plan)`:** built from `Song.slides`, `Song.sections` and a plan order. A song without
  sections becomes one part, with `dr_show` set. **Open (V11, with V5/V17):** where the plan order comes from.
  - If `Song.slides` is already the arrangement with repeats as separate slides, the map folds identical-text
    slides into one unique slide, and the plan is the slide sequence. No new field is needed.
  - If not, `Song` needs an order field (`#[serde(default)]`, empty = slides in order). That is about 2 lines in the
    conductor row.
- **`PartsFollower`:**
  - `window(t_known_ms, t_audio_ms, words) -> Option<Move>`
  - `onset(t_known_ms, t_on_ms, gap_ms) -> Option<Move>`
  - `reanchor(t_known_ms, slide, t_audio_ms: Option<_>)`
  - `trace() -> Follow`

  `Move { slide, why }` carries voxxa's slide id, meaning the presentation index of the planned occurrence. The
  follower knows no driver. Dispatch resolves the move per tier (ADR-032 § 16–18, the wiring share ≤ 20):
  - Tier A: `goto_slide` with verify.
  - Tier B: Right for the next slide; any other move becomes a suggestion; Left bursts are refused.
- **Conductor (share ≤ 20):**
  - `SmartConfig.follower`;
  - an onset input;
  - routing `Heard` to the follower when the config is `Some`;
  - `manual_step` and position sources call `reanchor`;
  - with `Some`, the evidence accumulator is bypassed.

  The audio time per window (finding 3) has to fit in this share. Replay must stay byte-identical with `None`.
- **`trace` line, `kind: follow`.** Numbers only. The payload struct is in the follower row; the `listen.rs` and
  `events.rs` lines are as ADR-003 amendment 1 says. Proposed meanings:

  | Field | Meaning |
  |---|---|
  | `part` | the current part |
  | `ptr` | the plan pointer |
  | `occ` | the presentation index |
  | `word` | the word position in the slide |
  | `m` | the matched words of the winning path |
  | `A` | its score |
  | `d` | the margin over the slide on screen |
  | `r` | the pace in words/s |
  | `gate` | a code: none, two-window pending, look-alike, self-repeat, grace, head-word, break-accepted |
  | `why` | a code |
  | `arm` | the onset arm state |
  | `brk` | ms since the last break |

  `why` mapping from the lab's seven values to the trace codes:

  | Lab value | Trace code |
  |---|---|
  | words | anchor |
  | onset, first-onset | onset |
  | jump | jump |
  | elim | elim |
  | lost | lost |
  | manual | operator |

  The codes hold, watchdog and stopped come from ADR-022's product gates and have no lab counterpart.

## 6. Tests to port (Rust names as in the ADRs; `src-tauri/tests/parts_follower.rs`)

| Source | Names | Lab pytest |
|---|---|---|
| ADR-031, the twelve names (V3d task 1) | `off_plan_needs_two_windows` | yes |
| | `shared_line_never_moves_alone`, `no_jump_between_look_alike_slides`, `no_skip_to_look_alike_too_soon`, `no_repeat_branch_on_self_repeating_slide`, `lost_searches_forward_first`, `elimination_shows_planned_part`, `guess_corrected_within_review`, `part_entered_at_first_slide`, `onset_fires_verse_to_chorus_only`, `no_dr_move_in_song_with_parts`, `unlabelled_song_uses_plan_order` | **no**: write from the ADR text (`scripts/lab/parts_gate_cases.py` is not written either) |
| R14 | `planned_next_from_mid_part_needs_two_windows`, `elim_flag_off_shows_nothing_after_chorus` | yes |
| Amendment 3 | `reanchor_blocks_repeat_of_the_corrected_jump`, `reanchor_onto_repeated_chorus_picks_next_planned_instance`, plus the lab's `reanchor_grace_blocks_onsets_and_words`, `reanchor_ends_review_and_boundary` | yes |
| Amendment 4 | `break_gate_accepts_a_jump_on_one_window_after_a_break`, `break_gate_ignores_a_break_long_before_the_new_words`, `a_new_part_is_not_entered_on_one_stray_word` | yes |
| Lab, extra | `planned_next_at_part_end_moves_on_one_window`, `confirmed_guess_keeps_the_plan_pointer_when_the_part_is_left_early`, `one_slide_part_sung_twice_collapses_in_plan` | yes |
| V11 task 2 | `one_part_song_with_dr_show_matches_plan_order_follower` | no lab code (finding 1) |
| | `break_gate_off_by_default_changes_nothing` | **rename:** the default is on. Proposed: `break_gate_none_keeps_two_windows` |
| | `break_gate_accepts_one_window_only_within_grace_after_break` | the two amendment-4 tests cover it; keep one set of names |
| | `jump_target_is_a_slide_id_not_a_layout_index`, `keys_only_profile_turns_jump_into_suggestion`, `tier_a_jump_verifies_by_readback`, `left_burst_is_refused_in_code` | dispatch side (`tests/dispatch.rs`) |
| Not ported here | the two `score_ids` tests | V2; the six `test_onset.py` tests: V3b |

13 of the lab's 15 parts tests port as they are. The tests read `ptr`, `part` and `why` from the `trace` payload, so they
need no private accessors.

## 7. Before / after, per take, at both lags (R3; amendment 4)

Base = `BREAK_GATE` off, `HEAD_N` 0. Default = gate on, `HEAD_N` 2. A cell shows base → default where they differ.
Columns: right / changes, wrong moves, early beyond 0.3 s, early beyond 1 s, share within 0.5 / 1.0 / 1.5 s, median
|delay| ms, wrong-slide seconds.

**Lag 1,000 ms (the offline assumption)**

| Take | Right | Wrong | > 0.3 s | > 1 s | ≤ 0.5 / 1.0 / 1.5 s | Median | Wrong-slide s |
|---|---|---|---|---|---|---|---|
| T1 | 11/11 | 1 | 0 | 0 | 0.60 / 0.60 / 0.70 → 0.60 | 236 | 10.7 → 11.7 |
| T2 | 6/6 | 0 | 0 | 0 | 0.60 / 0.60 / 0.60 | 37 | 7.5 |
| P1 | 8/11 | 1 | 0 | 0 | 0.00 / 0.10 / 0.10 | 3,681 | 67.0 → 66.0 |
| F1 | 12/14 | 0 | 0 | 0 | 0.54 / 0.54 / 0.54 | 232 | 21.8 |
| F2 | 8/8 | 1 | 0 | 0 | 0.71 / 0.71 / 0.71 | 232 | 13.7 → 9.7 |
| F3 | 10/10 | 1 | 1 | 0 | 0.44 / 0.56 / 0.56 | 669 | 25.1 → 21.1 |
| F4 | 12/12 | 0 | 1 | 0 | 0.45 / 0.55 / 0.55 | 659 | 15.2 |
| **7 sung** | 67/72 | 4 | 2 | 0 | | | **161.0 → 153.0** |
| live4 (approximate truth, not citable) | 15/16 | 1 | 4 | 1 | 0.40 / 0.67 / 0.67 | 778 | 17.6 |

**Lag 2,150 ms (the measured live lag)**

| Take | Right | Wrong | > 0.3 s | > 1 s | ≤ 0.5 / 1.0 / 1.5 s | Median | Wrong-slide s |
|---|---|---|---|---|---|---|---|
| T1 | 10/11 → 11/11 | 2 → 1 | 1 → 0 | 1 → 0 | 0.50 / 0.50 / 0.50 | 2,799 | 24.1 → 19.0 |
| T2 | 6/6 | 0 | 0 | 0 | 0.00 / 0.00 / 0.60 | 1,067 | 13.9 |
| P1 | 8/11 | 1 | 0 | 0 | 0.00 / 0.00 / 0.00 | 4,139 | 75.4 → 74.4 |
| F1 | 12/14 | 0 | 0 | 0 | 0.46 / 0.46 / 0.46 | 232 | 30.4 |
| F2 | 8/8 | 1 | 0 | 0 | 0.43 / 0.43 / 0.43 | 3,069 | 21.9 → 17.9 |
| F3 | 10/10 | 2 → 1 | 1 | 0 | 0.33 / 0.44 / 0.44 | 3,069 | 42.1 → 30.7 |
| F4 | 12/12 | 0 | 1 | 0 | 0.27 / 0.36 / 0.36 | 3,256 | 28.0 |
| **7 sung** | 66/72 → 67/72 | 6 → 4 | 3 → 2 | 1 → 0 | | | **235.8 → 214.3** |
| live4 (approximate truth, not citable) | 12/16 → 13/16 | 4 → 3 | 5 | 3 | 0.33 / 0.47 / 0.47 | 2,961 | 49.6 → 43.0 |

The totals match `reviews/2026-11-27-lab-v3d-break-gate.md`. P1 is scored as one part with no dead-reckoning
(finding 1).

## 8. R14 checklist

- [x] A planned-next part entered from mid-part is gated, and the relabel to `jump` happens before the gate. A
  confirmed guess no longer rewinds the plan pointer (2 tests).
- [x] `elim` is behind a flag (1 test).
- [x] `live.py` stamps windows with voxxad's audio time and logs `vx_t_ms`, and the status line shows the mic queue
  depth. Read lag measured: median 2,148 ms, p95 2,183.
- [x] The floor warm-up is 1 s (`test_onset.py`).
- [x] Both pytest files are written from the ADR text: 21 pass.
- [x] The timing columns are in `score_ids`.
- [x] The re-anchor hook (amendment 3): 4 tests.
- [ ] The two-singer, spoken-aside and full-mix takes: founder.
- [ ] The OLTW arm: deferred until the FA-45 takes.

## 9. Equivalence and what V11 measures

- **Reference lists.** On request, the lab writes each take's moves at both lags (ms, slide id, why) as numbers-only
  JSON in the private lab's `baselines/`. The founder-run check compares them with the port's goto lists on the same
  cached windows.
- **Size.** At Phase 2, V11 counts `aligner/follower/` with tokei 15.0.0 and applies § 10.

## 10. Row proposal (R12; ADR-028 amendment 1; FA-74 (b))

| Row | Proposal |
|---|---|
| follower (`aligner/follower/`) | new in `loc_budget.py` (`follower = total(["aligner/follower/"])`, subtracted from the conductor row, plus one check line), in V11's first commit. Cap: **3 × the measured port** at Phase 2 (FA-74 (b); about 2,580 at the estimate) |
| conductor | ≤ 20 for routing (unchanged share), including the audio time and, if needed, the plan-order field |
| onset (750) | adds about 25 for ADR-030's arm check, moved from the follower side; V3b's plan counts the detector |
| evidence (500) | 0. **Exception asked:** about 4 lines in `sections.rs` for the `PreChorus` and `Ending` variants (finding 4). The V11 prompt says this row "may shrink, never grow"; it is at 215 / 500 |
| wiring | ≤ 20 for Tier A/B dispatch (the ADR-028 amendment 1 table) |
| voxxa-eval | 0 here (`score_ids` goes to V2) |

**Founder line needed (the orchestrator numbers it).** 3 × measured on the follower row takes the core-engine sum to
about 4,580 against ADR-028's 3,000 bound. The choices:

- **(a)** 3 × measured, as FA-74 (b) says for a measured overrun. The bound moves. This is the proposal.
- **(b)** 1.3 × measured: ADR-028's original row formula without the 500 ceiling. That is about 1,120, and the core
  sum is about 3,120.
- **(c)** As (b), and the evidence row retires once the follower's default becomes `Some`. The core sum is then about
  2,620.

A smaller port is not proposed (round-3 prompt).

## 11. Open items the lab can close before V11 (each is private lab work, numbers only)

1. **`dr_show` arm, about 15 lab lines.** Run P1 as one part with dead-reckoning at both lags against the plan-order
   follower. This settles finding 1 before the port depends on it.
2. **The eleven ADR-031 names without a lab pytest,** with made-up words, written from the ADR text. Pass counts
   are reported before the port.
3. **The reference move lists** for the equivalence check (§ 9).

## Addendum (same day, lab): the three items of § 11 are closed

Reported by the lab session later on 2026-11-27; numbers only; nothing committed by the lab.

1. **`dr_show` arm (finding 1 settled).** Lab `parts.py` gained a dead-reckoning step for one-part songs only
   (11 lab code lines against the 8 estimated; about +5 Rust on the port estimate): a window with fewer than 3 matched
   words, or with no candidate, goes to the step; the step needs voice over the last 2 s and moves to the next slide
   when the pace times the elapsed time reaches the words left in the slide. No default number changed. P1, the only
   one-part take:

   | | lag 1,000 | lag 2,150 |
   |---|---|---|
   | parts map with `dr_show` | 11/11, 0 wrong, 0 early > 1 s, median 4,218 ms, 45.9 wrong-slide s | 11/11, 0 wrong, median 4,947 ms, 56.6 s |
   | plan-order follower | the same | the same |
   | parts map without dead-reckoning | 8/11, 1 wrong, 66.0 s | 8/11, 1 wrong, 74.4 s |

   The move lists are **identical** to the plan-order follower's at both lags (11 of 11 moves, same time and slide). On
   the seven labelled takes `dr_show` changes nothing (identical move lists with and without it). With P1 scored under
   `dr_show`, the seven-take default totals become **132.9 s** at lag 1,000 and **196.5 s** at lag 2,150 (§ 7 scored P1
   without it: 153.0 / 214.3). Caveat: one take of 11 moves plus one synthetic test; the held-out V6 corpus stays the
   real check. **For V11:** the `trace` `why` list has no dead-reckoning code, so these moves would log as `anchor`; the
   code `dr` is added (prompt pack V11 task 4).
2. **The eleven ADR-031 pytests** are written from the ADR text with made-up words, plus
   `one_part_song_with_dr_show_matches_plan_order_follower` (a synthetic one-part song with 8 s of missed words;
   identical moves to the lab's predictive follower, including one `dr` step). Lab pytest: **33 of 33 pass** (27 parts,
   6 onset). Mutation check: 13 rules removed one at a time in a scratch copy, **13 of 13 caught**, each by its named
   test. Four first drafts passed through another gate and caught nothing; they were fixed with control cases before
   the count. tokei after: `parts.py` 448 (+21), `test_parts.py` 291 (+146).

   | Rule removed | Caught by |
   |---|---|
   | family check | `no_jump_between_look_alike_slides` |
   | skip-to | `no_skip_to_look_alike_too_soon` |
   | line branch on a self-repeating slide | `no_repeat_branch_on_self_repeating_slide` |
   | part-again on a self-repeating slide | `no_repeat_branch_on_self_repeating_slide` |
   | lost forward-first | `lost_searches_forward_first` |
   | `M_OFF` | `shared_line_never_moves_alone` |
   | review pointer restore | `guess_corrected_within_review` |
   | review reach | `guess_corrected_within_review` |
   | dead-reckoning in songs with parts | `no_dr_move_in_song_with_parts` |
   | no dead-reckoning at all | `no_dr_move_in_song_with_parts` and `one_part_song_with_dr_show_matches_plan_order_follower` |
   | onset at every part end | `onset_fires_verse_to_chorus_only` (and 3 others) |
   | elimination before `ELIM_MS` | `elimination_shows_planned_part` |
   | head-only part entry | `part_entered_at_first_slide` |

3. **Reference move lists** for the equivalence check (§ 9) exist in the private lab's `baselines/` (one JSON file,
   20,871 bytes): 8 takes × lags 1,000 and 2,150; configurations default, gate off with `HEAD_N` 2, base, and `dr_show`
   for P1; moves as `[t_known_ms, slide, why]` plus each take's onset events as `[t_on_ms, t_known_ms, gap_ms]`, so a
   harness can feed identical inputs; the input recipe is in the file; scan 0 hits. Never committed (layout § 4).

With this, every lab input V11 names exists except the OLTW arm (deferred until the FA-45 takes) and FA-78's answer.
