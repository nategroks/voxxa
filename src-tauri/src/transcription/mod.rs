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

/// Transcription result from Whisper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub language: Option<String>,
    pub duration_ms: u64,
    pub is_partial: bool,
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

    /// Transcribe an audio buffer (16kHz mono f32 samples).
    pub fn transcribe(&self, samples: &[f32]) -> Result<TranscriptionResult> {
        let ctx = self
            .ctx
            .as_ref()
            .context("No model loaded. Call load_model first.")?;

        let start = std::time::Instant::now();

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

        state
            .full(params, samples)
            .context("Whisper transcription failed")?;

        let num_segments = state.full_n_segments().context("Failed to get segments")?;
        let mut text = String::new();
        for i in 0..num_segments {
            if let Ok(segment) = state.full_get_segment_text(i) {
                text.push_str(&segment);
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(TranscriptionResult {
            text: text.trim().to_string(),
            language: self.language.clone(),
            duration_ms,
            is_partial: false,
        })
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
