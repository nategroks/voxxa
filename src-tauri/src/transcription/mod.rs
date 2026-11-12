use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Available Whisper model variants.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum WhisperModel {
    /// Tiny model (~75MB) - fastest, least accurate
    Tiny,
    /// Base model (~142MB) - good balance for quick tasks
    #[default]
    Base,
    /// Small model (~466MB) - solid accuracy
    Small,
    /// Medium model (~1.5GB) - high accuracy
    Medium,
    /// Large-v3-turbo (~1.6GB) - best for multilingual
    LargeV3Turbo,
    /// Distil-large-v3 (~756MB) - best for English
    DistilLargeV3,
}

impl WhisperModel {
    pub fn filename(&self) -> &str {
        match self {
            Self::Tiny => "ggml-tiny.bin",
            Self::Base => "ggml-base.bin",
            Self::Small => "ggml-small.bin",
            Self::Medium => "ggml-medium.bin",
            Self::LargeV3Turbo => "ggml-large-v3-turbo.bin",
            Self::DistilLargeV3 => "ggml-distil-large-v3.bin",
        }
    }

    pub fn download_url(&self) -> String {
        let base = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main";
        format!("{}/{}", base, self.filename())
    }

    pub fn display_name(&self) -> &str {
        match self {
            Self::Tiny => "Tiny (75MB)",
            Self::Base => "Base (142MB)",
            Self::Small => "Small (466MB)",
            Self::Medium => "Medium (1.5GB)",
            Self::LargeV3Turbo => "Large V3 Turbo (1.6GB)",
            Self::DistilLargeV3 => "Distil Large V3 (756MB)",
        }
    }
}

/// Manages Whisper model loading and transcription.
pub struct TranscriptionEngine {
    model_dir: PathBuf,
    current_model: Option<WhisperModel>,
    ctx: Option<whisper_rs::WhisperContext>,
    language: Option<String>,
}

impl Default for TranscriptionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TranscriptionEngine {
    pub fn new() -> Self {
        let model_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("voxxa")
            .join("models");
        Self::with_model_dir(model_dir)
    }

    /// An engine keeping its models in `model_dir` (tests use a temp dir).
    pub fn with_model_dir(model_dir: PathBuf) -> Self {
        Self {
            model_dir,
            current_model: None,
            ctx: None,
            language: None,
        }
    }

    /// Get the directory where models are stored.
    pub fn model_dir(&self) -> &PathBuf {
        &self.model_dir
    }

    /// Check if a model file exists locally.
    pub fn is_model_downloaded(&self, model: &WhisperModel) -> bool {
        self.model_dir.join(model.filename()).exists()
    }

    /// Get the status of all available models. `loaded` indicates which model
    /// (if any) is currently in memory and ready for transcription.
    pub fn model_status(&self) -> Vec<(WhisperModel, bool, bool)> {
        let active = self.current_model.as_ref();
        vec![
            WhisperModel::Tiny,
            WhisperModel::Base,
            WhisperModel::Small,
            WhisperModel::Medium,
            WhisperModel::LargeV3Turbo,
            WhisperModel::DistilLargeV3,
        ]
        .into_iter()
        .map(|m| {
            let downloaded = self.is_model_downloaded(&m);
            let loaded = active == Some(&m);
            (m, downloaded, loaded)
        })
        .collect()
    }

    /// Currently loaded model, if any.
    pub fn current_model(&self) -> Option<&WhisperModel> {
        self.current_model.as_ref()
    }

