# voxxa: follow the singing, one master layout, a kit for many churches

Design document for founder approval. Date 2026-11-24. Base: `origin/main` @ `866b51e` (Merge #184), read only
with `git -C C:/dev/voxxa-newton show origin/main:<path>`. Every `path:line` below is on that commit.
Authors: the voxxa team synthesiser (Principal Architect + Engineering Manager + Staff Engineer), from drafts
10–13 (three senior engineers and the UI consultant), the independent ponytail lane (14) and two adversarial
reviews (20 safety, 21 feasibility). Rulings on every disagreement are in `adjudication.md`.

Naming rule in this document: the private test song is **"private song P1"**, the pilot church is **"the pilot
church"**, and the booth computer is **"the booth PC"** (Ryzen 3 1200, 4 cores, GTX 960 2 GB, Windows 11,
WorshipTools Presenter). Church names, song titles, host names and licence numbers never appear in voxxa
documents (review finding S-42). Examples use made-up words or public-domain titles only. Nothing here was
built or run, except the private onset lab of 2026-11-26 (ADR-030; its numbers are marked "lab").
**[UNVERIFIED]** marks what nobody has checked yet; **[est.]** marks a rough number.

How to approve: each numbered section ends with an **Approve** line. Reply "approve 3", "change 4: …" or
"approve all". Decisions that need more than a yes are listed in `founder-action-items.md`.

---

## 0. Summary in plain English (one page)

**What you asked for.** Run Python tools with uv; keep voxxa current ("latest updates": § 4.12 reads it as
five concrete things, from signed app updates to kit versions, and FA-32 asks whether that is what you meant);
a "listener DB" that learns each song in a practice session before the service; feed it any `.wav` you send; type lyrics in by hand for Creative
Commons, public-domain and church-owned songs instead of CCLI; one master layout file that works with any
presentation app and pairs hotkeys, API calls and command lines with songs, scripture, key words and
graphics; a kit so other churches can do the same; and a team process. Your bar: **95% accurate, moving on
time.**

**What the evidence says today.** On private song P1 (11 slide changes), today's song logic hides the lyrics
twice in normal 4–9 s vocal gaps and reaches 4 of 11 slides. A new "score follower" prototype reached 11 of 11
with 0 wrong moves on Whisper Small, but the same setting made 8 wrong moves when one recognizer setting
changed. Small also takes 3.4 s of computer time per 1 s of audio on the booth PC, so it cannot run live yet,
and installer builds run Whisper about 3.5× slower than they should (ticket #59). Every number is one song.

**What we will do, in order.**
1. **Cheap proofs first (weeks 1–4, about $45–110 of agent time, $85–210 with the independent ponytail
   lane [est.]):** fix #59 (two build settings);
   measure whether simple decoder settings make Small fast enough on the CPU; measure the GPU (needs the
   Vulkan SDK); score the follower on 10+ real songs it was never tuned on. You record about 3 Sundays of
   rehearsals.
2. **Make automatic moves safe (weeks 2–4):** fix a real bug where a failed key press makes every later
   slide one off; make every key profile check which window it types into; keep prayer words out of saved
   logs and help bundles.
3. **Build (weeks 4–8):** the follower inside voxxa (off by default until it passes on songs it never saw),
   the practice memory, the master layout with suggest-only cues, and a small "Run" screen with Pause and a
   one-line reason for each move.

**What "95%" will mean.** Two published numbers instead of one. **Correct:** the right slide is reached for
≥ 95% of sung slides, with at most 1 wrong move per 10 songs and lyrics never hidden mid-song. **On time:**
the share of slides shown within 1.5 s of the first sung word. We commit to "correct" first. Today's best
offline result puts "on time" at 45% or less, because the recognizer needs about 2 s to hear a line; reaching
95% on time needs prediction from practice (moving at the predicted end of a line, like a good operator) or the
onset trigger (ADR-030: words arm the slide, the start of the next sung line shows it; lab only so far), and
we will only promise a number after we measure it. After the first live test, the follower treats a song as a map
of parts, not one fixed list (ADR-031): the planned order is a hint, a chorus, a verse or a repeated line can
come back at any time, and a slide moves only on words or a phrase start, never on pace alone.

**Round 3 (draft, 2026-11-26; `round2/50-adjudication.md`).** The autonomy review found that the presentation app,
not listening, is the ceiling today: WorshipTools Presenter can only be told next and previous and never asked
where it is. voxxa therefore makes FreeShow its first-class target (Tier A: jump plus read-back, ADR-032), keeps
Presenter as Tier B (keys only, off-plan moves as suggestions), and moves the Sunday output only in stages, decided
at the end of January on measured "dark Sunday" numbers (FA-48, ADR-035). The volunteer becomes a *minder* with a
health check, HOLD and STOP FOLLOWING controls and a one-page card; nobody leaves the booth until a second, later
gate is met (FA-52). Learning stays offline in rehearsal review, never on Sunday. The move itself (decision to key press) takes under
100 ms.

**What we will not build now.** A database server or SQLite store (one small file per song is enough), an
in-app lyric editor (Notepad plus a licence spreadsheet first; your call), command lines started by heard
words, automatic cues before they are proven, new presentation-app drivers, GPU installers for other
churches, and a big kit nobody has tested. Where this differs from what you literally asked for, each difference
has its own founder item: the lyric editor (FA-14), command lines out to other programs (FA-33), automatic
scripture and key-word graphics (FA-34, suggest-only in v1) and outbound hotkey sequences (FA-35: shown as an
example in the binding profiles, run only after a test card per app).

**Other churches need the command-line tools.** Practice capture, `.wav` intake and the layout check run
through `voxxad` and `voxxa-eval`, which the installer leaves out on purpose. They will ship as a separate signed
download, `voxxa-tools-<version>-windows-x64.zip` (step V5c, FA-31). Until then these three jobs run only on
your development booth.

**What we need from you this week.** Approve the #59 fix; install uv and the Vulkan SDK; approve the two-number
"95%"; get the worship team's recording consent and plan 3 rehearsal recordings; approve one batch of GitHub
tickets. The full ordered list is in `founder-action-items.md`.

---

## 1. Problem

| # | Problem | Evidence |
|---|---|---|
| 1.1 | Normal vocal gaps hide the lyrics mid-song | 3 s without speech → `InterVerseSilence`, then 5 s → blank (`src-tauri/src/aligner/mod.rs:85-86`, `:354-363`); re-showing needs the 85 start bar (`aligner/mod.rs:93`). P1 hid twice on Small and Medium (brief § 4). |
| 1.2 | Song start depends on one character score | Whisper Tiny peaks at 83.3 < 85 on P1 → 0/11 (brief § 4). |
| 1.3 | The recognizer that is accurate enough is too slow on the booth CPU | Small 3.4 s per pass with a 6 s encoder; live needs < 1 s per 1 s hop (brief § 4). The keep-up gate then skips hops (`src-tauri/src/listen.rs:159-166`). |
| 1.4 | Installer builds run Whisper unoptimised | #59 (`.10x/handoff.md:92`, `.10x/status.md:304`): Tiny 461 s vs 130 s; cause found 2026-11-24 (brief § 4). |
| 1.5 | The follower prototype is fragile | Same fixed setting: 11/11·0 wrong (Small, 6 s encoder) vs 6/11·8 wrong (Small, 30 s encoder) (brief § 6). One song only. |
| 1.6 | The church is leaving CCLI | Lyrics must be typed in by hand with a licence and credit per song (brief § 2). Today voxxa has no licence field; `usage.csv` was planned for CCLI (`.10x/adrs/007-text-licensing-no-copyrighted-text-shipped.md:12`) and never built. |
| 1.7 | Operators cannot override while Presenter is in front | voxxa's keys work only in its own window; WorshipTools needs Presenter in front (ADR-019 § 2). The global-shortcut plugin is loaded and unused (`src-tauri/src/lib.rs:119`). |
| 1.8 | A failed key press leaves voxxa one or more slides off for the rest of the song | `sent.global` is written only after the whole key loop (`src-tauri/src/dispatch.rs:163-175`); an error mid-loop returns early (finding S-10). |
| 1.9 | No layout, no cue words, no graphics, no kit | Only song moves exist; drivers expose next/prev/goto/blank/unblank. |

**Approve 1:** the problem statement.

---

## 2. Goals and the measurable "95%"

### 2.1 Goals

1. Lyrics stay up for the whole song, through gaps, repeats and skips.
2. The right slide, every time, measured on songs the system was never tuned on.
3. On time, measured honestly and improved with practice memory.
4. One master layout per church, the same whatever presentation app runs, with hotkeys, API calls and (later,
   gated) command lines bound per app.
5. Songs entered by hand, each with a licence and a credit line; CCLI optional.
6. A second church can set up from the public kit alone, and nothing church-specific ever leaves its booth PC.

### 2.2 The definition (two numbers plus move speed)

**Unit.** A *slide occurrence*: one time a slide is sung. A chorus sung three times is three occurrences. The
statistical unit for confidence is the *song performance* (occurrences inside one song are correlated).

| Measure | Definition | Bar for "volunteer-ready" | Status today |
|---|---|---|---|
| **Correct: recall** | share of occurrences whose right slide is shown between `t_w − 1 s` and the next occurrence's `t_w − 1 s` (today's scorer, `src-tauri/src/cli/voxxa-eval/score.rs:49-51`) | ≥ 0.95 | 11/11 on one song (Wilson lower bound 0.74) |
| **Correct: wrong moves** | gotos to a slide that is neither the current nor the next truth occurrence, plus early hides (a slide replaced before its last sung word started), plus skips of ≥ 2 | ≤ 1 per 10 songs, none lasting > 3 s | 0 to 8 per song depending on setting |
| **Correct: hidden mid-song** | a blank (or no lyric slide) for > 1 s between a song's first sung word and its end where the truth has no blank | 0 | 2 per song with today's conductor |
| **Correct: shown during talk** | a goto or unblank inside a talk span | 0 | not measured |
| **On time** | share of non-song-start occurrences shown within `[early bound, t_w + 1.5 s]`; early bound = start of the previous occurrence's last word when known, else `t_w − 1 s` | target set after step V6b; the strategy's goal is ≥ 0.95 (`.10x/specs/2026-10-24-software-strategy.md:26`) | ≤ 0.45 offline (3.0 s median, feasibility review § 1.1). Lab, 2026-11-26, 1 s passes, median delay after the first slide, words only → with the onset trigger (ADR-030): T1 1.90 → 0.18 s, T2 2.12 → 0.04 s, band mix P1 4.30 → 4.01 s |
| **Tight on time** | share of non-song-start occurrences shown within ±300 ms of the first sung word (`t_w`) | reported, not gated (ADR-030) | lab, 1 s passes, words only → with the onset trigger: T1 10% → 80%, T2 20% → 60% |
| **Wrong-slide seconds** | time a slide other than the truth slide is on screen, from the first truth row to 10 s after the last (early, late and wrong slides in one audience-facing number) | reported, not gated (ADR-031) | lab, 7 sung takes incl. 4 re-cut "flow" takes and the live test: plan-order follower 327 s, parts map 112 s |
| **Song starts** | first slide of each song: median and p95 of `t_action − t_w`, reported separately | reported, not gated | needs about 4 heard words: ≥ 2 s by construction |
| **Move speed** | decision → command sent, p95 | ≤ 100 ms (strategy override bound, `software-strategy.md:28`) | not logged yet |

