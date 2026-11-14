//! ADR-016 (ADR-009's minimum): scripted conductor runs for a repeated
//! chorus, a bridge jump, dropped words and operator overrides, plus the
//! `sections[].kind` setlist schema and the word-token score. Hymn text comes
//! from the committed public-domain fixture setlists, never from this file.

use std::time::{Duration, Instant};
use voxxa_lib::evidence::{score, tokens, EvidenceConfig, Section, SectionKind, Tracker};
use voxxa_lib::listen::{check_setlist, Input};
use voxxa_lib::{Action, Conductor, Setlist, Slide, SmartConfig, Song};

/// It Is Well (song 0): verse, refrain, verse, verse, verse (labelled).
const REFRAIN: &str = "../fixtures/recordings/refrain-it-is-well/setlist.json";
/// Be Thou My Vision (song 0): four verses, no labels.
const JUMP: &str = "../fixtures/recordings/jump-be-thou-my-vision/setlist.json";

fn setlist(path: &str) -> Vec<Song> {
    let text = std::fs::read_to_string(path).unwrap();
    serde_json::from_str::<Setlist>(&text).unwrap().setlist
}

/// A slide's words as the stitcher hands them on: transcripts of 5 words,
/// the last one taking the rest when fewer than 4 would be left.
fn transcripts(words: &[String]) -> Vec<String> {
    let mut out: Vec<Vec<String>> = words.chunks(5).map(<[String]>::to_vec).collect();
    if out.len() > 1 && out.last().unwrap().len() < 4 {
        let tail = out.pop().unwrap();
        out.last_mut().unwrap().extend(tail);
    }
    out.iter().map(|w| w.join(" ")).collect()
}

/// A conductor fed one transcript every 1.5 s of stream time.
struct Run {
    songs: Vec<Song>,
    c: Conductor,
    t0: Instant,
    ms: u64,
    gotos: Vec<(usize, usize)>,
}

impl Run {
    fn new(songs: Vec<Song>) -> Run {
        Run::with_config(songs, SmartConfig::default())
    }

    fn with_config(songs: Vec<Song>, cfg: SmartConfig) -> Run {
        Run {
            c: Conductor::new(songs.clone(), cfg),
            songs,
            t0: Instant::now(),
            ms: 0,
            gotos: Vec::new(),
        }
    }

    fn now(&self) -> Instant {
        self.t0 + Duration::from_millis(self.ms)
    }

    fn record(&mut self, action: Action) -> Action {
        if let Action::Goto {
            song_index,
            slide_in_song,
            ..
        } = action
        {
            self.gotos.push((song_index, slide_in_song));
        }
        action
    }

    fn hear(&mut self, text: &str) -> Action {
        self.ms += 1500;
        let action = self.c.apply(&Input::Heard(text.into()), self.now());
        self.record(action)
    }

    /// Feed `words` as transcripts; the index of the first one that moved.
    fn sing_words(&mut self, words: &[String]) -> Option<usize> {
        let mut first = None;
        for (i, line) in transcripts(words).iter().enumerate() {
            if matches!(self.hear(line), Action::Goto { .. }) && first.is_none() {
                first = Some(i);
            }
        }
        first
    }

    fn words(&self, slide: usize) -> Vec<String> {
        tokens(&self.songs[0].slides[slide].text)
    }

    /// Sing a whole slide of song 0.
    fn sing(&mut self, slide: usize) -> Option<usize> {
        let words = self.words(slide);
        self.sing_words(&words)
    }

    fn step(&mut self, delta: i32) -> Action {
        self.ms += 200;
        let action = self.c.manual_step(delta, self.now());
        self.record(action)
    }
}

fn unlabelled(mut songs: Vec<Song>) -> Vec<Song> {
    songs.iter_mut().for_each(|s| s.sections.clear());
    songs
}

