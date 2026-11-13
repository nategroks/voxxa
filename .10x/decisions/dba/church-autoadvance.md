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

## 2026-11-13 — Q6: section labels in the setlist schema (`sections[].kind`, setlist.json v1; ADR-016 Decision 1)

**Decision: one optional field per song, `sections: [{"kind": "...", "slides": [i, ...]}]`, in `Song` (`src-tauri/src/aligner/mod.rs`); the types `Section` and `SectionKind` and the checks live in `src-tauri/src/aligner/evidence/sections.rs`. No version key, no migration.**

| Question | Answer |
|---|---|
| Where the data lives | Voxxa has no database; a setlist is a JSON file (native `.json`, `.voxxa-set`, `voxxad --setlist`, the eval fixtures). The conductor holds it in memory. |
| Backwards compatibility | `#[serde(default, skip_serializing_if = "Vec::is_empty")]`: a setlist without `sections` loads as before and serialises byte for byte as before, so the replay fingerprint (FNV-1a over the songs' JSON, ADR-013) of every existing setlist and session log is unchanged. Tested: `old_setlists_load_and_serialise_exactly_as_before`. |
| Forwards compatibility | `kind` is an enum with `#[serde(other)] Other`: an unknown kind ("pre-chorus") loads as `other`, so a newer writer never breaks an older reader. `refrain` is an alias of `chorus`. An older Voxxa reading a setlist with `sections` ignores the field (serde ignores unknown fields; `Song` does not deny them). |
| Integrity rules (checked when the setlist loads, in `check_setlist`, same error path as an empty song) | each section has at least one slide; its slide indices ascend; every index is inside the song; no slide is in two sections. Unlabelled slides are allowed. Tested: `bad_section_labels_are_refused_when_the_setlist_loads`. |
| Why indices and not per-slide labels | The prompt names `sections[].kind`; a section may span several slides; indices keep the slide list (and its numbering against the presentation app) untouched. |
| Fingerprint change when labels are added | Intended: labels change what the conductor does, so a log recorded without them must not replay against a labelled setlist. `refrain-it-is-well` gained labels in Q6 and its `session.jsonl` was re-recorded (`voxxa-eval --full --record`). |
| Who writes it | Nobody automatically yet: no importer emits `sections` (OpenSong `[V1]`/`[C]`, ChordPro and OpenLyrics carry the information; a follow-up). The GUI passes the field through unchanged (it forwards parsed song objects and shows the songs the backend returns); no GUI change. |
| Rust API | `Song` gained a public field, so struct literals need `sections: Vec::new()` (10 sites in importers, Hymnary and Planning Center were updated). `Conductor`'s methods, `Action`, `Input` and the event schema are unchanged. |
