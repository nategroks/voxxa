//! Plain-text song parser.
//!
//! Conventions accepted (in order of priority):
//!   - `---` or `===` on a line by itself is an explicit slide break.
//!   - Otherwise a blank line ends the current slide.
//!   - A leading line that looks like a section label (Verse 1, Chorus, Bridge…)
//!     is stripped — it's not meant to go on the slide. It names the section
//!     (`sections`, ADR-016) of the slides that follow it, up to the next
//!     label; a label with no slide after it is dropped.
//!   - A slide whose first line starts with `Credit:` is the credit slide
//!     (`kind: "credit"`, spec § 4.5) and belongs to no section.
//!   - If the very first non-empty block is a single short line, it's treated
//!     as the song title; otherwise the caller's `fallback_title` is used.

use crate::aligner::evidence::{Section, SectionKind};
use crate::aligner::{Slide, Song};
use anyhow::{anyhow, Result};

pub fn parse(text: &str, fallback_title: &str) -> Result<Song> {
    let mut blocks: Vec<Vec<&str>> = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    for raw in text.lines() {
        let trimmed = raw.trim();
        if trimmed == "---" || trimmed == "===" {
            if !current.is_empty() {
                blocks.push(std::mem::take(&mut current));
            }
            continue;
        }
        if trimmed.is_empty() {
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
    if blocks.is_empty() {
        return Err(anyhow!("empty text file"));
    }

    // First block, single line, not a section label → title.
    let (title, content_blocks): (String, Vec<Vec<&str>>) =
        if blocks[0].len() == 1 && section_label(blocks[0][0]).is_none() {
            (blocks[0][0].trim().to_string(), blocks[1..].to_vec())
        } else {
            (fallback_title.to_string(), blocks)
        };

    let mut slides: Vec<Slide> = Vec::new();
    let mut sections: Vec<Section> = Vec::new();
    // Whether the newest section still takes the slides that follow.
    let mut open = false;
    for block in content_blocks {
        let body = match section_label(block[0]) {
            Some(kind) => {
                sections.push(Section {
                    kind,
                    slides: Vec::new(),
                });
                open = true;
                &block[1..]
            }
            None => &block[..],
        };
        let text = body.join("\n").trim().to_string();
        if text.is_empty() {
            continue;
        }
        let id = slides.len();
        let kind = text.starts_with("Credit:").then(|| "credit".to_string());
        if kind.is_some() {
            // The credit slide is no part of a sung section.
            open = false;
        } else if let (true, Some(section)) = (open, sections.last_mut()) {
            section.slides.push(id);
        }
        slides.push(Slide { id, text, kind });
    }
    sections.retain(|s| !s.slides.is_empty());

    if slides.is_empty() {
        return Err(anyhow!("no slides after parsing"));
    }
    Ok(Song {
        title,
        slides,
        sections,
    })
}

/// The section a label line names ("Verse 2", "Chorus:", "Pre-Chorus"), read
/// the way a setlist's `sections[].kind` is (ADR-016: `refrain` is a chorus,
/// every other accepted label is `Other`); `None` for a lyric line.
fn section_label(line: &str) -> Option<SectionKind> {
    let t = line.trim().to_lowercase();
    let t = t.trim_end_matches(':');
    // Strip trailing digits and spaces ("Verse 2" → "verse", "Chorus 1:" → "chorus").
    let bare = t.trim_end_matches(|c: char| c.is_ascii_digit() || c == ' ');
    Some(match bare {
        "verse" => SectionKind::Verse,
        "chorus" | "refrain" => SectionKind::Chorus,
        "bridge" => SectionKind::Bridge,
        "tag" => SectionKind::Tag,
        "pre-chorus" | "pre chorus" | "prechorus" | "intro" | "outro" | "ending" | "interlude" => {
            SectionKind::Other
        }
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_title_and_blocks() {
        let s = "Amazing Grace\n\nAmazing grace, how sweet the sound\nThat saved a wretch like me\n\nI once was lost, but now am found";
        let song = parse(s, "fallback").unwrap();
        assert_eq!(song.title, "Amazing Grace");
        assert_eq!(song.slides.len(), 2);
        assert!(song.slides[0].text.starts_with("Amazing grace"));
    }

    #[test]
    fn strips_section_labels() {
        let s = "It Is Well\n\nVerse 1\nWhen peace like a river attendeth my way\n\nChorus:\nIt is well with my soul";
        let song = parse(s, "fallback").unwrap();
        assert_eq!(song.slides.len(), 2);
        assert!(!song.slides[0].text.contains("Verse"));
        assert!(!song.slides[1].text.contains("Chorus"));
    }

    #[test]
    fn explicit_slide_break() {
        let s = "Song\n\nA\nB\n---\nC\nD";
        let song = parse(s, "fallback").unwrap();
        assert_eq!(song.slides.len(), 2);
    }
}
