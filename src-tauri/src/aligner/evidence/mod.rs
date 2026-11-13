//! ADR-009's minimum (ADR-016): follow the position inside the current song
//! with repeat-aware priors, word-token scores on the last 12 recognised
//! words, and an evidence accumulator with decay. The conductor asks
//! [`Tracker::decide`] once per accepted transcript; everything else
//! (song detection, blanking, `manual_step`) stays in the conductor.

mod sections;

pub use sections::{check_sections, Section, SectionKind};

use super::{lcs_length, Song};
use serde::{Deserialize, Serialize};

/// Recognised words the scores look at (ADR-016 Decision 3).
pub const WINDOW_WORDS: usize = 12;

/// Priors, accumulator and dwell (ADR-016 Decisions 2, 4 and 5). Stored in
/// `SmartConfig.evidence`, so every session log records the values it ran
/// with; a log or a GUI setting without them gets these defaults.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EvidenceConfig {
    /// Staying on the current slide.
    pub prior_stay: f64,
    /// The next non-empty slide; after a chorus, the slide after the verse
    /// that led into it.
    pub prior_next: f64,
    /// The first slide of any chorus section.
    pub prior_chorus: f64,
    /// The first slide of any section (every slide of an unlabelled song).
    pub prior_section: f64,
    /// Any other slide of the song.
    pub prior_other: f64,
    /// Share of the old evidence kept per transcript (0..1).
    pub decay: f64,
    /// Evidence the best slide needs to move there.
    pub threshold: f64,
    /// How far the best slide must lead the current one.
    pub margin: f64,
    /// Least time between two moves, counted from any move (the
    /// conductor's, a song start or the operator's).
    pub min_dwell_ms: u64,
}

impl Default for EvidenceConfig {
    fn default() -> Self {
        Self {
            prior_stay: 1.0,
            prior_next: 0.9,
            prior_chorus: 0.8,
            prior_section: 0.6,
            prior_other: 0.4,
            decay: 0.3,
            threshold: 0.35,
            margin: 0.05,
            min_dwell_ms: 1500,
        }
    }
}

/// Lower-case word tokens: letters and digits. Apostrophes are dropped
/// ("'Twas" is "twas"); any other mark splits words ("half-shed").
pub fn tokens(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric() && c != '\'' && c != '\u{2019}')
        .map(|w| (w.chars().filter(|c| c.is_alphanumeric())).flat_map(char::to_lowercase))
        .map(String::from_iter)
        .filter(|w| !w.is_empty())
        .collect()
}

/// Slide words past the newest words' count that still count as the
/// slide's leading words.
const LEAD_SLACK: usize = 4;

/// Share of the newest `fresh` words of `window` that extend an in-order
/// alignment (LCS) of the window with `slide`; for a slide the room would be
/// `entering`, the share that aligns with its leading words, if more.
/// Words already in the window give context but no score; a dropped word
/// costs only its own share.
pub fn score(window: &[String], fresh: usize, slide: &[String], entering: bool) -> f64 {
    let fresh = fresh.min(window.len());
    if fresh == 0 {
        return 0.0;
    }
    let (context, new) = window.split_at(window.len() - fresh);
    let gain = lcs_length(window, slide) - lcs_length(context, slide);
    let lead = match entering {
        true => lcs_length(new, &slide[..slide.len().min(fresh + LEAD_SLACK)]),
        false => 0,
    };
    gain.max(lead) as f64 / fresh as f64
}

/// Position evidence for the slides of the current song.
#[derive(Debug, Default)]
pub struct Tracker {
    /// The last [`WINDOW_WORDS`] recognised words.
    window: Vec<String>,
    /// How many of them the latest transcript added.
    fresh: usize,
    /// The (song, slide) the evidence belongs to.
    at: Option<(usize, usize)>,
    /// One value per slide of that song.
    evidence: Vec<f64>,
    /// After a chorus: the slide after the verse that led into it.
    resume: Option<usize>,
    /// The slide an operator just left: never moved back to until the room
    /// sings the operator's slide or the tracker moves on its own (the
    /// operator always wins).
    left: Option<usize>,
}

