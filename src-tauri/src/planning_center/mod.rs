//! Planning Center Services v2 API client.
//!
//! Authentication: Personal Access Token only for Phase 1. The user creates a
//! token at <https://api.planningcenteronline.com/oauth/applications> and
//! supplies the `application_id` + `secret` pair. We send them as HTTP Basic.
//! OAuth 2.0 (for distributing Voxxa as a multi-tenant app) is Phase 2.
//!
//! The API speaks JSON:API 1.0. We deliberately keep deserialization loose —
//! attributes come back as `serde_json::Value` and we extract the fields we
//! actually use — because PCO has historically added fields without warning
//! and we don't want a deserialization error to break import.

use crate::aligner::{Slide, Song};
use anyhow::{anyhow, Result};
use base64::Engine;
use reqwest::Client;
use serde::Serialize;
use std::time::Duration;

const BASE: &str = "https://api.planningcenteronline.com/services/v2";

pub struct PcoClient {
    client: Client,
    auth_header: String,
}

impl PcoClient {
    pub fn new(application_id: &str, secret: &str) -> Result<Self> {
        if application_id.is_empty() || secret.is_empty() {
            return Err(anyhow!("application_id and secret are required"));
        }
        let token = base64::engine::general_purpose::STANDARD
            .encode(format!("{application_id}:{secret}"));
        let auth_header = format!("Basic {token}");
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()?;
        Ok(Self {
            client,
            auth_header,
        })
    }

    async fn get(&self, path: &str) -> Result<serde_json::Value> {
        let url = format!("{BASE}{path}");
        let res = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .header("Accept", "application/vnd.api+json")
            .send()
            .await?;
        let status = res.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(anyhow!(
                "Planning Center rejected the credentials (401). Double-check the application ID and secret."
            ));
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(anyhow!(
                "Planning Center rate-limited the request. Wait a minute and try again."
            ));
        }
        if !status.is_success() {
            let body = res.text().await.unwrap_or_default();
            return Err(anyhow!("PCO {path}: {status} — {body}"));
        }
        let json: serde_json::Value = res.json().await?;
        Ok(json)
    }

    /// Hit `/me` as a cheap credential check. Returns the authenticated user's
    /// display name.
    pub async fn verify(&self) -> Result<String> {
        let resp = self.get("/me").await?;
        let name = resp["data"]["attributes"]["name"]
            .as_str()
            .unwrap_or("(unknown)")
            .to_string();
        Ok(name)
    }

    pub async fn list_service_types(&self) -> Result<Vec<PcoServiceType>> {
        let resp = self.get("/service_types?per_page=100").await?;
        let data = resp["data"].as_array().cloned().unwrap_or_default();
        Ok(data
            .into_iter()
            .map(|d| PcoServiceType {
                id: str_or_empty(&d["id"]),
                name: str_or_empty(&d["attributes"]["name"]),
            })
            .filter(|s| !s.id.is_empty())
            .collect())
    }

    pub async fn list_plans(&self, service_type_id: &str) -> Result<Vec<PcoPlan>> {
        // `filter=future` keeps the list small and relevant; `order=sort_date`
        // surfaces this Sunday before later weeks.
        let path = format!(
            "/service_types/{service_type_id}/plans?filter=future&per_page=50&order=sort_date"
        );
        let resp = self.get(&path).await?;
        let data = resp["data"].as_array().cloned().unwrap_or_default();
        Ok(data
            .into_iter()
            .map(|d| PcoPlan {
                id: str_or_empty(&d["id"]),
                title: str_or_empty(&d["attributes"]["title"]),
                dates: str_or_empty(&d["attributes"]["dates"]),
                sort_date: str_or_empty(&d["attributes"]["sort_date"]),
            })
            .filter(|p| !p.id.is_empty())
            .collect())
    }

    /// Walk plan items in order; for each song-type item, pull the arrangement
    /// lyrics out of the sideloaded `included` block and parse them into slides.
    pub async fn import_plan(
        &self,
        service_type_id: &str,
        plan_id: &str,
    ) -> Result<Vec<Song>> {
        let path = format!(
            "/service_types/{service_type_id}/plans/{plan_id}/items?include=arrangement&per_page=100&order=sequence"
        );
        let resp = self.get(&path).await?;
        let items = resp["data"].as_array().cloned().unwrap_or_default();
        let included = resp["included"].as_array().cloned().unwrap_or_default();

        let mut songs = Vec::new();
        let mut next_id = 0usize;
        for item in items {
            if item["attributes"]["item_type"].as_str() != Some("song") {
                continue;
            }
            let title = item["attributes"]["title"]
                .as_str()
                .filter(|s| !s.is_empty())
                .unwrap_or("Untitled")
                .to_string();
            let arr_id = item["relationships"]["arrangement"]["data"]["id"].as_str();
            let lyrics = arr_id
                .and_then(|id| find_arrangement_lyrics(&included, id))
                .unwrap_or_default();
            if lyrics.trim().is_empty() {
                log::info!(
                    "[PCO] skipping {title}: no arrangement lyrics on the plan item"
                );
                continue;
            }
            match parse_pco_lyrics(&lyrics, &title, next_id) {
                Ok(song) => {
                    next_id += song.slides.len();
                    songs.push(song);
                }
                Err(e) => log::warn!("[PCO] {title}: lyric parse failed: {e}"),
            }
        }
        if songs.is_empty() {
            return Err(anyhow!(
                "Plan loaded but contained no song items with parseable lyrics."
            ));
        }
        Ok(songs)
    }
}

