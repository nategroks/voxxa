# ADR-024: One master layout per church, a data-only weekly service plan, compiled-in binding profiles, and a suggest-first trigger engine

**Status:** Proposed (2026-11-24) · **Date:** 2026-11-24 · **Feature:** church-autoadvance · **Reversibility:** Moderate (a file format churches will hold; versioned as `layout_version: 1`) · **Owner:** nategroks · **Author:** 10x-Team synthesiser, from drafts 11, 13, 14 and reviews 20, 21

Extends ADR-006 (adapters) and ADR-011 (never project lyrics). Security rules for executable steps are in ADR-025.

## Context
- The founder wants "a master layout file that will be referenced regardless of presentation software, paired with hotkey sequences, API commands and CLI commands to autonomously change slides based on song, scripture or key words identified with graphic" (brief § 1).
- The driver trait offers next, prev, blank, unblank, goto_slide and capability flags (`src-tauri/src/presenters/mod.rs`; draft 11 § 0). Dispatch turns conductor actions into driver calls (`src-tauri/src/dispatch.rs:99-180`, per draft 11).
- voxxa's inbound control already exists: window keys (`src/main.js:1582-1610`), a 127.0.0.1 bearer-token HTTP API with 6 routes (`src-tauri/src/http_api/mod.rs:76-86`) and documented Companion, Stream Deck, pedal and MIDI recipes (`docs/INTEGRATIONS.md`). A global-shortcut plugin is loaded and unused (`src-tauri/src/lib.rs:119`).
- The only live install is WorshipTools Presenter: keys only, no read-back, no blank (ADR-019). On it, every graphic, scripture and other-item action can only be an operator hint (draft 11 § 2.4). Three API drivers disagree with vendor docs (W1 FreeShow, W2 OpenLP, W3 OpenSong; draft 11 § 0.1), unverified live.
- Review 20 found: a free-form `run` action triggered by speech is arbitrary code execution (S-01); an e-mailed service file must not raise automation (S-02); cues armed by conductor state fire mid-song (S-15); a bag-of-words matcher fires on sermon sentences (S-17); zero-event gates on small samples prove little (S-18).
- "Everything else" has 1 line left (`.10x/handoff.md:111`); ADR-028 handles the budget.

## Decision
1. **Three file kinds, JSON, schema v1:**
   - `layout.json` (the **master layout**, per church, private): `presenter.profile`; a `graphics` catalogue (abstract ids, operator labels, and one entry per profile, `graphics.<id>.<profile> = {"via": …}`, § 2); `cues` (id, label, phrases, abstract actions, mode, cooldown); `defaults` (modes per trigger, suggest expiry); `hotkeys`; optional named `actions`. No free command anywhere.
   - `services\<date>-<slot>.service.json` (the **service plan**, weekly, private): `items[]` of kinds `song` (song file slug, arrangement of section labels from the `.txt` importer (ADR-022 § 7) or `library_default`, per-profile `app` refs including `service_position`), `scripture` (parsed reference), `graphic`, `spoken` (welcome, prayer, sermon, reading, communion, offering, other; armed cue ids), `media`, `announcement`, `blank`; on any item, optional `on_start` and `on_end` lists of abstract actions run when the item starts or ends (limited by § 3); optional `overrides` that may only **lower** a mode; free-text `note` fields marked "not for personal details".
     - **Arrangement labels:** `v<n>` = the n-th Verse section in file order; `c`/`c<n>` = Chorus (the first when no number); `b` = Bridge; `t` = Tag; `o<n>` = Other. Lower case in files; upper case in printed output (`voxxad --learn` prints "V1 C V2 C B C").
   - `bindings/<profile>.json` (the **binding profile**, public, compiled into the binary with `include_str!`): for each abstract action (keys named as in § 2), a list of steps from a closed set (`driver` method, `keys`, `http`, `ws`, `wait_ms`) or `{ "hint": … }` or `{ "unsupported": … }`; `identity` step; `target_process`; allowlisted keys; `leading_blank`; `goto_scope`. Church-side overrides are not supported in v1.
     - **Profile ids:** the loader maps kit profile ids to the names voxxa's saved config already uses: kit id `keystroke.worshiptools_presenter` = saved config `keystroke_profile: "worship_tools_presenter"` (`index.html:194`). Existing configs are not changed.
