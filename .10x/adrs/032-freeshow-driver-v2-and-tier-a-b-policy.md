# ADR-032: FreeShow driver v2 and the Tier A/B app policy

**Status:** Proposed (2026-11-26) · **Date:** 2026-11-26 · **Feature:** church-autoadvance · **Reversibility:** Easy (the driver sits behind the `freeshow` profile; the Presenter path does not change; the tier rule is one dispatch check) · **Owner:** nategroks · **Author:** 10x-Team, from the round-2 review `round2/42` § 1–2, the adjudication `round2/50` (C1–C4, C9, C17, R1, R8, R10, R17, FA-44, FA-47, FA-51, FA-55–FA-57) and the V8b prompt in `round2/52`

Amends ADR-024 § 2 (two 1-based placeholders; song select becomes a driver method), ADR-025 § 2 and § 6 (fire-then-verify replaces per-key counting on Tier A) and ADR-031 § 7 (names the jump). Records FA-44 as answered: (d) FreeShow trial via V3e, (e) suggest-only on Presenter until then (50 § 5). Accepted in V8b, after V3e records a go.

Tags: (measured) read in code by the review named; (cited) from a source named inline; (inferred). I did not read FreeShow's source; every FreeShow fact below is 42's reading of `ChurchApps/FreeShow` at commit `c84386f3` (2026-11-26, `package.json` `1.6.8-beta.6`, a beta on `main`). `api.ts` alone means `src/frontend/components/actions/api.ts`; the REST server is `src/electron/utils/api.ts`. Paths are written as 42 writes them.

## Context
- **The shipped driver has never moved a slide by index (C1).** FreeShow's REST handler takes the whole body as the action (`src/electron/utils/api.ts:141`, `:151`). The renderer passes that object to each handler, and each handler reads its fields at the top level (`api.ts:448-462`; `index_select_slide` at `api.ts:245`, `apiHelper.ts:197-208`). voxxa nests the fields under `data` (`presenters/freeshow.rs:47-49`). So `data.index` is undefined, FreeShow shows a toast and returns (`apiHelper.ts:204-205`), and the REST layer still answers 204 (`src/electron/utils/api.ts:152-156`). `next_slide`, `previous_slide`, `clear_slide` and `restore_output` take no fields and do work. (measured, 42; confirmed by 41, 46, 47)
- **Index base (C2).** `api.ts:453-454` decrements `index` on receipt ("API start at 1, code start at 0"). The wire is 1-based. `index: 0` becomes −1 and selects nothing, still 204. `get_output` returns the stored 0-based layout index (`apiHelper.ts:242-253`, `apiGet.ts:37-40`). The kit binding's `{app_slide}` would land one slide early. (measured, 42)
- **Which show a jump targets.** Without `showId`, `selectSlideByIndex` uses the editor's active show (`apiHelper.ts:200`), not the show on the output. If the operator clicks another song in the drawer, a bare jump lands in the wrong song. `get_output` returns the output's `slide.id` and `slide.layout`. (measured, 42)
- **`id_select_group` is not a jump (C3).** Its `id` is a global group key (defaults `break, bridge, chorus, intro, outro, pre_chorus, tag, verse`, `src/electron/data/defaults.ts:5-14`). It plays the first matching slide after the current one, wrapping (`apiHelper.ts:68-91`, `src/frontend/components/helpers/showActions.ts:622-661`). Two verses share `verse`, so it cannot reach verse 3. Planning Center and ChurchApps imports write label-style keys such as `verse 1` (`src/electron/contentProviders/planningCenter/request.ts:1102`, `…/churchApps/ChurchAppsShowBuilder.ts:84`), which it silently ignores. `name_select_slide` fuzzy-matches labels (`apiHelper.ts:210-240`). (measured, 42)
- **No error channel.** Unknown actions and failed selects answer 204 (`api.ts:455`). A locked output makes `index_select_slide` and `gotoGroup` no-ops (`apiHelper.ts:69`, `:243`). Read-back after a move is the only way to know it happened. Getter replies are polled every 50 ms for up to 10 s (`src/electron/utils/api.ts:267`), so a renderer stall can outlast a short timeout. (measured, 42)
- **Text getters.** `get_output_slide_text` returns the slide's words (`apiGet.ts:42-48`). `get_output_group_name` returns the show's group label (`apiGet.ts:50-56`). `get_show`, `get_slide` and `get_plain_text` also return text. (measured, 42; cited, 52 V3e)
- **Auth and binding (C4).** The password is optional: `checkAuth` accepts every request when none is set (`src/electron/utils/api.ts:17-38`). The token is read from `Authorization: Bearer`, raw header keys, `?auth=` and several body fields (`:20-36`). REST listens with no host (`:121`); WebSocket likewise (`:54-57`) with `cors: {origin: "*"}` (`:65`); OSC binds `0.0.0.0` (`:198`). All three listen on every interface. A 401 is JSON (`:136`). voxxa's driver sends no credential (0 matches for Authorization or bearer), and its module comment says there is no authentication (`freeshow.rs:5`). (measured, 42, 43, 46, 47)
- **Transition.** FreeShow's default text transition is a 500 ms fade (`src/frontend/stores.ts:245-247`). A 0.18 s median fire would look like a 0.4–0.7 s change. (measured + inferred, 42)
- **Drift.** `main` is a beta, and several actions carry `// BC` markers (`api.ts:221-256`). The booth will run the stable channel (F5). Whether stable matches `c84386f3` on every fact above: I don't know until V3e's FS3–FS5 rows. (measured, 42)
- **Song changes blank today (C17).** `can_query_state: false` (`freeshow.rs:87-91`) makes dispatch blank and ask the operator on every conductor-driven song change (`dispatch.rs:117-143`), pinned by `tests/dispatch.rs:289-305`. (measured, 42, 46, 47)
- **Layouts are many-to-one.** A FreeShow layout for `v1 c v2 c` lists the chorus slides twice; voxxa's setlist has each slide once. (inferred, 42 § 2)
- **Presenter.** WorshipTools Presenter has next and previous slide, next and previous item, and clear keys. It has no key for a section or a slide number, and no read-back (cited, ADR-031 context). live4 is not evidence of its ceiling (C9). On Presenter, voxxa sees a manual move only through its own hotkeys (R10).
- **Budget.** The dispatch changes need the V7 wiring row ("everything else" has 1 line left) and V8-lite's `position_uncertain` (C17). Presenters row 1,374 / 2,000 (measured, 52).