impl Tracker {
    /// Add the words of an accepted transcript to the window.
    pub fn hear(&mut self, text: &str) {
        let new = tokens(text);
        self.fresh = new.len().min(WINDOW_WORDS);
        self.window.extend(new);
        let excess = self.window.len().saturating_sub(WINDOW_WORDS);
        self.window.drain(..excess);
    }

    /// Score the newest words against every slide of `song`, update the
    /// evidence and return the slide to move to, if the evidence says so
    /// and `may_move` (the dwell has passed). `current` is where the
    /// conductor is; if someone else moved it, the evidence starts again.
    pub fn decide(
        &mut self,
        song_index: usize,
        song: &Song,
        current: usize,
        cfg: &EvidenceConfig,
        may_move: bool,
    ) -> Option<usize> {
        if self.at != Some((song_index, current)) || self.evidence.len() != song.slides.len() {
            self.moved_by_other(song_index, song, current);
        }
        let fresh = std::mem::take(&mut self.fresh);
        let next = self.next(song, current);
        let mut here = 0.0;
        for (i, slide) in song.slides.iter().enumerate() {
            let words = tokens(&slide.text);
            let prior = if words.is_empty() || Some(i) == self.left {
                0.0
            } else if i == current {
                cfg.prior_stay
            } else if Some(i) == next {
                cfg.prior_next
            } else if !sections::is_start(song, i) {
                cfg.prior_other
            } else if sections::is_chorus(song, i) {
                cfg.prior_chorus
            } else {
                cfg.prior_section
            };
            let s = score(&self.window, fresh, &words, i != current);
            here = if i == current { s } else { here };
            self.evidence[i] = cfg.decay * self.evidence[i] + (1.0 - cfg.decay) * prior * s;
        }
        // Once the room sings the operator's slide, the slide they left is
        // an ordinary candidate again.
        if here >= 0.5 {
            self.left = None;
        }
        // Ties keep the current slide, then go to the nearest one after it.
        let n = song.slides.len();
        let best = ((current + 1..n).chain(0..current)).fold(current, |b, i| {
            if self.evidence[i] > self.evidence[b] {
                i
            } else {
                b
            }
        });
        let (e_best, e_now) = (self.evidence[best], self.evidence[current]);
        if best == current || !may_move || e_best < cfg.threshold || e_best < e_now + cfg.margin {
            return None;
        }
        log::info!("[EVIDENCE] slide {current} -> {best}: {e_now:.2} -> {e_best:.2}");
        self.entered(song, current, best);
        self.at = Some((song_index, best));
        self.left = None;
        Some(best)
    }

    /// Someone else set the position (a song start, `manual_step`,
    /// `jump_to_song`): forget the evidence and trust the new slide.
    fn moved_by_other(&mut self, song_index: usize, song: &Song, current: usize) {
        self.left = None;
        self.resume = None;
        if let Some((_, from)) = self.at.filter(|&(s, _)| s == song_index) {
            self.entered(song, from, current);
            self.left = Some(from);
        }
        self.evidence = vec![0.0; song.slides.len()];
        if let Some(e) = self.evidence.get_mut(current) {
            *e = 1.0;
        }
        self.at = Some((song_index, current));
    }

    /// Remember where to go after a chorus entered from a verse.
    fn entered(&mut self, song: &Song, from: usize, to: usize) {
        if !sections::is_chorus(song, to) {
            self.resume = None;
        } else if !sections::is_chorus(song, from) {
            self.resume = (from + 1..song.slides.len())
                .find(|&i| !sections::is_chorus(song, i) && !song.slides[i].text.trim().is_empty());
        }
    }

    /// The likely next slide: the next non-empty one, except at the end of
    /// a chorus that a verse led into, where it is the slide after that verse.
    fn next(&self, song: &Song, current: usize) -> Option<usize> {
        let plain =
            (current + 1..song.slides.len()).find(|&i| !song.slides[i].text.trim().is_empty());
        let chorus_ends = sections::is_chorus(song, current)
            && plain.is_none_or(|p| !sections::same_section(song, current, p));
        match self.resume {
            Some(r) if chorus_ends => Some(r),
            _ => plain,
        }
    }
}