#[test]
fn a_repeated_chorus_is_followed_back_and_on_to_the_next_verse() {
    let songs = setlist(REFRAIN);
    assert_eq!(songs[0].sections.len(), 5, "the fixture is labelled");
    for songs in [songs.clone(), unlabelled(songs)] {
        let mut run = Run::new(songs);
        for slide in [0, 1, 2, 1, 3, 1] {
            run.sing(slide);
        }
        assert_eq!(run.gotos, [(0, 0), (0, 1), (0, 2), (0, 1), (0, 3), (0, 1)]);
    }
}

#[test]
fn a_chorus_of_two_slides_is_followed_through_both_and_on_to_the_next_verse() {
    // It Is Well with its refrain split over two slides: verse 0, chorus
    // 1-2, verse 3, verse 4. Covers `same_section` and `is_start` on a
    // section of more than one slide.
    let mut songs = setlist(REFRAIN);
    let old: Vec<String> = songs[0].slides.iter().map(|s| s.text.clone()).collect();
    let refrain: Vec<&str> = old[1].lines().collect();
    let parts = [&old[0], &refrain[..2].join("\n"), &refrain[2..].join("\n")];
    let texts = parts.into_iter().chain([&old[2], &old[3]]).cloned();
    songs[0].slides = (texts.enumerate())
        .map(|(id, text)| Slide { id, text })
        .collect();
    let section = |kind, slides: &[usize]| Section {
        kind,
        slides: slides.to_vec(),
    };
    let (verse, chorus) = (SectionKind::Verse, SectionKind::Chorus);
    songs[0].sections = vec![section(verse, &[0]), section(chorus, &[1, 2])];
    songs[0]
        .sections
        .extend([section(verse, &[3]), section(verse, &[4])]);
    let mut run = Run::new(songs);
    for slide in [0, 1, 2, 3, 1, 2, 4] {
        run.sing(slide);
    }
    let want = [(0, 0), (0, 1), (0, 2), (0, 3), (0, 1), (0, 2), (0, 4)];
    assert_eq!(run.gotos, want);
}

#[test]
fn after_a_labelled_refrain_the_verse_after_the_last_one_is_the_likely_next() {
    // Verse 3 after the second refrain, one word in five misheard. With
    // labels it is "next" (prior 0.9) and moves on its first transcript;
    // without, it is only a section start (0.6) and needs more words.
    let first_move = |songs: Vec<Song>| {
        let mut run = Run::new(songs);
        for slide in [0, 1, 2, 1] {
            run.sing(slide);
        }
        let words: Vec<String> = (run.words(3).into_iter().enumerate())
            .map(|(i, w)| if i % 5 == 2 { "mumble".into() } else { w })
            .collect();
        let moved = run.sing_words(&words);
        assert_eq!(run.gotos.last(), Some(&(0, 3)));
        moved
    };
    let songs = setlist(REFRAIN);
    let labelled = first_move(songs.clone());
    let plain = first_move(unlabelled(songs));
    assert_eq!(labelled, Some(0));
    assert!(plain > labelled, "{plain:?}");
}

#[test]
fn a_jump_to_a_bridge_and_back_to_the_chorus_is_followed() {
    let mut songs = setlist(REFRAIN);
    // Label the last verse a bridge: verse 1, then straight to the bridge.
    songs[0].sections[4].kind = SectionKind::Bridge;
    let mut run = Run::new(songs);
    for slide in [0, 4, 1] {
        run.sing(slide);
    }
    assert_eq!(run.gotos, [(0, 0), (0, 4), (0, 1)]);

    // The jump fixture's shape with no labels: verse 1, 3, 4, then 1 again.
    let mut run = Run::new(setlist(JUMP));
    for slide in [0, 2, 3, 0] {
        run.sing(slide);
    }
    assert_eq!(run.gotos, [(0, 0), (0, 2), (0, 3), (0, 0)]);
}

