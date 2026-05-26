use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slide {
    pub id: usize,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    pub title: String,
    pub slides: Vec<Slide>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setlist {
    pub setlist: Vec<Song>,
}

pub struct AlignConfig {
    pub similarity_threshold: f64,
    pub margin: f64,
    pub max_buffer_words: usize,
}

impl Default for AlignConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 70.0,
            margin: 10.0,
            max_buffer_words: 40,
        }
    }
}

/// Matches transcribed speech against slide lyrics using fuzzy string matching.
/// When the next slide's lyrics match better than the current slide, advances.
pub struct LyricsAligner {
    slides: Vec<Slide>,
    config: AlignConfig,
    current_index: usize,
    buffer_words: Vec<String>,
}

impl LyricsAligner {
    pub fn new(slides: Vec<Slide>, config: AlignConfig) -> Self {
        Self {
            slides,
            config,
            current_index: 0,
            buffer_words: Vec::new(),
        }
    }

    pub fn current_slide(&self) -> Option<&Slide> {
        self.slides.get(self.current_index)
    }

    pub fn next_slide(&self) -> Option<&Slide> {
        self.slides.get(self.current_index + 1)
    }

    pub fn current_index(&self) -> usize {
        self.current_index
    }

    pub fn total_slides(&self) -> usize {
        self.slides.len()
    }

    /// Feed new transcription text. Returns true if slide should advance.
    pub fn update(&mut self, text: &str) -> bool {
        if text.trim().is_empty() {
            return false;
        }

        let new_words: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
        self.buffer_words.extend(new_words);

        if self.buffer_words.len() > self.config.max_buffer_words {
            let start = self.buffer_words.len() - self.config.max_buffer_words;
            self.buffer_words = self.buffer_words[start..].to_vec();
        }

        let buffer_text = self.buffer_words.join(" ").to_lowercase();

        let curr_slide = match self.current_slide() {
            Some(s) => s.clone(),
            None => return false,
        };

        let score_curr = partial_ratio(&buffer_text, &curr_slide.text.to_lowercase());

        let score_next = if let Some(next) = self.next_slide() {
            partial_ratio(&buffer_text, &next.text.to_lowercase())
        } else {
            0.0
        };

        log::info!(
            "[ALIGN] curr={:.1} next={:.1} | ...{}",
            score_curr,
            score_next,
            self.buffer_words
                .iter()
                .rev()
                .take(10)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .cloned()
                .collect::<Vec<_>>()
                .join(" ")
        );

        if self.next_slide().is_some()
            && score_next >= self.config.similarity_threshold
            && score_next > (score_curr + self.config.margin)
        {
            log::info!(
                "[ALIGN] >>> MATCH! Advancing to slide {}",
                self.current_index + 1
            );
            self.current_index += 1;
            self.buffer_words.clear();
            return true;
        }

        false
    }

    pub fn reset(&mut self) {
        self.current_index = 0;
        self.buffer_words.clear();
    }
}

/// Simple partial ratio implementation (similar to rapidfuzz.fuzz.partial_ratio).
/// Slides a window of the shorter string across the longer and returns the best match %.
fn partial_ratio(s1: &str, s2: &str) -> f64 {
    if s1.is_empty() || s2.is_empty() {
        return 0.0;
    }

    let (shorter, longer) = if s1.len() <= s2.len() {
        (s1, s2)
    } else {
        (s2, s1)
    };

    let shorter_chars: Vec<char> = shorter.chars().collect();
    let longer_chars: Vec<char> = longer.chars().collect();

    if shorter_chars.is_empty() || longer_chars.is_empty() {
        return 0.0;
    }

    let mut best_score: f64 = 0.0;
    let window_len = shorter_chars.len();

    if window_len > longer_chars.len() {
        return simple_ratio(&shorter_chars, &longer_chars);
    }

    for i in 0..=(longer_chars.len() - window_len) {
        let window = &longer_chars[i..i + window_len];
        let score = simple_ratio(&shorter_chars, window);
        if score > best_score {
            best_score = score;
        }
        if best_score >= 100.0 {
            return 100.0;
        }
    }

    best_score
}

/// Character-level similarity ratio (0-100).
fn simple_ratio(a: &[char], b: &[char]) -> f64 {
    let matches = lcs_length(a, b);
    let total = a.len() + b.len();
    if total == 0 {
        return 0.0;
    }
    (2.0 * matches as f64 / total as f64) * 100.0
}

/// Longest common subsequence length.
fn lcs_length(a: &[char], b: &[char]) -> usize {
    let m = a.len();
    let n = b.len();
    let mut prev = vec![0usize; n + 1];
    let mut curr = vec![0usize; n + 1];

    for i in 1..=m {
        for j in 1..=n {
            if a[i - 1] == b[j - 1] {
                curr[j] = prev[j - 1] + 1;
            } else {
                curr[j] = curr[j - 1].max(prev[j]);
            }
        }
        std::mem::swap(&mut prev, &mut curr);
        curr.fill(0);
    }

    prev[n]
}
