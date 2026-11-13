//! `voxxa-eval` (ADR-015): the metric code on hand-computed cases, the
//! truth and baseline parsers, the audio recipe, and the binary's gate on
//! the committed fixtures (replay mode: no model, no audio).

#[allow(dead_code)]
#[path = "../src/bin/voxxa-eval/audio.rs"]
mod audio;
#[allow(dead_code)]
#[path = "../src/bin/voxxa-eval/metrics.rs"]
mod metrics;

use metrics::{
    check, parse_actions, parse_setlist, parse_truth, percentile, score, table, Act, Action,
    Report, Screen, Truth,
};
use std::path::Path;
use std::process::Command;

const FIXTURES: &str = "../fixtures/recordings";
const BASELINE: &str = "../fixtures/baseline.json";
const FIXTURE_NAMES: [&str; 2] = ["standin-paused-service", "standin-prayer-between"];

fn slide(t_ms: u64, slide: usize) -> Truth {
    let screen = Screen::Slide(0, slide);
    Truth { t_ms, screen }
}

fn blank(t_ms: u64) -> Truth {
    let screen = Screen::Blank;
    Truth { t_ms, screen }
}

/// Slide 0 at 1 s, slide 1 at 10 s, a blank (prayer) at 20 s, slide 2 at
/// 30 s. Windows (750 ms tolerance): [250, 9250), [9250, 19250),
/// [19250, 29250), [29250, end).
fn service() -> Vec<Truth> {
    vec![
        slide(1000, 0),
        slide(10000, 1),
        blank(20000),
        slide(30000, 2),
    ]
}

fn goto(t: u64, slide: usize) -> Action {
    let act = Act::Goto(0, slide);
    Action { t, act }
}

fn blank_at(t: u64) -> Action {
    let act = Act::Blank;
    Action { t, act }
}

fn perfect() -> Vec<Action> {
    vec![
        goto(2000, 0),
        goto(11000, 1),
        blank_at(21000),
        goto(31500, 2),
    ]
}

fn m(r: &Report, name: &str) -> Option<f64> {
    r.metric(name).unwrap()
}

#[test]
fn a_perfect_run_matches_every_row() {
    let r = score(&service(), &perfect());
    assert_eq!((r.slides, r.slides_matched, r.correct_advances), (3, 3, 3));
    assert_eq!(m(&r, "precision"), Some(1.0));
    assert_eq!(m(&r, "recall"), Some(1.0));
    assert_eq!(r.latencies, vec![1000, 1000, 1500]);
    // Nearest rank: median is rank ceil(1.5) = 2, p95 rank ceil(2.85) = 3.
    assert_eq!(m(&r, "latency_median_ms"), Some(1000.0));
    assert_eq!(m(&r, "latency_p95_ms"), Some(1500.0));
    assert_eq!(m(&r, "blank_latency_ms"), Some(1000.0));
    // Unblank: the first blank-to-slide change after the blank, 31.5 s - 30 s.
    assert_eq!(m(&r, "unblank_latency_ms"), Some(1500.0));
    for zero in [
        "false_advances",
        "double_advances",
        "missed_slides",
        "missed_blanks",
    ] {
        assert_eq!(m(&r, zero), Some(0.0), "{zero}");
    }
    for zero in ["false_blanks", "missed_unblanks"] {
        assert_eq!(m(&r, zero), Some(0.0), "{zero}");
    }
}

#[test]
fn a_missed_slide_lowers_recall_not_precision() {
    let actions = [goto(2000, 0), blank_at(21000), goto(31500, 2)];
    let r = score(&service(), &actions);
    assert_eq!(m(&r, "recall"), Some(2.0 / 3.0));
    assert_eq!(m(&r, "precision"), Some(1.0));
    assert_eq!(m(&r, "missed_slides"), Some(1.0));
    // [1000, 1500]: median rank ceil(1.0) = 1, p95 rank ceil(1.9) = 2.
    assert_eq!(m(&r, "latency_median_ms"), Some(1000.0));
    assert_eq!(m(&r, "latency_p95_ms"), Some(1500.0));
}

