//! A stand-in fixture's `audio.json` recipe (ADR-015 Decision 4): silence
//! and committed 16 kHz mono 16-bit WAVs, concatenated sample for sample
//! into one WAV. The bytes depend only on the committed files, so every
//! machine rebuilds the same `audio.wav`.

use serde_json::Value;
use std::fs;
use std::path::Path;

/// The WAV bytes `recipe` (an `audio.json` path) describes; `wav` paths in
/// it are relative to the recipe's folder.
pub fn build(recipe: &Path) -> Result<Vec<u8>, String> {
    let at = |e: &dyn std::fmt::Display| format!("{}: {e}", recipe.display());
    let text = fs::read_to_string(recipe).map_err(|e| at(&e))?;
    let v: Value = serde_json::from_str(&text).map_err(|e| at(&e))?;
    if v["v"] != 1 {
        return Err(at(&"need \"v\": 1"));
    }
    let parts = v["parts"]
        .as_array()
        .ok_or_else(|| at(&"no \"parts\" array"))?;
    let dir = recipe.parent().unwrap_or(Path::new("."));
    let mut pcm: Vec<u8> = Vec::new();
    for part in parts {
        if let Some(ms) = part["silence_ms"].as_u64() {
            pcm.resize(pcm.len() + ms as usize * 32, 0);
        } else if let Some(file) = part["wav"].as_str() {
            let path = dir.join(file);
            let bytes = fs::read(&path).map_err(|e| at(&format!("{}: {e}", path.display())))?;
            pcm.extend(pcm16_mono_16k(&bytes).map_err(|e| at(&format!("{file}: {e}")))?);
        } else {
            return Err(at(&format!("a part is neither silence_ms nor wav: {part}")));
        }
    }
    Ok(wav(&pcm))
}

/// The data chunk of a 16 kHz mono 16-bit PCM WAV.
pub fn pcm16_mono_16k(bytes: &[u8]) -> Result<&[u8], String> {
    if bytes.get(..4) != Some(b"RIFF") || bytes.get(8..12) != Some(b"WAVE") {
        return Err("not a WAV file".into());
    }
    let (mut pos, mut format_ok) = (12, false);
    while let Some(head) = bytes.get(pos..pos + 8) {
        let len = u32::from_le_bytes([head[4], head[5], head[6], head[7]]) as usize;
        let body = &bytes[pos + 8..(pos + 8 + len).min(bytes.len())];
        match &head[..4] {
            // PCM, 1 channel, 16000 Hz, then 16 bits per sample at byte 14.
            b"fmt " => {
                format_ok = body.get(..8) == Some(&[1, 0, 1, 0, 0x80, 0x3e, 0, 0][..])
                    && body.get(14) == Some(&16)
            }
            b"data" if format_ok => return Ok(body),
            b"data" => return Err("need 16 kHz mono 16-bit PCM".into()),
            _ => {}
        }
        pos += 8 + len + (len & 1);
    }
    Err("no data chunk".into())
}

/// A 16 kHz mono 16-bit PCM WAV around `pcm`.
pub fn wav(pcm: &[u8]) -> Vec<u8> {
    let len = pcm.len() as u32;
    let mut out = Vec::with_capacity(44 + pcm.len());
    out.extend(b"RIFF");
    out.extend((36 + len).to_le_bytes());
    out.extend(b"WAVEfmt ");
    out.extend(16u32.to_le_bytes());
    out.extend([1, 0, 1, 0]); // PCM, mono
    out.extend(16_000u32.to_le_bytes());
    out.extend(32_000u32.to_le_bytes()); // bytes per second
    out.extend([2, 0, 16, 0]); // block align, bits per sample
    out.extend(b"data");
    out.extend(len.to_le_bytes());
    out.extend(pcm);
    out
}
