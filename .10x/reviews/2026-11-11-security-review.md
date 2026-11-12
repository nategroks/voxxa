# Q2 security review: the merged base (2026-11-11)

**Role:** Security Engineer, Phase 5 (Verification). **Base reviewed:** `claude/trusting-newton-v71ebu` @ 1568810. **Fixes:** branch `q2/10x` up to b09373d. **Fix-now bar (user, 2026-11-11):** Critical and High are fixed now with tests, even when behaviour changes; Medium and Low become tickets. **Companion files:** `2026-11-11-qa-report.md`, `2026-11-11-q2-tickets.md`.

## 1. Verdict

- Three findings met the fix-now bar after adjudication and are **fixed with tests**: the HTTP API was open to any web page (S1, High), a malformed PDF aborted the app (S3, High; QA rated it Critical), and the missing capability file broke every event (S6 capability part, fixed as QA item Q2-09).
- Two findings the security lane first rated High were **downgraded to Medium by the refutation pass and ticketed** (S2 open keep-alive connections; S4 transcripts in the diagnostic report). §5 gives the reasons.
- No Critical security finding remains open. Eleven security tickets remain (Medium and Low).
- **Outbound counter:** every `reqwest` client is counted on every `send()`. WebSocket frames, redirect hops, mDNS and the unused updater client are not (Low ticket).
- **Tokens:** no token, Planning Center secret or presenter password reaches the ring-buffer logger at the default level (tested with canaries). At `RUST_LOG=trace` a third-party crate can log the ProPresenter WS password (Low ticket). Transcripts are logged at Info (Medium ticket).

## 2. Method

- **Isolated runtime.** 1568810 exported with `git archive` to a scratch folder; the only harness change was `additionalBrowserArgs: --remote-debugging-port=9339` in the copy's `tauri.conf.json`. Built with `--features tauri/custom-protocol`, launched with its own `WEBVIEW2_USER_DATA_FOLDER`, driven only through CDP `Runtime.evaluate`.
- **Safety.** Before any dispatch the presenter was FreeShow or OpenLP pointed at a 127.0.0.1 mock and confirmed with `get_presenter_info`. No microphone, no keystrokes, no `/listen/start`, no Planning Center call with real credentials.
- **Temporary unit tests** (not committed): PDF mutation fuzz (3,000), text-parser fuzz (16,000 plus 6,000 RTF), billion laughs, pptx amplification, EasyWorship SQLite defaults and a generated-column attempt, transcript to diagnostic report.
- **Code reading:** every network client, all 70 `log::` sites, every `innerHTML` sink, `tauri.conf.json`, `build.yml`, `SIGNING.md`, and the relevant dependency sources (tauri 2.12.5 IPC ACL, axum 0.7 serve, tungstenite frame logging, reqwest redirects, mdns-sd).
- **After the fixes:** the HTTP API was re-tested live on a scratch build of `q2/10x` with raw sockets (§4, S1), and the adversarial verifier re-ran every gate and mutated each fix at the head.

## 3. STRIDE-lite by trust boundary

The strategy's two boundaries are (a) and (b). (c)–(j) are the boundaries the May code adds. "After Q2" shows what changed.

