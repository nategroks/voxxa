//! Q9 / ADR-018, verifier survivors M2b and M5b: the Call for help steps the
//! GUI calls, run for real: the preview writes nothing, and Save with the
//! box unchecked leaves the words file out of the zip on disk. A process of
//! its own, because it sets the QA override (`VOXXA_HELP_DIR`) to a temp
//! folder. No window, microphone or network; placeholder words only.

use serde_json::json;
use std::io::Read;
use std::path::{Path, PathBuf};
use voxxa_lib::help::{self, Storage, WORDS_FILE};

/// The names in a folder, sorted.
fn entries(dir: &Path) -> Vec<String> {
    let names = std::fs::read_dir(dir).unwrap().flatten();
    let mut names: Vec<String> = names
        .map(|e| e.file_name().into_string().unwrap())
        .collect();
    names.sort();
    names
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

/// The Save command itself, as the GUI invokes it.
fn save(include_words: bool) -> Result<String, String> {
    tauri::async_runtime::block_on(help::write_help_bundle(include_words))
}

#[test]
fn the_preview_writes_nothing_and_unchecked_means_no_words_file() {
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var(help::QA_DIR, dir.path());
    let logs = dir.path().join("sessions");
    std::fs::create_dir(&logs).unwrap();
    let log = "session-20261119-1000.jsonl";
    let line = json!({ "v": 1, "t": 10, "type": "hypothesis", "text": "alpha bravo" });
    std::fs::write(logs.join(log), format!("{line}\n")).unwrap();
    let mut st = Storage::new();
    st.insert("voxxa.language".into(), "\"en\"".into());
    assert_eq!(save(true).unwrap_err(), "Press Call for help again");
    assert_eq!(
        entries(dir.path()),
        ["sessions"],
        "Save wrote before a review"
    );

    // The review: every file in memory, the words file among them; nothing on disk.
    let files = help::preview(&json!({ "report": 1 }), &st, Some(dir.path())).unwrap();
    let names: Vec<&str> = files.iter().map(|f| f.0.as_str()).collect();
    let all = [
        "diagnostic-report.json",
        "operator-settings.json",
        WORDS_FILE,
    ];
    assert_eq!(names, all);
    assert!(files[2].1.contains("alpha bravo"));
    assert_eq!(
        entries(dir.path()),
        ["sessions"],
        "the preview wrote a file"
    );
    assert_eq!(entries(&logs), [log], "the preview wrote beside the log");

    // Save with the box unchecked: the zip on disk has no words file at all.
    let without = PathBuf::from(save(false).unwrap());
    assert_eq!(without.parent(), Some(dir.path()));
    let zipped = unzip(&without);
    assert_eq!(zipped, files[..2], "unchecked, yet the words were written");
    assert!(zipped.iter().all(|(_, c)| !c.contains("alpha bravo")));
    // One Save per review: a second one writes nothing.
    assert_eq!(save(true).unwrap_err(), "Press Call for help again");
    assert_eq!(entries(dir.path()).len(), 2);

    // A new review with the box checked: the words file is in the zip.
    std::thread::sleep(std::time::Duration::from_millis(1100)); // a new file name each second
    help::preview(&json!({ "report": 2 }), &st, Some(dir.path())).unwrap();
    assert_eq!(entries(dir.path()).len(), 2, "the preview wrote a file");
    let with = PathBuf::from(save(true).unwrap());
    let names: Vec<String> = unzip(&with).into_iter().map(|f| f.0).collect();
    assert_eq!(names, all);
}
