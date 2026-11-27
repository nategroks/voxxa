# ADR-022: A score-following follower, off by default until it passes on held-out songs, with safety gates and a two-number "95%"

**Status:** Proposed (2026-11-24) · **Date:** 2026-11-24 · **Feature:** church-autoadvance · **Reversibility:** Easy while the default is `None` (one config field); Moderate after the default flips (re-recorded public logs) · **Owner:** nategroks · **Author:** 10x-Team synthesiser, from drafts 10 and 14, reviews 20 and 21

Extends ADR-009 (position tracking) and ADR-016 (its minimum). ADR-009's escalation path (acoustic alignment) stays.

## Context
- Today's conductor scores each new transcript against whole songs with a character `partial_ratio`; it starts a song at 85, advances at 70, goes to `InterVerseSilence` after 3 s of silence and blanks after 5 s more (`src-tauri/src/aligner/mod.rs:85-93`, `:354-363`). On private song P1 it hid the lyrics twice in normal vocal gaps and reached 4 of 11 slides (Small/Medium); Tiny never started (83.3 < 85) (brief § 4).
- A private prototype "follower" (word alignment over the setlist as a sequence of slide occurrences, never backward, pace penalty, learned pace, dead-reckoning) reached 11/11 with 0 wrong moves on Small and Medium at one fixed setting, but 6/11 with 8 wrong moves on Small with a 30 s encoder (brief § 6). One song; tuned over 768 settings.
- The scorer counts a goto as a hit until 1 s before the next slide (`src-tauri/src/cli/voxxa-eval/score.rs:49-51`). With a 3.0 s median delay, at most 5 of 11 hits fall within the strategy's 1.5 s bar (`.10x/specs/2026-10-24-software-strategy.md:26`): "11/11" is recall, not "on time".
- The strategy's fail-safe rule: no command after the last good transcript is older than 8 s; operator override within 100 ms (`software-strategy.md:28`).
- Committed session logs replay byte for byte (ADR-013); a default change re-records the public logs.
- A partial key sequence leaves dispatch's position stale (`src-tauri/src/dispatch.rs:163-175`): fixed under ADR-025 before this ships.
- The `.txt` importer, the path for hand-typed songs (ADR-026), recognises section label lines but drops them and writes `sections: Vec::new()` (`src-tauri/src/importers/txt.rs:69`). The setlist model already carries optional `sections` (`src-tauri/src/aligner/mod.rs:30-31`, ADR-016). `Slide` has only `id` and `text` (`aligner/mod.rs:19-22`), so no slide can be marked as a credit slide today.

## Decision
1. **The definition of "95%"** is two published numbers plus move speed (spec § 2.2): **Correct** (recall ≥ 0.95, wrong moves ≤ 1 per 10 songs, 0 lyrics hidden mid-song, 0 shown during talk) and **On time** (share of non-song-start occurrences shown within 1.5 s of the first sung word; song starts reported separately; target set after V6b). Move speed: decision → command p95 ≤ 100 ms. Evaluation rules: spec § 2.3 (held-out chosen by hand once and frozen, one frozen setting, live-fidelity hop dropping, a neighbour configuration).
2. **Port the prototype as measured**, into `src-tauri/src/aligner/follower/` (or into `aligner/evidence/` if the port is ≤ 250 lines), behind `SmartConfig.follower: Option<FollowerConfig>` with `#[serde(default)]` = `None`. The conductor keeps its states, blanking API, `manual_step`, `jump_to_song` and the `Action` enum; with `Some` it routes transcripts, ticks and silence rules to the follower (≤ 20 lines in the conductor row). The follower never moves backward except along a jump edge to a section start; sections come from the importer (§ 7). A song with no labels has one section, so its only back jump is to the song start.
3. **Ship only after the V6 gate passes** on the held-out set at one frozen setting, on the live configuration from ADR-021, and after the V11 safety tests pass. The default flips in its own PR after 2 consecutive shadow Sundays meet the Correct bar, re-recording the 6 public logs.
4. **Safety gates (each one a named test):**
   - *Margin:* act only when the best alignment beats the second by `Δ`; off-plan moves need a stronger bar or 2 agreeing windows; otherwise hold.
   - *Lost:* no anchor for > 20 s while singing → keep the slide, amber, never blank.
   - *One hide rule:* a song ends only when its planned occurrences are complete and (talk evidence ≥ 4 s or the learned outro has elapsed); otherwise hold; cold minimum 10 s. The 3 s/5 s rule remains only when the follower is `None`.
   - *Song start:* ≥ 4 matched words and 2 consecutive windows agree; no lowered bar for the planned next song in v1; during a spoken layout item, a song start is a suggestion.
   - *Dead-reckoning:* ≤ 1 step per anchor; never across songs; never the first slide of a song; anchor ≤ 8 s old; ≥ 2 on-score tokens in the last 6 s; never during spoken or media items.
   - *Fire-time re-check:* any scheduled move re-checks Pause, Off, the 5 s operator hold, the watchdog and the gate when it fires; Pause, Off and any operator action cancel pending moves.
   - *Watchdog:* no automatic move when the newest decoded window is > 8 s old or decode lag > 2 hops.
   - *Operator wins:* an operator move re-anchors the follower.
   - *Credit slides:* kind `credit` (set by the `.txt` importer, § 7) is never matched and is stepped past at song end.
   - *WorshipTools song end:* no early Right into the next song's blank in v1 (the #184 song change stays as is).
