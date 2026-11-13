//! A fixture's `audio.json` recipe: committed WAVs and silences joined into
//! the fixture's `audio.wav`, so `fixtures/` commits no audio (ADR-015).

use serde_json::Value;
use std::fs;
use std::path::Path;

/// `audio.json` is a list of parts, each `{"wav": <path from the fixture
/// folder>}` (16 kHz mono 16-bit PCM) or `{"silence_ms": <n>}`. Returns the
/// bytes of one 16 kHz mono 16-bit PCM WAV, the format `voxxad` reads.
pub fn build(dir: &Path, recipe: &str) -> Result<Vec<u8>, String> {
    let parts: Vec<Value> = serde_json::from_str(recipe).map_err(|e| format!("audio.json: {e}"))?;
    let mut pcm = Vec::new();
    for part in &parts {
        if let Some(ms) = part["silence_ms"].as_u64() {
            // 16 samples of 2 bytes per millisecond.
            pcm.resize(pcm.len() + ms as usize * 32, 0);
        } else if let Some(path) = part["wav"].as_str() {
            pcm.extend(pcm16(&dir.join(path))?);
        } else {
            return Err(format!(
                "audio.json: a part needs wav or silence_ms: {part}"
            ));
        }
    }
    let mut wav = b"RIFF".to_vec();
    wav.extend((36 + pcm.len() as u32).to_le_bytes());
    wav.extend(b"WAVEfmt ");
    wav.extend(16u32.to_le_bytes());
    // PCM, one channel, 16000 Hz, 32000 bytes/s, 2-byte frames, 16 bits.
    wav.extend([1, 0, 1, 0]);
    wav.extend(16_000u32.to_le_bytes());
    wav.extend(32_000u32.to_le_bytes());
    wav.extend([2, 0, 16, 0]);
    wav.extend(b"data");
    wav.extend((pcm.len() as u32).to_le_bytes());
    wav.extend(pcm);
    Ok(wav)
}

/// The data chunk of a 16 kHz mono 16-bit PCM WAV.
pub fn pcm16(path: &Path) -> Result<Vec<u8>, String> {
    let b = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let riff = b.starts_with(b"RIFF") && b.get(8..12) == Some(b"WAVE");
    let (mut at, mut format_ok) = (12, false);
    while riff && at + 8 <= b.len() {
        let len = u32::from_le_bytes([b[at + 4], b[at + 5], b[at + 6], b[at + 7]]) as usize;
        let body = &b[at + 8..(at + 8 + len).min(b.len())];
        match &b[at..at + 4] {
            // PCM, one channel, 16000 Hz, then 16 bits per sample.
            b"fmt " => {
                format_ok = body.get(..8) == Some(&[1, 0, 1, 0, 0x80, 0x3e, 0, 0][..])
                    && body.get(14) == Some(&16)
            }
            b"data" if format_ok => return Ok(body.to_vec()),
            b"data" => break,
            _ => {}
        }
        at += 8 + len + (len & 1);
    }
    Err(format!(
        "{}: need a 16 kHz mono 16-bit PCM WAV",
        path.display()
    ))
}
