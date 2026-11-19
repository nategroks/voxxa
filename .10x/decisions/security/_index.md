# security — Index

Decisions per feature, newest first inside each file.

## Active features
- `church-autoadvance`
  - 2026-11-18 **Q6 close-out**: RUSTSEC-2026-0543 (chrono) and RUSTSEC-2026-0548 (zip 9.0.0) cleared in code (zip 9.0.5; chrono replaced by `time`), no advisory ignored; cargo-deny green at 97cc21b.
  - 2026-11-11 **Q2** security review: STRIDE-lite on 10 boundaries; S1 HTTP API (High) and S3 PDF abort (Critical) fixed with tests; capability file least-privilege; S2 and S4 downgraded to Medium tickets; outbound counter covers every reqwest send; no credentials logged at Info. Review: `reviews/2026-11-11-security-review.md`.
