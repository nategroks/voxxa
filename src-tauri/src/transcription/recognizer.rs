//! The recogniser seam (ADR-004, ADR-014): any engine that turns a 16 kHz
//! stream into timed words. Whisper sits behind it as [`Rolling`], a 5 s
//! window decoded every 1 s of audio.

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Samples per second of the stream every recogniser is fed.
pub const RATE: u64 = 16_000;

/// A word, its start in stream seconds, and a confidence from 0 to 1.
pub type Word = (String, f32, f32);

/// What a recogniser heard in one hop.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Hypothesis {
    /// In order. Starts are stream time (samples fed / 16,000), not window
    /// time.
    pub words: Vec<Word>,
    /// Stream time, in seconds, where the decoded audio ends.
    pub end: f32,
    /// The hop's inference failed; `words` is empty.
    pub error: Option<String>,
}

impl Hypothesis {
    /// The words joined by spaces.
    pub fn text(&self) -> String {
        let words: Vec<&str> = self.words.iter().map(|w| w.0.as_str()).collect();
        words.join(" ")
    }
}

/// One segment's words: `text` (the segment's whole text) split on
/// whitespace, timed by `tokens` (text, start in seconds, probability).
/// A token led by a space, and the first token, starts a word: the word
/// takes its start and the mean probability of the word's tokens. Token
/// text can be cut inside a UTF-8 character, so only its leading space is
/// trusted. If the counts differ, the words left over take the last known
/// start and probability.
pub fn words_from_tokens(text: &str, tokens: &[(String, f32, f32)]) -> Vec<Word> {
    let mut starts: Vec<(f32, f32, f32)> = Vec::new(); // start, sum of p, tokens
    for (piece, t0, p) in tokens.iter().filter(|t| !t.0.trim().is_empty()) {
        match starts.last_mut() {
            Some(w) if !piece.starts_with(' ') => (w.1, w.2) = (w.1 + p, w.2 + 1.0),
            _ => starts.push((*t0, *p, 1.0)),
        }
    }
    (text.split_whitespace().enumerate())
        .map(|(i, word)| {
            let last = starts.last().copied().unwrap_or((0.0, 0.0, 1.0));
            let (t, p, n) = starts.get(i).copied().unwrap_or(last);
            (word.to_string(), t, p / n)
        })
        .collect()
}

/// The fewest words the conductor reads in a transcript
/// (`Conductor::on_transcript` drops shorter ones).
pub const MIN_WORDS: usize = 4;
/// Words the stitcher remembers: more than a 5 s window holds.
const KEEP: usize = 32;

/// Turns overlapping hypotheses into what the conductor hears (ADR-014
/// Decision 7). Consecutive windows share about 4 s of audio, so each
/// hypothesis starts with words already heard; only the words after them
/// are new. New words are held until there are [`MIN_WORDS`] of them, so
/// the conductor gets each word once, in order, every 1 to 3 s of singing.
#[derive(Debug, Default)]
pub struct Stitcher {
    /// The words taken from hypotheses so far (the latest [`KEEP`]).
    heard: Vec<Word>,
    /// New words not yet handed on.
    held: Vec<Word>,
}

impl Stitcher {
    /// The text to hand the conductor for one hypothesis's words, or `None`
    /// while fewer than [`MIN_WORDS`] new words are held.
    pub fn next(&mut self, words: &[Word]) -> Option<String> {
        let new = &words[overlap(&self.heard, words)..];
        self.heard.extend_from_slice(new);
        self.heard.drain(..self.heard.len().saturating_sub(KEEP));
        self.held.extend_from_slice(new);
        if self.held.len() < MIN_WORDS {
            return None;
        }
        let text: Vec<String> = self.held.drain(..).map(|w| w.0).collect();
        Some(text.join(" "))
    }

    /// After silence: fewer than [`MIN_WORDS`] held words, which the
    /// conductor would not read, are dropped. The words heard are kept
    /// while the next window may still hold some of them, so they are not
    /// handed on again; `forget` (the silence outlasted a window) clears
    /// them too, and a repeat is new again.
    pub fn pause(&mut self, forget: bool) {
        self.held.clear();
        if forget {
            self.heard.clear();
        }
    }
}

/// How many leading words of `new` repeat the end of `sent`. `new[0]` is
/// lined up with each word of `sent` in turn; the line-up where the most
/// words agree (ignoring case and punctuation, and at least half of those
/// compared) wins, and of equal ones the line-up whose word times agree
/// best. Whisper may mishear a word cut by the window edge, so not every
/// word has to agree.
pub fn overlap(sent: &[Word], new: &[Word]) -> usize {
    let key = |w: &Word| -> String {
        (w.0.chars().filter(|c| c.is_alphanumeric()))
            .flat_map(char::to_lowercase)
            .collect()
    };
    let (sent_keys, new_keys): (Vec<_>, Vec<_>) = (
        sent.iter().map(key).collect(),
        new.iter().map(key).collect(),
    );
    let mut best = (0, f32::MAX, 0); // words that agree, time gap, overlap
    for (i, at) in sent.iter().enumerate() {
        let len = (sent.len() - i).min(new.len());
        let agree = (0..len)
            .filter(|&k| sent_keys[i + k] == new_keys[k])
            .count();
        let gap = new.first().map_or(0.0, |w| (at.1 - w.1).abs());
        if agree > 0 && agree * 2 >= len && (agree > best.0 || agree == best.0 && gap < best.1) {
            best = (agree, gap, len);
        }
    }
    best.2
}

