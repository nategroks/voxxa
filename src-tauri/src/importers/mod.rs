//! Setlist importers.
//!
//! Each submodule parses a single source format into a [`crate::aligner::Song`].
//! The frontend collects N parsed songs, synthesises a `Setlist` JSON, and calls
//! `load_setlist` exactly like the existing JSON drop path.

use crate::aligner::Song;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

pub mod chordpro;
pub mod easyworship;
pub mod openlyrics;
pub mod opensong;
pub mod pdf;
pub mod pptx;
pub mod pro7;
pub mod txt;

/// Unescaped text of a quick-xml `Text` or `GeneralRef` event; empty for any
/// other event. quick-xml 0.38+ reports `&amp;`, `&#39;`, ... as separate
/// `GeneralRef` events instead of unescaping them inside `Text`.
fn xml_text(event: &quick_xml::events::Event) -> String {
    use quick_xml::events::Event;
    match event {
        Event::Text(t) => t.decode().map(|s| s.into_owned()).unwrap_or_default(),
        Event::GeneralRef(r) => match r.resolve_char_ref() {
            Ok(Some(c)) => c.to_string(),
            _ => r
                .decode()
                .ok()
                .and_then(|name| quick_xml::escape::resolve_predefined_entity(&name))
                .unwrap_or_default()
                .to_string(),
        },
        _ => String::new(),
    }
}

/// Source format. The frontend picks this from the file extension (or, for
/// `.xml`, by sniffing the root namespace).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportFormat {
    Txt,
    OpenLyrics,
    OpenSong,
    ChordPro,
    Pptx,
    Pdf,
    Pro7,
}

/// Parse a text-source song. Returns an error for binary formats.
pub fn parse_text(format: ImportFormat, content: &str, fallback_title: &str) -> Result<Song> {
    match format {
        ImportFormat::Txt => txt::parse(content, fallback_title),
        ImportFormat::OpenLyrics => openlyrics::parse(content, fallback_title),
        ImportFormat::OpenSong => opensong::parse(content, fallback_title),
        ImportFormat::ChordPro => chordpro::parse(content, fallback_title),
        ImportFormat::Pptx | ImportFormat::Pdf | ImportFormat::Pro7 => {
            Err(anyhow!("{:?} is binary — use parse_bytes", format))
        }
    }
}

/// Parse a binary-source song (.pptx, .pdf, .pro7).
pub fn parse_bytes(format: ImportFormat, bytes: &[u8], fallback_title: &str) -> Result<Song> {
    match format {
        ImportFormat::Pptx => pptx::parse(bytes, fallback_title),
        ImportFormat::Pdf => pdf::parse(bytes, fallback_title),
        ImportFormat::Pro7 => pro7::parse(bytes, fallback_title),
        _ => Err(anyhow!("{:?} expects text content", format)),
    }
}
