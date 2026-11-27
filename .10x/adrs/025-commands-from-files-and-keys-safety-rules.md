# ADR-025: Safety rules for every command voxxa sends: keys go only to a known window, positions are counted per key, files never run programs freely

**Status:** Proposed (2026-11-24) · **Date:** 2026-11-24 · **Feature:** church-autoadvance · **Reversibility:** Easy (guards; nothing users depend on) · **Owner:** nategroks · **Author:** 10x-Team synthesiser (Security Engineer lens), from drafts 11, 14 and review 20

Amends ADR-006's keystroke fallback (every keystroke profile gets a target, not only WorshipTools). Constrains ADR-024.

## Context
- Only the WorshipTools profile checks the front window (`src-tauri/src/presenters/keystroke.rs:59-63`). Every other keystroke profile types into whatever is in front; `voxxad --presenter keystroke` has no window to check and types into a terminal if one is focused (`docs/INTEGRATIONS.md:260-266`).
- Dispatch sends N Rights 60 ms apart and writes `sent.global` only after the loop; an error mid-loop returns early with the position stale (`src-tauri/src/dispatch.rs:163-175`). WorshipTools has no read-back to notice (review 20 S-10).
- Presenter run as administrator silently drops synthetic keys (runbook; review 20 S-12) **[UNVERIFIED that SendInput reports no error]**.
- The founder asked for CLI commands paired with the layout. Draft 14 proposed a `run` argv in `layout.json` fired by heard phrases; review 20 rated it Critical (S-01). Draft 11 proposed an allowlist with hashes and trust-on-first-use, with gaps (S-03, S-05).
- Strategy rank 4: zero outbound network calls during a service (`.10x/specs/2026-10-24-software-strategy.md:29`).

## Decision
1. **Every keystroke profile has a target.** Profiles without a known executable capture one: when Songs is turned on, the operator clicks the presentation window and voxxa records its process. Keys go only to that process; otherwise nothing is sent and the light is amber ("Bring <app> to the front."). `voxxad --presenter keystroke` refuses to start without `--target-exe <name>` (V8; ≤ 5 lines of ADR-028's voxxad share). The WorshipTools guard additionally checks the foreground window class or title from a booth test card (S-11), once the card exists.
2. **Positions are counted per key.** `sent.global` moves by ±1 after each key that succeeds. If a sequence fails part-way, `position_uncertain` is set: no automatic moves until the operator confirms where the app is (the Resume card asks in the app's terms, with no default button).
3. **Elevated targets are refused** with a red sentence when the target process is elevated and voxxa is not.
4. **No free program execution.** No `run` key in any file; the validator refuses it.
5. **CLI steps are not built in v1.** If ever built, all of these hold:
   - off unless `layout.json` has a `programs` allowlist; never callable from a service file or a cue in `auto`;
   - absolute path inside `%ProgramFiles%` or `<church_root>\tools`; voxxa copies the program to a voxxa-owned folder, hashes the copy (SHA-256) and runs the copy;
   - refused types: `.bat`, `.cmd`, `.vbs`, `.js`, `.hta`, `.lnk`; refused basenames: cmd, powershell (except below), pwsh, wscript, cscript, mshta, rundll32, regsvr32, curl, bitsadmin;
   - the only interpreters: `C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe -NoProfile -NonInteractive -File <pinned .ps1>` (the script refuses `Import-Module`, dot-sourcing, `iex`, `Invoke-Expression`: best effort) and `osascript <pinned .scpt>`;
   - argv from typed placeholders only (u32, app token, parsed scripture reference); no placeholder for heard words, titles or slide text;
   - cleared environment (`SystemRoot`, fixed `PATH`, `TEMP`), null stdin, 2 s timeout (max 5 s), output discarded, exit code and duration logged;
   - approval only through the GUI, stored outside the church folder (`%LOCALAPPDATA%\com.voxxa.app\trust\`, Tauri's app-local-data folder) as an HMAC over (canonical path, SHA-256, approved-at) keyed by a secret in the OS credential store; any mismatch → hints only.
6. **HTTP and WebSocket steps** (when a profile's steps are enabled): only GET/POST; only the configured presenter host; non-loopback hosts show an amber "credentials cross the network unencrypted" warning, pin the IP and the identity response; multi-step moves on apps that can blank run blank → steps → unblank and stay blank with "position unknown" if any step fails; steps are never retried automatically.
7. **Refused app actions** (compiled into each profile): content-editing or free-text actions (FreeShow `create_show`, `set_plain_text`, `set_show`, `add_group`, `rearrange_groups`, `send_rest_command`, `emit_action`, `delete_project`, `remove_project_item`; ProPresenter `PUT`/`DELETE`, stage messages, messages with text tokens; OpenSong alerts and song loads); global key refusals: Esc, Enter, Alt+Tab, Alt+F4, Ctrl+W/Q/N/O/S/P/V, Delete, F10 (except where a test card proves it safe), any Win-key chord, Ctrl+Alt+Del; at most 4 chords per action.

## Alternatives Considered
| Alternative | Pros | Cons | Why Not |
|---|---|---|---|
| `run` argv in the layout without a shell (draft 14) | covers any API through `curl.exe` | the file is the payload, speech the trigger; `PATH` hijack; breaks "no outbound calls" | Critical finding S-01 |
| Trust-on-first-use hashes in `<church_root>\trusted.json` (draft 11) | simple | whoever edits the file edits the trust record | trust outside the church folder, HMAC (S-03) |
| Keep the universal profile unguarded (today) | no setup | types into chats, browsers, terminals; the follower moves more often | target for every keystroke profile (S-09) |
| Re-read the app's position after every key | exact | WorshipTools has no read-back | per-key counting plus an uncertainty flag; read-back (#185, Q12) where possible |

## Consequences
### Positive
- Closes a pre-existing wrong-slide bug before the follower makes more moves.
- The founder's "CLI commands" exist as a safe inbound surface (the HTTP API from any shell) and a documented, gated outbound design.
### Negative
- The universal profile needs one extra click per service; `voxxad` keystroke users must name the target.
- No outbound CLI integrations in v1.
### Risks
- The elevation check may need Win32 calls whose cost is unverified.
- Window-class checks depend on the app's version; a changed class shows amber, never a silent pass.

## Dependencies
- Depends on: ADR-006, ADR-019, ADR-028 (budget for dispatch changes).
- Constrains: ADR-024, every future driver.
- Plan steps: V8 (dispatch and keystroke safety), V13 (refusals in the validator).

## Review trigger
A church asks for a CLI integration; an app adds a read-back API; a key sent to a wrong window is reported.