**Why two numbers.** "Correct" is what ends a trial if it fails (`.10x/adrs/009-position-tracking-over-transcription.md:18`).
"On time" is bounded by physics: a reactive system must hear words before it moves. Prediction from practice
memory (§ 4.3) is one route to "on time"; it is an experiment until step V6b shows a gain without new
wrong moves. The onset trigger (ADR-030, § 4.3) is a second route: words arm the slide, and the real start of
the next line fires it. It does not predict time. Lab takes (ADR-030): T1 = the founder singing It Is Well
(public domain) a cappella into a vocal mic, 11 changes; T2 = the public text-to-speech It Is Well fixture,
6 changes; P1 = private song P1 over a full band, 11 changes.

### 2.3 Evaluation rules (so tuning cannot fool us)

- **Corpus for the first gate:** ≥ 10 real sung songs, ≥ 120 slide changes, ≥ 2 worship leaders, ≥ 3 songs with
  an instrumental gap ≥ 6 s, desk feed. 95% on 120 changes is about ± 4 points.
- **Held-out set:** ≥ 4 songs, chosen **by hand once, stratified** (tempo, gaps, order changes), **before any
  tuning**, frozen by `sha256(truth.jsonl)`. At most one look per milestone; the tuner sees only the aggregate.
- **One frozen setting**, chosen on the tune set from a flat region (all ±1-step neighbours within 1 point).
- **Live fidelity:** the offline run uses the recognizer configuration the booth can run live, and drops the hops
  the live keep-up gate would skip at the measured seconds per pass (`listen.rs:159-166`).
- **Robustness:** one neighbour configuration (e.g. encoder context ± one step) must not be worse by more than
  1 wrong move per 10 songs.
- **Public claim of "95%":** ≥ 300 occurrences with a lower 95% bound ≥ 0.92 (song-level bootstrap).
- **Truth quality:** tap lag calibrated once on a text-to-speech fixture whose truth is exact; corrections
  subtracted.

**Approve 2:** the two-number definition, the bars and the evaluation rules (founder item FA-05).

---

## 3. Decomposition into sub-projects

| SP | Sub-project | Owner | ADR | Plan steps |
|---|---|---|---|---|
| SP1 | Recognizer speed: #59 flags, CPU decoder settings, Vulkan GPU proof; input conditioning and the Whisper prompt | Sr Eng 1 (listening) + Sr Eng 3 (platform) | 021, 030 | V0, V3, V3c, V4 |
| SP2 | Evaluation v2 and the corpus | Sr Eng 1 + founder | 022 (§ metrics) | V2, F1, V6, V6b |
| SP3 | Score-following follower with safety gates; onset trigger | Sr Eng 1 | 022, 030 | V3b, V11 |
| SP4 | Practice memory (the "listener DB") and capture privacy | Sr Eng 1 (logic), Sr Eng 3 (files) | 023 | V9, V12 |
| SP5 | Songs and licences (CC, public domain, church-owned) | Sr Eng 3 | 026 | V5 |
| SP6 | Master layout, service plan, binding profiles, trigger engine | Sr Eng 2 (integrations) | 024, 025 | V13 (later: Q12, Q13, Q18) |
| SP7 | Operator "Run" screen (Pause, reasons, certainty) | UI consultant | 024 (hotkeys) | V10, V14 |
| SP8 | uv for Python tooling | Sr Eng 3 | 027 | V1 |
| SP9 | Multi-church kit and private church folder; CLI tools zip for churches | Sr Eng 3 + engineer team | 029 | V5, V5c |
| SP10 | Safety fixes to existing dispatch | Sr Eng 2 | 025 | V8 |
| SP11 | Line budgets for all of the above | Engineering Manager | 028 | V7 |
| SP12 | Lowercase "voxxa" rename and the circle-logo icon | Sr Eng 3 | (later ADR) | V16 |

**Approve 3:** the split and owners.

---

## 4. Architecture per sub-project

### 4.1 SP1 Recognizer speed (ADR-021, ADR-030)

- **#59 fix (verified locally by the founder):** in the Windows jobs only, set
  `CMAKE_C_FLAGS_RELEASE` and `CMAKE_CXX_FLAGS_RELEASE` to `-MD -O2 -Ob2 -DNDEBUG`. Never workflow-wide or in a
  repo `.cargo/config.toml`: `-Ob2` is not a gcc/clang flag. Guards: `src-tauri/tests/release_config.rs`
  pins the two lines (it already pins `build.yml`, `.10x/handoff.md:99`); a post-build `CMakeCache.txt`
  check on Windows; new rust-cache keys (today the fixed string `release-portable-ggml`,
  `.github/workflows/build.yml:165`, `:354`), because whisper-rs-sys reruns its build only on `VULKAN_SDK`,
  `HIP_PATH`, `AMDGPU_TARGETS` changes (docs.rs build.rs, per drafts 12 and 21). Local recipe:
  `cargo clean --manifest-path src-tauri/Cargo.toml -p whisper-rs-sys --release` plus a PowerShell script that
  sets the two variables. The CMake `FORCE` route is **[UNVERIFIED]** and is not used.
- **CPU decoder first:** `words()` creates a new Whisper state per window and sets no `temperature_inc`,
  `max_tokens`, `n_threads` or `audio_ctx` (`src-tauri/src/transcription/mod.rs:179-198`). Step V3 adds, behind
  configuration (default unchanged): one reused state per recognizer, `temperature_inc = 0`, `max_tokens = 48`,
  `n_threads = physical cores`, `audio_ctx = 300` for Small and larger (Tiny keeps the default; Tiny got worse
  at 6 s, brief § 6). Logged per pass: encode ms, decode ms, fallback count (numbers only).
  **Decision rule:** Small ≤ 1.0 s/pass → CPU at a 1 s hop; ≤ 1.5 s → test a 1.5 s hop; otherwise the GPU is
  on the critical path.
- **GPU proof (Vulkan):** Cargo feature `gpu-vulkan = ["whisper-rs/vulkan"]` on the locked whisper-rs 0.13.2
  (the feature exists there, docs.rs, per drafts 10, 14, 21). Booth-only dev build: the build panics without
  `VULKAN_SDK`, and a binary that imports `vulkan-1.dll` at load time may not start on a PC without the loader
  **[UNVERIFIED]**. Target: Small, 6 s encoder, reused state, ≤ 0.8 s per pass, VRAM ≤ 1.2 GB, no visible
  stutter in Presenter. **No whisper-rs bump** (0.16 is three breaking minors away and forces fixture
  re-recording).
- **Measured in the lab (2026-11-26; GTX 960 2 GB, Vulkan build, Presenter running):** Small 0.85 s/pass on
  TTS but 2.57 s/pass on sung audio. VRAM 814 of 2048 MB with Presenter open, so Presenter is not the cause.
  Base 1.13 s/pass, but the follower dropped to 5 of 11 on T1 (Small 9 of 11). Cause found: whisper.cpp's temperature fallback re-decoded sung windows (up to 5 times). Lab, with the local-only switches: `temperature_inc` 0 gave 0.94 s/pass; adding `max_tokens` 48 gave 0.53 s/pass (2.7× real time), with the follower unchanged on T1 (9 of 11, same median). voxxa also creates a new Whisper state per window (V3 `reuse_state`). The onset
  trigger (ADR-030) needs about 1 s passes to pay off, so the decision rule above also decides whether it helps.
  The lab used the `VOXXA_TEMP_INC`, `VOXXA_MAX_TOKENS` and `VOXXA_AUDIO_CTX` env switches and voxxad streaming
  stdin (`--stdin-audio - --realtime`): local experiments in voxxa-private builds, never pushed.
