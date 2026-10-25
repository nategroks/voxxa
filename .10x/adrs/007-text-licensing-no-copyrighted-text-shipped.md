# ADR-007: Ship no copyrighted lyrics or Bibles; read lyrics from the church's licensed software; match scripture against public-domain texts

**Status:** Accepted · **Date:** 2026-10-24 · **Feature:** church-autoadvance · **Reversibility:** Easy · **Owner:** nategroks

## Context
The CCLI Church Copyright License lets a church store song files on its own computers for projection; all other rights are reserved (research B1). No authority covers using lyrics as a non-displayed matching index (B2). SongSelect's API is closed (B3). ESV and API.Bible free tiers are non-commercial (B7); public-domain Bibles with machine-readable downloads exist (B6).

## Decision
- Voxxa distributes **no lyrics** and **no copyrighted Bible text**. Lyrics are read at runtime from the adapter (the church's licensed library), from Planning Center Services (the church's account), or from a file the church exports. They live only in memory and in the church's own `setlist.json` cache on its machine.
- Voxxa **never displays lyrics**; it only emits slide positions. The presentation app displays under the church's licence.
- Bundled for **matching only**: World English Bible, KJV, ASV, Berean Standard Bible (public domain, with eBible.org/Berean attributions in About). No ESV/NIV/NLT text is fetched or stored.
- `usage.csv` (title, CCLI number if present, date) is written locally for the church's own reporting; Voxxa does not report to CCLI.
- Before launch, the founder sends CCLI a written description of index-only use and records the answer in `.10x/decisions/security/`. If CCLI objects, the fallback is unchanged in code: churches load setlists they are already licensed to store; Voxxa's position is that it adds no copy beyond the church's existing one.

## Alternatives
| Alternative | Why not |
|---|---|
| Bundle a lyric database | Infringing without a publisher licence; LyricFind/Musixmatch-class licences cost tens of thousands (B5 gap). |
| SongSelect API | Retired/closed (B3). |
| API.Bible for scripture text | Non-commercial tier conflicts with paid support; 30-day refresh and FUMS requirements fight the offline promise (B7). |

## Consequences
Good: no licensing dependency, aligns with the local-first promise, zero recurring text cost. Bad: paraphrase translations match less well against PD text; the spoken reference carries those cases. Risks: B2 is unsettled law; mitigated by the design (no copy, no display) and the written question.

## Review trigger
CCLI's written answer; any publisher contact; a church asking for a translation Voxxa cannot match well.
