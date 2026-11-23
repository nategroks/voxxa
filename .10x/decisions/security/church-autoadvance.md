# Security — church-autoadvance

## 2026-11-20 — Q9V final (`q9v/base` @ be4819b; both verifiers passed in round 2)
Details: the Q9V lane section at the end of this file; ADR-019 §§ 7-9 and its Q9V amendment; `reviews/2026-11-20-ponytail-q9v.md` §§ 6-8.
- **Decision: no Critical or High finding is open; the security position for Q9V is final.** Ready to merge once CI is green.
- **The repo refuses media, twice, and the suite now runs both checks against planted files.** `media_guard.rs` reads `git ls-files -z` from a real repository (an accented-path test proves the parsing), and runs the ci.yml `media` step with bash on a temporary repo of 17 planted paths staged as empty-blob index entries, so no media-named file is ever written to disk. Every planted path is named; the job may not carry `continue-on-error`. Verifier B listed every path in all 444 commits of `git log --all`: the only media are the 15 allowed WAVs, and nothing was ever under `fixtures/private` (also checked case-insensitively).
- **`--full` prints numbers only.** `voxxad`'s stderr goes to null; failures, error lines, bad lines and bad setlists are named by exit status, line number or line and column. Verifier A drove its own fake `voxxad` through 7 run shapes (failures, a panic, bad lines, an error line, `--record`, a normal run), 12 malformed truth lines and 10 malformed setlists carrying placeholder words: none was printed. A real `--full` run with Whisper Tiny on a public fixture wrote 0 bytes to stderr. Replay (public fixtures, CI) still shows `voxxad`'s text, as designed and now pinned.
- **No new crate, capability or network path.** Only a `[[test]]` entry in `Cargo.toml`; `Cargo.lock`, `deny.toml`, `tauri.conf.json`, `capabilities/` and `package.json` are unchanged (verifier B); cargo-deny passes. The `media` job uses only `actions/checkout@v4` and the default read-only token.
- **Accepted risks (tickets):** song text copied out of `fixtures/private/` passes both checks (Q9V-01, medium); "media guard" is not yet a required check (Q9V-02, founder); an `if:` gate on the job is untested (Q9V-03); the guard matches names, not content (Q9V-05); a killed `--full` run leaves its temporary session log (Q9V-06); `--record` on a private fixture is refused by the docs only (Q9V-07); a path with a newline can add a line to a failing job's log (Q9V-09).
- **Safety in every Q9V run:** no agent searched for, opened, copied, hashed or converted the .mp4 or any church media; nothing was read under `%APPDATA%\Presenter`; Presenter was not touched; no microphone; the GUI app was not started; nothing went to a network service or Google Drive; every plant was a dummy in a throwaway clone, a detached worktree or a test's temporary repo, never committed or pushed. `git log --all -- fixtures/private` is empty.

