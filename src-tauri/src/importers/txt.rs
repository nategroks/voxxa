//! Plain-text song parser.
//!
//! Conventions accepted (in order of priority):
//!   - `---` or `===` on a line by itself is an explicit slide break.
//!   - Otherwise a blank line ends the current slide.
//!   - A leading line that looks like a section label (Verse 1, Chorus, Bridge…)
//!     is stripped — it's not meant to go on the slide.
//!   - If the very first non-empty block is a single short line, it's treated
//!     as the song title; otherwise the caller's `fallback_title` is used.

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
        if blocks[0].len() == 1 && !looks_like_section_label(blocks[0][0]) {
            (blocks[0][0].trim().to_string(), blocks[1..].to_vec())
        } else {
            (fallback_title.to_string(), blocks)
        };

    let slides: Vec<Slide> = content_blocks
        .into_iter()
        .map(|block| {
            let body: Vec<&str> = if !block.is_empty() && looks_like_section_label(block[0]) {
                block[1..].to_vec()
            } else {
                block
            };
            body.join("\n").trim().to_string()
        })
        .filter(|s| !s.is_empty())
        .enumerate()
        .map(|(id, text)| Slide { id, text })
        .collect();

    if slides.is_empty() {
        return Err(anyhow!("no slides after parsing"));
    }
    Ok(Song { title, slides })
}

fn looks_like_section_label(line: &str) -> bool {
    let t = line.trim().to_lowercase();
    let t = t.trim_end_matches(':');
    // Strip trailing digits and spaces ("Verse 2" → "verse", "Chorus 1:" → "chorus").
    let bare = t.trim_end_matches(|c: char| c.is_ascii_digit() || c == ' ');
    matches!(
        bare,
        "verse"
            | "chorus"
            | "bridge"
            | "pre-chorus"
            | "pre chorus"
            | "prechorus"
            | "tag"
            | "outro"
            | "intro"
            | "refrain"
            | "ending"
            | "interlude"
    )
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
        let s = "How Great\n\nVerse 1\nThe splendor of the King\n\nChorus:\nHow great is our God";
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
