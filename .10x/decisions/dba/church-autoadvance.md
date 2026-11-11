# DBA — church-autoadvance

## 2026-11-11 — Q1: SQLite in the EasyWorship 6 importer (`src-tauri/src/importers/easyworship.rs`)

**Decision: keep `rusqlite` with the `bundled` feature; move to `rusqlite` 0.41 (`libsqlite3-sys` 0.39.1, SQLite 3.54.1).**

| Question | Answer |
|---|---|
| What uses SQLite | Only the EasyWorship 6 importer. It reads a song library the operator picks (`.db`); Voxxa has no database of its own. |
| Bundled or system | **Bundled.** Windows ships no system `libsqlite3` to link against, and the importer must behave the same on all three OSes. Bundling also means a security fix is a crate bump that `cargo deny` can see, not a property of the user's OS. Cost: SQLite compiled into the binary (size not measured) and a C compile in CI (`cc`, no system headers). |
| Licence | SQLite is public domain. `rusqlite` and `libsqlite3-sys` are MIT. Both are GPL-3.0-or-later compatible and pass `deny.toml`. |
| Why 0.41 | RUSTSEC-2026-0459 (CVE-2026-119375 in C SQLite) affects `libsqlite3-sys` < 0.39.0. `rusqlite` 0.41 is the smallest release that depends on 0.39 (0.32 → 0.41; MSRV 1.89 ≤ our 1.90). No API change touched the importer. |
| How the file is opened | The picked bytes are copied to a tempfile (`voxxa-ew6-*.db`, unlinked on return) and opened `SQLITE_OPEN_READ_ONLY | SQLITE_OPEN_NO_MUTEX`. Only `type='table'` entries are considered, and every interpolated identifier (table, columns) is matched against a hard-coded allow-list first, so the `format!`-built SQL cannot be steered by the file's schema. |

**For Q2 (security review of untrusted files), not changed in Q1:**
- The file is untrusted input. Consider `PRAGMA trusted_schema = OFF`, `SQLITE_DBCONFIG_DEFENSIVE`, and opening with the `immutable=1` URI so SQLite never creates a journal/WAL next to the tempfile.
- A large library is read fully into memory twice (picked bytes, then rows); no size cap.
- EW7 (Firebird) is out of scope, as `docs/SETLIST_FORMATS.md` says.
