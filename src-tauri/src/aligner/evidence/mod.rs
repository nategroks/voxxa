//! ADR-009's minimum (Q6): which slide of the current song the room is on.
//!
//! Every recogniser hop scores each slide of the song on the last
//! [`RECENT`] recognised words, word tokens compared by the conductor's LCS,
//! and folds the score, weighted by the transition's prior from
//! [`SmartConfig`], into a decaying evidence per slide. The conductor moves
//! only when another slide's evidence is high, clearly above every other
//! slide's, and the current slide has been up for `min_dwell_ms`. Any slide
//! can win, so a repeated chorus and a jump are followed, not only "next".

use super::{lcs_length, SmartConfig, Song};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Words of the transcript scored each hop.
const RECENT: usize = 12;
/// Words a slide is scored on from its start: a move lands on a slide's
/// first words, and a chorus that repeats a verse's last line must not
/// count as that verse.
const LEAD: usize = 24;

/// A labelled run of slides, from `start` to the next section's start
/// (`setlist.json` v1: `"sections":[{"kind":"chorus","start":1}]`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Section {
    /// `verse`, `chorus`, `bridge`, `tag`, … A `chorus` or `refrain` is
    /// expected to come back, so it gets `prior_chorus`.
    pub kind: String,
    /// Its first slide, 0-based within the song.
    pub start: usize,
}

/// Evidence per slide of the song the conductor is in.
#[derive(Debug, Default)]
pub struct Tracker {
    /// The (song, slide) the evidence was gathered at. The conductor at any
    /// other position (a move, the operator, a new song) starts it afresh.
    at: (usize, usize),
    evidence: Vec<f64>,
}

impl Tracker {
    /// One hop: `words` were heard at slide `at.1` of `song`, which has been
    /// up for `dwell` (`showing`), or is the slide behind a blank screen.
    /// Returns the slide to move to, if any.
    pub fn hop(
        &mut self,
        song: &Song,
        at: (usize, usize),
        (words, showing): (&str, bool),
        dwell: Duration,
        cfg: &SmartConfig,
    ) -> Option<usize> {
        let slides: Vec<Vec<String>> = song.slides.iter().map(|s| tokens(&s.text)).collect();
        if self.at != at || self.evidence.len() != slides.len() {
            *self = Tracker {
                at,
                evidence: vec![0.0; slides.len()],
            };
        }
        let heard = tokens(words);
        let recent = &heard[heard.len().saturating_sub(RECENT)..];
        let here = at.1;
        if here >= slides.len() {
            return None;
        }
        // On screen, the room moving to slide c sang the end of this slide,
        // then c's start; after a blank it may come back anywhere in c.
        let (tail, lead) = match showing {
            true => (
                &slides[here][slides[here].len().saturating_sub(RECENT)..],
                LEAD,
            ),
            false => (&[][..], usize::MAX),
        };
        for (c, slide) in slides.iter().enumerate() {
            let (target, prior) = if c == here {
                (slide.clone(), 1.0)
            } else {
                let lead = &slide[..slide.len().min(lead)];
                ([tail, lead].concat(), prior(song, &slides, here, c, cfg))
            };
            let score = match recent.is_empty() || slide.is_empty() {
                true => 0.0,
                false => lcs_length(recent, &target) as f64 / recent.len() as f64,
            };
            let d = cfg.evidence_decay;
            self.evidence[c] = d * self.evidence[c] + (1.0 - d) * prior * score;
        }
        let e = &self.evidence;
        let best = (0..e.len()).max_by(|&a, &b| e[a].total_cmp(&e[b]))?;
        // Clear: ahead of the current slide and of every other one, so words
        // that two slides share (a repeated line) never decide. A copy of the
        // best slide (a chorus repeated as slides) is the same words: the
        // prior picks between copies.
        let same = |c: usize| c != here && slides[c] == slides[best];
        let ahead =
            (0..e.len()).all(|c| c == best || same(c) || e[best] >= e[c] + cfg.evidence_margin);
        let clear = e[best] >= cfg.min_evidence && ahead;
        (best != here && clear && dwell >= Duration::from_millis(cfg.min_dwell_ms)).then_some(best)
    }
}

/// The prior of moving from slide `from` to slide `to` of `song`.
fn prior(song: &Song, slides: &[Vec<String>], from: usize, to: usize, cfg: &SmartConfig) -> f64 {
    // Empty slides (blank pptx slides) are kept for numbering but never sung.
    if (from + 1..slides.len()).find(|&i| !slides[i].is_empty()) == Some(to) {
        return cfg.prior_next;
    }
    match song.sections.iter().find(|s| s.start == to) {
        Some(s) if ["chorus", "refrain"].contains(&s.kind.to_lowercase().as_str()) => {
            cfg.prior_chorus
        }
        Some(_) => cfg.prior_section,
        // An unlabelled song: every slide may start a section.
        None if song.sections.is_empty() => cfg.prior_section,
        None => cfg.prior_other,
    }
}

/// Lower-case words, punctuation and apostrophes dropped.
fn tokens(text: &str) -> Vec<String> {
    let text = text.to_lowercase().replace(['\'', '\u{2019}'], "");
    (text.split(|c: char| !c.is_alphanumeric()))
        .filter(|w| !w.is_empty())
        .map(String::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn section_labels_choose_the_prior() {
        let json = r#"{"title":"t","slides":[{"id":0,"text":"a b"},{"id":1,"text":"c d"},
            {"id":2,"text":""},{"id":3,"text":"e f"},{"id":4,"text":"g h"},{"id":5,"text":"i j"}],
            "sections":[{"kind":"Verse","start":0},{"kind":"Refrain","start":1},
            {"kind":"bridge","start":3}]}"#;
        let mut song: Song = serde_json::from_str(json).unwrap();
        let cfg = SmartConfig::default();
        let slides: Vec<_> = song.slides.iter().map(|s| tokens(&s.text)).collect();
        let p = |song: &Song, from, to| prior(song, &slides, from, to, &cfg);
        assert_eq!(p(&song, 0, 1), cfg.prior_next);
        assert_eq!(
            p(&song, 1, 3),
            cfg.prior_next,
            "the empty slide 2 is skipped"
        );
        assert_eq!(p(&song, 3, 1), cfg.prior_chorus);
        assert_eq!(p(&song, 3, 0), cfg.prior_section);
        assert_eq!(p(&song, 1, 4), cfg.prior_other, "inside the bridge");
        song.sections.clear();
        assert_eq!(p(&song, 1, 4), cfg.prior_section);
        assert_eq!(
            tokens("Don't  alpha-Bravo, DON\u{2019}T!"),
            ["dont", "alpha", "bravo", "dont"]
        );
    }
}
