# 53 — Skills and context files for round 3: where they live and what each holds (2026-11-26)

Writer's deliverable for the "skills and context files" part of the round-2 brief. Sources: `40-brief.md` (hard
rules), `50-adjudication.md` (canonical step names, R1..R17, FA-46..72), the "skills and context" sections of
reviews 41 § 4, 42 § 6.4, 43 § 6.4, 44 § 6, 45 § 6, 46 § 9, 47 § 6.4, the staging kit `out/kit/` and the
read-only checkout `C:\dev\r2-review` (origin/main a5f7e2e). Step names below are the adjudication's. Nothing here
is committed; § 4 says what waits for the founder's go.

Three facts about the checkout shape everything below (measured, `ls` on a5f7e2e):

- There is **no `CLAUDE.md` and no `.claude/`** in the repository today. The standing safety block lives in
  `.10x/handoff.md` (§ "User decisions for Q10", line 12) and in each prompt. Both get created in round 3.
- There is **no `kit/`** in the repository. The kit is the staging draft `out/kit/` (README, 8 prompts, 5 skills,
  19 templates, 6 binding profiles) until plan step V5 commits it as `kit-v0.1.0`. Until then "`kit/`" below means
  the staging copy, edited in place and committed by V5 or a later kit tag.
- `.10x/` already has the shape the records need: `adrs/001..021`, `specs/`, `reviews/` (ponytail and ticket files
  named `YYYY-MM-DD-<lane>-<step>.md`), `decisions/<role>/church-autoadvance.md` for twelve roles (architect,
  cto, dba, devops, engineering-manager, product-manager, qa, sde, security, senior-engineer, sre, staff-engineer),
  `status.md`, `handoff.md`, `handoff-local-windows.md`. Nothing new in layout is needed there; only files.

Reviewer disagreements on names, resolved here (the adjudication did not rule on them): the minder card file is
`kit/templates/minder-role-card.md` (44, 45; not `minder-card.md` of 41/43/46/47); FreeShow test-card rows go
into `presenter-test-card.md` as FS rows (FS1-FS20, per 52) rather than a separate file; the
research notes go under `.10x/specs/` (§ 3.5), not `docs/research/` (45) or `.10x/research/` (44, 46); 44's
proposed skill `voxxa-minder-session` is absorbed by 45's `voxxa-minder-training` (R15 merges V5d into V18).
**ADR numbering:** 42, 43 and 45 each named their ADR "ADR-032". The adjudication's step table gives ADR-032 to V8b
(the FreeShow driver), and R5 reserves ADR-033 for `align`. So: **ADR-032 FreeShow jump driver (V8b), ADR-033
reserved for the aligner (V3f pass only), ADR-034 Sunday readiness (V13s), ADR-035 live output migration and
rollback (V17, Proposed until FA-48)**. The orchestrator should confirm this in round 3's first prompt.

## 1. The four homes, and the two places outside the repo