#[test]
fn dropped_and_misheard_words_still_follow_the_song_without_extra_moves() {
    let mut run = Run::new(setlist(REFRAIN));
    // The song starts on a clean first line (song detection is unchanged);
    // then every third word is dropped and every seventh misheard.
    let first = transcripts(&run.words(0))[0].clone();
    run.hear(&first);
    for slide in [0, 1, 2, 1, 3, 1] {
        let words: Vec<String> = (run.words(slide).into_iter().enumerate())
            .filter(|(i, _)| i % 3 != 2)
            .map(|(i, w)| if i % 7 == 6 { "mumble".into() } else { w })
            .collect();
        run.sing_words(&words);
    }
    assert_eq!(run.gotos, [(0, 0), (0, 1), (0, 2), (0, 1), (0, 3), (0, 1)]);

    // A transcript of words from no slide moves nothing, mid-verse.
    let mut run = Run::new(setlist(REFRAIN));
    let words = run.words(2);
    run.sing_words(&words[..10]);
    let before = run.gotos.clone();
    run.hear("the coffee and the donuts are in the hall");
    run.sing_words(&words[10..]);
    assert_eq!(run.gotos, before);
}

#[test]
fn the_operator_always_wins() {
    // Early step: the operator moves on while the room still sings the old
    // slide; its last words never pull the screen back.
    let mut run = Run::new(setlist(REFRAIN));
    let verse = run.words(0);
    run.sing_words(&verse[..10]);
    assert!(matches!(
        run.step(1),
        Action::Goto {
            slide_in_song: 1,
            ..
        }
    ));
    run.sing_words(&verse[10..]);
    run.sing(1);
    assert_eq!(run.gotos, [(0, 0), (0, 1)], "no move back, no second goto");
    run.sing(2);
    assert_eq!(run.gotos.last(), Some(&(0, 2)));

    // Step back: the room keeps singing the slide the operator left; the
    // conductor stays where the operator put it, then follows the next verse.
    let mut run = Run::new(setlist(REFRAIN));
    run.sing(0);
    run.sing(1);
    let verse = run.words(2);
    run.sing_words(&verse[..15]);
    assert_eq!(run.gotos.last(), Some(&(0, 2)));
    assert!(matches!(
        run.step(-1),
        Action::Goto {
            slide_in_song: 1,
            ..
        }
    ));
    run.sing_words(&verse[15..]);
    assert_eq!(run.gotos.last(), Some(&(0, 1)), "{:?}", run.gotos);
    run.sing(3);
    assert_eq!(run.gotos.last(), Some(&(0, 3)), "{:?}", run.gotos);

    // The operator restarts the song from the top: the conductor follows
    // from there, and once the room sings verse 1 the refrain the operator
    // left is a candidate again.
    let mut run = Run::new(setlist(REFRAIN));
    run.sing(0);
    run.sing(1);
    run.ms += 200;
    let jumped = run.c.jump_to_song(0, run.now());
    assert!(matches!(
        jumped,
        Action::Goto {
            slide_in_song: 0,
            ..
        }
    ));
    run.sing(0);
    run.sing(1);
    assert_eq!(
        run.gotos,
        [(0, 0), (0, 1), (0, 1)],
        "jump_to_song is not recorded"
    );
}

#[test]
fn no_move_comes_sooner_than_min_dwell_ms() {
    let mut songs = setlist(REFRAIN);
    songs[0].sections.clear();
    let mut cfg = SmartConfig::default();
    cfg.evidence.min_dwell_ms = 60_000;
    let mut c = Conductor::new(songs.clone(), cfg);
    let t0 = Instant::now();
    let mut t = 0;
    let mut gotos = 0;
    for slide in [0, 1, 2] {
        for line in transcripts(&tokens(&songs[0].slides[slide].text)) {
            t += 1500;
            let a = c.apply(&Input::Heard(line), t0 + Duration::from_millis(t));
            gotos += usize::from(matches!(a, Action::Goto { .. }));
        }
    }
    assert_eq!((gotos, c.current_index()), (1, 0), "only the song start");
}

