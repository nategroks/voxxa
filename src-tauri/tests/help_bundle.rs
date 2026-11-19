//! Q9 / ADR-018: Call for help. #102 (no recognised text reaches the
//! diagnostics buffer), the bundle's files, the words window, secret
//! redaction, the zip, the e-mail draft and the runbook links. No window,
//! microphone, network or Whisper model; placeholder words only.

use serde_json::{json, Value};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use voxxa_lib::help::{self, Storage, WORDS_FILE};
use voxxa_lib::listen;
use voxxa_lib::transcription::{Hypothesis, Recognizer};
use voxxa_lib::{Conductor, SmartConfig, Song, VadEngine};

/// The repository root (the tests run in `src-tauri/`).
fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .into()
}

/// `localStorage` as the GUI passes it: every value a string.
fn storage(pairs: &[(&str, Value)]) -> Storage {
    let raw = |v: &Value| Value::String(v.to_string());
    pairs.iter().map(|(k, v)| (k.to_string(), raw(v))).collect()
}

/// A session log line.
fn line(t: u64, kind: &str, text: &str) -> String {
    let line = match kind {
        "hypothesis" | "window" => json!({ "v": 1, "t": t, "type": kind, "text": text }),
        _ => json!({ "v": 1, "t": t, "type": kind }),
    };
    format!("{line}\n")
}

fn write_log(dir: &Path, name: &str, lines: &[String]) {
    std::fs::write(dir.join(name), lines.concat()).unwrap();
}

/// Every entry of a zip, by name.
fn unzip(path: &Path) -> Vec<(String, String)> {
    let mut zip = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
    let mut out = Vec::new();
    for i in 0..zip.len() {
        let mut f = zip.by_index(i).unwrap();
        let mut text = String::new();
        f.read_to_string(&mut text).unwrap();
        out.push((f.name().unwrap().to_string(), text));
    }
    out
}