    /// Current target language (None = auto-detect).
    pub fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }

    /// Load a Whisper model for transcription.
    pub fn load_model(&mut self, model: &WhisperModel) -> Result<()> {
        let model_path = self.model_dir.join(model.filename());
        if !model_path.exists() {
            anyhow::bail!(
                "Model {} not found. Please download it first.",
                model.display_name()
            );
        }

        let ctx = match whisper_rs::WhisperContext::new_with_params(
            &model_path.to_string_lossy(),
            whisper_rs::WhisperContextParameters::default(),
        ) {
            Ok(ctx) => ctx,
            Err(e) => {
                // A broken file (an older build saved partial downloads) would
                // otherwise show as Downloaded forever with no way to fetch it
                // again. Move it aside, not delete: it may be a good file that
                // failed for another reason.
                let aside = with_suffix(&model_path, ".unreadable");
                let _ = std::fs::rename(&model_path, &aside);
                anyhow::bail!(
                    "Failed to load Whisper model {} ({e}); moved it to {} so it can be downloaded again",
                    model.display_name(),
                    aside.display()
                );
            }
        };

        self.ctx = Some(ctx);
        self.current_model = Some(model.clone());
        log::info!("Loaded Whisper model: {}", model.display_name());
        Ok(())
    }

    /// Set the target language for transcription (None = auto-detect).
    pub fn set_language(&mut self, lang: Option<String>) {
        self.language = lang;
    }

    /// The words Whisper hears in `samples` (16 kHz mono), each with its start
    /// in seconds from the start of `samples` and its lowest token
    /// probability. `prompt` becomes Whisper's initial_prompt.
    pub fn words(&self, samples: &[f32], prompt: Option<&str>) -> Result<Vec<Word>> {
        let ctx = self
            .ctx
            .as_ref()
            .context("No model loaded. Call load_model first.")?;
        let mut state = ctx
            .create_state()
            .context("Failed to create Whisper state")?;

        let mut params =
            whisper_rs::FullParams::new(whisper_rs::SamplingStrategy::Greedy { best_of: 1 });
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_suppress_blank(true);
        params.set_suppress_non_speech_tokens(true);
        if let Some(ref lang) = self.language {
            params.set_language(Some(lang));
        }
        // Single-segment mode for low latency
        params.set_single_segment(true);
        params.set_no_context(true);
        params.set_token_timestamps(true);
        if let Some(prompt) = prompt {
            // shortcut: whisper-rs 0.13 leaks this CString on every call (a few
            // hundred bytes per hop while biasing is on); recheck on a bump.
            params.set_initial_prompt(prompt);
        }

        state
            .full(params, samples)
            .context("Whisper transcription failed")?;

        // Tokens from the end-of-text id up are special or timestamps. A
        // token that starts with a space starts a word; others continue it.
        let eot = ctx.token_eot();
        let mut words: Vec<Word> = Vec::new();
        for s in 0..state.full_n_segments().context("Failed to get segments")? {
            for i in 0..state.full_n_tokens(s).context("Failed to get tokens")? {
                let data = state.full_get_token_data(s, i)?;
                if data.id >= eot {
                    continue;
                }
                let text = state.full_get_token_text_lossy(s, i)?;
                match words.last_mut() {
                    Some(w) if !text.starts_with(' ') => {
                        w.0.push_str(&text);
                        w.2 = w.2.min(data.p);
                    }
                    // t0 is in 10 ms units.
                    _ => words.push((text.trim().to_string(), data.t0 as f32 / 100.0, data.p)),
                }
            }
        }
        words.retain(|w| !w.0.is_empty());
        Ok(words)
    }
}

/// A word, its start in seconds and its confidence (ADR-004).
pub type Word = (String, f32, f32);

/// What a recogniser heard in its latest window (ADR-004); word times are
/// seconds of audio since the stream started.
#[derive(Debug, Clone, PartialEq)]
pub struct Hypothesis {
    pub words: Vec<Word>,
}

impl Hypothesis {
    /// The words, space-separated: what the conductor reads.
    pub fn text(&self) -> String {
        let words: Vec<&str> = self.words.iter().map(|w| w.0.as_str()).collect();
        words.join(" ")
    }
}

/// The ADR-004 seam between audio and the conductor (ADR-014).
pub trait Recognizer {
    /// The next audio of the stream (16 kHz mono). A slice of digital silence
    /// (all zeros: the loop mutes audio in which the VAD heard no speech) is
    /// kept in the window but not transcribed. A hypothesis has a word.
    fn feed(&mut self, pcm16k: &[f32]) -> Result<Vec<Hypothesis>>;
    /// Text the recogniser should expect next, such as the coming lyrics.
    fn bias(&mut self, text: &str);
}

/// Audio a hop must hold before the loop feeds it (ADR-004: 1 s hop).
pub const HOP_MS: usize = 1000;
/// The rolling window Whisper reads on each feed (ADR-004: 5 s window).
pub const WINDOW_MS: usize = 5000;

/// Whisper over a rolling window: each feed transcribes the last
/// [`WINDOW_MS`] of audio with `infer` (`TranscriptionEngine::words`, or a
/// fake in tests) and turns window times into stream times.
pub struct Rolling<F> {
    infer: F,
    window: Vec<f32>,
    /// Samples fed since the stream started.
    fed: u64,
    prompt: Option<String>,
    /// Pass the text from `bias` to Whisper as initial_prompt. Off: Whisper
    /// can repeat its prompt as if it heard it (research E6); measure first.
    pub bias_enabled: bool,
}