## 2026-11-20 — Q9W final (`q9w/base` @ b0de71b; both verifiers passed)
Details: the Q9W lane section at the end of this file; ADR-019 and its four amendments; `reviews/2026-11-20-ponytail-q9w.md` § 6.
- **Decision: no Critical or High finding is open; the security position for Q9W is final.** Ready to merge once CI is green.
- **Keys go to fewer places than before.** With the WorshipTools profile, `tap()` asks the pure `guard(profile, front_exe)` before every key (`presenters/keystroke.rs:206`); a key aimed at any window that is not Presenter's is not sent. Verifier B's guard mutants (removed, inverted, cfg-disabled, always Ok, unreadable window allowed) are all killed, and verifier A tried 7 non-Presenter front windows: each refused, no key recorded.
- **No test can press a real key.** Every keystroke test driver uses a recording sender and a fake front window (8022bfd); `tests/worshiptools.rs` connects a real driver only to read its capabilities.
- **Process enumeration stays read-only:** `PROCESS_QUERY_LIMITED_INFORMATION` and `QueryFullProcessImageNameW`, handle closed; only the fixed name "WorshipTools Presenter" leaves the backend. Whether a query handle on Presenter's process counts as "touching" it is a founder question (Q9W-06); the ponytail lane's Toolhelp32 snapshot would open none.
- **Key map:** F10, F11, F12, Up, Down and the number pad's + and − are never sent; "." is gone; blanking stays off until the founder confirms the unblank key (Q9W-02).
- **No new crate, capability or network path.** `Cargo.toml`, `Cargo.lock`, `deny.toml`, `tauri.conf.json` and `capabilities/default.json` are unchanged (verifier B); cargo-deny passes.
- **Accepted risks:** any `Presenter.exe` counts as Presenter (Q9W-05); Presenter run as administrator drops keys silently (Q9W-07, M15); the guard passes off Windows (ADR-019 amendment; #33); `voxxad --presenter keystroke` stays universal with no guard (#48); a multi-key step stopped halfway by the guard leaves Voxxa behind the screen (Q9W-01, medium).
- **The guarded real-app runs** (`qa9wx`, `qa9wp`, `qa9wf`, `qa9wa`, `qa9wb`) never listened and never dispatched; Presenter's 8 processes and the installed Voxxa were never touched; every scratch `%LOCALAPPDATA%` folder was deleted.

## 2026-11-19 — Q9 final (`q9/base` @ 89f4ccb; both verifiers passed)
Details: the Q9 section below; ADR-018; `reviews/2026-11-19-ponytail-q9.md` § 6.
- **Decision: no Critical or High finding is open; the security position for Q9 is final.** Ready to merge once CI is green.
- **#102 (Q7-08) is fixed** (2c28c15) and can be closed at merge. Verifier B's mutants on it: `Heard: <text>` at Info and a `warn!` of the window text are both killed. One survivor: the same text at Debug reaches the ring buffer only under `RUST_LOG=debug`, because the test's logger runs at Info (Q9-11, low). ADR-018 § 1 says the text is never logged at any level; the code holds to that today.
- **Redaction widened after verifier round 1** (3e3efd7): any name containing `key` (once `keystroke` is removed), plus `applicationid`, `clientid` and `bearer`. Verifier A checked 18 placeholder secrets in a real written zip, all redacted, with harmless fields kept. Redaction is by property name only (Q9-16, low); no secret is stored under a harmless name today.
- **No network, no new capability, no new crate.** Verifier B found no network code in the new files. The only targets opened are the zip's folder, the recipient-free `mailto:` and the whitelisted GitHub runbook URL. `capabilities/default.json`, `Cargo.toml`, `Cargo.lock`, `deny.toml` and the workflows are unchanged.
- **Nothing written before Save, and Save writes only what was shown:** proved by `tests/help_commands.rs` running the real preview and Save steps (round 1 survivors M2b and M5b are now killed), and in the guarded real app.
- **The QA override `VOXXA_HELP_DIR` ships in the backend.** It opens nothing and only moves the folder; whoever can set the process environment already runs code as the user. Accepted (ADR-018 § 6).
- **Accepted risks, unchanged:** CSP is still null (#8; Q10 sets a strict CSP), so a compromised webview could preview any `storage` and write it to the Desktop under the fixed name; the report lists audio device names and song titles (Q9-14, for Q11's over-collection review); the words file can come from an old `voxxad` session (Q9-08).
- **Next:** Q11's security review covers the support bundle for over-collection; the shell plugin's deprecated `open` moves to `tauri-plugin-opener` in its own PR (Q9-03).

## 2026-11-19 — Q7 final (`q7/base` @ 6386f6b; both verifiers passed)
Details: `reviews/2026-11-19-security-review.md` §11; the ponytail audit: `reviews/2026-11-19-ponytail-q7.md` §3.
- **Both verifiers passed at 6386f6b.** A TCP-connect mutant and a UDP-socket mutant in the replay branch each fail the zero-sockets test on Windows (ETW); the model hash and every action pin were re-checked; product code changed only by the `src/bin` → `src/cli` rename and one error string.
- **Decision: no Critical or High finding is open; the security position is final.** Ready to merge once the PR's first CI run shows the Linux and Windows socket tests green and the Linux mutant red then green.
- **Added from the ponytail audit:** Q7-08 (Medium, privacy): the GUI logs each transcript at Info, and the diagnostics report copies it. Fix before any pilot leaves shadow mode. Q7-09 (Low): `voxxad --device` opens the microphone before checking its inputs. Q7-11 (Low): `voxxa-eval`'s "voxxad failed" error carries recognised words.

## 2026-11-19 — Q7 verifier fixes on `q7/base` (aa16e90)
Full text: `reviews/2026-11-19-security-review.md` section 10.
- **Decision: on Windows, a replay is charged only with its own sockets, during its own lifetime.** The ETW session adds `Microsoft-Windows-Kernel-Process` (start and exit), and only socket-creation events (1000) count (until the PR #95 CI fix, also connect 1017, 1018, 1020, 1021: a busy CI runner charged six ConnectEx completions with no socket to a replay's pid, so only creation, which AFD logs in the caller's context, counts now). Before, any AFD event with a matching pid counted, and gate 4 was flaky (a reused pid or a borrowed context). A child without a recorded start or exit fails the test.
- **Decision: CI's Linux job requires a network namespace** (`VOXXA_REQUIRE_NETNS=1`). A runner that refuses `unshare -rn` now fails the test instead of passing with networking enabled. A green job proves which tracer ran.
- **Re-proof:** 4 of 4 full `cargo test --features cli` runs pass. The replay-branch TCP-connect mutant fails with 4 socket/connect events.

## 2026-11-19 — Q7 finish on `q7/base`
Full text: `reviews/2026-11-19-security-review.md` section 9.
- **Decision: the network claim is "`--replay` opens no socket; `--shadow` makes no network connection"** (on Linux/macOS tokio's signal driver creates one local AF_UNIX socketpair, `src/cli/voxxad/main.rs:140`). "`--shadow` opens no socket" was too strong; fixed in the release notes, INTEGRATIONS, the review and Q7-07 (b0efc3f). A future strace check of `--shadow` must allow `socketpair(AF_UNIX)` or count only AF_INET/AF_INET6 sockets and connects.
- **Decision: Linux CI stays on `ubuntu-22.04`** until `unshare -rn` is shown to work on a newer image (Ubuntu 24.04's AppArmor default refuses it, and the test would fall back to strace with networking enabled). Comment in `ci.yml` (5fa4c76).
- **Re-proof at the finish (Windows, ETW):** pass; the replay-branch TCP connect mutant fails with 11 AFD events; pass again after the revert.
- **Before merge (must-fix evidence, not yet run):** in the first CI run, (a) Linux and Windows print `traced with strace inside unshare -rn` and `traced with ETW`; (b) the throwaway mutant commit turns Linux red and its revert green; (c) full-eval green or warning on three OSes with only the table in the summary and artifact. Record the run URLs in both reports.

## 2026-11-19 — Q9: Call for help (the help bundle), #102, runbook links (10x lane, `q9/10x`)
ADR-018. Security Engineer review of the new surface, written with the ADR and checked again at the finish.
- **#102 (Q7-08, Q7-S6) fixed first, in its own commit (2c28c15).** `listen.rs` logs `Heard N words` at Info instead of the text, and no longer logs the text at any level (Debug would still reach the ring buffer under `RUST_LOG=debug`, because the ring logger takes its level from `RUST_LOG`). All 70 `log::` calls in `src-tauri/src` outside `cli/` were read. None carries recognised text: the conductor logs indices, scores and titles, the presenters log slide indices, and dispatch errors name titles at most. `tests/help_bundle.rs::no_recognised_text_reaches_the_diagnostics_buffer` runs the shared loop with four placeholder words and checks the buffer, which is the report's `recent_log`. It fails on the old line.
- **Where the heard words go now.** Only into the optional `words-last-2-minutes.txt`. It is built from the newest `voxxad` session log (the GUI writes none, #53), shown in full on the review screen, and controlled by a checkbox that is on by default. Unchecked, the file is absent from the zip (Rust test and e2e).
- **Nothing is written before the user confirms.** `help_bundle_preview` builds the files in memory (Rust test: the folder is unchanged). `write_help_bundle` writes exactly the pending files that were shown, with `File::create_new` (no overwrite), and removes a half-written file. A second write without a new review is refused (shown in the real app).
- **Redaction (operator settings).** Only `voxxa.*` keys go in. `voxxa.recentSetlists` (whole setlists, slide text included) is replaced by a placeholder. A property, at any depth, whose name reads as a secret is replaced by `[redacted]`: after `keystroke` is removed from it, the name contains token, secret, pass, pwd, credential, authoriz, appid, applicationid, clientid, bearer or key, or ends in auth or pat (widened after the Q9 verifier found `apiKeys`, `privateKeyPem`, `key_id`, `application_id` and `clientId` unredacted). The rule is tested on nested, top-level, camelCase, snake_case and upper-case names. The test also checks that no `SmartConfig` field and no stored key is redacted. The first version missed `passphrase`; the test caught it and the rule was widened. Passwords and the HTTP API token are not in `localStorage` today (`src/main.js:1131-1133,1492`), and Planning Center credentials are kept in memory only (`src/main.js:157`), so this is a second line of defence.
- **No network, no new capability.** `capabilities/default.json` is unchanged (`core:event:default`; `tests/capabilities.rs` passes). The e-mail draft, the folder and the runbook URL are opened from Rust with `tauri-plugin-shell`'s `open`, which applies no scope when called from Rust. The webview cannot choose what is opened:
  - `open_help_email` takes no argument and opens only `mailto:` + the empty `SUPPORT_EMAIL` + a subject, plus a body built from the zip this process wrote, percent-encoded (Rust test: only unreserved characters and `%XX`), and then that zip's folder;
  - `open_runbook` accepts three fixed names (`../../evil` was refused in the real app).

  A `mailto:` link hands the draft to the mail program, which sends nothing until the user presses Send. There is no upload and no telemetry.
- **QA override `VOXXA_HELP_DIR`.** The zip is written to that folder, session logs are read from `<dir>/sessions`, and nothing is opened. Whoever can set this process's environment already runs code as the user. Accepted.
- **Accepted risks:**
  - A compromised webview (CSP is off, as before) could ask for a preview with any `storage` and write it to the Desktop under the fixed `voxxa-help-<UTC>.zip` name.
  - The report lists audio device names, which can contain a person's name. They are shown on the review screen before anything is written.
  - The words file can come from an old `voxxad` session. Its header names the log file, so the date is visible.
- **The real app was never allowed to open anything:** the guarded run wrote both zips through `write_help_bundle` into the scratch folder and never called `open_help_email`; no zip appeared on the real Desktop.

## 2026-11-19 — Q7: the CLI surface, the zero-sockets test, the full-eval workflow's supply chain
Full review: `reviews/2026-11-19-security-review.md`. Tickets: `reviews/2026-11-19-q7-tickets.md` (Q7-01, -03, -05, -06 and -07 are security-tagged).
- **Decision: "replay opens no socket" is a test, not a claim.** `src-tauri/tests/replay_no_sockets.rs` runs `voxxad --replay` on every committed log under an OS tracer:
  - Linux: `strace -e trace=%network`, inside `unshare -rn` (networking disabled) when user namespaces allow it.
  - Windows: an ETW session on `Microsoft-Windows-Winsock-AFD`.

  A positive control (the test binary connecting to 127.0.0.1) must be seen first, so a blind tracer fails the test. CI sets `VOXXA_REQUIRE_SOCKET_TRACE=1` on Linux and Windows, so a missing tracer fails there instead of skipping. macOS is skipped and says so (no tracer without SIP changes; Q7-01). Mutation proof on Windows: a TCP connect added to the replay branch makes the test fail with 11 AFD events.
- **Shadow mode:** a full `--shadow` run, with Whisper and a network presenter named on the command line, made 0 AFD events under ETW. This was checked locally only, because CI's PR gate has no model (Q7-07).
- **Fixed now:**
  - Q7-S1 (Medium): the Windows installer carried `voxxa-eval.exe` at 46cb6d1. Fixed in b9692c5; see the DevOps record.
  - Q7-S2 (Low): `ci.yml` now has `permissions: contents: read`.
- **Full-eval workflow:**
  - The model is pinned by SHA-256 and checked after every download and every cache restore. The hash equals the Hugging Face LFS oid.
  - Every action is pinned to a commit SHA, with `contents: read`, `persist-credentials: false`, no secrets, and the `pull_request` trigger (not `_target`).
  - Whisper's stderr (recognised words) is kept out of the log.
- **Accepted:** a crafted session log can make replay print any actions; replay never dispatches. CLI inputs are the user's own local files.
- **Still open from earlier, docs done in Q7:** #48 (keystroke focus guard), #49 (transcripts in the roaming profile).

## 2026-11-18 — Q6 close-out: two new RustSec advisories cleared in code
- **What happened:** between 2026-11-13 and 2026-11-18 the advisory database gained RUSTSEC-2026-0543 (chrono unmaintained, no fixed release) and RUSTSEC-2026-0548 (zip 9.0.0, `ZipStreamReader` trusts local-header names). Gate 5 (cargo-deny) failed on an unchanged `Cargo.lock`, on `q6/base` and on 733c7e1 alike.
- **Exposure:** none known. Voxxa reads `.pptx` with `ZipArchive` by entry name and never uses `ZipStreamReader`.
- **Decision:** fix in code, not by ignoring. b7a5382 moves zip to 9.0.5 (the smallest fixed release; the manifest now asks for it, so a fresh resolve cannot pick a vulnerable 9.0.x). 734b467 replaces chrono with `time` 0.3.47, which was already in the tree. `deny.toml` is unchanged and no advisory is ignored. New transitive crates: nix 0.31.4 (MIT) and num_threads 0.1.8 (MIT OR Apache-2.0, Unix only). chrono stays in `Cargo.lock` only as an optional transitive dependency, which the workspace scope does not flag.
- **Result:** cargo-deny `check licenses advisories bans` passes at 97cc21b (verified by the finisher and both verifiers). No sockets, no new network path; Q6 otherwise changed no security boundary.
- **Note:** the advisory gate can turn red with no code change (R13). Q7's CI runs will see the current database.

## 2026-11-11 — Q2 security review of the merged base (Phase 5)

Full review: `reviews/2026-11-11-security-review.md`; tickets: `reviews/2026-11-11-q2-tickets.md`.

### Decisions
- **Fix-now bar** (user): Critical and High fixed now with tests, even when behaviour changes; Medium and Low are tickets. Voxxa has no released users, so each behaviour change is recorded rather than avoided.
- **Fixed:**
  - S1 / Q2-13 (High): local HTTP API. Token mandatory (generated 32-hex when blank, at least 16 characters), one `authorize` guard on every route: any `Origin` → 403, `Host` other than `127.0.0.1:<port>`/`localhost:<port>` → 403, constant-time Bearer check (scheme case-insensitive). Tailscale Funnel advice removed from INTEGRATIONS.md.
  - S3 / Q2-01 (Critical): untrusted-file parsers. `catch_unwind` around pdf-extract; imports run in `spawn_blocking`.
  - S6 capability part / Q2-09: `capabilities/default.json` grants `core:event:default` only, to `main` and `stage`.
  - Q2-25: single-instance plugin (new dependency, Apache-2.0 OR MIT, cargo-deny clean).
- **Downgraded by the refutation pass and ticketed:** S2 / Q2-14 (open keep-alives survive Off; Medium, the per-request guard still runs) and S4 / Q2-15 (transcripts in the diagnostic report; Medium, a retention breach under user review, not an exploitable leak).
- **Outbound counter:** every `reqwest` `send()` is counted (17 sites). Not counted: ProPresenter WS frames after connect, redirect hops, mDNS, the unused updater client. Privacy copy wrong (Q2-32, Low).
- **Logging:** no token, PCO secret or presenter password at Info (canary-tested); transcripts at Info (Q2-15); trace level can log the Pro7 WS password via tungstenite (Q2-33, Low).
- **ADR-011:** stage window not audience-facing by default; S14 copy ticket (Q2-30).
- **Open for later increments:** CSP (Q2-26), model integrity (Q2-27), release pipeline pinning and updater (Q2-28, cut-list ADR), SQLite hardening (Q2-31), mDNS (Q2-34), PCO scope (Q2-35), token persistence (Q2-51).

### Security behaviour changes (Q2)
1. HTTP API always needs `Authorization: Bearer <token>`; blank field → generated token, shown once, not saved.
2. Requests with an `Origin` header or a non-loopback `Host` get 403; browser-based controllers stop working.
3. Lowercase `bearer` is accepted.
4. Both windows may listen to events; no other plugin API is granted.
5. Imports run off the UI thread; a malformed PDF returns an error.
6. A second launch focuses the running Voxxa.

## 2026-11-20 — Q9W (10x lane, `q9w/10x`): WorshipTools Presenter by process, keys guarded
ADR-019 §§ 1-4. Security Engineer review of the new surface.
- **Process enumeration is read-only.** `presenters/keystroke.rs:67-115` opens each process with `PROCESS_QUERY_LIMITED_INFORMATION` only, reads its image path with `QueryFullProcessImageNameW`, and closes the handle. No window is read, nothing is injected, no process is signalled. Processes the user may not query are skipped. Kernel32 FFI only: no new crate; `Cargo.toml`, `Cargo.lock`, `deny.toml` and `capabilities/default.json` are unchanged; cargo-deny passes.
- **What leaves the process list:** one fixed name ("WorshipTools Presenter") when an image path matches; the paths themselves are not logged, stored or sent to the webview.
- **Keys only to Presenter.** With this profile, `tap()` (`keystroke.rs:222`) checks the front window's process path before every key; a key aimed at any other window is not sent. Narrower than before. Off Windows the guard passes (ADR-019 amendment); the pilot is Windows.
- **Accepted risk: name match.** Any executable named `Presenter.exe` in front counts as Presenter (test card T1; R0 item 18). A wrong match can only receive Right or Left. The founder's M15 checks the real booth PC.
- **Accepted risk: Presenter run as administrator.** Windows (UIPI) drops the keys silently and Voxxa cannot tell; documented in the runbook and in M15. Voxxa never asks for elevation.
- **Key map:** F10, F11, F12, Up, Down and numpad +/- are never sent (unit test); "." is gone; the blank is off until the founder confirms the unblank key.
- **Guarded real-app run:** scratch identifier `com.voxxa.qa9wx` (checked in the binary; `com.voxxa.app` absent), CDP on 9361, no microphone (startup only builds `AudioEngine::new()`; capture opens only in `start_listening_with_state`), no key dispatch; Presenter's process was enumerated, never touched; scratch data deleted afterwards.

## 2026-11-20 — Q9V (10x lane, `q9v/10x`): the media guard and a numbers-only `--full`
ADR-019 §§ 7-9 and its Q9V amendment. Security Engineer review (Phase 5).
- **The repo refuses media, twice.** `src-tauri/tests/media_guard.rs` and the `media` job in `ci.yml` both read `git ls-files -z` (a non-ASCII path is not quoted), lower-case the path, and refuse `.mp4 .mov .m4v .mkv .webm .wav .flac .mp3 .m4a` and anything under `fixtures/private/` (also nested), naming the path. The allowlist is the 15 public-domain WAVs by exact path, so a case variant or the same name in another folder is refused. The test keeps the two lists in step and fails when an allowed file is no longer tracked. Proved on a scratch clone (never pushed, deleted) with three tiny dummy files: `X.WAV`, `Músicas de adoración/canción.mp4` (four zero bytes each) and an empty `fixtures/private/see-a-victory/dummy.txt`: both checks failed and named all three.
- **`--full` prints numbers only.** `voxxad`'s stderr (Whisper's log) goes to null; a failure names the exit status, never stdout; error lines, bad `voxxad` lines and bad truth lines are named by number; a bad setlist by line and column (serde's "invalid type" message quotes the value). Proved with a fake `voxxad` and placeholder words (`tests/eval_numbers_only.rs`). Everything `voxxad` hears stays in `voxxa-eval`'s memory and in the session log in a temporary folder that is deleted when the run ends.
- **The founder's tools write nothing else.** The setlist snippet prints only a slide count and writes `setlist.json` in the private folder without a BOM; `tap-truth.ps1` plays the WAV (no microphone), writes only `truth.jsonl` and refuses to overwrite it. Neither touches the network. Agents ran both only in a scratch folder outside the repo, with placeholder words and a silent WAV.
- **No new dependency, no capability change.** `Cargo.toml` gains only a `[[test]]` entry (`harness = false`, `required-features = ["cli"]`); `Cargo.lock`, `deny.toml` and `capabilities/` are unchanged; cargo-deny passes. The new CI job has `contents: read` (workflow default) and runs no third-party action besides `actions/checkout@v4`, like the other jobs.
- **Accepted risks (tickets proposed).**
  - The guard matches names, not content: audio renamed to `.bin`, or a type outside the nine (`.ogg .opus .aac .wma .aiff .avi .wmv`), passes. The nine are the founder's list in ADR-019 § 8.
  - Lyrics copied out of `fixtures/private/` (for example a `setlist.json` into `fixtures/recordings/`) pass both checks; review and ADR-007's title test are the only guard.
  - A PR can edit `ci.yml` and the test together; the founder should make "media guard" a required status check on `main`.
  - If `voxxa-eval --full` is killed, its temporary session log (every word heard) stays in `%TEMP%`.
  - `--record` on a private fixture saves that log as `session.jsonl` in the private folder (ignored by git and refused by the guard); the README says not to use it there.
- **Safety in this run:** no agent searched for, opened, copied, hashed or converted the .mp4 or any church media; nothing was read under `%APPDATA%\Presenter`; Presenter was not touched; no microphone; no GUI app was started; nothing went to a network service. `git log --all -- fixtures/private` is empty.

## 2026-11-23 — Q10 (10x lane, `q10/10x`): updates on request only, a strict CSP, the signing order
Details: ADR-020; `.10x/decisions/sde/church-autoadvance.md` (Q10 lane); `docs/RELEASING.md` § 6.
- **Updater threat model.** Trust chain: TLS to github.com → `latest.json` (unsigned) → artifact verified by minisign against the committed pubkey **84F2CD98C608E9C9**. Guards: (1) only the operator starts a check, never at startup or on a timer, never while listening (`updater::refused`, before any network call, and again after the download); (2) `should_update` refuses equal, lower, build-metadata-only and malformed versions, and is both the plugin's default comparator and the instance comparator, so no path uses the built-in check; `tests/release_config.rs` forbids `allowDowngrades`, every `dangerous*` flag and non-https endpoints; (3) install takes only the exact version the operator confirmed after a fresh check. **Residual:** the endpoint's `version` is not signed, so whoever can write the repository's releases could pair a higher number with an older, genuinely signed artifact. `requireSignedVersion` closes this, but the pinned CLI 2.10.1 writes no signed version (verified in the binary); turning it on now would refuse every update. Ticket.
- **No webview updater permission.** `capabilities/default.json` is unchanged (`core:event:default` only; `tests/capabilities.rs`). The plugin's JS commands stay unreachable. A script injected into the page could still call Voxxa's own `check_for_update` / `install_update` (app commands are open to the app's windows), which reaches only GitHub, installs only a newer validly signed release, and is refused while listening. Accepted.
- **CSP** (`tauri.conf.json:49`): `default-src 'self'`; scripts, styles, images and fonts `'self'` only; `connect-src ipc: http://ipc.localhost`; `object-src 'none'`; `base-uri 'self'`; `form-action 'none'`. No `unsafe-eval`, no `unsafe-inline` (the three inline style attributes were removed). Proven: zero violations in the guarded real app (main and stage windows, Simple mode, Call for help, Advanced) and in the mocked e2e, which serves the same header; a planted inline style is refused in both.
- **Signing order and identity.** Authenticode (SignPath, or Azure) before the minisign signatures, so the updater verifies the bytes users download. One Windows path per build, so one publisher identity. The release job refuses a commit not on `main`, a tag whose version differs from `tauri.conf.json`, and any published release (`docs/signing-policy.md` updated to match: drafts, `main` only, rc tags). The updater key reaches only the job steps that sign (not the Windows build step).
- **Privacy promise:** network only for a model download, a Planning Center import, a Hymnary.org lookup or an update check, all operator-started (README, release body, quick-start, signing policy, the privacy card). Each check and download is counted on the privacy card. SignPath's "option to disable update checks" TODO is resolved: there is no automatic check to disable.
- **Key custody:** `docs/RELEASING.md` § 6 (two offline holders; lost and leaked procedures). **Founder checklist:** confirm you hold the private key and password matching 84F2CD98C608E9C9, or generate a new pair yourself and replace the pubkey. Agents never generated, read or printed any key or secret.
- **Supply chain:** `wingetcreate` pinned by release (v1.9.14.0) and SHA-256; `trusted-signing-cli` 0.11.0 `--locked`; actions still pinned by tag (Q2-28). `semver` is a direct dependency now, already in the tree; cargo-deny passes.
- **Safety in this run:** no microphone; Presenter untouched; the real identifier `com.voxxa.app` never started (its data folder's time unchanged); the scratch app (`com.voxxa.qa10x`, PID 12580) was stopped by PID and its data deleted; Check for updates was never clicked in the real app (outbound counter 0 before and after); no installer was built or installed; no key or secret handled.

## 2026-11-23 — Q10 base (`q10/base`): grafts and review fixes
- **Supply chain (supersedes "actions still pinned by tag" above):** every action in `build.yml` and `winget.yml`, the jobs that hold the updater key, the SignPath, Azure and Apple secrets and a `contents: write` token, is pinned to a full commit SHA; `tests/release_config.rs` fails on any `uses:` without a 40-hex ref and a version comment. A moved tag can no longer change what signs an update. SignPath's v3.2 is three days old at pinning; bump deliberately, by SHA.
- **Least exposure of the updater key:** the latest.json job (which runs `gh` and a repository script) no longer receives `TAURI_SIGNING_PRIVATE_KEY`; it reads only the Windows job's `updater` output (`yes`/`no`). The key now reaches four steps of `build.yml`: on macOS/Linux the "Tauri config override" step (tests it for emptiness) and tauri-action (signs); on Windows "Which signing path" (tests it for emptiness) and "Updater signatures over the final installers" (`tauri signer sign`). All four run pinned code or inline scripts.
- **Azure partial setup:** chosen only when all six values exist, so a half-configured Azure never signs with an empty endpoint or account.
- **Updater wording:** `ReleaseNotFound` (any non-success HTTP status in plugin 2.14.1) is never reported as "up to date"; nothing about the trust chain changed.
- **Safety in this run:** no microphone; Presenter untouched (it was running; the first-run guide was skipped at step 1, 1.2 s after start, so no discovery and no presenter connection happened); `com.voxxa.app` never started (data folder time unchanged); the scratch app `com.voxxa.qa10f` (PID 9132) stopped by PID, its data deleted; Check for updates never clicked (outbound 0 before and after); no installer built or installed; no key or secret generated, read or printed (the signing-path block was run with fake values).
