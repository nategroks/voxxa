# ADR-026: Songs are typed in by the church with a licence and credit record; Creative Commons, public domain and church-owned first, CCLI optional (amends ADR-007, ADR-019 § 7, ADR-003's file list)

**Status:** Proposed (2026-11-24) · **Date:** 2026-11-24 · **Feature:** church-autoadvance · **Reversibility:** Easy (a register file and a check script; no runtime code) · **Owner:** nategroks · **Author:** 10x-Team synthesiser, from drafts 12, 13, 14 and review 20

## Context
- Founder decision (brief § 2): songs are Creative Commons-licensed, public domain or the church's own; lyrics are entered and managed by hand; each song records its licence and credit line; CCLI fields may stay optional.
- ADR-007 assumed lyrics come from the church's licensed software or exports and that voxxa writes `usage.csv` for CCLI reporting (`.10x/adrs/007-text-licensing-no-copyrighted-text-shipped.md:9`, `:12`); ADR-003 lists `usage.csv` as durable data (`.10x/adrs/003-single-config-and-jsonl-protocol.md:16`). `usage.csv` was never built.
- The `.txt` importer already turns a plain file into slides (blank line or `---` = new slide; section label lines stripped; a short first line = title) (`src-tauri/src/importers/txt.rs:1-9`); slides can be fixed by double-click and exported (`src/main.js:626-692`).
- voxxa never projects (ADR-011, `.10x/adrs/011-operator-previews-allowed-never-project-lyrics.md:13`): on-screen credit is the presentation app's job.
- Committed fixtures are public-domain only (`fixtures/README.md:266-267`).
- Review 20: CC 4.0 treats music synced with moving images as an adaptation, so a livestream of an ND song may not be allowed and SA songs carry ShareAlike duties **[UNVERIFIED legal]** (S-30); missing credit can terminate a CC licence (cured within 30 days of discovery) **[UNVERIFIED legal]** (S-31); copied rights from another church may be wrong (S-32); a church leaving CCLI still holds CCLI songs (S-33).

## Decision
1. **Lyrics:** one `songs\<slug>.txt` per song in the church's private folder, in the importer's format; the same text is pasted into the presentation app so both have the same slides. A credit slide, if the church uses one, starts with a `Credit:` line (so the importer marks it `kind: "credit"`, ADR-022 § 7) and is in both copies.
2. **Register:** `songs.csv` in the private folder, one row per `.txt`, columns
   `file,title,words_by,music_by,licence,licence_version,credit_line,source_url,copyright_notice,pd_basis,published_year,owner,owner_permits_sharing,modified,modification,evidence,ccli_song,presenter_slides,checked_by,checked_at`.
3. **Licence codes:** `PD`, `CC0`, `CC-BY`, `CC-BY-SA`, `CC-BY-NC`, `CC-BY-NC-SA`, `CC-BY-ND`, `CC-BY-NC-ND`, `CHURCH-OWNED`, `PERMISSION`, `CCLI` (optional, legacy), `UNKNOWN`. Policy is data (`licence-kinds.csv`): needs credit, allows adaptation, share-alike, non-commercial, OK when streamed, public-fixture OK, church-to-church sharing.
4. **Required fields:** CC codes need `licence_version`, `credit_line` (title, author, source, licence: TASL), `source_url` if supplied, and `modification` when `modified=1`; ND codes allow only typo fixes against the source; `PD` needs `pd_basis`, one of `us_published_95_years`, `author_died_70_years`, `traditional`, `dedicated_cc0`; `us_published_95_years` also needs `published_year`, and songcheck computes the rule `published_year <= current_year - 96` each time it runs, so the basis never goes stale at a new year; `CHURCH-OWNED` needs `owner`; `CCLI` needs `ccli_song`; every non-`UNKNOWN` row needs `evidence` (where and when the licence was checked). On keys-only apps `presenter_slides` must equal voxxa's slide count.
5. **Church profile** (`church-profile.json`): `streams`, `stream_monetised`, `asks_for_gifts_on_stream`, `ccli.active`, allowed codes. When the church streams or records: ND songs are red (not "ready"); SA and NC are amber with the reason; counsel before adopting ND or SA songs for streamed services. When `ccli.active` is false: `CCLI` songs red, `UNKNOWN` amber. Sunday is never blocked; the warning names the risk.
6. **Imports from another church** arrive with `checked_by` and `checked_at` empty and `evidence` = "claimed by <exporting church id>"; not "ready" until the church records its own evidence.
7. **Check:** `kit/tools/songcheck.py` (stdlib, PEP 723, `--self-test`) enforces 2–6 and prints file and field names only, never song text; exit 1 on a problem.
8. **Credit shown:** the service-day checklist lists every CC song in today's plan with "credit shown in <app>? [ ]"; the presentation app's footer or a credit slide carries the line.
9. **Public repo:** committed lyric fixtures and kit examples are `PD` or `CC0` only until the founder decides otherwise (FA-13); each committed lyric fixture carries a `LICENCE.json` and a test `fixtures_carry_an_allowed_licence` enforces it. `NC`, `ND`, `CHURCH-OWNED`, `CCLI` and `UNKNOWN` never.
10. **Amendments:**
    - ADR-007 `:9`: add "or typed by the church into its private folder, with a licence record"; still never in the binary or installer.
    - ADR-007 `:12` and ADR-003 `:16`: `usage.csv` is dropped; the dated service files are the usage record; a credits/usage export is a later kit tool (Q22).
    - ADR-007 `:13`: the written CCLI question becomes conditional on a church using CCLI songs with voxxa (FA-27).
    - ADR-019 § 7: "licensed test audio" becomes licence classes (private: CCLI, church-owned unless relicensed, NC, ND, UNKNOWN; may become public with written performer consent: PD, CC0, and later BY/BY-SA by founder decision); the public `fixtures/LICENSED_RUNS.md` template has the columns `alias`, `licence_class`, `numbers` only, with no church or machine column. `credit_line`, titles and authors live only in the private `<church_root>\reports\LICENSED_RUNS.md`, because a credit line names the song and its authors.
    - Records name private things by alias ("pilot church", "private song P1", "booth PC").

## Alternatives Considered
| Alternative | Pros | Cons | Why Not |
|---|---|---|---|
| In-app song library and editor with an SQLite store (drafts 12, 13) | "in voxxa" literally; per-licence forms | 1,050–1,700 GUI lines plus a store; ruled out by the strategy (`software-strategy.md:30`) | Notepad + register first; editor is founder item FA-14 |
| Licence inside each `.txt` as a header | one file per song | the importer would show the header as a slide | separate register |
| Keep CCLI fields required | familiar | the church is leaving CCLI | optional, legacy code |
| Allow BY-ND freely (draft 14) | more songs | livestream adaptation risk (S-30) | allowed only when the church does not stream or record |

## Consequences
### Positive
- Works with what exists today: no Rust change, no GUI change.
- A church can show any auditor where each song's licence came from.
### Negative
- Two copies of each song's text (voxxa and the presentation app) until read-back or #185 removes one; the slide-count check guards drift.
- Spreadsheet editing is less friendly than a form.
### Risks
- Legal readings (NC on monetised streams, ND/SA and the sync clause, cure periods) are unverified: the kit says so and recommends counsel.

## Dependencies
- Depends on: ADR-007, ADR-011, ADR-019, ADR-027 (uv for the check script), ADR-029 (private folder).
- Plan steps: V5.

## Review trigger
A rights holder's complaint; a church adopts NC or ND songs and streams; the founder requires in-app entry; CCLI's answer to the written question.
