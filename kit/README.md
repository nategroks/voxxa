# voxxa church kit

This kit helps a church set up voxxa: its songs and licences, its master layout, its practice sessions and
its Sunday checks. A church's tech lead can follow it by hand, or let an AI agent (Claude Code) run the
numbered prompts with them.

**Windows booth PCs only** until voxxa's macOS and Linux steps ship.

Status: kit `kit-v0.1.0`, written 2026-11-24 and committed in plan step V5a (2026-11-27). Tools marked *(built in
step Vn)* do not exist yet; until then, the checklists say how to do the same check by hand. The kit's own tools
(`tools/songcheck.py`, `tools/kit_check.py`) come with step V5b.

**Practice sessions, recording intake and the layout check need voxxa's command-line tools** (`voxxad`,
`voxxa-eval`). The voxxa installer does not include them. They will ship as a separate signed download,
`voxxa-tools-<version>-windows-x64.zip`, from plan step V5c. **Until V5c ships, these three jobs run only on the
voxxa founder's development booth, from a local build.** A second church can do everything else in the kit now
(songs and licences, the layout by hand, the presenter test card, Sunday checks, the review). See "Tools and when
they arrive" below.

---

## Two folders, never mixed

| Folder | What it holds | Where | Shared? |
|---|---|---|---|
| **The kit** (`kit/`) | templates, prompts, skills, checklists; made-up examples and public-domain titles only | the public voxxa repository | yes, public |
| **The church folder** (`<church_root>`) | the church's songs and lyrics, licence register, church profile, master layout, weekly service plans, practice memory, recordings, reports | the booth PC only; default `%LOCALAPPDATA%\voxxa-church\`, beside voxxa's install and app-data folders, never inside them | **never**: no git, no cloud sync, no e-mail, no upload |

`private-folder.md` shows the church folder's layout. Copy templates **out of** the kit into the church folder;
never copy anything **into** the kit.

## The rules every person and agent follows

1. **Lyrics stay in the church folder.** Nobody prints, pastes, quotes or uploads song words or the words voxxa
   heard. Agents work with counts, section labels (V1, C, B), file names and line numbers only.
2. **Numbers only** in any report, ticket or message that leaves the booth PC.
3. **No church data to outside services** (no Planning Center, Drive or Gmail connectors during setup; see
   `templates/agent-settings.example.md`).
4. **voxxa never projects lyrics.** The presentation app shows the words and the credit line; voxxa only tells it
   which slide to show.
5. **Agents never press keys in the presentation app** and never start voxxa listening. The tech lead does every
   step that touches the real app, the microphone or a recording.
6. **Every song has a licence record** before it is "ready". Missing licences warn; they never block Sunday.

## Two tiers of presentation app (FA-47)

Not every presentation app lets voxxa work alone. The kit sorts apps into two tiers.

| Tier | Apps | What voxxa does |
|---|---|---|
| **A: jump and read-back** | FreeShow; ProPresenter; OpenLP (in part) | voxxa can jump to a named slide and read back which slide is showing. Only a Tier A app may follow a service without a person at the keys, and only after the shadow Sundays in `prompts/07-review.md` pass. |
| **B: keys only** | WorshipTools Presenter; the universal keystroke profile | voxxa presses Next and Back and shows a suggestion card for anything else. A minder stays at the keys for the whole service. |

The kit never promises that voxxa runs on its own on a Tier B app. Write the tier in `church-profile.json`
(`presentation_app_tier`).

## How a new church uses the kit (10 steps)

| # | Step | Prompt | Skill | Who | Time [est.] |
|---|---|---|---|---|---|
| 1 | Intake: app, PC, audio feed, streaming, licences | `prompts/01-intake.md` | voxxa-church-setup | tech lead + agent | 45 min |
| 2 | Install uv, ffmpeg, voxxa (and the voxxa-tools zip once V5c ships); hardware check | in 01 | voxxa-church-setup | tech lead | 30 min |
| 3 | Make the church folder from `private-folder.md` | in 01 | voxxa-church-setup | tech lead + agent | 15 min |
| 4 | Type in the songs, one `.txt` each, plus `songs.csv` | `prompts/02-song-library.md` | voxxa-song-intake | tech lead | 5 min per song |
| 5 | Write the master layout (`layout.json`) | `prompts/03-layout.md` | voxxa-layout-author | tech lead + agent | 1 h |
| 6 | Check the binding profile and run the presenter test card | `prompts/04-bindings.md` | voxxa-layout-author | tech lead | 30 min |
| 7 | Rehearsal: voxxa listens and learns each song (needs `voxxad`, V5c) | `prompts/05-practice-session.md` | voxxa-practice-session | operator | rehearsal + 15 min |
| 8 | Sunday readiness checks | `prompts/06-live-readiness.md` | voxxa-church-setup | operator | 10 min |
| 9 | After the service: numbers and next steps | `prompts/07-review.md` | voxxa-church-setup | operator + agent | 15 min (+ tapping for a scored shadow service) |
| 10 | Any recording (.wav or video): intake and evaluation (needs `voxxa-eval`, V5c) | `prompts/08-wav-intake.md` | voxxa-wav-intake | tech lead + agent | 20 min + tapping |

