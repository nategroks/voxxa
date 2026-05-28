use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Available Whisper model variants.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WhisperModel {
    /// Tiny model (~75MB) - fastest, least accurate
    Tiny,
    /// Base model (~142MB) - good balance for quick tasks
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

impl Default for WhisperModel {
    fn default() -> Self {
        Self::Base
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

impl TranscriptionEngine {
    pub fn new() -> Self {
        let model_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("voxxa")
            .join("models");

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

        let ctx = whisper_rs::WhisperContext::new_with_params(
            model_path.to_str().unwrap(),
            whisper_rs::WhisperContextParameters::default(),
        )
        .context("Failed to load Whisper model")?;

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

        let mut state = ctx.create_state().context("Failed to create Whisper state")?;

        let mut params = whisper_rs::FullParams::new(whisper_rs::SamplingStrategy::Greedy { best_of: 1 });
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

    /// Download a model file with progress reporting.
    pub async fn download_model<F>(
        &self,
        model: &WhisperModel,
        progress_callback: F,
    ) -> Result<()>
    where
        F: Fn(u64, u64) + Send + 'static,
    {
        use futures_util::StreamExt;

        std::fs::create_dir_all(&self.model_dir)
            .context("Failed to create model directory")?;

        let url = model.download_url();
        let dest = self.model_dir.join(model.filename());

        log::info!("Downloading {} to {:?}", model.display_name(), dest);

        let client = reqwest::Client::new();
        crate::net_stats::record_request();
        let response = client
            .get(&url)
            .send()
            .await
            .context("Failed to start download")?;

        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded: u64 = 0;

        let mut file = tokio::fs::File::create(&dest)
            .await
            .context("Failed to create model file")?;

        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Error downloading chunk")?;
            tokio::io::AsyncWriteExt::write_all(&mut file, &chunk)
                .await
                .context("Failed to write chunk")?;
            downloaded += chunk.len() as u64;
            progress_callback(downloaded, total_size);
        }

        log::info!("Download complete: {}", model.display_name());
        Ok(())
    }
}
