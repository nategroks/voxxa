//! Q9V / ADR-019 § 8: the repository refuses media. Every tracked path is
//! read with `git ls-files -z` (without `-z`, git quotes a non-ASCII path and
//! the check would miss it), lower-cased, and refused when it ends in an audio
//! or video extension or sits under `fixtures/private/`. Only the 15
//! public-domain recordings tracked before the rule are allowed, by their
//! exact path (founder decision, 2026-11-20). The "media guard" job in
//! `.github/workflows/ci.yml` runs the same check in bash, so a PR that adds
//! media goes red even if this test is skipped; a test below runs that bash
//! step on a planted tree. Needs `git` on PATH (and bash 4 for that test).

use std::env::consts::EXE_SUFFIX;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

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
        let private =
            lower.starts_with("fixtures/private/") || lower.contains("/fixtures/private/");
        let media = MEDIA.iter().any(|ext| lower.ends_with(ext));
        private || (media && !ALLOWED.contains(path))
    };
    paths.into_iter().filter(refuse).collect()
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// Runs `git` in `dir` with `input` on stdin and returns its output; panics
/// if it fails.
fn git(dir: &Path, args: &[&str], input: &str) -> Output {
    let mut child = (Command::new("git").arg("-C").arg(dir).args(args))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the media guard runs git: git must be on PATH");
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(input.as_bytes()).unwrap();
    drop(stdin);
    let out = child.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    out
}

/// Every path tracked in the repository at `root`, as written.
fn tracked(root: &Path) -> Vec<String> {
    let out = git(root, &["ls-files", "-z"], "");
    (String::from_utf8_lossy(&out.stdout).split('\0'))
        .filter(|p| !p.is_empty())
        .map(String::from)
        .collect()
}

/// A temporary git repository with an empty file staged at each path. Only
/// the index holds them (`update-index --index-info`), so nothing named like
/// media is written to disk, and `X.WAV` beside `x.wav` works on a
/// case-insensitive file system.
fn repo_with(paths: &[&str]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q"], "");
    let blob = git(dir.path(), &["hash-object", "-w", "--stdin"], "").stdout;
    let blob = String::from_utf8(blob).unwrap();
    let index: String = (paths.iter())
        .map(|path| format!("100644 {}\t{path}\0", blob.trim()))
        .collect();
    git(
        dir.path(),
        &["update-index", "--add", "-z", "--index-info"],
        &index,
    );
    dir
}

#[test]
fn tracked_paths_are_read_as_written_even_with_accents() {
    // Without -z, git prints this path quoted and escaped:
    // "caf\303\251/x.txt", which ends in `.txt"` and so would hide a .wav.
    let paths = ["café/x.txt", "Músicas/canción.wav", "plain/y.txt"];
    let repo = repo_with(&paths);
    let mut listed = tracked(repo.path());
    listed.sort();
    let mut want = paths.map(String::from).to_vec();
    want.sort();
    assert_eq!(listed, want);
    assert_eq!(
        refused(listed.iter().map(String::as_str)),
        ["Músicas/canción.wav"]
    );
}

#[test]
fn no_tracked_path_is_media_or_under_fixtures_private() {
    let paths = tracked(&repo_root());
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
        // Look-alikes: a prefix match without the slash of `fixtures/private/`,
        // or a substring match on an extension, would refuse them.
        "fixtures/privateer.md",
        "docs/wave.md",
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
        "fixtures/private/new-life/setlist.json",
        "fixtures/private/new-life/truth.jsonl",
        "fixtures/private/new-life/audio.wav",
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

/// The `media` job of ci.yml, up to the next line indented less than its
/// body (`lines()` also drops the \r of a CRLF checkout).
fn media_job() -> String {
    let ci = fs::read_to_string(repo_root().join(".github/workflows/ci.yml")).unwrap();
    let job = (ci.lines())
        .skip_while(|l| *l != "  media:")
        .skip(1)
        .take_while(|l| l.is_empty() || l.starts_with("    "))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(job.contains("runs-on:"), "ci.yml has no `media` job");
    job
}

/// The job's `run: |` block as the bash script GitHub runs.
fn media_script(job: &str) -> String {
    let body: Vec<&str> = (job.lines())
        .skip_while(|l| l.trim() != "run: |")
        .skip(1)
        .take_while(|l| l.is_empty() || l.starts_with("          "))
        .map(|l| l.get(10..).unwrap_or(""))
        .collect();
    assert!(body.len() > 10, "no `run: |` block in the media job: {job}");
    body.join("\n") + "\n"
}

/// bash 4 or later from PATH (the step needs associative arrays), or None.
/// Not Windows' System32\bash.exe, which is WSL and would see another tree;
/// macOS's /bin/bash is 3.2.
fn bash4() -> Option<PathBuf> {
    let name = format!("bash{EXE_SUFFIX}");
    let bash = std::env::split_paths(&std::env::var_os("PATH")?)
        .filter(|dir| {
            let dir = dir.to_string_lossy().to_lowercase();
            !dir.contains("system32") && !dir.contains("windowsapps")
        })
        .map(|dir| dir.join(&name))
        .find(|p| p.is_file())?;
    let out = Command::new(&bash).arg("--version").output().ok()?;
    let version = String::from_utf8_lossy(&out.stdout).into_owned();
    let major: u32 = version
        .split("version ")
        .nth(1)?
        .split('.')
        .next()?
        .parse()
        .ok()?;
    (major >= 4).then_some(bash)
}

/// Runs the step's script with bash in `dir`: its exit code and output.
fn run_step(bash: &Path, script: &Path, dir: &Path) -> (Option<i32>, String) {
    let script = script.to_string_lossy().replace('\\', "/");
    let out = (Command::new(bash).arg(script).current_dir(dir))
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout) + String::from_utf8_lossy(&out.stderr);
    (out.status.code(), text.into_owned())
}