Start in **shadow mode** (voxxa listens and logs, moves nothing) for at least two Sundays. Let voxxa move slides
only when the shadow numbers meet the bar in `prompts/07-review.md`. A shadow Sunday is scored from a recording of
the desk feed, tapped truth and `voxxa-eval --actions` (prompt 07 says how), so it also needs the tools zip and
recording consent that covers service audio.

## What is in the kit

```
kit/
  README.md                      this file
  private-folder.md              the church folder's layout and rules
  templates/
    church-profile.template.json the intake answers (no lyrics, no secrets)
    layout.template.json         the master layout: graphics, cue words, modes, hotkeys
    service.template.json        a weekly service plan
    bindings/*.json              how each presentation app does each action (read-only reference)
    song.template.txt            one song's words, as voxxa and the presentation app read them
    songs.template.csv           the licence register
    licence-kinds.csv            what each licence allows
    church-setup-checklist.md
    practice-session-checklist.md
    wav-intake-checklist.md
    service-day-checklist.md
    presenter-test-card.md
    recording-consent.template.md
                                 the consent form for anyone whose voice is recorded
    intake.template.json         one recording's numbers and licence classes (no titles, no words)
    fixtures-manifest.template.json
                                 which test recordings are tune or held-out, truth hashes, look count
    truth-v2.example.jsonl       a made-up truth file with the optional v2 rows
    agent-settings.example.md    agent permissions for church setup
    lab/pyproject.template.toml  optional private lab for recordings and evaluation
    incident.template.md         draft for step V13s: a one-paragraph incident note, numbers and labels only
    minder-role-card.md          draft v0 for step V18: the minder's one-page card
    repertoire-inventory.template.csv
                                 draft for step V17: columns for the song inventory before a migration
    rota.template.csv            draft for step V18: columns only, no names
  prompts/01-…08-…md             what an agent runs, in order
  skills/<name>/SKILL.md         five skills that wrap the prompts
  tools/                         songcheck.py, kit_check.py (built in step V5b; not in kit-v0.1.0)
```

## Install the skills

Copy each folder under `kit/skills/` into the project's `.claude/skills/` (a checkout of the voxxa repository on
the booth PC) or into the user's skills folder. In PowerShell, from the repository root:

```powershell
New-Item -ItemType Directory -Force .claude\skills | Out-Null
Copy-Item -Recurse -Force kit\skills\* .claude\skills\
```

Use a **tagged** kit (`git checkout kit-v0.1.0`), not whatever is on `main` today.

## The master layout in one paragraph

`layout.json` says **what** should happen in this church's services: which graphics exist (logo, welcome,
prayer card, giving), which spoken phrases suggest them ("let us pray" → prayer card), which modes each
trigger uses (voxxa does it / voxxa suggests / off), and which hotkeys the operator uses. It never says
**how**: the binding profile for the church's presentation app does that (a key press, an API call, or a
sentence to the operator). The weekly service plan lists the items in order and can only make voxxa *more*
cautious than the layout. Change presentation app → change one field (a graphic with no mapping for the new app
becomes a reminder to the operator, `hint`). Command lines started from a layout are
not supported (a safety rule).

## Versioning

The kit is tagged `kit-vMAJOR.MINOR.PATCH` in the voxxa repository. MAJOR: a template's format changes
(`layout_version`, `service` format). MINOR: new prompts, templates or skills. PATCH: wording. The church
folder records the kit version it came from in `church-profile.json` (`kit_version`). Templates are never
applied over a church's files automatically.

**Upgrading a church folder to a new kit tag.** Every kit release has upgrade notes (below). To upgrade:

