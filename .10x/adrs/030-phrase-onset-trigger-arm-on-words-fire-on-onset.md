# ADR-030: Phrase-onset trigger: arm on words, fire on the onset

**Status:** Proposed (2026-11-26) · **Date:** 2026-11-26 · **Feature:** church-autoadvance · **Reversibility:** Easy (one follower config field, default off; the lab helper and the env switches are local only and never pushed) · **Owner:** nategroks · **Author:** 10x-Team, from the onset lab (takes T1, T2, P1)

Amends ADR-022 § 5 ("Prediction is off in v1"). Time-based prediction from practice stays off. The onset trigger is a different, event-driven mechanism: it reacts to the real start of the next line, armed only by heard words. It adds gates to ADR-022 § 4 and a `why` value to ADR-022 § 6.

## Context
- The word follower (private prototype, one fixed setting) places the singer from Whisper windows. Those words arrive 1–3 s after the audio. At 1 s passes the median delay of every slide after the first was 1.90 s on T1 and 2.12 s on T2. Words alone cannot reach "on time".
- ADR-022 § 5 keeps time-based prediction off until V6b shows early hides ≤ 0.4% per prediction. That route needs practice memory and the corpus, and it fires on a guess of time.
- A new line starts with a rise in vocal energy after a breath. A causal detector sees it within tens of ms.
- **Takes (lab, numbers only):** T1 = the founder singing It Is Well (public domain) a cappella into a vocal mic, 11 slide changes, truth tapped on a replay with reaction-lag calibration (524 ms). T2 = the public text-to-speech It Is Well fixture, 6 changes, exact truth. P1 = a private church song sung over a full band (lyric-video audio), 11 changes.
- **Detector vs slide starts (within ±300 ms):** T1 10 of 11; T2 5 of 6 (5 within 30 ms); P1 2 of 11 (the band fills the breaths).
- **Results at 1 s Whisper passes** (live fidelity: results arrive 1 s after their audio; overlapping passes dropped), Small model, 6 s encoder, median |delay| of every slide after the first, before → after:

| Take | Median delay | Within ±300 ms | Notes |
|---|---|---|---|
| T1 | 1.90 s → 0.18 s | 10% → 80% | 7 changes fired by onset; no wrong slide shown; two early moves: one dead-reckoning move across a 10 s pause before any onset logic, one at −0.26 s |
| T2 | 2.12 s → 0.04 s | 20% → 60% | no early fires; 2 changes leaving a refrain whose closing words repeat inside the slide fall back to words |
| P1 | 4.30 s → 4.01 s | not reported | 11 of 11 both ways; no harm |

- **At 2.6 s passes:** T1 3.28 → 2.66 s; T2 3.84 → 3.72 s; P1 6.40 → 4.36 s and 9/11 → 10/11. The trigger needs about 1 s passes to pay off. At slow passes the words most likely arm a slide after the next line has started, so no onset is left to fire on [UNVERIFIED].
- **Speed (GTX 960 2 GB, Vulkan build, Presenter running):** Small 0.85 s/pass on TTS but 2.57 s/pass on sung audio. VRAM 814 of 2048 MB with Presenter open, so Presenter is not the cause. Base 1.13 s/pass, but the follower dropped to 5 of 11 on T1 (Small 9 of 11). Cause found: whisper.cpp's temperature fallback re-decoded sung windows (up to 5 times). Lab, with the local-only switches: `temperature_inc` 0 gave 0.94 s/pass; adding `max_tokens` 48 gave 0.53 s/pass (2.7× real time), with the follower unchanged on T1 (9 of 11, same median). voxxa also creates a new Whisper state per window (V3 `reuse_state`).
- **Live dry run (2026-11-26):** the private helper replayed T1 at real speed through voxxad's local-only streaming input, with the 0.53 s/pass setting: median 0.18 s after the first slide, 70% within ±300 ms, 7 onset fires, no wrong slide shown. A 20 s run on the live mic (dry run, no keys) showed no dropouts and no false fire in room noise.
- **Audio feed:** the booth feed was a WING output on an unbalanced cable into the PC mic jack (retasked as Line In). Voice was on the left channel only. The floor was −60 dBFS, mostly 60 Hz ground-loop hum (60/180/300/420 Hz). Raw singing at about −34 dBFS made the Silero VAD (enter probability 0.60) skip almost everything: 2 Whisper passes in 200 s. The same take +12 dB gave 152 passes.
- **Whisper prompt today:** `voxxad --bias` passes slide text as Whisper's `initial_prompt`. It is off by default because it may make Whisper hallucinate (ADR-004, E6). The text is the conductor's current and next slide, so it follows the conductor, which reached 2 of 11 on T1.
- **Local only:** the lab used the `VOXXA_TEMP_INC`, `VOXXA_MAX_TOKENS` and `VOXXA_AUDIO_CTX` env switches and voxxad streaming stdin (`--stdin-audio - --realtime`). These are local experiments in voxxa-private builds, never pushed. Plan V3 writes the decoder settings fresh.