- **Input conditioning (new requirement, ADR-030 § 5):** the lab feed was a WING output on an unbalanced cable
  into the PC mic jack (retasked as Line In), voice on the left channel only, floor −60 dBFS, mostly 60 Hz
  ground-loop hum (60/180/300/420 Hz). Raw singing at about −34 dBFS made the Silero VAD (enter probability
  0.60) skip almost everything: 2 Whisper passes in 200 s; the same take +12 dB gave 152. voxxa needs gain or
  AGC and a 100 Hz high-pass before the VAD, a channel select, and a level and hum check at setup. The booth
  prefers a balanced or isolated connection and, for the onset trigger, a vocal-only feed; both are deferred (FA-38, FA-39, answered 2026-11-26), so the current full-mix feed is a supported degraded condition that the input chain flags from numbers and the onset trigger yields to (ADR-030 amendment 1).

### 4.2 SP2 Evaluation v2 and corpus

- `voxxa-eval` keeps its 8 metric names (`score.rs:116-125`) and `fixtures/baseline.json`; adds `--model`
  passthrough (Tiny is hard-coded today, `src-tauri/src/cli/voxxa-eval/main.rs:188`), the columns
  `on_time_1500`, `song_start_median_ms`, `song_start_p95_ms`, `wrong_per_10_songs`, `hidden_mid_song`, and an
  `--actions <file.jsonl>` input so the private prototype's moves are scored by the same scorer (≤ 100 lines,
  row 488/600 at `.10x/handoff.md:118`).
