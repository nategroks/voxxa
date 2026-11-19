# Q7 security review: the CLI surface and the full-eval workflow (2026-11-19)

**Role:** Security Engineer, Phase 5 (Verification), Q7, the close of Increment 1′.
**Branch:** `q7/10x`, from 46cb6d1 (`main` after the Q6 merge).
**Scope:** `voxxad` and `voxxa-eval` (`src-tauri/src/cli/`, moved from `src/bin/` in b9692c5), and the new `.github/workflows/full-eval.yml`. `ci.yml` was reviewed where Q7 changed it.
**Fix-now bar:** as in Q2. Critical and High are fixed now with tests; Medium and Low become tickets.
**Companion files:** `2026-11-19-qa-report.md` (§6 has the Q7 tickets) and `decisions/security/church-autoadvance.md`.

## 1. Verdict

- **No Critical or High finding.** The CLI cannot reach the network in `--replay` or `--shadow`: `--replay` opens no socket; `--shadow` makes no network connection (on Linux/macOS tokio's signal driver creates one local AF_UNIX socketpair). The Windows ETW run of `--shadow` (§4.3) saw no socket at all; Windows has no such socketpair. It writes only its own session logs and, for `voxxa-eval`, fixture files inside the repository.
- **The zero-sockets requirement is now a test:** `src-tauri/tests/replay_no_sockets.rs` (2817556). It runs on Linux (strace inside `unshare -rn`, networking disabled) and on Windows (ETW Winsock-AFD) in every CI run. A mutation proves it: a TCP connect added to replay fails it. On macOS it is skipped, and the test says so (§4).
- **A packaging finding that was already there, now fixed (Medium; Q5-03 #75):** the Windows installer built from 46cb6d1 contained `voxxa-eval.exe`, an unsigned developer tool. Fixed in b9692c5 and proved with the generated NSIS script (`decisions/devops/church-autoadvance.md`).
- **Full-eval workflow:** the model is pinned by SHA-256 and checked on every run (cached copies too). Every action is pinned to a commit SHA. `permissions: contents: read`, no secrets, `persist-credentials: false`, and the `pull_request` trigger (not `pull_request_target`). A fork PR runs it with a read-only token and no secrets.
- **One CI hardening fix now:** `ci.yml` had no `permissions:` block, so its token had the repository's default permissions. It is now `contents: read`. Its actions are still pinned by tag, `dtolnay/rust-toolchain@master` included (ticket Q7-03).

## 2. Method

- **Code reading** of `src-tauri/src/cli/voxxad/{main,cli,events}.rs` and `src-tauri/src/cli/voxxa-eval/{main,audio,score}.rs`, of the paths they open, create and delete, and of every place a socket could come from: `connect_presenter`, the tokio runtime, `reqwest` (through presenters only) and model loading.
- **Dynamic, in tests:** `replay_no_sockets` on Windows with ETW. Linux runs in CI (strace); this PC has no Linux.
- **Dynamic, local only:** a full `voxxad --stdin-audio … --shadow --presenter propresenter7_rest --host 127.0.0.1 --port 9 --model tiny` run under ETW on Windows, with a `curl` positive control in the same trace (§4.3).
- **Workflow review:** triggers, permissions, action pins (resolved with `gh api`, read-only), the model URL and its hash (the Hugging Face LFS pointer read over HTTPS), cache keys, and what reaches the CI log.
- **Safety rules kept:** no microphone (never `--device`), no GUI, no keystroke presenter. The positive control is this test binary connecting to 127.0.0.1:9, not a `voxxad` with a presenter. Every process was stopped. Full mode's stderr went to `/dev/null`.

## 3. STRIDE-lite: the CLI surface

| Boundary | Spoofing | Tampering | Repudiation | Information disclosure | Denial of service | Elevation | Status |
|---|---|---|---|---|---|---|---|
| (k) Files `voxxad` reads: `--setlist` JSON, `--stdin-audio` WAV (or stdin), `--replay` log | a log or setlist is not authenticated: anyone who can write the file decides what replay prints | a crafted log sets any `SmartConfig` (the start line's `config`), and a crafted transcript changes replay's actions. Replay **only prints**: it builds no presenter, so it cannot move a screen. The setlist fingerprint check stops a log being replayed against another setlist (`events.rs:211`) | — | — | whole files are read into memory (`main.rs:200`, `events.rs:194`); a huge local file costs memory. The WAV parser bounds-checks every chunk (`get(..)`) | none: no path from the file content to code or to the network | accepted (local inputs, the user's own files) |
| (l) Files `voxxad` writes: session logs | — | log names are `session-YYYYMMDD-HHMM[-N].jsonl`, opened with `create_new` (no overwrite, `main.rs:312-341`) | each run's log holds every input and action, so replay reproduces it (ADR-013) | **full transcripts, prayers and names, for 14 days**, in the roaming `%APPDATA%` on Windows (Q3-05, #49; now in the release notes and INTEGRATIONS); default file permissions (on Unix, umask-readable to other local users: Q7-05) | expiry deletes only names that parse as `session-<date>….jsonl` older than 14 days, in the log folder only (`events.rs:247`). With `--log-dir` pointed at a shared folder it would delete other tools' files with that name pattern (documented) | — | docs done; #49 stays open (`data_local_dir` decision); Q7-05 |
| (m) Network from `voxxad` | presenters authenticate as in Q2 boundary (b) | — | — | — | — | — | **replay: no socket (CI test, Linux and Windows); shadow: no network connection (local ETW run, Windows: no socket at all; on Linux/macOS tokio's signal driver creates one local AF_UNIX socketpair, `main.rs:140` `tokio::signal::ctrl_c`)**. Without `--shadow`, only the chosen network presenter connects (`connect_presenter`, `main.rs:266-292`), and `none` (the default) builds nothing |
| (n) Keystroke presenter | — | types into **whatever window has focus**, including the terminal that started `voxxad` (Q3-04, #48) | — | — | — | a keystroke in a terminal can run a command | a stderr warning exists; the docs now say "focus the presentation app" (INTEGRATIONS, release notes). The code guard stays open (#48) |
| (o) stdout and stderr | — | — | — | stdout carries every `hypothesis` (the transcript) to whatever reads the pipe; Whisper prints recognised words to stderr (Q2-33, #20) | — | — | documented; the full-eval workflow drops stderr |
| (p) `voxxa-eval` | — | `--full` rewrites `<fixture>/audio.wav` (git-ignored); `--record` overwrites `<fixture>/session.jsonl`, both only in the folders given or `fixtures/recordings` | — | session logs go to a temporary folder (`tempfile`), deleted after the run | — | runs the `voxxad` sitting beside it (`current_exe`); whoever can replace that file already owns the build folder | accepted |
| (q) Packaging | — | — | — | — | — | **an unsigned `voxxa-eval.exe` was installed beside the app (46cb6d1)** | **fixed** (b9692c5): out of all installers, `tests/packaging.rs` |

## 4. The zero-sockets test

### 4.1 What it does
`replay_opens_no_socket` builds one list of runs: a positive control (this test binary re-run on its `#[ignore]`d `socket_probe`, which connects to 127.0.0.1:9), then `voxxad --replay` on each of the six fixture logs plus the `voxxad_events` test log. It runs them all under the OS tracer and asserts:
1. the tracer saw at least one event from the control, so a tracer that sees nothing cannot pass;
2. every replay succeeded and printed actions, so it really replayed;
3. zero network events from every replay.

| OS | Tracer | Networking | Covers | In CI |
|---|---|---|---|---|
| Linux | `strace -f -qq -e trace=%network -e signal=none` (every network system call of the process and its threads) | **disabled** with `unshare -rn` when user namespaces allow it (only a down loopback); the test prints which | any `socket`, `connect`, `bind`, `sendto`, `socketpair`, … | ubuntu-22.04, `VOXXA_REQUIRE_SOCKET_TRACE=1`, `strace` installed by ci.yml |
| Windows | ETW session on `Microsoft-Windows-Winsock-AFD`, all keywords, read back with `Get-WinEvent` and grouped by process id | enabled | every socket the kernel's AFD driver creates, connects or cleans up for that process (event 1000 = create) | windows-latest (administrator), `VOXXA_REQUIRE_SOCKET_TRACE=1` |
| macOS | none: `dtruss` and DTrace need SIP disabled | — | **nothing at run time.** The replay code is the same Rust on every OS: `events::replay` and the replay branch of `main.rs` have no `cfg` | skipped, with a message |

**The Linux runner stays on `ubuntu-22.04` on purpose** (comment in `ci.yml`'s matrix). Ubuntu 24.04's AppArmor default (`kernel.apparmor_restrict_unprivileged_userns=1`) refuses unprivileged `unshare -rn`, so on `ubuntu-latest` the test would fall back to strace with networking enabled: still green, still tracing every socket call, but the "networking disabled" claim of the release notes would quietly stop being true. After any runner bump, check that the job log still says `traced with strace inside unshare -rn`.

Not covered anywhere: a socket made by another process that `voxxad` started (it starts none), and non-socket channels (named pipes, files). On Linux, strace sees system calls, so a socket opened by any library is caught. On Windows, ETW sees AFD sockets: Winsock, which is what Rust's `std::net`, tokio/mio and `reqwest` use.

### 4.2 Proof by mutation (Windows, ETW)
- **Unmutated:** passes, with the control seen (`replay_opens_no_socket: traced with ETW Microsoft-Windows-Winsock-AFD`). It takes 57 s in debug.
- **Mutant:** `let _ = std::net::TcpStream::connect("127.0.0.1:9");` added as the first statement of the replay branch (`src/cli/voxxad/main.rs`, `if let Some(path) = &args.replay {`). Result: **FAILED**, `jump-be-thou-my-vision: voxxad --replay touched the network under ETW …: ["11 Winsock-AFD events for pid 4212"]`. The connect's error is ignored, so only the tracer could catch this mutant. Reverted; never committed.
- **Linux:** the same mutant is not run here (no Linux on this PC, no WSL). The positive control proves that strace sees a socket from a traced process on every CI run. If CI's ubuntu runner cannot run strace, the test fails (not skips) because of `VOXXA_REQUIRE_SOCKET_TRACE=1`.

### 4.3 Shadow mode (local, Windows, not in CI)
`voxxad --stdin-audio fixtures/recordings/prayer-without-pause/audio.wav --setlist … --shadow --presenter propresenter7_rest --host 127.0.0.1 --port 9 --model tiny --log-dir <temp>` ran under ETW, with a `curl` control in the same trace. Result: **0 Winsock-AFD events from `voxxad`** (the control had 29; the trace held 1,604 events in all). The run was complete: a start line with `"shadow": true`, 12 hypothesis lines, 3 actions, and a stop line with reason `end_of_audio`. So a shadow run with a network presenter named on the command line made no socket, from start to the end of the audio, model load and Whisper included. A shadow run needs the Whisper model, which CI's PR gate does not have, so this is local evidence only (ticket Q7-07).

## 5. The full-eval workflow's supply chain

| Item | Finding |
|---|---|
| Trigger | `workflow_dispatch` plus `pull_request` with `paths: [.github/workflows/full-eval.yml]`. It is `pull_request`, not `pull_request_target`, so a fork's PR gets a read-only token and no secrets. The workflow uses no secrets. |
| Token | `permissions: contents: read`; `actions/checkout` with `persist-credentials: false`, so later steps (cargo build scripts, npm) cannot reuse the token. |
| Action pins | checkout 11bd719 (v4.2.2), rust-toolchain 683b588 (master on 2026-10-16), rust-cache 9d47c6a (v2.7.8; resolved from the annotated tag object), setup-node 49933ea (v4.4.0), cache 5a3ec84 (v4.2.3), upload-artifact ea165f8 (v4.6.2). Resolved with `gh api …/git/ref/tags/…`. |
| Model | URL `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin` (the app's, and a moving `main`). Pinned SHA-256 `be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21`, which equals the LFS pointer's oid (size 77,691,713) and the reference PC's copy. The hash is checked after download **and after a cache restore**; a mismatch deletes the file and fails the job. `curl --proto '=https' --tlsv1.2 -fL`. |
| Cache | Key `whisper-ggml-tiny-<OS>-<sha256>`. A PR can only write caches in its own scope, and a poisoned model fails the hash check anyway. `rust-cache` (the target folder) is the usual CI trust level, the same as ci.yml. |
| Logs | stderr (Whisper's recognised words) is dropped; only `voxxa-eval:` lines, cut to 300 characters, are shown. The table holds counts and timings only. |
| Failure semantics | Exit 1 (a metric differs) is a warning; any other exit fails. A change in Whisper's output cannot turn the job red, but a broken build, download, checksum or run does. |
| npm | `npm ci` runs the lockfile's packages, as in ci.yml. No new dependency. |

Not covered: the toolchain that `dtolnay/rust-toolchain` installs comes from rustup's channel (signed by the Rust project). The rustup installer on hosted runners is the image's.

## 6. Findings and triage

| ID | Severity | Finding | Decision |
|---|---|---|---|
| Q7-S1 | Medium | `voxxa-eval.exe` shipped in the Windows installer (46cb6d1); which tool ships depended on directory order | **fixed now**, b9692c5 + `tests/packaging.rs` (fails if a tool moves back to `src/bin/`, loses `required-features`, or `cli` becomes a default feature) |
| Q7-S2 | Low | `ci.yml` had no `permissions:` block | **fixed now**: `contents: read` (this branch) |
| Q7-S3 | Low | `ci.yml` actions pinned by tag; `dtolnay/rust-toolchain@master` is a moving branch | ticket Q7-03 |
| Q7-S4 | Low | No run-time socket check on macOS; the shadow-mode check is local only | tickets Q7-01, Q7-07 |
| Q7-S5 | Low | Session logs are created with default permissions; on a shared Unix machine other users may read transcripts | ticket Q7-05 |
| — | Medium (open since Q3) | keystroke presenter has no focus guard in `voxxad` | docs done in Q7; code stays on #48 |
| — | Medium (open since Q3) | 14 days of transcripts in the roaming profile | docs done in Q7; the default-folder decision stays on #49 |
| — | accepted | a crafted session log can make replay print any actions | replay never dispatches; documented here |

## 7. Not tested
- The Linux and macOS runs of the zero-sockets test (CI only). The Linux mutant was not run.
- `voxxad --device` (safety rule: no microphone), and any non-shadow presenter run.
- The full-eval workflow on GitHub (its first run is on the Q7 PR).
- MSI, deb, AppImage and `.app` bundles (NSIS only; they share the binary list).

## 8. For the founder's checklist (adds to the Q2 list)
- Before a pilot runs `voxxad` live: `--log-dir` set to a local folder, `--shadow` for a rehearsal, and a network presenter, not keystroke.

## 9. Finish on `q7/base` (2026-11-19)

`q7/10x` was merged into `q7/base` (008aa50). The security-relevant changes after the merge:

- **Wording (10x review #1, b0efc3f).** "`--shadow` opens no socket" was too strong. On Linux and macOS, `tokio::signal::ctrl_c` (`src/cli/voxxad/main.rs:140`) makes tokio's signal driver create one local AF_UNIX socketpair. The claim is now: **`--replay` opens no socket; `--shadow` makes no network connection** (on Linux/macOS tokio's signal driver creates one local AF_UNIX socketpair). This is now in §1, row (m), the release notes, INTEGRATIONS and Q7-07. The Windows ETW run in §4.3 saw no socket at all, and it stays as recorded. A future strace check of `--shadow` must allow `socketpair(AF_UNIX)`, or count only AF_INET/AF_INET6 `socket` calls and `connect` calls (Q7-07).
- **Runner pin (5fa4c76).** `ci.yml` now says why Linux stays on `ubuntu-22.04` (§4.1).
- **Shared log list (3472540).** The socket test and the determinism test now read one list (`tests/common/mod.rs`), so a new log cannot be covered by one test and missed by the other.
- **Loud failure without the feature (c9271f0).** A plain `cargo test` now fails `replay_opens_no_socket` with "run: cargo test --features cli". Before, it skipped the test without a word. The check is on `cfg!(feature = "cli")`, not `option_env!`, so a stale `voxxad` from an earlier build is never traced in place of the current one.

**Re-proof at the finish (Windows, ETW, administrator).**

| Run | Result |
|---|---|
| Unmutated | pass: `traced with ETW Microsoft-Windows-Winsock-AFD`, and the probe made 18 events (45 s) |
| Mutant: `let _ = std::net::TcpStream::connect("127.0.0.1:9");` as the first line of the replay branch | **FAILED**: `jump-be-thou-my-vision: voxxad --replay touched the network …`, with 11 Winsock-AFD events for that pid |
| Reverted | pass (45 s) |

The mutant was never committed.

**Before merge, the PR's first CI run must show** the following. Ponytail's review raised the same must-fix items against its own lane, and they apply equally to these paths, which have never run.

- **(a)** The `ubuntu-22.04` and `windows-latest` jobs are green. Their `replay_no_sockets` output prints `traced with strace inside unshare -rn` and `traced with ETW`.
- **(b)** A throwaway commit with the mutant above turns the Linux job red. Reverting it turns the job green again.
- **(c)** `full-eval.yml` is green, or only warns, on all three OSes. The job summary and the artifact hold only the table. The run time per OS is recorded.
- **Node 20.** Check that first full-eval run for Node 20 deprecation warnings on the older pins: checkout v4.2.2, rust-cache v2.7.8, cache v4.2.3, setup-node v4.4.0 and upload-artifact v4.6.2. If any warn, bump them, SHA-pinned (ponytail nit 7).
- **Run URLs:** not yet. This step may not push. The orchestrator records them here and in the QA report §9.

The verdict in §1 is unchanged: no Critical or High finding.
