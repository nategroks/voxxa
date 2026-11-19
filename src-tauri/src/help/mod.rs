//! Call for help (ADR-018): a bundle the volunteer reviews first, then saves
//! as one zip on the Desktop and attaches to an e-mail draft. Nothing is
//! written before the user confirms, and nothing goes over the network.

use crate::AppState;
use serde_json::{Map, Value};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};
use tauri::{AppHandle, State};
use time::{macros::format_description, OffsetDateTime};
use zip::{result::ZipResult, write::SimpleFileOptions, CompressionMethod::Deflated};

/// The support address. Empty until the project has one (ticket Q9-01): the
/// draft then opens with no recipient.
pub const SUPPORT_EMAIL: &str = "";
/// The runbooks a link next to the status light may open (ADR-018 § 7),
/// each `docs/runbooks/<name>.md`.
pub const RUNBOOKS: &str = "no-audio-device presentation-app-not-reachable model-missing";
pub const WORDS_FILE: &str = "words-last-2-minutes.txt";
const WORDS_MS: u64 = 120_000;
/// QA override (ADR-018 § 6): write the zip here, read session logs from
/// `<dir>/sessions`, and open nothing.
pub const QA_DIR: &str = "VOXXA_HELP_DIR";

/// The webview's `localStorage`, key by key.
pub type Storage = Map<String, Value>;
/// One file of the bundle: (name, content).
pub type BundleFile = (String, String);

/// The bundle on the review screen; Save writes exactly this.
static PENDING: Mutex<Option<Vec<BundleFile>>> = Mutex::new(None);
/// The zip this process wrote last: the only file the e-mail step names.
static WRITTEN: Mutex<Option<PathBuf>> = Mutex::new(None);

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

const SECRET_IN: &str = "token secret pass pwd credential authoriz appid";
const SECRET_END: &str = "key auth pat";

/// A property name that reads as a secret (ADR-018 § 4), compared
/// lower-cased with everything but letters and digits removed.
pub fn is_secret(name: &str) -> bool {
    let n: String = name.chars().filter(char::is_ascii_alphanumeric).collect();
    let n = n.to_ascii_lowercase();
    SECRET_IN.split(' ').any(|s| n.contains(s)) || SECRET_END.split(' ').any(|s| n.ends_with(s))
}

/// Replace the value of every property named like a secret, at any depth.
pub fn redact(v: &mut Value) {
    match v {
        Value::Object(m) => {
            for (k, x) in m.iter_mut() {
                match is_secret(k) {
                    true => *x = "[redacted]".into(),
                    false => redact(x),
                }
            }
        }
        Value::Array(a) => a.iter_mut().for_each(redact),
        _ => {}
    }
}

/// The operator's `localStorage` as exported: `voxxa.*` keys, values parsed
/// as JSON, whole setlists left out, secrets redacted.
pub fn operator_settings(storage: &Storage) -> Value {
    let mut out = Map::new();
    for (k, v) in storage.iter().filter(|(k, _)| k.starts_with("voxxa.")) {
        let v = match v {
            _ if k == "voxxa.recentSetlists" => "[left out: whole setlists]".into(),
            Value::String(s) => serde_json::from_str(s).unwrap_or_else(|_| v.clone()),
            _ => v.clone(),
        };
        out.insert(k.clone(), v);
    }
    let mut out = Value::Object(out);
    redact(&mut out);
    out
}

/// `session-YYYYMMDD-HHMM[-N].jsonl` in order: a longer `-N` is a later one.
fn session_order(name: &str) -> Option<(&str, usize, &str)> {
    let stem = name.strip_prefix("session-")?.strip_suffix(".jsonl")?;
    Some((stem.get(..13)?, stem.len(), stem))
}

/// The words file: the `hypothesis` lines of the newest session log in `dir`
/// whose `t` is within the last 2 minutes of that log's end. None without a log.
pub fn recent_words(dir: &Path) -> Option<String> {
    let names = std::fs::read_dir(dir).ok()?.flatten();
    let names = names.filter_map(|e| e.file_name().into_string().ok());
    let name = names.max_by(|a, b| session_order(a).cmp(&session_order(b)))?;
    session_order(&name)?;
    let text = std::fs::read_to_string(dir.join(&name)).ok()?;
    let lines: Vec<Value> = text.lines().flat_map(serde_json::from_str).collect();
    let end = lines.iter().filter_map(|l| l["t"].as_u64()).max();
    let words: String = (lines.iter().filter(|l| l["type"] == "hypothesis"))
        .filter_map(|l| Some((l["t"].as_u64()?, l["text"].as_str()?)))
        .filter(|(t, _)| *t >= end.unwrap_or(0).saturating_sub(WORDS_MS))
        .map(|(t, w)| format!("{:.1} s  {w}\n", t as f64 / 1000.0))
        .collect();
    let none = "(no words in that time)\n".to_string();
    let words = Some(words).filter(|w| !w.is_empty()).unwrap_or(none);
    let head = format!("Words Voxxa heard in the last 2 minutes of {name}");
    Some(format!("{head}, in seconds from its start.\n\n{words}"))
}

