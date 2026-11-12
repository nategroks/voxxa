//! voxxad's JSON lines and `--replay`, run as a process. None of these needs
//! a microphone or a Whisper model, so they run in CI.

use std::process::{Command, Output};

const DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/voxxad");

fn voxxad(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_voxxad"))
        .args(args)
        .output()
        .expect("run voxxad")
}

fn stdout(o: &Output) -> String {
    String::from_utf8(o.stdout.clone()).expect("utf-8 stdout")
}

fn replay(session: &str) -> Output {
    let setlist = format!("{DIR}/setlist.json");
    voxxad(&["--replay", session, "--setlist", &setlist])
}

/// `session.jsonl` was written by a live `voxxad --stdin-audio demo.wav
/// --shadow` run (Whisper tiny on synthetic public-domain speech). Replay
/// must print exactly the lines that run printed: every line in the log
/// except the conductor ticks, byte for byte.
#[test]
fn replay_reprints_a_live_session_byte_for_byte() {
    let session = format!("{DIR}/session.jsonl");
    let out = replay(&session);
    assert!(out.status.success(), "{}", stdout(&out));
    let log = std::fs::read_to_string(&session).unwrap();
    let printed: Vec<&str> = log
        .lines()
        .filter(|l| !l.contains(r#""ev":"tick""#))
        .collect();
    assert_eq!(stdout(&out).lines().collect::<Vec<_>>(), printed);
    assert!(printed.iter().any(|l| l.contains(r#""ev":"action""#)));
    // Deterministic: a second replay prints the same bytes.
    assert_eq!(stdout(&replay(&session)).as_bytes(), out.stdout.as_slice());
}

/// The wire format: "v":1, "t" in ms, "ev" naming the event, the
/// conductor's Action in snake_case.
#[test]
fn events_serialise_as_versioned_json_lines() {
    let session = format!("{DIR}/mini.jsonl");
    let out = replay(&session);
    assert!(out.status.success(), "{}", stdout(&out));
    // Line by line: a Windows checkout may give the fixture CRLF endings.
    let expected = std::fs::read_to_string(format!("{DIR}/mini.expected")).unwrap();
    assert_eq!(
        stdout(&out).lines().collect::<Vec<_>>(),
        expected.lines().collect::<Vec<_>>()
    );
}

#[test]
fn a_bad_session_line_is_an_error_naming_the_line() {
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.jsonl");
    std::fs::write(&session, "{\"v\":1,\"t\":0,\"ev\":\"tick\"}\nnot json\n").unwrap();
    let out = replay(session.to_str().unwrap());
    assert_eq!(out.status.code(), Some(1));
    let line = stdout(&out);
    assert!(
        line.starts_with(r#"{"v":1,"t":0,"ev":"error","message":""#),
        "{line}"
    );
    assert!(line.contains("session.jsonl:2"), "{line}");
}

#[test]
fn audio_that_is_not_16_khz_mono_pcm_is_refused_before_listening() {
    let dir = tempfile::tempdir().unwrap();
    let wav = dir.path().join("44k.wav");
    // A 44.1 kHz mono 16-bit WAV header with no samples.
    let mut b = b"RIFF\x24\0\0\0WAVEfmt \x10\0\0\0\x01\0\x01\0".to_vec();
    b.extend(44_100u32.to_le_bytes());
    b.extend(88_200u32.to_le_bytes());
    b.extend(b"\x02\0\x10\0data\0\0\0\0");
    std::fs::write(&wav, b).unwrap();
    let setlist = format!("{DIR}/setlist.json");
    let wav = wav.to_str().unwrap();
    let out = voxxad(&["--stdin-audio", wav, "--setlist", &setlist, "--shadow"]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
        stdout(&out).trim_end(),
        r#"{"v":1,"t":0,"ev":"error","message":"the WAV must be 16-bit PCM, 16 kHz mono"}"#
    );
}
