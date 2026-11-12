# SDE — church-autoadvance

## Increment 0 (2026-10-25) — reposition, dead weight, licence, CI

### Built (P0.1)
- README rewritten for the church product; landing page `docs/index.html` copy replaced in place (title, meta, hero, six feature cards, three steps, footer licence) with the design untouched; `Cargo.toml` description; release body in `build.yml`. `grep -ri dictat` is clean outside git history.

### Built (P0.2)
- Removed the `mcp` module, the `rmcp` optional dependency and the `mcp` feature; removed `tauri-plugin-store` (code, Cargo, lockfile). Lockfile pruned accordingly.
- Added `LICENSE` (GPL-3.0-or-later, 674 lines), `DCO` (Developer Certificate of Origin 1.1), `CONTRIBUTING.md`, `license` fields in `Cargo.toml` and `package.json`, and `src-tauri/deny.toml` (allow-list of GPL-compatible permissive licences; anything else fails).

### Built (P0.3)
- `.github/workflows/ci.yml`: on pull_request, push to main and manual dispatch; matrix ubuntu-22.04 / windows-latest / macos-latest running frontend build → `cargo fmt --check` → `cargo clippy --all-targets -D warnings` → `cargo test`; separate `cargo deny` job (EmbarkStudios action) and a line-budget job (`scripts/loc_budget.py` via tokei: core Rust ≤ 3,000, GUI ≤ 800). Current counts: 918 and 598.
- `cargo fmt` applied to three files; six clippy findings fixed with minimal edits (`Default` impls for `AudioEngine`, `TranscriptionEngine`, `VadEngine`; derived `Default` for `WhisperModel`; two `2 * 1 * 64` → `2 * 64`).

### Deviations from the prompts (and why)
- Added the `macos-private-api` Cargo feature to `tauri`: the tauri build script refuses a plain `cargo clippy`/`cargo test` while `macOSPrivateApi: true` is set in `tauri.conf.json` (the Tauri CLI adds the feature implicitly). Both the flag and the hidden `overlay.html` that needs it go away in P2.1.
- `cargo deny` surfaced 17 advisories in transitive dependencies. Fixed by compatible updates to `anyhow`, `h2`, `rustls`, `rustls-webpki`, `tokio`, `webpki-root-certs` and `plist` (which carries `quick-xml`). Three `zip` advisories cannot be fixed yet (every `tauri-plugin-updater` release through 2.14.0 requires `zip` 4.x); they are ignored in `deny.toml` with the reasoning and a removal trigger. `unmaintained` is set to `workspace` scope because the `unic-*`, `fxhash` and `proc-macro-error` findings are transitive through tauri with no upgrade.
- Reverted a speculative `tauri-plugin-updater` 2.10.1 → 2.12.0 bump that did not change `zip`; kept the diff to what the gate needed.

### CI finding fixed after the first run
- The Windows job failed in the tauri build script: `RC2175: icons/icon.ico is not in 3.00 format`. Both `icon.ico` and `icon.icns` were PNG files with the wrong extension (pre-existing; the tag-triggered release build would have failed the same way on Windows and macOS). Regenerated both with `npx tauri icon` from `icons/icon.png` and copied only those two files in. shortcut: the master `icon.png` is 128×128, so the 256 px and retina entries are upscaled; replace with a 1024 px master before the first public release.

### Tech debt created or carried
- `overlay.html` and `macOSPrivateApi` remain until P2.1 deletes them.
- `deny.toml` advisory ignores for `zip` must be removed when the updater plugin moves to `zip` 9.
- No automated tests exist yet; `cargo test` passes trivially. P1.1 adds the first aligner tests.

### Verification
Local (Ubuntu, rust 1.97): `cargo fmt --check` ok; `cargo clippy --all-targets -- -D warnings` ok; `cargo deny check licenses advisories bans` ok; `cargo test` ok; `scripts/loc_budget.py` ok. CI on the three-OS matrix: see the workflow run linked in the commit.