/// The bundle's files, in memory: the report (JSON), the operator settings
/// and, when there is a session log in `logs`, the words file.
pub fn bundle_files(report: String, storage: &Storage, logs: Option<&Path>) -> Vec<BundleFile> {
    let settings = serde_json::to_string_pretty(&operator_settings(storage));
    let mut files = vec![("diagnostic-report.json".into(), report)];
    files.push((
        "operator-settings.json".into(),
        settings.unwrap_or_default(),
    ));
    files.extend(logs.and_then(recent_words).map(|w| (WORDS_FILE.into(), w)));
    files
}

/// Write `files` as a new zip at `path`, without the words file unless
/// `words`. Never overwrites; a failed write leaves no file.
pub fn write_zip(path: &Path, files: &[BundleFile], words: bool) -> Result<(), String> {
    let file = std::fs::File::create_new(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = SimpleFileOptions::default().compression_method(Deflated);
    let written = (|| -> ZipResult<()> {
        for (name, content) in files.iter().filter(|f| words || f.0 != WORDS_FILE) {
            zip.start_file(name, opts)?;
            zip.write_all(content.as_bytes())?;
        }
        zip.finish().map(drop)
    })();
    written.map_err(|e| {
        let _ = std::fs::remove_file(path);
        e.to_string()
    })
}

/// Percent-encode all but RFC 3986's unreserved characters.
fn pct(s: &str) -> String {
    let keep = |b: u8| b.is_ascii_alphanumeric() || b"-._~".contains(&b);
    let enc = |b: u8| (b as char).to_string();
    (s.bytes())
        .map(|b| if keep(b) { enc(b) } else { format!("%{b:02X}") })
        .collect()
}

/// What the e-mail step opens: the draft (a `mailto:` link cannot attach a
/// file, so the body says which one), then the zip's folder.
pub fn email_targets(zip: &Path) -> [String; 2] {
    let name = zip.file_name().unwrap_or_default().to_string_lossy();
    let body = pct(&format!("Hello,\r\n\r\nVoxxa needs help. Please attach this file to this e-mail before you send it:\r\n\r\n{}\r\n\r\nIts folder has just opened: drag {name} into this e-mail.\r\n\r\nWhat happened:\r\n\r\n", zip.display()));
    let subject = pct("Voxxa: call for help");
    let mailto = format!("mailto:{SUPPORT_EMAIL}?subject={subject}&body={body}");
    [mailto, zip.parent().unwrap_or(zip).display().to_string()]
}

fn qa_dir() -> Option<PathBuf> {
    std::env::var_os(QA_DIR).map(PathBuf::from)
}

#[allow(deprecated)] // the shell plugin's open: no new crate, no webview capability (ADR-018)
fn open(app: &AppHandle, target: &str) -> Result<(), String> {
    use tauri_plugin_shell::ShellExt;
    app.shell().open(target, None).map_err(|e| e.to_string())
}

/// Build the bundle in memory for the review screen; writes nothing.
#[tauri::command]
pub async fn help_bundle_preview(
    state: State<'_, AppState>,
    storage: Storage,
) -> Result<Vec<BundleFile>, String> {
    let report = crate::diagnostics::generate_report(&state).await;
    let report = serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?;
    let data = dirs::data_dir().map(|d| d.join("voxxa"));
    let logs = qa_dir().or(data).map(|d| d.join("sessions"));
    let files = bundle_files(report, &storage, logs.as_deref());
    *lock(&PENDING) = Some(files.clone());
    Ok(files)
}

/// Save the reviewed bundle as `voxxa-help-<UTC time>.zip`; returns its path.
#[tauri::command]
pub async fn write_help_bundle(include_words: bool) -> Result<String, String> {
    let mut pending = lock(&PENDING);
    let files = pending.as_ref().ok_or("Press Call for help again")?;
    let dir = qa_dir().or_else(dirs::desktop_dir);
    let dir = dir.ok_or("No Desktop folder")?;
    let stamp = format_description!("[year][month][day]-[hour][minute][second]");
    let stamp = OffsetDateTime::now_utc().format(stamp).unwrap_or_default();
    let path = dir.join(format!("voxxa-help-{stamp}.zip"));
    write_zip(&path, files, include_words)?;
    *pending = None;
    *lock(&WRITTEN) = Some(path.clone());
    Ok(path.display().to_string())
}

/// Open the e-mail draft and the saved zip's folder (nothing under the QA
/// override); returns the two targets.
#[tauri::command]
pub fn open_help_email(app: AppHandle) -> Result<Vec<String>, String> {
    let zip = lock(&WRITTEN).clone().ok_or("No help file saved yet")?;
    let targets = email_targets(&zip);
    if qa_dir().is_none() {
        targets.iter().try_for_each(|t| open(&app, t))?;
    }
    Ok(targets.to_vec())
}

/// The GitHub page of one of the [`RUNBOOKS`]; None for any other name.
pub fn runbook_url(name: &str) -> Option<String> {
    let base = "https://github.com/nategroks/voxxa/blob/main/docs/runbooks";
    (RUNBOOKS.split(' ').any(|r| r == name)).then(|| format!("{base}/{name}.md"))
}

/// Open one of the [`RUNBOOKS`] on GitHub; any other name is refused.
#[tauri::command]
pub fn open_runbook(app: AppHandle, name: String) -> Result<(), String> {
    let url = runbook_url(&name).ok_or(format!("no runbook named {name}"))?;
    open(&app, &url)
}