#[test]
fn a_wrong_slide_is_a_false_advance_and_the_right_one_after_it_still_matches() {
    let mut actions = perfect();
    actions.insert(1, goto(5000, 3));
    let r = score(&service(), &actions);
    assert_eq!(m(&r, "false_advances"), Some(1.0));
    assert_eq!(m(&r, "precision"), Some(0.75));
    assert_eq!(m(&r, "recall"), Some(1.0));
    assert_eq!(r.latencies, vec![1000, 1000, 1500]);
}

#[test]
fn a_goto_to_the_slide_already_shown_is_a_double_advance() {
    let mut actions = perfect();
    actions.insert(2, goto(12000, 1));
    let r = score(&service(), &actions);
    assert_eq!(m(&r, "double_advances"), Some(1.0));
    assert_eq!(m(&r, "false_advances"), Some(0.0));
    assert_eq!(m(&r, "precision"), Some(1.0));
    assert_eq!(r.latencies, vec![1000, 1000, 1500]);
}

#[test]
fn a_late_advance_inside_its_window_is_late_and_after_it_is_missed_and_false() {
    let late = [
        goto(2000, 0),
        goto(18000, 1),
        blank_at(21000),
        goto(31500, 2),
    ];
    let r = score(&service(), &late);
    assert_eq!(r.latencies, vec![1000, 8000, 1500]);
    assert_eq!(m(&r, "latency_p95_ms"), Some(8000.0));
    assert_eq!(m(&r, "recall"), Some(1.0));
    // Past the next row's window start (19.25 s) the operator has moved on.
    let too_late = [
        goto(2000, 0),
        goto(19500, 1),
        blank_at(21000),
        goto(31500, 2),
    ];
    let r = score(&service(), &too_late);
    assert_eq!(m(&r, "missed_slides"), Some(1.0));
    assert_eq!(m(&r, "false_advances"), Some(1.0));
    assert_eq!(m(&r, "precision"), Some(2.0 / 3.0));
    assert_eq!(m(&r, "blank_latency_ms"), Some(1000.0));
}

#[test]
fn an_early_advance_within_the_tolerance_has_a_negative_latency() {
    let early = [goto(400, 0), goto(9300, 1), blank_at(21000), goto(31500, 2)];
    let r = score(&service(), &early);
    assert_eq!(r.latencies, vec![-600, -700, 1500]);
    assert_eq!(m(&r, "false_advances"), Some(0.0));
}

#[test]
fn blank_and_unblank_timing() {
    // The silence rule blanks 1 s before the operator (outside the
    // tolerance): a false blank, and the blank row is already met.
    let early = [
        goto(2000, 0),
        goto(11000, 1),
        blank_at(19000),
        goto(33000, 2),
    ];
    let r = score(&service(), &early);
    assert_eq!(m(&r, "false_blanks"), Some(1.0));
    assert_eq!(m(&r, "missed_blanks"), Some(0.0));
    assert_eq!(m(&r, "blank_latency_ms"), None);
    assert_eq!(m(&r, "unblank_latency_ms"), Some(3000.0));
    // Lyrics shown again during the prayer: a false advance and an
    // unblank 6 s early; slide 2 still matches at 31.5 s.
    let reshown = [
        goto(2000, 0),
        goto(11000, 1),
        blank_at(21000),
        goto(24000, 1),
    ];
    let r = score(&service(), &[&reshown[..], &[goto(31500, 2)]].concat());
    assert_eq!(m(&r, "false_advances"), Some(1.0));
    assert_eq!(m(&r, "unblank_latency_ms"), Some(-6000.0));
    assert_eq!(r.latencies, vec![1000, 1000, 1500]);
    // An `unblank` shows the last slide again.
    let unblank = Action {
        t: 30500,
        act: Act::Unblank,
    };
    let truth = [slide(1000, 0), blank(20000), slide(30000, 0)];
    let r = score(&truth, &[goto(2000, 0), blank_at(20500), unblank]);
    assert_eq!(r.latencies, vec![1000, 500]);
    assert_eq!(m(&r, "unblank_latency_ms"), Some(500.0));
    assert_eq!(m(&r, "blank_latency_ms"), Some(500.0));
}

