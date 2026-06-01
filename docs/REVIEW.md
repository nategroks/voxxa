# Self-review punchlist

A code review I did near the end of Phase 3, after ~36 feature commits.
Reading the modules in order surfaced a few real bugs (one nasty, several
papercuts) and a fair amount of dead code. This is the running record.

## Findings

### Critical correctness

1. **Manual prev/next desynced the Conductor.** Operator hits Next →
   `presenter.next_slide()` fires, `last_dispatched_global` increments, but
   the conductor's `current_slide_in_song` stayed put. Then the next
   lyric-triggered Goto computed `delta = global_target - last_dispatched`
   off the now-skewed value and could send a backwards keystroke that
   undid the operator's move. Same bug in the HTTP API's `/api/v1/next`
   and `/api/v1/prev` routes.

   Fix in `c202fbc` and the HTTP follow-up:
   `Conductor::manual_step(delta, now)` bounds-clamped to the active
   song. Both Tauri commands and HTTP routes now go through it before
   hitting the dispatcher. One code path, one source of truth.

2. **Audio loop never reset `is_running` on disconnect.** Mic unplugged
   mid-service → cpal channel disconnected → loop broke out → UI still
   showed "Listening" until app restart.

   Fix in `d05f439`: loop ALWAYS resets `is_running` on exit and emits
   a `listening-stopped` event with `disconnected: bool`. Frontend
   re-arms the start button and surfaces an error toast on disconnect.

3. **Stage Display "Next" panel never populated.** `refreshNextSlide`
   was a stub.

   Fix in `c202fbc`: `Action::Goto` now carries
   `next_slide_text: Option<String>`, threaded through `SlideAdvanced`.
   Stage Display reads it directly. No backend round-trip.

### Stale state / dead code

4. **`StatusInfo.model_loaded` hardcoded `true`** — broken since the
   real model load lifecycle landed. Frontend couldn't show "no model
   loaded" warnings. Fixed `8e67112`.

5. **`commands::Settings` + `get_settings` + `save_settings`** —
   vestigial. Replaced by per-feature commands (`set_smart_config`,
   `set_language`, `select_audio_device`) but never deleted. Fields
   like `similarity_threshold`, `margin`, `max_buffer_words`,
   `block_duration` flowed nowhere. Dropped in `8e67112`.

6. **`src-tauri/src/mcp/`** — leftover from the original voice-dictation
   app before the worship pivot. No callers. Module + `rmcp` dep + the
   `mcp` feature flag all dropped in `8e67112`.

7. **`StatusInfo.current_slide` was `0` when no song was detected** —
   misleading. Now `Option<usize>`, `None` until the conductor commits.
   `8e67112`.

### Audio robustness

8. **Audio config was hardcoded to mono / 16 kHz.** Failed on pro audio
   interfaces locked to 48 kHz and stereo-only USB devices (e.g.
   webcams) with a cryptic cpal error.

   Fix in `d05f439`: probes the device's supported configs, picks the
   first that brackets 16 kHz, prefers mono if available, downmixes
   multi-channel input to mono in the callback. Unsupported devices
   now error with a clear "device X does not support 16 kHz capture
   (configs: ...)" message.

## Process notes

This review was done by reading the modules end-to-end after I'd
otherwise called Phase 3 "done." Two of the four critical bugs (#1
manual desync, #3 stage Next stub) were silent — they'd survive a
ship and only surface in real-world use. Worth pausing for an
audit pass before claiming completion.

The conductor unit tests caught a fifth bug not in this list
(`3e13077`) — purely softmax-relative detection could commit to a
song that had no good absolute match. That fix added the
`min_song_match` config gate.
