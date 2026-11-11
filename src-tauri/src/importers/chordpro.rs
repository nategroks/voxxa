//! ChordPro parser.
//!
//! Recognises:
//!   - `{title: ...}` / `{t: ...}` → title
//!   - `{start_of_chorus}` / `{soc}`, `{start_of_verse}` / `{sov}`,
//!     `{start_of_bridge}` / `{sob}` → start a new slide (and end the previous)
//!   - `{end_of_*}` / `{eoc}` / `{eov}` / `{eob}` → end the current slide
//!   - Other directives `{...}` → ignored
//!   - `[X]` inline chord markers → stripped from lyric output
//!   - Blank lines → slide break

use crate::aligner::{Slide, Song};
use anyhow::{anyhow, Result};

pub fn parse(text: &str, fallback_title: &str) -> Result<Song> {
    let mut title = String::new();
    let mut slides: Vec<Slide> = Vec::new();
    let mut current = String::new();

    let finalize = |current: &mut String, slides: &mut Vec<Slide>| {
        let body = current.trim().to_string();
        if !body.is_empty() {
            slides.push(Slide {
                id: slides.len(),
                text: body,
            });
        }
        current.clear();
    };

    for raw in text.lines() {
        let line = raw.trim_end();
        let trimmed = line.trim();

        if trimmed.is_empty() {
            finalize(&mut current, &mut slides);
            continue;
        }

        // Directives: `{key: value}` or `{flag}`.
        if let Some(rest) = trimmed.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
            let (key, value) = match rest.find(':') {
                Some(i) => (rest[..i].trim().to_lowercase(), rest[i + 1..].trim()),
                None => (rest.trim().to_lowercase(), ""),
            };
            match key.as_str() {
                "title" | "t" => {
                    if !value.is_empty() {
                        title = value.to_string();
                    }
                }
                "start_of_chorus" | "soc" | "start_of_verse" | "sov"
                | "start_of_bridge" | "sob" | "start_of_tab" | "sot" => {
                    // Section start ends whatever we were building.
                    finalize(&mut current, &mut slides);
                }
                "end_of_chorus" | "eoc" | "end_of_verse" | "eov"
                | "end_of_bridge" | "eob" | "end_of_tab" | "eot" => {
                    finalize(&mut current, &mut slides);
                }
                // Comment / subtitle / artist / album / key / tempo / time / etc — ignore.
                _ => {}
            }
            continue;
        }

        // Lyric line — strip inline chord brackets.
        let cleaned = strip_chords(line);
        if !cleaned.trim().is_empty() {
            if !current.is_empty() {
                current.push('\n');
            }
            current.push_str(&cleaned);
        }
    }
    finalize(&mut current, &mut slides);

    let title = if title.trim().is_empty() {
        fallback_title.to_string()
    } else {
        title.trim().to_string()
    };
    if slides.is_empty() {
        return Err(anyhow!("ChordPro song has no lyric content"));
    }
    Ok(Song { title, slides })
}

/// Strip `[Chord]` markers from a lyric line. ChordPro chord names can include
/// letters, digits, `#`, `b`, `/`, `m`, `sus`, etc — we don't validate; we just
/// drop anything between matching square brackets.
fn strip_chords(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut depth = 0u32;
    for c in line.chars() {
        match c {
            '[' => depth += 1,
            ']' if depth > 0 => depth -= 1,
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_chord_markers() {
        assert_eq!(strip_chords("Amazing [G]grace, how [D]sweet"), "Amazing grace, how sweet");
    }

    #[test]
    fn parses_title_and_sections() {
        let s = "{title: Amazing Grace}\n\nAmazing [G]grace\nHow [D]sweet\n\n{start_of_chorus}\nHow [C]great Thou [G]art\n{end_of_chorus}\n";
        let song = parse(s, "fallback").unwrap();
        assert_eq!(song.title, "Amazing Grace");
        assert_eq!(song.slides.len(), 2);
        assert!(song.slides[1].text.contains("How great Thou art"));
    }
}
