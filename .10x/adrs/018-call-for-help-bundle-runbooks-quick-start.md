# ADR-018: Call for help writes a reviewed zip to the Desktop; runbooks linked from the light; a one-page quick-start

**Status:** Accepted 2026-11-19 (the user's binding Q9 decisions of 2026-11-19: privacy first with #102 fixed in its own commit, the review screen and the words checkbox, secret redaction, an empty support address, no network call, the existing budget rows only, two ways of GUI testing, new commands allowed, ADR number) · **Date:** 2026-11-19 · **Feature:** church-autoadvance · **Reversibility:** Easy (one Rust module, one GUI file, four commands; removing them restores Q8's placeholder) · **Owner:** nategroks · **Author:** 10x-Team (Principal Architect + Security Engineer), Q9 10x lane

Implements the strategy's "Call for help" button and the three runbooks (`specs/2026-10-24-software-strategy.md` lines 161 and 242) and research objective O7's quick-start card. Amends ADR-017 § 1 ("Call for help (Q8)") and § 2 (runbook links). Supersedes nothing.

## Context
- Q8's Call for help only shows a sentence (`src/simple/simple.js:24-25,35`). It calls no command because `generate_diagnostic_report` carries every transcript the GUI heard: `log::info!("Heard: …")` at `src-tauri/src/listen.rs:182` goes into the ring buffer that `diagnostics/mod.rs:153` copies into the report (Q7-08, **#102**, medium).
- The strategy's bundle was `voxxa.log` + `voxxa.toml` + the last 2 minutes of transcript, attached to a support e-mail. There is no `voxxa.toml` (ADR-010 Decision 2): the operator's settings live in the webview's `localStorage`. There is no log file either: the log is the in-memory ring buffer behind `generate_diagnostic_report`.
- **Only `voxxad` writes session logs** (`<data dir>/voxxa/sessions/session-YYYYMMDD-HHMM[-N].jsonl`, `src/cli/voxxad/main.rs:310-341`, JSON lines with `t` in ms and `type`; `hypothesis` lines carry the recognised words). The GUI writes none (Q3-09, #53). So on a GUI-only machine there are no recent words to bundle.
- `localStorage` holds `voxxa.firstLaunchDone`, `voxxa.audioDevice`, `voxxa.presenterForm` (kind, host, port, username, keystroke profile; passwords are never stored, `src/main.js:1131-1133`), `voxxa.lastModel`, `voxxa.smartConfig`, `voxxa.language`, `voxxa.httpApi` (port and on/off) and **`voxxa.recentSetlists`: up to eight whole setlists, slide text included** (`src/main.js:456-477`). Planning Center's app id and secret are kept in memory only (`src/main.js:157`); the HTTP API token is never stored in `localStorage`.
- A `mailto:` link cannot attach a file. The webview may call no plugin API (`capabilities/default.json`: `core:event:default` only; `tests/capabilities.rs`). `tauri-plugin-shell` 2.6 is already initialised (`lib.rs`); from Rust its `open` needs no capability and no scope.
- Forces: no network call of any kind; nothing written before the user confirms; the GUI is a public repo's code, so the support address is not known yet; the budget rows may not grow (backend "everything else in src-tauri/src" 2,309 / 2,500, 191 left; "Simple mode (src/simple/*)" 352 / 600, 248 left; GUI row 2,780 / 3,000 without growing panels).

## Decision

### 1. #102 first (its own commit, with a test)
- `listen.rs` logs `Heard N words` at Info instead of the text. Debug would still reach the buffer under `RUST_LOG=debug` (the ring logger's level comes from `RUST_LOG`, `diagnostics/mod.rs:84-88`), so the text is not logged at any level.
- Every other `log::` call in `src-tauri/src` outside `cli/` was checked (70 calls): the conductor logs song indices, probabilities and titles (`aligner/mod.rs:419,608`), the presenters log slide indices, dispatch errors name titles at most (ADR-017 § 2). None carries recognised text. `Transcription error: {e}` carries Whisper's error, not words.
- Test (`tests/help_bundle.rs`): install the ring logger, run `listen::run` with a fake recogniser that hears three placeholder words, then assert the buffer (the report's `recent_log`, exposed as `voxxa_lib::recent_log`) holds `Heard 3 words` and none of the words.

### 2. The bundle: contents and format
One zip, `voxxa-help-YYYYMMDD-HHMMSS.zip` (UTC), deflate (`zip` 9.0.5, already a dependency), opened with `create_new` so nothing is overwritten. Files:

| File | What | Source |
|---|---|---|
| `diagnostic-report.json` | `generate_diagnostic_report`'s `DiagnosticReport`, pretty JSON: version, OS, audio device names, presenter kind and state, song title and slide numbers, machine state, outbound request count, the last 500 log lines (no heard text after § 1) | `diagnostics::generate_report` |
| `operator-settings.json` | the `voxxa.*` keys of `localStorage`, each value parsed as JSON, secrets redacted (§ 4); `voxxa.recentSetlists` replaced by `"[left out: whole setlists]"` (they hold slide text) | the GUI passes `localStorage` to the command |
| `words-last-2-minutes.txt` (optional) | the `hypothesis` lines of the newest session log whose `t` is within the last 120,000 ms of that log's last `t`, one per line as `<seconds> s  <words>`, under a header naming the log file | `<data dir>/voxxa/sessions/` |

- "Newest" is by file name (`session-YYYYMMDD-HHMM[-N]`, `N` compared as a number), not by modified time. A log without `hypothesis` lines in the window gives a words file that says so. No log at all gives no words file, and the review screen says that Voxxa found no record of what it heard (the GUI writes none, #53, ticket Q9-02).
- No audio, no setlist, no slide text. Song titles may appear (the report's `current_song`, conductor log lines): the operator window shows them too (ADR-011).

### 3. Review before write, and the words opt-out
1. **Call for help** opens a dialog (`src/simple/help.js`) and calls `help_bundle_preview { storage }`. The backend builds the files **in memory** and keeps them as the pending bundle; it writes nothing.
2. The dialog lists every file with its size, and shows each file's content (the words file open, the others one click away in `<details>`). Text goes in with `textContent`, never as HTML.
3. A checkbox, **on by default**: "Include the last 2 minutes of words the app heard". It is shown only when there is a words file.
4. **Save to Desktop** calls `write_help_bundle { includeWords }`. It writes exactly the pending files that were shown; unchecked, the words file is **left out of the zip** (absent, not empty). A second Save finds no pending bundle and writes nothing. **Cancel** (or Escape) closes the dialog; nothing is written.
5. While the dialog is shown, a capture-phase `keydown` listener on `window` stops every key from reaching `main.js`'s shortcuts (as ADR-017's guide does, Q8-10), so Space cannot start listening behind it.

### 4. Secret redaction (operator settings)
- Only `voxxa.*` keys are exported. A value that is not JSON is kept as a string.
- In every object, at any depth, a property **whose name** reads as a secret has its value replaced by `"[redacted]"`. The name is lower-cased with everything but letters and digits removed. It is a secret if it contains `token`, `secret`, `pass` (password, passwd, passphrase), `pwd`, `credential`, `authoriz` or `appid`, or ends in `key`, `auth` or `pat`. So `voxxa.presenterForm.password`, `apiToken`, `API_KEY`, `pco_app_id`, `appId`, `pcoSecret`, `http_api_token`, `key`, `client_secret` and `Authorization` are redacted; `keystroke_profile`, `port`, `host`, `username` and every `SmartConfig` field are not. Over-redaction (a harmless name such as `hotkey`) is accepted.
- A top-level key that reads as a secret is redacted whole.
- Test: a storage map with each of those names, nested and at top level, plus a `recentSetlists` with placeholder slide text: none of the secret values and none of the slide text appears in the output; the harmless fields do.

### 5. E-mail draft and open-folder
- `SUPPORT_EMAIL` is one named constant in `src-tauri/src/help/mod.rs`, **empty** (ticket Q9-01). The draft opens with no recipient; the volunteer types their tech lead's address.
- After a successful write, the GUI calls `open_help_email`. The backend builds `mailto:<SUPPORT_EMAIL>?subject=…&body=…` (every byte outside RFC 3986's unreserved set percent-encoded). Subject "Voxxa: call for help"; the body says which file to attach, its full path, and that its folder has just opened. Then it opens the zip's folder. Both go through `tauri-plugin-shell`'s `open` from Rust (no webview capability, no scope widening). The command takes no argument: it opens only the zip this process wrote last.
- **No network call.** Opening a `mailto:` URL hands it to the user's mail program, which sends nothing until the user presses Send. Nothing is uploaded and there is no telemetry.

### 6. QA override
`VOXXA_HELP_DIR=<folder>` (environment, read per call) sandboxes the feature for testing: the zip is written there, session logs are read from `<folder>/sessions`, and `open_help_email` **opens nothing** (it returns the two targets it would have opened). Without it, the zip goes to `dirs::desktop_dir()`. It exists so the guarded real app can write a bundle without touching the user's Desktop, their real session logs, their mail program or Explorer. It is a developer setting, documented here only.

### 7. Runbooks linked from the light
- `docs/runbooks/no-audio-device.md`, `presentation-app-not-reachable.md`, `model-missing.md`: symptoms, what the light says, numbered steps, when to Call for help; each under about 60 lines; written for a volunteer on Windows. `docs/quick-start.md`: one printed page, in the guide's order (microphone, presentation app, voice model, 20-second test), the three controls, every light colour, Call for help.
- `statusLight` (ADR-017 § 2) returns a third element, the runbook, on four rows; the sentences do not change:

| Row | Light | Runbook |
|---|---|---|
| 1 mic lost | red | `no-audio-device` |
| 3 no model (grey, shown only while Off) | grey | `model-missing` |
| 5 presenter refused | red | `presentation-app-not-reachable` |
| 6 presenter not found | red | `presentation-app-not-reachable` |

  "Model missing" is not red in ADR-017's table, so its link sits on the grey row 3 sentence. A "What to do" link shows under the sentence on those rows only.
- The link calls `open_runbook { name }`. `help::runbook_url(name)` gives `https://github.com/nategroks/voxxa/blob/main/docs/runbooks/<name>.md` (the repo is public) for the three names and `None` for any other, which `open_runbook` refuses; otherwise it opens the URL with the same `open`.
- **Links resolve** (tests): every name in the backend's list and every name `statusLight` returns is a file under `docs/runbooks/`; `runbook_url` gives exactly the URL above (owner, repo and branch included) and `None` for `../../evil`; every relative link inside the new docs points to a file that exists.

### 8. Budget fit (no cap raised, no row added)
Backend: `src-tauri/src/help/mod.rs` and five lines in `lib.rs` and `diagnostics/mod.rs`, in "everything else in src-tauri/src". GUI: `src/simple/help.js`, a few CSS lines and the link in `simple.js`, in "Simple mode (src/simple/*)". `index.html`, `main.js` and the panels are unchanged. Tests live in `src-tauri/tests/` and `tests/gui/`, which are not counted. Measured numbers are in the SDE record.

### 9. Testing
1. Rust (`tests/help_bundle.rs`, CI on three OSes): #102 (§ 1); redaction (§ 4); the words window on a temp session folder with placeholder words (newest by name including `-10` after `-2`, the 120 s cut, no log, a log with no words); the zip with and without the words file (absent, not empty) and `create_new`; the preview writes nothing; the mailto URL (no recipient, encoded, names the file); links resolve (§ 7).
2. Browser with the mocked backend (`tests/gui/e2e.mjs`, Q8's harness): the dialog lists every file and shows the words; nothing is written before Save; Cancel writes nothing; unchecked sends `includeWords: false`; Save then calls `open_help_email` once; a slow write keeps Save disabled; Space on the dialog starts nothing; each runbook link calls `open_runbook` with its name.
3. The guarded real app (scratch build, identifier `com.voxxa.qa9x`, `VOXXA_HELP_DIR` set to a scratch folder): Call for help opened to its review screen; the bundle written there by the real backend; the e-mail and folder step never triggered.

## Alternatives
| Alternative | Why not |
|---|---|
| Keep the words in the diagnostics buffer and filter them out of the report | Any later path that copies the buffer leaks them again; the user decision is that no transcript reaches the buffer. |
| A GUI-side ring of the last 2 minutes of `transcription` events | A new in-memory transcript store in the webview, and the task names the session log. Ticket Q9-02 with #53 (a GUI session log) instead. |
| Grant `shell:allow-open` to the webview | The default scope (`^((mailto:\w+)|(tel:\w+)|(https?://\w+)).+`) refuses `mailto:?subject…` (no recipient) and folders, so the scope would have to widen to anything; a Rust command that opens three fixed things is narrower. |
| `tauri-plugin-opener` | A new crate for what the existing shell plugin already does from Rust; its `open` is deprecated in favour of the opener but still supported in 2.x (ticket Q9-03 to move when the shell plugin drops it). |
| Write the zip, then show it | The task and the user decision: shown before written. |
| Regenerate the bundle on Save | What is written could differ from what was shown (new log lines, a new time stamp). The pending bundle is written as shown. |
| Upload to a support portal | No network call of any kind (user decision). |
| `README.txt` inside the zip | Lines; the e-mail body and the file names say what it is. |
| A destination picker in the review screen | More controls for a volunteer and a new dialog API; the QA override covers testing. |

## Consequences
Good: a volunteer gets one zip on the Desktop and an e-mail draft that says what to attach, after seeing every byte; no transcript reaches the diagnostics report any more; the red light points at a one-screen runbook. Bad: on a GUI-only machine there is no words file until the GUI writes session logs (#53); the support address is empty, so every draft needs a typed recipient; the runbook links need a browser and the network (GitHub), which the volunteer opens themselves; the e-mail opens through the shell plugin's deprecated `open`. Risks: a webview compromise could ask for a preview with arbitrary `storage` and write it to the Desktop under a fixed name (the webview is the app's own code; CSP is off, as before, Q2).

## Review trigger
Re-open when the support address exists (Q9-01), when the GUI writes session logs (#53, Q9-02), when `tauri-plugin-shell` drops `open` (Q9-03), or when a pilot's bundle turns out to miss what support needed.
