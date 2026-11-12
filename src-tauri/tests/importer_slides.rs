//! Importers on untrusted and slide-numbering-sensitive files (Q2-01, Q2-05,
//! Q2-06). Every slide the presentation app has must stay a slide, empty or
//! not, so next/prev arithmetic and goto indices line up; and no file may
//! panic out of an importer (a panic in a Tauri command aborts the app).
//! The PDFs come from tests/fixtures/pdf/make_pdfs.py (public-domain text).

use std::io::{Cursor, Write};
use std::path::PathBuf;

fn pdf(name: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pdf")
        .join(name);
    std::fs::read(path).unwrap()
}

fn texts(song: &voxxa_lib::Song) -> Vec<&str> {
    song.slides.iter().map(|s| s.text.as_str()).collect()
}

#[test]
fn malformed_pdfs_return_an_error_instead_of_panicking() {
    for name in ["unknown-font.pdf", "mediabox-3.pdf"] {
        let result = std::panic::catch_unwind(|| voxxa_lib::pdf_parse(&pdf(name), "t"));
        let parsed = result.unwrap_or_else(|_| panic!("{name}: the importer panicked"));
        assert!(parsed.is_err(), "{name} should not parse");
    }
}

#[test]
fn pdf_pages_become_slides_in_order() {
    let song = voxxa_lib::pdf_parse(&pdf("three-pages.pdf"), "Amazing Grace").unwrap();
    assert_eq!(
        texts(&song),
        [
            "Amazing grace how sweet the sound",
            "Through many dangers toils and snares",
            "The Lord has promised good to me",
        ]
    );
    let song = voxxa_lib::pdf_parse(&pdf("blank-middle.pdf"), "t").unwrap();
    assert_eq!(song.slides.len(), 3);
    assert!(
        song.slides[1].text.is_empty(),
        "the empty page stays a slide"
    );
    let ids: Vec<usize> = song.slides.iter().map(|s| s.id).collect();
    assert_eq!(ids, [0, 1, 2]);
}

fn pptx(slides: &[&str]) -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default();
    for (i, text) in slides.iter().enumerate() {
        let body = if text.is_empty() {
            String::new()
        } else {
            format!("<a:p><a:r><a:t>{text}</a:t></a:r></a:p>")
        };
        zip.start_file(format!("ppt/slides/slide{}.xml", i + 1), options)
            .unwrap();
        let xml = format!(
            r#"<?xml version="1.0"?><p:sld xmlns:a="a" xmlns:p="p"><p:txBody>{body}</p:txBody></p:sld>"#
        );
        zip.write_all(xml.as_bytes()).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

#[test]
fn pptx_keeps_a_blank_middle_slide() {
    let bytes = pptx(&["Amazing grace how sweet the sound", "", "I once was lost"]);
    let song = voxxa_lib::pptx_parse(&bytes, "t").unwrap();
    assert_eq!(
        texts(&song),
        ["Amazing grace how sweet the sound", "", "I once was lost"]
    );
    assert!(voxxa_lib::pptx_parse(&pptx(&["", ""]), "t").is_err());
}

#[test]
fn pro7_keeps_a_repeated_chorus() {
    let block = |t: &str| format!("{{\\rtf1 {t}\\par}}");
    let mut bytes = b"\x00header\x00".to_vec();
    for text in [
        "When peace like a river attendeth my way",
        "It is well it is well with my soul",
        // The stage-display copy of the same slide, stored right after it.
        "It is well it is well with my soul",
        "Though Satan should buffet though trials should come",
        "It is well it is well with my soul",
    ] {
        bytes.extend_from_slice(block(text).as_bytes());
        bytes.extend_from_slice(b"\x01pad\x01");
    }
    let song = voxxa_lib::pro7_parse(&bytes, "It Is Well").unwrap();
    assert_eq!(song.slides.len(), 4, "{:?}", texts(&song));
    assert_eq!(song.slides[3].text, "It is well it is well with my soul");
}
