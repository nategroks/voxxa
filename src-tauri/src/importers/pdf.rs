//! PDF importer.
//!
//! Each PDF page becomes one slide. Text is extracted via `pdf-extract` which
//! handles PDF text encodings (Identity-H, ToUnicode mappings) that the naive
//! "concatenate /T strings" approach gets wrong on real-world worship PDFs.
//!
//! Limitations:
//!   - Image-only PDFs (scanned bulletins) extract no text. Caller sees an
//!     empty-slides error.
//!   - Multi-column layouts read left-to-right line-by-line, which is usually
//!     wrong for two-column hymn sheets. Worship-Together-style single-column
//!     lyric PDFs are the happy path.

use crate::aligner::{Slide, Song};
use anyhow::{anyhow, Result};

pub fn parse(bytes: &[u8], fallback_title: &str) -> Result<Song> {
    // pdf-extract returns a single string with `\u{c}` (form feed) between
    // pages. Splitting on form-feed gives us one block per PDF page.
    let text = pdf_extract::extract_text_from_mem(bytes)
        .map_err(|e| anyhow!("pdf parse failed: {e}"))?;

    let mut slides: Vec<Slide> = Vec::new();
    for page in text.split('\u{c}') {
        let body = page.trim().to_string();
        if body.is_empty() {
            continue;
        }
        slides.push(Slide {
            id: slides.len(),
            text: body,
        });
    }
    if slides.is_empty() {
        return Err(anyhow!(
            "PDF contained no extractable text (scanned image? OCR needed)"
        ));
    }
    Ok(Song {
        title: fallback_title.to_string(),
        slides,
    })
}