#[test]
fn a_blank_that_never_comes_is_missed_and_so_is_its_unblank() {
    let truth = [slide(1000, 0), blank(10000), slide(20000, 0)];
    let r = score(&truth, &[goto(1500, 0)]);
    // The slide after the prayer is already showing: matched, no sample.
    assert_eq!((r.slides_matched, r.correct_advances), (2, 1));
    assert_eq!(m(&r, "missed_blanks"), Some(1.0));
    assert_eq!(m(&r, "missed_unblanks"), Some(1.0));
    assert_eq!(m(&r, "unblank_latency_ms"), None);
    // Moving on to the next slide without ever blanking is not an unblank.
    let next = [slide(1000, 0), blank(10000), slide(20000, 1)];
    let r = score(&next, &[goto(1500, 0), goto(20500, 1)]);
    assert_eq!(r.latencies, vec![500, 500]);
    assert_eq!(m(&r, "missed_unblanks"), Some(1.0));
}

#[test]
fn an_empty_action_list_misses_everything_and_shows_nothing_wrong() {
    let r = score(&service(), &[]);
    assert_eq!(m(&r, "precision"), Some(1.0));
    assert_eq!(m(&r, "recall"), Some(0.0));
    assert_eq!(m(&r, "missed_slides"), Some(3.0));
    // Nothing was shown, so the blank row is already met.
    assert_eq!(m(&r, "missed_blanks"), Some(0.0));
    assert_eq!(m(&r, "missed_unblanks"), Some(1.0));
    for none in ["latency_median_ms", "latency_p95_ms", "blank_latency_ms"] {
        assert_eq!(m(&r, none), None, "{none}");
    }
    assert_eq!(score(&[], &[]), Report::default());
}

#[test]
fn a_goto_before_the_first_row_is_false_and_a_blank_on_a_blank_screen_is_not_scored() {
    let r = score(&service(), &[blank_at(50), goto(100, 0), goto(2000, 0)]);
    assert_eq!(m(&r, "false_advances"), Some(1.0));
    // At 1 s the operator's slide is already up: matched without a sample.
    assert_eq!((r.slides_matched, r.correct_advances), (1, 0));
    assert_eq!(m(&r, "double_advances"), Some(1.0));
    assert_eq!(m(&r, "false_blanks"), Some(0.0));
}

#[test]
fn percentiles_are_nearest_rank_and_totals_pool_the_samples() {
    let ten: Vec<i64> = (1..=10).map(|x| x * 100).collect();
    assert_eq!(percentile(&ten, 50), Some(500));
    assert_eq!(percentile(&ten, 95), Some(1000));
    assert_eq!(percentile(&[7], 95), Some(7));
    assert_eq!(percentile(&[], 50), None);
    let mut total = score(&service(), &perfect());
    total.add(&score(&service(), &[goto(2000, 0)]));
    assert_eq!(m(&total, "recall"), Some(4.0 / 6.0));
    assert_eq!(total.latencies, vec![1000, 1000, 1500, 1000]);
    let text = table(&[("a".into(), total.clone())], &total);
    assert_eq!(text.lines().count(), 3, "{text}");
    assert!(
        text.lines().all(|l| l.split_whitespace().count() == 13),
        "{text}"
    );
}

fn songs() -> Vec<(String, usize)> {
    vec![("Amazing Grace".into(), 4), ("Be Thou My Vision".into(), 4)]
}

#[test]
fn truth_takes_indices_or_titles_skips_event_lines_and_names_bad_lines() {
    let text = "{\"t_ms\":0,\"event\":\"start\",\"song\":\"Amazing Grace\"}\r\n\
                {\"t_ms\":1620,\"song\":0,\"slide\":0}\r\n\
                {\"t_ms\":9000,\"song\":\"be thou my vision\",\"slide\":2}\n\
                \n{\"t_ms\":28900,\"kind\":\"blank\"}\n";
    let rows = parse_truth(text, &songs()).unwrap();
    let want = [
        slide(1620, 0),
        Truth {
            t_ms: 9000,
            screen: Screen::Slide(1, 2),
        },
        blank(28900),
    ];
    assert_eq!(rows, want);
    let bad = [
        (
            "{\"t_ms\":5,\"kind\":\"unblank\"}",
            "truth line 1: unknown kind unblank",
        ),
        (
            "{\"t_ms\":5,\"song\":0,\"slide\":4}",
            "truth line 1: no slide 0/4 in the setlist",
        ),
        (
            "{\"t_ms\":5,\"song\":\"Hymn\",\"slide\":0}",
            "truth line 1: no song titled Hymn",
        ),
        ("{\"song\":0,\"slide\":0}", "truth line 1: no t_ms"),
        (
            "{\"t_ms\":9,\"kind\":\"blank\"}\n{\"t_ms\":5,\"kind\":\"blank\"}",
            "truth line 2: t_ms goes back",
        ),
    ];
    for (text, want) in bad {
        assert_eq!(parse_truth(text, &songs()).unwrap_err(), want);
    }
}

