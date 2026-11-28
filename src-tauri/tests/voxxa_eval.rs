//! Q5 / ADR-015: `voxxa-eval`'s metrics on hand-computed cases, the audio
//! recipe, and the CI gate (replay of the committed fixture logs against
//! `fixtures/baseline.json`). No Whisper model or audio device is needed.

#[allow(dead_code)]
#[path = "../src/cli/voxxa-eval/audio.rs"]
mod audio;
#[allow(dead_code)]
#[path = "../src/cli/voxxa-eval/score.rs"]
mod score;

use score::{check, delays, parse_actions, parse_screen, parse_truth, score, screen_time, table};
use score::{At, Cue, Metrics};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::process::Command;

/// The binary is built only with `--features cli`. Cargo sets `CARGO_BIN_EXE_voxxa-eval`
/// even when it skips the binary, and the path may hold a stale build,
/// so the feature is checked: a plain `cargo test` fails here, loudly.
fn eval_exe() -> &'static str {
    if !cfg!(feature = "cli") {
        panic!("run: cargo test --features cli");
    }
    env!("CARGO_BIN_EXE_voxxa-eval")
}

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
    let text = table(&rows, true);
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
    let text = table(&rows, true);
    let cells: Vec<Vec<&str>> = text
        .lines()
        .map(|l| l.split_whitespace().collect())
        .collect();
    // fixture, slides, gotos, hits, missed, precision, recall, median ms,
    // p95 ms, false/set, double, blank ms, unblank ms; then V2: on_time_1500,
    // tight_300, early_300, early_1000, wrong_per_10_songs, hidden_mid_song,
    // song_start_median_ms, song_start_p95_ms, wrong_slide_s. Verse 2 is
    // never shown (its on-time and tight shares are 0 of 1); the wrong run's
    // slide 2 is a wrong move (10 per 10 songs for one song, 5 for the two)
    // and stays up from 5 s: 0.5 + 8 + 12 + 0.3 s of wrong slide.
    let want = [
        "missed 2 1 1 1 1.00 0.50 500 500 0 0 300 - 0.00 0.00 0 0 0.00 0 500 500 12.80",
        "wrong 2 2 1 1 0.50 0.50 500 500 1 0 300 - 0.00 0.00 0 0 10.00 0 500 500 20.80",
        "total 4 3 2 2 0.67 0.50 500 500 1 0 300 - 0.00 0.00 0 0 5.00 0 500 500 33.60",
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
            "truth line 2: no slide 0/4",
        ),
        (
            r#"{"t_ms":1,"song":1,"slide":0}"#,
            "truth line 1: no slide 1/0",
        ),
    ] {
        let e = parse_truth(bad, &songs).unwrap_err();
        assert!(e.contains(want), "{e}");
        // The number, not the line (ADR-019 § 7, Q9V).
        assert!(!e.contains("t_ms"), "{e}");
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
    // A bad line is named by its number, never quoted: in --full, voxxad's
    // lines carry what Whisper heard (ADR-019 § 7, Q9V).
    let e = parse_actions("\n{\"v\":1}\nalpha bravo").unwrap_err();
    assert!(e.starts_with("voxxad line 3"), "{e}");
    assert!(!e.contains("alpha"), "{e}");
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
    let out = Command::new(eval_exe()).args(args).output().unwrap();
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
    for row in [
        "jump-be-thou-my-vision",
        "prayer-before-first-song",
        "prayer-between-songs",
        "prayer-then-reprise",
        "prayer-without-pause",
        "refrain-it-is-well",
        "total",
    ] {
        assert!(out.lines().any(|l| l.starts_with(row)), "{row}: {out}");
    }
    // The total row adds up every fixture (slides 4 + 2 + 4 + 3 + 2 + 6,
    // and since ADR-016 every slide is shown once: gotos and hits are the
    // same sums) and takes the worst blank and unblank (7280 and 2490 ms,
    // both from prayer-between-songs).
    let total: Vec<&str> = out
        .lines()
        .find(|l| l.starts_with("total"))
        .unwrap()
        .split_whitespace()
        .collect();
    assert_eq!(total[1..4], ["21", "21", "21"], "{out}");
    assert_eq!(total[11..13], ["7280", "2490"], "{out}");
    // V2 (plan task 5): an old truth file scores identically on the eight
    // names. The first 13 cells of every row, measured at 8df5893 before the
    // V2 columns existed; a re-recorded log (fixtures/README.md) updates them.
    let eight = [
        "jump-be-thou-my-vision 4 4 4 0 1.00 1.00 1200 5850 0 0 - -",
        "prayer-before-first-song 2 2 2 0 1.00 1.00 1710 2800 0 0 6800 1710",
        "prayer-between-songs 4 4 4 0 1.00 1.00 2490 4870 0 0 7280 2490",
        "prayer-then-reprise 3 3 3 0 1.00 1.00 2900 2900 0 0 200 1500",
        "prayer-without-pause 2 2 2 0 1.00 1.00 1900 2900 0 0 6600 -",
        "refrain-it-is-well 6 6 6 0 1.00 1.00 2120 3900 0 0 - -",
        "total 21 21 21 0 1.00 1.00 2310 4870 0 0 7280 2490",
    ];
    for want in eight {
        let name = want.split(' ').next().unwrap();
        let row = out.lines().find(|l| l.starts_with(name)).unwrap();
        let cells: Vec<&str> = row.split_whitespace().take(13).collect();
        assert_eq!(cells.join(" "), want, "{out}");
        // Every V2 cell is a number, never `-` or `n/a`, on a tapped truth.
        assert_eq!(row.split_whitespace().count(), 22, "{out}");
        assert!(
            !row.split_whitespace().skip(13).any(|c| c == "n/a"),
            "{out}"
        );
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

#[test]
fn a_broken_per_fixture_threshold_fails_the_gate_while_total_passes() {
    // Each fixture row is checked on its own: a fixture that regresses
    // while staying under the looser total thresholds still fails.
    // Measured: prayer-then-reprise blank 200 ms, prayer-without-pause p95
    // 2900 ms (8900 before ADR-016); total allows 7500 and 5000.
    let text = std::fs::read_to_string("../fixtures/baseline.json").unwrap();
    let mut limits: serde_json::Value = serde_json::from_str(&text).unwrap();
    limits["prayer-then-reprise"]["blank_ms"] = json!(100);
    limits["prayer-without-pause"]["p95_ms"] = json!(2000);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("baseline.json");
    std::fs::write(&path, limits.to_string()).unwrap();
    let (out, code) = eval(&["--baseline", path.to_str().unwrap()]);
    assert_eq!(code, 1, "{out}");
    let want = [
        "FAIL prayer-then-reprise: blank_ms is 200, baseline needs <= 100",
        "FAIL prayer-without-pause: p95_ms is 2900, baseline needs <= 2000",
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
    let out = Command::new(eval_exe()).arg(&fixture).output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(2), "{stderr}");
    assert!(stderr.contains("different setlist"), "{stderr}");
    assert!(!String::from_utf8_lossy(&out.stdout).contains("mode:"));
}

// V2 (plan task 5): the new columns on made-up truth and action lists. The
// v2() order: on_time_1500, tight_300, early_300, early_1000,
// wrong_per_10_songs, hidden_mid_song, song_start_median_ms,
// song_start_p95_ms, wrong_slide_s.

/// One song: verse 1 at 1 s, verse 2 at 13 s, verse 3 at 25 s.
fn verses() -> Vec<At> {
    vec![slide(1000, 0), slide(13000, 1), slide(25000, 2)]
}

#[test]
fn the_signed_error_is_shown_minus_truth_and_feeds_the_early_counts() {
    // +400 (late), -400 (early, inside the match window: a hit and an
    // early_300), 0. The first row is the song start, reported apart.
    let acts = [slide(1400, 0), slide(12600, 1), slide(25000, 2)];
    assert_eq!(delays(&verses(), &acts), [Some(400), Some(-400), Some(0)]);
    let m = score(&verses(), &acts);
    assert_eq!(
        (m.hits, m.false_advances, m.latency.clone()),
        (3, 0, vec![400, -400, 0])
    );
    assert_eq!(
        (m.starts.clone(), m.signed.clone()),
        (vec![Some(400)], vec![Some(-400), Some(0)])
    );
    let v = m.v2();
    assert_eq!(
        v[..6],
        [
            Some(1.0),
            Some(0.5),
            Some(1.0),
            Some(0.0),
            Some(0.0),
            Some(0.0)
        ]
    );
    assert_eq!(v[6..], [Some(400.0), Some(400.0), Some(0.8)]);
    // A slide never shown for its row is None: not early, not tight, not on time.
    let m = score(&verses(), &[slide(1000, 0), slide(25000, 2)]);
    assert_eq!(m.signed, [None, Some(0)]);
    assert_eq!(m.v2()[..4], [Some(0.5), Some(0.5), Some(0.0), Some(0.0)]);
}

#[test]
fn an_early_move_beyond_the_match_window_is_counted_early_but_not_a_hit() {
    // 1.5 s early: the eight names call it a false advance and a missed
    // slide (the 1 s window stays as it is); early_300 and early_1000 see it.
    let m = score(
        &verses(),
        &[slide(1000, 0), slide(11500, 1), slide(25000, 2)],
    );
    assert_eq!((m.hits, m.false_advances, m.wrong_moves), (2, 1, 0));
    assert_eq!(m.signed, [Some(-1500), Some(0)]);
    assert_eq!(m.v2()[..4], [Some(0.5), Some(0.5), Some(1.0), Some(1.0)]);
    // Early by exactly 300 ms is tight, not early; 301 ms is early.
    let m = score(
        &verses(),
        &[slide(1000, 0), slide(12700, 1), slide(24699, 2)],
    );
    assert_eq!(m.v2()[..4], [Some(1.0), Some(0.5), Some(1.0), Some(0.0)]);
}

#[test]
fn song_starts_are_timed_apart_and_follow_a_blank_or_a_song_change() {
    // Song 0 verses, a prayer, verse 2 again, then song 1: three starts
    // (the first row, the row after the blank, the new song), two others.
    let t = [
        song(1000, 0, 0),
        song(13000, 0, 1),
        blank(25000),
        song(40000, 0, 1),
        song(60000, 1, 0),
        song(72000, 1, 1),
    ];
    let acts = [
        song(3000, 0, 0),
        song(13100, 0, 1),
        blank(25200),
        song(42000, 0, 1),
        song(61500, 1, 0),
        song(72200, 1, 1),
    ];
    let m = score(&t, &acts);
    assert_eq!(m.starts, [Some(2000), Some(2000), Some(1500)]);
    assert_eq!(m.signed, [Some(100), Some(200)]);
    let v = m.v2();
    assert_eq!(
        v[..2],
        [Some(1.0), Some(1.0)],
        "starts never count against on time or tight"
    );
    assert_eq!(v[6..8], [Some(2000.0), Some(2000.0)]);
    assert_eq!(
        v[4],
        Some(0.0),
        "no wrong move over three song performances"
    );
    // No non-start row: on time and tight have nothing to measure.
    let m = score(&[slide(1000, 0)], &[slide(1500, 0)]);
    assert_eq!(m.v2()[..2], [None, None]);
}

#[test]
fn wrong_per_10_songs_counts_gotos_to_neither_the_current_nor_the_next_row() {
    // Slide 2 while verse 1 is sung (a skip of two): wrong. Verse 2 one
    // second before its row is the next row, so not wrong.
    let acts = [
        slide(1500, 0),
        slide(5000, 2),
        slide(12000, 1),
        slide(25400, 2),
    ];
    let m = score(&verses(), &acts);
    assert_eq!((m.wrong_moves, m.v2()[4]), (1, Some(10.0)), "one song");
    // Two songs: the same count is 5 per 10. A goto before the first row to
    // a slide that is not the first row's is wrong too.
    let t = [song(1000, 0, 0), song(13000, 1, 0)];
    let m = score(&t, &[song(500, 1, 0), song(1200, 0, 0), song(13100, 1, 0)]);
    assert_eq!((m.wrong_moves, m.v2()[4]), (1, Some(5.0)));
    // The scorer's own cases: a double advance is not a wrong move.
    let m = score(&truth(), &[slide(1500, 0), slide(4000, 0), slide(13400, 1)]);
    assert_eq!((m.double_advances, m.wrong_moves), (1, 0));
}

#[test]
fn wrong_slide_seconds_count_the_time_the_wrong_slide_id_is_up() {
    let t = [slide(1000, 0), slide(11000, 1)];
    // 2 s late into verse 2: 2.00 s wrong, then right until 10 s after the last row.
    assert_eq!(
        screen_time(&t, &[slide(1000, 0), slide(13000, 1)]),
        (2000, 0)
    );
    assert_eq!(
        score(&t, &[slide(1000, 0), slide(13000, 1)]).v2()[8],
        Some(2.0)
    );
    // A wrong slide by id (slide 2, or another song's slide 1) stays wrong to the end.
    assert_eq!(
        screen_time(&t, &[slide(1000, 0), slide(11000, 2)]).0,
        10_000
    );
    assert_eq!(
        screen_time(&t, &[slide(1000, 0), song(11000, 1, 1)]).0,
        10_000
    );
    // Nothing shown at all: wrong from the first row to 10 s after the last.
    assert_eq!(screen_time(&t, &[]), (20_000, 0));
    assert_eq!(screen_time(&[], &[slide(1000, 0)]), (0, 0));
    // An unblank shows the last slide again.
    let acts = [slide(1000, 0), blank(3000), unblank(3500), slide(11000, 1)];
    assert_eq!(screen_time(&t, &acts), (500, 0));
}

#[test]
fn hidden_mid_song_counts_blanks_over_1_s_that_start_while_a_slide_is_sung() {
    let t = [slide(1000, 0), slide(11000, 1), blank(21000)];
    // 0.9 s blank: not hidden; 6 s blank: hidden once; the prayer blank is the operator's.
    let acts = [
        slide(1000, 0),
        blank(3000),
        unblank(3900),
        blank(5000),
        slide(11000, 1),
        blank(21000),
    ];
    assert_eq!(screen_time(&t, &acts), (900 + 6000, 1));
    assert_eq!(score(&t, &acts).v2()[5], Some(1.0));
    // Before the first goto the screen is blank: a late start, not a hide.
    assert_eq!(screen_time(&[slide(1000, 0)], &[slide(4000, 0)]), (3000, 0));
    // A blank that outlives the prayer into the next song is a late start too.
    let t = [song(1000, 0, 0), blank(10000), song(20000, 1, 0)];
    let acts = [song(1000, 0, 0), blank(10000), song(23000, 1, 0)];
    assert_eq!(screen_time(&t, &acts), (3000, 0));
    // A blank after the last truth row: the song may have ended (no song-end
    // row until truth v2, V6), so not hidden, but still wrong-slide time.
    assert_eq!(
        screen_time(&[slide(1000, 0)], &[slide(1000, 0), blank(5000)]),
        (6000, 0)
    );
}

#[test]
fn an_actions_file_is_scored_like_voxxad_output_and_a_bad_line_names_the_file() {
    // The committed session log as the actions file (its action lines are
    // what replay prints): the same eight numbers as the replay row.
    let from = Path::new("../fixtures/recordings/prayer-then-reprise");
    let text = fs::read_to_string("../fixtures/baseline.json").unwrap();
    let mut limits: Value = serde_json::from_str(&text).unwrap();
    limits["total"] = limits["prayer-then-reprise"].clone();
    let dir = tempfile::tempdir().unwrap();
    let baseline = dir.path().join("baseline.json");
    fs::write(&baseline, limits.to_string()).unwrap();
    let actions = from.join("session.jsonl");
    let args = [
        "--actions",
        actions.to_str().unwrap(),
        from.to_str().unwrap(),
    ];
    let (out, code) = eval(&[&args[..], &["--baseline", baseline.to_str().unwrap()]].concat());
    assert_eq!(code, 0, "{out}");
    let row = out
        .lines()
        .find(|l| l.starts_with("prayer-then-reprise"))
        .unwrap();
    let cells: Vec<&str> = row.split_whitespace().take(13).collect();
    assert_eq!(
        cells.join(" "),
        "prayer-then-reprise 3 3 3 0 1.00 1.00 2900 2900 0 0 200 1500"
    );
    // A bad line: exit 2, the file and the line number, never the line.
    let bad = dir.path().join("bad.jsonl");
    fs::write(
        &bad,
        "{\"v\":1,\"t\":1,\"type\":\"action\",\"action\":\"goto\"}\nalpha bravo\n",
    )
    .unwrap();
    let out = Command::new(eval_exe())
        .args(["--actions", bad.to_str().unwrap()])
        .arg(from)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(2), "{stderr}");
    assert!(
        stderr.contains("bad.jsonl") && stderr.contains("line 1"),
        "{stderr}"
    );
    assert!(!stderr.contains("alpha"), "{stderr}");
    // An error line in the actions file (a saved --full run) is named by its
    // number, never quoted: its message may hold what Whisper heard.
    let noisy = dir.path().join("noisy.jsonl");
    let goto = r#"{"v":1,"t":5000,"type":"action","action":"goto","song":0,"slide":0,"global":0,"title":"x"}"#;
    let error = r#"{"v":1,"t":1,"type":"error","message":"alpha bravo"}"#;
    fs::write(&noisy, format!("{error}\n{goto}\n")).unwrap();
    let out = Command::new(eval_exe())
        .args(["--actions", noisy.to_str().unwrap()])
        .arg(from)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(1),
        "scored, below the baseline: {stderr}"
    );
    assert!(stderr.contains("an error on its line 1"), "{stderr}");
    assert!(!stderr.contains("alpha"), "{stderr}");
    // A missing value is an error, not a silent default.
    let out = Command::new(eval_exe()).arg("--actions").output().unwrap();
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn a_screen_truth_scores_the_correct_bar_and_prints_n_a_for_timing() {
    // A made-up log (plan task 7, FA-66): two read-back positions as `screen`
    // trace lines carrying extra fields V8c may add, and voxxad's two moves,
    // 0.5 and 0.2 s after them. The setlist beside it has 4, 4 and 5 slides.
    let dir = tempfile::tempdir().unwrap();
    let from = Path::new("../fixtures/recordings/prayer-then-reprise");
    fs::copy(from.join("setlist.json"), dir.path().join("setlist.json")).unwrap();
    let log = [
        r#"{"v":1,"t":0,"type":"status","state":"start"}"#,
        r#"{"v":1,"t":2000,"type":"trace","kind":"screen","slide":0,"hash":7,"source":2}"#,
        r#"{"v":1,"t":2500,"type":"action","action":"goto","song":0,"slide":0,"global":0,"title":"x"}"#,
        r#"{"v":1,"t":16000,"type":"trace","kind":"screen","slide":1}"#,
        r#"{"v":1,"t":16200,"type":"action","action":"goto","song":0,"slide":1,"global":1,"title":"x"}"#,
    ]
    .join("\n");
    let path = dir.path().join("session.jsonl");
    fs::write(&path, &log).unwrap();
    let (out, code) = eval(&["--truth-from-screen", path.to_str().unwrap()]);
    assert_eq!(code, 0, "{out}");
    assert!(out.starts_with("truth: operator-timed\n"), "{out}");
    let cells: Vec<&str> = out.lines().nth(2).unwrap().split_whitespace().collect();
    assert_eq!(
        cells[..7],
        ["screen", "2", "2", "2", "0", "1.00", "1.00"],
        "{out}"
    );
    // Every timing cell is n/a: median, p95, blank, unblank, on_time_1500,
    // tight_300, early_300, early_1000, the song-start percentiles.
    for i in [7, 8, 11, 12, 13, 14, 15, 16, 19, 20] {
        assert_eq!(cells[i], "n/a", "cell {i}: {out}");
    }
    // The Correct bar still scores: false, double, wrong moves, hidden, wrong-slide seconds.
    assert_eq!(cells[9..11], ["0", "0"], "{out}");
    assert_eq!(
        (cells[17], cells[18], cells[21]),
        ("0.00", "0", "0.70"),
        "{out}"
    );
    assert_eq!(cells.len(), 22, "{out}");
    // The parser: only t and slide are read; slide is the global index.
    let songs = [
        ("a".to_string(), 4),
        ("b".to_string(), 4),
        ("c".to_string(), 5),
    ];
    assert_eq!(
        parse_screen(&log, &songs).unwrap(),
        [slide(2000, 0), slide(16000, 1)]
    );
    let line = r#"{"v":1,"t":1,"type":"trace","kind":"screen","slide":5}"#;
    assert_eq!(parse_screen(line, &songs).unwrap(), [song(1, 1, 1)]);
    let past = r#"{"v":1,"t":1,"type":"trace","kind":"screen","slide":13}"#;
    assert_eq!(parse_screen(past, &songs).unwrap_err(), "session line 1");
    let other = r#"{"v":1,"t":1,"type":"trace","kind":"level","slide":99}"#;
    assert_eq!(parse_screen(other, &songs).unwrap(), []);
}
