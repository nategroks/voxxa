# voxxa: current-state brief for the autonomy research and review round

Date 2026-11-26. Author: the session that built the onset trigger, the live helper and the parts map. Slide
indices only; no lyric text. Take names: T1 = vocal1tap3 (the founder, It Is Well, public domain, solo vocal
mic, 11 changes, tapped truth); T2 = the public text-to-speech It Is Well fixture (6 changes, exact truth);
P1 = a private church song over a full band (11 changes, no part labels); F1–F4 = re-cuts of T1; live4 = the
first live Presenter test. Sources: `out/adrs/021–031`, `out/spec.md`, `out/plan.md`,
`out/founder-action-items.md`.

## 1. Architecture as built today

**Product (Tauri app plus CLI, public repo; read via `C:\dev\voxxa-newton`, `git show origin/main:`)**
- Audio in: `src-tauri/src/listen.rs`. It has a keep-up gate that drops hops when decoding lags.
- Recognizer: `src-tauri/src/transcription/` (whisper-rs; a Silero VAD gate, enter probability 0.60). Small model,
  6 s encoder context in the lab.
- Today's conductor: `src-tauri/src/aligner/mod.rs`. It scores each transcript against whole songs with a
  character `partial_ratio`: start at 85, advance at 70, a silence state after 3 s, blank after 5 s more.
  Sections live in `aligner/evidence/sections.rs` (ADR-016); the `.txt` importer drops them today
  (`importers/txt.rs`; plan V5 fixes it).
- Dispatch: `src-tauri/src/dispatch.rs`. Drivers: keystroke (WorshipTools Presenter, universal), FreeShow,
  ProPresenter 7 REST, OpenLP v2, OpenSong. Binding profiles: `kit/templates/bindings/*.json`.