/// Decode a `%XX` URL component.
fn unpct(s: &str) -> String {
    let b = s.as_bytes();
    let (mut out, mut i) = (Vec::new(), 0);
    while i < b.len() {
        if b[i] == b'%' {
            out.push(u8::from_str_radix(&s[i + 1..i + 3], 16).unwrap());
            i += 3;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    String::from_utf8(out).unwrap()
}

/// Hears four placeholder words on its first hop, then nothing.
struct Hears(Option<Vec<&'static str>>);

impl Recognizer for Hears {
    fn feed(&mut self, _: &[f32]) -> Vec<Hypothesis> {
        let Some(words) = self.0.take() else {
            return Vec::new();
        };
        let words = (words.iter().enumerate())
            .map(|(i, w)| (w.to_string(), i as f32 * 0.3, 0.9))
            .collect();
        vec![Hypothesis {
            words,
            end: 1.0,
            error: None,
        }]
    }
    fn bias(&mut self, _: &str) {}
}

/// #102 (Q7-08): the listening loop logs how many words it heard, never the
/// words, so neither the ring buffer nor `generate_diagnostic_report`'s
/// `recent_log` (the same buffer) carries a transcript.
#[test]
fn no_recognised_text_reaches_the_diagnostics_buffer() {
    voxxa_lib::init_logger();
    let words = vec!["zulu", "yankee", "xray", "whiskey"];
    let song: Song = serde_json::from_value(serde_json::json!({
        "title": "Alpha Hymn",
        "slides": [{ "id": 0, "text": "alpha bravo charlie delta" }]
    }))
    .unwrap();
    let conductor = Mutex::new(Some(Conductor::new(vec![song], SmartConfig::default())));
    let vad = Mutex::new(VadEngine::new());
    let (tx, rx) = mpsc::channel();
    tx.send(vec![0.0f32; 1600]).unwrap();
    drop(tx);
    let base = Instant::now();
    let clock = |n: u64| base + Duration::from_millis(n / 16);
    let mut heard = Vec::new();
    let sink = |_, e| {
        if let listen::Event::Input(listen::Input::Heard(t)) = e {
            heard.push(t)
        }
    };
    let run = AtomicBool::new(true);
    listen::run(
        rx,
        &run,
        &vad,
        &conductor,
        &mut Hears(Some(words.clone())),
        clock,
        sink,
    );
    assert_eq!(heard, ["zulu yankee xray whiskey"], "the loop heard them");
    let log = voxxa_lib::recent_log().join("\n");
    assert!(log.contains("Heard 4 words"), "{log}");
    for w in words {
        assert!(!log.contains(w), "the buffer holds a heard word");
    }
}

/// Presenter passwords, the HTTP API token, Planning Center credentials and
/// anything named like token/secret/password/key are redacted, nested or at
/// the top level; whole setlists are left out; the rest stays readable.
#[test]
fn the_operator_settings_export_holds_no_secret() {
    let secret = "s3cr3t-placeholder";
    let st = storage(&[
        (
            "voxxa.presenterForm",
            json!({ "kind": "open_lp_v2", "host": "127.0.0.1", "port": 4316, "username": "booth", "password": secret, "keystroke_profile": "universal" }),
        ),
        (
            "voxxa.httpApi",
            json!({ "port": 7575, "enabled": true, "token": secret, "http_api_token": secret }),
        ),
        (
            "voxxa.pco",
            json!({ "appId": secret, "pco_app_id": secret, "secret": secret, "credentials": { "user": secret } }),
        ),
        (
            "voxxa.opensong",
            json!({ "key": secret, "apiKey": secret, "API_KEY": secret, "Authorization": secret, "auth": secret }),
        ),
        (
            "voxxa.nested",
            json!([{ "deep": { "pwd": secret, "passphrase": secret, "client_secret": secret, "pat": secret } }]),
        ),
        ("voxxa.pcoSecret", json!(secret)),
        ("voxxa.apiToken", json!(secret)),
        (
            "voxxa.smartConfig",
            serde_json::to_value(SmartConfig::default()).unwrap(),
        ),
        (
            "voxxa.recentSetlists",
            json!([{ "summary": "Alpha Hymn", "setlist": [{ "title": "Alpha Hymn", "slides": [{ "id": 0, "text": "alpha bravo charlie" }] }] }]),
        ),
        ("voxxa.language", json!("en")),
        ("other.app", json!({ "note": "not from Voxxa" })),
    ]);
    let out = help::operator_settings(&st);
    let text = out.to_string();
    assert!(!text.contains(secret), "a secret survived: {text}");
    assert!(!text.contains("alpha bravo"), "slide text in the export");
    assert!(!text.contains("not from Voxxa"), "only voxxa.* keys");
    assert_eq!(out["voxxa.presenterForm"]["password"], "[redacted]");
    assert_eq!(out["voxxa.pcoSecret"], "[redacted]");
    assert_eq!(out["voxxa.recentSetlists"], "[left out: whole setlists]");
    // What support needs is still there, and nothing harmless is redacted.
    let form = &out["voxxa.presenterForm"];
    assert_eq!(form["kind"], "open_lp_v2");
    assert_eq!(form["port"], 4316);
    assert_eq!(form["username"], "booth");
    assert_eq!(form["keystroke_profile"], "universal");
    assert_eq!(out["voxxa.httpApi"]["port"], 7575);
    assert_eq!(out["voxxa.language"], "en");
    let smart = serde_json::to_value(SmartConfig::default()).unwrap();
    assert_eq!(
        out["voxxa.smartConfig"], smart,
        "a SmartConfig name reads as a secret"
    );
    // The names the GUI stores today are none of them secret.
    for k in [
        "firstLaunchDone",
        "audioDevice",
        "presenterForm",
        "lastModel",
        "smartConfig",
        "language",
        "httpApi",
    ] {
        assert!(!help::is_secret(&format!("voxxa.{k}")), "{k}");
    }
    for k in [
        "host",
        "port",
        "kind",
        "username",
        "keystroke_profile",
        "enabled",
        "title",
    ] {
        assert!(!help::is_secret(k), "{k}");
    }
}

/// A value that is not JSON is kept as a string, and still redacted by name.
#[test]
fn non_json_values_are_kept_as_strings() {
    let mut st = Storage::new();
    st.insert("voxxa.audioDevice".into(), "Desk USB input".into());
    st.insert("voxxa.httpApiToken".into(), "not json either".into());
    let out = help::operator_settings(&st);
    assert_eq!(out["voxxa.audioDevice"], "Desk USB input");
    assert_eq!(out["voxxa.httpApiToken"], "[redacted]");
}

/// The words file: the newest session log by name (`-10` after `-2`), only
/// its `hypothesis` lines, only those within 120,000 ms of the log's end.
#[test]
fn the_words_file_holds_the_last_two_minutes_of_the_newest_log() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    assert_eq!(help::recent_words(d), None, "no log, no words file");
    write_log(
        d,
        "session-20261119-0900.jsonl",
        &[line(0, "hypothesis", "alpha old")],
    );
    write_log(
        d,
        "session-20261119-1000-2.jsonl",
        &[line(0, "hypothesis", "bravo second")],
    );
    write_log(
        d,
        "session-20261119-1000.jsonl",
        &[line(0, "hypothesis", "charlie first")],
    );
    write_log(d, "notes-zz.jsonl", &[line(0, "hypothesis", "delta stray")]);
    write_log(
        d,
        "session-20261119-1000-10.jsonl",
        &[
            line(0, "status", ""),
            line(1_000, "hypothesis", "echo early"),
            line(59_999, "hypothesis", "foxtrot cut"),
            line(60_000, "hypothesis", "golf kept"),
            line(61_000, "window", "hotel window"),
            line(150_000, "hypothesis", "india kept"),
            line(180_000, "tick", ""),
        ],
    );
    let words = help::recent_words(d).unwrap();
    assert!(words.contains("session-20261119-1000-10.jsonl"), "{words}");
    assert!(words.contains("60.0 s  golf kept\n"), "{words}");
    assert!(words.contains("150.0 s  india kept\n"), "{words}");
    for gone in [
        "echo", "foxtrot", "hotel", "alpha", "bravo", "charlie", "delta",
    ] {
        assert!(!words.contains(gone), "{gone} in {words}");
    }
    // A log with no words in its last two minutes still gives the file.
    write_log(
        d,
        "session-20261119-1100.jsonl",
        &[line(0, "status", ""), line(500, "tick", "")],
    );
    assert!(help::recent_words(d)
        .unwrap()
        .contains("(no words in that time)"));
}

/// The preview is built in memory: the report, the settings and (with a log)
/// the words file; nothing is written anywhere.
#[test]
fn the_preview_lists_every_file_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let logs = dir.path().join("sessions");
    std::fs::create_dir(&logs).unwrap();
    let st = storage(&[("voxxa.language", json!("en"))]);
    let names = |files: &[help::BundleFile]| files.iter().map(|f| f.0.clone()).collect::<Vec<_>>();
    let files = help::bundle_files("{}".into(), &st, Some(&logs));
    assert_eq!(
        names(&files),
        ["diagnostic-report.json", "operator-settings.json"]
    );
    write_log(
        &logs,
        "session-20261119-1000.jsonl",
        &[line(10, "hypothesis", "kilo lima")],
    );
    let before = std::fs::read_dir(dir.path()).unwrap().count();
    let files = help::bundle_files("{\"report\": 1}".into(), &st, Some(&logs));
    assert_eq!(
        names(&files),
        [
            "diagnostic-report.json",
            "operator-settings.json",
            WORDS_FILE
        ]
    );
    assert_eq!(files[0].1, "{\"report\": 1}");
    assert!(files[2].1.contains("kilo lima"));
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), before);
    assert_eq!(std::fs::read_dir(&logs).unwrap().count(), 1);
    assert_eq!(names(&help::bundle_files("{}".into(), &st, None)).len(), 2);
}

