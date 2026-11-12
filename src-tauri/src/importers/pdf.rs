//! PDF importer.
//!
//! Each PDF page becomes one slide, in page order. A page with no text stays
//! as an empty slide, so slide numbers match the deck in the presentation app;
//! the conductor never matches empty slides. Text is extracted via
//! `pdf-extract`, which handles PDF text encodings (Identity-H, ToUnicode
//! mappings) that the naive "concatenate /T strings" approach gets wrong.
//!
//! The PDF is untrusted input and pdf-extract panics on some malformed files
//! (unknown font names, short MediaBox arrays, ...). The panic is caught here
//! and becomes an ordinary error, so a bad file cannot abort the app.
//!
//! Limitations:
//!   - Image-only PDFs (scanned bulletins) extract no text. Caller sees an
//!     empty-slides error.
//!   - Multi-column layouts read left-to-right line-by-line, which is usually
//!     wrong for two-column hymn sheets. Worship-Together-style single-column
//!     lyric PDFs are the happy path.

use crate::aligner::{Slide, Song};
use anyhow::{anyhow, Result};
use std::panic::{catch_unwind, AssertUnwindSafe};

pub fn parse(bytes: &[u8], fallback_title: &str) -> Result<Song> {
    let pages = catch_unwind(AssertUnwindSafe(|| {
        pdf_extract::extract_text_from_mem_by_pages(bytes)
    }))
    .map_err(|_| anyhow!("this PDF could not be read"))?
    .map_err(|e| anyhow!("pdf parse failed: {e}"))?;

    let slides: Vec<Slide> = pages
        .iter()
        .enumerate()
        .map(|(id, page)| Slide {
            id,
            text: page.trim().to_string(),
        })
        .collect();
    if slides.iter().all(|s| s.text.is_empty()) {
        return Err(anyhow!(
            "PDF contained no extractable text (scanned image? OCR needed)"
        ));
    }
    Ok(Song {
        title: fallback_title.to_string(),
        slides,
    })
}