- Truth v2 adds optional keys (`last_ms`) and event rows (`talk_start`, `talk_end`, `song_end`); the current
  parser skips unknown event rows **[UNVERIFIED: per draft 10's reading of `score.rs:271-288`]**.
- Corpus lives in the private church folder: `fixtures/{tune,heldout}/<date>-<id>/` with `audio.wav`,
  `setlist.json`, `truth.jsonl`, `intake.json` (numbers and licence class), plus `fixtures/manifest.json`
  (split, truth hashes, look count). Templates: `kit/templates/fixtures-manifest.template.json`,
  `kit/templates/intake.template.json` and `kit/templates/truth-v2.example.jsonl` (made-up times).

### 4.3 SP3 Score-following follower (ADR-022, ADR-030, ADR-031)

**Where it sits.** A new module `src-tauri/src/aligner/follower/` behind
`SmartConfig.follower: Option<FollowerConfig>` (`#[serde(default)]`, `None` by default), so every committed
session log replays byte for byte (ADR-013). The conductor keeps its states, blanking API, `manual_step`,
`jump_to_song` and the `Action` enum; with `Some`, it routes transcripts, ticks and the silence rules to the
follower (≤ 20 lines in the conductor row, 27 left).

**What it does (the prototype, ported as measured):** local word alignment of each decoded window against the
setlist expanded into occurrences (repeats are separate copies); never moves backward except along a jump edge
to a section start (section starts come from the setlist's `sections`, which the `.txt` importer fills, § 4.5);
penalises jumps faster than the singer's pace; learns pace; dead-reckons one slide when
words are unclear. **ADR-031 replaces the fixed list for songs with part labels:** the song is a map of parts and
the plan is a hint (below); songs without labels keep this plan-order follower.

**Safety gates (new, from the reviews; all are tests):**

| Gate | Rule | Finding |
|---|---|---|
| Margin | act only when the best alignment beats the second by `Δ`; off-plan moves need a stronger bar or 2 agreeing windows; otherwise hold | 14 § 4.7, 10 § 2.4 |
| Lost | no anchor for > 20 s while singing → keep the slide, amber "Not sure where we are", never blank | 10 § 2.9 |
| Gap hold (one hide rule) | a song ends only when its planned occurrences are complete **and** (talk evidence ≥ 4 s, or the learned outro has elapsed); otherwise hold; cold minimum 10 s. Replaces 3 s/5 s when the follower is on | S-27, 10 § 2.10 |
| Song start | ≥ 4 matched words and 2 consecutive windows agree; no lowered bar for the planned next song in v1; when the layout's item pointer is on a spoken item (sermon, prayer), a song start is a suggestion only | S-16 |
| Leaving the plan (ADR-031) | a move off the plan must fit the words better than anything in the slide on screen by 1.5 matched words, in 2 consecutive windows; a part is entered at its first slide; words never jump between look-alike slides (≥ 70% the same words) | live4, F1–F4 |
| Part ends (ADR-031) | the onset fires inside a part and from a verse into the chorus; after any other part, words decide, with elimination (2 s of singing that fits no part's start → the plan's next part) and an 8 s review in which words can correct any guess | live4: 2 onset fires after a chorus came 2.7 s and 5.0 s early |
| Dead-reckoning | never moves the slide in a song with part labels (ADR-031 § 5); otherwise ≤ 1 step per anchor; never across songs; never the song's first slide; anchor ≤ 8 s old; ≥ 2 on-score tokens heard in the last 6 s; not during a spoken or media item | S-23, S-24 |
| Fire-time re-check | every scheduled move re-checks Pause, Off, the 5 s operator hold, the watchdog and the gate at the moment it fires; Pause, Off or any operator action cancels all pending moves | S-22 |
| Watchdog | no automatic move when the newest decoded window is > 8 s old or decode lag > 2 hops (strategy rank 3, `software-strategy.md:28`) | S-23 |
| Operator wins | any operator move re-anchors the follower there | 10 § 2.2 |
| Credit slide | a slide of kind `credit` (set by the `.txt` importer, § 4.5) is never matched and is stepped past at song end | S-28 |
| Onset trigger (ADR-030) | time-based prediction from practice stays **off in v1** (ADR-022 § 5; V6b). The onset trigger is allowed behind config (default off) once V3b's gates pass: arm only when the slide's last 2 words are heard; the next phrase onset shows the next slide; no trigger on a slide whose final 3 words occur earlier in the slide more than an echo (4 words) away; an onset never narrows the word search; dead-reckoning only while something sounds above the long-term floor; first-slide onset only when the operator cued the song and ≥ 2 s of quiet; early hides ≤ 0.4% per fire (shown in V11 on the tune corpus and the stress set); no onset → words move the slide as before | S-21; lab T1, T2, P1 |

**Onset trigger (ADR-030, plan V3b).** A causal detector on the audio (300–4000 Hz band energy, 10 ms frames)
sees a new line start within tens of ms; words from Whisper arrive 1–3 s late. Words arm the slide; the onset
fires it. Lab at 1 s passes: T1 median delay 1.90 → 0.18 s with no wrong slide shown; T2 2.12 → 0.04 s with
no early fires; on the band mix (P1) it rarely fires and falls back to words: 4.30 → 4.01 s, 11 of 11 both
ways. It works best on a vocal-only feed and needs about 1 s passes; on a full mix it falls back to words (degraded mode; FA-38/FA-39 answered 2026-11-26, ADR-030 amendment 1). Supervised live tests use a private lab helper (local
only, never pushed) that presses Right in Presenter only while Presenter is in front (ADR-019 § 2), never Left.

**Parts map (ADR-031, plan V3d).** The first live Presenter test (live4, the founder singing It Is Well) moved
16 times in order, but 5 changes came more than 1 s early: two dead-reckoning steps while the singer held long
notes, two onset fires on a repeated "with my soul" after a chorus, one between the chorus's two look-alike
slides. A fixed list also cannot go back to a chorus. The follower now keeps a position (slide, word) in a map
of parts. Each window is aligned along the rest of the current part and along branches into the first slide of
every part (handicaps: planned next 0, chorus −0.5, same part −1.0, other −1.5 matched words), with the gates
above. Lab (Small, 1 s passes), over 7 sung takes (T1, T2, four re-cuts of T1 into other part orders, and live4):
right changes 49 → 74 of 77, wrong moves 31 → 4, early by more than 1 s 14 → 1, wrong-slide seconds 327 → 112.
The cost: when the plan is kept, the verse after a chorus shows 1–3 s after its first word instead of on the
breath (`boundary = plan`, off by default, restores it for plan-keeping leaders). P1 (band, no labels) stays on
the plan-order follower (11 of 11; the parts map got 8). A jump in the presentation app needs a driver that can
go to any slide; WorshipTools Presenter's keys cannot (FA-44).

**Logging:** a numbers-only `follow` line (`occ`, `word`, `m`, `A`, `d`, `r`, `gate`, `why`), safe for help
bundles; ADR-030 adds `why=onset`. **Budget:** if the measured port is ≤ 250 lines it goes into the evidence
row (215/500); otherwise a new row "follower (aligner/follower/)" capped at 1.3× the measured size, at most
500 (ADR-028).

#### 4.3.1 Whisper prompt (bias) strategy

An experiment (plan V3c, ADR-030 § 7), separate from the onset trigger; not adopted.

- **Today:** `voxxad --bias` passes slide text as Whisper's `initial_prompt`. It is off by default because it
  may make Whisper hallucinate (ADR-004, E6). The text is the conductor's current and next slide, so it follows
  the conductor, which reached 2 of 11 on T1.
- **Proposal:** build the prompt from the follower's position: the current slide's remaining line plus the next
  slide's first line, ≤ 40 tokens.
- **A/B on cached takes:** Base + bias vs Small unbiased, for speed (s/pass) and accuracy (changes reached,
  wrong moves, Tight on time). Base is faster (1.13 s/pass) but reached 5 of 11 on T1 unbiased (Small 9 of 11).
- **Guards:** decode only while the VAD hears voice; heard words may not run ahead of the onset and voice
  timeline; a phantom-word metric (words matched while the VAD hears no voice) on every run; off by default.
- **Decision:** adopting it needs its own decision with V3c's numbers.

### 4.4 SP4 Practice memory: the "listener DB" (ADR-023)

**Storage: one JSON file per song**, not a database (ADR-003 forbids one, `.10x/adrs/003-single-config-and-jsonl-protocol.md:17`;
its review trigger is "any file exceeds ~10 MB or two machines share state", `:41`; a practice file is
< 25 KB [est.]). Location: `<church_root>\practice\<song_fp>.json`; for new churches `church_root` defaults to
`%LOCALAPPDATA%\voxxa-church\` (local, never roaming, one church per PC). It is a sibling of the install folder
`%LOCALAPPDATA%\Voxxa\` (NTFS names are case-insensitive, so that folder is also `%LOCALAPPDATA%\voxxa\`) and of
Tauri's app-local-data folder `%LOCALAPPDATA%\com.voxxa.app\`, never inside either: install, update, uninstall and
the V16 rename never touch it. On the founder's development booth
`church_root` is `C:\dev\voxxa-church-private\pilot\`; the fake second church for the kit test is
`C:\dev\voxxa-church-private\fake-church-2\`. The key `song_fp` reuses the existing FNV-1a fingerprint
(`src-tauri/src/cli/voxxad/events.rs:131-138`) applied per song, so edited lyrics never use stale data and no
title appears in a file name.

```json
{
  "v": 1,
  "song_fp": "9c1d0e7a52b4f831",
  "title": "Amazing Grace",
  "recog": { "model": "ggml-small.bin", "window_ms": 6000, "hop_ms": 1000, "audio_ctx": 300, "backend": "cpu" },
  "runs": [
    { "date": "2026-12-06", "kind": "practice", "status": "confirmed",
      "order": [0, 1, 2, 1, 3, 1], "dwell_ms": [12400, 11800, 9900, 10100, 12000, 9800],
      "gaps_ms": [300, 600, 7200, 400, 500, 0], "outro_ms": 6100, "words_per_s": 1.9, "confidence": 0.82 }
  ],
  "plan": { "order": [0, 1, 2, 1, 3, 1], "dwell_ms": [12400, 11800, 9900, 10100, 12000, 9800],
            "max_gap_ms": 7200, "outro_ms": 6100, "words_per_s": 1.9 },
  "heard": { "2": ["zelom", "tarvin"] }
}
```

- `order`: 0-based voxxa slide indices as sung (Presenter's opening blank is not a slide here; dispatch counts it,
  `src-tauri/src/presenters/mod.rs:17-44`). `heard`: at most 20 tokens per slide that this recognizer
  configuration outputs for that slide; valid only while `recog` matches. Private (lyric-derived).
- Merge rule: keep the last 3 confirmed runs; `plan.order` = the newest confirmed run with `confidence ≥ 0.6`;
  times are medians. Only **confirmed** runs set the plan or any gate threshold (S-25).
- Writes are atomic (temp file + rename). Never in a help bundle, never printed by a tool, deleted with the song.
- **Capture (works today on the founder's development booth):** during rehearsal the operator runs
  `voxxad --device "<desk feed>" --shadow --setlist <church_root>\services\<date>-<slot>.setlist.json
  --log-dir <church_root>\sessions` (`src-tauri/src/cli/voxxad/cli.rs:31-41`). `voxxad` is a developer tool
  behind the off-by-default `cli` feature, so the installer does not contain it (`src-tauri/Cargo.toml:21-35`,
  `:121-122`). Until the V5c tools zip ships (§ 4.9), capture runs only on the founder's booth from a local build.
  The GUI writes no session log yet (#53, ADR-018 line 10), so GUI capture waits for Q14.
- **Learn (new):** `voxxad --learn <session.jsonl> --setlist <file> --out <church_root>\practice` runs a
  hindsight (non-causal) alignment with moves stay / next / back to a section start / back to the song start /
  skip one. Because "back to the song start" is a move, a song the band restarts needs no operator input. It
  prints per song only section labels (from the setlist's `sections`, § 4.5), counts and confidence (e.g.
  "V1 C V2 C B C · 6 parts · 1 unsure"), and writes runs as `proposed`.
- **Confirm (new):** `voxxad --confirm <song_fp> --practice-dir <church_root>\practice` marks the newest proposed
  run of that song confirmed, after the operator agrees with what `--learn` printed.
- **Live use:** `plan.order` becomes the expected path; `dwell_ms` the pace prior; `max_gap_ms + 2 s` the gap
  hold; `heard` extra match tokens. No file, stale `recog`, or confidence < 0.6 → cold rules.

**Capture privacy:** session logs move from roaming `%APPDATA%` (`src-tauri/src/help/mod.rs:181`,
`fixtures/README.md:277-282`) to `%LOCALAPPDATA%\com.voxxa.app\sessions\`, Tauri's app-local-data folder. They are
app data: kept 14 days (today's rotation, `src-tauri/src/cli/voxxad/main.rs:310-325`) and may be removed with the
app; what the NSIS uninstaller's "delete app data" option removes is **[UNVERIFIED]**. A rehearsal captured for
`--learn` is written to `<church_root>\sessions\` with `--log-dir`, as above. Every export (corpus, practice file,
help bundle) drops words heard outside songs; the help bundle's "include words" box defaults to unticked
(`src/simple/help.js:38` has it `checked`).

### 4.5 SP5 Songs and licences (ADR-026)

- **Lyrics:** `songs\<slug>.txt`, the existing `.txt` importer rules (`src-tauri/src/importers/txt.rs:1-9`:
  blank line or `---` = new slide, a section label line is kept off the slide, a single short first line is the
  title). The same file is pasted into the presentation app, so both have the same slides. Typos are fixed with
  the existing double-click editor and Export (`src/main.js:626-692`).
- **Section labels and credit slides (new, plan V5):** today the importer drops label lines ("Verse 1",
  "Chorus", "Bridge" …) and writes `sections: Vec::new()` (`src-tauri/src/importers/txt.rs:69`), although the
  setlist model already carries optional `sections` (`src-tauri/src/aligner/mod.rs:27-31`, ADR-016). V5 makes
  the importer keep each label line as a `sections` entry: the label plus the slides it covers, starting at its
  first slide index (the existing `Section { kind, slides }`, `src-tauri/src/aligner/evidence/sections.rs:25-28`).
  The label stays off the slide text, as today. A slide whose first line starts with `Credit:` gets
  `kind: "credit"`, a new optional `Slide` field (`aligner/mod.rs:18-21`) with `#[serde(default)]`, not written
  when absent (like `sections`), so old setlists and session logs replay byte for byte. Tests:
  `txt_section_labels_fill_sections`, `credit_slide_kind_detected`, `old_setlists_replay_unchanged`. The
  follower's jump edges and credit gate (§ 4.3), ADR-022 and `voxxad --learn`'s labels (§ 4.4) rely on this.
- **Arrangement labels (ADR-024 § 1):** `v<n>` = the n-th Verse section in file order; `c` or `c<n>` = Chorus
  (`c` with no number is the first chorus); `b` = Bridge; `t` = Tag; `o<n>` = Other. Lower case in files; upper
  case in printed output (`voxxad --learn` prints "V1 C V2 C B C").
- **Licence register:** `songs.csv`, one row per song file (template: `kit/templates/songs.template.csv`).
  Columns: `file,title,words_by,music_by,licence,licence_version,credit_line,source_url,copyright_notice,pd_basis,published_year,owner,owner_permits_sharing,modified,modification,evidence,ccli_song,presenter_slides,checked_by,checked_at`.
  Codes: `PD`, `CC0`, `CC-BY`, `CC-BY-SA`, `CC-BY-NC`, `CC-BY-NC-SA`, `CC-BY-ND`, `CC-BY-NC-ND`, `CHURCH-OWNED`,
  `PERMISSION`, `CCLI` (optional, legacy), `UNKNOWN`. Policy as data: `kit/templates/licence-kinds.csv`.
- **Public-domain basis (`pd_basis`):** `us_published_95_years` (with `published_year`; the song is public
  domain in the US when `published_year <= current_year - 96`, which `songcheck` computes each run, so the rule
  never goes stale), `author_died_70_years`, `traditional`, `dedicated_cc0`. A fixed cut-off year such as
  "before 1931" is not used: it is right only during 2026.
- **Church profile:** `church-profile.json` records `streams`, `stream_monetised`, `ccli.active`, the allowed
  licence codes and the presentation app. ND songs cannot reach "ready" when the church streams or records; SA
  and NC show amber with the reason (S-30). A `CCLI` song shows red and `UNKNOWN` amber when `ccli.active` is
  false (S-33). Sunday is never blocked; the warning names the risk.
- **Check:** `uv run kit/tools/songcheck.py <church_root>` (stdlib, PEP 723, `--self-test`): each `.txt` has one
  row and each row a file; required fields per licence (credit for CC; `pd_basis` for PD, plus
  `published_year` when the basis is `us_published_95_years`; owner for church-owned). On keys-only apps
  (WorshipTools Presenter is one) `presenter_slides` is **required** and must equal voxxa's slide count; on other
  apps it is checked when set. Prints file and field names only.
- **Credit on screen is the presentation app's job** (voxxa never projects, ADR-011,
  `.10x/adrs/011-operator-previews-allowed-never-project-lyrics.md:13`). A credit slide, if used, must be in
  both copies (S-28).
- **Records amended:** ADR-007 (lyrics may be typed by the church into its private folder; `usage.csv`
  dropped; the CCLI letter becomes conditional), ADR-019 § 7 (licence classes, not "CCLI-licensed"), ADR-003's
  file list (`usage.csv` removed). Public fixtures: PD and CC0 only until you decide otherwise (S-29). The
  public `fixtures/LICENSED_RUNS.md` template keeps only `alias, licence_class, numbers`; credit lines, titles
  and authors live only in the private `<church_root>\reports\LICENSED_RUNS.md` (ADR-026 § 10).

### 4.6 SP6 Master layout, service plan, binding profiles, trigger engine (ADR-024, ADR-025)

**Three file kinds, each with one owner:**

| File | Written by | Changes | Holds commands? | Where |
|---|---|---|---|---|
| `layout.json`, the **master layout** | tech lead, once per church | rarely | no free commands; abstract actions, cue phrases, hotkeys, graphics catalogue | private church folder |
| `services\<date>-<slot>.service.json`, the **service plan** | operator or worship leader, weekly | every service | **no**: items, songs, references, modes; may only **lower** a mode | private church folder |
| `bindings/<profile>.json`, the **binding profile** | voxxa team | per app version | yes, from a closed allowlist of step kinds, compiled into the app | public `kit/templates/bindings/` |

The master layout names **what** happens (show the prayer graphic, next slide, show Psalm 23); the binding
profile says **how** each app does it (a key chord, an HTTP call, a WebSocket message, a hint to the operator).
Changing presentation app means changing one field (`presenter.profile`), not the layout: a graphic the new
profile has no mapping for degrades to an operator hint.

**Format: JSON** (ADR-003 already makes durable data JSON; `serde_json` is in the tree; the GUI has no npm
runtime dependency). Validation `voxxad --check-layout <file>` (rules in ADR-024): references resolve; no key
named `text`, `lyrics`, `slides` or `words`; a service file may not contain steps, programs, actions, hosts or
ports and may not raise a mode; phrases are 2–8 words, letters only; no `run` key anywhere (S-01); app tokens
match `^[A-Za-z0-9_-][A-Za-z0-9._:-]{0,127}$` and are never `.` or `..` (S-06); UTF-8 without BOM.

**Compile, don't replace:** a service compiles to today's `setlist.json` (songs only) plus an item timeline, so
the conductor, follower, replay and eval are unchanged.

**Abstract actions:** `next`, `prev`, `goto(item, slide)`, `blank`, `unblank`, `show_graphic(id)`,
`clear_graphic(id?)`, `show_scripture(ref)`, `scripture_next`, `scripture_prev`, `clear_scripture`,
`clear_slide`, `clear_media` and `clear_all` (never automatic, never from a service file), `action(id)` (a named
composite in `layout.json`; a service may call it only when it is marked `service_callable` and holds no HTTP
or CLI step, meaning no step written in `layout.json` of kind `http` or `cli`; abstract actions inside the
composite (`show_graphic`, `blank`, …) are allowed, because they resolve through the compiled-in binding
allowlist, ADR-024 § 2).

**Binding action names (one `via` convention, the same wording as ADR-024 § 2):**
- `via` is a name defined per binding profile. A profile's `actions` keys are `<abstract_action>` or
  `<abstract_action>.<via>`. Examples: `show_graphic.overlay` (freeshow), `show_graphic.macro`
  (propresenter7_rest), `goto.item`, `goto.keys`.
- `hint` is a via every profile has implicitly. It shows the operator the graphic's name and never sends
  anything.
- A layout graphic entry is per profile: `graphics.<id>.<profile> = {"via": ..., ...}`. Its `via` must exist in
  **that** profile, not in every profile.
- A missing entry for the active profile falls back to `hint`, and `--check-layout` prints a warning, not an
  error. So "changing app = changing one field" stays true: a missing mapping degrades to a hint.
- A key that exists runs only after its contract test passes; until then the driver turns it into a hint
  (ADR-024 § 5).

**Hotkeys, API calls, command lines (what "paired" means):**
- *Hotkeys into voxxa* (operator override while Presenter is in front): global hotkeys registered only while
  Songs is on. Defaults F13–F24 for pedals, Stream Deck and Companion (no keyboard has them); keyboard F-keys
  only for apps other than WorshipTools (its shortcuts page documents F6–F9), after a registration conflict
  check; never Ctrl+Alt+arrows; a key injected after a hotkey waits until
  the physical modifiers are released (≤ 300 ms) or is refused; no hotkey stops listening or turns voxxa Off
  (S-13).
- *API into voxxa*: the existing 127.0.0.1 bearer-token API (`src-tauri/src/http_api/mod.rs:76-86`) gains
  `POST /api/v1/pause`, `/resume`, `/accept`, `/dismiss`. Any CLI drives voxxa through it (PowerShell
  `Invoke-RestMethod`, curl, Companion).
- *Keys and API calls out to the presentation app*: binding profile steps, compiled in, allowlisted per profile
  (S-04). In v1 only the existing driver methods run (next, prev, goto, blank, unblank); graphic and scripture
  steps run only for a profile whose steps were verified on a live install; everything else is an operator
  hint. On WorshipTools every graphic and scripture action is a hint.
- *Hotkey sequences out to the presentation app*: a `keys` step sends up to 4 chords per action (ADR-025 § 7).
  `keystroke.universal.json` and `keystroke.worshiptools_presenter.json` show one example `keys` step each,
  marked **[UNVERIFIED]** per app until the presenter test card's row T8 confirms the chord (Home, or Right
  Right). Founder item FA-35.
