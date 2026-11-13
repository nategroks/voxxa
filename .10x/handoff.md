# Current Handoff

## User decisions for Q4 (2026-11-12, before the unattended run)
- **Line budget:** move `src-tauri/src/transcription/` (316 lines) and all new recognizer code (trait, rolling window, `Hypothesis`) into one new row, "recognizer (transcription/ + recognizer code)", capped at **700** lines and excluded from "everything else". Record it as a deviation in the ADR and in `scripts/loc_budget.py`. "Everything else in src-tauri/src" stays at 2500; no other budget is raised.
- **Pre-approved:** push `q4/base` and open a draft PR against `main`; fix CI forward; merge that PR into `main` once Q4's exit criteria are met and every CI job is green; file Q4 tickets as GitHub issues (label `q4`); re-record `src-tauri/tests/fixtures/voxxad/session-pd-hymns.jsonl` with Whisper Tiny on the public-domain WAVs if the hop changes what the conductor hears (the reason goes in the commit).
- **Q4 owns Q3-02 (#46):** the GUI event mapping becomes pure and tested when the hop rewrites the shared loop.
- **ADR:** ADR-012 is still reserved for the cut list. Q4 amends ADR-013 or writes ADR-014.
- **QA mode** (as in Q2 and Q3): no microphone, no keystroke dispatch; `--shadow`, `--presenter none` or a local mock only; public-domain WAVs; `ggml-tiny.bin` is present.

---

## From: Q4 → To: Q5 (voxxa-eval and the first fixtures)
**Date:** 2026-11-13
**Phase:** Phase 4 Implementation done on `q4/base` (head a4bae0e plus the state commit); CI, PR and merge into `main` by the orchestrator (pre-approved) → Q5

### What Q4 did
- Replaced the recogniser closure and the 5 s speech batch in the shared loop (`listen::run`) with ADR-004's `Recognizer` trait: Whisper Tiny over a 5 s window, decoded every 1 s (hop). ADR-014 (Accepted, with dated notes) records the design; ADR-013 has a dated amendment.
- Ran as two lanes (`q4/10x`, `q4/ponytail`) with a cross-review, a comparator (base 10x, three ponytail grafts, four review fixes), a finisher and three verifier rounds (rounds 1 and 2 failed and were fixed with tests; round 3 passed with two verifiers): `reviews/2026-11-12-ponytail-q4.md`.
- Closed Q3-02 (#46: the GUI event mapping is pure and tested) and Q3-08 (#52: no goto to the slide still showing after inter-verse silence).
- All 7 gates pass on Windows at a4bae0e; 142 tests (55 unit + 87 integration). Only the real-model 10 s WAV test needs `ggml-tiny.bin`; it skips without it, as in CI.
- Record: `decisions/sde/church-autoadvance.md` ("Q4 final"). Tickets: `reviews/2026-11-12-q4-tickets.md` (13 open: 4 medium, 9 low; to be filed with label `q4`).

### What Q5 must know
**User decisions already taken for Q5** (2026-11-13, recorded by the orchestrator; confirm at the start of Q5):
- Fixtures: the committed spoken Amazing Grace WAVs stand in for the two hymn recordings; real recordings come later, by ticket.
- CI eval gate: it replays committed session logs (JSON only; no audio and no model in CI).
- Budget: a new "voxxa-eval" row capped at 600, recorded as a deviation.

**Event schema v1 (ADR-013, ADR-014).** One JSON object per line, keys in order `v`, `t`, `type`. Seven types are used (ADR-001 allows 8):
- On stdout and in the session log:
  - `status`: `start` (with `info`: `SmartConfig`, setlist fingerprint, model, `bias`, `shadow`), `listening`, `singing`, `inter_verse_silence`, `blank_hold`, `stop`.
  - `hypothesis` (`text`).
  - `action`: `goto` with `song`, `slide`, `global` and `title`; `blank`; `unblank`.
  - `error` (`message`).
- In the session log only:
  - `tick`, every 200 ms.
  - `speech`, coalesced to the last one before any later line.
  - `window` (`text`), new in Q4.

  Session-log `t` never decreases.

**Hypothesis cadence.** There are two kinds of recogniser line:
- **`window`:** the whole text of each decoded 5 s window. At file speed there is one at every whole second while the VAD heard speech in the second before (exit demo: 4000-8000, 10000-15000, 17000-26000, 31000-38000 ms). Not a conductor input. Overlapping windows repeat words.
- **`hypothesis`:** the stitched text, only the words a window adds, held until there are 4. It comes every 1-3 s of speech (exit demo: 6000 8000 10000 11000 14000 18000 20000 23000 25000 32000 35000 36000). This is the conductor's input and the only recogniser line replay applies.
- With `--realtime`, `--device` and in the GUI, hops are skipped while inference is slower than real time. On the 4-core reference PC that means one decode every 2-5 s (Q4-02). Run the eval at file speed.

**Replay.**
- Command: `voxxad --replay <session.jsonl> --setlist <the same setlist>`. It needs no model and opens no audio.
- It applies the logged `tick`, `speech` and `hypothesis` lines at `base + t` through `Conductor::apply` and skips `window` lines.
- It prints the `status`, `hypothesis` and `action` lines without the start and stop lines. They are byte-identical to the live run's.
- It refuses another setlist (FNV-1a fingerprint in the start line), a bad line, a `v` other than 1, and `--bias`.
- What this means for the CI gate: replay re-runs today's conductor on recorded recogniser output. A conductor change (Q6) is measured by replay alone. A recogniser change (window, hop, stitcher, model) needs the logs re-recorded locally with Whisper Tiny, with the reason in the commit (as `session-pd-hymns.jsonl` was in Q4).

**Aligning actions and timing to truth.**
- **Timeline:** at file speed `t` is ms of audio from the start of the WAV, leading silence included (a sample clock, deterministic, identical on every machine). Truth `t_ms` must use the same timeline: ms from the start of the same WAV.
- **Indices:** `song` is the 0-based index in the setlist JSON and `slide` the 0-based slide within that song; `global` counts slides across the setlist. Use the same indices in `truth.jsonl`.
- **Matching:** a truth row `{t_ms, song, slide}` is matched by the first `goto` to the same `song` and `slide` at or after it. Latency is action `t` minus `t_ms`.
- **Where the latency comes from:**
  - a decision is timed at the end of its hop: whole seconds for transcripts, the 200 ms tick grid for tick blanks;
  - the stitcher holds words until there are 4 (1-3 s);
  - the conductor's 40-word buffer still holds the previous verse (Q4-04: an advance came 9.5 s into a 12 s verse in a scripted test).
  - Q6's budget is p95 latency at most 2.5 s, so expect the first numbers to miss it.
- **Double advances and repeats:** count a `goto` to the slide already shown as a defect. Q3-08's path is fixed (613b894) and verifier round 3 saw none, so any repeat now is a regression. Dispatch drops repeats, so the screen would not show them.
- **Blank and unblank:**
  - A blank is `{"action":"blank"}`. It comes from the silence rule or from the unmatched-speech rule:
    - Silence rule: `inter_verse_silence` after 3 s without speech (`silence_to_blank_secs`), then a blank after 5 s.
    - Unmatched-speech rule: a blank after 5 s of unmatched transcripts (`unrecognized_speech_to_blank_secs`), checked on a transcript or on a tick.
    - In the exit demo the silence rule fired 0.3 s into the prayer, before the VAD reported its speech.
  - Expect a blank 5-7 s into a prayer that follows a song without a pause (Q4-03). With a pause, the blank comes sooner, through the silence rule.
  - After a blank the conductor usually shows the slide again with a `goto`. It sends `unblank` only when it has no slide to go to. Time "unblank" as the first `goto` or `unblank` after a blank.

**Recording and running.**
- `voxxad --stdin-audio` takes only 16 kHz mono WAV (Q3-10, #54). Convert recordings first.
- A debug `voxxad` decodes a hop in about 1.8 s: a 44 s service takes about 65 s of wall time.
- A release build on Windows compiles whisper.cpp unoptimised (Q4-01), so the same run takes about 6.5 s per hop, over 10 minutes for 44 s. Use a debug build, or set `CMAKE_C_FLAGS_RELEASE` and `CMAKE_CXX_FLAGS_RELEASE` as in the ticket.
- There is no operator key log (`--log-keys` in the 2026-10-24 plan was never built; operator inputs are not logged, Q3-09, #53). For now, `truth.jsonl` is written by hand.
- Committed test audio: `src-tauri/tests/fixtures/voxxad/amazing-grace-v1.wav` (verse 1, 12 s), `amazing-grace-v2.wav` (verse 2, 10.3 s) and `prayer.wav` (8.1 s). These are public-domain text read by the Windows SAPI voice, plus a synthetic prayer written for Q2. Also there: `pd-hymns-setlist.json` and the re-recorded session `session-pd-hymns.jsonl` (3 actions: goto 0/0 at 6000, goto 0/1 at 25000, blank at 30600).
- The verifier WAVs (service4, service2, gap2, gap4) are in the session scratchpad only, not committed. The eval needs its own committed logs.
- Pointing `APPDATA` at an empty folder does not hide the model on Windows (Q4-09). Do not use it to prove a no-model CI path.

**Open Q4 tickets an eval should measure:** Q4-03 (blank timing), Q4-04 (late advances), Q4-06 (a repeat right after a short pause can be dropped), Q4-11 (`--bias`, ADR-004 E6), Q4-13 (word times are rough and not logged; the eval can time actions and lines, not words).

**Budget numbers (after Q4, a4bae0e).**
| Row | Used / cap | Left |
|---|---|---|
| everything else in src-tauri/src (holds `listen.rs`, `commands/`) | 2,286 / 2,500 | 214 |
| recognizer (`transcription/` + recognizer code; new in Q4, user decision) | 432 / 700 | 268 |
| conductor (`aligner/`) | 1,188 / 1,200 | **12** |
| voxxad binary (`src/bin/voxxad*`) | 548 / 600 | 52 |
| presenters | 1,149 / 2,000 | 851 |
| importers + planning_center | 1,362 / 1,600 | 238 |
| GUI | 2,951 / 3,000 | 49 |

`voxxa-eval` goes in its own row (user decision above). Do not grow the conductor: 12 lines are left for Q6. Tests in `src-tauri/tests/` are not counted.

**Other things to keep.**
- One dispatch path: `dispatch::drive`. `--shadow` and `--presenter none` never build a presenter.
- Safety rules for unattended runs are unchanged: no microphone, no keystroke dispatch, a local mock presenter only, and stop every process by PID.
- Text written through a Bash heredoc on this PC can turn `\0` into a NUL byte (it happened again in Q4). Write files with the Edit or Write tools and check with `git grep -lIP '[\x00\x0b]'` before committing (Q3-12, #56).
- Session logs now hold a `window` line per decoded hop, so several times more transcript text sits in the roaming `%APPDATA%` (Q3-05, #49). The fixtures `README.md` and the privacy text should say so.

### Still open from earlier steps
- **Founder checklist** before any pilot leaves shadow mode: QA report §10 (M1–M14) and security review §10, plus Q3-07 (#51): start and stop listening in the GUI (now with the hop: a `transcription` event every 1-3 s of speech), unplug, and one `voxxad --device default --shadow` run on the booth PC. First clear `%LOCALAPPDATA%\com.voxxa.app`; keep `%APPDATA%\voxxa\models\ggml-tiny.bin`.
- **User decision:** the cut-list ADR-012 (evidence in the QA report §9).
- **Q1 A15 follow-ups:** `fixtures/setlists/pd-hymns.json` and `fixtures/pco/plan-sample.json` do not fit the May parsers (this matters to Q5's fixtures); re-run the Windows verification (a003992) on the merged head (`handoff-local-windows.md`); finish reconciling `docs/signing-policy.md` with `SIGNING.md`.

**Process rule (user, 2026-11-11):** every prompt (Q2…Q27) also runs independently through ponytail. Run it as two lanes plus cross-review for code steps, or `ponytail` + `ponytail-audit` for verification and close steps. Record the result in `reviews/YYYY-MM-DD-ponytail-<Q>.md`.

---

## Handoff History
- 2026-11-12 Q3 → Q4: Q3 added the headless `voxxad` CLI and moved the listening loop into `voxxa_lib::listen::run` (ADR-013; schema v1, byte-identical replay); code head 912b978, merged into `main` as ec6d85e (PR #45); 12 tickets filed as issues #46–#57; Q3-02 (#46) handed to Q4; the full section is in this file's git history (a4bae0e).
- 2026-11-12 Q2 → Q3 (handed to the user for ADR-012, then Q3): Q2 verified the merged base in simulate mode and fixed every crash, data-loss, freeze and wrong-slide defect found, with tests (code head 713f40f after the ponytail-audit re-run, merged into `main` as 7bd84a1); 43 tickets filed as issues #2–#44; the full section, with the unattended-run needs and the cut-list evidence, is in this file's git history (f2bcea1).
- 2026-11-11 Q2: QA + security verification in simulate mode; 18 fix-now items + verifier finding Q2-44 fixed with tests; two fix lanes (10x, ponytail) → 10x base with ponytail grafts; 34 tickets; ADR-011 stage window compliant by default; cut-list evidence for ADR-012.
- 2026-11-11 Q1: May branch merged per ADR-010 + A1–A15; two lanes (10x, ponytail) → 10x base with ponytail grafts; all local gates green; build.yml YAML fixed.
- 2026-11-11 Q0 gate: the user accepted ADR-010 as amended; ADR-011; cuts deferred until after Q2; Q2 widened to cover wrong-slide defects.
- 2026-11-11 Q0: claims verified, ADR-010 path table + amendments A1–A15, ponytail lane; stopped at the user gate.
- 2026-11-11 re-planning: May branch discovered; ADR-010 Proposed; prompt pack regenerated for /10x-Team:10x-team.
- 2026-10-24 init → Discovery: `.10x/` created.
- 2026-10-24 Discovery → Strategy: codebase scanned; critical finding: model never loaded, VAD never initialized.
- 2026-10-24 Strategy → Design → Planning: specs + ADR-001…009 + prompt plan written in one pass after five-angle research.
