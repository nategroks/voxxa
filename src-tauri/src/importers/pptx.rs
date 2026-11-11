//! PowerPoint .pptx importer.
//!
//! A .pptx is a zip archive; slide text lives in `ppt/slides/slide{N}.xml` as
//! `<a:t>` elements inside `<a:p>` paragraphs inside `<a:r>` runs. One PPTX slide
//! becomes one Voxxa slide. Slide ordering follows the natural sort of the
//! file names, which matches PowerPoint's authoring order.

use crate::aligner::{Slide, Song};
use anyhow::{anyhow, Context, Result};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::io::{Cursor, Read};

pub fn parse(bytes: &[u8], fallback_title: &str) -> Result<Song> {
    let cursor = Cursor::new(bytes);
    let mut zip = zip::ZipArchive::new(cursor).context("not a valid .pptx archive")?;

    // Collect slide entries by their numeric suffix so slide10 comes after slide9.
    let mut entries: Vec<(u32, String)> = Vec::new();
    for i in 0..zip.len() {
        let name = match zip.by_index(i) {
            Ok(f) => match f.name() {
                Ok(name) => name.into_owned(),
                Err(_) => continue,
            },
            Err(_) => continue,
        };
        if let Some(num) = slide_number(&name) {
            entries.push((num, name));
        }
    }
    entries.sort_by_key(|(n, _)| *n);
    if entries.is_empty() {
        return Err(anyhow!("no ppt/slides/slide*.xml entries in archive"));
    }

    let mut slides: Vec<Slide> = Vec::new();
    for (_, name) in entries {
        let mut file = zip
            .by_name(&name)
            .with_context(|| format!("reading {name}"))?;
        let mut xml = String::new();
        file.read_to_string(&mut xml)
            .with_context(|| format!("decoding {name} as UTF-8"))?;
        let text = extract_slide_text(&xml)?;
        slides.push(Slide {
            id: slides.len(),
            text: text.trim().to_string(),
        });
    }
    // Drop blank slides (title-only, image-only, etc.) — they would confuse the
    // conductor by making the song match score against empty strings.
    slides.retain(|s| !s.text.is_empty());
    for (i, s) in slides.iter_mut().enumerate() {
        s.id = i;
    }
    if slides.is_empty() {
        return Err(anyhow!("no readable text in any slide"));
    }
    Ok(Song {
        title: fallback_title.to_string(),
        slides,
    })
}

fn slide_number(name: &str) -> Option<u32> {
    let after = name.strip_prefix("ppt/slides/slide")?;
    let stem = after.strip_suffix(".xml")?;
    stem.parse().ok()
}

fn extract_slide_text(xml: &str) -> Result<String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();
    let mut out = String::new();
    let mut in_t = false;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"a:t" => in_t = true,
                // Paragraph break — separate runs with a newline so multi-line
                // slides stay multi-line.
                b"a:p" if !out.is_empty() && !out.ends_with('\n') => {
                    out.push('\n');
                }
                _ => {}
            },
            Ok(Event::End(e)) => {
                if e.name().as_ref() == b"a:t" {
                    in_t = false;
                }
            }
            Ok(ev @ (Event::Text(_) | Event::GeneralRef(_))) if in_t => {
                let s = super::xml_text(&ev);
                out.push_str(&s);
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("pptx slide XML parse error: {e}")),
            _ => {}
        }
        buf.clear();
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    use zip::{CompressionMethod, ZipWriter};

    fn slide_xml(paragraphs: &[&str]) -> String {
        let body: String = paragraphs
            .iter()
            .map(|p| format!("<a:p><a:r><a:t>{p}</a:t></a:r></a:p>"))
            .collect();
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?><p:sld xmlns:a="a" xmlns:p="p"><p:cSld><p:spTree><p:sp><p:txBody>{body}</p:txBody></p:sp></p:spTree></p:cSld></p:sld>"#
        )
    }

    fn pptx(files: &[(&str, String)]) -> Vec<u8> {
        let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        for (name, xml) in files {
            zip.start_file(*name, options).unwrap();
            zip.write_all(xml.as_bytes()).unwrap();
        }
        zip.finish().unwrap().into_inner()
    }

    #[test]
    fn reads_slides_in_order_and_unescapes_entities() {
        // Archive order is deliberately reversed: order must come from the names.
        let bytes = pptx(&[
            (
                "ppt/slides/slide2.xml",
                slide_xml(&["Through many dangers, toils &amp; snares"]),
            ),
            (
                "ppt/slides/slide1.xml",
                slide_xml(&[
                    "Amazing grace, how sweet the sound",
                    "That saved a wretch like me",
                ]),
            ),
            ("ppt/presentation.xml", "<p:presentation/>".to_string()),
        ]);
        let song = parse(&bytes, "Amazing Grace").unwrap();
        assert_eq!(song.title, "Amazing Grace");
        assert_eq!(song.slides.len(), 2);
        assert_eq!(
            song.slides[0].text,
            "Amazing grace, how sweet the sound\nThat saved a wretch like me"
        );
        assert_eq!(song.slides[1].text, "Through many dangers, toils & snares");
        assert_eq!((song.slides[0].id, song.slides[1].id), (0, 1));

        // slide10 sorts after slide9 (numeric, not lexical).
        let bytes = pptx(&[
            ("ppt/slides/slide10.xml", slide_xml(&["ten"])),
            ("ppt/slides/slide9.xml", slide_xml(&["nine"])),
        ]);
        let song = parse(&bytes, "t").unwrap();
        let texts: Vec<&str> = song.slides.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(texts, ["nine", "ten"]);
    }
}
