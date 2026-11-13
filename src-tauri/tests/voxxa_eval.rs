//! Q5 / ADR-015: `voxxa-eval`'s metrics on hand-computed cases, the audio
//! recipe, and the CI gate (replay of the committed fixture logs against
//! `fixtures/baseline.json`). No Whisper model or audio device is needed.

#[allow(dead_code)]
#[path = "../src/bin/voxxa-eval/audio.rs"]
mod audio;
#[allow(dead_code)]
#[path = "../src/bin/voxxa-eval/score.rs"]
mod score;

use score::{check, parse_actions, parse_truth, score, table, At, Cue, Metrics};
use serde_json::json;
use std::process::Command;

fn slide(t: u64, s: usize) -> At {
    At {
        t,
        cue: Cue::Slide(0, s),
    }
}

fn blank(t: u64) -> At {
    At { t, cue: Cue::Blank }
}

/// Verse 1 at 1 s, verse 2 at 13 s, a prayer blanked at 25 s.
fn truth() -> Vec<At> {
    vec![slide(1000, 0), slide(13000, 1), blank(25000)]
}

#[test]
fn a_perfect_run_scores_full_marks() {
    let m = score(&truth(), &[slide(1500, 0), slide(13400, 1), blank(25300)]);
    assert_eq!((m.slides, m.gotos, m.hits), (2, 2, 2));
    assert_eq!((m.false_advances, m.double_advances), (0, 0));
    assert_eq!(m.latency, [500, 400]);
    assert_eq!(m.blank, [Some(300)]);
    assert!(m.unblank.is_empty(), "nothing follows the blank");
    let v = m.values();
    assert_eq!(v[..4], [Some(1.0), Some(1.0), Some(400.0), Some(500.0)]);
    assert_eq!(v[6..], [Some(300.0), None]);
}

#[test]
fn a_missed_slide_lowers_recall_only() {
    let m = score(&truth(), &[slide(1500, 0), blank(25300)]);
    assert_eq!((m.hits, m.slides - m.hits, m.false_advances), (1, 1, 0));
    assert_eq!(m.values()[..2], [Some(1.0), Some(0.5)]);
}

#[test]
fn a_wrong_slide_or_an_early_blank_is_a_false_advance() {
    // Slide 2 during verse 1, and a blank in the middle of verse 2.
    let acts = [
        slide(1500, 0),
        slide(5000, 2),
        slide(13400, 1),
        blank(18000),
    ];
    let m = score(&truth(), &acts);
    assert_eq!((m.hits, m.gotos, m.false_advances), (2, 3, 2));
    assert_eq!(m.values()[0], Some(2.0 / 3.0));
    assert_eq!(
        m.blank,
        [None],
        "the early blank does not count for the prayer"
    );
}

#[test]
fn a_goto_to_the_slide_already_shown_is_a_double_advance() {
    let m = score(&truth(), &[slide(1500, 0), slide(4000, 0), slide(13400, 1)]);
    assert_eq!((m.hits, m.double_advances, m.false_advances), (2, 1, 0));
    assert_eq!(m.values()[0], Some(2.0 / 3.0));
    // After a blank the same slide is shown again: not a double.
    let m = score(
        &[slide(1000, 0)],
        &[slide(1500, 0), blank(3000), slide(4000, 0)],
    );
    assert_eq!((m.double_advances, m.false_advances), (0, 2));
}

#[test]
fn a_late_advance_counts_until_the_next_truth_row_and_is_false_after_it() {
    let m = score(&truth(), &[slide(1500, 0), slide(21000, 1)]);
    assert_eq!((m.hits, m.latency.clone()), (2, vec![500, 8000]));
    // 1 s early still matches; later than the prayer's blank it is wrong.
    let m = score(&truth(), &[slide(100, 0), slide(26000, 1)]);
    assert_eq!(
        (m.hits, m.latency.clone(), m.false_advances),
        (1, vec![-900], 1)
    );
    // More than 1 s early does not match.
    let m = score(&truth(), &[slide(11900, 1)]);
    assert_eq!((m.hits, m.false_advances), (0, 1));
}