## 2026-11-11 — dependency baseline moved to tauri 2.12 (CI fix on PR #1)
- Nine advisories published after 2026-10-25 turned `cargo deny` red: eight in `openssl` (fixed by the compatible 0.10.82) and RUSTSEC-2026-0429 in `zip` 4.x via `tauri-plugin-updater`.
- `tauri-plugin-updater` 2.14.1 depends on `zip` 9 but needs `tauri` 2.12, which needs Rust 1.90; the resolver held `tauri` at 2.10.3 because of `rust-version = "1.77"`. Raised `rust-version` to 1.90 (CI builds on stable 1.97) and updated the tauri family together (`tauri`, `tauri-build`, `tauri-codegen`, `tauri-macros`, `tauri-plugin`, `tauri-runtime`, `tauri-runtime-wry`, `tauri-utils`, `wry`, `tao`, the three plugins). `zip` is now 9.0.0; the three earlier `zip` advisory ignores are deleted from `deny.toml`.
- Verified locally: fmt, clippy -D warnings, test, budgets; CI green on all five jobs at f177fbe. Note for prompt Q1: the merge must keep this baseline (the May branch declares `rust-version = "1.77"` and older tauri plugins).

## 2026-11-11 — Q1: merge of `claude/affectionate-lamport-8al8B` (ADR-010 as amended by A1–A15)
**Merge record.** `git merge --no-ff origin/claude/affectionate-lamport-8al8B` (f7b56e6) onto 97aa915. Six textual conflicts, each resolved hunk by hunk per Appendix A: `build.yml` ours ×2; `README.md` ours; `Cargo.toml` May's dependency block (our `[package]` auto-merged); `Cargo.lock` started from ours, cargo added May's 66 crates, no bare `cargo update`; `commands/mod.rs` May ×2; `vad/mod.rs` May's `backend,` plus the auto-merged `impl Default`. Auto-merges in `audio`, `lib` (no store plugin), `transcription` kept; `text_insert` deleted by git. Follow-up commits: A9/A10 content, A11 README, A12 build.yml + SIGNING.md, fmt, clippy (4 lints), budgets (A8), GUI trim, deps (A2/A5), regenerated schemas. **GUI budget, revised:** the first GUI trim (2e3dcd9) deleted the dead `src/overlay.html` and folded seven duplicate rules in May's `main.css`. The CSS fold was reverted in a later commit: tokei 15 counts any line after an ASCII apostrophe in HTML text as a string, so `index.html` read 408 code lines instead of 351. Three apostrophes in visible text (`Don't`, `you've`, `won't`) became U+2019, which changes no behaviour, and `main.css` is May's again. `overlay.html` stays deleted. Result: GUI 2,966/3,000 (A4: edits to May code only to pass gates; the fold was no longer needed).

**Dependency changes (from → to, why).**
| Crate | From | To | Why |
|---|---|---|---|
| zip | 2 (2.6.1) | 9 (9.0.0) | RUSTSEC-2026-0367/0368/0369/0429; same version the updater already uses |
| quick-xml | 0.36.2 | 0.41.0 | RUSTSEC-2026-0194/0195; `unescape()` gone, entities now `GeneralRef` → `importers::xml_text` |
| pdf-extract (lopdf) | 0.8.2 (0.34) | 0.12.1 (0.42) | RUSTSEC-2026-0187; fixed release exists, so no feature gate |
| rusqlite (libsqlite3-sys) | 0.32.1 (0.30.1) | 0.41.1 (0.39.1) | RUSTSEC-2026-0459; bundled kept (see `decisions/dba/`) |
| reqwest 0.12 | default (native-tls) | rustls-tls-native-roots | Q1 prompt: no OpenSSL at runtime; openssl remains a build-dep of `ort-sys`. Native roots (OS trust store via `rustls-native-certs` 0.8.3, already locked) instead of bundled webpki roots, so church networks with a TLS-inspecting proxy keep working; lock change: +`rustls-native-certs` edges, −`webpki-roots` 1.0.12 |
| env_logger | 0.11 | removed | unused (A5) |
| tauri / tauri-build / updater / shell / global-shortcut | "2" | "2.12" / "2.7" / "2.14" / "2.6" / "2.4" | A2 floors at the locked post-f177fbe versions |
| @tauri-apps/api (npm) | ^2 | removed | unused runtime dependency (A8) |

