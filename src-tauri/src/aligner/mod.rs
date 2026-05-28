//! Smart-blanking state machine — the differentiator described in §4 of the plan.
//!
//! Drives slide control via three inputs:
//!   - VAD speech / silence events (per audio frame)
//!   - Whisper transcript chunks (every ~5 s when speech is present)
//!   - Periodic tick (every ~100 ms) for time-based transitions
//!
//! Emits `Action` values that the audio loop dispatches through the active
//! [`crate::presenters::PresentationController`]: `Goto` (move to a specific
//! global slide), `Blank`, `Unblank`, or `Noop`.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

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

/// States from §4.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MachineState {
    /// No song detected; audience output blanked, scanning every setlist song.
    Listening,
    /// Inside a song, output is showing the active slide.
    Singing,
    /// Mid-song silence between verses — slide held, output still showing.
    InterVerseSilence,
    /// Mid-song speech that doesn't match any setlist song — output blanked.
    BlankHold,
}

/// Tunable parameters. Defaults come from §4.4.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartConfig {
    pub silence_to_blank_secs: f64,
    pub unrecognized_speech_to_blank_secs: f64,
    pub song_confidence_floor: f64,
    pub song_second_max: f64,
    pub min_song_dwell_secs: f64,
    pub slide_advance_debounce_ms: u64,
    pub min_advance_match: f64,
    pub advance_margin: f64,
    pub max_buffer_words: usize,
    pub softmax_temperature: f64,
}

impl Default for SmartConfig {
    fn default() -> Self {
        Self {
            silence_to_blank_secs: 3.0,
            unrecognized_speech_to_blank_secs: 5.0,
            song_confidence_floor: 0.60,
            song_second_max: 0.40,
            min_song_dwell_secs: 8.0,
            slide_advance_debounce_ms: 1500,
            min_advance_match: 70.0,
            advance_margin: 10.0,
            max_buffer_words: 40,
            softmax_temperature: 12.0,
        }
    }
}

/// Imperative action the state machine asks the audio loop to perform on the
/// active presenter driver.
#[derive(Debug, Clone)]
pub enum Action {
    Noop,
    /// Move audience output to a specific slide. `global_index` is the index in
    /// the flat setlist; drivers that lack `goto_slide` get translated to a run
    /// of next/prev keys by the dispatcher.
    Goto {
        song_index: usize,
        slide_in_song: usize,
        global_index: usize,
        slide_text: String,
        song_title: String,
    },
    Blank,
    Unblank,
}

#[derive(Debug, Clone, Serialize)]
pub struct SongScore {
    pub song_index: usize,
    pub raw_score: f64,
    pub probability: f64,
}

pub struct Conductor {
    songs: Vec<Song>,
    /// `song_offsets[i]` = global index of the first slide of song i.
    song_offsets: Vec<usize>,
    total_slides: usize,
    config: SmartConfig,

    state: MachineState,
    is_blank: bool,
    state_since: Instant,

    current_song: Option<usize>,
    current_slide_in_song: usize,
    song_started_at: Option<Instant>,

    buffer_words: Vec<String>,

    last_speech: Option<Instant>,
    last_advance: Option<Instant>,

    /// Most recent per-song softmax scores. Refreshed on every on_transcript
    /// call so the UI can render confidence bars in real time. Empty before
    /// the first transcript arrives.
    last_scores: Vec<SongScore>,
}

impl Conductor {
    pub fn new(songs: Vec<Song>, config: SmartConfig) -> Self {
        let mut song_offsets = Vec::with_capacity(songs.len());
        let mut total = 0usize;
        for s in &songs {
            song_offsets.push(total);
            total += s.slides.len();
        }
        let now = Instant::now();
        Self {
            songs,
            song_offsets,
            total_slides: total,
            config,
            state: MachineState::Listening,
            is_blank: true,
            state_since: now,
            current_song: None,
            current_slide_in_song: 0,
            song_started_at: None,
            buffer_words: Vec::new(),
            last_speech: None,
            last_advance: None,
            last_scores: Vec::new(),
        }
    }

    /// Most recent per-song confidences. Cheap clone — at most one entry per
    /// song in the setlist.
    pub fn last_scores(&self) -> &[SongScore] {
        &self.last_scores
    }

    /// Map song index → title for the UI's confidence display.
    pub fn song_titles(&self) -> Vec<String> {
        self.songs.iter().map(|s| s.title.clone()).collect()
    }

