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
    /// Absolute minimum partial-ratio score (0–100) the top candidate must
    /// reach before we'll commit to a song. Guards against the case where
    /// softmax-relative confidence is high but every candidate is a poor
    /// absolute match (random speech, instrumental intro, etc.). Plan §4.6:
    /// "bad blank is better than wrong slide."
    pub min_song_match: f64,
    pub min_song_dwell_secs: f64,
    pub slide_advance_debounce_ms: u64,
    pub min_advance_match: f64,
    /// partial_ratio the new transcript needs to start a song or re-show
    /// lyrics. Stricter than an advance: short worship phrases score 70-81.
    pub min_start_match: f64,
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
            min_song_match: 40.0,
            min_song_dwell_secs: 8.0,
            slide_advance_debounce_ms: 1500,
            min_advance_match: 70.0,
            min_start_match: 85.0,
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
        /// Next slide's text (or `None` at the end of the song). Threaded
        /// through to the slide-advanced UI event so the Stage Display can
        /// show the Next panel without a follow-up backend query.
        next_slide_text: Option<String>,
        song_title: String,
    },
    Blank,
    Unblank,
}

/// One input to the conductor. The listening loop and `voxxad --replay` both
/// apply inputs through [`Conductor::apply`], so a session log that records
/// every input, with its time, reproduces every action (ADR-013).
#[derive(Debug, Clone, PartialEq)]
pub enum Input {
    /// The periodic tick that drives time-based transitions.
    Tick,
    /// VAD says someone is speaking.
    Speech,
    /// A recogniser transcript.
    Heard(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct SongScore {
    pub song_index: usize,
    pub raw_score: f64,
    pub probability: f64,
}

/// Most songs a setlist may hold. Every transcript is scored against every
/// song (about 70 ms per song in release), so a whole EasyWorship library
/// would delay slide control by minutes. 50 songs take about 3.5 s, longer
/// than the 1-2 s between transcripts with the hop (ADR-014): the bound then
/// rests on the loop's lag gate, which counts this time and skips hops
/// instead of queueing audio.
pub const MAX_SETLIST_SONGS: usize = 50;

/// Reject setlists the conductor cannot follow, before loading them. An
/// empty setlist leaves the conductor in Listening forever; a song with zero
/// slides commits internally but can't fire a Goto.
pub fn check_setlist(songs: &[Song]) -> Result<(), String> {
    if songs.is_empty() {
        return Err("Setlist is empty — no songs to load.".into());
    }
    if songs.len() > MAX_SETLIST_SONGS {
        return Err(format!(
            "This library has {} songs; pick today's songs (at most {MAX_SETLIST_SONGS}).",
            songs.len()
        ));
    }
    match songs.iter().find(|s| s.slides.is_empty()) {
        Some(bad) => Err(format!(
            "Song \"{}\" has no slides — every song needs at least one.",
            bad.title
        )),
        None => Ok(()),
    }
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
    /// When the first transcript that does not match the current song came
    /// in (None after a match). Unmatched transcripts for
    /// `unrecognized_speech_to_blank_secs` blank.
    unmatched_since: Option<Instant>,
    /// More than one unmatched transcript came since `unmatched_since`.
    unmatched_again: bool,

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
            unmatched_since: None,
            unmatched_again: false,
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
    /// Index of the song the conductor is currently committed to, or `None`
    /// if it hasn't detected one yet (state == Listening at startup).
    pub fn current_song(&self) -> Option<usize> {
        self.current_song
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
        self.start_song(song_idx, 0, now)
    }

    /// Replace one slide's text (the operator's inline edit) without moving:
    /// the current song, slide and blank state are kept. `false` when
    /// `global` is out of range.
    pub fn update_slide_text(&mut self, global: usize, text: String) -> bool {
        let Some(song) = self.song_offsets.iter().rposition(|&o| o <= global) else {
            return false;
        };
        match self.songs[song]
            .slides
            .get_mut(global - self.song_offsets[song])
        {
            Some(slide) => {
                slide.text = text;
                true
            }
            None => false,
        }
    }

    /// Apply one [`Input`] at `now`: the only way the loop and replay drive
    /// the conductor.
    pub fn apply(&mut self, input: &Input, now: Instant) -> Action {
        match input {
            Input::Tick => self.tick(now),
            Input::Speech => {
                self.on_speech(now);
                Action::Noop
            }
            Input::Heard(text) => self.on_transcript(text, now),
        }
    }

    /// Notify the machine that VAD says the user is speaking.
    ///
    /// Only records the time. `voxxad` relies on this: its session log keeps
    /// the last `speech` before each tick or transcript and drops the rest
    /// (ADR-013). If this ever does more, log every speech input.
    pub fn on_speech(&mut self, now: Instant) {
        self.last_speech = Some(now);
    }

    /// Time-based transitions. Call regularly (~100 ms is fine).
    pub fn tick(&mut self, now: Instant) -> Action {
        let since_speech = match self.last_speech {
            Some(t) => now.saturating_duration_since(t),
            None => Duration::from_secs(86_400),
        };
        // Unmatched speech goes on after a second unmatched transcript: a
        // short prayer may give no transcript 5 s after its first one.
        let limit = Duration::from_secs_f64(self.config.unrecognized_speech_to_blank_secs);
        let unmatched = self.unmatched_since.map_or(Duration::ZERO, |t| now - t);
        if !self.is_blank
            && self.unmatched_again
            && unmatched >= limit
            && since_speech < Duration::from_secs_f64(self.config.silence_to_blank_secs)
        {
            return self.blank_hold(now);
        }

        match self.state {
            MachineState::Singing => {
                if since_speech >= Duration::from_secs_f64(self.config.silence_to_blank_secs) {
                    self.transition(MachineState::InterVerseSilence, now);
                }
            }
            MachineState::InterVerseSilence => {
                if since_speech >= limit && !self.is_blank {
                    return self.blank_hold(now);
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
        // Whisper turns music and applause into 1-3 words ("Amen."), and a
        // short string scores ~100 against any song.
        if text.split_whitespace().count() < 4 {
            return Action::Noop;
        }

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
        // Two-gate confidence: relative (softmax distribution) AND absolute
        // (raw partial-ratio score). The absolute gate stops a near-tie of
        // poor matches from masquerading as a confident detection.
        let confident = top.probability >= self.config.song_confidence_floor
            && second_p <= self.config.song_second_max
            && top.raw_score >= self.config.min_song_match;
        // With one song p is always 1.0: the new words must read as the song.
        let confident =
            confident && self.matches_song(top.song_index, text, self.config.min_start_match);

        log::info!(
            "[CONDUCTOR] top={} p={:.2} raw={:.1} second_p={:.2} confident={} state={:?}",
            top.song_index,
            top.probability,
            top.raw_score,
            second_p,
            confident,
            self.state
        );

        // No song committed yet — wait for high confidence before showing anything.
        let Some(current) = self.current_song else {
            if confident {
                let slide = self.best_slide(top.song_index, &buffer_text);
                return self.start_song(top.song_index, slide, now);
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
            let slide = self.best_slide(top.song_index, &buffer_text);
            return self.start_song(top.song_index, slide, now);
        }

        let showing = matches!(
            self.state,
            MachineState::Singing | MachineState::InterVerseSilence
        ) && !self.is_blank;

        // Try to advance within the current song. While the lyrics are off,
        // an advance re-shows them, so it needs the start bar too, and only
        // the latest words count: speech heard during the blank is not lyrics.
        let words = if showing {
            buffer_text
        } else {
            text.to_lowercase()
        };
        if showing || self.matches_song(current, text, self.config.min_start_match) {
            let advance = self.maybe_advance_within_current(&words, now);
            if !matches!(advance, Action::Noop) {
                return advance;
            }
        }

        // Words that do not match the current song never re-show its lyrics.
        // A few seconds of them may be a shout-out or a misheard line; when
        // nothing has matched for `unrecognized_speech_to_blank_secs` the
        // service moved on (prayer, sermon), so blank, even though VAD keeps
        // hearing speech: here, or on a tick after a second unmatched one.
        // Time, not a count: transcripts come every 1-3 s.
        // Keeping lyrics up needs the advance bar; re-showing them, the start bar.
        let bar = if showing {
            self.config.min_advance_match
        } else {
            self.config.min_start_match
        };
        if !self.matches_song(current, text, bar) {
            self.unmatched_again = self.unmatched_since.is_some();
            let since = *self.unmatched_since.get_or_insert(now);
            let limit = Duration::from_secs_f64(self.config.unrecognized_speech_to_blank_secs);
            if showing && now.saturating_duration_since(since) >= limit {
                return self.blank_hold(now);
            }
            return Action::Noop;
        }
        self.unmatched_since = None;

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

    /// The transcript reads as a line of song `i` (partial_ratio >= `bar`);
    /// ordinary speech already scores 40-50 against any whole song.
    fn matches_song(&self, i: usize, text: &str, bar: f64) -> bool {
        let Some(song) = self.songs.get(i) else {
            return false;
        };
        let combined = song
            .slides
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
        partial_ratio(&text.to_lowercase(), &combined) >= bar
    }

    /// Blank the output and hold. The rolling buffer is cleared so the words
    /// heard before the blank cannot bring the lyrics back afterwards.
    fn blank_hold(&mut self, now: Instant) -> Action {
        self.transition(MachineState::BlankHold, now);
        self.is_blank = true;
        self.buffer_words.clear();
        self.unmatched_since = None;
        Action::Blank
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
        // Empty slides (blank pptx slides, empty PDF pages) are kept so slide
        // numbers match the presentation app, but are never matched.
        let Some(next_idx) =
            (curr_idx + 1..song.slides.len()).find(|&i| !song.slides[i].text.trim().is_empty())
        else {
            return Action::Noop;
        };
        let next = song.slides.get(next_idx);

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
            self.current_slide_in_song = next_idx;
            self.buffer_words.clear();
            self.unmatched_since = None;
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

    /// The slide of `song_idx` that best matches the recent words, so a song
    /// detected mid-way (after a restart or a dropout) starts where the room
    /// is. Empty slides are never chosen; ties go to the earlier slide.
    fn best_slide(&self, song_idx: usize, buffer_text: &str) -> usize {
        let mut best = (0, f64::MIN);
        for (i, slide) in self.songs[song_idx].slides.iter().enumerate() {
            if slide.text.trim().is_empty() {
                continue;
            }
            let score = partial_ratio(buffer_text, &slide.text.to_lowercase());
            if score > best.1 {
                best = (i, score);
            }
        }
        best.0
    }

    fn start_song(&mut self, song_idx: usize, slide: usize, now: Instant) -> Action {
        log::info!(
            "[CONDUCTOR] start song {} ({}) at slide {slide}",
            song_idx,
            self.songs[song_idx].title
        );
        self.current_song = Some(song_idx);
        self.current_slide_in_song = slide;
        self.song_started_at = Some(now);
        self.buffer_words.clear();
        self.unmatched_since = None;
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
        let next_slide_text = song
            .slides
            .get(self.current_slide_in_song + 1)
            .map(|s| s.text.clone());
        Some(Action::Goto {
            song_index,
            slide_in_song: self.current_slide_in_song,
            global_index: global,
            slide_text: slide.text.clone(),
            next_slide_text,
            song_title: song.title.clone(),
        })
    }

    /// Notify the conductor that audience output was blanked outside its
    /// control (operator hit the tray Blank, the manual Blank button, or the
    /// HTTP API /blank route). Without this the conductor's `is_blank` stays
    /// false, the on_transcript "currently blanked → unblank" branch never
    /// fires, and the audience output stays blank through the next verse
    /// until something else triggers a Goto.
    pub fn notify_external_blank(&mut self) {
        self.is_blank = true;
    }

    /// Operator-initiated manual step within the current song. Bounded by the
    /// song's slide count. Returns the Goto action for the new position, or
    /// `Noop` if we're not in a song (so the caller can fall back to a raw
    /// presenter keystroke) or if the step would land out of bounds and we're
    /// already at the boundary.
    pub fn manual_step(&mut self, delta: i32, now: Instant) -> Action {
        let Some(song_idx) = self.current_song else {
            return Action::Noop;
        };
        let song_len = self
            .songs
            .get(song_idx)
            .map(|s| s.slides.len() as i32)
            .unwrap_or(0);
        if song_len == 0 {
            return Action::Noop;
        }
        let target = (self.current_slide_in_song as i32 + delta).clamp(0, song_len - 1);
        if (target as usize) == self.current_slide_in_song {
            return Action::Noop;
        }
        self.current_slide_in_song = target as usize;
        self.last_advance = Some(now);
        self.unmatched_since = None;
        self.is_blank = false;
        self.transition(MachineState::Singing, now);
        // Operator overrode the conductor — drop the rolling buffer so a
        // stale half-verse doesn't trigger a competing auto-advance.
        self.buffer_words.clear();
        self.goto_action_for_current().unwrap_or(Action::Noop)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_setlist() -> Vec<Song> {
        vec![
            Song {
                title: "Amazing Grace".into(),
                slides: vec![
                    Slide {
                        id: 0,
                        text: "Amazing grace how sweet the sound that saved a wretch like me"
                            .into(),
                    },
                    Slide {
                        id: 1,
                        text: "I once was lost but now am found was blind but now I see".into(),
                    },
                    Slide {
                        id: 2,
                        text: "Twas grace that taught my heart to fear and grace my fears relieved"
                            .into(),
                    },
                ],
            },
            Song {
                title: "Holy, Holy, Holy".into(),
                slides: vec![
                    Slide {
                        id: 3,
                        text: "Holy holy holy Lord God Almighty".into(),
                    },
                    Slide {
                        id: 4,
                        text: "Early in the morning our song shall rise to Thee".into(),
                    },
                ],
            },
            Song {
                title: "Come, Thou Fount".into(),
                slides: vec![
                    Slide {
                        id: 5,
                        text: "Come Thou fount of every blessing tune my heart to sing Thy grace"
                            .into(),
                    },
                    Slide {
                        id: 6,
                        text: "Streams of mercy never ceasing call for songs of loudest praise"
                            .into(),
                    },
                ],
            },
        ]
    }

    fn cfg_with_dwell(min_dwell_secs: f64) -> SmartConfig {
        SmartConfig {
            min_song_dwell_secs: min_dwell_secs,
            ..Default::default()
        }
    }

    #[test]
    fn starts_in_listening_and_blank() {
        let c = Conductor::new(mk_setlist(), SmartConfig::default());
        assert_eq!(c.state(), MachineState::Listening);
        assert!(c.is_blank());
        assert_eq!(c.current_index(), 0);
        assert!(c.current_song_title().is_none());
    }

    #[test]
    fn confident_match_commits_to_song() {
        let mut c = Conductor::new(mk_setlist(), SmartConfig::default());
        let now = Instant::now();
        // A long lyric line from song 0 should be a clear winner.
        let action = c.on_transcript(
            "amazing grace how sweet the sound that saved a wretch like me",
            now,
        );
        match action {
            Action::Goto {
                song_index,
                slide_in_song,
                ..
            } => {
                assert_eq!(song_index, 0);
                assert_eq!(slide_in_song, 0);
            }
            other => panic!("expected Goto, got {other:?}"),
        }
        assert_eq!(c.state(), MachineState::Singing);
        assert!(!c.is_blank());
        assert_eq!(c.current_song_title(), Some("Amazing Grace"));
    }

    #[test]
    fn unmatched_speech_stays_listening() {
        let mut c = Conductor::new(mk_setlist(), SmartConfig::default());
        // Words that don't appear in any setlist song. The absolute-min
        // partial_ratio gate (min_song_match) must reject this even if the
        // softmax favours one candidate.
        let action = c.on_transcript(
            "the rain in Spain falls mainly on the plain",
            Instant::now(),
        );
        assert!(matches!(action, Action::Noop));
        assert_eq!(c.state(), MachineState::Listening);
        assert!(c.is_blank());
    }

    #[test]
    fn silence_to_blank_then_to_blank_hold() {
        let mut c = Conductor::new(mk_setlist(), cfg_with_dwell(8.0));
        let t0 = Instant::now();
        // Get into SINGING.
        let _ = c.on_transcript(
            "amazing grace how sweet the sound that saved a wretch like me",
            t0,
        );
        assert_eq!(c.state(), MachineState::Singing);
        // 4 s later, no speech → InterVerseSilence (slide held, not yet blanked).
        let action = c.tick(t0 + Duration::from_secs(4));
        assert!(matches!(action, Action::Noop));
        assert_eq!(c.state(), MachineState::InterVerseSilence);
        assert!(!c.is_blank());
        // Another tick past unrecognized_speech_to_blank → BlankHold + Blank action.
        let action = c.tick(t0 + Duration::from_secs(10));
        assert!(matches!(action, Action::Blank));
        assert_eq!(c.state(), MachineState::BlankHold);
        assert!(c.is_blank());
    }

    #[test]
    fn dwell_blocks_immediate_resong_switch() {
        // Tight dwell test: enter song 0, then *immediately* fire a song-1
        // matching transcript; the conductor must NOT switch yet because
        // min_song_dwell hasn't elapsed.
        let mut c = Conductor::new(mk_setlist(), cfg_with_dwell(8.0));
        let t0 = Instant::now();
        let _ = c.on_transcript(
            "amazing grace how sweet the sound that saved a wretch like me",
            t0,
        );
        assert_eq!(c.current_song_title(), Some("Amazing Grace"));
        // Right after, sing a line of song 1.
        let _ = c.on_transcript(
            "early in the morning our song shall rise to Thee",
            t0 + Duration::from_millis(500),
        );
        assert_eq!(
            c.current_song_title(),
            Some("Amazing Grace"),
            "dwell window should hold us in the original song"
        );
    }

    #[test]
    fn dwell_satisfied_allows_song_switch() {
        let mut c = Conductor::new(mk_setlist(), cfg_with_dwell(2.0));
        let t0 = Instant::now();
        let _ = c.on_transcript(
            "amazing grace how sweet the sound that saved a wretch like me",
            t0,
        );
        // After dwell expires, a song-1 match SHOULD switch.
        let _ = c.on_transcript(
            "early in the morning our song shall rise to Thee",
            t0 + Duration::from_secs(5),
        );
        assert_eq!(c.current_song_title(), Some("Holy, Holy, Holy"));
    }

    #[test]
    fn jump_to_song_overrides_state() {
        let mut c = Conductor::new(mk_setlist(), SmartConfig::default());
        let t0 = Instant::now();
        // Establish that we're nowhere.
        assert!(c.current_song_title().is_none());
        let action = c.jump_to_song(2, t0);
        match action {
            Action::Goto {
                song_index,
                slide_in_song,
                ..
            } => {
                assert_eq!(song_index, 2);
                assert_eq!(slide_in_song, 0);
            }
            other => panic!("expected Goto from jump_to_song, got {other:?}"),
        }
        assert_eq!(c.current_song_title(), Some("Come, Thou Fount"));
        assert_eq!(c.state(), MachineState::Singing);
        assert!(!c.is_blank());
    }

    #[test]
    fn jump_out_of_range_is_noop() {
        let mut c = Conductor::new(mk_setlist(), SmartConfig::default());
        let action = c.jump_to_song(99, Instant::now());
        assert!(matches!(action, Action::Noop));
        assert!(c.current_song_title().is_none());
    }

    #[test]
    fn config_hot_swap_keeps_position() {
        let mut c = Conductor::new(mk_setlist(), SmartConfig::default());
        let _ = c.on_transcript(
            "amazing grace how sweet the sound that saved a wretch like me",
            Instant::now(),
        );
        let before_song = c.current_song_title().map(String::from);
        let before_index = c.current_index();
        c.set_config(SmartConfig {
            silence_to_blank_secs: 10.0,
            ..Default::default()
        });
        assert_eq!(c.current_song_title().map(String::from), before_song);
        assert_eq!(c.current_index(), before_index);
    }

    #[test]
    fn manual_step_within_song_advances() {
        let mut c = Conductor::new(mk_setlist(), SmartConfig::default());
        let t0 = Instant::now();
        // Get into song 0, slide 0.
        let _ = c.on_transcript(
            "amazing grace how sweet the sound that saved a wretch like me",
            t0,
        );
        assert_eq!(c.current_index(), 0);
        let action = c.manual_step(1, t0 + Duration::from_millis(100));
        match action {
            Action::Goto {
                song_index,
                slide_in_song,
                global_index,
                ..
            } => {
                assert_eq!(song_index, 0);
                assert_eq!(slide_in_song, 1);
                assert_eq!(global_index, 1);
            }
            other => panic!("expected Goto, got {other:?}"),
        }
        assert_eq!(c.current_index(), 1);
    }

    #[test]
    fn manual_step_clamped_at_song_end() {
        let mut c = Conductor::new(mk_setlist(), SmartConfig::default());
        let t0 = Instant::now();
        let _ = c.on_transcript(
            "amazing grace how sweet the sound that saved a wretch like me",
            t0,
        );
        // Song 0 has 3 slides; step forward 5 times — last 3 must be clamped.
        for _ in 0..5 {
            c.manual_step(1, t0);
        }
        assert_eq!(c.current_index(), 2, "clamped to last slide of song");
    }

    #[test]
    fn manual_step_outside_song_is_noop() {
        let mut c = Conductor::new(mk_setlist(), SmartConfig::default());
        let action = c.manual_step(1, Instant::now());
        assert!(matches!(action, Action::Noop));
    }

    #[test]
    fn goto_action_carries_next_slide_text() {
        let mut c = Conductor::new(mk_setlist(), SmartConfig::default());
        // Land on song 0 slide 0 — next slide is song 0 slide 1.
        let action = c.on_transcript(
            "amazing grace how sweet the sound that saved a wretch like me",
            Instant::now(),
        );
        match action {
            Action::Goto {
                next_slide_text, ..
            } => {
                let next = next_slide_text.expect("song 0 slide 1 exists");
                assert!(next.contains("I once was lost"));
            }
            other => panic!("expected Goto, got {other:?}"),
        }
    }

    #[test]
    fn goto_action_next_text_none_at_song_end() {
        let mut c = Conductor::new(mk_setlist(), SmartConfig::default());
        let t0 = Instant::now();
        let _ = c.on_transcript(
            "amazing grace how sweet the sound that saved a wretch like me",
            t0,
        );
        // Step to the last slide (index 2 of 3).
        c.manual_step(1, t0);
        let action = c.manual_step(1, t0);
        match action {
            Action::Goto {
                next_slide_text, ..
            } => {
                assert!(next_slide_text.is_none(), "no next slide at song end");
            }
            other => panic!("expected Goto, got {other:?}"),
        }
    }

    #[test]
    fn external_blank_recovers_on_next_match() {
        // Operator (or tray, or HTTP API) blanks externally mid-song. The
        // conductor's next on_transcript with a matching lyric must emit an
        // unblank — otherwise the audience stays blank through the rest of
        // the verse.
        let mut c = Conductor::new(mk_setlist(), SmartConfig::default());
        let t0 = Instant::now();
        let _ = c.on_transcript(
            "amazing grace how sweet the sound that saved a wretch like me",
            t0,
        );
        assert!(!c.is_blank());
        // Simulate the operator hitting Blank externally.
        c.notify_external_blank();
        assert!(c.is_blank());
        // Now feed another transcript that still matches the same song. The
        // conductor must respond with either a Goto (to re-show the slide)
        // or Unblank. Either way, output should be visible after.
        let action = c.on_transcript(
            "I once was lost but now am found was blind but now I see",
            t0 + Duration::from_secs(2),
        );
        assert!(
            matches!(action, Action::Goto { .. } | Action::Unblank),
            "expected Goto or Unblank, got {action:?}"
        );
        assert!(
            !c.is_blank(),
            "conductor should have cleared its blank flag"
        );
    }

    fn example_setlist() -> Vec<Song> {
        let json = include_str!("../../../setlist.example.json");
        serde_json::from_str::<Setlist>(json).unwrap().setlist
    }

    const VERSE_1: &str = "amazing grace how sweet the sound that saved a wretch like me";

    #[test]
    fn prayer_after_blank_keeps_the_lyrics_off() {
        // Audit 1 / Q2-02: song, 10 s of silence (blank), then a prayer.
        let mut c = Conductor::new(example_setlist(), SmartConfig::default());
        let t0 = Instant::now();
        assert!(matches!(c.on_transcript(VERSE_1, t0), Action::Goto { .. }));
        let blanks = (1..=50)
            .map(|i| c.tick(t0 + Duration::from_millis(200 * i)))
            .filter(|a| matches!(a, Action::Blank))
            .count();
        assert_eq!(blanks, 1);
        assert!(c.is_blank());
        let action = c.on_transcript(
            "let us bow our heads and pray father we thank you for this morning",
            t0 + Duration::from_secs(11),
        );
        assert!(matches!(action, Action::Noop), "got {action:?}");
        assert!(c.is_blank(), "a prayer must not re-show the lyrics");
    }

    #[test]
    fn words_heard_before_a_blank_cannot_bring_the_lyrics_back() {
        // Review of q2/ponytail: two lines on slide 1 leave lyric words in the
        // rolling buffer; after the silence blank, a prayer must not re-show
        // the song on the strength of those old words.
        let mut c = Conductor::new(example_setlist(), SmartConfig::default());
        let t0 = Instant::now();
        assert!(matches!(c.on_transcript(VERSE_1, t0), Action::Goto { .. }));
        let line2 = "i once was lost but now am found was blind but now i see";
        let _ = c.on_transcript(line2, t0 + Duration::from_secs(5));
        let t1 = t0 + Duration::from_secs(5);
        let blanked = (1..=60)
            .map(|i| c.tick(t1 + Duration::from_millis(200 * i)))
            .any(|a| matches!(a, Action::Blank));
        assert!(blanked && c.is_blank());
        let prayer = "let us bow our heads and pray father we thank you for this morning";
        let action = c.on_transcript(prayer, t1 + Duration::from_secs(13));
        assert!(matches!(action, Action::Noop), "got {action:?}");
        assert!(c.is_blank());
    }

    #[test]
    fn continuous_unmatched_speech_blanks() {
        // Audit 1 / Q2-02: the pastor talks straight after the last song. VAD
        // hears speech all the time, so the silence timer never fires.
        let cfg = SmartConfig::default();
        let limit = Duration::from_secs_f64(cfg.unrecognized_speech_to_blank_secs);
        let mut c = Conductor::new(example_setlist(), cfg);
        let t0 = Instant::now();
        let _ = c.on_transcript(VERSE_1, t0);
        let sermon = [
            "good morning church please turn in your bibles to the book of romans",
            "and the apostle paul tells us that nothing can separate us from god",
            "so this week i want us to think about what that means for our families",
        ];
        let tick = Duration::from_millis(200);
        let mut first_unmatched = None;
        let mut blanked_at = None;
        for step in 1..=600u32 {
            let now = t0 + tick * step;
            c.on_speech(now);
            let mut action = Action::Noop;
            if step % 25 == 0 {
                first_unmatched.get_or_insert(now);
                action = c.on_transcript(sermon[(step / 25) as usize % 3], now);
            }
            if matches!(action, Action::Blank) || matches!(c.tick(now), Action::Blank) {
                blanked_at = Some(now);
                break;
            }
        }
        let start = first_unmatched.unwrap();
        let blanked = blanked_at.expect("never blanked");
        assert!(
            blanked - start <= limit + tick,
            "after {:?}",
            blanked - start
        );
        assert!(c.is_blank());
    }

    #[test]
    fn one_interjection_mid_song_does_not_blank() {
        // The leader shouts between lines while the room keeps singing; the
        // next transcript is at least 5 s away, so one stray line must not
        // blank the lyrics in the meantime.
        let mut c = Conductor::new(example_setlist(), SmartConfig::default());
        let t0 = Instant::now();
        assert!(matches!(c.on_transcript(VERSE_1, t0), Action::Goto { .. }));
        let shout = c.on_transcript(
            "come on church sing it out everybody",
            t0 + Duration::from_secs(5),
        );
        assert!(matches!(shout, Action::Noop), "got {shout:?}");
        for i in 26..=50u32 {
            let now = t0 + Duration::from_millis(200 * u64::from(i));
            c.on_speech(now);
            let action = c.tick(now);
            assert!(!matches!(action, Action::Blank), "blanked at {i}");
        }
        assert!(!c.is_blank());
    }

    #[test]
    fn a_shout_out_shorter_than_the_blank_limit_does_not_blank() {
        // Transcripts now come every 1-2 s (ADR-014): a 4.8 s shout-out mid
        // song is several unmatched transcripts in a row, still no blank.
        let mut c = Conductor::new(example_setlist(), SmartConfig::default());
        let t0 = Instant::now();
        assert!(matches!(c.on_transcript(VERSE_1, t0), Action::Goto { .. }));
        let shout = [
            "come on church",
            "sing it out",
            "everybody now",
            "lift it up",
        ];
        let shout = shout.map(|s| format!("{s} {s}"));
        for ms in [2000, 3600, 5200, 6800] {
            let now = t0 + Duration::from_millis(ms);
            let line = &shout[(ms as usize - 2000) / 1600];
            let action = c.on_transcript(line, now);
            assert!(matches!(action, Action::Noop), "{line}: {action:?}");
        }
        assert!(!c.is_blank());
        // 5 s of them blank.
        let action = c.on_transcript(&shout[0], t0 + Duration::from_millis(7000));
        assert!(matches!(action, Action::Blank), "{action:?}");
    }

    fn slide_of(action: &Action) -> Option<(usize, usize)> {
        match action {
            Action::Goto {
                song_index,
                slide_in_song,
                ..
            } => Some((*song_index, *slide_in_song)),
            _ => None,
        }
    }

    #[test]
    fn song_detected_mid_way_starts_at_the_matching_slide() {
        // Audit 5 / Q2-07: listening (re)starts during verse 3.
        let mut c = Conductor::new(example_setlist(), SmartConfig::default());
        let verse_3 = "through many dangers toils and snares i have already come \
                       tis grace hath brought me safe thus far and grace will lead me home";
        let action = c.on_transcript(verse_3, Instant::now());
        assert_eq!(slide_of(&action), Some((0, 2)));
    }

    #[test]
    fn inline_edit_keeps_position_and_advances_by_one() {
        // Audit 5 / Q2-07: editing the current slide used to reload the setlist.
        let mut c = Conductor::new(example_setlist(), SmartConfig::default());
        let t0 = Instant::now();
        let _ = c.on_transcript(VERSE_1, t0);
        let _ = c.manual_step(1, t0);
        assert!(c.update_slide_text(1, "Twas grace that taught my heart to fear".into()));
        assert_eq!((c.current_song(), c.current_index()), (Some(0), 1));
        assert!(!c.is_blank());
        let action = c.on_transcript(
            "through many dangers toils and snares i have already come",
            t0 + Duration::from_secs(3),
        );
        assert_eq!(slide_of(&action), Some((0, 2)));
        assert!(!c.update_slide_text(99, String::new()));
    }

    #[test]
    fn empty_slides_are_never_matched() {
        // Q2-05: importers keep empty slides as placeholders.
        let mut songs = example_setlist();
        songs[0].slides.insert(
            1,
            Slide {
                id: 99,
                text: String::new(),
            },
        );
        let mut c = Conductor::new(songs, SmartConfig::default());
        let t0 = Instant::now();
        assert_eq!(slide_of(&c.on_transcript(VERSE_1, t0)), Some((0, 0)));
        // The next lyric skips the placeholder (slide 1) and lands on slide 2.
        let action = c.on_transcript(
            "twas grace that taught my heart to fear and grace my fears relieved",
            t0 + Duration::from_secs(3),
        );
        assert_eq!(slide_of(&action), Some((0, 2)));
        assert_eq!(c.best_slide(0, ""), 0);
    }

    #[test]
    fn whole_libraries_are_refused_as_a_setlist() {
        // Q2-17 (audit 13): an EasyWorship import used to load every song.
        let song = example_setlist().remove(0);
        let at_most = vec![song.clone(); MAX_SETLIST_SONGS];
        assert_eq!(check_setlist(&at_most), Ok(()));
        let err = check_setlist(&vec![song; MAX_SETLIST_SONGS + 1]).unwrap_err();
        assert!(err.contains("pick today's songs"), "{err}");
        assert!(check_setlist(&[]).is_err());
    }

    #[test]
    fn short_transcripts_never_start_a_song() {
        // PT2-01: partial_ratio of a 1-3 word transcript is ~100 against any song.
        for text in ["Oh", "soul", "Amen.", "Thank you.", "God is good."] {
            let mut c = Conductor::new(example_setlist(), SmartConfig::default());
            let action = c.on_transcript(text, Instant::now());
            assert!(matches!(action, Action::Noop), "{text}: {action:?}");
        }
    }

    #[test]
    fn short_transcripts_after_a_blank_keep_the_lyrics_off() {
        let mut c = Conductor::new(example_setlist(), SmartConfig::default());
        let t0 = Instant::now();
        assert!(matches!(c.on_transcript(VERSE_1, t0), Action::Goto { .. }));
        let _ = c.tick(t0 + Duration::from_secs(4));
        assert!(matches!(
            c.tick(t0 + Duration::from_secs(11)),
            Action::Blank
        ));
        for text in ["the", "me", "Lord"] {
            let action = c.on_transcript(text, t0 + Duration::from_secs(12));
            assert!(matches!(action, Action::Noop), "{text}: {action:?}");
            assert!(c.is_blank(), "{text}");
        }
    }

    #[test]
    fn an_announcement_never_starts_the_only_song() {
        // One song makes softmax p = 1.0, so only the absolute gates can refuse.
        for song in example_setlist() {
            let mut c = Conductor::new(vec![song], SmartConfig::default());
            let text = "Good morning everyone, please take your seats.";
            let action = c.on_transcript(text, Instant::now());
            assert!(matches!(action, Action::Noop), "{action:?}");
        }
    }

    #[test]
    fn a_misheard_lyric_line_still_starts_the_right_slide() {
        let t0 = Instant::now();
        let mut c = Conductor::new(vec![example_setlist().remove(1)], SmartConfig::default());
        let action = c.on_transcript("Be thou my vision, oh Lord of my heart.", t0);
        assert_eq!(slide_of(&action), Some((0, 0)));
        let mut c = Conductor::new(example_setlist(), SmartConfig::default());
        let action = c.on_transcript("Though Satan should buffet, though trials should come", t0);
        assert_eq!(slide_of(&action), Some((2, 2)));
    }

    #[test]
    fn worship_phrases_never_start_a_song_or_re_show_lyrics() {
        // PT2-01 residual: these score 70-81 against It Is Well, real lines 94-100.
        let phrase = "Come on, praise the Lord!";
        let t0 = Instant::now();
        let mut c = Conductor::new(example_setlist(), SmartConfig::default());
        let action = c.on_transcript(phrase, t0);
        assert!(matches!(action, Action::Noop), "{action:?}");
        assert!(matches!(c.on_transcript(VERSE_1, t0), Action::Goto { .. }));
        let _ = c.tick(t0 + Duration::from_secs(4));
        assert!(matches!(
            c.tick(t0 + Duration::from_secs(11)),
            Action::Blank
        ));
        let action = c.on_transcript(phrase, t0 + Duration::from_secs(12));
        assert!(matches!(action, Action::Noop), "{action:?}");
        assert!(c.is_blank());
        let mut c = Conductor::new(vec![example_setlist().remove(2)], SmartConfig::default());
        let action = c.on_transcript("Let's give the Lord praise.", t0);
        assert!(matches!(action, Action::Noop), "{action:?}");
    }

    #[test]
    fn worship_phrases_during_a_blank_never_advance_to_the_next_slide() {
        // PT2-01 residual (§8.6): the in-song advance needs the start bar too.
        let t0 = Instant::now();
        let mut c = Conductor::new(example_setlist(), SmartConfig::default());
        let line = "Though Satan should buffet, though trials should come";
        assert_eq!(slide_of(&c.on_transcript(line, t0)), Some((2, 2)));
        let blanks = (1..=80)
            .map(|i| c.tick(t0 + Duration::from_millis(200 * i)))
            .filter(|a| matches!(a, Action::Blank))
            .count();
        assert_eq!(blanks, 1);
        let phrases = [
            "Come on, praise the Lord!",
            "Let's give the Lord praise.",
            "Praise the Lord, everybody",
            "Let's praise the Lord, praise the Lord together church",
        ];
        for (s, phrase) in (17..).zip(phrases) {
            let action = c.on_transcript(phrase, t0 + Duration::from_secs(s));
            assert!(matches!(action, Action::Noop), "{phrase}: {action:?}");
            assert!(c.is_blank(), "{phrase}");
        }
        let line = "Is nailed to the cross, and I bear it no more";
        let action = c.on_transcript(line, t0 + Duration::from_secs(22));
        assert_eq!(slide_of(&action), Some((2, 3)));
        assert!(!c.is_blank());
        let mut c = Conductor::new(example_setlist(), SmartConfig::default());
        let line = "Through many dangers, toils and snares I have already come";
        assert_eq!(slide_of(&c.on_transcript(line, t0)), Some((0, 2)));
        assert!(
            (1..=80).any(|i| matches!(c.tick(t0 + Duration::from_millis(200 * i)), Action::Blank))
        );
        let action = c.on_transcript(
            "The Lord has promised good things to us",
            t0 + Duration::from_secs(17),
        );
        assert!(matches!(action, Action::Noop) && c.is_blank(), "{action:?}");
    }

    #[test]
    fn a_loose_line_mid_song_keeps_the_lyrics_up() {
        // Keeping lyrics up uses the advance bar (70), not the start bar (85).
        let t0 = Instant::now();
        let mut c = Conductor::new(example_setlist(), SmartConfig::default());
        let line = "When peace, like a river, attendeth my way";
        assert_eq!(slide_of(&c.on_transcript(line, t0)), Some((2, 0)));
        for s in [1, 2] {
            let _ = c.on_transcript("Come on, praise the Lord!", t0 + Duration::from_secs(s));
        }
        assert!(!c.is_blank());
    }

    #[test]
    fn last_scores_populated_after_transcript() {
        let mut c = Conductor::new(mk_setlist(), SmartConfig::default());
        assert!(c.last_scores().is_empty());
        let _ = c.on_transcript(
            "amazing grace how sweet the sound that saved a wretch like me",
            Instant::now(),
        );
        let scores = c.last_scores();
        assert_eq!(scores.len(), 3, "one entry per song in the setlist");
        // The top entry must be song 0 (Amazing Grace).
        assert_eq!(scores[0].song_index, 0);
        // Probabilities sum to ~1 (softmax).
        let total: f64 = scores.iter().map(|s| s.probability).sum();
        assert!((total - 1.0).abs() < 1e-6);
    }
}