- `voxxad` (headless daemon, `src-tauri/src/cli/voxxad/`): `--stdin-audio <wav>`, `--setlist`,
  `--shadow | --replay | --presenter`, `--bias` (slide text as Whisper's prompt, off by default) and
  `--log-dir`. It writes a session log (`window`, `hypothesis`, `action` lines).
- `voxxa-eval` (`src-tauri/src/cli/voxxa-eval/score.rs`) scores an action list against `truth.jsonl`.

**Local-only experiments** (`C:\dev\voxxa-private`, never pushed; builds in `C:\dev\target-private\keep\`)
- env switches `VOXXA_AUDIO_CTX`, `VOXXA_TEMP_INC` and `VOXXA_MAX_TOKENS` (`transcription/mod.rs`);
- streaming stdin for voxxad (`--stdin-audio - --realtime`, `main.rs`);
- the `gpu-vulkan` feature (`voxxad-gpu-vulkan-t0.exe`, `voxxad-gpu-vulkan-live.exe`).

**Lab** (`C:\dev\voxxa-church-private\lab\`, private uv project; agents run it only with the founder; numbers only)
- `follower.py`, the plan-order follower (ADR-022's prototype): Smith-Waterman alignment of each Whisper window
  over the setlist expanded in sung order. It never moves backward, applies a pace penalty, and dead-reckons
  one slide when words are unclear.
- `predict.py`, the onset trigger (ADR-030). A causal phrase-onset detector works on 300–4000 Hz band energy in
  10 ms frames:
  - the floor follows quiet down at once and rises at most 2 dB/s;
  - an onset is a level above floor + 10 dB for 60 ms, after ≥ 200 ms below floor + 5 dB.
  The `Predictive` follower arms when words show the slide's last 2 words, and fires on the next onset. Two
  clocks: one for pace, one for search width.
- `parts.py`, the parts-map follower (ADR-031); see § 5.
- `flows.py` re-cuts a tapped take into other part orders, with truth mapped.
- `evalparts.py` compares the plan-order follower and the parts map on all takes.
- `live.py` is the live helper. Line In (left channel) → 100 Hz high-pass → +22 dB → tanh limiter → (a) voxxad in
  streaming `--shadow` mode and (b) the onset detector. It presses Right in Presenter only while a Presenter.exe
  window is in front (ADR-019 § 2), at most 3 steps, never Left. It has `--dry-run` and `--wav` replay, and logs
  numbers only.
- `vocal_test.py` is the practice recorder and tapper. Record mode: mic and channel select, a prompter, SPACE per
  slide. Tap mode: tap along a replay over headphones, with a "pa!" reaction-lag calibration (524 ms last time).
  It writes `recordings\<date>-<id>\` with audio, setlist, sung order, truth and intake files.
- `heard.py` runs voxxad over a wav and keeps a heard-words cache (private, never printed).

**Listener DB (designed, not built):** practice memory, ADR-023 and plan V12.
- One JSON per song: `<church_root>\practice\<song_fp>.json`.
- Contents: learned order, dwell, gaps, outro, words per second, and ≤ 20 heard tokens per slide.
- `voxxad --learn` proposes a run from a rehearsal capture; `--confirm` accepts it.
- Only confirmed runs feed the follower.

## 2. Measured numbers to date (lab unless marked live)

**Feed.** Booth: WING output → unbalanced cable → PC jack as Line In, voice on the left channel.
- The floor is about −60 dBFS, mostly 60 Hz ground-loop hum.
- Raw singing sits at about −34 dBFS. Through the Silero VAD that gave 2 Whisper passes in 200 s; +12 dB gave
  152. Live needs about +22 dB with a soft limiter.

**Speed** (GTX 960 2 GB, Vulkan dev build, Small, sung audio):
- 2.57 s/pass as built. Cause: whisper.cpp's temperature fallback re-decodes sung windows.
- `temperature_inc 0`: 0.94 s/pass. Adding `max_tokens 48`: 0.53 s/pass, with the follower unchanged.
- Base: 1.13 s/pass, but 5 of 11 on T1.
- VRAM 814 of 2048 MB with Presenter open.
- Booth CPU (Tiny, 30 s encoder) on P1: 138.2 s, against a 150 s bar (V0).

**Vocal test 1 (T1, words only):**
- Plan-order follower: 9 of 11 in order, median 1.7 s late.
- Today's conductor: 2 of 11, median 8.9 s, 3 mid-song blanks.

**Onset trigger** (1 s passes; median |delay| after the first slide; words only → with the trigger):

| Take | Words only | With the trigger |
|---|---|---|
| T1 | 1.90 s (10% within ±300 ms) | 0.18 s (80% within ±300 ms) |
| T2 | 2.12 s | 0.04 s |
| P1 (band) | 4.30 s, 11 of 11 | 4.01 s, 11 of 11 |

- The detector found slide starts within ±300 ms for T1 10 of 11, T2 5 of 6, and P1 only 2 of 11 (the band fills
  the breaths).
- At 2.6 s passes the gain mostly disappears.

**dry1** (live chain, T1 replayed at real speed, no keys): median 0.18 s, 70% within ±300 ms, 7 onset fires,
0 wrong slides.

**Live tests** (supervised, founder present):
- live1–live3 failed on setup:
  - a start-up pop fired the first slide at 0.6 s;
  - no singing;
  - too quiet, and Presenter not in front.
- **live4** passed mechanically: 16 Right presses, all in order.
  - The founder's report: no lag, but moves came early, and it could not go back to the chorus.
  - Replay against a truth read from the heard words (±1 s; tapped truth is FA-45): 11 of 16 right and 5 more
    than 1 s early.
  - Causes of the 5: 2 dead-reckoning steps in mid-verse during held notes (−3.4 s and −4.2 s), 2 onset fires
    after a chorus on a repeated closing phrase (−2.7 s and −5.0 s), and 1 between the two look-alike chorus
    slides.

**Parts map** (ADR-031; Small, live decoder settings, 1 s passes). Each cell is right changes · wrong moves ·
moves more than 1 s early · wrong-slide seconds:

| Take | Changes | Plan-order follower | Parts map |
|---|---|---|---|
| T1 | 11 | 10 · 2 · 1 · 5.5 | 11 · 1 · 0 · 10.7 |
| T2 | 6 | 6 · 0 · 0 · 7.5 | 6 · 0 · 0 · 7.5 |
| F1 extra chorus | 14 | 9 · 3 · 2 · 49.0 | 12 · 0 · 0 · 21.8 |
| F2 skipped verse | 8 | 4 · 5 · 1 · 63.0 | 8 · 1 · 0 · 13.7 |
| F3 chorus first | 10 | 0 · 13 · 2 · 140.7 | 10 · 1 · 0 · 25.1 |
| F4 last line repeated | 12 | 9 · 3 · 3 · 37.6 | 12 · 0 · 0 · 15.2 |
| live4 | 16 | 11 · 5 · 5 · 24.1 | 15 · 1 · 1 · 17.6 |
| **Total (7 sung takes)** | **77** | **49 · 31 · 14 · 327** | **74 · 4 · 1 · 112** |
| P1 (band, no labels) | 11 | 11 · 0 · 0 · 45.9 | 8 · 1 · 0 · 66.0 → stays on the plan-order follower |

**The known cost:**
- When the plan is kept, the verse after a chorus appears 1–3 s after its first word: T1 2.2 and 1.3 s; live4
  2.8, 2.6 and 2.6 s.
- A slide whose last words Whisper missed now waits for words instead of a pace guess: T1 slide 5 at +3.7 s;
  live4 slide 1 at +2.6 s, where dead-reckoning had moved it 3.4 s early.

Every threshold was set on these same takes, and F1–F4 share T1's singer and room.

## 3. Open technical problems, ranked, with hypotheses

1. **Jumping in the presentation app (blocks flowing worship without a volunteer).** Presenter cannot go to a
   section or slide (§ 4). *Hypotheses:*
   - clearing the text, then stepping, then showing again may hide the in-between slides (untested);
   - FreeShow's API makes a jump one call;
   - clicking the slide through UI automation works, but is fragile.
2. **Generalisation.** Tuned on one highly repetitive hymn, its re-cuts and one band song. *Hypothesis:* songs
   whose parts share opening words will slow part-end decisions. Real detour takes (V3d task 4) and the held-out
   corpus (V6) will show whether the handicaps (planned next 0, chorus −0.5, same part −1.0, other −1.5), the
   1.5-word margin, the 2-window rule, the 0.7 look-alike cut, 2 s elimination and the 8 s review hold.
3. **Sung-speech recognition errors.**
   - Whisper misheard T1's verse-2 first line for about 10 s.
   - It writes repeated phrases during held notes, which caused false jumps before the gates.
   *Hypotheses:*
   - a prompt built from the follower's position, or decoding constrained to the candidate parts' first lines
     (V3c);
   - a short decode right after an onset, for the first-word decision;
   - phoneme or acoustic alignment (ADR-009's escalation path).
4. **Speed at part ends after a chorus (1–3 s).** *Hypotheses:*
   - per-leader habits from practice memory switch `boundary = plan` where the leader keeps the plan;
   - an onset-triggered first-word spotter.
5. **Band mix without a vocal-only feed.** The onset trigger rarely fires there, and words run about 4 s late.
   *Hypotheses:* a vocal-only desk output (FA-39) is the cheap fix; source separation is too heavy beside Whisper
   on a 2 GB GPU [est.].
6. **Full autonomy safety with nobody at the PC:**
   - who corrects a wrong slide;
   - the front-window guard fails if something else takes focus;
   - position drift after a failed key sequence (ADR-025 § 2 freezes until the operator confirms);
   - song starts (the first-slide onset is supervised-only, since a cough also fires it);
   - talk at part ends (elimination can fire on a spoken aside).
   *Hypotheses:* an app with read-back (FreeShow `get_output`) removes drift; amber "not sure" holds instead of
   moves; ADR-022's lost and watchdog gates.
7. **Self-training from practice.** The ADR-023 design learns order, dwell and heard tokens. Not built.
   *Hypothesis:* hindsight alignment of rehearsal captures with the parts map, plus per-leader detour statistics,
   gives the plan hint and `boundary` per leader.
8. **Unlabelled songs.** The parts map needs sections. *Hypothesis:* the `.txt` importer (V5), service-plan
   arrangements and practice memory fill them; plan repeats suggest a chorus.
9. **Input chain in the product.** Channel select, high-pass, gain or AGC before the VAD, and a level and hum
   check at setup are lab-only today (input-chain ticket, V3b task 6).

## 4. Jumping: WorshipTools Presenter, FreeShow, FA-44

**WorshipTools Presenter** (its documented shortcuts; the booth test card):
- Keys: Right, Space or PgDn for the next slide; Left or PgUp for the previous; Down and Up for the service item;
  F10–F12 clear (text, media, all). F2/F3 create items and F6–F9 open windows.
- No section key, no slide-number key, no remote-control API, no read-back.
- A song opens on a leading blank. Right from a song's last slide goes into the next item's blank (seen on the
  booth PC, #183).
- The profile `kit/templates/bindings/keystroke.worshiptools_presenter.json`:
  - allows only Right and Left;
  - never sends F10–F12, Up/Down, Enter, Esc, F2/F3, F6–F9 or Ctrl+Alt+arrows;
  - leaves blank and unblank unsupported (F10 clears lyrics, and the key that restores them is unconfirmed, test
    card T4);
  - has `goto_scope: steps_only`, positions counted per key.
- Going back N slides is N Left presses, and each in-between slide shows.

**FreeShow** (`kit/templates/bindings/freeshow.json`):
- HTTP on port 5506, bearer auth, 750 ms timeout.
- `next`, `prev`, `goto`, `blank` and `unblank` go through the existing driver methods.
- `goto.item` uses `index_select_project_item` then `index_select_slide` (showId, layoutId, index).
- Overlays, actions, scripture and `clear_slide` over HTTP.
- Read-back by `get_output`, keeping only the show id, layout and index.
- Refused: create_show, set_plain_text, set_show, add_group, rearrange_groups, send_rest_command, emit_action and
  deletes.
- Open question W1: the public API puts fields at the top level of the body, while the driver nests them under
  `data`. `index_select_slide` may ignore its index until a live test settles it.
- Every HTTP step is UNVERIFIED live and runs as a hint until its contract test passes (ADR-024 § 5).

**FA-44 options:**
- (a) Presenter arrow bursts (in-between slides flash; visibility unmeasured);
- (b) Presenter hide while stepping (F10/F11, arrows, show; unknown whether Presenter stays clear);
- (c) click the slide through Windows UI automation (direct, fragile, needs the front-window guard);
- (d) an app with commands: FreeShow (free, open source, driver exists, W1 to verify) or ProPresenter (paid,
  network API);
- (e) suggest, don't jump: one operator button ("Chorus?").

Recommended: a 10-minute test of (b) and (a); (e) during services until a jump is proven; trial (d) FreeShow as
the long-term path. Until FA-44, the helper sends only Right. On any other move it logs the target, sends
nothing and stops for that song.

## 5. Plan steps V3b–V3d; what ADR-030 and ADR-031 decided

**ADR-030 (Proposed): arm on words, fire on the onset.** Words decide *which* slide; the onset decides *when*.
- Arm only when the slide's last 2 words are heard. No trigger on a slide whose closing 3 words occur earlier in
  it more than 4 words away.
- Two clocks. Dead-reckoning only while something sounds above the floor.
- The first-slide onset needs an operator cue and ≥ 2 s of quiet (supervised tests only).
- Early-fire budget ≤ 0.4% per fire. Behind config, default off. It needs a vocal-only feed and about 1 s passes.
- § 7: the Whisper-prompt experiment (V3c) is separate and not adopted. § 8: the live helper's rules.

**ADR-031 (Proposed; the founder chose option A): a song is a map of parts, and the plan is a hint.**
- Position = (slide, word). Each window is aligned along the rest of the current part, and along branches into
  every part's first slide (handicaps above).
- Gates:
  - leaving the plan needs a 1.5-word margin over the slide on screen, in 2 consecutive windows;
  - a part is entered at its first slide;
  - words never jump between look-alike slides (≥ 70% the same words), and skip to one only after time to sing
    the words in between at 1.5 words/s;
  - no repeat branch on a slide that repeats its own closing words;
  - lost: 10 s → the whole part, 20 s → the whole song, forward first.
- Timing:
  - the onset fires inside a part and from verse into chorus;
  - after any other part, words decide;
  - elimination: 2 s of singing that fits no part's start → the planned next part;
  - an 8 s review after any guess, in which words can correct it.
- Dead-reckoning never moves the slide in a labelled song. Unlabelled songs keep the plan-order follower.
- New `why` values `jump`, `elim` and `lost`; a new measure, wrong-slide seconds; the flow set joins the
  evaluation.

**V3b — the onset trigger.**
- Gate cases (`scripts/lab/onset_gate_cases.py`) and the `tight_300` column in voxxa-eval.
- Founder checklists: run the gates, score every take, and F4 (the supervised live Presenter test).
- The port plan to `aligner/follower/onset.rs` (`FollowerConfig.onset`, default `None`), the input-chain ticket
  and kit checks.

**V3c — Whisper prompt from the follower (bias A/B).**
- Arms: Small unbiased against Base with the follower's prompt, with a phantom-word count; recorded takes only.
- Base is chosen only if it is no worse on every take, causes no phantom-word move, and runs ≤ 1.0 s/pass.

**V3d — the parts map.**
- Gate cases (`scripts/lab/parts_gate_cases.py`, 12 names), and `wrong_slide_s` plus scoring by slide id in
  voxxa-eval.
- The flow set on every tapped take, and ≥ 3 founder takes with real detours.
- A supervised live test with FA-44's driver.
- The port plan to `aligner/follower/parts.rs` (`FollowerConfig.parts`, default `None`).
- V11 ports both, behind config, default off.

Founder items open: FA-43 (approve ADR-031), FA-44 (the jump driver), FA-45 (tap live4's truth), plus
FA-38–FA-42 from the onset round.

## 6. Constraints a reviewer must respect

- **Words and media.**
  - Never quote lyrics or slide text; use slide indices. Public-domain songs may be named.
  - Never read, print or copy `setlist.json`, `truth.jsonl`, `heard_*.jsonl`, session-log words or audio under
    any private folder (`C:\dev\voxxa-private\fixtures\private\`, `C:\dev\voxxa-church-private\`). Output is
    numbers only.
  - Licensed media: agents never copy, upload or move the church's licensed video. Nothing derived from it leaves
    the booth PC, Google Drive included.
  - Song sources are licence-checked (ADR-026: CC, public domain or church-owned).
- **Microphone and keys.**
  - Agents never open the microphone, and never unattended.
  - Live listening and key presses happen only in supervised tests that the founder starts and attends: keys only
    through `live.py`'s front-window guard, Right only.
  - Never run Presenter or press keys in it otherwise.
  - Never write in `%APPDATA%\Presenter`; read only scratch copies in `%TEMP%`, deleted afterwards.
  - Never play audio to the room system output; select devices by name.
- **Builds and repos.**
  - Never touch the installed voxxa or the identifier `com.voxxa.app`; never touch `C:\dev\voxxa`; read the repo
    through `C:\dev\voxxa-newton` (`git show origin/main:`).
  - `C:\dev\voxxa-private` and its env switches, streaming stdin and `gpu-vulkan` feature stay local, never pushed.
  - Lane worktrees are private.
  - No release publishing, rc tags, `build.yml` runs, or updater and signing secrets without founder approval.
- **Budgets** (ADR-028 rows, caps never raised later):
  - follower 1.3 × measured port, ≤ 500 lines (inside `aligner/evidence/` if ≤ 250); practice 300; layout 300;
    Run screen 350 (JS/HTML/CSS); kit tools 300;
  - voxxa-eval additions ≤ 20 lines (V3b) and ≤ 25 lines (V3d); the conductor change ≤ 20 lines; voxxad
    practice flags ≤ 20 lines;
  - cost estimate before any workflow, and anything above $50 [est.] needs the founder's yes.
- **Privacy in public text.** Never name the private song (use P1), the church, its CCLI number or the booth PC's
  host name. No church data, song titles or paths go to external services, and the Planning Center connector is
  not used.
- **Autonomy defaults.** Every new behaviour lands behind config, default off; committed session logs must replay
  byte for byte (ADR-013); a default flips only after shadow Sundays meet the Correct bar (ADR-022).
