# Setlist file formats Voxxa accepts

Drop any of these onto the setlist loader — single file or many, mixed
types in one drop. The frontend routes each file to the right parser and
combines the results into one setlist.

| Extension | Format | One file = |
|---|---|---|
| `.json` | Native Voxxa setlist | A whole setlist (replaces current) |
| `.voxxa-set` | Voxxa export envelope | A whole setlist (replaces current) |
| `.txt` | Plain text | One song |
| `.openlyrics` / `.xml` | OpenLyrics XML | One song |
| `.xml` | OpenSong XML | One song (detected by root element) |
| `.cho` / `.chordpro` / `.pro` | ChordPro | One song |
| `.pptx` | PowerPoint | One song |
| `.pdf` | PDF | One song (one slide per page) |
| `.pro7` | ProPresenter 7 file | One song |
| `.db` | EasyWorship 6 SQLite library | Many songs |

You can also pull a whole service plan directly from **Planning Center
Services v2** via Settings → "Or import from Planning Center" — see
`README.md` for the Personal Access Token setup.

---

## Native JSON

The simplest format. Drop a `.json` file shaped like this:

```json
{
  "setlist": [
    {
      "title": "How Great Is Our God",
      "slides": [
        { "id": 0, "text": "The splendor of the King clothed in majesty" },
        { "id": 1, "text": "Age to age He stands and time is in His hands" },
        { "id": 2, "text": "Name above all names worthy of all praise" }
      ]
    }
  ]
}
```

Slide IDs are re-numbered on load; you can leave them as `0` or omit
them entirely (in the JS path they get reassigned).

## `.voxxa-set` envelope

Exported via the **Export** link in the song-title row. Same JSON as
the native format with an envelope:

```json
{
  "voxxa_set_version": 1,
  "exported_at": "2026-05-28T10:30:00Z",
  "setlist": [ ... ]
}
```

The version field exists so future format changes can be detected. For
now version 1 is "literally the native JSON wrapped".

## Plain text

```
Amazing Grace

Amazing grace, how sweet the sound
That saved a wretch like me

I once was lost, but now am found
Was blind, but now I see
```

Rules Voxxa applies:

- The first single-line block — if it's not a section label — is the
  song title. Otherwise the file name (minus extension) is used.
- Blank lines separate slides.
- A line containing only `---` or `===` is an explicit slide break.
- A leading section label (`Verse 1`, `Chorus`, `Bridge`, `Pre-Chorus`,
  `Tag`, `Outro`, `Intro`, `Refrain`, `Ending`, `Interlude` — with
  optional digit suffix and trailing colon) is **stripped** from the
  slide body. It's authoring metadata, not lyrics that should appear
  on screen.

## OpenLyrics XML

The format used by OpenLP. Each `<verse>` becomes one slide.

```xml
<?xml version="1.0" encoding="UTF-8"?>
<song xmlns="http://openlyrics.info/namespace/2009/song" version="0.8">
  <properties>
    <titles><title>Amazing Grace</title></titles>
  </properties>
  <lyrics>
    <verse name="v1">
      <lines>Amazing grace, how sweet the sound<br/>That saved a wretch like me</lines>
    </verse>
  </lyrics>
</song>
```

Notes:

- `<br/>` inside `<lines>` becomes a newline.
- Multi-part verses (`<lines part="men">`) are concatenated into one
  slide. If you want separate slides per part, use separate `<verse>`
  elements.

## OpenSong XML

OpenSong stores lyrics as a single `<lyrics>` blob with inline section
markers — no structured tags.

```xml
<song>
  <title>Amazing Grace</title>
  <lyrics>[V1]
Amazing grace, how sweet the sound
That saved a wretch like me

[C]
How great Thou art</lyrics>
</song>
```

Rules Voxxa applies:

- `[V1]`, `[C]`, `[B]`, `[T]`, … (short bracketed section markers) start
  a new slide.
- Lines starting with `.` are chord rows — skipped.
- Lines starting with `;` are comments — skipped.
- Blank lines within a section break into a new slide.

## ChordPro

A common chord-chart format used by SongSelect downloads, OnSong,
Chordify, and many other tools.

```
{title: Amazing Grace}
{artist: John Newton}

Amazing [G]grace, how [D]sweet the [G]sound
That [G7]saved a [C]wretch like [G]me

{start_of_chorus}
How [C]great Thou [G]art
{end_of_chorus}
```

Rules Voxxa applies:

- `{title: ...}` or `{t: ...}` → song title.
- `{start_of_*}` (`soc`, `sov`, `sob`, `sot`) and matching `{end_of_*}` →
  start / end a slide.
- All other `{directive}` lines (comments, key, tempo, artist…) are
  ignored.
- Inline `[Chord]` markers are stripped from lyric output.
- Blank lines break into slides.

## .pptx (PowerPoint)

Each PowerPoint slide becomes one Voxxa slide. Voxxa pulls text from
the `<a:t>` elements inside `ppt/slides/slide{N}.xml`. Slides with no
extractable text (image-only, title slides without text) are dropped.

Tips:

- Make sure your lyrics are real text boxes, not images of lyrics.
  Voxxa can't OCR.
- One verse / chorus per slide reads best for the smart-blanking
  conductor, which fuzzy-matches the audio against each slide's text.

## PDF

One PDF page becomes one Voxxa slide. Text is extracted page-by-page
via `pdf-extract` (which handles ToUnicode mappings and Identity-H
encoded PDFs correctly).

Limitations:

- Scanned / image-only PDFs (bulletin scans) extract no text. Voxxa
  shows an error in that case — OCR isn't included.
- Two-column lyric sheets read left-to-right line-by-line, which is
  usually wrong. Single-column Worship-Together-style lyric PDFs are
  the happy path.

## ProPresenter 7 (`.pro7`)

ProPresenter's native format. Voxxa pulls **lyrics only** — themes,
fonts, media references, and stage display layouts are discarded.

Each RTF blob inside the protobuf becomes one slide. Duplicate slides
(audience + stage display copies of the same text) are folded into one.
Blocks shorter than 3 characters are dropped — they're usually theme
labels or footer fields.

## EasyWorship 6 (`.db`)

The whole EW6 song library in a single file. Voxxa imports **every
song** in the database; you can prune the unwanted ones in Voxxa's
setlist editor afterward.

Voxxa probes for the song table (`song` / `songs` / `tblSongs`), the
lyric column (`words` / `lyrics` / `song_words` / `song_text` /
`content`), and the title column (`title` / `song_title` / `name`).
Lyrics stored as RTF are stripped to plain text.

**EasyWorship 7 is not supported** — it switched from SQLite to
Firebird. For EW7, export your songs to ChordPro or `.txt` first.

---

## Authoring tips

The smart-blanking conductor (§4 of the project plan) fuzzy-matches the
transcribed audio against each slide's text. To help it:

1. **Keep slides short.** One verse or one chorus per slide is ideal.
   A whole song on one slide is too much for the matcher.
2. **Match the slides to what's actually being sung.** If your bridge
   has a different lyric on the second pass, that should be a separate
   slide; otherwise the conductor will think you're still on the first.
3. **Don't put titles on lyric slides.** "Amazing Grace" is the song
   title, not lyric text. The first non-label line in a `.txt` file
   becomes the title automatically.
4. **Don't include chord annotations.** Voxxa strips `[G]` from
   ChordPro and `.` chord rows from OpenSong, but mixed-in chord
   names in plain text will confuse matching.

If a song misfires, the inline slide editor (double-click a slide card)
lets you tune the text on the fly. Edits apply immediately to the
conductor without restarting.