## Decision
### A. FreeShow driver v2 (V8b)
1. **Body.** Every request is `{"action": "<name>", ...fields}` with the fields at the top level. No `data` key, ever. One helper builds it.
2. **Index.** voxxa's 0-based `app_index` goes on the wire as `index: app_index + 1`. Read-back is compared with `app_index` as is. `index: 0` is never sent. The kit binding uses `{app_slide1}` and `{item.project_item_index1}`; both join ADR-024 § 2's placeholder list.
3. **Show and layout, always.** Every `index_select_slide` carries `showId` and `layoutId`. Both come from the last `get_output` (`slide.id`, `slide.layout`), read on `connect` and refreshed by every verify. A jump body without both is never sent.
4. **One jump primitive.** `index_select_slide` is the only way voxxa goes to a slide that is not the next one. `id_select_group` and `name_select_slide` are never used for following. `docs/INTEGRATIONS.md` names `id_select_group` a Companion convenience only.
5. **Song select via `name_select_show` (50 § 4, 52 V8b).** The driver trait gains `select_presentation(title) -> Result<bool, CtlError>`, default `Ok(false)`. FreeShow sends `name_select_show {value}`, then `get_output`. It returns `Ok(true)` only if the returned show id equals the id that `get_shows` mapped to that title at connect (normalised as `presentation_is` does). Dispatch calls it on `new_song && !operator`, before `presentation_is`. Anything but `Ok(true)` keeps today's blank-and-ask. `presentation_is` also accepts an id match when the service item carries `{item.show_id}`. The title is the setlist's own string and is never logged. Whether `name_select_show` matches exactly or fuzzily: I don't know (42 does not say). The id check makes either safe. (inferred)
6. **State from ids only.** `can_query_state: true`. `current_state()` reads `get_output` and keeps `slide.id`, `slide.layout` and `slide.index` (0-based). It drops everything else. `PresenterState` gains `presentation_id: Option<String>`.
7. **Fire, then verify once.** After every move, one `get_output` re-read. Its timeout is 3 × the verify-latency median V3e measured, capped at 750 ms. Compare `(id, layout, index)` with the target.
   - Match: done.
   - Mismatch or timeout: set `position_uncertain` (V8-lite's one setter, ADR-025 § 2); hold (no automatic move until the operator confirms); amber "FreeShow shows a different slide." No retry (ADR-025 § 6).
   - Log one `trace` line, `kind: verify`, `{t_sent, t_ack, result, show_ix, layout_ix, index}`, numbers only.
   - Read-back changes dispatch (the hold), never the conductor, so replay stays byte-identical.
8. **Credential.** `PresenterConfig.password` goes in `Authorization: Bearer` on every request. It never goes in a URL, a query string, a body field, a log line, a help bundle, a kit file or a church file. `connect` logs the base URL only. 401 → `CtlError::AuthRequired`. The comment at `freeshow.rs:5` is corrected.
9. **Open API refused (FA-55 a).** On `connect`, one probe without the header. Anything but 401 → red "FreeShow's API has no password. Set one in Settings → Connections." and no connection, unless the setting `require_password` (default true) is set to false. `require_password` replaces the earlier draft names `allow_open_api` and `refuse_open_api` (ruling after integration). voxxa talks to FreeShow on `127.0.0.1`; a non-loopback host keeps ADR-025 § 6's amber. **voxxa's own HTTP API stays on `127.0.0.1`** and is never widened for a stage button (R8, FA-58). FreeShow's output lock is the external panic path. The firewall rule (5505/5506 from loopback only) is F5's; the "answers only on loopback" self-probe is ADR-034's.
10. **Never-call list**, compiled into the profile beside the existing write refusals: `get_output_slide_text`, `get_output_group_name`, `get_plain_text`, `get_slide`, `start_slide_recording`, `send_rest_command`, `emit_action`, `set_plain_text`. Any other getter found to return slide text joins the list in the same PR. So no lyric text enters voxxa's memory, logs or bundles (ADR-007/011, cited by 42). Q12's plan to read `get_output_slide_text` is struck.
11. **`get_show` and `get_shows` only through an id-and-name parser (ruling after integration, 2026-11-26).** Either reply may carry slide text, so they are called only through one parser that keeps each show's id and name and discards the rest of the body before anything is logged or stored; a named test (`freeshow_show_list_body_never_written`) asserts the raw body never reaches a log sink, a file or a bundle. `get_output_slide_text` and `get_output_group_name` stay never-call. V3e's probe records which getters carry text (key names only, values redacted); a getter found to carry text and not covered by the parser joins the never-call list.
12. **`voxxad --watch freeshow`** (only with `--shadow`): read-only; polls `current_state()` every 250 ms; writes `trace kind: screen {t, song, slide}` on a change; never sends a non-getter. It feeds V2's `--truth-from-screen` and V15's shadow truth for the Correct bar (FA-66).

### B. Kit rules for followed songs (v1)
13. **Unique layouts (FA-56).** A followed song's FreeShow layout lists every slide once; the parts map does the repeating. So `app_index` equals voxxa's slide index, and `songcheck`'s `presenter_slides` count applies to FreeShow. A per-song occurrence resolver for repeating layouts comes later, with Q12 (FA-56 b). **Amended 2026-11-27 (FA-56 as answered):** the object of record is a per-song **layout file in JSON** in the church folder (sections, slide order, group labels, slide counts, the app's ids; never words). FreeShow: its show file is JSON already; voxxa derives the layout file from the show through Decision 11's id-and-name parser in V17, and `songcheck` compares it with the kit `.txt`. Presenter: no API and no documented export, so the layout file comes from the kit `.txt` import (ADR-016 sections and slide splits), checked against Presenter's slide count at F4; F4 also records whether Presenter's data folder holds a readable song store (yes/no only). Unique layouts stay the v1 rule because the jump needs an index; the layout file is what the later resolver reads.
14. **Text transition stays at FreeShow's 500 ms default (FA-57, answered 2026-11-27: the fade is comfortable reading time).** F5 records the default (FS12); nothing is set. The verify read-back is unaffected by the fade; the visible change lands about 0.25 s after the command, which the eval notes beside the on-time bar.
15. **Version pin (FA-51).** Stable channel only, auto-update off, the version in `church-profile.json`, checked by the health check (ADR-034). Upgrades happen on a weekday. The V8b stub is the regression net.