#[test]
fn an_operator_step_starts_the_dwell() {
    // ADR-016 Decisions 5 and 6: `manual_step` sets `last_advance`, so the
    // next slide sung right after a step waits out `min_dwell_ms`, even
    // when the last move before it (the song start) is long past.
    let mut cfg = SmartConfig::default();
    cfg.evidence.min_dwell_ms = 15_000;
    let mut run = Run::with_config(unlabelled(setlist(REFRAIN)), cfg);
    run.sing(0);
    run.ms += 30_000;
    assert!(matches!(
        run.step(1),
        Action::Goto {
            slide_in_song: 1,
            ..
        }
    ));
    // Slide 2 takes 5 transcripts (7.5 s), all inside the dwell.
    assert_eq!(run.sing(2), None);
    assert_eq!(run.gotos, [(0, 0), (0, 1)]);
    run.ms += 15_000;
    assert_eq!(run.sing(2), Some(0), "the first transcript after the dwell");
    assert_eq!(run.gotos.last(), Some(&(0, 2)));
}

/// ADR-016 Decision 7: a tracker move clears the 40-word song-detection
/// buffer. Song 0 moves on to its refrain through the tracker; the next
/// song is then sung straight after the move, with no blank or silence.
/// Song detection reads only the words since the move, so song 1 starts on
/// its first transcript (`min_song_dwell_secs` has passed by then). With
/// verse 1 still in the buffer it starts later.
#[test]
fn a_tracker_move_clears_the_song_detection_buffer() {
    let songs = setlist(REFRAIN);
    let next = transcripts(&tokens(&songs[1].slides[0].text));
    let mut run = Run::new(songs);
    run.sing(0);
    // The refrain's first transcript moves to it at 10.5 s. Song 0 started
    // at 1.5 s, so the 8 s dwell is over when song 1 comes in at 12 s.
    let refrain = transcripts(&run.words(1));
    assert!(matches!(run.hear(&refrain[0]), Action::Goto { .. }));
    assert_eq!(run.gotos, [(0, 0), (0, 1)]);
    let started = next.iter().position(|line| {
        let a = run.hear(line);
        matches!(a, Action::Goto { .. })
    });
    // Without the clear: Some(4), five transcripts (6 s) late.
    assert_eq!(started, Some(0), "{:?}", run.gotos);
    assert_eq!(run.gotos, [(0, 0), (0, 1), (1, 0)]);
}

/// A song of made-up words (no hymn text), for the tracker on its own.
fn synthetic(slides: &[&str], sections: Vec<Section>) -> Song {
    Song {
        title: "Synthetic".into(),
        slides: (slides.iter().enumerate())
            .map(|(id, text)| Slide {
                id,
                text: text.to_string(),
            })
            .collect(),
        sections,
    }
}

#[test]
fn evidence_is_kept_across_transcripts_with_decay() {
    // Slide 3 is inside a labelled verse, so it is an "other" slide (prior
    // 0.4). One clean transcript of it gives 0.7 x 0.4 = 0.28, under the
    // 0.35 threshold; the second adds 0.28 to the 0.084 kept from the
    // first and moves. Without the kept share it would never move.
    let verse = Section {
        kind: SectionKind::Verse,
        slides: vec![0, 1, 2, 3],
    };
    let song = synthetic(
        &[
            "c1 c2 c3 c4 c5",
            "d1 d2 d3 d4 d5",
            "e1 e2 e3 e4 e5",
            "o1 o2 o3 o4 o5 o6 o7 o8 o9 o10",
        ],
        vec![verse],
    );
    let cfg = EvidenceConfig::default();
    let mut t = Tracker::default();
    t.hear("o1 o2 o3 o4 o5");
    assert_eq!(t.decide(0, &song, 0, &cfg, true), None, "one transcript");
    t.hear("o6 o7 o8 o9 o10");
    assert_eq!(
        t.decide(0, &song, 0, &cfg, true),
        Some(3),
        "two transcripts"
    );
}

