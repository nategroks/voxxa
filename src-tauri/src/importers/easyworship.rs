//! EasyWorship 6 SQLite song-library importer.
//!
//! EW6 stores its songs in a SQLite file (typically `SongsWords.db` /
//! `Songs.db` under `Profiles\Default\Data\`). EW7 switched to Firebird, so
//! this importer is EW6-specific. The schema:
//!
//!   CREATE TABLE song (
//!     rowid INTEGER PRIMARY KEY,
//!     title TEXT,
//!     author TEXT,
//!     copyright TEXT,
//!     administrator TEXT,
//!     words TEXT,           -- the slide content, often RTF
//!     ...
//!   );
//!
//! In practice the column names vary across EW6 minor versions — some installs
//! have `words`, others `lyrics`, others `song_words`. We probe the available
//! columns and pick whichever lyric-bearing one exists. Each song row in the
//! file becomes one Voxxa Song; we split the lyric text on blank lines for
//! slide boundaries (same convention as the .txt importer).

use crate::aligner::{Slide, Song};
use anyhow::{anyhow, Context, Result};
use rusqlite::{Connection, OpenFlags};
use std::io::Write;

/// Parse all songs out of an EW6 SQLite file. rusqlite needs a path on disk,
/// so we stage the uploaded bytes into a tempfile first. The tempfile is
/// dropped (and unlinked) when this function returns.
pub fn parse_database(bytes: &[u8]) -> Result<Vec<Song>> {
    let mut tmp = tempfile::Builder::new()
        .prefix("voxxa-ew6-")
        .suffix(".db")
        .tempfile()
        .context("creating SQLite tempfile")?;
    tmp.write_all(bytes).context("writing tempfile")?;
    tmp.flush().ok();

    // Open read-only so a malformed query can't corrupt the user's library
    // even if they accidentally pointed us at the wrong file.
    let conn = Connection::open_with_flags(
        tmp.path(),
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .with_context(|| format!("opening {} as SQLite", tmp.path().display()))?;

    let table = find_song_table(&conn)?;
    let cols = list_columns(&conn, &table)?;
    let lyric_col = first_present(
        &cols,
        &["words", "lyrics", "song_words", "song_text", "content"],
    )
    .ok_or_else(|| {
        anyhow!(
            "no lyric column on table {table} (have: {})",
            cols.join(", ")
        )
    })?;
    let title_col = first_present(&cols, &["title", "song_title", "name"])
        .ok_or_else(|| anyhow!("no title column on table {table}"))?;

    let sql = format!(
        "SELECT {title_col}, {lyric_col} FROM {table} WHERE {lyric_col} IS NOT NULL"
    );
    let mut stmt = conn.prepare(&sql).context("preparing SELECT")?;
    let mut rows = stmt.query([]).context("executing SELECT")?;

    let mut out = Vec::new();
    let mut slide_id_offset = 0usize;
    while let Some(row) = rows.next().context("reading row")? {
        let title: Option<String> = row.get(0).ok();
        let title = title
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "Untitled".to_string());

        // EW stores `words` as either plain text (TEXT column) or an RTF blob
        // (BLOB column). Try text first since modern builds use TEXT; fall back
        // to bytes for older databases with BLOB lyrics.
        let lyric_text = match row.get::<_, String>(1) {
            Ok(s) => {
                if s.starts_with("{\\rtf") {
                    super::pro7::rtf_to_text(s.as_bytes())
                } else {
                    s
                }
            }
            Err(_) => {
                let raw: Vec<u8> = row.get(1).context("reading lyric column")?;
                if raw.starts_with(b"{\\rtf") {
                    super::pro7::rtf_to_text(&raw)
                } else {
                    String::from_utf8_lossy(&raw).into_owned()
                }
            }
        };

        let slides = split_into_slides(&lyric_text, slide_id_offset);
        if slides.is_empty() {
            continue;
        }
        slide_id_offset += slides.len();
        out.push(Song { title, slides });
    }
    if out.is_empty() {
        return Err(anyhow!(
            "{} found in the file but every row had empty lyrics",
            table
        ));
    }
    Ok(out)
}

fn find_song_table(conn: &Connection) -> Result<String> {
    // EW6 always has `song`, but some community-exported variants use `songs`
    // or `tblSongs`. Probe for the first one that exists.
    let candidates = ["song", "songs", "tblSongs", "Song"];
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table'")
        .context("listing tables")?;
    let rows: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(0))?
        .filter_map(|r| r.ok())
        .collect();
    for c in candidates {
        if rows.iter().any(|t| t.eq_ignore_ascii_case(c)) {
            return Ok(c.to_string());
        }
    }
    Err(anyhow!(
        "no recognisable song table (saw: {})",
        rows.join(", ")
    ))
}

fn list_columns(conn: &Connection, table: &str) -> Result<Vec<String>> {
    // PRAGMA doesn't accept bound params; the table name is one of our
    // hard-coded candidates so interpolation is safe.
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .context("listing columns")?;
    let names: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(1))?
        .filter_map(|r| r.ok())
        .collect();
    Ok(names)
}

fn first_present(have: &[String], want: &[&str]) -> Option<String> {
    for w in want {
        if let Some(found) = have.iter().find(|h| h.eq_ignore_ascii_case(w)) {
            return Some(found.clone());
        }
    }
    None
}

fn split_into_slides(text: &str, start_id: usize) -> Vec<Slide> {
    // Same convention as the .txt importer: blank lines separate slides, also
    // accept `---` / `===` as explicit breaks. We don't try to strip section
    // labels — EW lyrics rarely include them in the words column.
    let mut blocks: Vec<Vec<&str>> = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    for raw in text.lines() {
        let trimmed = raw.trim();
        if trimmed == "---" || trimmed == "===" || trimmed.is_empty() {
            if !current.is_empty() {
                blocks.push(std::mem::take(&mut current));
            }
            continue;
        }
        current.push(raw);
    }
    if !current.is_empty() {
        blocks.push(current);
    }
    blocks
        .into_iter()
        .map(|b| b.join("\n").trim().to_string())
        .filter(|s| !s.is_empty())
        .enumerate()
        .map(|(i, text)| Slide {
            id: start_id + i,
            text,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn make_db() -> tempfile::NamedTempFile {
        let f = tempfile::Builder::new()
            .prefix("voxxa-ew6-test-")
            .suffix(".db")
            .tempfile()
            .unwrap();
        let conn = Connection::open(f.path()).unwrap();
        conn.execute_batch(
            "CREATE TABLE song(rowid INTEGER PRIMARY KEY, title TEXT, words TEXT);
             INSERT INTO song(title, words) VALUES
               ('Amazing Grace', 'Amazing grace, how sweet the sound\nThat saved a wretch like me\n\nI once was lost, but now am found'),
               ('How Great', 'V1\nThe splendor of the King\n\nC\nHow great is our God');",
        )
        .unwrap();
        drop(conn);
        f
    }

    #[test]
    fn imports_two_songs() {
        let f = make_db();
        let bytes = std::fs::read(f.path()).unwrap();
        let songs = parse_database(&bytes).unwrap();
        assert_eq!(songs.len(), 2);
        assert_eq!(songs[0].title, "Amazing Grace");
        assert_eq!(songs[0].slides.len(), 2);
        assert!(songs[0].slides[0].text.starts_with("Amazing grace"));
    }
}