**Gates (Windows 11, rustc 1.99.1):** frontend build ok; fmt ok; clippy `--all-targets -D warnings` ok; test 37 unit + 4 integration ok; `cargo deny check licenses advisories bans` ok (one warning: `voice_activity_detector` has no licence field, MIT by file); `loc_budget.py` ok (GUI 2,966/3,000). Linux and macOS are CI-only. Re-run from scratch after the review fixes below (npm ci + build, fmt, clippy, test, deny, budgets, `git grep -i dictat`): all pass.

**Review fixes (comparator of the 10x and ponytail lanes, applied as new commits).** GUI budget via U+2019 apostrophes, CSS fold reverted (ponytail graft); reqwest `rustls-tls-native-roots`; README matching bullet and ADR-011 preview wording; website roadmap cards labelled "Coming soon"; metainfo/.desktop/release-body positioning copy (Appendix A row 61); ADR-007 public-domain text in importer tests and `SETLIST_FORMATS.md` (aligner fixtures handed to Q2, see Debt); `.pptx` importer unit test; `loc_budget.py` counts tokei blobs; Cargo.toml OpenSSL comment.

**Debt.** GUI is 34 lines under budget; copyrighted lyric fixtures remain only in `src-tauri/src/aligner/mod.rs` ("How Great Is Our God", "Way Maker"): **hand-off to Q2**, because Q1 must not touch the conductor (ADR-007; the importer tests, `planning_center` tests and `docs/SETLIST_FORMATS.md` now use public-domain hymn text); the 12 Q0 audit defects are untouched (Q2); `fixtures/setlists/pd-hymns.json` and `fixtures/pco/plan-sample.json` still do not fit May's parsers (A15 follow-up); `docs/signing-policy.md` and `SIGNING.md` both describe Windows signing (A6 reconcile, partly done).

## 2026-11-12 — Q3: `voxxad` CLI over the library (ADR-013), 10x lane `q3/10x`
**Commits.** 592949b ADR-013 (Phase 2, before code); 0669328 refactor: the loop moves to the library; 2f588df feat: the `voxxad` binary, budget row, tests and fixtures.