fn str_or_empty(v: &serde_json::Value) -> String {
    v.as_str().unwrap_or_default().to_string()
}

fn find_arrangement_lyrics(included: &[serde_json::Value], id: &str) -> Option<String> {
    for inc in included {
        if inc["type"].as_str() == Some("Arrangement") && inc["id"].as_str() == Some(id) {
            return inc["attributes"]["lyrics"].as_str().map(String::from);
        }
    }
    None
}

/// Parse a PCO arrangement `lyrics` field into slides.
///
/// PCO's convention: blank lines separate slides; a line by itself matching a
/// short section code (V1, V2, C, B1, T, P, I, O, E, M, R, F — case-insensitive,
/// optional trailing digits or `:`) is a label and is stripped from the slide
/// body.
fn parse_pco_lyrics(lyrics: &str, title: &str, start_id: usize) -> Result<Song> {
    let mut blocks: Vec<Vec<&str>> = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    for raw in lyrics.lines() {
        if raw.trim().is_empty() {
            if !current.is_empty() {
                blocks.push(std::mem::take(&mut current));
            }
            continue;
        }
        current.push(raw);
    }
    if !current.is_empty() {
        blocks.push(current);
    }

    let mut slides: Vec<Slide> = Vec::new();
    for block in blocks {
        let body: Vec<&str> = if !block.is_empty() && looks_like_pco_label(block[0]) {
            block[1..].to_vec()
        } else {
            block
        };
        let text = body.join("\n").trim().to_string();
        if !text.is_empty() {
            slides.push(Slide {
                id: start_id + slides.len(),
                text,
            });
        }
    }
    if slides.is_empty() {
        return Err(anyhow!("no slides after parsing"));
    }
    Ok(Song {
        title: title.to_string(),
        slides,
    })
}

fn looks_like_pco_label(line: &str) -> bool {
    let mut t = line.trim().to_lowercase();
    while t.ends_with(':') {
        t.pop();
    }
    // Reject anything with whitespace — labels are single-token by convention.
    if t.contains(char::is_whitespace) {
        return false;
    }
    let chars: Vec<char> = t.chars().collect();
    if chars.is_empty() || chars.len() > 4 {
        return false;
    }
    if !"vcbtpioemrf".contains(chars[0]) {
        return false;
    }
    chars[1..].iter().all(|c| c.is_ascii_digit())
}

#[derive(Debug, Serialize)]
pub struct PcoServiceType {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct PcoPlan {
    pub id: String,
    pub title: String,
    pub dates: String,
    pub sort_date: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pco_label_matching() {
        assert!(looks_like_pco_label("V1"));
        assert!(looks_like_pco_label("v1"));
        assert!(looks_like_pco_label("V"));
        assert!(looks_like_pco_label("C"));
        assert!(looks_like_pco_label("C1:"));
        assert!(looks_like_pco_label("B"));
        assert!(looks_like_pco_label("T"));
        assert!(!looks_like_pco_label("Verse 1"));
        assert!(!looks_like_pco_label("Amazing"));
        assert!(!looks_like_pco_label("V1 the"));
        assert!(!looks_like_pco_label("XX"));
    }

    #[test]
    fn parses_pco_lyrics() {
        let lyrics = "V1\nAmazing grace, how sweet the sound\nThat saved a wretch like me\n\nC\nHow great Thou art\nHow great Thou art\n\nV2\nI once was lost, but now am found";
        let song = parse_pco_lyrics(lyrics, "Amazing Grace", 0).unwrap();
        assert_eq!(song.title, "Amazing Grace");
        assert_eq!(song.slides.len(), 3);
        assert!(song.slides[0].text.starts_with("Amazing grace"));
        assert!(song.slides[1].text.starts_with("How great"));
        assert!(song.slides[2].text.starts_with("I once was lost"));
    }
}