impl<F: FnMut(&[f32], Option<&str>) -> Result<Vec<Word>>> Rolling<F> {
    pub fn new(infer: F) -> Self {
        Rolling {
            infer,
            window: Vec::new(),
            fed: 0,
            prompt: None,
            bias_enabled: false,
        }
    }
}

impl<F: FnMut(&[f32], Option<&str>) -> Result<Vec<Word>>> Recognizer for Rolling<F> {
    fn feed(&mut self, pcm16k: &[f32]) -> Result<Vec<Hypothesis>> {
        self.fed += pcm16k.len() as u64;
        self.window.extend_from_slice(pcm16k);
        let excess = self.window.len().saturating_sub(WINDOW_MS * 16);
        self.window.drain(..excess);
        if pcm16k.iter().all(|&s| s == 0.0) {
            return Ok(Vec::new());
        }
        let start = (self.fed - self.window.len() as u64) as f32 / 16_000.0;
        let prompt = self.prompt.as_deref().filter(|_| self.bias_enabled);
        let words: Vec<Word> = ((self.infer)(&self.window, prompt)?.into_iter())
            .map(|(w, t, p)| (w, start + t, p))
            .collect();
        Ok(match words.is_empty() {
            true => Vec::new(),
            false => vec![Hypothesis { words }],
        })
    }

    fn bias(&mut self, text: &str) {
        self.prompt = Some(text.to_string());
    }
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

/// Download `model` from `url` without holding the engine lock, so the
/// listening loop keeps transcribing (and slides keep moving) during a long
/// download. Then load it if no model is loaded yet.
pub async fn download_and_load<F>(
    engine: &tokio::sync::Mutex<TranscriptionEngine>,
    model: &WhisperModel,
    url: &str,
    progress: F,
) -> Result<()>
where
    F: Fn(u64, u64) + Send + 'static,
{
    let dir = engine.lock().await.model_dir.clone();
    std::fs::create_dir_all(&dir).context("Failed to create model directory")?;
    let dest = dir.join(model.filename());
    log::info!("Downloading {} to {:?}", model.display_name(), dest);
    download_file(url, &dest, progress).await?;
    log::info!("Download complete: {}", model.display_name());
    let mut engine = engine.lock().await;
    if engine.current_model().is_none() {
        if let Err(e) = engine.load_model(model) {
            log::warn!("auto-load after download failed: {e}");
        }
    }
    Ok(())
}

/// Download `url` to `dest` through `<dest>.part`. An HTTP error status, a
/// dropped connection or a body shorter than Content-Length leaves neither
/// file behind, so a broken download is never listed as Downloaded.
pub async fn download_file<F>(url: &str, dest: &Path, progress: F) -> Result<()>
where
    F: Fn(u64, u64) + Send + 'static,
{
    // One download at a time: a second click on Download would truncate the
    // same .part file under the first download and corrupt the model.
    static DOWNLOADS: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
    let _one_at_a_time = DOWNLOADS.lock().await;
    let part = with_suffix(dest, ".part");
    match fetch_to(url, &part, progress).await {
        Ok(()) => std::fs::rename(&part, dest).context("Failed to finish the model file"),
        Err(e) => {
            let _ = std::fs::remove_file(&part);
            Err(e)
        }
    }
}

async fn fetch_to<F: Fn(u64, u64)>(url: &str, part: &Path, progress: F) -> Result<()> {
    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;

    crate::net_stats::record_request();
    let response = reqwest::Client::new()
        .get(url)
        .send()
        .await
        .context("Failed to start download")?
        .error_for_status()
        .context("The model server refused the download")?;
    let total = response.content_length();
    let mut file = tokio::fs::File::create(part)
        .await
        .context("Failed to create model file")?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Error downloading chunk")?;
        file.write_all(&chunk)
            .await
            .context("Failed to write chunk")?;
        downloaded += chunk.len() as u64;
        progress(downloaded, total.unwrap_or(0));
    }
    file.flush().await.context("Failed to write chunk")?;
    match total {
        Some(t) if t != downloaded => {
            anyhow::bail!("Download incomplete: {downloaded} of {t} bytes")
        }
        _ => Ok(()),
    }
}