| Boundary | Spoofing | Tampering | Repudiation | Information disclosure | Denial of service | Elevation | After Q2 |
|---|---|---|---|---|---|---|---|
| (a) Microphone → transcripts | anyone in the room can steer detection (inherent; "a bad blank beats a wrong slide") | played audio can switch songs | — | transcripts logged at Info into the 500-line ring and the diagnostic report (S4); the Stage Display shows the "heard" line (S14); audio never touches disk (the only `File::create` is the model) | unbounded audio channel (audit 13); a download held the transcription lock (audit 6) | mic could be opened by any web page through the HTTP API (S1) | S1 fixed; audit 6 fixed (Q2-08); audit 13 fixed by the 50-song cap (Q2-17); S4 ticket Q2-15; S14 ticket Q2-30 |
| (b) Presenter channels (Keystroke, Pro7 REST/WS, FreeShow, OpenLP, OpenSong) | mDNS answers are unauthenticated (S12) | keystrokes went to whichever window had focus, including Voxxa's own (audit 3) | — | Basic auth and WS passwords travel the LAN in cleartext (inherent to those protocols); system proxy applies to LAN clients (S11); trace-level WS password logging (S11) | 3 s timeouts; unbounded response bodies (low) | — | audit 3 fixed (Q2-04: keys refused while Voxxa has focus); S11, S12 tickets; reqwest drops `Authorization` on cross-host redirects |
| (c) HTTP API on 127.0.0.1 | **no token by default; DNS rebinding worked (S1)** | **CSRF by simple POST: blank, next, listen/start (S1)** | only "blank"/"next" logged, no client info | `/state` readable through rebinding | no connection limit; any local user | open connections kept the old (possibly tokenless) config after Off or a token change (S2) | **S1 fixed (Q2-13)**; S2 ticket Q2-14 |
| (d) Hymnary and Planning Center outbound | rustls with native roots | response text becomes setlist text, rendered safely | — | Hymnary receives the query and a UA naming the repo; the PCO PAT lives in renderer memory only, never persisted or logged | unbounded bodies; 10/15 s timeouts | the PCO PAT grants the whole account (S13) | S13 ticket Q2-35; Hymnary returns 403 (Q2-29) |
| (e) mDNS Auto-detect | spoofable service entries (S12) | — | — | queries reveal "a ProPresenter client" on the LAN | 2.5 s browse, on click only | — | ticket Q2-34; not run (firewall prompt) |
| (f) EasyWorship SQLite (untrusted file) | — | — | — | `-shm`/`-wal` leftovers in %TEMP% (S9) | no size cap; ran on the UI thread | none found: identifiers allow-listed, read-only, no ATTACH, generated-column trick blocked; but `trusted_schema=1` and DEFENSIVE off | UI-thread part fixed (Q2-01: imports run in `spawn_blocking`); S9 ticket Q2-31 |
| (g) pptx / PDF / .pro7 / OpenLyrics / ChordPro parsers (untrusted files) | — | — | — | — | **PDF panic aborted the app (S3)**; pptx 1,025× zip amplification (S5) | none; XML has no entity expansion (quick-xml predefined only, tested); text parsers 0 panics in 22,000 mutations | **S3 fixed (Q2-01)**; S5 ticket Q2-21; stack overflow inside pdf-extract is not catchable (noted in Q2-21) |
| (h) Webview | — | inline script runs (CSP null) | — | — | — | injected script would reach all app commands from either window (S6); plugin IPC denied, which also broke `listen()` | capability file added with least privilege (Q2-09); CSP and `withGlobalTauri` ticket Q2-26; remote origins still blocked by tauri 2.12.5 |
| (i) Updater and signing | updater manifest path unused | release secrets handed to tag-pinned actions (S8) | — | — | no channel for security updates (updater never called) | — | ticket Q2-28 |
| (j) Logger and diagnostics | — | — | operations are logged | transcripts (S4); third-party trace frames (S11); no credentials at Info (canary-tested) | 500-line ring, bounded | — | tickets Q2-15, Q2-33 |

## 4. Findings, triage and status

| # | Finding | Lane severity | Final severity | Q2 id | Status |
|---|---|---|---|---|---|
| S1 | HTTP API: token optional, no Origin or Host check; any web page can blank or advance the audience screen or start the mic (audit 10) | High | High | Q2-13 | **fixed** (4817656) |
| S2 | Off, or on with a new token, leaves open keep-alive connections served with the old config | High | Medium | Q2-14 | ticket |
| S3 | A malformed PDF aborts the app (audit 11) | High (QA: Critical) | Critical | Q2-01 | **fixed** (cb14775) |
| S4 | Transcripts logged at Info into the diagnostic report | High | Medium | Q2-15 | ticket |
| S5 | No import size caps; pptx 1,025× amplification | Medium | Medium | Q2-21 | ticket |
| S6 | CSP null, `withGlobalTauri`, no capability files | Medium | capability part fixed with Q2-09; CSP Medium | Q2-09 / Q2-26 | capability **fixed** (018de6d); CSP ticket |
| S7 | Model download: no checksum, mutable `resolve/main`, no timeout | Medium | Medium | Q2-27 (data-loss part Q2-12 fixed) | ticket |
| S8 | Release workflow: secrets to tag-pinned actions; updater unwired; `.asc` never released | Medium | Medium | Q2-28 | ticket |
| S9 | EasyWorship SQLite hardening | Low | Low | Q2-31 | ticket |
| S10 | Outbound counter misses WS frames, redirects, mDNS; privacy copy wrong | Low | Low | Q2-32 | ticket |
| S11 | Presenter clients honour the system proxy; `RUST_LOG=trace` logs the Pro7 WS password; whisper.cpp prints tokens to stderr | Low | Low | Q2-33 | ticket |
| S12 | mDNS: unauthenticated answers; binds every interface | Low | Low | Q2-34 | ticket |
| S13 | Planning Center PAT grants the whole account | Low | Low | Q2-35 | ticket |
| S14 | Stage window copy invites a second screen; shows transcripts (ADR-011) | Low | Low | Q2-30 | ticket |
| — | The generated HTTP token is not kept across launches; the API is not restored at launch | — | Low | Q2-51 | ticket (found in the cross-review) |
| — | No single-instance guard: two conductors drive one presenter | QA | High | Q2-25 | **fixed** (a9a16dd); listed here because it adds a dependency |

