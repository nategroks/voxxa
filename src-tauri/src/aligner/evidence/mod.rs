//! ADR-009's minimum (Q6): where in the current song the room is.
//!
//! Each recogniser hop scores a few candidate slides on the last
//! [`RECENT_WORDS`] words, word by word, and adds the score to a decaying
//! evidence total per slide. The candidates are the current slide, the next
//! one, the one after the furthest slide reached (after a repeated chorus)
//! and every section start (`sections[].kind` in the setlist; a song without
//! sections is one section). Any other slide is the operator's to choose.

use super::{SmartConfig, Song};
use serde::{Deserialize, Serialize};

/// The scores look at the last 12 recognised words.
pub const RECENT_WORDS: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SectionKind {
    Verse,
    Chorus,
    Bridge,
    Tag,
    Other,
}

/// A labelled run of slides, from slide `start` to the next section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub kind: SectionKind,
    pub start: usize,
}

#[derive(Default)]
pub struct Evidence {
    /// The (song, slide) the totals belong to: a move made anywhere else (a
    /// song start, the operator) starts them again from there.
    at: Option<(usize, usize)>,
    furthest: usize,
    total: Vec<f64>,
}

impl Evidence {
    /// Score one hop at `at` (song, slide). Returns the slide to move to when
    /// one leads the current slide by `advance_margin`, has at least
    /// `min_advance_match`, and the current slide has had its dwell.
    pub fn step(
        &mut self,
        at: (usize, usize),
        song: &Song,
        words: &str,
        cfg: &SmartConfig,
        dwelled: bool,
    ) -> Option<usize> {
        let current = at.1;
        if self.at != Some(at) {
            let total = vec![0.0; song.slides.len()];
            *self = Self {
                at: Some(at),
                furthest: current,
                total,
            };
        }
        let slides: Vec<Vec<String>> = song.slides.iter().map(|s| tokens(&s.text)).collect();
        let recent = tokens(words);
        let recent = last(&recent);
        let tail = last(&slides[current]);
        for (i, prior) in priors(song, current, self.furthest, cfg).into_iter().enumerate() {
            if prior == 0.0 {
                continue;
            }
            // A move shows as the current slide's last words, then the new one's.
            let sim = if i == current {
                partial_ratio(recent, &slides[i])
            } else {
                partial_ratio(recent, &[tail, &slides[i]].concat())
            };
            let keep = cfg.evidence_decay;
            self.total[i] = keep * self.total[i] + (1.0 - keep) * prior * sim;
        }
        let t = &self.total;
        let best = (0..t.len()).max_by(|&a, &b| t[a].total_cmp(&t[b]))?;
        let clear = t[best] >= cfg.min_advance_match && t[best] >= t[current] + cfg.advance_margin;
        if best == current || !clear || !dwelled {
            return None;
        }
        self.at = Some((at.0, best));
        self.furthest = self.furthest.max(best);
        self.total.fill(0.0);
        Some(best)
    }
}

/// Prior of moving from `current` to each slide; 0 means not a candidate.
fn priors(song: &Song, current: usize, furthest: usize, cfg: &SmartConfig) -> Vec<f64> {
    let n = song.slides.len();
    let mut p = vec![0.0; n];
    let one = song.sections.is_empty().then_some((0, SectionKind::Verse));
    let starts = song.sections.iter().map(|s| (s.start, s.kind)).chain(one);
    for (start, kind) in starts {
        if let Some(x) = p.get_mut(start) {
            *x = match kind {
                SectionKind::Chorus => cfg.prior_chorus,
                _ => cfg.prior_section,
            };
        }
    }
    let filled = |i: usize| !song.slides[i].text.trim().is_empty();
    for from in [current, furthest] {
        if let Some(next) = (from + 1..n).find(|&i| filled(i)) {
            p[next] = p[next].max(cfg.prior_next);
        }
    }
    for (i, x) in p.iter_mut().enumerate() {
        if !filled(i) {
            *x = 0.0;
        }
    }
    p[current] = 1.0;
    p
}

/// Lower-case words without punctuation ("'Twas" and "twas" agree).
fn tokens(text: &str) -> Vec<String> {
    (text.split(|c: char| !c.is_alphanumeric()))
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect()
}

fn last(words: &[String]) -> &[String] {
    &words[words.len().saturating_sub(RECENT_WORDS)..]
}

/// [`partial_ratio`] over characters: song detection and the blank bars.
pub fn char_ratio(a: &str, b: &str) -> f64 {
    let chars = |s: &str| s.chars().collect::<Vec<_>>();
    partial_ratio(&chars(a), &chars(b))
}

/// Sliding-window similarity, 0-100 (rapidfuzz-style partial ratio): the
/// shorter sequence against every window of the longer one.
fn partial_ratio<T: PartialEq>(a: &[T], b: &[T]) -> f64 {
    let (short, long) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    if short.is_empty() {
        return 0.0;
    }
    let mut best: f64 = 0.0;
    for window in long.windows(short.len()) {
        best = best.max(simple_ratio(short, window));
        if best >= 100.0 {
            break;
        }
    }
    best
}

fn simple_ratio<T: PartialEq>(a: &[T], b: &[T]) -> f64 {
    let total = a.len() + b.len();
    if total == 0 {
        return 0.0;
    }
    (2.0 * lcs_length(a, b) as f64 / total as f64) * 100.0
}

fn lcs_length<T: PartialEq>(a: &[T], b: &[T]) -> usize {
    let n = b.len();
    let mut prev = vec![0usize; n + 1];
    let mut curr = vec![0usize; n + 1];
    for x in a {
        for j in 1..=n {
            curr[j] = if *x == b[j - 1] {
                prev[j - 1] + 1
            } else {
                curr[j - 1].max(prev[j])
            };
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[n]
}
