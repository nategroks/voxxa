//! OpenLyrics XML parser (<http://openlyrics.info/>).
//!
//! Each `<verse>` becomes one slide. `<lines>` text content is the body, with
//! `<br/>` rendered as a newline. Title comes from the first `<title>` under
//! `<properties><titles>`. Multi-part verses (`<lines part="men">`) are
//! concatenated; we don't try to split into multiple slides per verse since
//! that's authoring intent the lyric author would have expressed as separate
//! `<verse>` elements if they meant it.

use crate::aligner::{Slide, Song};
use anyhow::{anyhow, Result};
use quick_xml::events::Event;
use quick_xml::reader::Reader;

pub fn parse(text: &str, fallback_title: &str) -> Result<Song> {
    let mut reader = Reader::from_str(text);
    reader.config_mut().trim_text(false);

    let mut buf = Vec::new();
    let mut title = String::new();
    let mut in_title = false;
    let mut in_lines = false;
    let mut in_verse = false;
    let mut current_text = String::new();
    let mut slides: Vec<Slide> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match e.local_name().as_ref() {
                b"title" => {
                    if title.is_empty() {
                        in_title = true;
                    }
                }
                b"verse" => {
                    in_verse = true;
                    current_text.clear();
                }
                b"lines" => {
                    in_lines = true;
                    if !current_text.is_empty() && !current_text.ends_with('\n') {
                        // Separator between consecutive <lines> blocks in the same verse.
                        current_text.push('\n');
                    }
                }
                _ => {}
            },
            Ok(Event::End(e)) => match e.local_name().as_ref() {
                b"title" => {
                    in_title = false;
                }
                b"lines" => {
                    in_lines = false;
                }
                b"verse" => {
                    in_verse = false;
                    let body = current_text.trim().to_string();
                    if !body.is_empty() {
                        slides.push(Slide {
                            id: slides.len(),
                            text: body,
                        });
                    }
                    current_text.clear();
                }
                _ => {}
            },
            // `<br/>` inside <lines> is the canonical OpenLyrics line break.
            Ok(Event::Empty(e)) => {
                if in_lines && e.local_name().as_ref() == b"br" {
                    current_text.push('\n');
                }
            }
            Ok(Event::Text(t)) => {
                let s = t.unescape().unwrap_or_default().into_owned();
                if in_title {
                    title.push_str(&s);
                }
                if in_verse && in_lines {
                    current_text.push_str(&s);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("OpenLyrics parse error: {e}")),
            _ => {}
        }
        buf.clear();
    }

    let title = if title.trim().is_empty() {
        fallback_title.to_string()
    } else {
        title.trim().to_string()
    };
    if slides.is_empty() {
        return Err(anyhow!("OpenLyrics song has no <verse> content"));
    }
    Ok(Song { title, slides })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_openlyrics() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<song xmlns="http://openlyrics.info/namespace/2009/song" version="0.8">
  <properties>
    <titles><title>Amazing Grace</title></titles>
  </properties>
  <lyrics>
    <verse name="v1"><lines>Amazing grace<br/>How sweet the sound</lines></verse>
    <verse name="c1"><lines>How great Thou art</lines></verse>
  </lyrics>
</song>"#;
        let song = parse(xml, "fallback").unwrap();
        assert_eq!(song.title, "Amazing Grace");
        assert_eq!(song.slides.len(), 2);
        assert!(song.slides[0].text.contains("Amazing grace"));
        assert!(song.slides[0].text.contains("How sweet"));
    }
}
