# Voxxa — Delivery Plan and Claude Code Prompt Budget

Status: Proposed for user review · Date: 2026-10-24 · Owner: nategroks · Phase: 10x-Team Phase 3 (Engineering Manager + Senior Engineer)
Inputs: `2026-10-24-software-strategy.md`, ADR-001…009, `2026-10-24-business-strategy.md`, `2026-10-24-user-research-plan.md`.

## 1. How to read the estimates

Every prompt below is one Claude Code session in this repo, run with **ponytail on** (`/plugin install ponytail@ponytail`, level `full`) and the 10x-Team state in `.10x/` present. Ponytail's own benchmark reports −45% output tokens and −26% cost against the same agent without it (self-reported, 39 tasks, Opus 5.5); the estimates below already assume it.

Calibration anchors (all from this session's research, not computed here): ponytail's published board shows an average of **$0.72 per turn** and about **168k output tokens per 56.7M total tokens** over a 33-turn task on Opus 5.5; Opus 5.5 list price is $4 per million input tokens, $20 per million output, about $0.20 per million cache reads. In Claude Code, cache reads dominate total tokens, so cost tracks turns more than output.

| Size | Turns | Output tokens | Total tokens processed | API list cost | Wall time |
|---|---|---|---|---|---|
| S | 5–10 | 25–50k | 5–15M | $3–8 | 0.5–1.5 h |
| M | 15–30 | 75–150k | 20–50M | $10–25 | 1.5–4 h |
| L | 30–60 | 150–300k | 50–100M | $25–50 | 4–8 h (split across two sessions if context fills) |

On a Claude subscription (Pro/Max) the same work is bounded by the plan's session limits rather than dollars; check the current limits before committing to a daily cadence. Expect **1.5× rework** on L prompts that touch the aligner or adapters (fixtures reveal surprises). All figures are bands, not forecasts.

## 2. Totals

| | Count | Output tokens | Total tokens | API list cost |
|---|---|---|---|---|
| S prompts | 9 | 0.2–0.45M | 45–135M | $27–72 |
| M prompts | 19 | 1.4–2.9M | 380–950M | $190–475 |
| L prompts | 5 | 0.75–1.5M | 250–500M | $125–250 |
| **Plan (33 prompts)** | | **≈ 2.4–4.8M** | **≈ 0.7–1.6B** | **≈ $340–800; ≈ $500–1,200 with rework** |

**Tokens per day**

| Cadence | Prompts/day | Output tokens/day | Total tokens/day | API list cost/day | Calendar for all 33 prompts (prompting time only) |
|---|---|---|---|---|---|
| Part-time (≈ 3 h/day, 5 days/week) | 1 | 75–150k | 20–50M | $10–25 | ≈ 7 weeks |
| Full-time | 2–3 | 150–400k | 50–150M | $25–75 | ≈ 3 weeks |

**Why the calendar is 22 weeks anyway:** tokens are not the bottleneck. The pilot churches (weeks 3–8), SignPath/Trusted Signing applications, Partner Center review, Apple notarization, founder-recorded fixtures and hardware testing gate the increments. Full-time compresses the plan to about 12 weeks; the pilot still gates the recognizer bake-off at week 8.

## 3. Roadmap

```
Now   (weeks 0–6):  Inc 0 reposition → Inc 1 the loop works and is measured → Inc 2 signed Windows build a volunteer can run
Next  (weeks 5–14): Inc 3 native adapters (pilots start) → Inc 4 scripture → Inc 5 recognizer bake-off, Store, support ops
Later (weeks 14–22): Inc 6 macOS → Inc 7 Linux AppImage → 1.0
```

Each increment leaves a working, operable system. The first increment retires the biggest risk (recognition on singing, measured), not the prettiest screen.

## 4. Increments

### Increment 0 — Reposition and housekeeping (week 0)
Goal: nobody who finds the repo thinks it is a dictation app; dead weight is gone; CI protects every PR.
Risk retired: positioning confusion; unreviewed changes.
In: README, site copy, Cargo description, release body; delete `mcp` module and `tauri-plugin-store`; LICENSE (GPL-3.0-or-later) + DCO; PR CI. Out: any feature work.
Exit: `cargo fmt --check && cargo clippy -D warnings && cargo test && cargo deny check` green on all three OSes on PR; README describes the church product; LICENSE present.
Dependencies: none. Duration: 1–2 days. Rollback: revert.

### Increment 1 — The loop works, and it is measured (weeks 1–3)
Goal: `voxxad --stdin-audio service.wav --setlist today.json` prints correct `goto` decisions on fixture recordings, and CI scores it.
Risk retired: R1 (recognition on singing) becomes a number instead of a fear.
In: wire model loading and VAD; recognizer trait + whisper_cpp streaming; SongGraph; aligner redesign (ADR-009); supervisor; `voxxad` CLI and JSON-lines v1; `voxxa-eval` + two synthetic fixtures. Out: GUI changes, adapters beyond keystroke.
Exit: eval on fixtures reports precision ≥ 0.9, zero double advances; replay reproduces a logged session byte-for-byte; `cargo test` covers graph transitions and decision rules.
Dependencies: founder records two PD-hymn fixtures (20 min each) with slide-change timestamps. Duration: 2–3 weeks part-time. Rollback: feature flag `aligner = "legacy"` for one release.

### Increment 2 — Headless split and a signed Windows build (weeks 3–6)
Goal: a volunteer installs from GitHub or the Store, presses one button, sees green.
Risk retired: R5 (install friction), QA rank 2 (operability).
In: GUI as thin client (ADR-001), `voxxa.toml` (ADR-003), first-run flow, NSIS per-user + MSI + MSIX, signing in CI, winget manifest, Call-for-help bundle, three runbooks, quick-start card. Out: macOS/Linux polish, adapters.
Exit: signed installer; 5-of-5 volunteers reach green in the usability test (research O7) without a call; `voxxa.toml` round-trips; CSP strict.
Dependencies: SignPath or Trusted Signing approval; Partner Center account. Duration: 3 weeks. Rollback: previous release via `latest.json` repoint.

### Increment 3 — Native adapters and pilots in shadow mode (weeks 5–8)
Goal: Voxxa reads the real setlist and real position from ProPresenter, FreeShow and OpenLP; three pilot churches run shadow mode.
Risk retired: R2-adjacent (keystroke blindness), setlist licensing (no SongSelect).
In: adapter trait + contract-test harness; ProPresenter, FreeShow, OpenLP adapters; Planning Center setlist source. Out: Proclaim, Holyrics, MIDI.
Exit: each adapter ≤ 300 LOC with a recorded fixture test; re-sync from `current()` demonstrated; pilot churches produce the first real corpus files.
Dependencies: access to one real install of each app (pilot churches or trial licences). Duration: 3–4 weeks. Rollback: adapters are config-selected; keystroke remains.

### Increment 4 — Scripture mode (weeks 8–11)
Goal: "Songs + Scripture" follows a spoken reference and the verses read after it.
Risk retired: competitive parity with FreeShow/SmartVerses on scripture (E1), decided by research O6; if O6 says "later", this increment moves after Inc 5.
In: PD Bible bundle (ADR-007), spoken-reference grammar, detector in the accumulator, `show_ref` per adapter. Out: copyrighted translations, paraphrase handling beyond reference-only.
Exit: 200-form grammar test suite passes; eval on two spoken-passage fixtures ≥ 0.9 reference precision; FreeShow and OpenLP display the verse; ProPresenter shows the operator hint.
Dependencies: Inc 3 adapters. Duration: 3 weeks.

### Increment 5 — Recognizer bake-off, Store listing, support operations (weeks 11–14)
Goal: the default engine is chosen on real church audio; the Store listing is live; a church can buy Sunday Support.
Risk retired: R1 (final), R2 (revenue lines live), R5.
In: sherpa-onnx recognizer (feature-flagged), bake-off script and report, per-hardware recommendations, Store submission, stable/community channels, support contract + SLA docs, pricing page, `usage.csv`. Out: Moonshine unless sherpa-onnx fails.
Exit: bake-off report in `.10x/reviews/`; default engine set by data; Store listing approved; first paid Setup & Calibration delivered.
Dependencies: pilot corpus (week 8), Partner Center review. Duration: 3 weeks.

### Increment 6 — macOS (weeks 14–18)
Goal: notarized DMG and Homebrew cask; Accessibility and microphone permission flows guided.
Exit: Gatekeeper opens the app with no override; keystroke fallback works after the guided permission; eval passes on an Apple Silicon machine. Dependencies: Apple Developer Program ($99). Duration: 3–4 weeks.

### Increment 7 — Linux AppImage and 1.0 (weeks 18–22)
Goal: single-file AppImage built on Ubuntu 22.04 with embedded update info; Wayland detected (network adapters only); 1.0 released with upstream documentation PRs to FreeShow and OpenLP.
Exit: AppImage runs on Ubuntu 22.04/24.04 and Fedora current; zsync delta update works; docs list Voxxa in both projects' integration pages. Duration: 3–4 weeks.

## 5. Staffing, skills, hidden work

Assumed: one founder with Rust and Tauri familiarity, comfortable running Claude Code daily, able to visit or video-call three churches. Skills gaps to budget time for: Windows packaging (MSIX/Partner Center), macOS notarization, audio-engineering basics for the calibration service. Hidden work the prompts do not cover: recording fixtures, the user-research interviews, legal entity, SignPath/Trusted Signing paperwork, Store listing assets (screenshots, privacy statement), and Sunday on-call itself.

## 6. Decision freeze vs open

Frozen by this plan: ADR-001…003, 005, 007, 008 (structure, licence, text policy, distribution). Open until data: ADR-004 engine default (Inc 5), ADR-009 priors and thresholds (calibration), scripture at launch (research O6), exact prices (research O2).

## 7. The prompts

Conventions for every prompt: paste as the first message of a fresh Claude Code session in the repo root; ponytail `full` active; the agent must read `.10x/handoff.md` and the named ADRs first, write `.10x/decisions/sde/church-autoadvance.md` and update `.10x/status.md` before finishing, keep `cargo test` green, and end with the ponytail "skipped / not checked / risk" lines. Each prompt lists what **not** to build so ponytail has something to refuse.

---

**P0.1 — Fix the positioning** (S)
> Read `.10x/handoff.md`, `.10x/specs/2026-10-24-business-strategy.md` §1 and §4. Voxxa is a free, local, voice-following slide advancer for churches; the README, `docs/index.html`, `src-tauri/Cargo.toml` description and the release body in `.github/workflows/build.yml` still describe voice dictation. Rewrite only those four places so each states, in plain words a church volunteer understands: what Voxxa does, what it does not do (never shows lyrics, never uploads audio), which presentation apps it works with (ProPresenter 7.9+, FreeShow, OpenLP natively; others by keystrokes), and that the software is free with optional paid Sunday Support. Keep the existing page design. Do not add features, screenshots, pricing numbers, or a blog. Exit: `grep -ri dictation` returns nothing outside git history.

**P0.2 — Delete dead weight, add the licence** (S)
> Read ADR-003 and ADR-005. Remove the `mcp` module, its `rmcp` dependency and feature flag, and `tauri-plugin-store` (code, Cargo, capabilities, package.json). Add `LICENSE` (GPL-3.0-or-later text), a `DCO` file and a one-paragraph CONTRIBUTING.md asking for `Signed-off-by`. Add `deny.toml` for `cargo deny` allowing MIT/Apache-2.0/Unlicense/BSD/MPL-2.0/ISC/Zlib and denying AGPL and non-commercial licences. Grep every caller before deleting. Do not refactor anything else, do not touch the aligner. Exit: `cargo build` and `cargo deny check licenses` pass; `git grep -i mcp` finds nothing.

**P0.3 — CI on every PR** (S)
> Read `.10x/specs/2026-10-24-software-strategy.md` § Maintenance model. Add `.github/workflows/ci.yml` running on pull_request and push to main: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo deny check`, and a `tokei`-based line budget step that fails if `src-tauri/src` (excluding `adapters/`) exceeds 3,000 lines of Rust or `src/` exceeds 800 lines of JS/HTML/CSS. Matrix: ubuntu-22.04, windows-latest, macos-latest, with the same Linux apt dependencies `build.yml` already installs. Fix whatever clippy finds in the existing code with the smallest change. Do not change `build.yml`. Exit: CI green on a no-op PR.

**P1.1 — Make the existing loop actually run** (S)
> Read `.10x/decisions/staff-engineer/_index.md` (known defects) and `src-tauri/src/commands/mod.rs`. Today `start_listening` never calls `TranscriptionEngine::load_model` and `VadEngine::initialize` is never called, so transcription fails with "No model loaded" and VAD passes all audio. Fix: load the selected model on `start_listening` if not loaded (error message in plain English if no model is downloaded), initialize VAD, make `get_status.model_loaded` truthful, make `next_slide_manual` move the aligner index instead of pushing a sentinel word, and emit real `curr_score`/`next_score` in `TranscriptionEvent`. Add unit tests for `LyricsAligner::update` covering advance, no-advance below threshold, and margin. Do not redesign the aligner yet (ADR-009 is the next prompt). Exit: `cargo test` green; running the app with a downloaded Base model prints transcripts in the log.

**P1.2 — Recognizer trait and streaming whisper.cpp** (M)
> Read ADR-004 and `.10x/specs/2026-10-24-software-strategy.md` § Recognition and alignment design. Introduce `trait Recognizer { fn feed(&mut self, pcm16k: &[f32]) -> Vec<Hypothesis>; fn bias(&mut self, text: &str); }` and `Hypothesis { words: Vec<Word> }` with `Word { text, t_start_ms, conf }`. Implement `WhisperCpp` over the existing `whisper-rs` context: rolling window of `window_ms` (default 5000) advanced every `hop_ms` (default 1000), `single_segment`, `no_context`, token timestamps on, `initial_prompt` from `bias()` capped at 224 tokens and only when `bias_enabled`. Replace the energy VAD with whisper.cpp's built-in Silero VAD via `WhisperVadContext` (download the small VAD model next to the Whisper models, pinned by SHA-256). Replace the 5-second batch in `start_listening` with this hop loop. Keep `TranscriptionEngine` for model download only. Do not add sherpa-onnx yet. Exit: a test feeds a 10 s WAV fixture and asserts non-empty hypotheses with monotonically increasing timestamps; log shows one hypothesis per second.

**P1.3 — SongGraph and setlist schema v1** (M)
> Read ADR-009 and ADR-003. Replace the flat `Setlist { setlist: Vec<Song> }` with `setlist.json` v1: `{ "v": 1, "songs": [ { "title", "ccli": optional, "sections": [ { "kind": "verse|chorus|bridge|tag|other", "label", "slides": [ { "text" } ] } ] } ] }`. Build `SongGraph` with a flat slide index, `transitions(from) -> Vec<(to, prior)>` using priors from config (`next 0.6, section_start 0.25, next_song 0.1, other 0.05`), and loaders for: v1 JSON, the old example JSON (migrated to one section per song), and plain text (songs separated by `---`, slides by blank lines, section labels in `[brackets]`). Property-test: priors sum to 1 per node; every slide reachable; migration round-trips. Do not add OpenLyrics/OpenSong XML yet. Exit: `cargo test` green; `setlist.example.json` migrated and documented in README.

**P1.4 — Aligner redesign: position tracking with an evidence accumulator** (L)
> Read ADR-009 in full and `software-strategy.md` § Recognition and alignment design. Rewrite `aligner/mod.rs` as `PositionTracker`: normalise words (lowercase, strip punctuation, spell small numbers); for each candidate slide reachable from the current position score the last 12 recognized words against the candidate's leading 12 words and the previous slide's trailing 6 words with the existing LCS ratio applied to word tokens; `evidence[c] = decay * evidence[c] + prior(c) * score`; decide at most once per hop: move when best ≠ current, evidence ≥ threshold (0.70), evidence ≥ current + margin (0.10), and dwell ≥ 2,500 ms; emit `Decision::Goto { song, slide, conf }`. Add `resync(Position)` for adapter feedback and `manual(Delta)` for operator keys that resets evidence. Unit tests with scripted hypothesis sequences: linear song, repeated chorus, bridge jump, dropped words, hallucinated filler, operator override mid-song, end of set. Keep the old aligner behind `aligner = "legacy"` for one release. Do not touch the GUI or adapters. Exit: all scripted tests pass; `legacy` still compiles; LOC of the module ≤ 450.

**P1.5 — `voxxad` CLI, JSON-lines protocol v1, supervisor** (M)
> Read ADR-001 and ADR-003. Add a second binary `voxxad` in `src-tauri` (or a workspace crate `voxxad/` if cleaner, no new frameworks) that runs capture → VAD → recognizer → tracker → supervisor headlessly. Flags: `--config <path>`, `--list-devices`, `--stdin-audio <wav>` (16 kHz mono WAV, runs at file speed unless `--realtime`), `--replay <session.jsonl>`, `--check-config`. Stdout: one JSON object per line, all with `"v":1` and `"t"` ms: `status {state: listening|following|lost|off, song, slide}`, `hypothesis {words}`, `decision {song, slide, conf}`, `error {code, message}`. Supervisor: no hypothesis for 8 s → `status lost`; a decision is only emitted to the adapter when state is `following`; log every event to `session-YYYYMMDD-HHMM.jsonl` in the OS data dir, rotated after 14 days. Wire the existing keystroke adapter as the only adapter for now. Do not add sockets or HTTP. Exit: `voxxad --stdin-audio fixture.wav --setlist s.json` produces decisions; `--replay` of the resulting session reproduces identical decision lines.

**P1.6 — `voxxa-eval` and the first fixtures** (M)
> Read `software-strategy.md` § Measurement is part of the design. Add `voxxa-eval` (a bin or `cargo xtask`) that takes a directory of fixtures, each `audio.wav` + `setlist.json` + `truth.jsonl` (`{t_ms, song, slide}` operator slide changes), runs `voxxad --stdin-audio`, and reports per fixture and overall: advance precision, recall, median and p95 latency from the truth timestamp to the decision, false advances per set, double advances. Add `fixtures/README.md` describing how the founder records a fixture (desk feed, 16 kHz mono, timestamps from the operator's key log via `voxxad --log-keys`). Add a CI job that runs eval on `fixtures/` and fails if precision drops below the value stored in `fixtures/baseline.json`. Do not generate audio; the founder supplies two PD-hymn recordings. Exit: eval prints a table; CI gate works on a deliberately broken threshold.

**P2.1 — GUI as a thin client with the one-screen UI** (L)
> Read ADR-001, `software-strategy.md` § Simple mode. Rewrite `src/` (vanilla JS, no dependencies, ≤ 800 lines total) as a thin client: spawn `voxxad` via `tauri-plugin-shell` with the config path, read its stdout JSON lines, render exactly: a three-position mode switch (Off / Songs / Songs + Scripture), a status light (grey/amber/green/red) with one line of text, the current song and slide, and a "Call for help" button (stub that calls a `bundle_support` command returning the zip path). Move model download and device selection into a first-run wizard view shown only when `voxxa.toml` is missing or the model is absent. Delete the sliders, the Models tab, and `overlay.html`. Keep the tray icon with Start/Stop/Quit. Do not add a framework, a bundler plugin, or settings beyond the three the spec allows. Exit: the window drives `voxxad` end to end on a fixture WAV via a hidden dev flag; tokei ≤ 800 lines for `src/`.

**P2.2 — `voxxa.toml`** (M)
> Read ADR-003 and the `voxxa.toml` example in `software-strategy.md`. Implement config loading with the `toml` crate: OS config dir path, write a fully commented default file on first run, strict parse errors with line numbers in plain English, `--check-config`, and reload on file change (notify crate only if already a dependency; otherwise poll mtime every 2 s). Replace `get_settings`/`save_settings` with `read_config`/`write_mode_device_adapter` that rewrites only the first block and preserves comments elsewhere (write the three keys via targeted line replacement, not re-serialisation). Do not add a settings UI. Exit: round-trip test; a hand-edited comment survives a GUI write; invalid TOML yields a red status with the line number.

**P2.3 — First-run flow and adapter discovery** (M)
> Read `software-strategy.md` § Simple mode (First run) and ADR-006 (discovery rules). Implement the wizard steps: pick audio device (pre-select a USB interface if present), detect presentation app by probing localhost 5505/5506 (FreeShow), 4316 (OpenLP), 52195 (Proclaim) and the user-entered ProPresenter port, confirming via each app's version endpoint; fall back to keystroke mode with an honest message; download the default model with progress and SHA-256 verification; run a 20-second "sing anything" test showing hypotheses. Write results to `voxxa.toml`. Do not implement the adapters themselves (Inc 3); discovery only needs the version endpoints. Exit: wizard completes on a machine with no apps running (keystroke fallback) and on one with FreeShow running.

**P2.4 — Windows packaging, signing, winget, updater hardening** (M)
> Read ADR-008 and ADR-002. In `build.yml`: NSIS `perUser` install mode and MSI, WebView2 `downloadBootstrapper` kept, add a signing step using the SignPath GitHub Action (inputs from secrets; skip gracefully when secrets are absent) with Azure Trusted Signing as a commented alternative; add a `winget` manifest template under `packaging/winget/` and a release step that opens the PR to `microsoft/winget-pkgs` via `wingetcreate` (behind a secret). In the updater client, refuse any manifest whose version is lower than the installed version. Set a strict CSP in `tauri.conf.json` (self only). Do not build MSIX yet (Inc 5). Exit: a tag build produces signed NSIS and MSI when secrets exist; unit test for version monotonicity; CSP no longer null.

**P2.5 — Call-for-help bundle, runbooks, quick-start card** (S)
> Read `software-strategy.md` § Security and § Operability minimum. Implement `bundle_support`: zip `voxxa.toml` with tokens redacted, `voxxa.log`, the last 2 minutes of hypotheses from the current session log, OS and app versions, into the Desktop folder, show the user the file list before writing, and open the support e-mail draft with the path. Write three one-screen runbooks in `docs/runbooks/`: no audio device, presentation app not reachable, model missing or corrupt; link each from the matching red status message. Write `docs/quick-start.md`, one page, for the Windows launch. Do not add telemetry or automatic upload. Exit: bundle created and listed; runbook links resolve.

**P3.1 — Adapter trait, capabilities, contract-test harness** (M)
> Read ADR-006. Define `trait Adapter { fn caps(&self) -> Caps; fn advance(&mut self); fn prev(&mut self); fn goto(&mut self, song, slide) -> Result; fn show_ref(&mut self, r: &Reference) -> Result; fn current(&mut self) -> Option<Position>; fn setlist(&mut self) -> Option<Setlist>; }` in `adapters/mod.rs`. Refactor the keystroke adapter onto it with a foreground-window check (Windows and macOS via existing deps or a tiny platform call; Linux returns true) and `caps { drive: true, read: false, goto: false }`. Build the contract-test harness: a tiny in-process HTTP/WS stub that replays recorded request/response fixtures from `adapters/fixtures/<app>/`. Make the supervisor route `Goto` to `goto` when supported, else to ±1 `advance`/`prev`, else red. Do not write the vendor adapters yet. Exit: keystroke adapter passes its tests; harness replays a sample fixture.

**P3.2 — ProPresenter adapter** (L)
> Read ADR-006 and research finding A1 in `2026-10-24-research-report.md`. Implement `adapters/propresenter.rs` (≤ 300 lines) against the 7.9+ HTTP API: `GET /version` to confirm identity, `GET /v1/status/slide` for current and next text, `GET /v1/status/updates` (chunked stream) to observe operator changes, `POST /v1/trigger/next|previous`, `POST /v1/presentation/{uuid}/trigger/{index}` for `goto`, `GET /v1/playlist/current` and `GET /v1/presentation/{uuid}` to build `setlist.json` (one section per presentation group where groups exist). Host and port from `[adapter.propresenter]`. Record fixtures from a real ProPresenter install (the founder runs `voxxad --record-adapter` once) and commit them. Do not implement Network Link, MIDI, or messages/looks. Exit: contract tests pass; re-sync demonstrated when the operator clicks a slide in the stub; LOC ≤ 300.

**P3.3 — FreeShow adapter** (M)
> Read ADR-006 and finding A2. Implement `adapters/freeshow.rs` against REST `http://host:5506` (POST `{action,...}`) with optional `Authorization: Bearer`: `next_slide`, `previous_slide`, `index_select_slide {showId, index}` for `goto`, `get_output_slide_text` for `current()`, `get_projects` + `get_show {id}` for `setlist()`, `start_scripture {reference}` for `show_ref`. Use WebSocket 5505 only if REST lacks an event stream; otherwise poll `get_output_slide_text` every 500 ms. Record fixtures from a real FreeShow. Do not implement OSC or the Stage API. Exit: contract tests; LOC ≤ 300.

**P3.4 — OpenLP adapter** (M)
> Read ADR-006 and finding A3. Implement `adapters/openlp.rs` against `http://host:4316/api/v2`: `GET /core/system` (confirm `api_version: 2`), `GET /controller/live-items` for `current()` (selected slide text), `POST /controller/progress` and `/controller/show` for advance/goto, `GET /service/items` for `setlist()` (titles, ccli_number; slide text via `live-items` when the item goes live, cached), `POST /plugins/bibles/search` + `/plugins/bibles/live` for `show_ref`, `POST /core/login` when `authentication enabled`. Listen on WebSocket 4317 for `slide`/`item` changes. Record fixtures from a real OpenLP 3.x. Exit: contract tests; LOC ≤ 300.

**P3.5 — Planning Center Services setlist source** (M)
> Read ADR-007 and `software-strategy.md` § Setlist and text sources. Add `setlist/planning_center.rs`: with a personal access token stored in the OS keychain (`keyring` crate), fetch the next plan for a chosen service type and build `setlist.json` from plan items → songs → selected arrangement lyrics, splitting sections on arrangement section labels and slides on blank lines. Rate-limit politely; cache for the day. Do not write back to Planning Center or touch SongSelect. Exit: unit test over a recorded JSON fixture builds a valid v1 setlist.

**P4.1 — Public-domain Bible bundle** (M)
> Read ADR-007 and finding B6. Add `build/fetch_bibles.sh` (or a Rust build script step) that downloads WEB, KJV, ASV and BSB plain-text/USFX from eBible.org at pinned URLs with SHA-256 checks, converts to a compact `book|chapter|verse\ttext` format, compresses with zstd, and embeds via `include_bytes!`. Expose `Bible::verse(book, ch, v)`, `chapter(book, ch)`, and a book-name/alias table (Gen, Genesis, 1 Cor, First Corinthians…). Add attribution text for About and the README. Do not add any copyrighted translation or API client. Exit: all four load; `John 3:16` returns the WEB text; binary growth documented.

**P4.2 — Spoken scripture reference grammar** (M)
> Read `software-strategy.md` § Scripture detector. Implement `scripture/parser.rs`: recognise spoken references in a word stream — book aliases including ordinals ("first", "second", "third"), "chapter", "verse(s)", cardinals as words ("twenty-eight") or digits, ranges ("through", "to"), and "the next verse". Return `Reference { book, chapter, verse_start, verse_end, span_words, conf }`. Build a 200-case test table of spoken forms (Romans eight twenty-eight; first Corinthians thirteen verse four; Psalm twenty-three; John three sixteen through eighteen) including negatives ("eight o'clock", "first of all"). Compare with the `bibleref` crate (MIT) and use it only if it already handles spoken number words; otherwise keep this ≤ 300-line parser. Exit: ≥ 195/200 pass; false positives on the negatives = 0.

**P4.3 — Scripture detection in the tracker** (L)
> Read ADR-009 and § Scripture detector. Extend the supervisor with a `Songs+Scripture` mode: run the reference parser on hypotheses; when a reference appears with conf ≥ 0.8, load the chapter from the PD bundle and confirm by matching ≥ 5 of the next 7 recognized words against the verse; on confirmation emit `show_ref` and switch the tracker's graph to the chapter (verses as nodes, next-verse prior 0.8) until silence > 20 s or a song match returns. Add two eval fixtures (founder reads two passages aloud with timestamps) and extend `voxxa-eval` with reference precision and time-to-display. Do not display anything yourself; adapters do. Exit: eval ≥ 0.9 reference precision on the fixtures; song following resumes after the reading.

**P4.4 — `show_ref` per adapter** (M)
> Read ADR-006 capability flags. FreeShow: `start_scripture {reference}` (already stubbed) with the church's chosen version id from config. OpenLP: bibles search + live. ProPresenter: no API for Bible lookup in the examined spec; set `caps.show_ref = false`, show the reference on the GUI status line, and optionally send a stage message via `/v1/stage/message` when `[scripture] stage_hint = true`. Keystroke: status line only. Do not build a Bible presentation generator for ProPresenter. Exit: contract tests for the two real paths; status-line behaviour tested in the GUI dev flag.

**P5.1 — sherpa-onnx recognizer** (L)
> Read ADR-004. Behind the Cargo feature `sherpa`, implement `Recognizer` for sherpa-onnx's streaming Zipformer (English int8) using the `sherpa-onnx` Rust crate (static link), with its Silero VAD, emitting partial hypotheses every hop with word timestamps where available; implement `bias()` via hotwords if the crate exposes them, otherwise a no-op with a log line. Pin model download URLs and SHA-256; add to the first-run model list as "Streaming (fast, small)". Do not add Moonshine unless sherpa fails to build on Windows. Exit: `voxxad --recognizer sherpa_onnx --stdin-audio fixture.wav` produces decisions; CI builds the feature on all three OSes.

**P5.2 — Bake-off and default selection** (M)
> Read ADR-004's selection rule. Add `voxxa-eval --matrix` that runs every fixture (including the pilot corpus in `fixtures/pilot/`, not committed) across whisper base.en, small.en (CPU and, if available, Vulkan) and sherpa-onnx, recording precision, recall, p95 latency, false advances and CPU time per audio second. Write `.10x/reviews/YYYY-MM-DD-recognizer-bakeoff.md` with the table and a recommendation per hardware class; set the default engine and hop/window in `voxxa.toml` defaults accordingly; update ADR-004 status. Do not tune thresholds per fixture by hand. Exit: report committed; defaults changed only if the data says so.

**P5.3 — Microsoft Store packaging and release channels** (M)
> Read ADR-008. Add an MSIX packaging step (Windows SDK `makeappx`/`signtool` with Store-provided identity values from secrets) producing a Store-ready package from the same build; document the Partner Center submission checklist in `packaging/store/README.md` (screenshots list, privacy statement text pointing to the local-first policy, age rating). Add `channel = "community" | "stable"` to `voxxa.toml` and make the updater fetch `latest.json` or `stable.json`; the release job writes `stable.json` only when the tag carries `-stable`. Do not add in-app purchases or licence checks; the Store build is identical. Exit: MSIX validates with the Windows App Certification Kit; channel switching tested.

**P5.4 — Support operations artefacts** (S)
> Read `business-strategy.md` §4. Write `docs/support/sunday-support.md` (tiers, response commitments, Sunday window, what is and is not included), `docs/support/setup-and-calibration.md`, and `docs/support/contract-template.md` (plain-language annual agreement; mark for legal review). Add `usage.csv` writing in the supervisor: title, CCLI number if present, date, once per song per service. Update the docs site with a pricing section that reads prices from one place. Do not build a billing system; link to an invoice/payment page placeholder. Exit: docs render; `usage.csv` appears after a replay.

**P6.1 — macOS permissions and audio** (M)
> Read ADR-002 macOS row. On macOS: request microphone permission with a clear purpose string in `Info.plist`; detect missing Accessibility permission before the keystroke adapter sends anything and show a guided panel that opens System Settings; handle CoreAudio device changes (unplugged interface) by returning to amber, not crashing. Do not add Apple Speech or any macOS-only recognizer. Exit: fresh macOS VM reaches green with FreeShow and with keystroke fallback after granting permission.

**P6.2 — Notarized DMG and Homebrew cask** (M)
> Read ADR-008 macOS. Verify the existing notarization steps in `build.yml` run end to end with real secrets, producing a stapled DMG for arm64 and x86_64; add `packaging/homebrew/voxxa.rb` cask and a release step that updates a `homebrew-voxxa` tap via PR. Confirm the cask passes Homebrew's Gatekeeper checks. Do not submit to the Mac App Store. Exit: `brew install --cask nategroks/voxxa/voxxa` installs and opens without override.

**P6.3 — macOS QA pass** (S)
> Run the full fixture eval and the five-volunteer first-run script from `.10x/specs/2026-10-24-user-research-plan.md` §2 on an Apple Silicon machine; record results in `.10x/reviews/`; update `docs/quick-start.md` with the macOS section (permissions). Fix only blocking defects found. Exit: review file committed; quick-start updated.

**P7.1 — AppImage build** (M)
> Read ADR-008 Linux and Tauri's AppImage and linux-graphics docs cited in the research report (D6). Build the AppImage in an `ubuntu:22.04` container in CI with webkit2gtk 4.1; embed zsync update information pointing at the GitHub release; ship an `AppRun` wrapper that detects Wayland (sets the keystroke adapter unavailable with a clear message) and applies the documented WebKitGTK env-var workarounds only when a known failing GPU/driver is detected; keep `.deb` as a by-product. Do not add Flatpak. Exit: AppImage runs on Ubuntu 22.04, 24.04 and current Fedora; `AppImageUpdate` applies a delta.

**P7.2 — Linux audio and adapters QA** (S)
> On Ubuntu 24.04 with PipeWire and Fedora current: verify device enumeration and capture via cpal, run the fixture eval, test FreeShow and OpenLP adapters end to end (OpenLP is Linux-native). Record results in `.10x/reviews/`; fix only blocking defects. Exit: review file; eval numbers match Windows within noise.

**P7.3 — Release 1.0 and upstream documentation** (S)
> Write `CHANGELOG.md` for 1.0 from the git history and ADRs; finalize README (install per OS, supported apps table from ADR-006, support link); open documentation PRs to FreeShow and OpenLP adding Voxxa to their integrations pages (text only); add a note for Bitfocus Companion users on driving `voxxad` from the CLI. Tag `v1.0.0`. Do not add features. Exit: release published on all three platforms; PRs opened.

## 8. What to do this week (engineering)

1. Run P0.1–P0.3 (one afternoon).
2. Record the two PD-hymn fixtures with slide timestamps (needs a mic, 1 hour).
3. Run P1.1 and P1.2; start P1.3.
4. Apply to SignPath Foundation and open the Partner Center account so Inc 2 is not blocked on paperwork.
