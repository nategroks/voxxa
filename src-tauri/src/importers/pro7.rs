//! ProPresenter `.pro7` importer.
//!
//! A `.pro7` file is a protobuf envelope wrapping per-slide RTF blobs (greyshirtguy/
//! ProPresenter7-Proto has the full schema). We **don't** decode the protobuf —
//! pulling in `prost` + a build-time `.proto` step is heavy for a one-direction
//! lyric extractor. Instead, we scan the raw bytes for `{\rtf1` markers, walk to
//! the matching closing brace, and run each block through a focused RTF
//! stripper. Each surviving non-trivial RTF block becomes one slide.
//!
//! Quirks the stripper handles:
//!   - Destination groups (`\fonttbl{...}`, `\colortbl{...}`, `\stylesheet{...}`,
//!     `\themedata{...}`, `\datastore{...}`, `\pict{...}`, `\info{...}`, …) get
//!     skipped wholesale — they hold formatting metadata, not lyrics.
//!   - `\*` introduces an "ignore if unknown" destination — also skipped.
//!   - `\u{N}` unicode escapes + a substitute fallback char.
//!   - `\'XX` hex byte escapes.
//!   - `\par`, `\line`, `\page`, `\tab` become whitespace.
//!
//! Deduplication: ProPresenter often stores the same lyric text twice (audience
//! view + stage display). Identical case-insensitive blocks are folded into one.

use crate::aligner::{Slide, Song};
use anyhow::{anyhow, Result};
use std::collections::HashSet;

pub fn parse(bytes: &[u8], fallback_title: &str) -> Result<Song> {
    let blocks = find_rtf_blocks(bytes);
    if blocks.is_empty() {
        return Err(anyhow!("no RTF blocks found in .pro7 file"));
    }
    let mut slides: Vec<Slide> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for rtf in blocks {
        let text = rtf_to_text(&rtf);
        let trimmed = text.trim().to_string();
        // Skip very short fragments — typically theme labels, transition titles,
        // or stage-display footer fields.
        if trimmed.len() < 3 {
            continue;
        }
        let key = trimmed.to_lowercase();
        if !seen.insert(key) {
            continue;
        }
        slides.push(Slide {
            id: slides.len(),
            text: trimmed,
        });
    }
    if slides.is_empty() {
        return Err(anyhow!("no readable lyric text in .pro7 RTF blocks"));
    }
    Ok(Song {
        title: fallback_title.to_string(),
        slides,
    })
}

/// Find every `{\rtf1` … matching `}` block in `bytes`. The outer brace is
/// part of the returned slice. Skips overlapping / nested matches.
fn find_rtf_blocks(bytes: &[u8]) -> Vec<Vec<u8>> {
    const NEEDLE: &[u8] = b"{\\rtf1";
    let mut out = Vec::new();
    let mut i = 0;
    while i + NEEDLE.len() <= bytes.len() {
        if &bytes[i..i + NEEDLE.len()] != NEEDLE {
            i += 1;
            continue;
        }
        let Some(end) = find_balanced_brace_end(&bytes[i..]) else {
            break;
        };
        out.push(bytes[i..i + end].to_vec());
        i += end;
    }
    out
}

/// Given a byte slice starting at `{`, return the index just past the matching
/// `}`. `None` if unbalanced.
fn find_balanced_brace_end(bytes: &[u8]) -> Option<usize> {
    let mut depth: i32 = 0;
    let mut escape = false;
    for (i, &b) in bytes.iter().enumerate() {
        if escape {
            escape = false;
            continue;
        }
        match b {
            b'\\' => {
                escape = true;
            }
            b'{' => {
                depth += 1;
            }
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i + 1);
                }
            }
            _ => {}
        }
    }
    None
}