/// ADR-004's recogniser. `feed` gets every sample of the stream once, in
/// order.
pub trait Recognizer {
    fn feed(&mut self, pcm16k: &[f32]) -> Vec<Hypothesis>;
    /// Text the next decodes may be biased towards (the slides around the
    /// current one).
    fn bias(&mut self, text: &str);
    /// While closed, audio still enters the stream but nothing is decoded:
    /// the loop closes it on silence and while it is behind (ADR-014).
    /// Engines that cost little may ignore it.
    fn gate(&mut self, _open: bool) {}
}

/// Decodes one window of audio into words timed from the window's start.
pub trait Decode {
    fn decode(&mut self, window: &[f32], prompt: Option<&str>) -> Result<Vec<Word>>;
}

impl Decode for super::TranscriptionEngine {
    fn decode(&mut self, window: &[f32], prompt: Option<&str>) -> Result<Vec<Word>> {
        self.words(window, prompt)
    }
}

/// The GUI's shared engine: locked per decode, not per chunk, so model
/// commands are not held up between hops.
impl Decode for Arc<Mutex<super::TranscriptionEngine>> {
    fn decode(&mut self, window: &[f32], prompt: Option<&str>) -> Result<Vec<Word>> {
        self.blocking_lock().words(window, prompt)
    }
}

/// A rolling window over the stream, decoded once per hop while the gate is
/// open. Several hop boundaries in one `feed` give one decode, of the newest
/// window.
pub struct Rolling<D> {
    decode: D,
    pub window_ms: u64,
    pub hop_ms: u64,
    /// Pass the text given to [`Recognizer::bias`] to the engine (Whisper's
    /// `initial_prompt`). Off by default: it may hallucinate (ADR-004, E6).
    pub bias_enabled: bool,
    prompt: Option<String>,
    window: Vec<f32>,
    fed: u64,
    next_hop: u64,
    open: bool,
}

impl<D: Decode> Rolling<D> {
    /// A 5 s window and a 1 s hop, unbiased, gate open.
    pub fn new(decode: D) -> Self {
        Rolling {
            decode,
            window_ms: 5000,
            hop_ms: 1000,
            bias_enabled: false,
            prompt: None,
            window: Vec::new(),
            fed: 0,
            next_hop: 0,
            open: true,
        }
    }

    /// Samples fed so far.
    pub fn fed(&self) -> u64 {
        self.fed
    }

    pub fn decoder(&self) -> &D {
        &self.decode
    }
}

impl<D: Decode> Recognizer for Rolling<D> {
    fn feed(&mut self, pcm16k: &[f32]) -> Vec<Hypothesis> {
        let hop = (self.hop_ms * RATE / 1000).max(1);
        let keep = (self.window_ms * RATE / 1000) as usize;
        if self.next_hop == 0 {
            self.next_hop = hop;
        }
        self.window.extend_from_slice(pcm16k);
        self.fed += pcm16k.len() as u64;
        let over = self.window.len().saturating_sub(keep);
        self.window.drain(..over);
        if self.fed < self.next_hop {
            return Vec::new();
        }
        // The next boundary after everything fed: hops that fell inside
        // this call are not decoded one by one.
        self.next_hop = (self.fed / hop + 1) * hop;
        if !self.open {
            return Vec::new();
        }
        let start = (self.fed - self.window.len() as u64) as f32 / RATE as f32;
        let end = self.fed as f32 / RATE as f32;
        let prompt = self.prompt.as_deref().filter(|_| self.bias_enabled);
        let hypothesis = match self.decode.decode(&self.window, prompt) {
            Ok(words) if words.is_empty() => return Vec::new(),
            Ok(words) => Hypothesis {
                // Whisper's token times are a heuristic: late tokens can
                // land past the window or before the word ahead of them.
                // Clamp each to the window and never before the last word.
                words: (words.into_iter())
                    .scan(0.0f32, |last, (w, t, c)| {
                        *last = t.max(*last).min(end - start);
                        Some((w, start + *last, c))
                    })
                    .collect(),
                end,
                error: None,
            },
            Err(e) => Hypothesis {
                end,
                error: Some(e.to_string()),
                ..Hypothesis::default()
            },
        };
        vec![hypothesis]
    }

    fn bias(&mut self, text: &str) {
        let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
        self.prompt = Some(text).filter(|t| !t.is_empty());
    }

    fn gate(&mut self, open: bool) {
        self.open = open;
    }
}