#[test]
fn the_best_slide_must_lead_the_current_one_by_the_margin() {
    // Seven new words: all of slide 1 (next, prior 0.9: E = 0.63) and three
    // of the current slide 0 (E = 0.3 kept + 0.7 x 3/7 = 0.6). Slide 1 is
    // ahead by 0.03, less than the 0.05 margin, so the screen stays.
    let song = synthetic(&["c1 w1 w2 w3 c2", "w1 w2 w3 w4 w5 w6 w7"], vec![]);
    let decide = |margin: f64, line: &str| {
        let cfg = EvidenceConfig {
            margin,
            ..EvidenceConfig::default()
        };
        let mut t = Tracker::default();
        t.hear(line);
        t.decide(0, &song, 0, &cfg, true)
    };
    let near_tie = "w1 w2 w3 w4 w5 w6 w7";
    assert_eq!(decide(0.05, near_tie), None);
    assert_eq!(decide(0.0, near_tie), Some(1));
    // A clean line of slide 1 alone: 0.63 against 0.3 moves at the default
    // margin, but not when the margin asks for a lead of 0.5.
    assert_eq!(decide(0.05, "w4 w5 w6 w7"), Some(1));
    assert_eq!(decide(0.5, "w4 w5 w6 w7"), None);
}

#[test]
fn the_window_holds_the_last_12_words() {
    // Slide 1 has "n1" after five other words and before "x". While "x" is
    // in the window it comes before "n1", so "n1" extends no alignment;
    // once "x" has left the window, "n1" is a full score for slide 1.
    let song = synthetic(&["c1 c2 c3 c4", "a1 a2 a3 a4 a5 n1 x"], vec![]);
    let cfg = EvidenceConfig::default();
    let decide = |fillers: usize| {
        let older: Vec<String> = (1..=fillers).map(|i| format!("f{i}")).collect();
        let mut t = Tracker::default();
        t.hear(&format!("x {}", older.join(" ")));
        t.hear("n1");
        t.decide(0, &song, 0, &cfg, true)
    };
    assert_eq!(decide(10), None, "x is the 12th newest word: still heard");
    assert_eq!(decide(11), Some(1), "x is the 13th newest word: gone");
}

#[test]
fn old_setlists_load_and_serialise_exactly_as_before() {
    let text = std::fs::read_to_string(JUMP).unwrap();
    let songs = serde_json::from_str::<Setlist>(&text).unwrap().setlist;
    assert!(songs.iter().all(|s| s.sections.is_empty()));
    check_setlist(&songs).unwrap();
    let json = serde_json::to_string(&songs).unwrap();
    assert!(!json.contains("sections"), "{json}");
    let again: Vec<Song> = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_string(&again).unwrap(), json);
}

