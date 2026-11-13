//! `voxxad`'s command line: the flags and the recogniser they configure.
//! Uses only `voxxa_lib`, so `tests/voxxad_events.rs` includes this file
//! and tests the wiring without a Whisper model.

use voxxa_lib::transcription::{Decode, Rolling};

#[derive(Default)]
pub struct Args {
    pub setlist: Option<String>,
    pub wav: Option<String>,
    pub realtime: bool,
    pub device: Option<String>,
    pub presenter: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub shadow: bool,
    pub replay: Option<String>,
    pub list_devices: bool,
    pub model: Option<String>,
    pub log_dir: Option<String>,
    pub bias: bool,
    pub help: bool,
}

pub fn parse_args(mut it: impl Iterator<Item = String>) -> Result<Args, String> {
    let mut a = Args::default();
    while let Some(flag) = it.next() {
        let mut value = || it.next().ok_or(format!("{flag} needs a value"));
        match flag.as_str() {
            "--setlist" => a.setlist = Some(value()?),
            "--stdin-audio" => a.wav = Some(value()?),
            "--device" => a.device = Some(value()?),
            "--presenter" => a.presenter = Some(value()?),
            "--host" => a.host = Some(value()?),
            "--port" => a.port = Some(value()?.parse().map_err(|e| format!("--port: {e}"))?),
            "--replay" => a.replay = Some(value()?),
            "--model" => a.model = Some(value()?),
            "--log-dir" => a.log_dir = Some(value()?),
            "--realtime" => a.realtime = true,
            "--shadow" => a.shadow = true,
            "--bias" => a.bias = true,
            "--list-devices" => a.list_devices = true,
            "--help" | "-h" => a.help = true,
            _ => return Err(format!("unknown argument {flag}; see voxxad --help")),
        }
    }
    Ok(a)
}

/// The hop recogniser over `decode`, biased only with `--bias` (ADR-014).
pub fn recognizer<D: Decode>(args: &Args, decode: D) -> Rolling<D> {
    let mut recognizer = Rolling::new(decode);
    recognizer.bias_enabled = args.bias;
    recognizer
}