### C. Tier A/B policy (FA-47)
16. **Tier A** = a driver with a deterministic jump and a read-back. Today that is FreeShow under this ADR. A song that passed its promotion pass (51 § 2.7) may run autonomously ("voxxa moves it"). Only Tier A can pass the no-minder gate (R10; ADR-034).
17. **Tier B** = keys only (WorshipTools Presenter). Assisted, with suggestion cards. voxxa steps forward in plan with Right only, at most 3 steps, when the target is ahead (ADR-031 § 7). For any other move it shows a suggestion card, logs the target, sends nothing and stops automatic moves for that song. **Left bursts are refused in code** (FA-44 (a)–(c) dropped). Tier B stays the universal fallback; `KeystrokeDriver` and the WorshipTools profile do not change.
18. **One interface.** The follower emits voxxa's slide id. Dispatch chooses per driver from `capabilities().can_goto_slide`. No follower code knows a driver (V11). `presenter.profile` in the church profile selects the path (ADR-024 § 1).

### D. Named tests (V8b; CI, no FreeShow, no model)
An in-process `std::net::TcpListener` stub in `src-tauri/tests/common/` (no new crate) implements the source-derived contract: top-level fields; `index--`; 204 for non-getters and unknown actions; `{action, data}` for getter replies; 401 JSON when a password is set and the header is missing; a lock flag; a configurable reply delay. Its fixture bodies come from V3e's redacted records. The eight named tests (50 § 4):