2. **Abstract actions:** `next`, `prev`, `goto(item, slide)`, `blank`, `unblank`, `show_graphic(id)`, `clear_graphic(id?)`, `show_scripture(ref)`, `scripture_next`, `scripture_prev`, `clear_scripture`, `clear_slide`, `clear_media`, `clear_all`, `action(id)`. `clear_media` and `clear_all` are never automatic and never in a service file. A service may call `action(id)` only if the action is marked `service_callable` and holds no HTTP or CLI step: no step **written in `layout.json`** of kind `http` or `cli`. Abstract actions inside the composite (`show_graphic`, `blank`, …) are allowed, because they resolve through the compiled-in binding allowlist.
   - **Binding keys (one `via` convention; spec § 4.6 and `kit/prompts/03-layout.md` use the same wording):**
     - `via` is a name defined per binding profile. A profile's `actions` keys are `<abstract_action>` or `<abstract_action>.<via>`. Examples: `show_graphic.overlay` (freeshow), `show_graphic.macro` (propresenter7_rest), `goto.item`, `goto.keys`.
     - `hint` is a via every profile has implicitly. It shows the operator the graphic's name and never sends anything.
     - A layout graphic entry is per profile: `graphics.<id>.<profile> = {"via": ..., ...}`. Its `via` must exist in **that** profile, not in every profile.
     - A missing entry for the active profile falls back to `hint`, and `--check-layout` prints a warning, not an error. So "changing app = changing one field" stays true: a missing mapping degrades to a hint.
     - `goto(item, slide)` maps to `goto` (a slide in the item already live), `goto.item` (put the item live, then the slide) or `goto.keys` (a key chord, confirmed on the presenter test card, T8); `show_graphic(id)` and `clear_graphic(id)` map to `show_graphic.<via>` and `clear_graphic.<via>`, with the `via` of that graphic's entry for the active profile.
     - A key that exists runs only after its contract test passes; until then the driver turns it into a hint (§ 5).
   - **Typed placeholders** (the only ones a step may use; an unknown placeholder fails the profile's load test; draft 11 § 2.2):
     - whole numbers (u32): `{slide}` (voxxa's slide index), `{app_slide}` and `{app_slide1}` (the app's slide index after `leading_blank`; `1` = 1-based), `{item.project_item_index}`, `{graphic.index}`, and `{item.set_slide_offset_plus_app_slide1}` (OpenSong only: `set_slide_offset + app_slide + 1`, computed in code, not in the file);
     - app tokens (the § 3 pattern): `{item.show_id}`, `{item.layout_id}`, `{item.service_item_id}`, `{item.presentation_uuid}`, `{graphic.id}`, `{service.freeshow_project}`;
     - a parsed scripture reference: `{ref.book}`, `{ref.chapter}`, `{ref.verse_start}`, `{ref.verse_end}`, and `{ref.display}` built from them by a fixed formatter;
     - `{secret}` from the OS credential store, only in an auth header or field, never in a URL path or a log;
     - labels `{item.label_hint}` and `{graphic.label_hint}`, only inside a `hint` shown to the operator; refused in any step.
     No placeholder that reaches an app carries heard words, titles, slide text or lyrics (ADR-025 § 5).
3. **Validation** (`voxxad --check-layout <file>`, line-numbered plain sentences; the validator lives in the layout row, and voxxad only parses the flag and dispatches, ≤ 10 lines, ADR-028 § 3): every id resolves; each graphic entry's `via` exists in that entry's profile as `show_graphic.<via>`, or is `hint`; a graphic with no entry for the active profile is a warning, not an error, and falls back to `hint`; no key named `text`, `lyrics`, `slides`, `words`, `run`, `password`, `token`, `secret`, `api_key`; service files contain no `steps`, `programs`, `actions`, `host` or `port` and raise no mode; `on_start`/`on_end` only `show_graphic`, `clear_graphic`, `blank`, `unblank`; phrases 2–8 words of letters, spaces and apostrophes, never a substring of a song title in the service; app tokens match `^[A-Za-z0-9_-][A-Za-z0-9._:-]{0,127}$` and are never `.` or `..`, substituted in one pass, percent-encoded into an existing path segment only; UTF-8 without BOM. Each rule has a refusal test (≥ 25).
4. **Compile, don't replace:** a service compiles to today's `setlist.json` (songs only) plus an item timeline; the conductor, follower, replay and eval are unchanged. For keys-only apps, the `service_position` that each song item carries in the service plan (`app.<profile>.service_position`) lets voxxa check the "nothing between two songs" rule before the service.
5. **Trigger engine v1** (new module `src-tauri/src/layout/`, row ≤ 300):
   - inputs: follower events, the item pointer, the cue matcher, operator events; scripture later (Q16–Q18);
   - **cue matcher:** phrase tokens in order, contiguous with ≤ 1 inserted word, within the last 6 s of words; tokens of ≥ 5 letters may differ by one edit; cooldown 120 s per cue, 10 s for any graphic;
   - **arming:** cues only inside a `spoken` item that lists them and when no song anchor has been seen for ≥ 30 s; never armed by conductor state; all audio triggers disarmed on `media` and `announcement` items;
   - **modes:** `off`, `suggest`, `auto`; v1 ships cues and scripture at `suggest` only; `auto` for a cue requires 0 wrong fires in ≥ 300 armed opportunities or ≥ 30 h of armed talk audio (CC-licensed public sermons, never private services) and a typed confirm recorded in `layout.json`;
   - **priority:** operator 100 > safety hold 90 > follower 80 > scripture 60 > cue 40 > item start/end 30; one action per output at a time (150 ms coalescing); lower priorities held 5 s after any operator action; a cue or scripture trigger inside a song is downgraded to a suggestion; suggestion cards show only between items;
   - **execution:** song moves through existing driver methods; a graphic or scripture key that exists in the profile runs only after its contract test passes on a live install (none in v1); until then the driver turns it into a hint. Cue events log ids and scores only (session log `cue` lines).
6. **Hotkeys and API into voxxa:** global hotkeys registered only while Songs is on, released on Off; defaults F13 Next, F14 Back, F15 Pause/Resume, F16 Accept, F17 Dismiss, F18 "screen is ahead", F19 "screen is behind"; keyboard F-keys only after a conflict check at registration; never Ctrl+Alt+arrows; a key injected after a hotkey waits until physical modifiers are released (≤ 300 ms) or is refused; no hotkey stops listening or turns voxxa Off. The HTTP API gains `POST /api/v1/pause`, `/resume`, `/accept`, `/dismiss` with the same token and origin rules.
7. **Names on screen** (UI consultant): "Service plan", "Who moves it" (voxxa / voxxa suggests / You), "Cue words", "Pause voxxa".

## Alternatives Considered
| Alternative | Pros | Cons | Why Not |
|---|---|---|---|
| One minimal `layout.json` with `say` phrases and `run` argv (draft 14) | smallest | `run` is code execution from a shared file (S-01); bag-of-words matcher misfires (S-17); no service/church split (S-02) | keep its size discipline, replace its execution and matcher |
| Full bindings executor, cue engine, TOFU trust, CLI allowlist and new drivers now (draft 11, ~1,200 Rust lines, 9 slices) | covers every app | nothing but WorshipTools can be tested; W1–W3 unverified; budget wall | ship the format and the WorshipTools path; enable app steps per live install |
| TOML or YAML | comments | nested arrays read badly; no parser in the tree; ADR-003 rejected YAML | JSON with `note` fields and a JSON Schema |
| Church-editable binding overrides | flexibility | a file that becomes a command runner; needs the trust store | compiled-in allowlists in v1 (S-04) |
| Bare F6–F9 global hotkeys (draft 13) or Ctrl+Alt chords (draft 14) | familiar keys | collisions with other apps; display rotation on some Intel drivers [UNVERIFIED]; chord leakage into injected keys | F13–F24 for devices; checked F-keys for keyboards (S-13) |

## Consequences
### Positive
- One file per church describes the service in app-neutral terms; switching apps changes one field (a graphic with no entry for the new app degrades to a hint, with a `--check-layout` warning).
- No path from heard words to a command; e-mailed plans can only make voxxa more cautious.
- The volunteer sees before the service which items voxxa moves and which they move.
### Negative
- On WorshipTools, graphics and scripture stay manual (hints) until MIDI or an app API exists.
- Cue auto mode is far away by design.
### Risks
- App ids in a layout go stale when an app re-imports a song: the pre-service check compares (Q12 read-back where available).
- Global hotkeys may be taken by another program: registration failure shows a sentence and Run continues.

## Dependencies
- Depends on: ADR-006, ADR-011, ADR-019, ADR-022 (item pointer, song events), ADR-025 (security), ADR-028 (budget).
- Constrains: Q12 (import fills `app` refs), Q13 (contract fixtures for profile steps), Q18 (scripture items).
- Plan steps: V13 (layout v1), V14 (Run screen).

## Review trigger
A second church on an API app with a test install (enable its profile steps); cue exposure bar met (auto mode); a request for church-specific bindings (then ADR-025's trust store first).