- *Command lines out to other programs*: **not built in v1** (founder item FA-33). When built (ADR-025): off by default; a
  `programs` allowlist in `layout.json`; absolute path; SHA-256 of a voxxa-owned copy; typed placeholders only,
  never heard words; no `.bat`, `.cmd`, `.vbs`, `.js`, `.hta`, `.lnk`; the only interpreters are
  `powershell.exe -NoProfile -NonInteractive -File <pinned .ps1>` and `osascript <pinned .scpt>`; cleared
  environment; 2 s timeout; approval only in the GUI, recorded outside the church folder (in the trust store
  `%LOCALAPPDATA%\com.voxxa.app\trust\`) with an HMAC
  (S-01, S-03, S-05).

**Trigger engine v1:**
- Inputs: follower events (song start, slide, song end), the item pointer (moved by song starts, song ends and
  the operator), the cue matcher, and later scripture (Q16–Q18).
- **Cue matcher:** phrase tokens in order, contiguous with ≤ 1 inserted word, inside the last 6 s of words; tokens
  of ≥ 5 letters may differ by one edit; cooldown 120 s per cue, 10 s for any graphic (S-17).
- **Arming:** cues only inside a `spoken` item that lists them, when no song anchor has been seen for ≥ 30 s;
  disarmed on `media` and `announcement` items; never armed by conductor state (S-15, S-19).
- **Modes:** `off`, `suggest`, `auto`. v1 ships cues and scripture at **suggest only** (founder item FA-34: you
  asked for autonomous changes on scripture and key words, so this deviation is yours to confirm). Auto needs the exposure
  bar (0 wrong fires in ≥ 300 armed opportunities or ≥ 30 h of armed talk audio from CC-licensed public
  sermons, never private services) and a typed confirm in `layout.json` (S-18, S-20).
- **Priority:** operator 100 > safety hold 90 > follower 80 > scripture 60 > cue 40 > item start/end 30; one
  action per output at a time; lower priorities held 5 s after any operator action.
- **Never while singing:** a cue or scripture trigger in a song becomes a suggestion, and suggestion cards
  appear only between items.

### 4.7 SP7 Operator "Run" screen (U1 subset)

Simple mode stays the default view (ADR-017). U1 adds: Pause/Resume (instant; F15 (pedal), or Space when
voxxa's window is in front), Back/Next through
voxxa (F14/F13), a one-line reason for every move ("Heard the words", "By pace (words unclear)", "You pressed
Next"), a certainty word (Sure / Not sure / Lost), an Off confirm while singing was heard in the last 5 min, and
on keys-only apps a Resume card that asks "Presenter shows slide __ of __ (not counting its blank)?" with **no
default button** (S-14), plus "screen is ahead / behind" hotkeys (F18/F19). Accessibility: a new
`--accent-strong` token (white on `#6366f1` is 4.47:1 and fails AA), a `--field-border` token (1.32:1 fails),
one `:focus-visible` ring, 44 px live targets (draft 13 § 4.3). Showing the first line of the current and next
slide on Run is founder decision FA-21 (amends ADR-017; ADR-011 allows operator previews).

**Round 3 (draft): the minder controls and screens (R8–R10; FA-58, FA-60, FA-61, FA-62; plan V13a, V13s, V14).**

- **Controls:** **HOLD** (F15, toggle: pause following, keep listening and logging); **STOP FOLLOWING** (F20, no
  confirm: dispatch off for the rest of the service, listening and logging continue; post-state: the current slide
  is held and nothing is sent); **Back one** (F14) on the pedal or Companion page. HTTP routes on 127.0.0.1:
  `/pause`, `/resume`, `/panic`, `/state`. Off stays in the mode switch. No Escape (ADR-025 § 7 refuses it; voxxa is
  never the front window). FreeShow's output lock is the external panic path. A Companion or Stream Deck page is deferred
  (FA-59 answered "not now", 2026-11-27); if wanted later it is a docs recipe, not voxxa code.
- **Badge words:** "voxxa moves it" / "voxxa suggests" / "You move it". A manual move re-anchors the follower, which keeps going from the
  operator's slide after a short grace (FA-61, 2026-11-27; demotion is an off-by-default switch); on Presenter a manual move is observable only through voxxa's own hotkeys, one reason the
  no-minder gate is FreeShow-only. All Presenter output reaches the stream as a lower third (FA-29), so voxxa's own windows are never on a captured display, and the FreeShow switch needs the same path (FA-75).
- **NOW / NEXT** on Run: the current slide firm, the predicted next change provisional; an amber hold sentence when
  voxxa holds.
- **Stage strip:** voxxa's own strip in three states (following / holding / you're driving). The heard-words line
  is **removed** (today `stage.html:31`, `src/stage.js:52-55`; ADR-011): a ticket now, the fix in V14. FA-29 (the
  stage output is never captured into the stream) is confirmed before any stage display is turned on. voxxa never
  writes FreeShow stage text.