#[test]
fn section_kinds_parse_with_refrain_and_unknown_kinds() {
    let json = r#"[{"kind":"verse","slides":[0]},{"kind":"refrain","slides":[1]},
        {"kind":"chorus","slides":[2]},{"kind":"bridge","slides":[3]},
        {"kind":"tag","slides":[4]},{"kind":"pre-chorus","slides":[5]}]"#;
    let sections: Vec<Section> = serde_json::from_str(json).unwrap();
    let kinds: Vec<SectionKind> = sections.iter().map(|s| s.kind).collect();
    use SectionKind::*;
    assert_eq!(kinds, [Verse, Chorus, Chorus, Bridge, Tag, Other]);
    let songs = setlist(REFRAIN);
    check_setlist(&songs).unwrap();
    assert_eq!(songs[0].sections[1].kind, Chorus);
    assert!(serde_json::to_string(&songs[0])
        .unwrap()
        .contains(r#""kind":"chorus""#));
}

#[test]
fn bad_section_labels_are_refused_when_the_setlist_loads() {
    let cases = [
        (r#"[{"kind":"verse","slides":[5]}]"#, "past its 5 slides"),
        (r#"[{"kind":"verse","slides":[]}]"#, "no slides"),
        (r#"[{"kind":"verse","slides":[1,0]}]"#, "out of order"),
        (
            r#"[{"kind":"verse","slides":[0]},{"kind":"chorus","slides":[0]}]"#,
            "two sections",
        ),
    ];
    for (json, says) in cases {
        let mut songs = setlist(REFRAIN);
        songs[0].sections = serde_json::from_str(json).unwrap();
        let err = check_setlist(&songs).unwrap_err();
        assert!(
            err.contains("It Is Well") && err.contains(says),
            "{json}: {err}"
        );
    }
}

#[test]
fn a_config_without_evidence_values_gets_the_adr_016_defaults() {
    // The start line of a log recorded before Q6.
    let old = r#"{"advance_margin":10.0,"max_buffer_words":40,"min_advance_match":70.0,
        "min_song_dwell_secs":8.0,"min_song_match":40.0,"min_start_match":85.0,
        "silence_to_blank_secs":3.0,"slide_advance_debounce_ms":1500,
        "softmax_temperature":12.0,"song_confidence_floor":0.6,"song_second_max":0.4,
        "unrecognized_speech_to_blank_secs":5.0}"#;
    let cfg: SmartConfig = serde_json::from_str(old).unwrap();
    assert_eq!(cfg.evidence, EvidenceConfig::default());
    let ev = &cfg.evidence;
    assert_eq!(
        [
            ev.prior_stay,
            ev.prior_next,
            ev.prior_chorus,
            ev.prior_section,
            ev.prior_other
        ],
        [1.0, 0.9, 0.8, 0.6, 0.4]
    );
    assert_eq!([ev.decay, ev.threshold, ev.margin], [0.3, 0.35, 0.05]);
    assert_eq!(ev.min_dwell_ms, 1500);
    // One value given: the rest are defaults.
    let some: EvidenceConfig = serde_json::from_str(r#"{"decay":0.5}"#).unwrap();
    assert_eq!((some.decay, some.threshold), (0.5, 0.35));
}

#[test]
fn tokens_are_lower_case_words_without_marks() {
    assert_eq!(
        tokens("'Twas grace, half-shed (it is) WELL! === 2"),
        ["twas", "grace", "half", "shed", "it", "is", "well", "2"]
    );
    assert!(tokens(" ... === ").is_empty());
}

#[test]
fn only_the_newest_words_score_and_a_dropped_word_costs_its_share() {
    let w = |s: &str| tokens(s);
    let old = w("one two three four five six");
    let new = w("alpha beta gamma delta");
    let window: Vec<String> = old.iter().chain(&new).cloned().collect();
    // The old slide's words are context, not evidence for it.
    assert_eq!(score(&window, 4, &old, false), 0.0);
    assert_eq!(
        score(&window, 4, &w("alpha beta gamma delta epsilon"), false),
        1.0
    );
    // One of four new words missing from the slide: 3/4.
    assert_eq!(
        score(&window, 4, &w("alpha gamma delta epsilon"), false),
        0.75
    );
    // Out of order words do not extend the alignment.
    assert_eq!(score(&window, 4, &w("delta gamma beta alpha"), false), 0.25);
    assert_eq!(score(&window, 0, &new, true), 0.0);
}

#[test]
fn a_slide_being_entered_also_scores_on_its_leading_words() {
    let w = |s: &str| tokens(s);
    // The context lines up with the slide's last words ("six seven"), so
    // the new words, which open the slide, add nothing to that alignment.
    let slide = w("alpha beta gamma delta epsilon one two six seven");
    let window = w("four five six seven alpha beta gamma delta");
    assert_eq!(score(&window, 4, &slide, false), 0.5);
    assert_eq!(score(&window, 4, &slide, true), 1.0);
    // Leading words are the first `fresh + 4` of the slide.
    let late = w("one two three four five six seven eight alpha beta gamma delta");
    assert_eq!(
        score(&window, 4, &late, true),
        score(&window, 4, &late, false)
    );
}