## Decision
1. **Mechanism.** The word follower stays the only judge of *which* slide. The onset detector decides *when*.
   - *Detector (causal, on the audio):* 300–4000 Hz band energy in 10 ms frames. A floor follows quiet moments down at once and rises at most 2 dB/s. An onset is a level above floor + 10 dB for 60 ms, after at least 200 ms below floor + 5 dB. The floor starts from the median of the first second of real audio (1 s; a device's first blocks can be near-silent while it warms up; amended 2026-11-26, round 2, to match the lab code).
   - *ARM:* once the words show the singer within the last 2 words of the slide on screen, and ADR-022's margin gate holds.
   - *FIRE:* the next phrase onset shows the next slide (one step, never backward, never across songs).
   - *No onset:* words move the slide as before.
2. **Five failure modes found in the lab, each fixed by a gate (each gate a named test):**

| # | Failure found | Fix | Test |
|---|---|---|---|
| 1 | arming anywhere in the last line fired 3–5 s early on breaths inside the line | arm only when the slide's last 2 words are heard | `onset_arms_only_on_last_two_words` |
| 2 | a refrain whose closing words occur earlier in the same slide fired 5 s early | no onset trigger on a slide whose final 3 words occur earlier in the slide more than an echo (4 words) away; such slides move on words | `no_onset_on_slide_with_repeated_closing_words` |
| 3 | an onset move reset the follower's clock, narrowing the word search (it got lost behind the singer at slow passes); keeping the old clock instead let dead-reckoning run a slide ahead | two clocks: the pace and dead-reckoning clock restarts at the onset; the word-search width keeps the clock of the last word evidence, so an onset neither narrows nor widens it | `onset_move_keeps_word_search_width` |
| 4 | dead-reckoning walked through a 10 s pause | dead-reckoning only while something sounds above the long-term floor | `no_dr_through_quiet` |
| 5 | the detector's floor started from digital silence | start the floor from the median of the first second of real audio | `onset_floor_starts_from_real_audio` |

3. **Further gates:**
   - *First slide:* an onset may show a song's first lyric slide only when the operator cued the song (on Presenter's leading blank) and ≥ 2 s of quiet came before it. A cough after silence would also trigger it, so this mode is a separate switch, off by default, and used in supervised tests only. ADR-022's song-start rule applies otherwise. Test: `first_slide_onset_needs_cue_and_quiet`.
   - *Early-fire budget:* the same as ADR-022 § 5: early hides ≤ 0.4% per fire.
   - *Fire-time re-check:* every fire passes ADR-022 § 4's re-check (Pause, Off, the 5 s operator hold, the watchdog, the gate). Pause, Off or any operator action disarms.