/// Checked, the zip holds the words file; unchecked, the file is absent
/// (not empty). The zip holds exactly what was shown, and never overwrites.
#[test]
fn the_zip_holds_what_was_shown_and_the_words_only_when_checked() {
    let dir = tempfile::tempdir().unwrap();
    let files: Vec<help::BundleFile> = vec![
        ("diagnostic-report.json".into(), "{\"a\": 1}".into()),
        ("operator-settings.json".into(), "{}".into()),
        (WORDS_FILE.into(), "mike november\n".into()),
    ];
    let with = dir.path().join("with.zip");
    help::write_zip(&with, &files, true).unwrap();
    assert_eq!(unzip(&with), files);
    let without = dir.path().join("without.zip");
    help::write_zip(&without, &files, false).unwrap();
    let entries = unzip(&without);
    assert_eq!(entries, files[..2]);
    assert!(entries
        .iter()
        .all(|(n, c)| n != WORDS_FILE && !c.contains("mike")));
    // An existing file is never overwritten.
    let err = help::write_zip(&with, &files[..1], false).unwrap_err();
    assert!(err.contains("with.zip"), "{err}");
    assert_eq!(unzip(&with).len(), 3);
    // A folder that does not exist: an error and no file.
    let nowhere = dir.path().join("missing").join("x.zip");
    assert!(help::write_zip(&nowhere, &files, true).is_err());
    assert!(!nowhere.exists());
}

