//! Q9V / ADR-019 § 8: the repo refuses media. No tracked path may end in a
//! media extension or sit under `fixtures/private/`, apart from the 15
//! public-domain recordings below, allowed by name (founder decision). The
//! `ci.yml` job "no media in the repo" runs the same check with grep and
//! reads its allowlist from `ALLOWED` here.

use std::process::Command;

const MEDIA: [&str; 9] = [
    ".mp4", ".mov", ".m4v", ".mkv", ".webm", ".wav", ".flac", ".mp3", ".m4a",
];

/// A new public-domain recording needs a line here and `git add -f`
/// (the root `.gitignore` ignores every .wav).
const ALLOWED: [&str; 15] = [
    "src-tauri/tests/fixtures/voxxad/amazing-grace-v1.wav",
    "src-tauri/tests/fixtures/voxxad/amazing-grace-v2.wav",
    "src-tauri/tests/fixtures/voxxad/prayer.wav",
    "src-tauri/tests/fixtures/voxxad/tts-be-thou-my-vision-v1-david.wav",
    "src-tauri/tests/fixtures/voxxad/tts-be-thou-my-vision-v3-david.wav",
    "src-tauri/tests/fixtures/voxxad/tts-be-thou-my-vision-v4-david.wav",
    "src-tauri/tests/fixtures/voxxad/tts-come-thou-fount-v1-zira.wav",
    "src-tauri/tests/fixtures/voxxad/tts-come-thou-fount-v2-zira.wav",
    "src-tauri/tests/fixtures/voxxad/tts-holy-holy-holy-v1-david.wav",
    "src-tauri/tests/fixtures/voxxad/tts-holy-holy-holy-v2-david.wav",
    "src-tauri/tests/fixtures/voxxad/tts-it-is-well-refrain-zira.wav",
    "src-tauri/tests/fixtures/voxxad/tts-it-is-well-v1-zira.wav",
    "src-tauri/tests/fixtures/voxxad/tts-it-is-well-v2-zira.wav",
    "src-tauri/tests/fixtures/voxxad/tts-it-is-well-v3-zira.wav",
    "src-tauri/tests/fixtures/voxxad/tts-prayer-zira.wav",
];

/// The refused paths in `git ls-files -z` output. `-z` because git quotes
/// non-ASCII paths otherwise; lower-cased so `X.WAV` is caught too.
fn refused(ls_files_z: &str) -> Vec<&str> {
    (ls_files_z.split('\0'))
        .filter(|path| {
            let lower = path.to_lowercase();
            let media = MEDIA.iter().any(|ext| lower.ends_with(ext));
            (media || lower.starts_with("fixtures/private/")) && !ALLOWED.contains(path)
        })
        .collect()
}

#[test]
fn the_repo_tracks_no_media_and_nothing_under_fixtures_private() {
    // cargo runs tests in src-tauri/, so ".." is the repo root (a scratch
    // clone too, when this binary is run from there).
    let out = Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir("..")
        .output()
        .expect("git must be on PATH");
    assert!(out.status.success(), "git ls-files failed");
    let paths = String::from_utf8_lossy(&out.stdout);
    assert!(
        paths.contains(ALLOWED[0]),
        "not run from the repo's src-tauri/"
    );
    let bad = refused(&paths);
    assert!(
        bad.is_empty(),
        "media or fixtures/private/ is tracked (ADR-019 § 8): {bad:?}"
    );
}

#[test]
fn the_check_catches_case_accents_private_files_and_allowed_names_elsewhere() {
    let mut list = ALLOWED.to_vec();
    list.extend([
        "src/main.js",
        "fixtures/README.md",
        "fixtures/LICENSED_RUNS.md",
        "fixtures/recordings/refrain-it-is-well/audio.json",
        "docs/wave.md",
        "fixtures/privateer.md",
        "fixtures/église/Song A.Mp4",
        "media/X.WAV",
        "fixtures/private/see-a-victory/setlist.json",
        "Fixtures/Private/notes.txt",
        "other/amazing-grace-v1.wav",
        "src-tauri/tests/fixtures/voxxad/PRAYER.WAV",
        "a/b.mov",
        "a/b.m4v",
        "a/b.mkv",
        "a/b.webm",
        "a/b.flac",
        "a/b.mp3",
        "a/b.m4a",
    ]);
    let text = list.join("\0") + "\0";
    assert_eq!(
        refused(&text),
        [
            "fixtures/église/Song A.Mp4",
            "media/X.WAV",
            "fixtures/private/see-a-victory/setlist.json",
            "Fixtures/Private/notes.txt",
            "other/amazing-grace-v1.wav",
            "src-tauri/tests/fixtures/voxxad/PRAYER.WAV",
            "a/b.mov",
            "a/b.m4v",
            "a/b.mkv",
            "a/b.webm",
            "a/b.flac",
            "a/b.mp3",
            "a/b.m4a",
        ]
    );
}
