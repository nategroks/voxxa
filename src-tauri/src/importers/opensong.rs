//! OpenSong XML parser.
//!
//! OpenSong stores lyrics as a single `<lyrics>` text blob using inline
//! conventions instead of structured tags:
//!   - `[V1]`, `[C]`, `[B]`, `[T]`, `[O]`, … start a section
//!   - A line beginning with `.` is a chord row — skip
//!   - A line beginning with `;` is a comment — skip
//!   - A line beginning with a digit followed by a space is a presentation
//!     order group marker — we treat it as text
//!   - Blank lines inside a section start a new slide

use crate::aligner::{Slide, Song};
use anyhow::{anyhow, Result};
use quick_xml::events::Event;
use quick_xml::reader::Reader;

pub fn parse(text: &str, fallback_title: &str) -> Result<Song> {
    let (title, lyrics) = extract_title_and_lyrics(text)?;
    let title = if title.trim().is_empty() {
        fallback_title.to_string()
    } else {
        title.trim().to_string()
    };

    let mut slides: Vec<Slide> = Vec::new();
    let mut current = String::new();
    let mut have_started = false; // True once we see any non-chord, non-comment content.

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

    for raw in lyrics.lines() {
        let line = raw.trim_end();
        let stripped = line.trim_start();

        // Section header `[V1]`, `[C]`, etc — start a fresh slide.
        if stripped.starts_with('[') && stripped.ends_with(']') && stripped.len() <= 8 {
            finalize(&mut current, &mut slides);
            have_started = true;
            continue;
        }
        // Chord row — leading `.` followed by chord names; skip.
        if stripped.starts_with('.') {
            continue;
        }
        // Comment row.
        if stripped.starts_with(';') {
            continue;
        }
        // Blank line inside a section is a slide break.
        if stripped.is_empty() {
            if have_started && !current.is_empty() {
                finalize(&mut current, &mut slides);
            }
            continue;
        }

        have_started = true;
        // Strip the OpenSong leading-space prefix that aligns lyrics under chords.
        let body = line.trim_start_matches(' ');
        if !current.is_empty() {
            current.push('\n');
        }
        current.push_str(body);
    }
    finalize(&mut current, &mut slides);

    if slides.is_empty() {
        return Err(anyhow!("OpenSong song has no lyric content"));
    }
    Ok(Song { title, slides })
}

fn extract_title_and_lyrics(text: &str) -> Result<(String, String)> {
    let mut reader = Reader::from_str(text);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();
    let mut title = String::new();
    let mut lyrics = String::new();
    let mut in_title = false;
    let mut in_lyrics = false;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match e.local_name().as_ref() {
                b"title" => in_title = true,
                b"lyrics" => in_lyrics = true,
                _ => {}
            },
            Ok(Event::End(e)) => match e.local_name().as_ref() {
                b"title" => in_title = false,
                b"lyrics" => in_lyrics = false,
                _ => {}
            },
            Ok(Event::Text(t)) => {
                let s = t.unescape().unwrap_or_default().into_owned();
                if in_title {
                    title.push_str(&s);
                }
                if in_lyrics {
                    lyrics.push_str(&s);
                }
            }
            Ok(Event::CData(c)) => {
                if in_lyrics {
                    lyrics.push_str(&String::from_utf8_lossy(c.as_ref()));
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("OpenSong parse error: {e}")),
            _ => {}
        }
        buf.clear();
    }
    Ok((title, lyrics))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_opensong() {
        let xml = r#"<song>
<title>Amazing Grace</title>
<lyrics>[V1]
Amazing grace, how sweet the sound
That saved a wretch like me

[C]
How great Thou art</lyrics>
</song>"#;
        let song = parse(xml, "fallback").unwrap();
        assert_eq!(song.title, "Amazing Grace");
        assert_eq!(song.slides.len(), 2);
        assert!(song.slides[0].text.contains("Amazing grace"));
        assert!(song.slides[1].text.contains("How great Thou art"));
    }

    #[test]
    fn skips_chord_and_comment_rows() {
        let xml = r#"<song>
<title>T</title>
<lyrics>[V1]
.G       D
Amazing grace
;this is a comment
how sweet the sound</lyrics>
</song>"#;
        let song = parse(xml, "fallback").unwrap();
        assert_eq!(song.slides.len(), 1);
        assert!(!song.slides[0].text.contains("G       D"));
        assert!(!song.slides[0].text.contains("comment"));
    }
}