4. **Ship rule.** Allowed behind follower config (an `onset` field, `#[serde(default)]`, off), so committed logs replay byte for byte (ADR-013). It may be turned on for a church only after plan V3b's gates pass (lab prototype, gates and supervised live tests) and V11 shows the early-fire budget on the tune corpus and the synthetic stress set. A default-on change needs a later amendment with shadow-Sunday numbers.
5. **Feed dependency** (amended 2026-11-26, Amendment 1: FA-38 and FA-39 answered). The trigger works best on a vocal-only feed; the full mix on the current unbalanced, low-level feed is a supported, degraded condition, detected from numbers, in which the trigger disarms and words move the slide. On a band mix it rarely fires and falls back to words: never worse (P1: 4.30 → 4.01 s, 11 of 11 both ways). The input needs gain or AGC and a 100 Hz high-pass before the VAD, a channel select, and a level and hum check at setup (spec § 4.1). A balanced or isolated connection is the fix for hum.
6. **Logging:** ADR-022 § 6's `follow` line gains `why=onset` and an `arm` flag. Numbers only. Review reports onset fires and "Tight on time" (spec § 2.2).
7. **Whisper prompt (bias) strategy: a separate decision point that stays an experiment (plan V3c).**
   - Proposal: build the prompt from the follower's position: the current slide's remaining line plus the next slide's first line, ≤ 40 tokens.
   - A/B on cached takes: Base + bias vs Small unbiased, for speed (s/pass) and accuracy (changes reached, wrong moves, Tight on time).
   - Guards: decode only while the VAD hears voice; heard words may not run ahead of the onset and voice timeline; a phantom-word metric (words matched while the VAD hears no voice) on every run; off by default.
   - Adopting it needs its own decision with V3c's numbers. This ADR does not adopt it.
8. **Supervised live tests (V3b) use a private lab helper, `live.py`,** never pushed: input (Line In, left channel) → 100 Hz high-pass → +12 dB → (a) a local voxxad build with streaming stdin in `--shadow` mode (Whisper words; voxxad sends nothing) and (b) the onset detector. The follower decides. The helper presses Right in WorshipTools Presenter only while a Presenter.exe window is in front (the ADR-019 § 2 guard), at most 3 steps per move, never Left. `--dry-run` presses nothing. It logs numbers only and stops on Ctrl+C or `--minutes`.

## Alternatives Considered
| Alternative | Pros | Cons | Why Not |
|---|---|---|---|
| Time-based prediction from practice (ADR-022 § 5, V6b) | works on a band mix | needs practice memory and the corpus; fires on a guess of time; early-hide risk | stays off; the onset reacts to the real event and needs neither |
| Arm anywhere in the last line (first lab version) | more fires | breaths inside the line fired 3–5 s early | failure mode 1 |
| Let an onset move narrow the word search | faster re-anchor | the follower got lost behind the singer at slow passes | failure mode 3 |
| A faster model alone (Base) | no new mechanism | follower 5 of 11 on T1; words are still 1–3 s late | speed cannot remove window delay |
| Today's `--bias` with the conductor's slides | exists | follows the conductor (2 of 11 on T1); hallucination risk | V3c tests a prompt from the follower instead |
| Separate the voice from a band mix with a model | would help without a vocal feed | 2–3 GB tools (spec § 9); too heavy beside Whisper on the booth PC [est.] | a vocal-only desk feed is cheaper |