**What was built.**
- `voxxa_lib::listen::run(rx, run, vad, conductor, transcribe, clock, sink) -> bool` (`src-tauri/src/listen.rs`): the body of `commands::start_listening_with_state`, unchanged in order and locking, with three parameters. `clock: FnMut(samples_received) -> Instant`, `transcribe: FnMut(&[f32]) -> Result<String>` (the ADR-004 seam as a closure), `sink: FnMut(Instant, Event)`. `Event` = `Input(Input)`, `Action(Action, Option<usize>)`, `State(MachineState, bool)`, `Mic(MicLevel)`, `Detection(Vec<DetectionRow>)`, `Error(String)`. Returns true when the channel closes while `run` is set (device loss or end of file).
- The GUI calls it with `|_| Instant::now()` and a sink that emits `transcription`, `presenter-error`, `machine-state`, `mic-level`, `song-detection` exactly as before and dispatches through `commands::dispatch`; `listening-stopped {disconnected}` and `Sessions::end` stay in `commands`. `MicLevel` and `DetectionRow` moved to `listen.rs` (same fields and serde names).
- `aligner::Input { Tick, Speech, Heard(String) }` and `Conductor::apply(&Input, now) -> Action`: the only way the loop and replay drive the conductor. `on_speech` carries a comment: it must keep only recording `now`, or `voxxad`'s coalesced speech log stops replaying exactly.
- `src-tauri/src/bin/voxxad/main.rs` (flags, WAV reader, feeder thread, presenter, model pick, session file) and `events.rs` (schema v1 `Line`/`Kind`/`Act`, `Recorder`, `replay`, `fingerprint`, `expired`). Modes: file speed (sample clock, `t` = audio ms), `--realtime` and `--device` (wall clock rounded to whole ms). Ctrl-C clears the run flag, so the `stop` line is written.
- Session log: `<data dir>/voxxa/sessions/session-YYYYMMDD-HHMM[-N].jsonl`, one line per write (no buffering), expired names (> 14 days by the name's date) deleted at start.
- Tests: `src-tauri/tests/voxxad_events.rs` (13 tests, no model or device): every type's exact wire form and round trip; Noop dropped and no slide text on `action`; speech coalescing; replay of a real recorded session byte-identical on actions, recomputed from inputs only (actions and status lines stripped first), different when the transcripts change, refused for another setlist; fingerprint; 14-day expiry; the shared loop on 1 s of silence ticks at 200/400/600/800/1000 ms of audio and reports end of audio, and a stopped session returns false; the binary's `--replay` output and its error lines (8 kHz WAV, missing setlist, no source, unknown flag, unknown presenter). Fixtures: `tests/fixtures/voxxad/pd-hymns-setlist.json` (Amazing Grace, Be Thou My Vision, It Is Well) and `session-pd-hymns.jsonl`, the exit-demo session (Whisper Tiny on synthetic speech of the public-domain text plus a spoken prayer; absolute log path reduced to the file name).
- Mutation check: replay ignoring `speech` lines fails 3 of the tests.

**Exit demo (Windows 11, debug build).** `voxxad --stdin-audio service.wav --setlist pd-hymns-setlist.json --shadow` (47 s: verse 1, verse 2, 7 s gap, prayer, 7 s gap): goto AG/0 at 8.1 s, goto AG/1 at 24.2 s, blank at 29.2 s; the prayer shows nothing. `voxxad --replay <its session log> --setlist …`: the three action lines are byte-identical (`cmp`), and so are all hypothesis and status lines. With `--presenter propresenter7_rest --port 18125 --shadow` against `scripts/qa/mock_pro7.py`, the mock's `/_log` stayed `[]`; the same run without `--shadow` sent version, slide_index, two triggers and a clear. A `--realtime` run fed from stdin also replayed byte-identically.

**Gates (from the worktree root, rustc 1.99.1):** npm ci + build ok; fmt ok; clippy `--all-targets -D warnings` ok; test 102 passed (54 unit + 48 integration, 13 new); cargo-deny licenses/advisories/bans ok; budgets ok: everything else 2,496/2,500 (+13 for the loop and its parameters), conductor 1,153/1,200, voxxad 504/600, no other row changed; `git grep -i dictat` empty.

**Deviations from the prompt.**
- Flags added: `--host`/`--port` (presenter address; needed for the mock), `--model` (default: smallest downloaded), `--log-dir`; `--device default` names the system default; `--stdin-audio -` reads stdin. A source must be given explicitly, so no flag combination opens the microphone by accident.
- Session logs sit in a `sessions/` folder under `<data dir>/voxxa`, so rotation never looks at models or other files.
- `start`, `stop` and `--list-devices`' `devices` are `status` states; `tick` and `speech` exist only in the session log (6 types, ADR-001 allows 8).
- `action` leaves out slide text (the setlist has it).
- `--replay` needs `--setlist` (fingerprint-checked) and prints hypothesis, action and conductor status lines, not start/stop.
- `--shadow` never builds or connects a presenter, so it also does not check that the presenter is reachable.
- `Cargo.toml` gains `default-run = "voxxa"` so `cargo run` and `tauri dev` still start the app.

**Debt / for later prompts.**
- `--presenter keystroke` from a terminal has no own-window guard (voxxad has no window): the keys go to whatever has focus, the terminal included. Q5 docs must say so; candidate ticket (foreground-window check against the console's window).
- Transcription stays synchronous in the loop (Q0): in a debug build `--realtime` falls behind (a 47 s file ended at t = 92 s) and its timing differs from file speed (one extra re-goto of slide 0). Replay is still exact. Measure in release for Q4.
- `--device` was never run (no microphone, by the safety rules): founder checklist.
- The GUI writes no session log; operator inputs (manual steps, song jumps, external blanks) are not `Input`s yet, so a GUI session log would need new variants before it could replay.
- WAV input must already be 16 kHz mono (16-bit PCM or 32-bit float); no resampling of files. No `--language` flag (auto-detect, as the GUI's default).

## 2026-11-12 — Q3 finish on `q3/base`: 10x base + ponytail grafts
**Commits.** f6d18b3 merges `q3/10x` (`--no-ff`); then one commit per item chosen by the comparator:
- 7cc75b2 the 10x review's should-fix: `tests/voxxad_events.rs` held four raw NUL bytes and two raw newlines inside string literals, so git and grep saw a binary file and `rg` skipped it. Now escapes; `rg replay_reproduces src-tauri/tests` finds the test.
- 029acf3 the file feeder uses `sync_channel(64)` (6.4 s of audio), so an hour-long WAV is not copied into the queue a second time (about 230 MB). The clock counts samples, so determinism is unchanged.
- 587be77 replay errors name the line (`<path>: session line N: …`) and a line whose `v` is not 1 is refused; test with a cut-off last line and a `v:2` line.
- 5cb0d76 every stdout line goes through `say()` (`writeln!` on the locked stdout, error ignored). Before: `voxxad --replay … | true` exited 101 with "failed printing to stdout: The pipe is being closed. (os error 232)"; after: exit 0, nothing on stderr.
- 23c459c `--presenter keystroke` without `--shadow` prints one stderr line, "voxxad: keystroke types into the focused window; focus the presentation app", before any audio is read; tested with an 8 kHz WAV so no presenter is built.
- d5cf9cf `--replay` with any live-run flag, and `--realtime` without `--stdin-audio`, are usage errors (they were ignored); tested.
- e8d6306, b0943f7, 1f5e31b ADR-013: the privacy sentence corrected (14-day transcripts under the roaming `%APPDATA%`; `data_local_dir` recorded as the alternative), a release-packaging risk (installers will probably carry `voxxad`; guard `required-features` or a bundler exclusion; ticket **Q3-T1**), and the text matched to the grafted code.

**Gates (from the worktree root, rustc 1.99.1, Windows 11):** npm ci + build ok; fmt ok; clippy `--all-targets -D warnings` ok; test 103 passed (54 unit + 49 integration; `voxxad_events` 14); cargo-deny ok; budgets ok: everything else 2,496/2,500, voxxad 531/600, conductor 1,153/1,200, no other row changed; dictat grep empty.

**Exit demo (debug build, Whisper Tiny).** A 45 s WAV (1 s silence, Amazing Grace verses 1 and 2, 7 s silence, a spoken prayer, 7 s silence; public-domain synthetic speech) with `tests/fixtures/voxxad/pd-hymns-setlist.json`: `--shadow` printed goto Amazing Grace/0 at 8.1 s, goto Amazing Grace/1 at 22.8 s, blank at 27.8 s; the prayer showed nothing. `--replay <its session log>`: the 3 action lines are byte-identical (`cmp`) to stdout and to the log, and every hypothesis and conductor status line matches too; replay wrote nothing to stderr (no model loaded). With `--presenter propresenter7_rest --port 18225 --shadow` against `scripts/qa/mock_pro7.py` the mock's `/_log` stayed `[]`; a control run without `--shadow` sent version, slide_index, triggers 0 and 1 and a clear.

**Debt added.** Q3-T1: check whether `tauri build` bundles `voxxad` (`tauri build --no-bundle` or the CI build job), then guard it or decide to ship and sign it (Q5).
