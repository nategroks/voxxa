# Security — church-autoadvance

## 2026-11-19 — Q7 final (`q7/base` @ 6386f6b; both verifiers passed)
Details: `reviews/2026-11-19-security-review.md` §11; the ponytail audit: `reviews/2026-11-19-ponytail-q7.md` §3.
- **Both verifiers passed at 6386f6b.** A TCP-connect mutant and a UDP-socket mutant in the replay branch each fail the zero-sockets test on Windows (ETW); the model hash and every action pin were re-checked; product code changed only by the `src/bin` → `src/cli` rename and one error string.
- **Decision: no Critical or High finding is open; the security position is final.** Ready to merge once the PR's first CI run shows the Linux and Windows socket tests green and the Linux mutant red then green.
- **Added from the ponytail audit:** Q7-08 (Medium, privacy): the GUI logs each transcript at Info, and the diagnostics report copies it. Fix before any pilot leaves shadow mode. Q7-09 (Low): `voxxad --device` opens the microphone before checking its inputs. Q7-11 (Low): `voxxa-eval`'s "voxxad failed" error carries recognised words.

## 2026-11-19 — Q7 verifier fixes on `q7/base` (aa16e90)
Full text: `reviews/2026-11-19-security-review.md` section 10.
- **Decision: on Windows, a replay is charged only with its own sockets, during its own lifetime.** The ETW session adds `Microsoft-Windows-Kernel-Process` (start and exit), and only socket (1000) and connect (1017, 1018, 1020, 1021) events count. Before, any AFD event with a matching pid counted, and gate 4 was flaky (a reused pid or a borrowed context). A child without a recorded start or exit fails the test.
- **Decision: CI's Linux job requires a network namespace** (`VOXXA_REQUIRE_NETNS=1`). A runner that refuses `unshare -rn` now fails the test instead of passing with networking enabled. A green job proves which tracer ran.
- **Re-proof:** 4 of 4 full `cargo test --features cli` runs pass. The replay-branch TCP-connect mutant fails with 4 socket/connect events.

## 2026-11-19 — Q7 finish on `q7/base`
Full text: `reviews/2026-11-19-security-review.md` section 9.
- **Decision: the network claim is "`--replay` opens no socket; `--shadow` makes no network connection"** (on Linux/macOS tokio's signal driver creates one local AF_UNIX socketpair, `src/cli/voxxad/main.rs:140`). "`--shadow` opens no socket" was too strong; fixed in the release notes, INTEGRATIONS, the review and Q7-07 (b0efc3f). A future strace check of `--shadow` must allow `socketpair(AF_UNIX)` or count only AF_INET/AF_INET6 sockets and connects.
- **Decision: Linux CI stays on `ubuntu-22.04`** until `unshare -rn` is shown to work on a newer image (Ubuntu 24.04's AppArmor default refuses it, and the test would fall back to strace with networking enabled). Comment in `ci.yml` (5fa4c76).
- **Re-proof at the finish (Windows, ETW):** pass; the replay-branch TCP connect mutant fails with 11 AFD events; pass again after the revert.
- **Before merge (must-fix evidence, not yet run):** in the first CI run, (a) Linux and Windows print `traced with strace inside unshare -rn` and `traced with ETW`; (b) the throwaway mutant commit turns Linux red and its revert green; (c) full-eval green or warning on three OSes with only the table in the summary and artifact. Record the run URLs in both reports.

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