1. Read the upgrade notes of every tag after the church's `kit_version`, up to the new tag.
2. `git checkout kit-v<new>` in the repository checkout on the booth PC; copy the skills again ("Install the
   skills" above).
3. Copy only the templates the notes name into `<church_root>`, and carry the church's own values across by hand.
   Never copy a template over a church file that holds the church's answers.
4. Run `uv run kit/tools/kit_check.py` and `uv run kit/tools/songcheck.py <church_root>` (step V5b) and fix what
   they report.
5. Set `kit_version` in `church-profile.json` to the new tag.

### Upgrade notes

- **kit-v0.1.0**: first release. Nothing to upgrade.

### Keeping the tools current

- **uv**: version 0.13.10 or newer (the version on the voxxa founder's development booth, FA-03). A church
  installs uv with `winget install --id astral-sh.uv -e` and updates it with `winget upgrade --id astral-sh.uv -e`.
- **voxxa itself** updates through its signed updater. voxxa never installs an update between Saturday 18:00
  and Sunday 14:00 local time, or while Songs mode is on.
- **The voxxa-tools zip** (after V5c): use the zip with the same version number as the installed voxxa app.
  After each app update, replace the tools folder with the matching zip.

## Tools and when they arrive

`songcheck.py` and `kit_check.py` come with the kit from step V5b and run with `uv run`. `voxxad` and `voxxa-eval` are
voxxa's command-line tools. On the voxxa repository today they are developer tools behind the off-by-default
`cli` build feature, so the installer leaves them out. Plan step **V5c** builds them with `--features cli` in the
Windows release job and publishes them as a separate signed release asset,
**`voxxa-tools-<version>-windows-x64.zip`**, next to the unchanged installer. A church downloads the zip and its
`.sha256` file (`voxxa-tools-<version>-windows-x64.zip.sha256`) from the same release page, checks the zip with
`Get-FileHash -Algorithm SHA256`, then unzips it **outside** `<church_root>` (for example into
`%LOCALAPPDATA%\voxxa-tools\`; never inside `<church_root>`, voxxa's install or app-data folder, or a synced
folder), adds that folder to its user `PATH`, and checks it with `voxxad --version` and `voxxa-eval --help`:

```powershell
$zip  = "$env:USERPROFILE\Downloads\voxxa-tools-<version>-windows-x64.zip"
$want = (Get-Content "$zip.sha256" -Raw).Trim().Split()[0].ToLower()
if ((Get-FileHash -Algorithm SHA256 $zip).Hash.ToLower() -ne $want) { throw "hash mismatch: do not use this zip" }
Expand-Archive $zip -DestinationPath "$env:LOCALAPPDATA\voxxa-tools" -Force
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*voxxa-tools*") {
  [Environment]::SetEnvironmentVariable("Path", "$userPath;$env:LOCALAPPDATA\voxxa-tools", "User")
}
```

Open a new PowerShell window before `voxxad --version`, so it sees the new `PATH`. **Until V5c ships, every row marked
"tools zip" below runs only on the voxxa founder's development booth, from a local build.** Never put a local or
test build on a church's booth PC.

| Tool | Comes from | Does | Until it exists |
|---|---|---|---|
| `uv run kit/tools/songcheck.py <church_root>` *(step V5b)* | the kit | checks `songs\*.txt` against `songs.csv` and the licence rules; prints file and field names only | `templates/church-setup-checklist.md` § Songs, by hand |
| `uv run kit/tools/kit_check.py` *(step V5b)* | the kit | refuses lyrics, media and private strings in files before they become public | never copy church files into the repository |
| `voxxad --list-devices`; `voxxad --device <name> --shadow …` | tools zip *(step V5c)* | finds the desk feed; listens in shadow mode and writes a session log (practice capture, shadow Sundays) | founder's development booth only |
| `voxxad --check-layout <file>` *(step V13)* | tools zip *(step V5c)* | validates `layout.json` and service plans | `prompts/03-layout.md` by hand |
| `voxxad --learn <session.jsonl> --setlist <file> --out <church_root>\practice` *(step V12)* | tools zip *(step V5c)* | turns a rehearsal log into proposed practice memory; prints part labels and counts only | rehearse in shadow mode; nothing is learned yet |
| `voxxad --confirm <song_fp> --practice-dir <church_root>\practice` *(step V12)* | tools zip *(step V5c)* | confirms the newest proposed run of one song | nothing to confirm yet |
| `voxxa-eval --full --model <m> <fixture>` *(step V2 adds `--model`)* | tools zip *(step V5c)* | scores a recording against tapped truth, numbers only | `voxxa-eval --full` (Tiny only), founder's development booth only |
| `voxxa-eval --actions <actions.jsonl> <folder>` *(step V2)* | tools zip *(step V5c)* | scores a list of voxxad actions (a shadow-Sunday replay) against tapped truth | shadow Sundays cannot be scored yet |
| `voxxad --version` *(step V5c)* | tools zip | prints the tools' version, to match the app | none |
