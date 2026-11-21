//! Q9V / ADR-019 § 8: the repository refuses media. Every tracked path is
//! read with `git ls-files -z` (without `-z`, git quotes a non-ASCII path and
//! the check would miss it), lower-cased, and refused when it ends in an audio
//! or video extension or sits under `fixtures/private/`. Only the 15
//! public-domain recordings tracked before the rule are allowed, by their
//! exact path (founder decision, 2026-11-20). The "media guard" job in
//! `.github/workflows/ci.yml` runs the same check in bash, so a PR that adds
//! media goes red even if this test is skipped. Needs `git` on PATH.

use std::path::Path;
use std::process::Command;

/// The refused extensions, as in the root `.gitignore`.
const MEDIA: [&str; 9] = [
    ".mp4", ".mov", ".m4v", ".mkv", ".webm", ".wav", ".flac", ".mp3", ".m4a",
];

/// Exactly the tracked public-domain recordings, by path. The same list is in
/// ci.yml's media guard; a test below keeps the two in step.
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

/// The paths the repository must refuse, in the order given: under
/// `fixtures/private/` (at the root or deeper), or a media extension on any
/// path that is not exactly one of [`ALLOWED`]. Matching is on the
/// lower-cased path, so `X.WAV` and `Fixtures/Private/` are caught; the
/// allowlist compares the path as tracked, so `PRAYER.WAV` or `prayer.wav`
/// in another folder is refused.
fn refused<'a>(paths: impl IntoIterator<Item = &'a str>) -> Vec<&'a str> {
    let refuse = |path: &&str| {
        let lower = path.to_lowercase();
        let private = lower.starts_with("fixtures/private/") || lower.contains("/fixtures/private/");
        let media = MEDIA.iter().any(|ext| lower.ends_with(ext));
        private || (media && !ALLOWED.contains(path))
    };
    paths.into_iter().filter(refuse).collect()
}

fn tracked() -> Vec<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let out = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["ls-files", "-z"])
        .output()
        .expect("the media guard runs `git ls-files -z`: git must be on PATH");
    assert!(
        out.status.success(),
        "git ls-files failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    (String::from_utf8_lossy(&out.stdout).split('\0'))
        .filter(|p| !p.is_empty())
        .map(String::from)
        .collect()
}

#[test]
fn no_tracked_path_is_media_or_under_fixtures_private() {
    let paths = tracked();
    assert!(paths.len() > 100, "git listed only {} paths", paths.len());
    let bad = refused(paths.iter().map(String::as_str));
    assert!(
        bad.is_empty(),
        "tracked media or private fixture files (ADR-019 § 8; untrack with `git rm --cached`): {bad:?}"
    );
    // "Exactly the 15": an allowlisted recording that is gone must leave the
    // list (here and in ci.yml), so the list never allows more than is there.
    for path in ALLOWED {
        assert!(
            paths.iter().any(|p| p == path),
            "{path} is allowed but no longer tracked: remove it from ALLOWED and ci.yml"
        );
    }
}

#[test]
fn the_check_refuses_media_anywhere_and_private_files_and_allows_only_the_15() {
    let fine = [
        "src/main.js",
        "fixtures/README.md",
        "fixtures/LICENSED_RUNS.md",
        "fixtures/recordings/prayer-then-reprise/audio.json",
        "fixtures/private-notes.md",
        "docs/audio.wav.md",
        "src-tauri/tests/fixtures/voxxad/prayer.wav",
        "src-tauri/tests/fixtures/voxxad/tts-prayer-zira.wav",
    ];
    let bad = [
        // An accented folder name (git ls-files without -z would quote it).
        "Músicas de adoración/canción.wav",
        "fixtures/recordings/café/audio.wav",
        // Upper- and mixed-case extensions.
        "X.WAV",
        "clips/Song A.Mp4",
        "media/Song A.MOV",
        "a.m4v",
        "a.mkv",
        "a.webm",
        "a.flac",
        "a.mp3",
        "a.m4a",
        // Anything under fixtures/private/, media or not, in any case.
        "fixtures/private/see-a-victory/setlist.json",
        "fixtures/private/see-a-victory/truth.jsonl",
        "fixtures/private/see-a-victory/audio.wav",
        "Fixtures/Private/notes.txt",
        "src-tauri/fixtures/private/a.json",
        // An allowlisted name in another folder or another case.
        "fixtures/recordings/prayer-then-reprise/prayer.wav",
        "prayer.wav",
        "src-tauri/tests/fixtures/voxxad/PRAYER.WAV",
        "src-tauri/tests/fixtures/voxxad/sub/prayer.wav",
        // A new recording beside the 15 needs its own allowlist entry.
        "src-tauri/tests/fixtures/voxxad/new-hymn.wav",
    ];
    assert_eq!(refused(fine), Vec::<&str>::new());
    assert_eq!(refused(bad), bad);
    assert_eq!(refused(ALLOWED), Vec::<&str>::new());
    let mixed = ["src/main.js", "X.WAV", "fixtures/README.md"];
    assert_eq!(refused(mixed), ["X.WAV"], "the refused path is named");
}

#[test]
fn ci_runs_the_same_check_with_the_same_list() {
    let ci = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../.github/workflows/ci.yml"),
    )
    .unwrap();
    // The `media` job, up to the next line indented less than its body
    // (`lines()` also drops the \r of a CRLF checkout).
    let job = (ci.lines())
        .skip_while(|l| *l != "  media:")
        .skip(1)
        .take_while(|l| l.is_empty() || l.starts_with("    "))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(job.contains("runs-on:"), "ci.yml has no `media` job");
    assert!(job.contains("git ls-files -z"), "{job}");
    assert!(job.contains("lower=${path,,}"), "{job}");
    assert!(job.contains("fixtures/private/*|*/fixtures/private/*"), "{job}");
    for ext in MEDIA {
        assert!(job.contains(&format!("*{ext}")), "{ext} missing: {job}");
    }
    let listed: Vec<&str> = (job.split_whitespace())
        .filter(|w| w.starts_with("src-tauri/tests/fixtures/voxxad/"))
        .collect();
    assert_eq!(listed, ALLOWED, "ci.yml's allowlist differs from ALLOWED");
}