| # | Test | Asserts |
|---|---|---|
| 1 | `freeshow_goto_sends_top_level_fields_and_one_based_index` | `goto_slide(3)` posts top-level `action`, `showId`, `layoutId` and `index: 4`, and no `data` key |
| 2 | `freeshow_readback_index_is_zero_based` | after that move the stub's `get_output` holds `index: 3`, and `current_state()` reports 3 |
| 3 | `freeshow_goto_verifies_and_flags_uncertain_on_mismatch` | a stub that lands elsewhere (or is locked) → `position_uncertain` set; the next automatic Goto sends nothing |
| 4 | `freeshow_goto_always_names_show_and_layout` | no `index_select_slide` body lacks `showId` or `layoutId` |
| 5 | `freeshow_never_calls_text_getters` | over a scripted session the stub receives no action on the never-call list |
| 6 | `freeshow_token_only_in_authorization_header_and_never_logged` | the token is only in `Authorization`; the captured log sink, URLs and bodies never hold it |
| 7 | `freeshow_open_api_is_refused_by_default` | a stub with no password → connect refused with the red sentence; with `require_password` set to false it connects |
| 8 | `freeshow_refused_actions_never_sent` | refused write actions never reach the stub |

52's V8b prompt adds two more: `freeshow_goto_flags_uncertain_on_timeout_without_retry`, and `freeshow_select_presentation_then_goto` in `tests/dispatch.rs` (expects `["select", "goto 0", "unblank"]`). **All ten are required** (ruling after integration: the adjudication's "8" was a floor; the ten are 52's V8b list, nine driver tests plus `freeshow_select_presentation_then_goto` in `tests/dispatch.rs`); the id-and-name parser test in Decision 11 is added to them. `capabilities_match_implementations` still passes. The driver stays UNVERIFIED live until the founder's contract checklist against the real install matches the stub, or names the differences and the stub is corrected in the same PR (ADR-024 § 5).

**Budget (ADR-028 as amended in V7):** presenters row ≤ 150 (`freeshow.rs` ≤ 400); wiring row ≤ 30; voxxad ≤ 10 (`--watch`). Stop and report at any overrun.