#[test]
fn blank_and_unblank_are_timed_from_the_operator_rows() {
    // Prayer blanked at 25 s, verse 2 again at 40 s.
    let t = [truth(), vec![slide(40000, 1)]].concat();
    let acts = [
        slide(1500, 0),
        slide(13400, 1),
        blank(31000),
        slide(42500, 1),
    ];
    let m = score(&t, &acts);
    assert_eq!(
        (m.blank.clone(), m.unblank.clone()),
        (vec![Some(6000)], vec![Some(2500)])
    );
    assert_eq!((m.hits, m.false_advances, m.double_advances), (3, 0, 0));
    // An explicit unblank also ends the blank.
    let unblank = At {
        t: 41000,
        cue: Cue::Unblank,
    };
    let m = score(&t, &[blank(25500), unblank]);
    assert_eq!(
        (m.blank.clone(), m.unblank.clone()),
        (vec![Some(500)], vec![Some(1000)])
    );
    // voxxad never blanked: both are missed and fail any threshold.
    let m = score(&t, &[slide(1500, 0)]);
    assert_eq!(
        (m.blank.clone(), m.unblank.clone()),
        (vec![None], vec![None])
    );
    assert_eq!(m.values()[6..], [Some(f64::INFINITY), Some(f64::INFINITY)]);
}

#[test]
fn an_empty_action_list_has_no_hits_and_nothing_to_time() {
    let m = score(&truth(), &[]);
    assert_eq!((m.gotos, m.hits, m.false_advances), (0, 0, 0));
    let v = m.values();
    assert_eq!(v[..4], [Some(1.0), Some(0.0), None, None]);
    let limits = json!({ "x": { "recall": 0.5, "p95_ms": 9000 } });
    let fails = check("x", &m, &limits);
    assert_eq!(fails.len(), 2, "{fails:?}");
}

#[test]
fn percentiles_use_the_nearest_rank() {
    let m = Metrics {
        latency: (1..=20).map(|i| i * 100).collect(),
        ..Metrics::default()
    };
    assert_eq!(m.values()[2..4], [Some(1000.0), Some(1900.0)]);
}

#[test]
fn the_total_row_sums_counts_and_pools_latencies() {
    let mut total = Metrics::default();
    total.add(&score(&truth(), &[slide(1500, 0), blank(25300)]));
    total.add(&score(&truth(), &[slide(1300, 0), slide(14000, 1)]));
    assert_eq!((total.slides, total.gotos, total.hits), (4, 3, 3));
    assert_eq!(total.latency, [500, 300, 1000]);
    assert_eq!(
        total.values()[6],
        Some(f64::INFINITY),
        "one prayer was never blanked"
    );
    let rows = vec![
        ("a".to_string(), Metrics::default()),
        ("total".into(), total),
    ];
    let text = table(&rows);
    assert_eq!(text.lines().count(), 3, "{text}");
    assert!(text.lines().nth(2).unwrap().starts_with("total "), "{text}");
    assert!(text.contains("miss"), "{text}");
}

#[test]
fn the_baseline_check_names_each_worse_metric_and_refuses_unknown_names() {
    let m = score(&truth(), &[slide(1500, 0), slide(13400, 1), blank(25300)]);
    let ok =
        json!({ "a": { "precision": 1.0, "p95_ms": 500, "blank_ms": 300, "false_advances": 0 } });
    assert_eq!(check("a", &m, &ok), Vec::<String>::new());
    let bad = json!({ "a": { "p95_ms": 499, "recal": 1.0 } });
    let fails = check("a", &m, &bad);
    assert_eq!(fails.len(), 2, "{fails:?}");
    assert!(
        fails.iter().any(|f| f.contains("p95_ms is 500")),
        "{fails:?}"
    );
    assert!(fails.iter().any(|f| f.contains("recal")), "{fails:?}");
    assert_eq!(
        check("b", &m, &ok).len(),
        1,
        "a row without thresholds fails"
    );
}

