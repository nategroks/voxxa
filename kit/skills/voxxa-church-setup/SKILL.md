---
name: voxxa-church-setup
description: Set up and run voxxa for one church from the voxxa church kit - intake and church profile, installs, the private church folder, Sunday readiness checks and the after-service review. Use when a church starts with voxxa, before a service, or after one. Never reads lyrics or heard words; numbers only.
---

# voxxa church setup

Walks a church's tech lead or operator through the kit's church-level prompts. The song, layout, practice and
recording jobs have their own skills.

## Safety rules (every step)

- Never read, print, paste or copy song lyrics, setlists, session logs, practice files, truth files, actions
  files or audio. Work with counts, part labels (V1, C, B), file names, line numbers and yes/no answers only.
- Never send church data to any external service, connector or website. Do not use Planning Center, Drive or
  Gmail tools in this work, even if they are connected.
- Never press keys in the presentation app and never start voxxa listening; the person at the booth does that.
- Never write outside `<church_root>` except `.claude/settings.local.json` when the tech lead asks.
- Never commit, push or upload anything from `<church_root>`.
- Name songs by their plan position ("song 2") in anything that might leave the booth PC.

## Which prompt for which job

| Job | Prompt | When |
|---|---|---|
| New church: profile, installs, folder | `kit/prompts/01-intake.md` | once |
| Sunday readiness | `kit/prompts/06-live-readiness.md` | 10 minutes before each service |
| After the service | `kit/prompts/07-review.md` | the same day or Monday |
| Songs and licences | skill `voxxa-song-intake` | as songs are added |
| Master layout and bindings | skill `voxxa-layout-author` | once, then when graphics change |
| Rehearsal learning | skill `voxxa-practice-session` | before each service |
| A recording to evaluate | skill `voxxa-wav-intake` | when a recording arrives |

## How to run

1. Read `kit/README.md` and `kit/private-folder.md`. Find `<church_root>` (ask; default
   `%LOCALAPPDATA%\voxxa-church\`, beside voxxa's install folder `%LOCALAPPDATA%\voxxa\` and app-data folder
   `%LOCALAPPDATA%\com.voxxa.app\`, never inside either). If `church-profile.json` is missing, run prompt 01 first.
2. Run the prompt for the job, step by step, waiting for the person's answer at each step.
3. Keep the checklists in `kit/templates/` as the record: tick boxes in a copy inside `<church_root>`.

## Stop and hand back when

- an install or any action on the booth PC's apps is needed (give the exact command; the person runs it);
- a step would need lyrics, heard words or audio;
- the person asks to switch a cue or scripture trigger to automatic (that needs voxxa's proven-safe bar and a typed
  confirm in `layout.json` by the tech lead);
- the presentation app runs as administrator (fix first).

## Output contract

End every session with a short list: what is done, what is open, the next prompt to run. No lyrics, no heard words,
no paths under the Windows user-profile folder.