## Alternatives Considered
| Alternative | Pros | Cons | Why Not |
|---|---|---|---|
| Keep Presenter, keys only | nothing to migrate; volunteers know it | no jump and no read-back; going back flashes every slide in between; voxxa sees manual moves only through its hotkeys (R10); the WorshipTools API request has had no staff reply since 2024-07 (cited, 51) | kept as Tier B; never autonomy-eligible; FA-53's MIDI check is its last chance |
| Holyrics driver | named by the research report as a fallback | closed-source freeware with its own bus factor (cited, 41 via 52 § 3) | only on a V3e no-go where FS3 and FS5 fail, so that no deterministic FreeShow jump exists (R17) |
| `id_select_group` as the jump | one call for "chorus again" | global key, next match, wrapping; cannot reach verse 3; ignores label-style keys from imports (measured, 42) | not deterministic (C3); Companion convenience only |
| ProPresenter | not examined | 42, 50, 51 and 52 hold no facts on its jump or read-back; I don't know them | no evidence in this round; FreeShow is free, GPL, on Windows and source-checked (50 § 1); Q13's ProPresenter items stay unchanged |
| Song select by `index_select_project_item` (42 § 2) | 1-based, no names | needs the FreeShow project order to match voxxa's setlist (inferred) | 50 § 4 rules `name_select_show`, verified by id |
| Amber only on an open API (42's option b) | fewer refusals at setup | any device on the church Wi-Fi could move slides; `cors: *` | FA-55 (a): refuse by default |
| Occurrence resolver now (FA-56 b) | any layout works | needs `get_show_layout`; whether it carries text is unverified (42) | later, with Q12 |

## Consequences
### Positive
- voxxa can show any slide (a chorus again, verse 3) with one call, and knows whether it arrived.
- Song changes on FreeShow select the song instead of blanking (C17 closed).
- Read-back gives operator-timed truth for the Correct bar (FA-66), which cuts tapping.
- No lyric text enters voxxa from FreeShow.
### Negative
- Autonomy needs the church to run FreeShow (ADR-035). Presenter churches stay assisted.
- Unique layouts are a kit rule the volunteers must keep. Repeating layouts are not followable in v1.
- Every move costs one extra request. Any doubt holds voxxa, so a slow renderer means more holds.
- About 30 more lines in the wiring row and up to 150 in presenters.
### Risks
- **W1 is settled (C1), not open.** The residual risk is that the stable release differs from `c84386f3`. Mitigation: V3e's FS3–FS5 on the pinned stable; if a fact differs, the stub is corrected in the same PR.
- **The API binds all interfaces with an optional password (C4).** Mitigation: F5 sets the password and the loopback firewall; voxxa refuses an open API (FA-55); bearer header only; ADR-034's loopback self-probe.
- **FreeShow version drift** (weekly betas, an auto-updater, `// BC` markers). Mitigation: pin per FA-51; the health check compares the version; upgrades on a weekday; the stub is the regression net.
- **No error channel; renderer stall.** A stall past the timeout holds voxxa (42 § 6.6). Holding is the safe failure; SLO S8 (51 § 2.7) counts it.
- **Output lock.** Every select becomes a no-op; the verify catches it and voxxa holds. This is also the external panic path (R8).
- **GPU.** FreeShow is Electron with GPU-composited output; its VRAM beside Whisper on the GTX 960 is unmeasured (C7). V3e FS8 measures it; ADR-034 reds above 1,800 MiB.
- **`get_show`/`get_shows` may carry slide text** (Decision 11): only the id-and-name parser touches them; V3e records which getters carry text.

## Dependencies
- Depends on: ADR-019 (§ 2 front-window guard, which does not apply to HTTP moves, 51 § 2.5; WorshipTools stays blind), ADR-024 § 2 and § 5, ADR-025 § 2, § 6 and § 7, ADR-028 (rows as amended in V7), ADR-031 § 7; V7 (wiring row), V8-lite (`position_uncertain`), V3e go (body shape, index base, verify latency, `get_output` key paths), F5.
- Amends: ADR-024 § 2, ADR-025 § 2 and § 6, ADR-031 § 7. 52's V8b also asks for an ADR-006 note on the tiers; it should point here.
- Constrains: V11 (Tier A/B jump interface), Q12 (ids-only read-back, id match), Q13 (the stub becomes the FreeShow fixture; W1 leaves its list), V2 `--truth-from-screen`, F7, ADR-034 (Slides-app row), ADR-035.
- Founder decisions: FA-44 (answered), FA-47, FA-49, FA-51, FA-55, FA-56, FA-57.

## Review trigger
V3e records a no-go, or a fact that differs from § Context; a FreeShow upgrade fails the stub; any `position_uncertain` on a dark Sunday (FA-48 needs 0); a getter is found to return slide text; a church needs repeating layouts; WorshipTools ships a jump and a read-back (Presenter could then become Tier A).
