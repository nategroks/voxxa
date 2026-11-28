# 02 — Song library: lyrics by hand, a licence for every song

| | |
|---|---|
| Run with | skill `voxxa-song-intake` |
| Who | the tech lead types or pastes the words; the agent handles the licence record and the checks |
| Time [est.] | about 5 minutes per song |
| Cost band | S |
| Inputs | `church-profile.json`; `kit/templates/song.template.txt`, `songs.template.csv`, `licence-kinds.csv`; the song's licence source (a web page, a songbook, a written permission) |
| Outputs | `songs\<slug>.txt` (by the tech lead), a row in `songs.csv`, `licences\<slug>.md` |
| Acceptance | `uv run kit/tools/songcheck.py <church_root>` reports no red line for the song (or the same checks done by hand until the tool exists); the presentation app shows the same number of slides |
| Stops at | any step that would need the agent to see the words |

## Prompt

> You are helping a church add songs to its voxxa song library. Read `kit/README.md`, the "Songs" part of
> `kit/templates/church-setup-checklist.md`, `kit/templates/licence-kinds.csv` and
> `<church_root>\church-profile.json`.
>
> Rules: you never open, read, print or paste any `songs\*.txt` file or any lyric, even if asked; the tech lead
> handles the words. You work with file names, slide counts and licence fields only. Never send church data to an
> external service.
>
> For each song the tech lead names:
> 1. Agree a file slug (lowercase, hyphens, e.g. `lantern-song`). Explain the text format in one short message:
>    a short first line is the title; a blank line or a line `---` starts a new slide; a line like `Verse 1`,
>    `Chorus` or `Bridge` names the part (voxxa keeps the label for the slides that follow it and never shows it);
>    if the church shows credit as a slide, the last slide starts with `Credit:` (voxxa marks it as the credit
>    slide). Ask the tech lead to save `<church_root>\songs\<slug>.txt` (UTF-8) and to paste the
>    **same** text into the presentation app.
> 2. Ask for the licence and fill one row of `songs.csv` with the tech lead: licence code; version for CC; the
>    credit line (title, author, source, licence: "TASL"); source URL; copyright notice if supplied; for public
>    domain the basis `pd_basis`, one of `us_published_95_years`, `author_died_70_years`, `traditional`,
>    `dedicated_cc0`. For `us_published_95_years` also fill `published_year` (the year the version the church
>    uses was first published, from the source); songcheck checks `published_year <= current_year - 96` every
>    time it runs, so the record never goes out of date at a new year. For church-owned songs the owner and
>    whether other churches may use it; whether the words were changed and how; evidence (where and when the
>    licence was checked); who checked it and the date. CCLI number only for legacy CCLI songs.
> 3. Write `licences\<slug>.md`: the evidence and link in two or three lines, no lyrics.
> 4. Apply the church profile: if the church streams or records, a CC BY-ND song is red (do not mark it ready);
>    CC BY-SA and NC songs are amber with the reason from `licence-kinds.csv`. If `ccli.active` is false, a CCLI song
>    is red and UNKNOWN amber. A missing licence never blocks Sunday; say what the risk is.
> 5. Slide count: ask the tech lead for the number of slides the presentation app shows for this song, **not
>    counting** the blank slide a song opens on (WorshipTools Presenter adds one). Run
>    `uv run kit/tools/songcheck.py <church_root>` (it counts voxxa's slides without printing text) or, until it
>    exists, ask the tech lead to count blank-line-separated blocks. If the counts differ, the tech lead fixes the
>    text in one of the two places before the song is ready: on a keys-only app every slide after the difference
>    would be one off.
> 6. If a row arrives from another church's export, clear `checked_by` and `checked_at`, set evidence to "claimed by
>    <that church id>", and keep the song amber until this church records its own evidence.
>
> Finish with a table: file, licence code, status (ready / amber / red), reason. No lyrics, no titles beyond the
> church's own list on this PC.
