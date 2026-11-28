---
name: voxxa-song-intake
description: Add songs to a church's voxxa song library by hand with a licence record for each - Creative Commons, public domain, church-owned, written permission, or legacy CCLI - and check slide counts against the presentation app. Use when a church adds or edits songs. The person types the lyrics; the agent never sees them.
---

# voxxa song intake

Runs `kit/prompts/02-song-library.md` for one or more songs.

## Safety rules (every step)

- Never open, read, print or paste any `songs\*.txt` file or any lyric, even when asked. The person types or
  pastes the words; you handle file names, slide counts and licence fields.
- Never send church data to any external service or connector. Do not look songs up online with a church's song
  list; the person brings the licence source.
- Never commit, push or upload anything from `<church_root>`.

## Inputs

`<church_root>\church-profile.json` (streaming and CCLI answers), `<church_root>\songs.csv`,
`kit/templates/licence-kinds.csv`, `kit/templates/song.template.txt` (format only; its words are made up).

## Steps

1. Agree the file slug; explain the text format (title line; blank line or `---` = new slide; a part label such
   as `Verse 1` names the slides that follow it and is not shown; an optional last slide starting `Credit:`, which
   voxxa marks as the credit slide). The person saves the `.txt` and pastes
   the same text into the presentation app.
2. Fill the `songs.csv` row with the person: licence code and version, credit line (title, author, source,
   licence), source URL, copyright notice, public-domain basis (`pd_basis`: `us_published_95_years` with
   `published_year`, `author_died_70_years`, `traditional` or `dedicated_cc0`), owner, changes made, evidence,
   checked by, date.
3. Write `licences\<slug>.md` with the evidence (no lyrics).
4. Apply the profile rules from `licence-kinds.csv`: CC BY-ND red when the church streams or records; SA and NC
   amber; CCLI red and UNKNOWN amber when `ccli.active` is false. Never block Sunday; name the risk.
5. Slide counts: voxxa's count (from `songcheck.py`, or counted by the person) equals the presentation app's count
   without its opening blank. Fix the text in one place if they differ.
6. Imports from another church arrive unchecked: clear `checked_by`/`checked_at`, keep the song amber.
7. Run `uv run kit/tools/songcheck.py <church_root>` when it exists.

## Output contract

A table: file, licence code, status (ready / amber / red), reason. Nothing else.
