//! Q9V / ADR-019 § 7: `voxxa-eval --full` prints numbers only. In `--full`,
//! `voxxad` runs Whisper, which logs the words it hears to stderr, and its
//! stdout carries them too; a private fixture is a licensed song. So nothing
//! `voxxad` prints may reach `voxxa-eval`'s output: errors name the exit code
//! or the line number only.
//!
//! No model and no audio: this test binary is also a fake `voxxad`. It copies
//! `voxxa-eval` and itself (as `voxxad`) into a temporary folder, and
//! `voxxa-eval` there runs the fake, which prints placeholder words
//! ("alpha bravo") on stderr and stdout. `harness = false` (Cargo.toml):
//! `main` is the fake when `VOXXA_FAKE_VOXXAD` is set.

use std::env::consts::EXE_SUFFIX;
use std::fs;
use std::path::Path;
use std::process::Command;

/// What the fake "heard": none of it may appear in `--full` output.
const HEARD: [&str; 2] = ["alpha", "bravo"];
const START: &str = r#"{"v":1,"t":0,"type":"status","state":"start"}"#;
const HYPOTHESIS: &str = r#"{"v":1,"t":900,"type":"window","text":"alpha bravo"}"#;

fn main() {
    if let Ok(mode) = std::env::var("VOXXA_FAKE_VOXXAD") {
        fake_voxxad(&mode);
    }
    let dir = tempfile::tempdir().unwrap();
    let bin = setup(dir.path());
    a_failing_full_run_prints_the_exit_code_and_no_heard_words(&bin);
    a_bad_line_in_a_full_run_is_named_by_its_number_only(&bin);
    an_error_line_in_a_full_run_is_named_by_its_number_only(&bin);
    a_bad_setlist_is_named_by_line_and_column_not_quoted(&bin, dir.path());
    replay_still_shows_voxxad_output_so_this_test_can_see_a_leak(&bin);
    println!("eval_numbers_only: 5 checks passed");
}

/// The fake: the words go to stderr (like Whisper's log) and stdout.
fn fake_voxxad(mode: &str) -> ! {
    eprintln!("whisper heard: alpha bravo");
    println!("{START}\n{HYPOTHESIS}");
    match mode {
        "fail" => {
            println!(r#"{{"v":1,"t":0,"type":"error","message":"alpha bravo"}}"#);
            std::process::exit(1)
        }
        "bad-line" => println!("alpha bravo"),
        "error-line" => println!(r#"{{"v":1,"t":950,"type":"error","message":"alpha bravo"}}"#),
        _ => unreachable!("unknown fake mode {mode}"),
    }
    std::process::exit(0)
}

struct Bin {
    eval: std::path::PathBuf,
    fixture: std::path::PathBuf,
}

/// `voxxa-eval` and the fake `voxxad` side by side, and a fixture with one
/// song, one truth row and an empty `audio.wav` (the fake never reads it).
fn setup(dir: &Path) -> Bin {
    let eval = dir.join(format!("voxxa-eval{EXE_SUFFIX}"));
    fs::copy(env!("CARGO_BIN_EXE_voxxa-eval"), &eval).unwrap();
    let me = std::env::current_exe().unwrap();
    fs::copy(me, dir.join(format!("voxxad{EXE_SUFFIX}"))).unwrap();
    let fixture = dir.join("song-a");
    fs::create_dir(&fixture).unwrap();
    let setlist = r#"{"setlist":[{"title":"Song A","slides":[{"id":0,"text":"charlie delta"}]}]}"#;
    fs::write(fixture.join("setlist.json"), setlist).unwrap();
    fs::write(
        fixture.join("truth.jsonl"),
        "{\"t_ms\":1000,\"song\":0,\"slide\":0}\n",
    )
    .unwrap();
    fs::write(fixture.join("session.jsonl"), "").unwrap();
    fs::write(fixture.join("audio.wav"), "").unwrap();
    Bin { eval, fixture }
}

/// Runs the copied `voxxa-eval` with the fake in `mode`; returns its exit
/// code and everything it printed, stdout and stderr together.
fn eval(bin: &Bin, mode: &str, full: bool) -> (Option<i32>, String) {
    let mut cmd = Command::new(&bin.eval);
    if full {
        cmd.arg("--full");
    }
    let out = (cmd.arg(&bin.fixture).env("VOXXA_FAKE_VOXXAD", mode))
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout) + String::from_utf8_lossy(&out.stderr);
    (out.status.code(), text.into_owned())
}

fn assert_no_heard_words(text: &str) {
    for word in HEARD {
        assert!(!text.contains(word), "--full printed a heard word:\n{text}");
    }
}

fn a_failing_full_run_prints_the_exit_code_and_no_heard_words(bin: &Bin) {
    let (code, text) = eval(bin, "fail", true);
    assert_eq!(code, Some(2), "{text}");
    assert_no_heard_words(&text);
    assert!(text.contains("voxxad failed"), "{text}");
    assert!(text.contains("exit"), "names the exit code: {text}");
    assert!(text.contains('1'), "names the exit code: {text}");
}

fn a_bad_line_in_a_full_run_is_named_by_its_number_only(bin: &Bin) {
    let (code, text) = eval(bin, "bad-line", true);
    assert_eq!(code, Some(2), "{text}");
    assert_no_heard_words(&text);
    assert!(text.contains("voxxad line 3"), "{text}");
}

fn an_error_line_in_a_full_run_is_named_by_its_number_only(bin: &Bin) {
    // The run scores (one goto-less fixture) and the error line is reported.
    let (code, text) = eval(bin, "error-line", true);
    assert_eq!(code, Some(1), "no thresholds for song-a: {text}");
    assert_no_heard_words(&text);
    assert!(text.contains("mode: full"), "{text}");
    assert!(text.contains("an error on its line 3"), "{text}");
}

/// serde quotes a value of the wrong type ("invalid type: string ..."), and
/// a setlist's values are lyrics: the error names where, not what.
fn a_bad_setlist_is_named_by_line_and_column_not_quoted(bin: &Bin, dir: &Path) {
    let fixture = dir.join("song-b");
    fs::create_dir(&fixture).unwrap();
    let setlist = "{\"setlist\":[{\"title\":\"Song B\",\n\"slides\":\"alpha bravo\"}]}";
    fs::write(fixture.join("setlist.json"), setlist).unwrap();
    let out = (Command::new(&bin.eval).args(["--full".as_ref(), fixture.as_os_str()]))
        .env("VOXXA_FAKE_VOXXAD", "fail")
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(2), "{text}");
    assert_no_heard_words(&text);
    assert!(
        text.contains("not a setlist (Data error at line 2"),
        "{text}"
    );
}

/// The default (replay, CI) path is unchanged: there `voxxad`'s stdout is
/// still shown on failure. It is public-domain text there, and it proves the
/// checks above would see a leak.
fn replay_still_shows_voxxad_output_so_this_test_can_see_a_leak(bin: &Bin) {
    let (code, text) = eval(bin, "fail", false);
    assert_eq!(code, Some(2), "{text}");
    assert!(text.contains("alpha bravo"), "{text}");
}
