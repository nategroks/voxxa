# Security — church-autoadvance

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
