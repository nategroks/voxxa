//! Regression tests for the defects fixed in Q2 (.10x/reviews/2026-11-11-ponytail-q0.md §4).
//! Each test names the defect it guards.

use std::path::{Path, PathBuf};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn rust_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// ADR-007: no copyrighted lyrics in the sources, test fixtures included.
#[test]
fn no_copyrighted_lyric_fixtures_in_sources() {
    // Split so this file does not match itself.
    let banned = [
        ["way", " maker"].concat(),
        ["how great", " is our god"].concat(),
    ];
    let mut files = Vec::new();
    rust_sources(&manifest_dir().join("src"), &mut files);
    for f in files {
        let text = std::fs::read_to_string(&f).unwrap().to_lowercase();
        for b in &banned {
            assert!(!text.contains(b.as_str()), "{} contains '{b}'", f.display());
        }
    }
}

/// A minimal PDF with one text line per page. `font` is the font name the
/// content stream uses; the page resources only define /F1.
fn pdf(pages: &[&str], font: &str, media_box: &str) -> Vec<u8> {
    let kids: Vec<String> = (0..pages.len())
        .map(|i| format!("{} 0 R", 4 + 2 * i))
        .collect();
    let mut objs = vec![
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        format!(
            "<< /Type /Pages /Kids [{}] /Count {} >>",
            kids.join(" "),
            pages.len()
        ),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string(),
    ];
    for (i, text) in pages.iter().enumerate() {
        let content = if text.is_empty() {
            String::new()
        } else {
            format!("BT /{font} 18 Tf 72 720 Td ({text}) Tj ET")
        };
        objs.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox {media_box} \
             /Resources << /Font << /F1 3 0 R >> >> /Contents {} 0 R >>",
            5 + 2 * i
        ));
        objs.push(format!(
            "<< /Length {} >>\nstream\n{content}\nendstream",
            content.len()
        ));
    }
    let mut out = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for (i, o) in objs.iter().enumerate() {
        offsets.push(out.len());
        out.extend(format!("{} 0 obj\n{o}\nendobj\n", i + 1).bytes());
    }
    let xref = out.len();
    out.extend(format!("xref\n0 {}\n0000000000 65535 f \n", objs.len() + 1).bytes());
    for off in offsets {
        out.extend(format!("{off:010} 00000 n \n").bytes());
    }
    out.extend(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objs.len() + 1
        )
        .bytes(),
    );
    out
}

const LETTER: &str = "[0 0 612 792]";

/// Q2-06: one slide per page, in page order.
#[test]
fn pdf_pages_become_slides_in_order() {
    let bytes = pdf(
        &[
            "Amazing grace how sweet the sound",
            "Twas grace that taught my heart to fear",
            "When we have been there ten thousand years",
        ],
        "F1",
        LETTER,
    );
    let song = voxxa_lib::pdf_parse(&bytes, "Amazing Grace").unwrap();
    let texts: Vec<&str> = song.slides.iter().map(|s| s.text.as_str()).collect();
    assert_eq!(texts.len(), 3, "{texts:?}");
    assert!(texts[0].contains("Amazing grace"));
    assert!(texts[1].contains("taught my heart"));
    assert!(texts[2].contains("ten thousand years"));
}

/// Q2-05/Q2-06: an empty page stays a slide so slide numbers match the deck.
#[test]
fn pdf_empty_page_is_kept_as_empty_slide() {
    let bytes = pdf(
        &["Be thou my vision", "", "Riches I heed not"],
        "F1",
        LETTER,
    );
    let song = voxxa_lib::pdf_parse(&bytes, "T").unwrap();
    assert_eq!(song.slides.len(), 3);
    assert!(song.slides[1].text.is_empty());
    assert!(song.slides[2].text.contains("Riches"));
}

/// Q2-01: malformed PDFs that make pdf-extract panic return an error instead.
#[test]
fn malformed_pdfs_return_err_instead_of_panicking() {
    let unknown_font = pdf(&["Holy holy holy"], "F9", LETTER);
    let short_media_box = pdf(&["Holy holy holy"], "F1", "[0 0 612]");
    for bytes in [unknown_font, short_media_box] {
        assert!(voxxa_lib::pdf_parse(&bytes, "T").is_err());
    }
}