### S1 (High, fixed): the HTTP API was open to any web page
- **Code at 1568810:** `http_api/mod.rs` `check_token` returned `Ok` when no token was set; no Host, Origin or CORS handling; `commands/mod.rs` mapped `""` to `None`; `index.html` said "Leave blank for no auth"; `docs/INTEGRATIONS.md` recommended Tailscale **Funnel**, which publishes the port to the internet.
- **Ran it (no token):** `POST /api/v1/blank` with `Origin: https://evil.example` and `Content-Type: text/plain` → 204 and the mock received a clear; `GET /api/v1/state` with a rebinding `Host` → 200. With a token set, a rebinding Host still got 200.
- **Fix:** one guard, `authorize(headers, port, token)`, on every route: any `Origin` header → 403; `Host` other than `127.0.0.1:<port>` or `localhost:<port>` → 403; `Bearer` scheme in any case, constant-time comparison, else 401. `resolve_token`: a blank token generates 32 hex characters (getrandom 0.3, already in the lock file, now a direct dependency); tokens under 16 characters are refused. `start_http_api` returns the token; the GUI writes it into the field and shows it. The INTEGRATIONS.md security notes are rewritten, the Funnel advice is gone, and every example sends the token.
- **Tests:** `tests/http_api.rs` (generated and short tokens; missing, wrong, right and lowercase-`bearer` tokens; foreign Host and any Origin get 403 even with the right token). All three fail with the 1568810 semantics patched back in.
- **Live after the fix:** right token 200; no token 401; `Host: evil.example` 403; cross-origin POST /blank 403 with the mock untouched; token POST /blank 204.

### S3 (Critical, fixed): a malformed PDF aborted the app
- `parse_song_bytes` was a sync command, so it ran on the main thread, and `importers/pdf.rs` called pdf-extract with no guard. A 584-byte PDF with `/MediaBox [0 0 612]` gave `thread 'main' panicked at …lib.rs:2414 index out of bounds`, then `panic in a function that cannot unwind … aborting`. Fuzz: 144 of 3,000 mutations panic, at 11 sites.
- **Fix:** `catch_unwind` around pdf-extract (the release profile has no `panic = "abort"`), and the three import commands run in `spawn_blocking`.
- **Test:** `tests/importer_slides.rs::malformed_pdfs_return_an_error_instead_of_panicking`. Live: both PDFs return "this PDF could not be read" and the app stays alive.
- **Residual:** `catch_unwind` cannot catch a stack overflow or an abort inside pdf-extract; a deeply nested malicious PDF could still end the process (noted in ticket Q2-21).

### S6 capability part (fixed with Q2-09)
- `src-tauri/capabilities/default.json`: `windows: ["main", "stage"]`, `permissions: ["core:event:default"]` and nothing broader (QA proposed `core:default`; least privilege chosen because the GUI uses only `core.invoke` and `event.listen`). Test: `tests/capabilities.rs::both_windows_may_listen_to_events_and_nothing_more`.
- No XSS sink was found: every untrusted string goes through `textContent` or `escapeHtml`. Tauri 2.12.5 refuses custom commands from remote origins. CSP and `withGlobalTauri` stay a Medium ticket (Q2-26).

### Q2-25 single instance (fixed; dependency change)
- Added `tauri-plugin-single-instance` 2.5 (lock 2.5.5, Apache-2.0 OR MIT, +32 lock packages, mostly zbus for Linux D-Bus). cargo-deny passes. On Linux it needs a session bus at runtime; that path is not exercised by CI.

## 5. Adjudication of the two downgraded findings

- **S2 → Medium (Q2-14).** The mechanism is real: axum 0.7.9 spawns a task per connection with the router it was accepted with, and `stop()` aborts only the accept loop. But after S1, every request on an old connection still runs the Origin/Host guard and the old token check. What remains is "the off switch is delayed for a client that was already authorised and holds an idle keep-alive". A browser attacker is stopped per request; a local process with the user's rights is already inside the boundary. The fix (a `live` flag in the context plus graceful shutdown) is a few lines and is the first item in the ticket.
- **S4 → Medium (Q2-15).** The strategy's support-bundle control allows "last 2 min transcript", "shown to the user before sending". The report is shown in a read-only text box and Voxxa uploads nothing, so the "shown" part holds; the breach is retention (500 lines, tens of minutes, against 2 minutes). The disclosure needs the operator to paste visible text into a public tracker. That is over-collection under user review, not an exploitable leak. The one-line fix (log the length, not the text) is in the ticket.

## 6. Outbound-request counter: every client