#[test]
fn only_action_lines_are_read_from_voxxad() {
    let stdout = r#"{"v":1,"t":0,"type":"status","state":"start"}
{"v":1,"t":6000,"type":"hypothesis","text":"Amazing grace"}
{"v":1,"t":6000,"type":"action","action":"goto","song":0,"slide":1,"global":1,"title":"Amazing Grace"}
{"v":1,"t":9000,"type":"action","action":"blank"}
{"v":1,"t":9500,"type":"action","action":"unblank"}
"#;
    let want = [
        goto(6000, 1),
        blank_at(9000),
        Action {
            t: 9500,
            act: Act::Unblank,
        },
    ];
    assert_eq!(parse_actions(stdout).unwrap(), want);
    assert!(parse_actions("{\"type\":\"action\",\"t\":1,\"action\":\"jump\"}").is_err());
    assert!(parse_actions("not json").is_err());
}

#[test]
fn the_gate_names_each_metric_worse_than_its_threshold() {
    let r = score(
        &service(),
        &[goto(2000, 0), blank_at(21000), goto(31500, 2)],
    );
    let limits = |json: &str| serde_json::from_str::<metrics::Limits>(json).unwrap();
    let pass = limits(r#"{"recall_min":0.5,"latency_p95_ms_max":1500,"double_advances_max":0}"#);
    assert_eq!(check("x", &r, &pass).unwrap(), Vec::<String>::new());
    let fail = limits(r#"{"recall_min":0.9,"latency_p95_ms_max":1000,"missed_slides_max":0}"#);
    let want = [
        "FAIL x: latency_p95_ms = 1500, worse than max 1000",
        "FAIL x: missed_slides = 1, worse than max 0",
        "FAIL x: recall = 0.667, worse than min 0.9",
    ];
    let mut failures = check("x", &r, &fail).unwrap();
    failures.sort();
    assert_eq!(failures, want);
    // No sample: not checked (missed_slides and recall catch it).
    let empty = score(&service(), &[]);
    assert!(check("x", &empty, &limits(r#"{"latency_p95_ms_max":0}"#))
        .unwrap()
        .is_empty());
    for bad in [
        r#"{"recal_min":1}"#,
        r#"{"recall":1}"#,
        r#"{"recall_min":"1"}"#,
    ] {
        assert!(check("x", &r, &limits(bad)).is_err(), "{bad}");
    }
}

#[test]
fn each_committed_fixture_rebuilds_its_audio_and_its_truth_fits_inside_it() {
    for name in FIXTURE_NAMES {
        let dir = Path::new(FIXTURES).join(name);
        let wav = audio::build(&dir.join("audio.json")).unwrap();
        assert_eq!(wav, audio::build(&dir.join("audio.json")).unwrap());
        let pcm = audio::pcm16_mono_16k(&wav).unwrap();
        let ms = pcm.len() as u64 / 32;
        let read = |f: &str| std::fs::read_to_string(dir.join(f)).unwrap();
        let songs = parse_setlist(&read("setlist.json")).unwrap();
        let truth = parse_truth(&read("truth.jsonl"), &songs).unwrap();
        assert!(
            truth.len() >= 3 && truth.last().unwrap().t_ms < ms,
            "{name}"
        );
        // The setlist is the one the session log was recorded with.
        let fixture_setlist = read("setlist.json");
        let test_setlist = std::fs::read_to_string("tests/fixtures/voxxad/pd-hymns-setlist.json");
        assert_eq!(
            parse_setlist(&fixture_setlist),
            parse_setlist(&test_setlist.unwrap())
        );
    }
    // 1.5 s + 12 s + 2 s + 10.3 s + 3 s + 8.1 s + 4 s of 16 kHz 16-bit audio.
    let wav = audio::build(
        &Path::new(FIXTURES)
            .join(FIXTURE_NAMES[0])
            .join("audio.json"),
    );
    assert_eq!(
        wav.unwrap().len(),
        44 + (1500 + 11995 + 2000 + 10285 + 3000 + 8085 + 4000) * 32
    );
}

#[test]
fn a_recipe_refuses_audio_that_is_not_16_khz_mono_pcm() {
    let dir = tempfile::tempdir().unwrap();
    let mut wav = audio::wav(&[0; 64]);
    wav[24..28].copy_from_slice(&8000u32.to_le_bytes());
    std::fs::write(dir.path().join("8k.wav"), wav).unwrap();
    let recipe = dir.path().join("audio.json");
    std::fs::write(
        &recipe,
        r#"{"v":1,"parts":[{"silence_ms":10},{"wav":"8k.wav"}]}"#,
    )
    .unwrap();
    let err = audio::build(&recipe).unwrap_err();
    assert!(
        err.ends_with("8k.wav: need 16 kHz mono 16-bit PCM"),
        "{err}"
    );
    std::fs::write(&recipe, r#"{"v":1,"parts":[{"silence":10}]}"#).unwrap();
    assert!(audio::build(&recipe)
        .unwrap_err()
        .contains("neither silence_ms nor wav"));
}

fn eval(args: &[&str]) -> (String, String, Option<i32>) {
    let out = Command::new(env!("CARGO_BIN_EXE_voxxa-eval"))
        .args([
            "--voxxad",
            env!("CARGO_BIN_EXE_voxxad"),
            "--fixtures",
            FIXTURES,
        ])
        .args(args)
        .output()
        .unwrap();
    let text = |b: Vec<u8>| String::from_utf8(b).unwrap();
    (text(out.stdout), text(out.stderr), out.status.code())
}

#[test]
fn replay_on_the_committed_fixtures_meets_the_committed_baseline() {
    let (stdout, stderr, code) = eval(&["--baseline", BASELINE]);
    assert_eq!(code, Some(0), "{stdout}{stderr}");
    for name in FIXTURE_NAMES.iter().chain(&["total"]) {
        assert_eq!(
            stdout.lines().filter(|l| l.starts_with(name)).count(),
            1,
            "{stdout}"
        );
    }
    assert!(stdout.contains("gate: every metric meets"), "{stdout}");
}

#[test]
fn a_broken_threshold_fails_the_gate_and_names_the_metric() {
    let dir = tempfile::tempdir().unwrap();
    let mut baseline: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(BASELINE).unwrap()).unwrap();
    baseline["total"]["latency_p95_ms_max"] = 1000.into();
    let broken = dir.path().join("baseline.json");
    std::fs::write(&broken, baseline.to_string()).unwrap();
    let (stdout, _, code) = eval(&["--baseline", broken.to_str().unwrap()]);
    assert_eq!(code, Some(1), "{stdout}");
    let fails: Vec<&str> = stdout.lines().filter(|l| l.starts_with("FAIL")).collect();
    assert_eq!(fails.len(), 1, "{stdout}");
    assert!(
        fails[0].starts_with("FAIL total: latency_p95_ms = "),
        "{stdout}"
    );
    assert!(fails[0].ends_with("worse than max 1000"), "{stdout}");
    // A misspelt threshold is an error before any run, not a silent pass.
    baseline["total"]["latency_p59_ms_max"] = 1.into();
    std::fs::write(&broken, baseline.to_string()).unwrap();
    let (_, stderr, code) = eval(&["--baseline", broken.to_str().unwrap()]);
    assert_eq!(code, Some(2));
    assert!(stderr.contains("unknown metric latency_p59_ms"), "{stderr}");
}

#[test]
fn a_fixture_without_a_session_log_fails_replay_instead_of_being_skipped() {
    let dir = tempfile::tempdir().unwrap();
    let fx = dir.path().join("new");
    std::fs::create_dir(&fx).unwrap();
    let from = Path::new(FIXTURES).join(FIXTURE_NAMES[0]);
    for f in ["setlist.json", "truth.jsonl"] {
        std::fs::copy(from.join(f), fx.join(f)).unwrap();
    }
    let out = Command::new(env!("CARGO_BIN_EXE_voxxa-eval"))
        .args(["--voxxad", env!("CARGO_BIN_EXE_voxxad"), "--fixtures"])
        .arg(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("new: no session.jsonl"), "{stderr}");
}