| Home | What belongs there | The rule | Exists today |
|---|---|---|---|
| **Public `kit/`** (church-facing) | `README.md`, `private-folder.md`, `prompts/01..10`, `skills/<name>/SKILL.md`, `templates/` (checklists, cards, profile and layout templates, `bindings/*.json`, consent form, examples), `tools/` (songcheck, kit_check) | Made-up examples and public-domain titles only. No number measured at a real church, no host or church name, no person's name, no password, no path under a user profile. Every file passes `kit_check.py` (V5) and the lane scan (§ 4.4). Copy templates **out of** the kit into a church folder; never copy anything in. Volunteer-facing text in plain language, no "AI" (FA-70). | staging `out/kit/` only |
| **Repo `.10x/`** (process memory) | `adrs/`, `specs/` (strategy outputs, the V-step prompt pack addendum, this round's research report and notes, this layout), `reviews/` (ponytail records, tickets, step records, the FreeShow source notes), `decisions/<role>/church-autoadvance.md`, `status.md`, `handoff.md` | Numbers only; `file:line` references; part labels V1/C/B; take ids T1, live1..live4, F1..F4; the private song is "P1"; leaders are L1/L2; "the booth PC" and "the pilot church", never names. Never a lyric or a heard word. Every record says who measured what and when. | yes |
| **Repo `.claude/` and root `CLAUDE.md`** (agent-facing) | `.claude/skills/voxxa-freeshow-probe/`, `.claude/skills/voxxa-lab-eval/`, root `CLAUDE.md` with the sections in § 3.1 | Only what an agent working on **this repository** must know and a church never needs. Short sections that point into `.10x/` for the reasoning. Kit skills are **not** copied here on the development checkout; agents load them by path from `kit/skills/`. (The copy step in `kit/README.md` § "Install the skills" is for a church's booth checkout.) | no; created in V7 (`CLAUDE.md`) and V3e (first skill) |
| **Private church folder** `<church_root>` (pilot: `C:\dev\voxxa-church-private\pilot\`) and the lab `C:\dev\voxxa-church-private\lab\` | Everything that carries church data: songs and setlists, the FreeShow test show and its show/layout ids per service item (`services\*.service.json` `app.freeshow`), probe outputs and their raw replies, rehearsal captures and OLTW reference takes, the lab `.py` files (`parts.py`, `predict.py`, `live.py`, `flows.py`, `evalparts.py`) and their pytest, `fixtures\manifest.json`, `recordings\`, `sessions\`, `practice\<song_fp>.json`, `manifests\service-<date>.json` (frozen at the last rehearsal), `reports\service-<date>.json`, `reports\trend.csv`, `reports\review-<date>.md`, the repertoire inventory and its signed sign-off, the filled rota, `consent\`, the filled test-card answers `presenter-test-card-freeshow-<date>.md`, the FreeShow API password (founder's environment only), the migration dates and team note as sent, incident reports, and `scan-terms.txt` (§ 4.4) | Never a git repository, never synced, never e-mailed. Agents open only the file a prompt names, and only the three lab files unless a prompt widens it. Counts, indices and labels are the only things that leave. The founder runs anything that touches the real app, the mic or a recording. | yes (ADR-029) |
| **User-level memory** (the Claude Code memory folder, outside every repo): the `voxxa-*` notes (ponytail lane, repo layout, unattended runs, skill selection) | The founder's standing instructions to agents across sessions. Agents read them at session start; they never edit them unless asked, and never copy their content into the repo. When a rule in them hardens into a project rule, it is restated in `CLAUDE.md` and the memory note points at the section. | yes |
| **Launcher** `~/bin/voxxa-claude.sh` (KEEP/OFF plugin list) | The founder's launcher decides which plugins a lane sees. Agents never edit it. `CLAUDE.md` § "Plugins" mirrors the KEEP list as plain text so a lane started without the launcher knows what it is missing and stops rather than improvising. | yes |

The one-line placement test: *could a second church need it* → `kit/`; *does it explain why voxxa is the way it
is or what a step measured* → `.10x/`; *does only an agent on this repository need it* → `.claude/`; *does it name a
person, a place, a non-public-domain song, a recording, an id from a real install or a secret* → private.

## 2. Skills

| Skill | Home | New / extended | Steps that use it | What it must contain | What it must never contain |
|---|---|---|---|---|---|
| `voxxa-church-setup` | `kit/skills/` | **extend** (V13s docs, V17, V18, V15) | 01, 06, 07, 09 (via job table), F5, F6, F7, V15 | A tier table (Tier A: jump + read-back, autonomy-eligible; Tier B: keys only, suggestion cards; FA-47). **FreeShow rows:** install the stable channel beside Presenter, auto-update off, `freeshow_version` in `church-profile.json` checked by the health check (FA-51); API password set and 5505/5506 firewalled to loopback (FA-55); stage 1 = no output assigned; the output lock is the external panic path; the unique-layout rule for followed songs (FA-56). **Health check** (V13s): the six rows and their colours, "Ready, with a note" may start Songs, only a red slides-app row blocks (FA-62), the two keys HOLD F15 and STOP FOLLOWING F20 plus Back one F14 (FA-58), when to press which, the incident template. **Sunday policy** (FA-67): no RDP Sat 18:00 to Sun 14:00, Windows Update active hours cover services and auto-restart off, voxxa started and armed by a person, health check within 30 min of the service, Thursday 18:00 rollback rule during double-build Sundays, Presenter kept a month. Job-table rows for prompts 09 and 10. The after-service review reads `reports\service-<date>.json` and states the SLO table and the trend sentence (43). | A password, token or port other than the documented defaults; any path inside FreeShow's data folder; a lyric or heard word; a church or host name; the word "AI" in anything a volunteer reads; an instruction for the agent to press a key or start listening. |
| `voxxa-minder-training` | `kit/skills/` | **new** (V18; role card v0 ships as docs in week 1) | 10, F6, V15 acceptance ("two volunteers passed the card") | Wraps `prompts/10-minder-training.md`. The role card in two versions (v0 Presenter: hotkeys and "put the song live" amber cards; v1 FreeShow: read-back, demotion visible), the ten-minute session with its four "done when" lines (44 § 2.6), the monthly drill with six fault cards and one manual song a month (F6), rota rules (>= 3 minders from the slide team, a 3-month veteran trains the next, the founder is never the Sunday minder from month two; FA-69), the worship-leader card (three-state stage strip, HOLD on the stand), the congregation notice template with "the pastor decides the wording" (FA-70), the badge words verbatim "voxxa moves it / voxxa suggests / You move it" and the demotion rule (FA-61), the post-STOP sentence ("I move the slides; voxxa still listens"), and the drill replay recipe on a **public-domain** fixture through `voxxad --stdin-audio ... --shadow` with a deliberately mis-ordered setlist (45). Same safety block as the other kit skills. | Names of volunteers or leaders (the rota template has blanks; the filled rota is private); private fixture or take names; "AI"; any instruction that starts the installed app or opens the mic; numbers from the pilot church. |
| `voxxa-practice-session` | `kit/skills/` | **extend** (V12; V18 edits to prompt 05) | 05, V12, V15 | `--learn` = the parts-map run non-causally over a rehearsal log; `status` {not_rehearsed, proposed, promoted, demoted} and what each means on Sunday; the `--review` keyboard flow with labels and counts only (44 § 2.3), the review-hold `why` values (ADR-031 § 4), `--confirm`; >= 3 confirmed runs before learned handicaps (FA-64); leader alias L1/L2, never a name; any manual move demotes the song to suggest for the rest of the song (FA-61); what "stop" means for the week (cold follow on Sunday, nothing learned); who runs the review (the minder on rota, about three minutes for six songs); the data stays in `<church_root>\practice\` and `sessions\`. | A lyric, heard word or transcript; the contents of a practice file beyond labels and counts; a leader's name; learning on Sunday; `boundary = plan` (C8, FA-64). |
| `voxxa-layout-author` | `kit/skills/` | **extend** (V8b kit rows; V17) | 03, 04, 09, V8b kit tasks, F5 | **FreeShow unique-layout rule:** every slide appears once in a followed song's layout; a repeated part is a second slide (FA-56; later a per-song occurrence resolver with Q12). **Binding fields:** `show_id`, `layout_id`, `project_item_index`, the 1-based `{app_slide1}` placeholder (C2), "a jump without `showId`/`layoutId` targets the editor's active show, not the output", `id_select_group` is a global group key and cannot address verse 3 (C3), `index_select_slide` with explicit show and layout is the only deterministic jump. Test-card rows T13..T17 and how to record answers (counts and key names only). The who-moves-it table with tiers. Text transition 0 ms for the followed output style (FA-57). The stage display is never in the stream (FA-29). | Editing a binding profile (they are compiled into voxxa); a password; a FreeShow data-folder path; show or layout **ids from the pilot install** (they go into the private `service.json`); a lyric. |
| `voxxa-song-intake` | `kit/skills/` | small extension (V5a sections; V17 inventory step) | 02, 09 | The `.txt` section markers V5a adds; the repertoire inventory step: counts from `songs.csv` licence columns (licensed source / none / legacy CCLI) that the worship leader signs before any migration weekend (FA-68); first ten re-imported songs = most-sung licensed songs. | Reading any `songs\*.txt`; titles beyond public domain in anything that leaves the PC. |
| `voxxa-wav-intake` | `kit/skills/` | small extension (V2) | 08, 07 scored Sundays | `--service-report` (numbers-only `service-<date>.json` + a `trend.csv` row), `--truth-from-screen` (read-back as operator-timed truth for the Correct bar only, never Tight; FA-66), the new columns `early_300`, `early_1000`, `tight_300`, `wrong_slide_s`, signed error. | Opening audio or truth content; renaming the eight baseline columns (R11). |
| `voxxa-freeshow-probe` | `.claude/skills/` | **new** (V3e) | V3e, F5, V8b (tests and `--watch`), F7 dark Sundays, Q12/Q13 fixture | The numbers-only probe script rules: the script POSTs to `127.0.0.1:5506` with a bearer header whose token comes from an environment variable (never an argument, never a file in any repo), prints only HTTP status, latency ms, indices, ids and counts, and drops every other field before printing (redaction at the parse step, not the print step). The probe matrix: body top-level vs nested under `data` (C1); send index 1, read back 0 (C2); with and without `showId`/`layoutId`; 204 means "accepted", not "done", so always one re-read; p50/p95 over 20 calls; the "no password" case must be refused by the script itself (FA-55 (a)). Also VRAM and CPU with Presenter and FreeShow open and a video playing (`nvidia-smi` numbers; C7), PD-hymn import counts, the Presenter MIDI check (FA-53), the one non-projected parts-map run (FreeShow with no output assigned). **Never-call list** pinned in the skill and mirrored in `bindings/freeshow.json` (`refused` plus a new `never_read` list): every action whose reply carries slide text or show names in clear (`get_output_slide_text` and its kin) and every write action in `refused` (`create_show`, `set_plain_text`, `set_show`, ...); `get_shows` is used to count, never to print names. Output goes to `.10x/reviews/YYYY-MM-DD-v3e-freeshow.md` as tables; raw replies stay in `<church_root>\reports\probe-<date>\`. | The token or password in any log or record; a raw reply pasted anywhere; a lyric; show names; running while an output is assigned; running on a Sunday; pressing keys; opening FreeShow's data folder; any write action. |
| `voxxa-lab-eval` | `.claude/skills/` | **new** (V3d; used from V3b on) | V3b, V3d, V3f, V6, V11 (replay byte-identical check), V12 (`--learn` validation) | How to run `evalparts.py` and `flows.py` on the take set (T1, live1..live4, F1..F4, the detour, spoken-aside and two-singer takes) by alias paths `<lab>` and `<takes>`; the column set (right / wrong / early / wrong-slide s; within 0.5 / 1.0 / 1.5 s per take; `tight_300`, `early_300`, `early_1000`, median delay); the one-switch-at-a-time before/after discipline with the 7-take table (R3, C8 as the worked example); which take is tune and which is held-out and that held-out is never re-tuned on; the R14 must-fixes as a checklist (relabel-to-`jump` bug, `elim` behind a flag, audio-time stamps, stdin queue depth, floor warm-up 1 s, pytest from ADR text); how to paste tables into `.10x/reviews/` with take ids and part labels and mark each number (measured); the lab-to-port line-count table for `decisions/sde/`. | Printing a heard word or transcript; opening a `.wav` other than passing its path to a script; the private song's title (it is "P1"); citing live4 for the break gate (R3, FA-63); tuning on held-out takes; any product code (R5). |
| `10x-Team:10x-team` and the role skills | plugin | installed | every step | The step protocol: roles per prompt, decision records per role, the comparator after two lanes. | — |
| ponytail lane (`full` + `ponytail-audit`; `full` + `ponytail-review`) | plugin | installed | `audit` on discovery, lab and close steps (V3e, V3f, V15, F-steps' records); `review` on build steps (V7, V2, V8b, V11, V12, V13a, V13s, V14, V17, V18); a comparator after every build step | Runs independently of the 10x lane from the same base commit; its record goes to `.10x/reviews/YYYY-MM-DD-ponytail-<step>.md`. | Reading the 10x lane's branch before its own record is written. |
| `workflow-authoring` | built-in | installed | any unattended lane | The Workflow script that drives a Q/V step unattended, written per the memory note's safety rules and blockers. | A script that touches the mic, Presenter, FreeShow or the installed app. |
| `superpowers:test-driven-development` | plugin | installed | V2, V8b, V11, V12, V13a, V13s, V14 (GUI tests) | Tests first; the 8 named V8b tests; the `TcpListener` stub; replay byte-identical in V11. | — |
| `superpowers:systematic-debugging` | plugin | installed | CI red on any step; V11 replay mismatch; V3 decoder diagnosis | — | — |
| `security-review` (built-in) and the `backend-design:security-reviewer` agent | built-in / plugin | installed | close of V8b (`current()`, token handling, open-API refusal), V9, V13s (`/panic` route, watchdog), V15; the V3e record before commit | — | — |
| `supermatt:dependency-vetting` | plugin | installed | V8b (any new crate), V7 (none expected), the FreeShow version pin (F5) | — | — |
| `writing-skills:plain-language` | plugin | installed | every volunteer-facing file: cards, runbooks, health-check sentences (V10, V14), prompts 09/10, the notice, the team note | — | — |
| `ui-consistency:verifying` (+ `ui-consistency:accessibility`) | plugin | installed | **V14 only** (and the V10 mock review) | Contrast fixes, the 420 px rule, the three badge states. | Its hooks on any other step (memory: the 10x + ponytail process beats the beads/iceberg/ui-consistency hooks). |

**The launcher's KEEP list must include:** `10x-Team`, the ponytail plugin, `superpowers`, `writing-skills`,
`supermatt` (for `dependency-vetting`), `backend-design` (for the security-reviewer agent), and the two built-ins
`security-review` and `workflow-authoring` must not be disabled. `ui-consistency` is KEEP but its hooks run only in
the V14 lane (turned on for that lane, off again after). **OFF on every unattended Q/V lane:** `beads-superpowers`,
`iceberg`, `agentic-council`, every connector that reaches church data or an outside service (Planning Center,
Google Drive, Gmail, Slack, Notion, Canva, Wix), and `desktop-commander` (it runs processes and file operations
outside the permission UI; opinion). The kit's `templates/agent-settings.example.md` already says "no Planning
Center, Drive or Gmail connectors during setup"; the launcher enforces the same for development lanes.

## 3. Context files

### 3.1 Root `CLAUDE.md` (created in V7's PR; each section at most 15 lines, pointing into `.10x/` for the reasons)

| Section | Lines it must carry | Added in |
|---|---|---|
| Read first | `.10x/handoff.md`, `.10x/status.md`, `.10x/specs/2026-11-26-skills-and-context-layout.md` (this file), the prompt you are running. | V7 |
| Sunday safety lines | The standing block (adjudication § 6): never open the mic unattended; never press keys in Presenter or FreeShow; never start the installed app (`com.voxxa.app`) or touch `%LOCALAPPDATA%\Voxxa\`; never read or print lyrics or heard words; never open the private folders (named by path) beyond what the prompt names; stop own processes by PID; media guard stays required; numbers only in anything that leaves the PC. Plus 43's lines: never run the health check or the hotkeys against the real app; use the canary fixture names; no RDP in the Sunday window (FA-67). | V7 |
| FreeShow driver facts | Actions read fields at the **top level of the body** (C1). Index is **1-based on the wire, 0-based in read-back** (C2). **Always send `showId` and `layoutId`**, or the jump targets the editor's active show. **Never call a text getter**; `get_output` is parsed to `slide.id`, `slide.layout`, `slide.index` and everything else is dropped before logging. **Bearer header only**, token from the environment, never logged, never bundled. Refuse an API that answers without a password unless `require_password` is set to false (FA-55). voxxa's own API stays on 127.0.0.1. `id_select_group` is a global group key, not a slide address (C3). 204 is "accepted"; verify with one re-read. Never read FreeShow's data folder. FreeShow is pinned to the stable channel (FA-51). | V3e (facts), V8b (final) |
| Church kit work rules | Never touch Presenter or FreeShow; never read `<church_root>` content; copy out of the kit, never in; made-up examples and public-domain titles only; numbers only; lower-case "voxxa"; one plain sentence, the next action second; no "AI" in volunteer text (FA-70); no numbers for the minder on screen (44). | V5 / V17 |
| Budget rows | Never raise a cap; a new module gets a new row through an ADR; "everything else" has 1 line left (`handoff.md` row table); the rows V7 adds: `onset` 750, `input chain` 600, `health check (src/health/)` 450 (the estimates 250 / ~200 / 150 × 3, FA-54 answered 2026-11-27; FA-74 asks whether the rule extends to the follower, practice, layout, Run and kit-tools rows), `vad/` moves to the recognizer row, `align` 1,200 only on an R5 pass (FA-54); V11 ports one module `aligner/follower/parts.rs` replacing `aligner/evidence/` under one switch (R12). | V7 |
| GUI testing | Mock first (`tests/gui/mock-tauri.js`, `e2e.mjs`), the guarded scratch build second (scratch identifier, CDP port, `--no-bundle`, scratch `VOXXA_HELP_DIR`), never the real app, never a real display for the stage window (44). | V10 / V14 |
| Plugins | The KEEP/OFF list of § 2 as plain text, with "stop and say so if a KEEP plugin is missing". | V7 |
| JSONL and records | One `trace` line type with `kind` (FA-65); every record marks (measured) / (cited) / (inferred) / (opinion); the private song is "P1", leaders L1/L2, takes by id. | V7 |

### 3.2 `.10x/` records per step

| Step | ADRs | `reviews/` | `decisions/<role>/church-autoadvance.md` entry | `status.md` / `handoff.md` |
|---|---|---|---|---|
| V7 | ADR-028 amendment (rows, `vad/` move; FA-54); ADR-001/003 amendment (`trace` line; FA-65); ADR-005 amendment (model weights under the dependency rule; FA-50) | `YYYY-MM-DD-ponytail-v7.md`, `-v7-tickets.md` | engineering-manager (row decisions), staff-engineer (`trace` shape) | handoff row table updated; status "V7 merged" |
| V3e / F5 | none (facts only); the ADR-006 "two tiers" dated amendment is **drafted** here and accepted in V8b | `YYYY-MM-DD-v3e-freeshow.md` (the probe tables, VRAM/CPU, import counts, MIDI answer, the non-projected run), `2026-11-26-freeshow-source-notes.md` (42 § 1 facts with commit and `file:line`; FreeShow is GPL and public), `-ponytail-v3e.md` (audit) | sde (what the probe measured), architect (body/index/ids facts), security (API posture, token handling) | handoff: "V3e go / no-go" line and the latency number V8b uses |
| V3d | ADR-031 amendment (R3 break-gate lab switch; § 7 Tier A/B jump interface; FA-63) | `YYYY-MM-DD-v3d-port-plan.md` (before/after tables per take, R14 checklist ticked, timing columns), `-ponytail-v3d.md` | sde (lab-to-port line counts and the row proposal, 47) | handoff: V11 pointed at ADR-030/031 and the two port plans |
| V8b | **ADR-032 FreeShow jump driver** (fire-then-verify, ids-only read-back, bearer only, refuse open API, never-call list); ADR-006 amendment accepted | `-ponytail-v8b.md`, `-v8b-tickets.md`, the 8 named tests listed with results | architect (driver shape), security (`current()` review), qa (the TcpListener stub as the Q12/Q13 fixture) | handoff: `can_query_state: true` for FreeShow; `tests/dispatch.rs:289-305` pin changed |
| V13a | ADR-025 § 7 amendment (F14/F15/F20 and the four routes; no Escape; FA-58) | `-ponytail-v13a.md` | sde | — |
| V13s | **ADR-034 Sunday readiness** (six rows, watchdog with four signals, hold/panic post-state, manifest frozen at last rehearsal, RDP check, device pinned by name); ADR-022 amendment (the second, no-minder gate; FA-52); ADR-029 amendment (`manifests\`, `reports\service-*.json`, `trend.csv`, `consent\`, `rota` in the private layout) | `-ponytail-v13s.md`; the SLO, alert and failure-mode tables (43 §§ 2-3) | **sre** (SLO table, alert table, failure-mode table; 43), security (`/panic` route) | handoff: the Sunday window rules |
| V10 / V14 | ADR-011 note (heard-words line removed; FA-60); ADR-017 note (badge, three states) | `-ponytail-v14.md`; the V10 mock record with every approved sentence word for word (FA-62) | product-manager (approved words), qa (GUI test results, contrast numbers) | — |
| V12 | ADR-023 § 4 amendment (status field, leader alias, >= 3 runs; FA-64) | `-ponytail-v12.md` | dba (data model), sde | — |
| V17 | **ADR-035 live output migration and rollback** (Proposed; per-service switch, double-build Sundays, Thursday 18:00 rule, Presenter kept a month; Accepted on FA-48) | `-ponytail-v17.md`; the migration record with counts only | product-manager (the church-side plan), security (privacy of the inventory) | status: the FA-48 decision and date |
| V18 | none | `-ponytail-v18.md`; the drill record template | product-manager | status: rota count (number of minders, no names) |
| V15 | ADR-022/034 amendments if gates moved | `-ponytail-v15.md`, the acceptance-in-occurrences table | cto, sre | status: "V increment closed", founder minutes per scored Sunday |

### 3.3 Kit templates to add or change (all public, scanned before commit)

| File | Add / change | Step |
|---|---|---|
| `templates/bindings/freeshow.json` | Replace the W1 `note` with the settled facts; `transport.http.auth: "bearer"` stays, add `require_password: true`; `goto.item` uses `{app_slide1}` (1-based) and always `showId`/`layoutId`; add `never_read` (text getters); `readback.current` documents the three kept fields and the one re-read; drop `[UNVERIFIED]` where V3e measured | V3e (note), V8b (final) |
| `templates/presenter-test-card.md` | Rows **T13** index base (send 1, read back 0), **T14** group-id semantics (which slide `id_select_group` chose), **T15** stage display shown and not in the stream (FA-29), **T16** API answers without a password? (must be "no"), **T17** text transition ms on the followed style (FA-57) | V8b |
| `templates/church-profile.template.json` | Fields `stage_display`, `stream_capture`, `scripture_source`, `announcement_items_per_service`, `freeshow_version` (FA-51); `presentation_app` gains the tier | V17 (fields), V8b (`freeshow_version`) |
| `templates/service.template.json` | Per item `app.freeshow: { show_id, layout_id, project_item_index }` as empty placeholders (values are private) | V8b |
| `templates/minder-role-card.md` | v0 (Presenter) and v1 (FreeShow) on one page each; 44 § 2.5 copy as the starting text | V18 (v0 as docs in week 1) |
| `templates/minder-session.md` | The ten-minute session (44 § 2.6), four "done when" lines, pass rule | V18 |
| `templates/monthly-drill.md` | Six fault cards; "one manual song a month" | V18 |
| `templates/rota.template.csv` | Columns only (date, slot, minder, veteran, drill done); no names | V18 |
| `templates/worship-leader-card.md` | The three stage-strip states, HOLD on the stand, what "voxxa not sure" means | V18 |
| `templates/congregation-notice.template.md` | One paragraph, no "AI", "the pastor decides the wording" (FA-70) | V18 |
| `templates/incident.template.md` | What, which song (by position), roughly when, which key was pressed, post-state; numbers and labels only | V13s |
| `templates/service-report.example.json`, `templates/trend.example.csv` | Made-up numbers in the V2 schema | V2 |
| `templates/service-day-checklist.md` | Rewritten around the health check; "window in front" lines apply to `keystroke.*` profiles only | V13s / V17 |
| `templates/practice-session-checklist.md` | `--review` flow, `status`, who runs it | V12 |
| `templates/migration-checklist.md` | Week by week with who / done when / rollback; W0 inventory sign-off; the API password "never written in any kit file" | V17 |
| `templates/repertoire-inventory.template.csv` | Columns: slug, licence class, times sung (count), re-import week; no titles beyond PD in the example rows | V17 |
| `templates/team-note-freeshow.template.md` | One page, plain language, what changes for the slide team | V17 |
| `prompts/09-freeshow-migration.md`, `prompts/10-minder-training.md` | In the format of 01..08 (table, prompt, acceptance, stops at) | V17, V18 |
| `prompts/04-bindings.md`, `06-live-readiness.md`, `07-review.md`, `05-practice-session.md` | FreeShow probe rows and tiers (04); health check and the two keys (06); `service-<date>.json`, SLO table, trend sentence (07); `--review`, `status` (05) | V8b, V13s, V2, V12 |
| `README.md` | Tier table; steps 11 and 12 (prompts 09/10); six skills; upgrade notes `kit-v0.2.0` | V17 / V18 |

### 3.4 `docs/`

| File | Content | Step |
|---|---|---|
| `docs/INTEGRATIONS.md` | A **Companion / Stream Deck recipe** (docs only, FA-59): a button whose colour follows `GET /api/v1/state` and whose press calls `/pause`; a second button for `/panic`; a phone page as the hardware-free variant. A **FreeShow** section: Tier A, API posture (password, loopback firewall, bearer, refuse open API), the output lock as the external panic, the never-call list, "FreeShow pinned to stable". The existing WorshipTools section gains "Tier B". | V13a (routes), V8b (FreeShow), V13s (recipe) |
| `docs/runbooks/sunday-window.md` | The FA-67 window; what the minder does and does not do; who to call | V13s |
| `docs/runbooks/watchdog-tripped.md` | The four heartbeat signals, what tripped means, the post-state, how to resume | V13s |
| `docs/runbooks/panic-pressed.md` | F20's defined post-state, "I move the slides; voxxa still listens", what the tech lead checks after the service | V13s |
| `docs/runbooks/voxxa-not-sure.md` | The amber hold, why it holds on purpose, when it clears, when to HOLD or STOP instead | V14 |
| `docs/runbooks/back-to-presenter.md` | The Thursday 18:00 rollback: which app owns the projector on Sunday, the one-line check, who decides | V17 |
| `docs/quick-start.md` § 3 | The badge words and the two keys, for the volunteer | V14 |

### 3.5 This round's research report and notes

Recommended homes, all under `.10x/` because they are process memory (why we chose what we chose), not church
documentation, and `docs/` is the public site's content:

- The report: `.10x/specs/2026-11-26-research-autonomous-following.md` (renamed from the staging title with
  spaces), with a one-paragraph header: "reviewed in round 2; corrections C1..C17 and rulings R1..R17 in
  `.10x/reviews/2026-11-26-r2-50-adjudication.md`; read the adjudication first".
- The six notes: `.10x/specs/research-notes-2026-11-26/` with their current file names
  (`00_voxxa_current_state_brief.md`, `arxiv_lyrics_alignment.md`, `arxiv_score_following.md`,
  `arxiv_streaming_singing_asr.md`, `human_ai_training_reliability.md`, `presentation_software_landscape.md`).
- The seven reviews and the adjudication: `.10x/reviews/2026-11-26-r2-<nn>-<role>.md` (41 cto, 42 architect,
  43 sre, 44 ui, 45 consultant, 46 ponytail, 47 implementer, 50 adjudication). This layout:
  `.10x/specs/2026-11-26-skills-and-context-layout.md`. The strategy outputs `out/{spec,plan,process,
  founder-action-items}.md` go to `.10x/specs/2026-11-24-*.md`; `out/adrs/021..031` to `.10x/adrs/` (021 replaces
  the existing file only if its text changed; diff first).

**Scan before any of them is committed** (same scan as § 4.4, plus a read-through): the booth host name, the church
name and its short forms, the private song title (must read "P1" everywhere; the lab file names mention it in
comments: check), the founder's Windows user name in any path (`C:\Users\...`), volunteer or leader names (L1/L2
only), `.wav`/`.mp4`/`.m4a`/`.mp3` paths, e-mail addresses, the CCLI licence number, any quoted sung phrase (the
current-state brief § 2 and the lyrics-alignment note are the two places a quote could have slipped in; read them),
`voxxa-church-private` and `voxxa-private` paths (allowed only as "the private lab" or `<lab>`), and FreeShow show
or layout ids from the real install. Cited URLs stay; forum quotes from public pages in the landscape note are
fine. Zero hits, or the file stays in staging.

## 4. Commit plan

**Gate:** nothing from this round is committed until the founder approves the strategy: FA-10 (approved 2026-11-26), FA-43 (approved), FA-44 (answered), FA-54 and FA-65 (answered 2026-11-27), FA-29, FA-50, FA-52, FA-55 … FA-61 (answered 2026-11-27),
FA-45 (open and urgent) and FA-46..FA-72 answered in `out/founder-action-items.md`. Until then every file lives in
`C:\dev\voxxa-strategy-staging\` (round2, out, reports, research_notes). After the go, each lane commits only the
files its step names, on its own branch, through a PR with green CI including the media guard.

| Step (order per R1) | Commits | Stays private |
|---|---|---|
| **Records PR** (with V7, or a docs-only PR just before it; founder's go) | `.10x/specs/` strategy outputs, ADRs 022..031 (021 if changed), the research report and six notes, the eight round-2 files, this layout; root `CLAUDE.md` (§ 3.1 sections "Read first", "Sunday safety lines", "Budget rows", "Plugins", "JSONL and records"); `status.md` and `handoff.md` updates | the raw take files the brief's numbers came from |
| V7 | `scripts/loc_budget.py` rows; ADR-028/001/003/005 amendments; records | — |
| V3e / F5 | `.claude/skills/voxxa-freeshow-probe/SKILL.md`; `.10x/reviews/*-v3e-freeshow.md`, `2026-11-26-freeshow-source-notes.md`, ponytail audit; `CLAUDE.md` § "FreeShow driver facts" (first version); the `freeshow.json` note change (staging kit until V5 merges); FA-53 answer in the record | the test show, raw probe replies (`reports\probe-<date>\`), the password, show/layout ids, FreeShow's data folder |
| V3b / V3d / V3f | `.claude/skills/voxxa-lab-eval/SKILL.md`; port plans, before/after tables, ADR-031 amendment; `scripts/lab/onset_gate_cases.py`, `parts_gate_cases.py` (public, placeholder words only); V3f record and ADR-033 only on a pass | `parts.py`, `predict.py`, `live.py`, `flows.py`, `evalparts.py`, their pytest, every take, OLTW reference captures, CC-BY-NC weights |
| V5 / V5a | the kit as `kit-v0.1.0` (after `kit_check` and the § 4.4 scan on every file) | — |
| V2 | eval columns, `service-report.example.json`, `trend.example.csv`, prompt 07 edits | `reports\service-*.json`, `trend.csv` |
| V8b | ADR-032, driver, 8 tests, stub; `freeshow.json` final, T13..T17, `service.template.json` fields, `church-profile` `freeshow_version`, INTEGRATIONS FreeShow section, `CLAUDE.md` § "FreeShow driver facts" final | the filled test card, ids, token |
| V13a / V13s | ADR-034, ADR-025 § 7, ADR-022 and ADR-029 amendments; runbooks sunday-window, watchdog-tripped, panic-pressed; `incident.template.md`; `service-day-checklist.md`; `voxxa-church-setup` health and Sunday sections; INTEGRATIONS Companion recipe; `CLAUDE.md` § "Sunday safety lines" additions | `manifests\`, incident reports, `trend.csv` |
| V10 / V14 | the mock and the approved sentences record; `voxxa-not-sure.md`; quick-start § 3; `CLAUDE.md` § "GUI testing" | — |
| V12 | ADR-023 amendment; `voxxa-practice-session` and prompt 05, checklist | `practice\`, `sessions\` |
| V17 | ADR-035 (Proposed); prompt 09, migration checklist, inventory template, team note, `back-to-presenter.md`, profile fields, README tier table; `CLAUDE.md` § "Church kit work rules"; `kit-v0.2.0` tag together with V18 | the filled inventory and its signature, migration dates, the note as sent, the password |
| V18 | `voxxa-minder-training`, prompt 10, the six templates (card, session, drill, rota, leader card, notice) | the filled rota, volunteers' names, drill results with names, consent forms |
| V15 | acceptance table, runbook set check, `status.md` close | founder minutes per Sunday are a number and may be committed; nothing else |

**Private forever** (never in any repo, any tag, any ticket, any artifact): lyrics and setlists, every recording,
`sessions\`, `practice\`, FreeShow show and layout ids, the FreeShow test show, raw probe replies, the API
password or token, the repertoire inventory and its signature, the filled rota and any volunteer's name, consent
forms, incident reports as written, `manifests\`, `reports\` (only the schema and made-up examples are public),
`trend.csv`, the filled test cards, the lab `.py` files and their pytest, OLTW reference captures, `scan-terms.txt`.

**The scan a lane runs before any commit** (§ 4.4). The terms live in one private file,
`C:\dev\voxxa-church-private\scan-terms.txt`, one per line, never in the repo: the booth host name, the church name
and its short forms, the private song title(s), the founder's Windows user name, every volunteer's and leader's name,
the street and town, the CCLI licence number, the church's e-mail domain. The lane runs, on the staged files only:

```
git diff --cached --name-only -z | xargs -0 grep -n -i -F -f C:/dev/voxxa-church-private/scan-terms.txt
git diff --cached --name-only -z | xargs -0 grep -n -i -E '\.(wav|mp4|m4a|mp3)\b|[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[a-z]{2,}|voxxa-church-private|voxxa-private|fixtures[/\\]private|recordings[/\\]|songs[/\\]|Bearer [A-Za-z0-9]|password *[:=]|C:\\Users\\|([0-9]{1,3}\.){3}[0-9]{1,3}'
uv run kit/tools/kit_check.py      (once V5 has merged)
```

Zero hits in the first two commands, or the lane stops and reports **counts per file** (never the matched line) in
its record. Allowed exceptions are written next to the regex in `CLAUDE.md` § "Sunday safety lines": `127.0.0.1`,
the string `<church_root>`, the alias `<lab>`, and the three lab file names when the prompt names them. The scan
runs on staged files only so that the two older `.10x/` files that already carry some of these strings (the
pilot-facts line in `handoff.md` § Q9W/Q9V and the pilot-church and test-song lines in `status.md`) do not block
every commit; whether to scrub those two files is a founder decision this layout only flags (no FA number assigned).

## 5. What an agent reads first on each step

| Step | Read first, in this order |
|---|---|
| **V3e** | 1 `.10x/handoff.md` (safety block, row table) · 2 `.10x/reviews/2026-11-26-r2-50-adjudication.md` § 2 C1..C4, § 3 R1/R2 · 3 `.claude/skills/voxxa-freeshow-probe/SKILL.md` · 4 `kit/templates/bindings/freeshow.json` · 5 `.10x/reviews/2026-11-26-freeshow-source-notes.md` (or 42 § 1 until it exists) · 6 the V3e prompt |
| **V8b** | 1 `.10x/handoff.md` · 2 `.10x/reviews/*-v3e-freeshow.md` (the go, the latency) · 3 `CLAUDE.md` § "FreeShow driver facts" · 4 `src-tauri/src/presenters/{freeshow,mod}.rs` and `tests/dispatch.rs:289-305` · 5 ADR-006, ADR-024 § 5, ADR-025 · 6 the V8b prompt (8 named tests, ADR-032 outline) |
| **V3d** | 1 `.10x/handoff.md` · 2 ADR-030, ADR-031 (with the R3 amendment text in the adjudication) · 3 `.claude/skills/voxxa-lab-eval/SKILL.md` · 4 the adjudication § 3 R3, R14 and § 2 C5, C6, C8 · 5 `47-implementer.md` §§ 3.1, 3.5 (until folded into the port plan) · 6 the V3d prompt |
| **V13s** | 1 `.10x/handoff.md` · 2 `43-sre.md` §§ 1-3, 5 (SLO, alert and failure-mode tables; the record once committed) · 3 ADR-022, ADR-025, the V13a record (routes) · 4 `scripts/loc_budget.py` (the `health check` row) and the row table · 5 `kit/templates/service-day-checklist.md` and `docs/runbooks/presentation-app-not-reachable.md` (the house style for runbooks) · 6 the V13s prompt |
| **V14** | 1 `.10x/handoff.md:112` (guarded GUI rules) · 2 the V10 mock record (every approved sentence, FA-62) · 3 `44-ui-design.md` §§ 2.1, 2.2, 2.4 (wireframes) · 4 `src/simple/logic.js`, `tests/gui/` (mock, e2e) · 5 ADR-011, ADR-017, ADR-024 §§ 6-7 · 6 the V14 prompt |
| **V17** | 1 `kit/README.md` and `kit/private-folder.md` · 2 `45-consultant.md` § 5 V17 (the prompt-09 draft) and FA-48, FA-51, FA-56, FA-68 · 3 ADR-026, ADR-029, ADR-035 draft · 4 `kit/prompts/02-song-library.md`, `04-bindings.md`, `templates/presenter-test-card.md` (T13..T17) · 5 the V17 prompt |
| **V18** | 1 `kit/README.md` § rules · 2 `44-ui-design.md` §§ 2.5-2.6 (card and session copy) and `45-consultant.md` § 5 V18 · 3 the V10 mock record (badge words, FA-61, FA-62) · 4 `kit/templates/service-day-checklist.md`, `practice-session-checklist.md`, prompts 05-07 · 5 FA-69, FA-70, FA-71 in `founder-action-items.md` · 6 the V18 prompt |

Every list is followed by the standing safety block, which the prompt repeats; an agent that cannot find file 1
stops and says so rather than reading around it.