| Client (merged tree) | Counted? |
|---|---|
| `transcription/mod.rs` model download, `reqwest::Client::new()` | yes, once per download; redirect hops to the CDN are not counted |
| `hymnary/mod.rs` builder | yes, per GET (3 sites) |
| `planning_center/mod.rs` builder | yes, per GET |
| `discovery/mod.rs` four localhost probe builders | yes, each |
| `discovery/mod.rs` mdns-sd `ServiceDaemon` (UDP multicast) | **no** |
| `presenters/propresenter7_rest.rs` shared client and probe | yes |
| `presenters/freeshow.rs` shared client and probe | yes |
| `presenters/openlp_v2.rs` shared client and probe | yes |
| `presenters/opensong.rs` shared client and probe | yes |
| `presenters/propresenter7_ws.rs` `tokio_tungstenite::connect_async` | the connect only; the auth message and every later slide command are **not** counted |
| `tauri-plugin-updater` 2.14.1 (its own reqwest 0.13.2) | **no** (documented exclusion in `net_stats`; never invoked) |

**Verdict:** every `reqwest` HTTP `send()` (17 sites) is immediately preceded by `record_request()`. Ran it: the counter was 1 after the FreeShow connect probe and 4 after blank, next and blank; QA's live run reconciled 17 exactly against the mock and proxy logs. Voxxa makes no outbound request at startup. The privacy copy (`index.html` and `main.js` "all to apps you connected to") omits Planning Center, Hymnary and Auto-detect (ticket Q2-32).

## 7. Tokens, transcripts and the ring-buffer logger

- **Default level (Info):** all 70 log sites were read. None formats a token, password, API key, PCO secret or Authorization header; drivers log only the base URL, the PCO client logs song titles, `http_api` logs "listening on".
- **Canaries:** HTTP token `q2-Secret-7c1f` and `q2-secret-TOKEN-6f1d`, OpenLP password `PW-canary-9f3e`, PCO app-id and secret: none in `recent_log`, the diagnostic report or stderr. The mock received the OpenLP password as Basic auth, as designed.
- **Exceptions:** transcripts at Info (`commands/mod.rs` "Heard: …") into the 500-line ring (Q2-15, Medium); at `RUST_LOG=trace`, tungstenite logs written frames including the Pro7 WS auth message, and the RingLogger has no target filter (Q2-33, Low); whisper.cpp prints token text to stderr, which is not part of the report (Q2-33).
- The report goes only to an in-app text box and the clipboard. Release builds have no console.

## 8. Behaviour changes from the security fix-now set

1. **HTTP API token is mandatory.** A blank field generates a 32-hex token and the GUI shows it; tokens under 16 characters are refused. Integrations must send `Authorization: Bearer <token>`. The generated token is not saved, so it changes each time the API is turned on with a blank field (ticket Q2-51).
2. **HTTP API refuses browsers and rebinding.** Any request with an `Origin` header, or with a `Host` other than `127.0.0.1:<port>` / `localhost:<port>`, gets 403. Browser-based controllers stop working (none are known; Companion and Stream Deck plugins run in Node and send no Origin).
3. **`bearer` in any case is accepted** (it was refused before).
4. **Events reach both windows** (`core:event:default` only); no other plugin API is granted.
5. **Imports run off the UI thread, and a malformed PDF returns an error** instead of ending the app.
6. **A second launch focuses the running Voxxa** instead of starting another instance.

S2 and S4 were not fixed, so the diagnostic report still contains transcripts and Off still leaves idle keep-alives open until they close.

## 9. Not tested

The microphone, `/listen/start`, keystrokes and Planning Center with real credentials (user rules); ProPresenter WS under `RUST_LOG=trace` (no WS mock; code reading); mDNS Auto-detect (firewall prompt); a real GB-scale OOM; zip and SQLite fuzzing beyond the cases above; the release workflow (not executed); Linux and macOS.

## 10. Security checklist for the founder

1. Connect a real Stream Deck or Companion profile with the generated token; next, prev and blank must work. In a browser tab on the booth PC, `fetch('http://127.0.0.1:7575/api/v1/blank',{method:'POST',mode:'no-cors'})` must not blank.
2. Click Auto-detect once; if Windows Firewall prompts, allow Private networks only.
3. After a rehearsal, generate a diagnostic report and see what it holds (transcripts are still there until Q2-15 lands).
4. Connect real ProPresenter 7 WS with a password, generate a report, confirm the password is absent.
5. Planning Center: create the PAT on an account limited to Services if possible.
6. GitHub secrets: confirm who holds the minisign updater key `84F2CD98C608E9C9` offline (the strategy asks for two holders) and that `TAURI_SIGNING_PRIVATE_KEY` is set before any tag push.
7. Use Extend, not Duplicate, display mode on the booth PC.