#[test]
fn the_ci_step_passes_the_real_tree_and_names_every_planted_file() {
    let job = media_job();
    assert!(
        !job.contains("continue-on-error"),
        "the media job must fail: {job}"
    );
    // Q9V-03: an `if:` on the job or on its step could skip the check.
    let has_if = |job: &str| {
        (job.lines()).any(|l| l.trim_start().trim_start_matches("- ").starts_with("if:"))
    };
    assert!(!has_if(&job), "the media job must always run: {job}");
    let planted = job.replacen("    runs-on:", "    if: false\n    runs-on:", 1);
    assert!(has_if(&planted), "a planted job-level if: is caught");
    let planted = job.replacen("      - name:", "      - if: false\n        name:", 1);
    assert!(has_if(&planted), "a planted step-level if: is caught");
    let Some(bash) = bash4() else {
        eprintln!("SKIPPED: no bash 4 or later on PATH to run ci.yml's media step");
        return;
    };
    let scripts = tempfile::tempdir().unwrap();
    let script = scripts.path().join("media-guard.sh");
    fs::write(&script, media_script(&job)).unwrap();

    let (code, text) = run_step(&bash, &script, &repo_root());
    assert_eq!(code, Some(0), "the real tree fails the CI step:\n{text}");
    assert!(text.contains("media guard: "), "{text}");

    // Empty placeholder files only, in a temporary repository's index.
    let planted = [
        "src-tauri/tests/fixtures/voxxad/PRAYER.WAV",
        "Fixtures/Private/notes.txt",
        "Músicas de adoración/canción.wav",
        "clips/Song A.MP4",
        "media/Song A.Mov",
        "a.M4V",
        "a.MKV",
        "a.WebM",
        "a.FLAC",
        "a.Mp3",
        "a.M4A",
        "fixtures/private/x.json",
        "src/Fixtures/Private/notes.txt",
        "src-tauri/fixtures/private/a.json",
        "prayer.wav",
        "src-tauri/tests/fixtures/voxxad/new.wav",
        "src-tauri/tests/fixtures/voxxad/sub/prayer.wav",
    ];
    let fine: Vec<String> = (0..100).map(|i| format!("src/file-{i}.txt")).collect();
    let mut paths: Vec<&str> = fine.iter().map(String::as_str).collect();
    paths.extend(["fixtures/privateer.md", "docs/audio.wav.md"]);
    paths.extend(ALLOWED);
    paths.extend(planted);
    let repo = repo_with(&paths);
    let (code, text) = run_step(&bash, &script, repo.path());
    assert_eq!(code, Some(1), "the CI step passed planted media:\n{text}");
    let errors: Vec<&str> = (text.lines())
        .filter(|l| l.starts_with("::error::"))
        .collect();
    for path in planted {
        let named = errors.iter().any(|l| l.ends_with(&format!(": {path}")));
        assert!(named, "the CI step did not name {path}:\n{text}");
    }
    assert_eq!(errors.len(), planted.len(), "only those are named:\n{text}");
}

#[test]
fn ci_runs_the_same_check_with_the_same_list() {
    let job = media_job();
    assert!(job.contains("git ls-files -z"), "{job}");
    assert!(job.contains("lower=${path,,}"), "{job}");
    assert!(
        job.contains("fixtures/private/*|*/fixtures/private/*"),
        "{job}"
    );
    for ext in MEDIA {
        assert!(job.contains(&format!("*{ext}")), "{ext} missing: {job}");
    }
    let listed: Vec<&str> = (job.split_whitespace())
        .filter(|w| w.starts_with("src-tauri/tests/fixtures/voxxad/"))
        .collect();
    assert_eq!(listed, ALLOWED, "ci.yml's allowlist differs from ALLOWED");
}