5. **Prediction is off in v1.** It may be enabled by an amendment only after V6b shows early hides ≤ 0.4% per prediction (the budget for ≤ 1 per 10 songs at ~24 predictions per song), firing at the 99.6th percentile of practice residuals, with on-time share +20 points and no added wrong moves.
6. **Logging:** a numbers-only `follow` line (`occ`, `word`, `m`, `A`, `d`, `r`, `gate`, `why` ∈ anchor | dr | jump | hold | lost | watchdog | operator); no words.
7. **Section labels and credit slides come from the `.txt` importer (plan V5):**
   - the importer keeps section label lines as `sections` entries (label plus first slide index, held in the existing ADR-016 `Section` type as `kind` and `slides`, `src-tauri/src/aligner/evidence/sections.rs:25-27`) instead of `Vec::new()` (`src-tauri/src/importers/txt.rs:69`);
   - a slide whose first line starts with `Credit:` gets `kind: "credit"`, a new optional `Slide` field with `#[serde(default)]`, not written when absent (like `sections`, `aligner/mod.rs:30`), so old setlists and logs replay byte for byte;
   - tests: `txt_section_labels_fill_sections`, `credit_slide_kind_detected`, `old_setlists_replay_unchanged`;
   - the jump edges (§ 2), the credit gate (§ 4), `voxxad --learn`'s section labels (ADR-023 § 4) and service-plan arrangements (ADR-024 § 1) rely on it.

## Alternatives Considered
| Alternative | Pros | Cons | Why Not |
|---|---|---|---|
| Tune today's conductor (start bar, silence timers) | small diff | whole-song character scores cannot tell repeated choruses apart; Tiny fails the start bar | measured worse than the prototype on P1 |
| Draft 10's full design now (new 700-line row, time-based prediction, pitch-DSP singing gate, heard signatures, `Input::Window`) | aims at "on time" | untested pieces; size unknown; the strategy's 3k core bound | port what was measured; add pieces only when an experiment shows a gain |
| Port as-is with no new gates and keep 3 s/5 s for cold songs (draft 14 V4) | smallest | the measured failure was the mid-song hide; P1 recall stays ≤ 0.44 without the gap hold | the gap hold ships with the port |
| Single "95%" number (SA within +1.0 s, draft 10) | one headline | song starts can never meet +1.0 s; no budget left for the rest | report correct and on time separately |
| Hash-based held-out split (draft 10) | no human choice | at ~10 songs about a 10% chance of ≤ 1 held-out song | chosen by hand once, stratified, before tuning |

## Consequences
### Positive
- Fixes the measured failure (mid-song hides) while keeping old logs replayable.
- Every automatic move is explainable from a numbers-only log line.
- Safety rules are tests, not intentions.
### Negative
- The follower cannot ship before the founder's corpus exists (about 3 Sundays).
- Song starts stay about 2 s late by construction; "on time" depends on a later experiment.
- Two code paths (follower on/off) until the default flips.
### Risks
- Overfitting to the tune set: mitigated by the frozen held-out set and the neighbour configuration.
- The S-24 dead-reckoning gate may remove the DR that made P1 perfect; V6 measures DR availability, and the founder decides if recall drops.
- Song detection may still fire on a spoken quote; the TTS test gates it, and the pitch gate is the fallback.

## Dependencies
- Depends on: ADR-021 (live configuration), ADR-025 (dispatch fixes), ADR-028 (budget row), ADR-015 (eval), ADR-013 (replay).
- Constrains: ADR-023 (practice memory feeds the plan and pace), ADR-024 (item pointer, arming).
- Plan steps: V2, V5 (importer sections and credit kind), V6, V6b, V11.

## Review trigger
V6 fails its bar (then: pitch gate, signatures or acoustic alignment per ADR-009); a wrong-slide report from a live Sunday; V6b passes (amend to enable prediction); the sherpa-onnx bake-off (Q21) changes the recognizer.
