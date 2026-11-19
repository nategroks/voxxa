//! Call for help (ADR-018): the words file for the review screen, the zip the
//! operator confirmed, and the e-mail draft that names it. Nothing is sent:
//! no network call; the operator attaches the file and sends the e-mail.

use serde_json::Value;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use time::macros::format_description;
use time::OffsetDateTime;
use zip::write::SimpleFileOptions;

/// The support address. Not chosen yet (Q9 ticket), so the draft has no recipient.
pub const SUPPORT_EMAIL: &str = "";
/// The words file reaches this far back from the end of the session log.
const WORDS_MS: u64 = 120_000;

/// Where the bundle goes: the Desktop, or `VOXXA_HELP_DIR` (the guarded QA build).
pub fn help_dir() -> Result<PathBuf, String> {
    (std::env::var_os("VOXXA_HELP_DIR").map(PathBuf::from))
        .or_else(dirs::desktop_dir)
        .ok_or_else(|| "no Desktop folder".into())
}

/// `voxxad`'s default session-log folder, or `VOXXA_SESSIONS_DIR` (the
/// guarded QA build, so it never reads the tester's own logs).
pub fn sessions_dir() -> Option<PathBuf> {
    (std::env::var_os("VOXXA_SESSIONS_DIR").map(PathBuf::from))
        .or_else(|| Some(dirs::data_dir()?.join("voxxa").join("sessions")))
}

/// The recognised words (`hypothesis` lines) of the newest session log in
/// `dir` from its last 2 minutes, one line each; `None` without any.
pub fn recent_words(dir: &Path) -> Option<String> {
    let newest = (fs::read_dir(dir).ok()?.flatten())
        .filter(|e| e.path().extension().is_some_and(|x| x == "jsonl"))
        .max_by_key(|e| e.metadata().and_then(|m| m.modified()).ok())?;
    let text = fs::read_to_string(newest.path()).ok()?;
    let lines: Vec<Value> = (text.lines().filter_map(|l| serde_json::from_str(l).ok())).collect();
    let end = lines.iter().filter_map(|l| l["t"].as_u64()).max()?;
    let words: Vec<&str> = (lines.iter())
        .filter(|l| {
            l["type"] == "hypothesis" && l["t"].as_u64().is_some_and(|t| t + WORDS_MS >= end)
        })
        .filter_map(|l| l["text"].as_str())
        .collect();
    (!words.is_empty()).then(|| words.join("\n"))
}

/// Writes `files` (name, text), exactly as the operator reviewed them, to a
/// new `voxxa-help-<UTC time>.zip` in `dir`.
pub fn write_bundle(dir: &Path, files: &[(String, String)]) -> Result<PathBuf, String> {
    // The names come from the webview: plain file names only.
    let plain = |n: &str| {
        !n.is_empty()
            && !n.starts_with('.')
            && n.chars()
                .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
    };
    if let Some((bad, _)) = files.iter().find(|(n, _)| !plain(n)) {
        return Err(format!("not a bundle file name: {bad:?}"));
    }
    let stamp = format_description!("[year][month][day]-[hour][minute][second]");
    let stamp = OffsetDateTime::now_utc()
        .format(stamp)
        .map_err(|e| e.to_string())?;
    let path = dir.join(format!("voxxa-help-{stamp}.zip"));
    let err = |e: &dyn std::fmt::Display| format!("{}: {e}", path.display());
    let mut zip = zip::ZipWriter::new(File::create_new(&path).map_err(|e| err(&e))?);
    for (name, text) in files {
        zip.start_file(name.as_str(), SimpleFileOptions::default())
            .map_err(|e| err(&e))?;
        zip.write_all(text.as_bytes()).map_err(|e| err(&e))?;
    }
    zip.finish().map_err(|e| err(&e))?;
    Ok(path)
}

/// The draft: `SUPPORT_EMAIL` (empty for now), a subject, and a body saying
/// which file to attach and where it is. A mailto: link cannot attach it.
pub fn mailto(zip: &Path) -> String {
    let enc = |s: &str| -> String {
        (s.bytes())
            .map(
                |b| match b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
                    true => (b as char).to_string(),
                    false => format!("%{b:02X}"),
                },
            )
            .collect()
    };
    let name = zip.file_name().unwrap_or_default().to_string_lossy();
    let folder = zip.parent().unwrap_or(zip).display();
    let body = format!("Please attach the file {name} from {folder} (that folder has just opened: drag the file into this e-mail).\r\n\r\nWhat happened:\r\n");
    format!(
        "mailto:{SUPPORT_EMAIL}?subject={}&body={}",
        enc("Voxxa: call for help"),
        enc(&body)
    )
}
