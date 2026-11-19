# security — Index

Decisions per feature, newest first inside each file.

## Active features
- `church-autoadvance`
  - 2026-11-19 **Q7** security review of the CLI surface and the full-eval workflow: no Critical/High; `tests/replay_no_sockets.rs` proves `voxxad --replay` opens no socket (strace in `unshare -rn` on Linux, ETW on Windows; mutation-tested; macOS skipped and says so); `voxxa-eval.exe` was in the Windows installer (Medium, fixed b9692c5); `ci.yml` now `permissions: contents: read`; full-eval workflow: SHA-256-pinned model, SHA-pinned actions, read-only token. Review: `reviews/2026-11-19-security-review.md`. Final at 6386f6b (both verifiers passed); added from the ponytail audit: Q7-08 (Medium, transcripts in the diagnostics report), Q7-09, Q7-11.
  - 2026-11-18 **Q6 close-out**: RUSTSEC-2026-0543 (chrono) and RUSTSEC-2026-0548 (zip 9.0.0) cleared in code (zip 9.0.5; chrono replaced by `time`), no advisory ignored; cargo-deny green at 97cc21b.
  - 2026-11-11 **Q2** security review: STRIDE-lite on 10 boundaries; S1 HTTP API (High) and S3 PDF abort (Critical) fixed with tests; capability file least-privilege; S2 and S4 downgraded to Medium tickets; outbound counter covers every reqwest send; no credentials logged at Info. Review: `reviews/2026-11-11-security-review.md`.