## Consequences
### Positive
- On a vocal-only feed, slides follow the singer within tens to hundreds of ms in the lab (T1 median 0.18 s, T2 0.04 s).
- Never worse on a band mix: without onsets, words move the slide as before.
- No practice memory or corpus needed to start; nothing fires on a guess of time.
- Every fire is explainable from a numbers-only log line.
### Negative
- Needs about 1 s passes (ADR-021 V3/V4); at 2.6 s passes the gain is small (T1 3.28 → 2.66 s).
- Works best on a vocal-only feed; on a full mix it falls back to words (degraded mode, measured per take). Input conditioning stays a setup step for every church; the hardware fix is deferred (FA-38).
- Slides whose closing words repeat inside the slide still move on words, so they stay late.
- One more config path and gate set in the follower row.
### Risks
- An early fire on a breath, count-in, cough, spoken aside or bleed into the vocal mic. Mitigated by arming on the last 2 words, the margin gate and the 0.4% budget.
- If the follower is on the wrong slide, a fire makes the wrong move faster. Mitigated by the margin gate at arm time and the fire-time re-check.
- Three lab takes (28 changes; only T1 is a live singer on a vocal mic) cannot show a 0.4% rate. V3b counts early fires on the founder's takes; V11 measures the rate on the tune corpus (F1) and the synthetic stress set (spec § 7.5).
- Detector constants were set on these takes; hum or a different room may need other values.

## Dependencies
- Amends: ADR-022 § 5. Adds to: ADR-022 § 4 (gates) and § 6 (logging).
- Depends on: ADR-021 (about 1 s passes), ADR-022 (the follower), ADR-019 § 2 (front-window guard), ADR-004 (bias off by default, E6), ADR-013 (replay).
- Constrains: ADR-029 (kit: the practice and readiness prompts check level, hum, channel and a vocal-only feed; the review prompt reports Tight on time and onset fires).
- Plan steps: V3b — Onset trigger: lab prototype, gates and supervised live tests; V3c — Whisper prompt from the follower (bias A/B); V3, V4, V6, V11.

## Review trigger
V3b's gates fail on the corpus or in a supervised live test; any wrong slide shown by an onset fire; the booth release build (V3, with `reuse_state`) does not reproduce the lab's 0.53 s/pass on sung audio; V3c shows the follower-driven prompt wins or produces phantom words; the degraded-mode fallback shows a wrong slide or fires on band energy; V6b passes (time-based prediction may join).

## Amendment 1 (draft, 2026-11-26, founder answers FA-38 and FA-39)

**What the founder decided.** FA-38: the USB interface is deferred for some time; a noisy, low-level, unbalanced feed is a
supported, degraded condition, and progress never blocks on the current booth feed. FA-39: a vocal-only aux is not a hard
requirement; the trigger and the voice path must function, with reduced performance where needed, with the full mix on
the feed.

**What changes in this ADR.**
1. § 5 no longer requires a vocal-only feed. It is the preferred condition. The full mix on the current feed is a supported
   degraded condition.
2. **Degraded detection (numbers only).** The input chain (plan V3b task 6, `src-tauri/src/input/`) sets a `degraded` flag on
   its level line when any of: the floor is above a limit; the hum share at 60/180/300/420 Hz is above a limit; band energy is
   present in the 200 ms before an onset candidate (no breath gap). The limits are constants in `InputConfig`, set from the
   booth's measured numbers (floor about −60 dBFS, singing about −34 dBFS) and reported in the review.
3. **Disarm rule.** While `degraded` is set the trigger does not arm; words move the slide as in ADR-022. The P1 result
   (4.30 → 4.01 s, 11 of 11 both ways) is the "never worse" bar for this mode and is re-measured on every band or
   full-mix take as the required degraded-mode row (plan V3b task 4, V3d task 4).
4. **Gain and hum code stay** in the input chain (a 100 Hz high-pass, a configurable gain, level and hum numbers); the
   ground-loop isolator (FA-38 option a) remains optional and nothing waits for it.
5. Tests added to V3b task 6: `degraded_flag_from_hum_share_and_floor`, `onset_disarmed_while_degraded`.

**What does not change.** The detector, the arm and fire rules, the gates, the ship rule (off by default, replay byte-identical),
the live-helper rules in § 8, and the expectation that clean solo-vocal takes reach the lab numbers.

**Review trigger added.** The degraded-mode fallback shows a wrong slide, or an onset fires on band energy while
`degraded` is set.
