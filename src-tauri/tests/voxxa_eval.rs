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
    song(t, 0, s)
}

fn song(t: u64, song: usize, s: usize) -> At {
    At {
        t,
        cue: Cue::Slide(song, s),
    }
}

fn unblank(t: u64) -> At {
    At {
        t,
        cue: Cue::Unblank,
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
    // A false advance costs precision, never recall: no slide was missed.
    assert_eq!((m.slides - m.hits, m.values()[1]), (0, Some(1.0)));
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
    assert_eq!((m.slides - m.hits, m.values()[1]), (0, Some(1.0)));
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
fn a_row_owns_from_1_s_before_it_to_1_s_before_the_next_row() {
    // Slide 0 at 1 s, slide 1 at 13 s: slide 0's span ends at 12 s.
    let t = [slide(1000, 0), slide(13000, 1)];
    let m = score(&t, &[slide(12500, 0), slide(13400, 1)]);
    let missed = m.slides - m.hits;
    assert_eq!((m.hits, missed, m.false_advances), (1, 1, 1));
    let m = score(&t, &[slide(11999, 0), slide(13400, 1)]);
    assert_eq!((m.hits, m.false_advances), (2, 0));
    assert_eq!(m.latency, [10999, 400]);
}

#[test]
fn the_span_boundaries_are_exact_to_the_millisecond() {
    // Slide 1's span starts at exactly 13000 - 1000 = 12000.
    let t = [slide(1000, 0), slide(13000, 1)];
    let m = score(&t, &[slide(12000, 1)]);
    assert_eq!(
        (m.hits, m.false_advances, m.latency.clone()),
        (1, 0, vec![-1000])
    );
    let m = score(&t, &[slide(12000, 0)]);
    assert_eq!(
        (m.hits, m.false_advances),
        (0, 1),
        "12000 belongs to slide 1"
    );
    // Exactly 1 s early for the first row still matches it.
    let m = score(&t, &[slide(0, 0)]);
    assert_eq!(
        (m.hits, m.false_advances, m.latency.clone()),
        (1, 0, vec![-1000])
    );
    // More than 1 s before the first row (an opening prayer, Q5-04) belongs
    // to no row: it is a false advance, never a hit with negative latency.
    let m = score(&[slide(2100, 0)], &[slide(1000, 0)]);
    assert_eq!((m.hits, m.false_advances), (0, 1));
}

#[test]
fn the_right_slide_number_in_the_wrong_song_is_a_false_advance() {
    let m = score(&[song(1000, 0, 1)], &[song(1500, 1, 1)]);
    assert_eq!((m.hits, m.false_advances, m.double_advances), (0, 1, 0));
    assert!(m.latency.is_empty());
    // Truth is song 1; the conductor picks song 0, slide 0.
    let m = score(&[song(1000, 1, 0)], &[song(1500, 0, 0)]);
    assert_eq!((m.hits, m.false_advances), (0, 1));
    assert_eq!(m.values()[..2], [Some(0.0), Some(0.0)]);
    // Slide 0 of another song after slide 0 is a change, not a double.
    let t = [song(1000, 0, 0), song(13000, 1, 0)];
    let m = score(&t, &[song(1500, 0, 0), song(13400, 1, 0)]);
    assert_eq!((m.hits, m.double_advances, m.false_advances), (2, 0, 0));
}

#[test]
fn an_unblank_ends_the_blank_so_the_next_blank_is_not_a_double() {
    // Blank, unblank, blank, all inside verse 1: two false advances.
    let acts = [slide(1500, 0), blank(5000), unblank(6000), blank(7000)];
    let m = score(&truth(), &acts);
    assert_eq!((m.false_advances, m.double_advances), (2, 0));
}

#[test]
fn a_blank_before_anything_is_shown_is_a_double_advance() {
    // voxxad's conductor starts blank, so a blank leaves the screen as it was.
    let m = score(&[slide(1000, 0)], &[blank(500)]);
    assert_eq!((m.double_advances, m.false_advances), (1, 0));
    assert!(m.blank.is_empty(), "the truth has no blank row");
}

#[test]
fn an_unblank_during_a_prayer_is_neither_false_nor_double() {
    // Known gap (ADR-015 D6): the unblank at 30 s shows verse 2 again in
    // the middle of the prayer, yet it only counts as an early unblank.
    let t = [truth(), vec![slide(40000, 1)]].concat();
    let unblank = At {
        t: 30000,
        cue: Cue::Unblank,
    };
    let acts = [
        slide(1500, 0),
        slide(13400, 1),
        blank(25300),
        unblank,
        slide(42000, 1),
    ];
    let m = score(&t, &acts);
    assert_eq!((m.hits, m.false_advances, m.double_advances), (3, 0, 0));
    assert_eq!(
        (m.blank.clone(), m.unblank.clone()),
        (vec![Some(300)], vec![Some(-10000)])
    );
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
    // A second blank is a double, not the unblank: the unblank is the goto.
    let m = score(&t, &[blank(25500), blank(30000), slide(42500, 1)]);
    assert_eq!(m.unblank.clone(), vec![Some(2500)]);
    assert_eq!(m.double_advances, 1);
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
    let limits = json!({ "x": {
        "precision": 1.0, "recall": 0.5, "p95_ms": 9000, "false_advances": 0,
        "double_advances": 0, "blank_ms": 9000
    } });
    let fails = check("x", &m, &limits);
    assert_eq!(fails.len(), 3, "{fails:?}");
    for want in ["recall is 0.00", "p95_ms is -", "blank_ms is miss"] {
        assert!(fails.iter().any(|f| f.contains(want)), "{want}: {fails:?}");
    }
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
    assert_eq!((total.false_advances, total.double_advances), (0, 0));
    // A double (the goto at 4 s) and a false advance (slide 2) per run.
    let acts = [
        slide(1500, 0),
        slide(4000, 0),
        slide(5000, 2),
        slide(13400, 1),
    ];
    let mut sum = Metrics::default();
    for _ in 0..2 {
        sum.add(&score(&truth(), &acts));
    }
    assert_eq!((sum.false_advances, sum.double_advances), (2, 2));
    assert_eq!((sum.slides, sum.gotos, sum.hits), (4, 8, 4));
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
fn the_table_prints_every_count_and_metric_cell() {
    // A missed slide (verse 2), and the same with a false advance (slide 2).
    let missed = score(&truth(), &[slide(1500, 0), blank(25300)]);
    let acts = [slide(1500, 0), slide(5000, 2), blank(25300)];
    let wrong = score(&truth(), &acts);
    let mut total = missed.clone();
    total.add(&wrong);
    let rows = vec![
        ("missed".to_string(), missed),
        ("wrong".into(), wrong),
        ("total".into(), total),
    ];
    let text = table(&rows);
    let cells: Vec<Vec<&str>> = text
        .lines()
        .map(|l| l.split_whitespace().collect())
        .collect();
    // fixture, slides, gotos, hits, missed, precision, recall, median ms,
    // p95 ms, false/set, double, blank ms, unblank ms.
    let want = [
        "missed 2 1 1 1 1.00 0.50 500 500 0 0 300 -",
        "wrong 2 2 1 1 0.50 0.50 500 500 1 0 300 -",
        "total 4 3 2 2 0.67 0.50 500 500 1 0 300 -",
    ];
    for (got, want) in cells[1..].iter().zip(want) {
        assert_eq!(got.join(" "), want, "{text}");
    }
    assert_eq!(cells.len(), 4, "{text}");
}

#[test]
fn the_baseline_check_names_each_worse_metric_and_refuses_unknown_names() {
    let m = score(&truth(), &[slide(1500, 0), slide(13400, 1), blank(25300)]);
    let mut ok = json!({ "a": {
        "precision": 1.0, "recall": 1.0, "median_ms": 400, "p95_ms": 500,
        "false_advances": 0, "double_advances": 0, "blank_ms": 300
    } });
    // unblank_ms is `-` here (nothing follows the blank), so it may be left out.
    assert_eq!(check("a", &m, &ok), Vec::<String>::new());
    ok["a"]["unblank_ms"] = json!(0);
    let fails = check("a", &m, &ok);
    assert!(fails[0].contains("unblank_ms is -"), "{fails:?}");
    let mut bad = ok.clone();
    bad["a"]["p95_ms"] = json!(499);
    bad["a"]["recal"] = json!(1.0);
    bad["a"].as_object_mut().unwrap().remove("unblank_ms");
    let fails = check("a", &m, &bad);
    assert_eq!(fails.len(), 2, "{fails:?}");
    assert!(
        fails.iter().any(|f| f.contains("p95_ms is 500")),
        "{fails:?}"
    );
    assert!(fails.iter().any(|f| f.contains("recal")), "{fails:?}");
    // A metric with a value but no threshold fails: dropping one cannot
    // switch its gate off.
    for name in ["recall", "false_advances", "double_advances", "p95_ms"] {
        let mut dropped = ok.clone();
        dropped["a"].as_object_mut().unwrap().remove(name);
        dropped["a"].as_object_mut().unwrap().remove("unblank_ms");
        let fails = check("a", &m, &dropped);
        let want = format!("a: {name} is");
        assert!(
            fails.len() == 1 && fails[0].contains(&want) && fails[0].contains("no threshold"),
            "{name}: {fails:?}"
        );
    }
    assert_eq!(
        check("b", &m, &ok).len(),
        1,
        "a row without thresholds fails"
    );
    // Each metric's direction on its own: precision must not fall below its
    // minimum, and counts must not rise above their maximum.
    let acts = [
        slide(1500, 0),
        slide(4000, 0),
        slide(5000, 2),
        slide(13400, 1),
        blank(25300),
    ];
    let m = score(&truth(), &acts);
    ok["a"].as_object_mut().unwrap().remove("unblank_ms");
    let fails = check("a", &m, &ok);
    // In the baseline's (sorted) key order.
    let want = [
        "a: double_advances is 1, baseline needs <= 0",
        "a: false_advances is 1, baseline needs <= 0",
        "a: precision is 0.50, baseline needs >= 1",
    ];
    assert_eq!(fails, want);
}

#[test]
fn truth_reads_indices_titles_and_blanks_and_skips_recorder_metadata() {
    let text = r#"{"t_ms":0,"event":"start","device":"x","song":"Amazing Grace"}
{"t_ms":1000,"song":"amazing grace","slide":0}
{"t_ms":2000,"song":1,"slide":3}

{"t_ms":3000,"blank":true}
{"t_ms":4000,"event":"stop"}"#;
    let songs = [
        ("Be Thou My Vision".to_string(), 4),
        ("Amazing Grace".into(), 4),
    ];
    let rows = parse_truth(text, &songs).unwrap();
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
        assert!(parse_truth(bad, &songs).is_err(), "{bad}");
    }
}

#[test]
fn a_truth_row_outside_the_setlist_is_an_error_naming_its_line() {
    let songs = [("Amazing Grace".to_string(), 4)];
    for (bad, want) in [
        (
            "{\"t_ms\":1,\"song\":0,\"slide\":0}\n{\"t_ms\":2,\"song\":0,\"slide\":4}",
            "truth line 2: {\"t_ms\":2,\"song\":0,\"slide\":4}: no slide 0/4",
        ),
        (r#"{"t_ms":1,"song":1,"slide":0}"#, "no slide 1/0"),
    ] {
        let e = parse_truth(bad, &songs).unwrap_err();
        assert!(e.contains(want), "{e}");
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
    // An action kind the scorer does not know is an error, not silently dropped.
    assert!(parse_actions(r#"{"v":1,"t":1,"type":"action","action":"bogus"}"#).is_err());
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
    // The header says 16000 Hz, 32000 bytes/s, 2-byte frames, 16 bits.
    assert_eq!(&wav[24..28], &16_000u32.to_le_bytes());
    assert_eq!(&wav[28..32], &32_000u32.to_le_bytes());
    assert_eq!(&wav[32..36], &[2, 0, 16, 0]);
}

/// A WAV whose fmt chunk says `channels`, `rate` and `bits`, with 64 bytes
/// of silence as data.
fn wav_header(channels: u16, rate: u32, bits: u16) -> Vec<u8> {
    let frame = channels * bits / 8;
    let mut w = b"RIFF".to_vec();
    w.extend((36u32 + 64).to_le_bytes());
    w.extend(b"WAVEfmt ");
    w.extend(16u32.to_le_bytes());
    w.extend(1u16.to_le_bytes());
    w.extend(channels.to_le_bytes());
    w.extend(rate.to_le_bytes());
    w.extend((rate * frame as u32).to_le_bytes());
    w.extend(frame.to_le_bytes());
    w.extend(bits.to_le_bytes());
    w.extend(b"data");
    w.extend(64u32.to_le_bytes());
    w.extend([0u8; 64]);
    w
}

#[test]
fn the_audio_recipe_refuses_a_wav_that_is_not_16_khz_mono_16_bit() {
    let dir = tempfile::tempdir().unwrap();
    let ok = dir.path().join("ok.wav");
    std::fs::write(&ok, wav_header(1, 16_000, 16)).unwrap();
    assert_eq!(audio::pcm16(&ok).unwrap(), [0u8; 64]);
    for (name, wav) in [
        ("cd.wav", wav_header(1, 44_100, 16)),
        ("stereo.wav", wav_header(2, 16_000, 16)),
        ("8bit.wav", wav_header(1, 16_000, 8)),
    ] {
        let path = dir.path().join(name);
        std::fs::write(&path, wav).unwrap();
        let e = audio::pcm16(&path).unwrap_err();
        assert!(
            e.contains("need a 16 kHz mono 16-bit PCM WAV"),
            "{name}: {e}"
        );
        let recipe = format!(r#"[{{"wav":"{name}"}}]"#);
        assert!(audio::build(dir.path(), &recipe).is_err(), "{name}");
    }
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
    // The total row adds up every fixture (3 + 2 slides) and takes the worst
    // blank (6600 ms, from prayer-without-pause) and unblank (1500 ms).
    let total: Vec<&str> = out
        .lines()
        .find(|l| l.starts_with("total"))
        .unwrap()
        .split_whitespace()
        .collect();
    assert_eq!(total[1..4], ["5", "5", "5"], "{out}");
    assert_eq!(total[11..13], ["6600", "1500"], "{out}");
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

#[test]
fn a_broken_per_fixture_threshold_fails_the_gate_while_total_passes() {
    // Each fixture row is checked on its own: a fixture that regresses
    // while staying under the looser total thresholds still fails.
    // Measured: prayer-then-reprise blank 200 ms, prayer-without-pause p95
    // 8900 ms; total allows 7000 and 9000.
    let text = std::fs::read_to_string("../fixtures/baseline.json").unwrap();
    let mut limits: serde_json::Value = serde_json::from_str(&text).unwrap();
    limits["prayer-then-reprise"]["blank_ms"] = json!(100);
    limits["prayer-without-pause"]["p95_ms"] = json!(8000);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("baseline.json");
    std::fs::write(&path, limits.to_string()).unwrap();
    let (out, code) = eval(&["--baseline", path.to_str().unwrap()]);
    assert_eq!(code, 1, "{out}");
    let want = [
        "FAIL prayer-then-reprise: blank_ms is 200, baseline needs <= 100",
        "FAIL prayer-without-pause: p95_ms is 8900, baseline needs <= 8000",
    ];
    for line in want {
        assert!(out.lines().any(|l| l == line), "{line}: {out}");
    }
    assert_eq!(out.matches("FAIL ").count(), 2, "{out}");
    assert!(!out.contains("FAIL total"), "{out}");
}

#[test]
fn a_session_log_recorded_with_another_setlist_is_an_error_not_a_score() {
    // voxxad --replay refuses the log (exit 1); voxxa-eval must stop with
    // exit 2 instead of scoring an empty action list.
    let dir = tempfile::tempdir().unwrap();
    let fixture = dir.path().join("mismatch");
    std::fs::create_dir(&fixture).unwrap();
    let from = std::path::Path::new("../fixtures/recordings");
    for (src, file) in [
        ("prayer-then-reprise", "session.jsonl"),
        ("prayer-then-reprise", "truth.jsonl"),
        ("prayer-without-pause", "setlist.json"),
    ] {
        std::fs::copy(from.join(src).join(file), fixture.join(file)).unwrap();
    }
    let out = Command::new(env!("CARGO_BIN_EXE_voxxa-eval"))
        .arg(&fixture)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(2), "{stderr}");
    assert!(stderr.contains("different setlist"), "{stderr}");
    assert!(!String::from_utf8_lossy(&out.stdout).contains("mode:"));
}