/// The draft has no recipient yet (ticket Q9-01), a subject, and a body that
/// names the file to attach and where it is; the second target is its folder.
#[test]
fn the_email_draft_names_the_file_and_has_no_recipient() {
    assert_eq!(
        help::SUPPORT_EMAIL,
        "",
        "the support address is not decided yet"
    );
    let zip = Path::new("C:/Users/Booth PC/Desktop/voxxa-help-20261119-101500.zip");
    let [mailto, folder] = help::email_targets(zip);
    let rest = mailto.strip_prefix("mailto:?subject=").expect(&mailto);
    let (subject, body) = rest.split_once("&body=").unwrap();
    assert_eq!(unpct(subject), "Voxxa: call for help");
    let body = unpct(body);
    assert!(body.contains("attach this file"), "{body}");
    assert!(body.contains(&zip.display().to_string()), "{body}");
    assert!(
        body.contains("drag voxxa-help-20261119-101500.zip into this e-mail"),
        "{body}"
    );
    let unreserved = |c: char| c.is_ascii_alphanumeric() || "-._~%".contains(c);
    assert!(
        rest.replace("&body=", "").chars().all(unreserved),
        "{mailto}"
    );
    assert_eq!(Path::new(&folder), zip.parent().unwrap());
}

/// "Links resolve": every runbook the backend opens and every runbook the
/// status light links to is a file in the repository, and every relative
/// link inside the Q9 docs points to a file that exists.
#[test]
fn every_runbook_link_resolves() {
    let runbooks = repo().join("docs/runbooks");
    for name in help::RUNBOOKS.split(' ') {
        assert!(
            runbooks.join(format!("{name}.md")).is_file(),
            "docs/runbooks/{name}.md"
        );
    }
    let logic = std::fs::read_to_string(repo().join("src/simple/logic.js")).unwrap();
    // `export const RUNBOOKS = { mic: "…", app: "…", model: "…" };`
    let table = logic.split("export const RUNBOOKS = {").nth(1).unwrap();
    let table = table.split('}').next().unwrap();
    let linked: Vec<&str> = table.split('"').skip(1).step_by(2).collect();
    assert_eq!(linked.len(), 3, "statusLight links {linked:?}");
    for name in linked {
        assert!(
            help::RUNBOOKS.split(' ').any(|r| r == name),
            "{name} is not a runbook the backend opens"
        );
    }
    let mut docs: Vec<PathBuf> = (help::RUNBOOKS.split(' '))
        .map(|n| runbooks.join(format!("{n}.md")))
        .collect();
    docs.push(repo().join("docs/quick-start.md"));
    for doc in docs {
        let text = std::fs::read_to_string(&doc).unwrap();
        for link in text
            .split("](")
            .skip(1)
            .map(|s| s.split(')').next().unwrap())
        {
            if link.starts_with("http") || link.starts_with('#') || link.starts_with("mailto:") {
                continue;
            }
            let target = doc.parent().unwrap().join(link.split('#').next().unwrap());
            assert!(
                target.exists(),
                "{}: {link} does not resolve",
                doc.display()
            );
        }
    }
}
