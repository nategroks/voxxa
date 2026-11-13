# ADR-014: Recognizer trait with a 1 s streaming hop over a 5 s window, in the shared loop

**Status:** Accepted 2026-11-12 (the user's pre-approved Q4 decisions: one 700-line "recognizer" budget row, Q4 owns Q3-02, the PD session fixture may be re-recorded, no GUI change beyond what the hop implies, no sockets) · **Date:** 2026-11-12 · **Feature:** church-autoadvance · **Reversibility:** Easy for the window, hop and gate numbers (constants); moderate for the trait (one more engine will implement it) · **Owner:** nategroks · **Author:** 10x-Team (Principal Architect + Staff Engineer)

Amends ADR-013 Decision 1 (the recogniser closure) and replaces its 5 s batch. Implements the Increment 1 part of ADR-004.

## Context
ADR-013 put the recogniser into the shared loop (`voxxa_lib::listen::run`) as a closure, `FnMut(&[f32]) -> Result<String>`, and named its own review trigger: "a second recogniser lands (turn the closure into ADR-004's trait)". Q4 turns it into the trait and replaces the batch, so the trigger applies.

What the loop does today (`listen.rs`, Q3):
- it appends each VAD utterance to a buffer and calls the recogniser once the buffer holds 5 s of speech (`accumulated.len() >= 16000 * 5`), then clears it;
- the conductor gets one transcript per ~5 s of speech, timed at the start of the chunk that completed the batch;
- the recogniser runs synchronously inside the loop (Q3-06, #50): ticks, timers and VAD stall while Whisper runs.

Forces:
- **ADR-004**: `trait Recognizer { fn feed(&mut self, pcm16k: &[f32]) -> Vec<Hypothesis>; fn bias(&mut self, text: &str); }`, `Hypothesis { words: Vec<(String, f32 /*t_start*/, f32 /*conf*/)> }`; Increment 1 ships whisper.cpp with a 1 s hop / 5 s window, `single_segment`, VAD gating and `initial_prompt` biasing behind a flag (E6: it may hallucinate; measure).
- **ADR-013 replay**: the conductor reads time only through the `now` it is given; replay applies the logged `tick`, `speech` and `hypothesis` lines at `base + t` and must stay byte-identical on action lines without a model. Every input the hop produces must therefore be a logged `hypothesis` line with its `t`.
- **Time**: whisper-rs token timestamps (`t0`, in 10 ms units) are relative to the audio passed to `full()`, so a rolling window needs an offset to give stream time.
- **Cost**: at a 1 s hop, each second of speech costs one inference on up to 5 s of audio (whisper.cpp encodes a padded 30 s mel whatever the input length). In a slow build a hop can take longer than a second; on a sample clock that only slows the run, but on a wall clock (`--realtime`, `--device`, the GUI) audio queues up.
- **CI** has no Whisper model and no audio device.
- **Budget**: "everything else in src-tauri/src" (which holds `listen.rs` and `commands/`) is at 2,496 / 2,500.

## Decision
1. **The trait** lives in `src-tauri/src/transcription/recognizer.rs`:
   ```rust
   pub trait Recognizer {
       fn feed(&mut self, pcm16k: &[f32]) -> Vec<Hypothesis>;
       fn bias(&mut self, text: &str);
       /// Default: ignored. See point 5.
       fn gate(&mut self, _open: bool) {}
   }
   pub struct Hypothesis {
       pub words: Vec<(String, f32, f32)>, // word, start in stream seconds, confidence 0..1
       pub end: f32,                       // stream seconds where the decoded audio ends
       pub error: Option<String>,          // the hop's inference failed; `words` is empty
   }
   ```
   `feed` takes every sample of the stream exactly once, in order. Stream time is samples fed / 16,000, the same count the loop's clock maps to `Instant`s, so `t_start` and `end` are absolute. Two deviations from ADR-004's literal shape, both additive: `gate` (a provided method; an engine that implements only `feed` and `bias` still compiles and is never gated) and the `end` and `error` fields (the loop needs errors to keep ADR-013's `error` line; `end` lets tests and `voxxa-eval` see which audio a hypothesis covers).
2. **Whisper behind the trait: `Rolling<D>`** keeps the last `window_ms = 5000` of audio (fewer at the start of a stream). Each time the samples fed cross a multiple of `hop_ms = 1000`, and the gate is open, it decodes the whole window once and returns one `Hypothesis`; when one `feed` call crosses several hop boundaries it still decodes once, on the newest window. `D: Decode` turns one window into words timed from the window start; `TranscriptionEngine` implements it with `single_segment`, `no_context`, `token_timestamps`, and `initial_prompt` set to the text given to `bias()` only when `bias_enabled` is true (default **false**). `Decode` is also implemented for the GUI's shared `Arc<Mutex<TranscriptionEngine>>` (the lock is held per hop, not per chunk). Words are built from tokens: special tokens (`id >= eot`) are dropped, a token that starts with a space starts a word, a word's start is its first token's `t0`, its confidence the mean token probability. Absolute start = (samples fed − window length) / 16,000 + `t0` / 100.
3. **The hop loop replaces the batch** in `listen::run`. Per chunk: tick, mic level, VAD as before; then the loop sets the gate and calls `recognizer.feed(chunk)`. For each hypothesis returned it takes `now = clock(samples received)`, the end of the audio it holds, and:
   - an `error` becomes an `Event::Error` (as before; no conductor input);
   - an empty text is dropped;
   - otherwise `Event::Input(Input::Heard(text))`, `Conductor::apply`, the detection rows and the action, in the same order as Q3. The text is the whole window's text: consecutive hypotheses overlap by about 4 s.
   - After a `Goto`, the loop calls `recognizer.bias(slide text + next slide text)`. With `bias_enabled` false this only stores the text.
4. **Absolute timestamps everywhere.** The hypothesis line's `t` is the session clock at the end of the hop's audio (audio time at file speed; wall time after inference with `--realtime`, `--device` and in the GUI). `t_start` and `end` in `Hypothesis` are stream seconds. Event schema v1 is unchanged: the `hypothesis` line still carries only `text`; word times are not logged (adding them later as an optional field is additive and does not need `"v":2`).
5. **Gate: VAD gating and backpressure in one rule.** Before each `feed`, the loop opens the gate only if
   - the VAD said speech within the last 1 s of audio (so silence, and the 4 s of old speech a window still holds after singing stops, are never decoded: no Whisper cost and no hallucinated "Thank you." on silence, and no hypothesis keeps `on_speech` alive after the room went quiet); **and**
   - the loop is not more than one hop behind the audio. `lag` = session clock elapsed − audio received; each time the loop finds the channel empty it is caught up, and that lag becomes the floor (device latency, start-up and dropped samples are absorbed there). `behind` = `lag − floor > 1 s`.
   
   **Backpressure rule: skip, do not catch up.** While behind, audio still enters the window, but hop boundaries pass without inference; the first hop after the loop has drained the queue decodes the newest 5 s. The queue is thus bounded by about one inference time plus one hop, and a stale window is never decoded. **File speed stays deterministic**: on a sample clock `lag` is exactly 0, so the gate depends only on the VAD and every hop with speech is decoded, whatever the machine's speed.
6. **Replay impact.** No new event type and no schema change. Every `Heard` the hop produces is logged as a `hypothesis` line at its `t` before it is applied, so replay reproduces the actions without the model. `speech` coalescing stays exact: `Conductor::on_speech` still only records `now`, and a hypothesis line flushes the pending `speech` line before it, as ticks do. The committed `tests/fixtures/voxxad/session-pd-hymns.jsonl` was recorded with the 5 s batch; the conductor now hears different inputs, so it is re-recorded with Whisper Tiny on the same public-domain WAVs (user decision (3)); its replay test stays.
7. **Overlapping text and the conductor.** The conductor's API and code are unchanged. It now gets about one transcript per second of speech, each covering the last 5 s. Expected effects, checked by tests with a fake recogniser and by the exit demo with Whisper Tiny on the PD WAVs:
   - detection is faster (the first window that holds 4+ words of a line can start the song);
   - an advance is still debounced (1.5 s) and clears the word buffer; the window that triggered it cannot advance again, because the next slide's words are not in it;
   - two unmatched transcripts in a row now span about 2 s of speech instead of about 10 s, so a prayer blanks sooner, and a misheard stretch of song can also blank sooner (bad blank, never a wrong slide);
   - after a blank, re-showing still needs the start bar (85) on the new window's text.
   No conductor tuning in Q4; `voxxa-eval` (Increment 5) tunes it on the pilot corpus.
8. **GUI.** Recognition timing changes in the GUI too: a `transcription` event about every second of speech instead of every 5 s of speech, and slides can follow sooner. That is the only GUI-visible change (user decision (4)). The GUI keeps `Instant::now()` as its clock, `bias_enabled` false, and its sink is rewritten as a pure mapping, `commands::gui_event(Event) -> GuiEvent` (Q3-02, #46): `transcription`, `machine-state`, `mic-level`, `song-detection` with their JSON payloads (serialised once, emitted with `emit_str`, byte-identical to `emit`), loop actions dispatched with `operator = false`, and `presenter-error` on a failed dispatch. It is tested in `src-tauri/tests/`.
9. **Budget (recorded deviation from ADR-010 Decision 3).** `scripts/loc_budget.py` gets one new row, "recognizer (transcription/ + recognizer code)", capped at **700** tokei code lines and excluded from "everything else". It holds `src-tauri/src/transcription/` (251 code lines before Q4) and the trait, `Hypothesis`, `Rolling` and `Decode`. "Everything else" stays at 2,500, holding `listen.rs` and `commands/`; no budget is raised. Tests in `src-tauri/tests/` are not counted.
10. **Out of scope:** sherpa-onnx or any second engine; moving inference off the loop's thread (Q3-06 stays open, now bounded by the gate); conductor changes; word times in the log; a GUI switch for biasing. `voxxad` gets `--bias` (off by default, recorded in the `start` line) so E6 can be measured.

## Alternatives Considered
| Alternative | Pros | Cons | Why Not |
|---|---|---|---|
| Keep the 5 s batch, only wrap it in the trait | No behaviour change, no re-recorded fixture | Latency stays at 5 s of speech; not ADR-004's Increment 1 | The prompt asks for the hop |
| Hop over VAD speech only (concatenated utterances), as the batch did | Never decodes silence | Timestamps become "speech time", not stream time; gaps vanish | Hypotheses need absolute stream time |
| Feed the conductor only the words new since the last hypothesis (dedupe by token time) | No overlapping text in the conductor's buffer | About 2 words per hop: under the conductor's 4-word floor, so nothing would match; `matches_song` would judge fragments | Needs conductor changes, which Q4 keeps out |
| LocalAgreement-style commit (emit only words two windows agree on) | Stable text, no repeats | Adds a hop or more of latency and ~80 lines; the conductor would still need its floor changed | Not needed to keep the PD WAVs right; revisit with `voxxa-eval` |
| Catch up on every missed hop when behind | Every hop decoded | Decoding stale windows makes the loop fall further behind; queue grows without bound when a hop takes longer than 1 s | Skip is bounded |
| Skip by draining the channel into one big `feed` | No gate method | At file speed the channel is always full, so results would depend on timing | File-speed runs must be deterministic |
| Gate by an energy threshold inside `Rolling` | Trait exactly as ADR-004 | Decodes music and noise; duplicates the VAD | The VAD already decides speech |
| Feed zeros for non-speech instead of a gate | Trait exactly as ADR-004 | Clips the first 250 ms of each utterance (the VAD latch), and changes what Whisper hears | Worse recognition |
| Run inference on a worker thread (fixes Q3-06) | Ticks never stall | Changes the loop's ordering guarantees and replay timing; bigger change | Out of scope; the gate bounds the stall |
| Raise "everything else" to fit the recogniser | One row | ADR-010: budgets only go down | User decision: a separate 700 row |
| A new crate for ring buffers or WAV | Less code | `cargo deny` surface | A `Vec` and the existing reader suffice |

## Consequences
### Positive
- One seam for every engine: a sherpa-onnx streaming recogniser implements `feed` and `bias` and needs neither the window nor the gate.
- About 1 s detection and advance latency instead of 5 s of accumulated speech; a prayer blanks after two unmatched seconds of speech.
- Replay, the schema and the conductor are unchanged; file-speed runs stay deterministic.
- The GUI sink is pure and tested (closes Q3-02).
### Negative
- About 5x more Whisper work per second of speech than the batch. In debug builds the hop cannot keep up in real time; the gate then skips hops (measured and reported in the SDE record).
- The conductor's word buffer now holds overlapping text. Its scores are computed over repeated words; the PD WAVs are fine (exit demo), real singing is unmeasured until `voxxa-eval`.
- `whisper-rs` 0.13 leaks the `initial_prompt` C string on every decode (`set_initial_prompt` uses `into_raw`), only when `bias_enabled`: about 200 bytes a hop.
### Risks
- **Mid-song blank on a misheard stretch** (two unmatched windows in 2 s). Mitigation: bad blank, never wrong slide; measure with `voxxa-eval`; conductor thresholds are config.
- **Bias hallucination (E6):** with `initial_prompt` Whisper can echo the prompt on silence or speech, which would read as lyrics. Mitigation: off by default, silence is never decoded (VAD gate), measured with `voxxad --bias` before any default change.
- **Device clock drift** shifts `lag` slowly; the floor is re-taken whenever the queue is empty, so drift never accumulates.

## Dependencies
- Depends on: ADR-004 (the trait and Increment 1 parameters), ADR-013 (shared loop, clock, schema v1, replay), ADR-010 Decision 3 (budgets, with this recorded deviation).
- Constrains: the sherpa-onnx and Moonshine engines (Increment 5) implement `Recognizer`; `voxxa-eval` reads hypothesis lines about once a second; Q3-06 (inference off the loop thread) can later move `Rolling`'s decode to a worker without changing the trait.

## Review trigger
Re-open when a second engine implements `Recognizer`, when `voxxa-eval` shows overlapping windows cost precision, if biasing is to be turned on by default, or if the recognizer row needs more than 700 lines.