    pub fn state(&self) -> MachineState {
        self.state
    }
    pub fn is_blank(&self) -> bool {
        self.is_blank
    }
    pub fn total_slides(&self) -> usize {
        self.total_slides
    }
    pub fn current_index(&self) -> usize {
        match self.current_song {
            Some(s) => self.song_offsets[s] + self.current_slide_in_song,
            None => 0,
        }
    }
    pub fn current_song_title(&self) -> Option<&str> {
        self.current_song
            .and_then(|i| self.songs.get(i).map(|s| s.title.as_str()))
    }

    /// Hot-swap tuning thresholds without losing position. Useful when the
    /// operator is dialing in smart-blanking mid-rehearsal.
    pub fn set_config(&mut self, cfg: SmartConfig) {
        log::info!("[CONDUCTOR] config updated");
        self.config = cfg;
    }

    /// Operator-initiated jump to a specific song. Resets to the song's
    /// first slide, clears the rolling transcript buffer (so a previous
    /// song's lyrics don't pollute matching), and arms the dwell clock so
    /// the conductor won't immediately re-detect a different song.
    /// Returns the Goto action the audio loop should dispatch.
    pub fn jump_to_song(&mut self, song_idx: usize, now: Instant) -> Action {
        if song_idx >= self.songs.len() {
            return Action::Noop;
        }
        self.start_song(song_idx, now)
    }

    /// Notify the machine that VAD says the user is speaking.
    pub fn on_speech(&mut self, now: Instant) {
        self.last_speech = Some(now);
    }

    /// Time-based transitions. Call regularly (~100 ms is fine).
    pub fn tick(&mut self, now: Instant) -> Action {
        let since_speech = match self.last_speech {
            Some(t) => now.saturating_duration_since(t),
            None => Duration::from_secs(86_400),
        };

        match self.state {
            MachineState::Singing => {
                if since_speech
                    >= Duration::from_secs_f64(self.config.silence_to_blank_secs)
                {
                    self.transition(MachineState::InterVerseSilence, now);
                }
            }
            MachineState::InterVerseSilence => {
                if since_speech
                    >= Duration::from_secs_f64(self.config.unrecognized_speech_to_blank_secs)
                    && !self.is_blank
                {
                    self.transition(MachineState::BlankHold, now);
                    self.is_blank = true;
                    return Action::Blank;
                }
            }
            MachineState::Listening | MachineState::BlankHold => {
                if !self.is_blank {
                    self.is_blank = true;
                    return Action::Blank;
                }
            }
        }
        Action::Noop
    }

    /// Process a fresh Whisper transcript.
    pub fn on_transcript(&mut self, text: &str, now: Instant) -> Action {
        if text.trim().is_empty() {
            return Action::Noop;
        }
        self.on_speech(now);

        for w in text.split_whitespace() {
            self.buffer_words.push(w.to_lowercase());
        }
        if self.buffer_words.len() > self.config.max_buffer_words {
            let drop = self.buffer_words.len() - self.config.max_buffer_words;
            self.buffer_words.drain(..drop);
        }
        let buffer_text = self.buffer_words.join(" ");

        let scores = self.score_all_songs(&buffer_text);
        self.last_scores = scores.clone();
        if scores.is_empty() {
            return Action::Noop;
        }

        let top = &scores[0];
        let second_p = scores.get(1).map(|s| s.probability).unwrap_or(0.0);
        let confident = top.probability >= self.config.song_confidence_floor
            && second_p <= self.config.song_second_max;

        log::info!(
            "[CONDUCTOR] top={} p={:.2} second_p={:.2} confident={} state={:?}",
            top.song_index,
            top.probability,
            second_p,
            confident,
            self.state
        );

        // No song committed yet — wait for high confidence before showing anything.
        let Some(current) = self.current_song else {
            if confident {
                return self.start_song(top.song_index, now);
            }
            return Action::Noop;
        };

        // Already in a song. Check whether the service moved on to a different song
        // (top winner differs AND min dwell satisfied).
        let dwelled = self
            .song_started_at
            .map(|t| {
                now.saturating_duration_since(t)
                    >= Duration::from_secs_f64(self.config.min_song_dwell_secs)
            })
            .unwrap_or(false);
        if confident && top.song_index != current && dwelled {
            return self.start_song(top.song_index, now);
        }

        // Try to advance within the current song.
        let advance = self.maybe_advance_within_current(&buffer_text, now);
        if !matches!(advance, Action::Noop) {
            return advance;
        }

        // We matched the current song but didn't advance — ensure output is unblanked
        // and showing the current slide (recovers from BlankHold/InterVerseSilence).
        if self.is_blank || self.state != MachineState::Singing {
            self.is_blank = false;
            self.transition(MachineState::Singing, now);
            if let Some(g) = self.goto_action_for_current() {
                return g;
            }
            return Action::Unblank;
        }

        Action::Noop
    }

