//! Hymnary.org public-domain hymn lookup.
//!
//! Hymnary doesn't publish a formal REST API — they expose JSON exports on
//! their normal page URLs via `?export=json` (and `?format=json` for newer
//! routes). This is the documented public mechanism described in §3.4 of
//! the plan: "JSON API for public-domain hymn metadata and partial lyrics.
//! No auth, generous rate limits."
//!
//! Lyric *availability* is per-hymn — Hymnary surfaces full text only for
//! hymns whose copyright has expired. Modern worship lyrics generally are
//! NOT here; this is a backstop for liturgical churches singing hymns
//! whose authors died before 1929 (the U.S. public-domain cutoff).

use crate::aligner::{Slide, Song};
use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const BASE: &str = "https://hymnary.org";

pub struct HymnaryClient {
    client: Client,
}

impl HymnaryClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            // Hymnary's CDN sometimes gates anonymous traffic by user-agent;
            // a real-looking UA string keeps the responses identical to what
            // a browser sees.
            .user_agent("Voxxa/0.1 (+https://github.com/nategroks/voxxa)")
            .build()?;
        Ok(Self { client })
    }

    /// Search by text query. Returns the top matches with their hymnary slug
    /// for follow-up fetches.
    pub async fn search(&self, query: &str) -> Result<Vec<HymnaryResult>> {
        let url = format!(
            "{BASE}/search?qu={}&export=json",
            urlencoding(query)
        );
        crate::net_stats::record_request();
        let res = self.client.get(&url).send().await?;
        if !res.status().is_success() {
            return Err(anyhow!("hymnary search returned {}", res.status()));
        }
        let body: SearchResponse = res.json().await?;
        Ok(body
            .results
            .into_iter()
            .filter_map(|r| {
                Some(HymnaryResult {
                    slug: r.id.or(r.slug)?,
                    title: r.title.unwrap_or_else(|| "(untitled)".into()),
                    author: r.author,
                    year: r.year,
                })
            })
            .collect())
    }

    /// Fetch full lyrics for a hymn by its Hymnary slug. Returns an error if
    /// Hymnary doesn't have full text (typically because the hymn is still
    /// in copyright).
    pub async fn fetch_song(&self, slug: &str) -> Result<Song> {
        // The /text/{slug} endpoint with ?export=json returns the parsed
        // hymn body. For older slugs Hymnary sometimes only honors the
        // plain ?format=text variant; we try JSON first, then fall back.
        let json_url = format!("{BASE}/text/{slug}?export=json");
        crate::net_stats::record_request();
        let res = self.client.get(&json_url).send().await?;
        if res.status().is_success() {
            if let Ok(body) = res.json::<TextResponse>().await {
                if let Some(text) = body.text.filter(|t| !t.trim().is_empty()) {
                    return song_from_text(
                        &body.title.unwrap_or_else(|| slug.to_string()),
                        &text,
                    );
                }
            }
        }
        // Fallback: scrape plain text export.
        let txt_url = format!("{BASE}/text/{slug}?format=text");
        crate::net_stats::record_request();
        let res = self.client.get(&txt_url).send().await?;
        if !res.status().is_success() {
            return Err(anyhow!(
                "hymnary returned {} for {slug}",
                res.status()
            ));
        }
        let text = res.text().await?;
        if text.trim().is_empty() {
            return Err(anyhow!(
                "Hymnary has no full text for {slug} — typically the hymn is still in copyright."
            ));
        }
        song_from_text(slug, &text)
    }
}

fn song_from_text(title: &str, text: &str) -> Result<Song> {
    // Hymnary exports verses separated by blank lines. Numeric leading
    // tokens like "1." or "2)" mark stanza numbers — we strip them so they
    // don't end up on the projection slide.
    let mut slides: Vec<Slide> = Vec::new();
    let mut current: Vec<String> = Vec::new();
    for raw in text.lines() {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            if !current.is_empty() {
                slides.push(Slide {
                    id: slides.len(),
                    text: current.join("\n"),
                });
                current.clear();
            }
            continue;
        }
        current.push(strip_stanza_prefix(trimmed).to_string());
    }
    if !current.is_empty() {
        slides.push(Slide {
            id: slides.len(),
            text: current.join("\n"),
        });
    }
    if slides.is_empty() {
        return Err(anyhow!("hymnary text body was empty"));
    }
    Ok(Song {
        title: title.to_string(),
        slides,
    })
}

/// "1. Amazing grace" → "Amazing grace". Leaves anything that isn't a
/// stanza marker untouched.
fn strip_stanza_prefix(line: &str) -> &str {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == 0 || i > 3 {
        return line;
    }
    // Optional `.` or `)` after the digits.
    if i < bytes.len() && matches!(bytes[i], b'.' | b')') {
        i += 1;
    } else {
        return line;
    }
    // Then whitespace.
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    &line[i..]
}

/// Minimal URL-encoder for query strings. Avoids pulling in `url` just for
/// the search builder.
fn urlencoding(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    #[serde(default)]
    results: Vec<SearchResultRaw>,
}

#[derive(Debug, Deserialize)]
struct SearchResultRaw {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    slug: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    author: Option<String>,
    #[serde(default)]
    year: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TextResponse {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    text: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HymnaryResult {
    pub slug: String,
    pub title: String,
    pub author: Option<String>,
    pub year: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_numbered_stanzas() {
        assert_eq!(strip_stanza_prefix("1. Amazing grace"), "Amazing grace");
        assert_eq!(strip_stanza_prefix("12) verse twelve"), "verse twelve");
        assert_eq!(strip_stanza_prefix("Verse 1"), "Verse 1");
        assert_eq!(strip_stanza_prefix("Amazing grace, how"), "Amazing grace, how");
    }

    #[test]
    fn url_encodes_basics() {
        assert_eq!(urlencoding("amazing grace"), "amazing+grace");
        assert_eq!(urlencoding("how great"), "how+great");
        assert_eq!(urlencoding("café"), "caf%C3%A9");
    }

    #[test]
    fn song_from_text_splits_on_blank_lines() {
        let body = "1. Amazing grace, how sweet the sound\nThat saved a wretch like me\n\n2. I once was lost, but now am found\nWas blind, but now I see";
        let song = song_from_text("Amazing Grace", body).unwrap();
        assert_eq!(song.slides.len(), 2);
        assert!(song.slides[0].text.starts_with("Amazing grace"));
        assert!(song.slides[1].text.starts_with("I once was lost"));
    }
}