/// Convert a single RTF blob to plain text. See module docs for details.
///
/// Shared with the EasyWorship importer, which encounters the same Microsoft
/// RTF dialect in its SQLite `words` columns.
pub(crate) fn rtf_to_text(rtf: &[u8]) -> String {
    let s = String::from_utf8_lossy(rtf);
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::new();
    let mut depth: i32 = 0;
    // Some(d) ⇒ we're inside a destination group that started at depth `d`
    // and should be discarded until it closes.
    let mut skip_at_depth: Option<i32> = None;
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        match c {
            '{' => {
                depth += 1;
                i += 1;
                // If the group's first non-whitespace token is a control word
                // we recognise as a destination, mark the whole group skipped.
                if skip_at_depth.is_none() {
                    let mut j = i;
                    while j < chars.len() && chars[j].is_whitespace() {
                        j += 1;
                    }
                    if j < chars.len() && chars[j] == '\\' {
                        j += 1;
                        // \* means "ignore if you don't grok the destination"
                        if j < chars.len() && chars[j] == '*' {
                            skip_at_depth = Some(depth);
                            i = j + 1;
                            continue;
                        }
                        let cw_start = j;
                        while j < chars.len() && chars[j].is_ascii_alphabetic() {
                            j += 1;
                        }
                        let cw: String = chars[cw_start..j].iter().collect();
                        if is_destination_group(&cw) {
                            skip_at_depth = Some(depth);
                        }
                    }
                }
            }
            '}' => {
                if let Some(d) = skip_at_depth {
                    if d == depth {
                        skip_at_depth = None;
                    }
                }
                depth -= 1;
                i += 1;
            }
            '\\' => {
                i += 1;
                if i >= chars.len() {
                    break;
                }
                // \* destination marker (handled in '{' branch when it's the
                // group's first token; here a bare \* just gets ignored).
                if chars[i] == '*' {
                    i += 1;
                    continue;
                }
                // Escaped literal — non-alphabetic single char follows \.
                if !chars[i].is_ascii_alphabetic() {
                    let lit = chars[i];
                    i += 1;
                    if skip_at_depth.is_some() || depth < 1 {
                        if lit == '\'' {
                            i += 2;
                        }
                        continue;
                    }
                    match lit {
                        '\\' => out.push('\\'),
                        '{' => out.push('{'),
                        '}' => out.push('}'),
                        '\'' => {
                            if i + 1 < chars.len() {
                                let hex: String = [chars[i], chars[i + 1]].iter().collect();
                                if let Ok(b) = u8::from_str_radix(&hex, 16) {
                                    // Hex byte escapes are usually Windows-1252
                                    // legacy text; we surface as Latin-1 which
                                    // is close enough for English worship lyrics.
                                    out.push(b as char);
                                }
                                i += 2;
                            }
                        }
                        '~' => out.push('\u{00A0}'), // non-breaking space
                        '-' => {} // optional hyphen — drop
                        _ => {}
                    }
                    continue;
                }
                // Control word.
                let cw_start = i;
                while i < chars.len() && chars[i].is_ascii_alphabetic() {
                    i += 1;
                }
                let cw: String = chars[cw_start..i].iter().collect();
                // Optional numeric parameter.
                let mut param = String::new();
                if i < chars.len() && chars[i] == '-' {
                    param.push('-');
                    i += 1;
                }
                while i < chars.len() && chars[i].is_ascii_digit() {
                    param.push(chars[i]);
                    i += 1;
                }
                // Optional single-space delimiter.
                if i < chars.len() && chars[i] == ' ' {
                    i += 1;
                }
                if skip_at_depth.is_some() {
                    continue;
                }
                match cw.as_str() {
                    "par" | "line" | "page" => out.push('\n'),
                    "tab" => out.push('\t'),
                    "u" if !param.is_empty() => {
                        if let Ok(n) = param.parse::<i32>() {
                            let n = if n < 0 { 65536 + n } else { n };
                            if let Some(uc) = char::from_u32(n as u32) {
                                out.push(uc);
                            }
                        }
                        // Skip the ASCII fallback character that follows.
                        if i < chars.len()
                            && chars[i] != '\\'
                            && chars[i] != '{'
                            && chars[i] != '}'
                            && !chars[i].is_whitespace()
                        {
                            i += 1;
                        }
                    }
                    _ => {}
                }
            }
            _ => {
                if skip_at_depth.is_none() && depth >= 1 {
                    out.push(c);
                }
                i += 1;
            }
        }
    }
    out
}

fn is_destination_group(cw: &str) -> bool {
    matches!(
        cw,
        "fonttbl"
            | "colortbl"
            | "stylesheet"
            | "info"
            | "themedata"
            | "datastore"
            | "operator"
            | "filetbl"
            | "listtables"
            | "rsidtbl"
            | "generator"
            | "pict"
            | "header"
            | "footer"
            | "fldinst"
            | "shppict"
            | "nonshppict"
            | "background"
            | "shp"
            | "object"
            | "result"
            | "title"
            | "author"
            | "company"
            | "comment"
            | "version"
            | "revtim"
            | "creatim"
            | "doccomm"
            | "buptim"
            | "printim"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_basic_rtf() {
        let rtf = br"{\rtf1\ansi{\fonttbl\f0 Helvetica;}\f0\fs24 Amazing grace\par how sweet the sound}";
        let text = rtf_to_text(rtf);
        assert_eq!(text.trim(), "Amazing grace\nhow sweet the sound");
    }

    #[test]
    fn unicode_escape() {
        let rtf = br"{\rtf1 H\u233?llo\par}";
        let text = rtf_to_text(rtf);
        assert_eq!(text.trim(), "Héllo");
    }

    #[test]
    fn dedupes_identical_blocks() {
        // Two identical RTF blocks → one slide.
        let inner = br"{\rtf1 same text\par}";
        let mut buf = Vec::new();
        buf.extend_from_slice(b"PROTOBUF_HEADER_BYTES");
        buf.extend_from_slice(inner);
        buf.extend_from_slice(b"\x00\x01padding\x00\x01");
        buf.extend_from_slice(inner);
        let song = parse(&buf, "Test Song").unwrap();
        assert_eq!(song.slides.len(), 1);
        assert!(song.slides[0].text.contains("same text"));
    }

    #[test]
    fn extracts_multiple_distinct_blocks() {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"\x00garbage\x00");
        buf.extend_from_slice(br"{\rtf1 first slide\par}");
        buf.extend_from_slice(b"\xff middle bytes \xff");
        buf.extend_from_slice(br"{\rtf1 second slide here}");
        let song = parse(&buf, "T").unwrap();
        assert_eq!(song.slides.len(), 2);
    }
}