- **Health check (V13s, ADR-034):** one page in `src/health/` with six rows and one top line, run from Simple mode
  when Songs is chosen and by `voxxad --check-ready` (the rows as `plan-round3.md` V13s defines them). Every
  sentence is approved word for word on the V10 mock (FA-62). "Ready, with a note" may start Songs; only a red
  slides-app row blocks.

### 4.8 SP8 uv (ADR-027)

Install uv on the booth (`winget install --id astral-sh.uv -e`); Python 3.9 stays untouched. Repo scripts are
stdlib-only and get a 4-line PEP 723 header; they still run with plain `python3`. No root `pyproject.toml`, no
uv in CI (CI keeps `python3`, `.github/workflows/ci.yml:149`). Delete the dead prototype (`main.py`, `voxxa/`,
`requirements.txt`: 337 lines, 5 unpinned packages). The two INTEGRATIONS bridges use `urllib.request` plus a
PEP 723 header. The private lab (follower prototype, eval scripts) becomes a uv project with a lock file in the
private folder (`C:\dev\voxxa-church-private\lab\` on the founder's booth, shared by the pilot and the fake
church; `<church_root>\lab\` at any other church), Python 3.12, `exclude-newer` 7 days. The minimum uv version is
the one V1 installs on the booth (§ 4.12).

### 4.9 SP9 Kit and private folder (ADR-029)

Public `kit/` (draft in `out/kit/`): README, templates (layout, service, six binding profiles, song, licence
register, licence kinds, church profile, checklists, and for evaluation and consent
`fixtures-manifest.template.json`, `intake.template.json`, `truth-v2.example.jsonl` and
`recording-consent.template.md`), eight prompts, five skills, `private-folder.md`. Private: `<church_root>`
(default `%LOCALAPPDATA%\voxxa-church\`, beside the install and app-data folders, never inside them, § 4.4;
`C:\dev\voxxa-church-private\pilot\` on the founder's booth) with
`songs\`, `songs.csv`, `church-profile.json`, `layout.json`, `services\`, `practice\`, `sessions\`,
`recordings\`, `inbox\`, `fixtures\`, `reports\`, `lab\`, `consent\`. Not a git repository; dated weekly copies
to one BitLocker-encrypted drive (`robocopy "<church_root>" "E:\voxxa-backup\<yyyy-mm-dd>" /E /XD sessions inbox`, keep the last 8,
test one restore every quarter; never `/MIR`, which copies an accidental delete into the backup); an `.ignore`
file holding `*` at its root so code-search tools skip it. Guards: media guard and `.gitignore` extended to the
new private file types (S-46); a lyric 4-gram scanner and a private deny-file check before anything moves to a
public path (S-43, S-45); founder CODEOWNERS on `kit/prompts/**`, `kit/skills/**` and
`kit/templates/bindings/**` (S-47); a kit agent
settings example that denies personal-data connectors during church setup and shell reads under
`<church_root>` (a seatbelt, not a lock: the real controls are never starting an agent in `<church_root>`,
skills that never ask for content, and numbers-only tools). Acceptance: a fake second church
(`C:\dev\voxxa-church-private\fake-church-2\`) is set up from the kit alone with zero edits to public files.
"Windows booth only until Q24/Q26."

**CLI tools for churches (V5c).** The kit's practice capture, `.wav` intake and layout check need `voxxad` and
`voxxa-eval`. Both are developer tools behind the off-by-default `cli` feature, and the installers leave them
out on purpose (`src-tauri/Cargo.toml:21-35`, `:121-122`; a church booth must not run test builds). Step V5c
builds them with `--features cli` in the Windows release job and attaches them, signed, as a separate release
asset `voxxa-tools-<version>-windows-x64.zip`. The GUI installer, the updater and `latest.json` do not change.
Until V5c ships, practice capture, `.wav` intake and `--check-layout` run only on the founder's development booth
from a local build; the kit README says so plainly. Founder item FA-31 chooses the delivery: (a) the separate
zip (recommended), (b) a Tauri `externalBin` sidecar inside the installer, (c) wait for GUI capture in Q14.

### 4.10 SP10 Safety fixes to existing dispatch (ADR-025)

- **Per-key position:** update `sent.global` by ±1 after each key that succeeds; a part-failed sequence sets
  `position_uncertain`; no automatic move until the operator confirms where the app is (S-10).
- **Every keystroke profile needs a target:** the universal profile captures the presentation window's process
  when Songs starts; keys go only there, otherwise amber; `voxxad --presenter keystroke` refuses to start
  without `--target-exe` (S-09; today only WorshipTools is guarded, `src-tauri/src/presenters/keystroke.rs:59-63`;
  `docs/INTEGRATIONS.md:260-266`).
- **Elevated target:** if Presenter runs as administrator and voxxa does not, show red: keys would be dropped
  silently (S-12) **[UNVERIFIED API cost]**.

### 4.11 SP11 Budgets (ADR-028)

"Everything else" has **1 line left** (`.10x/handoff.md:102`, `:111`), and caps only go down
(`scripts/loc_budget.py:6`). Proposal: move `dispatch.rs`, `listen.rs`, `http_api/` and `commands/` into a
recorded "wiring" row capped at their measured size + 150, and lower "everything else" by the moved lines. New
rows land with their code: follower (≤ 500, or inside evidence), practice 300, layout 300, Run GUI 350, kit tools
(Python) 300. The round-3 rows (onset, input chain, health check) land at three times their estimates, 750 / 600 / 450, by
the founder's FA-54 rule (2026-11-27; ADR-028 amendment 1); whether that rule extends to the rows just listed is FA-74. The voxxad row (552/600, 48 left) holds only flag parsing and wiring: V5c `--version` ≤ 3,
V8 `--target-exe` ≤ 5, V12 `--learn`/`--confirm`/`--out`/`--practice-dir` ≤ 20, V13 `--check-layout` ≤ 10 (≤ 38
in all, 10 spare); their logic goes in the practice and layout rows. The conductor row (27 left) takes the
follower routing (≤ 20) and `Slide.kind` (≤ 3); the importers row takes V5 step 9 (≤ 40). ADR-028 § 3 lists
every share. In total about +1,250 Rust, +350 GUI and +300 Python lines of new caps, against about +3,450,
+2,050 and +1,800 in the four 10x drafts (review 21 § 2.1). Core engine stays under the strategy's 3k (`software-strategy.md:30`): conductor 1,200 + evidence
500 + follower ≤ 500 + practice 300 = ≤ 2,500 in caps.

### 4.12 Keeping voxxa current ("latest updates")

The founder's request says "latest updates" and nothing more. This plan reads it as five concrete things.
Founder item FA-32 asks whether that reading is right.

1. **App updates for churches** use the existing signed updater (ADR-020: operator-initiated only, checked
   with the updater key, refused while voxxa is listening) on the stable channel; channels come in Q22. voxxa
   never installs an update between Saturday 18:00 and Sunday 14:00 local time, or while Songs is on. The
   weekend window is new (a ticket in plan § 5, in the updater row, 52 lines left, `.10x/handoff.md:112`). The
   V5c tools zip carries the same version as the app; a church replaces it when it updates the app.
2. **Dependencies:** a monthly agent-run window updates `Cargo.lock` and the GitHub Actions SHAs. `cargo deny`
   must pass, the eval gate must hold, and the founder approves the PR. whisper-rs stays at 0.13.2 (§ 9). This
   replaces the earlier "no upgrades for their own sake" ruling (adjudication § 3 #22) if FA-32 (a) is chosen.
3. **Whisper models** are pinned by SHA-256. A model change goes through the eval gate. Today the model download
   checks no hash (no SHA-256 code in `src-tauri/src` on `866b51e`), so pinning is a ticket in plan § 5.
4. **Kit versions** are tagged `kit-vX.Y.Z`. `church-profile.json` records `kit_version`. Every kit release has
   upgrade notes. A church upgrades by copying the new templates (following the upgrade notes; templates are
   never applied over its files automatically) and running `kit_check`.
5. **uv:** the minimum uv version is the one the booth PC installs in V1 (`uv --version`). V1's PR records it in
   `kit/README.md`.

**Approve 4:** the architecture (or per sub-project: "approve 4.3", "change 4.6: …").

---

### 4.13 Live-output staging (round 3 draft; R2, C12; ADR-035, FA-48)

| Stage | What runs | Who drives the screen |
|---|---|---|
| 0 | voxxa in shadow on Presenter (today) | the volunteer |
| 1 | FreeShow installed with **no output assigned**; voxxa drives it in shadow on weekdays, then during Sundays while Presenter stays live; read-back logged ("dark Sundays" Jan 10, 17, 24, 2027) | the volunteer, on Presenter |
| 2 | FreeShow live, a volunteer operates it; voxxa in shadow, scored from read-back (double-build Sundays; Thursday 18:00 rollback rule; Presenter maintained four weeks) | the volunteer, on FreeShow |
| 3 | one promoted song autonomous | voxxa for that song; minder at the PC |
| 4 | all promoted songs | voxxa; minder at the PC |
| 5 | no volunteer in the booth; the leader holds a Companion or phone HOLD (gate 2, FA-52) | voxxa |

The switch is per service, whole service, never one song at a time (two apps cannot own the projector within one
service). Window Jan 10 – Mar 21, 2027; Easter Mar 28, 2027 runs on whatever stage Mar 21 reached. Entry to
stage 2 needs FA-48: ≥ 2 dark Sundays with FreeShow's read-back agreeing with voxxa's own commanded slide on
≥ 99% of polls and 0 `position_uncertain`. That line measures the driver (Presenter has no read-back on a dark
Sunday); follower correctness is scored as in stage 0 (shadow plus tapping) until stage 2, where the volunteer's
FreeShow moves become operator-timed truth (FA-66). Rollback (`round2/51` § 2.6): at stage 1, FreeShow crashing Presenter, VRAM > 1,800 MB or
projector stutter closes FreeShow for that service; at stage 2, a FreeShow-caused visible failure (black
projector, wrong song, lost stream) is declared by Thursday 18:00 and Presenter runs the next Sunday.

## 5. Data flow

```
 WEEKDAY (tech lead, booth PC)                    REHEARSAL (Sunday 8:00)                  SERVICE (10:30)
 songs\<slug>.txt + songs.csv ──songcheck──┐      desk feed ─► voxxad --device --shadow     desk feed ─► audio ─► VAD
 layout.json (master layout) ─check-layout─┤                    │ session-*.jsonl              │
 services\<date>-<slot>.service.json ──────┤                    ▼ (church_root\sessions, 14 d)  ▼
                                           ▼      voxxad --learn ─► practice\<fp>.json     recognizer (Small, 6 s,
                          compile ─► setlist.json   (proposed) ─► operator confirms        1 s hop; CPU or Vulkan)
                                  + item timeline                    │                          │ words + times
                                                                     └──────────────► follower (plan, pace, gap hold,
 ANY .WAV (founder)                                                                      heard tokens) ◄─ setlist
 inbox\ ─ffmpeg 16 kHz mono─► recordings\ or fixtures\{tune,heldout}\                            │ Goto/Blank/Unblank
        ─tap truth─► voxxa-eval --full --model small ─► numbers-only table                       ▼
                                                                          trigger engine ◄─ item timeline, cue matcher
                                                                                │ priority, arming, modes
                                                           operator hotkeys ───►│ (operator always wins)
                                                           HTTP API ───────────►▼
                                                                      binding profile ─► driver ─► presentation app
                                                                          (keys / HTTP / WS / hint)
```

Nothing crosses the booth PC's boundary: no lyrics, words, audio or church identifiers go to GitHub, Drive,
e-mail or any service (strategy rank 4, `software-strategy.md:29`). Agents see counts, section labels, file
names, line numbers and exit codes only.

**Approve 5.**

---

## 6. Error handling (fail safe, operator first)

| Situation | Behaviour | Where |
|---|---|---|
| Recognizer stalls or lags | no automatic move after the newest window is > 8 s old or lag > 2 hops; amber; `why=watchdog` | ADR-022 |
| Follower unsure | hold the slide; amber "Not sure where we are. If the screen is behind, press Next." | ADR-022 |
| Follower lost > 20 s while singing | keep the slide (never blank on lost); keep aligning the whole song | ADR-022 |
| A key sequence fails part-way | position counted per key; `position_uncertain`; no automatic moves until the operator confirms | ADR-025 |
| Wrong window in front | keys not sent; amber "Bring <app> to the front." (all keystroke profiles) | ADR-025 |
| Presenter elevated | red "Presenter runs as administrator; voxxa's keys can't reach it." | ADR-025 |
| Operator presses Pause, Off or any move | pending scheduled moves cancelled; lower-priority triggers held 5 s | ADR-022, ADR-024 |
| Layout or service invalid | refused at load with line number and one plain sentence; the previous valid plan stays; songs still follow | ADR-024 |
| Practice file missing, stale or low confidence | cold rules; "Not rehearsed: voxxa may be a little late" | ADR-023 |
| Song not in the setlist heard | operator prompt only; never projects anything | ADR-022 |
| GPU unavailable (dev build) | start on CPU and say so in diagnostics | ADR-021 |
| Licence missing or risky | amber or red on the readiness checklist; Sunday is never blocked | ADR-026 |
| Multi-step app move fails (API apps, later) | blank → steps → unblank only after all succeed; otherwise stay blank, red, position unknown | ADR-025 (S-08) |
| FreeShow jump not confirmed by read-back (round 3) | one re-read; still different → `position_uncertain`, hold, amber; no further automatic moves until the minder confirms | ADR-032 |
| FreeShow API answers without a password (round 3) | refuse to connect unless `require_password` is set to false (default true); red; the token is never logged or bundled | ADR-032, FA-55 |
| Watchdog: audio frames or alignment updates stall (round 3) | the off-thread watchdog (four signals) holds following (one of its five hold reasons) and raises amber; re-arming needs a green check | ADR-034 |
| Minder presses HOLD or STOP FOLLOWING (round 3) | HOLD pauses following, listening continues; STOP: dispatch off for the service, current slide held, nothing sent | ADR-034, FA-58 |
| Booth reached over RDP on a service day (round 3) | health check red row (`SM_REMOTESESSION`); FA-67 policy: no RDP Sat 18:00 – Sun 14:00 | ADR-034 |
| Off-plan move on a keys-only app (round 3) | a suggestion card, never a Left burst (refused in code) | ADR-032 (Tier B) |

**Approve 6.**

---

## 7. Testing and evaluation strategy

1. **Gates on every PR (unchanged):** fmt, clippy, tests, `cargo deny`, budgets, replay determinism, the eval gate
   (no metric worse than `fixtures/baseline.json`, never loosened silently), media guard.
2. **New PR gates:** the private-data guard for new file types (S-46); `kit_check.py` on `kit/` (lyric keys,
   media, private paths); `release_config.rs` pins for #59.
3. **Accuracy gate (V6, then V11):** § 2.3 rules, held-out set, frozen setting, live-fidelity hop dropping, a
   neighbour configuration.
4. **Safety tests before any autonomy default turns on** (from review 20 § 5, made-up words only):
   `partial_key_sequence_updates_sent_per_key_and_flags_uncertain`, `every_keystroke_profile_requires_a_target`,
   `elevated_target_is_refused`, `hotkey_waits_for_modifier_release`, `cues_disarmed_through_mid_song_instrumental`,
   `spoken_next_song_quote_does_not_start_song` (TTS reading of public-domain hymn lines over silence → no Goto,
   no Unblank), `media_item_disarms_audio_triggers`, `scheduled_move_cancelled_on_gate_close_pause_off_operator`,
   `no_move_when_newest_window_older_than_8s`, `no_dr_without_on_score_tokens`,
   `worshiptools_no_song_end_right_on_last_song`, `layout_refuses_run_key_and_shell_programs`,
   `service_file_cannot_raise_modes`, `on_start_refuses_clear_all`, `apptoken_rejects_braces_and_dot_segments`,
   `session_logs_in_localappdata`, `help_bundle_words_default_off`, `private_data_guard_refuses_new_file_types`.
   Tests for CLI and HTTP steps (`trust_record_outside_church_root_and_hmac_checked`,
   `multi_step_goto_blanks_until_all_steps_succeed`, `bindings_are_allowlists`) ship with those features.
   The onset trigger's tests (ADR-030) ship with its module in V11: `onset_arms_only_on_last_two_words`,
   `no_onset_on_slide_with_repeated_closing_words`, `onset_move_keeps_word_search_width`, `no_dr_through_quiet`,
   `onset_floor_starts_from_real_audio`, `first_slide_onset_needs_cue_and_quiet`.
5. **Synthetic stress set (public, deterministic):** public-domain speech (e.g. LibriVox) and CC0 pads mixed with
   ffmpeg: ≥ 3 h talk over pad and ≥ 1 h hum over pad → 0 dead-reckoning fires, 0 shown-during-talk; plus a
   10 s instrumental gap, an unplanned chorus repeat and ±10% tempo copies of public-domain songs.
   **Flow set (ADR-031):** each tapped take is also re-cut into other part orders with the plan unchanged (an
   extra chorus, a skipped verse, the chorus first, the last line repeated; 30 ms crossfades at the quietest
   moment before each part; truth mapped). Every follower change reports right changes, wrong moves, early
   moves and wrong-slide seconds on the flow set beside the planned-order takes. Re-cuts share the source take's
   singer and room, so founder takes with real detours (V3d) are still needed.
6. **Shadow Sundays before autonomy:** 2 consecutive services in shadow meet the "correct" bar before the
   follower default flips (one PR, re-recording the 6 public logs). In shadow mode on WorshipTools voxxa cannot
   see the operator's moves (no read-back, ADR-019), so each shadow Sunday is scored like this:
   1. Record the desk feed to a `.wav` during the shadow service. This needs founder item FA-06 option (b):
      consent for service audio; the talk spans are muted after step 4 has scored the replay, before the
      recording is kept.
   2. Tap truth afterwards (`<repo>\fixtures\tools\tap-truth.ps1`, which writes `truth.jsonl` next to the wav
      it is given).
   3. Replay the recording offline on the booth PC with `voxxad --stdin-audio <wav> --shadow`, with
      `--setlist <church_root>\services\<date>-<slot>.setlist.json`, the same build and the same configuration
      as the live run, and `--realtime`, which feeds the audio at live speed so the keep-up gate drops the hops
      it would drop live (§ 2.3; `src-tauri/src/cli/voxxad/main.rs:44`, `:249-255`).
   4. Score that replay's actions with `voxxa-eval --actions` against the tapped truth.
   5. The live shadow session log from `voxxad --device … --shadow` is a cross-check only: its clock is not
      aligned to the recording.

   Operator notes are context, never the score.
   **Round 3 (draft): two gates, not one (C13, FA-52).** Gate 1 is the above: 2 scored shadow Sundays move songs
   to songs mode **with a minder at the PC**. Gate 2, later, removes the person: ≥ 4 consecutive green Sundays,
   ≥ 300 scored occurrences, 0 early moves beyond the line (0.3 s on rehearsal truth, 1.0 s live), 0 spurious
   presses, two volunteers passed the minder card, Tier A (FreeShow) only. FreeShow read-back may serve as
   operator-timed truth for the Correct bar, never for Tight (FA-66). Thresholds are tightened, never loosened.
7. **Practice memory:** on ≥ 5 rehearsal → service pairs, the learned order equals the service truth order in
   ≥ 90% of songs; with practice files the held-out eval is better on recall and median latency and worse on
   no metric.
8. **Kit:** a fake second church at `C:\dev\voxxa-church-private\fake-church-2\`, built from the kit alone (the
   new `fixtures-manifest.template.json`, `intake.template.json`, `truth-v2.example.jsonl` and
   `recording-consent.template.md` filled with made-up values), passes `songcheck`, `--check-layout` and
   `kit_check`, and runs `voxxad --version` and `voxxa-eval --help` from the V5c tools zip with no repo checkout.
9. **Usability:** a clickable Run mock reviewed by the founder (V10); the five-volunteer round stays with Q11 as
   planned; a later round for new screens is deferred.

**Approve 7.**

---

## 8. Security and privacy

| Threat | Control | ADR |
|---|---|---|
| A shared layout file runs a program when a phrase is heard | no `run` key; CLI steps not built in v1; later only through an allowlist, hashes, GUI approval and HMAC trust records outside the church folder | 025 |
| An e-mailed service file raises automation or clears media | service files may only lower modes; `on_start`/`on_end` limited to show/clear graphic and blank/unblank; no `clear_all`, no non-adjacent goto, no non-callable action | 024 |
| Keys typed into the wrong window | target process for every keystroke profile; `--target-exe` for `voxxad`; own-window guard stays | 025 |
| Wrong slide after a partial key sequence | per-key accounting; `position_uncertain` | 025 |
| A cue or song start fires during prayer or sermon | arming by item pointer; song start needs two windows and is a suggestion during spoken items; never while singing | 022, 024 |
| Heard words reach a command | no placeholder for speech, titles or text anywhere | 024, 025 |
| Prayer words leave the booth | session logs in `%LOCALAPPDATA%\com.voxxa.app\sessions\` (local, 14 days); exports drop talk words; help-bundle words off by default; numbers-only tools | 023 |
| Lyrics or church identifiers reach the public repo | media guard + new file types; lyric 4-gram scan; deny-file; aliases in records; founder review of the first kit PR; scrub of existing identifiers and song titles (FA-12), merged before the deny-file scan gates `.10x/` and `fixtures/` | 029 |
| A kit PR turns skills into an exfiltration path | founder CODEOWNERS on `kit/prompts/**`, `kit/skills/**` and `kit/templates/bindings/**`; churches use a tagged kit; skills say "never send data to external services"; connector deny list | 029 |
| Licence breach on a livestream | `streams` flag; ND red, SA/NC amber when streamed; counsel before adopting ND/SA | 026 |
| Copied rights from another church | imported rows arrive with `checked_by`/`checked_at` empty; not "ready" until the church records its own evidence | 026 |
| Recordings with pastoral prayer kept for ever | talk spans muted before a recording is kept; consent from every recorded speaker (`kit/templates/recording-consent.template.md`, reviewed by the church's counsel); retention date per recording; BitLocker on the booth PC | 023, 029 |
| Booth display duplicated to the projector or stream | readiness checklist item now; detection spike later **[UNVERIFIED that Tauri can detect it]** | 024 |

Controls that stay as they are (verified by review 20 § 3): the inbound API binds 127.0.0.1 with a bearer token
and refuses browser origins (`http_api/mod.rs:85-86`); the webview has only `core:event:default`; secrets are
never persisted; the app log records word counts; the WorshipTools key guard never sends F10–F12, Up, Down,
numpad ± or `.`.

**Approve 8.**

---

## 9. What is deferred (and what would bring it back)

| Deferred | Why | Revisit when |
|---|---|---|
| SQLite song library and practice store | ADR-003; < 25 KB per song; no cross-machine state | > ~500 songs, two machines share state, or a live query files cannot serve |
| In-app song library and lyric editor | the strategy ruled out lyrics editing (`software-strategy.md:30`); Notepad + `songs.csv` works in days | you require entry inside voxxa (FA-14), or church #2 asks |
| Command-line steps to other programs | highest-risk feature; no church needs one yet | a named need, after ADR-025's controls are built and tested (FA-33) |
| Executing graphic/scripture steps on API apps | no live install to verify on; three drivers disagree with vendor docs (W1–W3, draft 11 § 0.1) | a church on FreeShow/ProPresenter/OpenLP with a test install (Q13) |
| Cue and scripture auto mode | exposure bar not met | ≥ 300 armed opportunities or ≥ 30 h armed talk with 0 wrong fires (FA-34) |
| Prediction ("on time" by pace) | untested; early-hide risk | V6b passes (≤ 0.4% early hides per prediction, +20 points on time) |
| Pitch-based singing gate, heard signatures beyond 20 tokens, room calibration | no evidence of gain yet | V6/V11 show DR firing in talk, or C4-class churches |
| whisper-rs 0.16 bump | breaking; forces fixture re-records | the Vulkan proof fails on 0.13.2, or a security advisory |
| GPU installers for other churches; GPU CI matrix | loader start risk; separate artefact doubles signing | after Q21 and a no-loader start test |
| Draft-truth tool (torch, Demucs, wav2vec2) | 2–3 GB, accuracy on singing unproven | tapping becomes the measured bottleneck |
| sherpa-onnx probe | already Q20–Q21 | Q20 |
| Prepare, Rehearse and Review screens; usability round 2 | the features they show must exist first | after V12/V13 ship |
| Full kit (manifest generator, semver, 16 prompts) | one church; would rot | church #2 |
| `usage.csv` | CCLI reporting; the church is leaving CCLI | a church still on CCLI asks |
| Keyboard hook counting operator arrows | keylogger-shaped | not planned; read-back (#185, Q12, moved from Q11 by FA-37) instead |
| Lowercase rename and circle-logo icon | brand decision; affects exe, install folder, updater, winget, icon set | before the first full v0.1.0 publish (V16; FA-24, FA-36) |
| CLI tools inside the installer (`externalBin` sidecar) | changes the installer, the updater artefacts and the signing set | FA-31 option (b), or Q14's GUI capture replaces the need |

**Approve 9.**

---

## 10. Open founder decisions

All 72 items are deduplicated and numbered once in `founder-action-items.md` (FA-01 … FA-72), ordered by
urgency, each with options, a recommendation and why. FA-31 … FA-37 came from the critic's round 1
(`adjudication.md` § 8); FA-38 … FA-42 came from the onset lab (ADR-030); FA-43 … FA-45 came from the first live
test and the parts map (ADR-031); FA-46 … FA-72 came from the round-2 autonomy review (`round2/50-adjudication.md`
§ 5). The ADRs to accept are 021–032, 034 and 035 in `adrs/` (033 is reserved for an aligner row, written only after
V3f and V6 both pass).

## 11. What was not verified

The private prototype and tools (size, code, whether they model blanking); GTX 960 Vulkan behaviour and ggml
efficiency on Maxwell; whether a Vulkan build starts without `vulkan-1.dll`; whether rust-cache hashes `CMAKE*`
variables; WorshipTools Presenter's use of F6–F9 (documented on its shortcuts page; confirm on the test card,
T7), its dialogs' window structure and a footer/credit field;
vendor wire formats W1–W5; whether Tauri can detect a duplicated display; legal readings of CC NC/ND/SA and the
sync clause; the exact timings API of whisper-rs 0.13; whether SignPath's artifact configuration can sign the two
tools inside a zip or needs them as loose files; whether `winget_manifest.py` picks up only the installers; every
cost and calendar figure (bands from `.10x/specs/2026-11-11-prompts-10x-team.md:9`).
