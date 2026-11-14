# dba — Index

## Active features
- `church-autoadvance` — Q1 (2026-11-11): EasyWorship 6 importer keeps bundled SQLite via `rusqlite` 0.41 (RUSTSEC-2026-0459); see `church-autoadvance.md`.
  - Q6 (2026-11-13): optional `sections[].kind` per song in setlist v1 (ADR-016 Decision 1); no version key, old setlists serialise byte for byte, labels checked on load; no importer writes them yet (Q6-06).
