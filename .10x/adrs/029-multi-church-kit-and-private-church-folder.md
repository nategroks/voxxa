# ADR-029: A public multi-church kit under `kit/`, one private folder per church, and guards that keep church data out of the public repo

**Status:** Accepted 2026-11-27 (V5a; amendment 1 below: FA-01 and FA-12 (c)) · Proposed 2026-11-24 · **Date:** 2026-11-24 · **Feature:** church-autoadvance · **Reversibility:** Easy for the kit (files); Hard for anything already published (hence the guards) · **Owner:** nategroks · **Author:** 10x-Team synthesiser, from drafts 12, 14 and reviews 20, 21

## Context
- Founder decision (brief § 2): reusable kit (templates, prompts, skills, process) in the public repo under `kit/`; anything church-specific (song lists, licences, recordings, practice data) in a private folder that never leaves the booth PC (`C:\dev\voxxa-church-private` for the founder; § 2 splits it into `pilot\`, `fake-church-2\` and `lab\`).
- The public repo already holds church identifiers: the church's CCLI account number, its worship band's name as a private song's artist, and the booth PC's host name (review 20 S-42, citing `.10x/adrs/019-…:112-114`, `fixtures/LICENSED_RUNS.md:11-13`, `fixtures/README.md:288`); the prompt pack names the pilot church (`.10x/specs/2026-11-11-prompts-10x-team.md:117`). `LICENSED_RUNS.md`'s template invites each church to commit its licence number.
- Guards today: `.gitignore` refuses media and `fixtures/private/` (`.gitignore:33-45`); a media-guard test and CI job (`.github/workflows/ci.yml:155-165`); numbers-only eval output. Nothing guards the new private file types (`church-profile.json`, `layout.json`, `*.service.json`, `songs.csv`, `practice/*.json`, `truth.jsonl`, `session-*.jsonl`) (S-46). The licensing test's needles cannot see a private song's words (S-43).
- The private song P1's title or slug also appears in 36 lines in 8 files (verified), across `.10x/`, `fixtures/` and one test; the list is in plan V5 step 6 (critic gap G03, FA-12), and the title is never quoted.
- There is no `CODEOWNERS` file on `origin/main` (`git ls-tree`).
- The kit's church flows (practice capture, `.wav` intake, `--check-layout`, scoring) need `voxxad` and `voxxa-eval`. Both are developer tools behind the off-by-default `cli` feature (`src-tauri/Cargo.toml:27-30`, `:121-122`); `build.yml` never builds them and `tauri.conf.json` has no `externalBin`, so the installers do not ship them. `voxxad` has `--help` but no `--version` (`src-tauri/src/cli/voxxad/cli.rs:43`); `voxxa-eval` has `--help` (`src-tauri/src/cli/voxxa-eval/main.rs:49`).
- Kit prompts and skills are instructions agents run at every church with that church's data, sometimes with personal-data connectors attached (S-47). Generalising staging documents into kit prompts would carry private facts (S-45).
- There is one church. A kit no second church validates will rot (review 21 row 10).

## Decision
1. **Public `kit/`** (draft: `out/kit/`): `README.md`; `templates/` (church profile, master layout, service plan, six binding profiles, song text, licence register, licence kinds, church-setup, wav-intake, practice-session and service-day checklists, presenter test card, an agent settings example, and five more templates: `recording-consent.template.md`, `fixtures-manifest.template.json`, `intake.template.json`, `truth-v2.example.jsonl` and `lab/pyproject.template.toml`); `prompts/01–08` (intake, song library, layout, bindings, practice session, live readiness, review, wav intake); `skills/` (voxxa-church-setup, voxxa-song-intake, voxxa-wav-intake, voxxa-practice-session, voxxa-layout-author), each thin and sharing one safety block; `private-folder.md`; later `tools/songcheck.py` and `tools/kit_check.py` (V5). Lowercase "voxxa" everywhere; examples use made-up words and public-domain titles only. Prompts are written fresh from the kit's own README, never generalised from staging files.
2. **Private church folder** (`<church_root>`): `%LOCALAPPDATA%\voxxa-church\` by default for a new church (one church per PC). It is a sibling of the install folder `%LOCALAPPDATA%\Voxxa\` (NSIS `installMode: currentUser`, `productName` "Voxxa": `src-tauri/tauri.conf.json:3`, `:77`; NTFS names are case-insensitive, so that is also `%LOCALAPPDATA%\voxxa\`) and of Tauri's app-local-data folder `%LOCALAPPDATA%\com.voxxa.app\` (`identifier`, `tauri.conf.json:5`). Install, update, uninstall and the V16 rename never touch it. voxxa's own session logs and trust store are app data in `%LOCALAPPDATA%\com.voxxa.app\sessions\` and `%LOCALAPPDATA%\com.voxxa.app\trust\` (ADR-023 § 6, ADR-025 § 5), outside `<church_root>`. On the founder's development booth `church_root` is `C:\dev\voxxa-church-private\pilot\`; the fake second church for the kit acceptance test (§ 6) is `C:\dev\voxxa-church-private\fake-church-2\`; shared private tools (the lab, ADR-027 § 5) stay in `C:\dev\voxxa-church-private\lab\`. No other church-id folder is used. Layout in `kit/private-folder.md`. **Not a git repository** (a push hook is not a control); weekly copy to one encrypted drive; BitLocker on the booth PC for any church that keeps recordings; an `.ignore` file holding `*` at the root so code-search tools skip it; never under OneDrive or a network share; never inside the install folder or the app-data folder.
3. **Guards in the public repo:**
   - `.gitignore` and the media guard (test + CI job) refuse `church-profile.json`, `layout.json`, `*.service.json`, `songs.csv`, `practice/`, `sessions/`, `session-*.jsonl`, `truth.jsonl` and `*.sqlite3*` everywhere except `kit/templates/` (templates are named `*.template.*`) and the existing public fixtures;
   - `kit/tools/kit_check.py`: refuses lyric keys (`text`, `lyrics`, `slides`, `words`) in kit JSON, media files, `C:\Users\` paths, and any string in a private deny-file passed with `--deny-file` (church name, song titles, host name, licence numbers; the list itself stays private); with `--lyrics-src <church_root>\songs` it hashes every 4-word sequence of the church's songs and fails on any match in the candidate files, printing file:line only;
   - the scan runs on `kit/`, `.10x/` and `fixtures/` before any PR that touches them, and on any file before it moves from a private to a public path;
   - **order:** the scrub PR (FA-12, plan V5 step 6: church identifiers and song titles, by path:line) merges **before** the deny-file scan becomes a gate over `.10x/` and `fixtures/`. Until it merges, the scan gates `kit/` only and reports, without failing, on `.10x/` and `fixtures/`, which still hold the strings it would refuse;
   - `CODEOWNERS`: the founder owns `kit/prompts/**`, `kit/skills/**` and `kit/templates/bindings/**`;
   - churches install a **tagged** kit (`kit-vX.Y.Z`; the tag does not start with `v`, so no release build runs); a manifest generator is deferred.
4. **Agents at a church:** every kit skill states "never send church data to external services; numbers, counts, file names and line numbers only; never read or print lyrics or heard words". The kit's `templates/agent-settings.example.md` gives a local, never-committed `.claude/settings.local.json` for the repo checkout on the booth PC. It denies:
   - the personal-data connectors (Planning Center, Drive, Gmail);
   - `Read` on named church folders (`songs\`, `services\*.setlist.json`, `sessions\`, `practice\`, `recordings\`, `fixtures\`, `inbox\`, and `consent\`, `reports\` and `licences\` if present) and `Grep` on all of `<church_root>`;
   - shell patterns that read or copy from `<church_root>`: Bash `cat`, `type`, `head`, `tail`, `less`, `more`, `grep`, `rg`, `cp`, `copy`, `xcopy`, `robocopy`; PowerShell `Get-Content`, `gc`, `cat`, `type`, `Select-String`, `sls`, `Copy-Item`, `cp`, `Import-Csv`;
   - `git push` and `git remote`.

   Files without lyrics (`church-profile.json`, `layout.json`, `services\*.service.json`, `songs.csv`) stay readable. The example states plainly: "permission patterns are a seatbelt, not a lock; the real controls are (1) never start an agent with `<church_root>` as its working or added directory, (2) skills that never ask for content, (3) the numbers-only tools." The exact rule syntax is checked against the current Claude Code settings documentation before use **[UNVERIFIED]**.
5. **Records use aliases** ("pilot church", "private song P1", "booth PC"). Scrubbing the existing identifiers and song titles from current files is founder item FA-12 and plan V5 step 6, by path:line, never quoting a title (history rewrite is a separate, outward choice).
6. **Acceptance for the kit:** a fake second church (`C:\dev\voxxa-church-private\fake-church-2\`) is set up from the kit alone with zero edits to public files; `songcheck`, `--check-layout` (when built) and `kit_check` pass; the fake church runs `voxxad --version` and `voxxa-eval --help` from the `voxxa-tools-<version>-windows-x64.zip` (§ 7), with no repo checkout; the kit README says "Windows booth only until Q24/Q26".
7. **CLI tools for churches (plan V5c "voxxa-tools zip", founder item FA-31):**
   - `voxxad` and `voxxa-eval` ship as a separate signed release asset, `voxxa-tools-<version>-windows-x64.zip`, built with `--features cli` in the Windows release job, with the same ADR-021 release flags as the app; the GUI installer is unchanged, so the updater and signing paths stay the same;
   - `voxxad` gains a `--version` flag (≤ 3 lines of ADR-028's voxxad share);
   - V5c comes after V0 and before church #2; until it ships, practice capture, `.wav` intake and `--check-layout` run only on the founder's development booth from a local build, and the kit README says so plainly;
   - whether the current Windows signing setup (SignPath, `.github/workflows/build.yml:10-13`) signs the two exes inside a zip without a change to its artifact configuration is **[UNVERIFIED]**.

## Alternatives Considered
| Alternative | Pros | Cons | Why Not |
|---|---|---|---|
| Minimal kit: 1 skill, 6 prompts, 1 tool (draft 14) | least to maintain | the founder asked for prompt sets and skill sets | 5 thin skills over one shared safety block; validated on a fake church |
| Full kit with process/, onboarding/, library/, 11 prompts + 5 role prompts, 4 skills, manifest, semver (draft 12) | complete | rots without a second church; role prompts generalised from staging leak facts (S-45) | trimmed; process lives in `.10x/`, not the kit |
| Private folder as git with a refusing pre-push hook (draft 12) | history of config | hooks are skipped by `--no-verify` and do not stop `git remote add` (S-44) | no git; backups |
| Refuse every `C:\dev\…-private` path in kit files (S-45 as written) | stricter | the founder's path is generic and reveals nothing; the kit must describe it | refuse `C:\Users\` paths and the deny-file; allow the generic path |

## Consequences
### Positive
- Church #2 can start from a documented, tested kit; nothing church-specific can be committed by accident.
### Negative
- Two copies of process knowledge (`.10x/` for voxxa development, `kit/` for church setup) must stay consistent: the kit links, it does not copy.
- Until V5c ships the tools zip, church #2 cannot run a practice session, a `.wav` intake or a layout check; only the founder's development booth can.
### Risks
- The lyric scanner needs the church's songs folder, so it runs only on the booth PC; CI runs the deny-file-free checks only.

## Dependencies
- Depends on: ADR-019 § 8 (media guard), ADR-023, ADR-024, ADR-026, ADR-027.
- Plan steps: V5 (kit, guards, scrub), V5c (voxxa-tools zip), V15 (fake-church validation at close).
- Founder items: FA-12 (scrub), FA-31 (how the CLI tools reach churches).

## Review trigger
Church #2 starts (re-validate the kit with a real church); any private string found in the public repo; a kit PR from outside the team.

## Amendment 1 (2026-11-27, V5a): accepted; FA-12 (c) and FA-70; what V5a built
- Accepted on FA-01 (V5 pre-approved, `specs/2026-11-24-founder-action-items.md:14`) and FA-12 (c) (`specs/2026-11-24-founder-action-items.md:23-24`): there is no scrub PR, so Decision 3's "order" bullet and Decision 5's scrub are void; the deny-file scan gates `kit/` only, never `.10x/` or `fixtures/`.
- FA-70 (no congregation notice): `templates/congregation-notice.template.md` is not part of the kit; what voxxa keeps and does not keep stays stated in `private-folder.md`.
- Built in V5a: `kit/` (42 files: `README.md` with the FA-47 tier table, `private-folder.md`, 27 templates with the 6 binding profiles among them, 8 prompts, 5 skills), `.github/CODEOWNERS` (Decision 3) and the `.gitignore` half of Decision 3's guard (the eleven names, with `kit/templates/` and the public fixtures' `truth.jsonl` and the voxxad test log re-included by path). The media-guard test and CI job, `kit_check.py`, the tagged-kit manifest and Decision 6's fake-church acceptance are V5b; Decision 7 is V5c.