#[test]
fn truth_reads_indices_titles_and_blanks_and_skips_recorder_metadata() {
    let text = r#"{"t_ms":0,"event":"start","device":"x","song":"Amazing Grace"}
{"t_ms":1000,"song":"Amazing Grace","slide":0}
{"t_ms":2000,"song":1,"slide":3}

{"t_ms":3000,"blank":true}
{"t_ms":4000,"event":"stop"}"#;
    let titles = ["Be Thou My Vision".to_string(), "Amazing Grace".into()];
    let rows = parse_truth(text, &titles).unwrap();
    let want = [
        At {
            t: 1000,
            cue: Cue::Slide(1, 0),
        },
        At {
            t: 2000,
            cue: Cue::Slide(1, 3),
        },
        blank(3000),
    ];
    assert_eq!(rows, want);
    for bad in [
        r#"{"t_ms":1,"song":"Unknown","slide":0}"#,
        r#"{"t_ms":1,"song":0}"#,
        r#"{"t_ms":1,"blank":false}"#,
        "{\"t_ms\":5,\"blank\":true}\n{\"t_ms\":4,\"blank\":true}",
    ] {
        assert!(parse_truth(bad, &titles).is_err(), "{bad}");
    }
}

#[test]
fn actions_are_read_from_voxxad_stdout_and_other_lines_are_skipped() {
    let out = r#"{"v":1,"t":5000,"type":"hypothesis","text":"Amazing grace"}
{"v":1,"t":5000,"type":"action","action":"goto","song":0,"slide":0,"global":0,"title":"Amazing Grace"}
{"v":1,"t":30600,"type":"action","action":"blank"}
{"v":1,"t":30600,"type":"status","state":"blank_hold","blank":true}
{"v":1,"t":31000,"type":"action","action":"unblank"}"#;
    let acts = parse_actions(out).unwrap();
    let unblank = At {
        t: 31000,
        cue: Cue::Unblank,
    };
    assert_eq!(acts, [slide(5000, 0), blank(30600), unblank]);
    assert!(parse_actions(r#"{"v":1,"t":1,"type":"action","action":"goto"}"#).is_err());
}

#[test]
fn the_audio_recipe_joins_committed_wavs_and_silences() {
    let dir = std::path::Path::new("tests/fixtures/voxxad");
    let verse = audio::pcm16(&dir.join("amazing-grace-v1.wav")).unwrap();
    let recipe = r#"[{"silence_ms":250},{"wav":"amazing-grace-v1.wav"}]"#;
    let wav = audio::build(dir, recipe).unwrap();
    let out = tempfile::tempdir().unwrap();
    std::fs::write(out.path().join("a.wav"), &wav).unwrap();
    let pcm = audio::pcm16(&out.path().join("a.wav")).unwrap();
    assert_eq!(pcm.len(), 250 * 32 + verse.len());
    assert!(pcm[..250 * 32].iter().all(|&b| b == 0));
    assert_eq!(&pcm[250 * 32..], &verse[..]);
    assert!(audio::build(dir, r#"[{"wav":"pd-hymns-setlist.json"}]"#).is_err());
    assert!(audio::build(dir, r#"[{"silence":1}]"#).is_err());
}

fn eval(args: &[&str]) -> (String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_voxxa-eval"))
        .args(args)
        .output()
        .unwrap();
    (
        String::from_utf8(out.stdout).unwrap(),
        out.status.code().unwrap(),
    )
}

#[test]
fn replay_of_the_committed_fixtures_passes_the_baseline() {
    let (out, code) = eval(&[]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("mode: replay"), "{out}");
    for row in ["prayer-then-reprise", "prayer-without-pause", "total"] {
        assert!(out.lines().any(|l| l.starts_with(row)), "{row}: {out}");
    }
}

#[test]
fn a_broken_threshold_fails_the_gate_and_names_the_metric() {
    let text = std::fs::read_to_string("../fixtures/baseline.json").unwrap();
    let mut limits: serde_json::Value = serde_json::from_str(&text).unwrap();
    limits["total"]["p95_ms"] = json!(1000);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("baseline.json");
    std::fs::write(&path, limits.to_string()).unwrap();
    let (out, code) = eval(&["--baseline", path.to_str().unwrap()]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("FAIL total: p95_ms is"), "{out}");
    assert!(!out.contains("FAIL prayer"), "{out}");
}
