//! Optional section labels on a song (`sections[].kind`, setlist.json v1;
//! ADR-016 Decision 1). A setlist without them loads and serialises exactly
//! as before.

use crate::aligner::Song;
use serde::{Deserialize, Serialize};

/// What a section is. `refrain` reads as `Chorus`; any other unknown string
/// reads as `Other`, so a newer writer never breaks an older Voxxa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SectionKind {
    Verse,
    #[serde(alias = "refrain")]
    Chorus,
    Bridge,
    Tag,
    #[serde(other)]
    Other,
}

/// One labelled part of a song: its kind and its slides (0-based indices
/// within the song, ascending).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Section {
    pub kind: SectionKind,
    pub slides: Vec<usize>,
}

/// Reject labels the conductor cannot use: a section with no slides, slides
/// out of order, a slide past the song's end, or a slide in two sections.
pub fn check_sections(song: &Song) -> Result<(), String> {
    let mut seen = vec![false; song.slides.len()];
    let bad = |what: String| Err(format!("Song \"{}\": {what}.", song.title));
    for section in &song.sections {
        let kind = format!("{:?}", section.kind).to_lowercase();
        if section.slides.is_empty() {
            return bad(format!("a {kind} section has no slides"));
        }
        if !section.slides.windows(2).all(|w| w[0] < w[1]) {
            return bad(format!("a {kind} section lists its slides out of order"));
        }
        for &i in &section.slides {
            match seen.get_mut(i) {
                None => {
                    return bad(format!(
                        "section slide {i} is past its {} slides",
                        seen.len()
                    ))
                }
                Some(true) => return bad(format!("slide {i} is in two sections")),
                Some(slot) => *slot = true,
            }
        }
    }
    Ok(())
}

fn section_of(song: &Song, slide: usize) -> Option<&Section> {
    song.sections.iter().find(|s| s.slides.contains(&slide))
}

/// The first slide of its section; every slide of an unlabelled song, and
/// every slide outside a section, counts as one.
pub(super) fn is_start(song: &Song, slide: usize) -> bool {
    section_of(song, slide).is_none_or(|s| s.slides.first() == Some(&slide))
}

pub(super) fn is_chorus(song: &Song, slide: usize) -> bool {
    section_of(song, slide).is_some_and(|s| s.kind == SectionKind::Chorus)
}

/// Both slides are in the same labelled section.
pub(super) fn same_section(song: &Song, a: usize, b: usize) -> bool {
    section_of(song, a).is_some_and(|s| s.slides.contains(&b))
}