    fn score_all_songs(&self, buffer_text: &str) -> Vec<SongScore> {
        let mut raw: Vec<(usize, f64)> = self
            .songs
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let combined = s
                    .slides
                    .iter()
                    .map(|sl| sl.text.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
                    .to_lowercase();
                (i, partial_ratio(buffer_text, &combined))
            })
            .collect();
        raw.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let t = self.config.softmax_temperature.max(0.001);
        let max = raw.first().map(|x| x.1).unwrap_or(0.0);
        let exps: Vec<f64> = raw.iter().map(|x| ((x.1 - max) / t).exp()).collect();
        let sum: f64 = exps.iter().sum();
        let sum = if sum > 0.0 { sum } else { 1.0 };

        raw.into_iter()
            .zip(exps)
            .map(|((i, s), e)| SongScore {
                song_index: i,
                raw_score: s,
                probability: e / sum,
            })
            .collect()
    }

    fn maybe_advance_within_current(&mut self, buffer_text: &str, now: Instant) -> Action {
        let Some(current) = self.current_song else {
            return Action::Noop;
        };
        if let Some(last) = self.last_advance {
            if now.saturating_duration_since(last)
                < Duration::from_millis(self.config.slide_advance_debounce_ms)
            {
                return Action::Noop;
            }
        }

        let song = &self.songs[current];
        let curr_idx = self.current_slide_in_song;
        let curr = song.slides.get(curr_idx);
        let next = song.slides.get(curr_idx + 1);

        let score_curr = curr
            .map(|s| partial_ratio(buffer_text, &s.text.to_lowercase()))
            .unwrap_or(0.0);
        let score_next = next
            .map(|s| partial_ratio(buffer_text, &s.text.to_lowercase()))
            .unwrap_or(0.0);

        if next.is_some()
            && score_next >= self.config.min_advance_match
            && score_next > (score_curr + self.config.advance_margin)
        {
            self.current_slide_in_song = curr_idx + 1;
            self.buffer_words.clear();
            self.last_advance = Some(now);
            self.is_blank = false;
            self.transition(MachineState::Singing, now);
            log::info!(
                "[CONDUCTOR] advance within song {current} to slide {}",
                self.current_slide_in_song
            );
            return self.goto_action_for_current().unwrap_or(Action::Noop);
        }
        Action::Noop
    }

    fn start_song(&mut self, song_idx: usize, now: Instant) -> Action {
        log::info!(
            "[CONDUCTOR] start song {} ({})",
            song_idx,
            self.songs[song_idx].title
        );
        self.current_song = Some(song_idx);
        self.current_slide_in_song = 0;
        self.song_started_at = Some(now);
        self.buffer_words.clear();
        self.last_advance = Some(now);
        self.is_blank = false;
        self.transition(MachineState::Singing, now);
        self.goto_action_for_current().unwrap_or(Action::Noop)
    }

    fn goto_action_for_current(&self) -> Option<Action> {
        let song_index = self.current_song?;
        let song = self.songs.get(song_index)?;
        let slide = song.slides.get(self.current_slide_in_song)?;
        let global = self.song_offsets[song_index] + self.current_slide_in_song;
        Some(Action::Goto {
            song_index,
            slide_in_song: self.current_slide_in_song,
            global_index: global,
            slide_text: slide.text.clone(),
            song_title: song.title.clone(),
        })
    }

    fn transition(&mut self, to: MachineState, now: Instant) {
        if to != self.state {
            log::info!("[CONDUCTOR] {:?} → {:?}", self.state, to);
            self.state = to;
            self.state_since = now;
        }
    }
}

/// Sliding-window character similarity (rapidfuzz-style partial ratio).
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

fn simple_ratio(a: &[char], b: &[char]) -> f64 {
    let matches = lcs_length(a, b);
    let total = a.len() + b.len();
    if total == 0 {
        return 0.0;
    }
    (2.0 * matches as f64 / total as f64) * 100.0
}

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
