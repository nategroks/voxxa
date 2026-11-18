//! Q6 / ADR-009 minimum: the conductor's evidence tracker on scripted
//! recogniser output (a repeated chorus, a bridge jump, dropped words, an
//! operator override), and a trace showing that `MachineState`, blanking and
//! `manual_step` behave as they did before it (733c7e1). Placeholder words
//! only: no song text.

use std::time::{Duration, Instant};
use voxxa_lib::{Action, Conductor, Setlist, SmartConfig};

/// Song 0: verse, chorus, verse, bridge, verse. Song 1: two slides.
/// Every slide shares a few filler words, as lyrics do.
const SETLIST: &str = r#"{"setlist":[
  {"title":"Placeholder One","slides":[
    {"id":0,"text":"alpha bravo the charlie delta and echo foxtrot golf my hotel india juliet kilo lima"},
    {"id":1,"text":"mike november the oscar papa and quebec romeo sierra my tango uniform victor"},
    {"id":2,"text":"whiskey xray the yankee zulu and amber bronze copper my dune ember fjord glacier"},
    {"id":3,"text":"harbor island the jungle kettle and lagoon meadow nectar my orchard prairie quarry"},
    {"id":4,"text":"river summit the tundra upland and valley willow yarrow my zephyr anchor beacon"}],
   "sections":[{"kind":"verse","start":0},{"kind":"chorus","start":1},{"kind":"verse","start":2},
               {"kind":"bridge","start":3},{"kind":"verse","start":4}]},
  {"title":"Placeholder Two","slides":[
    {"id":5,"text":"cobalt dagger the falcon garnet and hazel indigo jasper my kestrel lantern marble"},
    {"id":6,"text":"nickel opal the pebble quartz and raven saffron topaz my umber violet walnut"}]}
]}"#;

fn conductor() -> Conductor {
    let setlist: Setlist = serde_json::from_str(SETLIST).unwrap();
    Conductor::new(setlist.setlist, SmartConfig::default())
}

fn slide_words(song: usize, slide: usize) -> Vec<String> {
    let setlist: Setlist = serde_json::from_str(SETLIST).unwrap();
    let text = &setlist.setlist[song].slides[slide].text;
    text.split_whitespace().map(String::from).collect()
}

/// The recogniser's output for a slide sung from `ms`: 5 words every 1.5 s.
fn sing(words: &[String], ms: &mut u64) -> Vec<(u64, String)> {
    words
        .chunks(5)
        .map(|chunk| {
            *ms += 1500;
            (*ms, chunk.join(" "))
        })
        .collect()
}

/// Feeds the hops and returns every goto as (ms, song, slide), every other
/// non-Noop action as (ms, usize::MAX, 0).
fn run(c: &mut Conductor, t0: Instant, hops: &[(u64, String)]) -> Vec<(u64, usize, usize)> {
    let mut out = Vec::new();
    for (ms, text) in hops {
        match c.on_transcript(text, t0 + Duration::from_millis(*ms)) {
            Action::Noop => {}
            Action::Goto {
                song_index,
                slide_in_song,
                ..
            } => out.push((*ms, song_index, slide_in_song)),
            _ => out.push((*ms, usize::MAX, 0)),
        }
    }
    out
}

/// Sings the given slides of song 0 in order; returns the slides gone to.
fn follow(slides: &[usize], drop: impl Fn(Vec<String>) -> Vec<String>) -> Vec<usize> {
    let mut c = conductor();
    let mut ms = 0;
    // The first slide is heard whole: starting a song is not the tracker's job.
    let hops: Vec<_> = (slides.iter().enumerate())
        .flat_map(|(i, &s)| {
            let words = slide_words(0, s);
            sing(&if i == 0 { words } else { drop(words) }, &mut ms)
        })
        .collect();
    let gotos = run(&mut c, Instant::now(), &hops);
    assert!(gotos.iter().all(|g| g.1 == 0), "{gotos:?}");
    gotos.iter().map(|g| g.2).collect()
}

#[test]
fn a_repeated_chorus_is_shown_again_after_each_verse() {
    assert_eq!(follow(&[0, 1, 2, 1, 4, 1], |w| w), [0, 1, 2, 1, 4, 1]);
}

#[test]
fn a_jump_to_the_bridge_and_back_to_the_chorus_is_followed() {
    assert_eq!(follow(&[0, 1, 3, 1], |w| w), [0, 1, 3, 1]);
}

#[test]
fn dropped_and_misheard_words_still_advance_once_per_slide() {
    // Every third word lost and every fifth misheard.
    let lossy = |w: Vec<String>| -> Vec<String> {
        (w.into_iter().enumerate())
            .filter(|(i, _)| i % 3 != 2)
            .map(|(i, w)| if i % 5 == 4 { "mumble".into() } else { w })
            .collect()
    };
    assert_eq!(follow(&[0, 1, 2], lossy), [0, 1, 2]);
}

#[test]
fn the_operator_wins_and_tracking_resumes_from_the_operators_slide() {
    let mut c = conductor();
    let t0 = Instant::now();
    let mut ms = 0;
    let verse = sing(&slide_words(0, 0), &mut ms);
    assert_eq!(run(&mut c, t0, &verse[..2]), [(1500, 0, 0)]);
    // The band skips to the bridge; the operator steps there.
    let step = c.manual_step(3, t0 + Duration::from_millis(3100));
    assert!(matches!(
        step,
        Action::Goto {
            slide_in_song: 3,
            ..
        }
    ));
    // Words of the first verse heard before the step change nothing.
    assert_eq!(run(&mut c, t0, &verse[2..]), []);
    // The bridge is sung: no goto back; then the next verse: one goto.
    ms += 2000;
    let bridge = sing(&slide_words(0, 3), &mut ms);
    assert_eq!(run(&mut c, t0, &bridge), []);
    let next = sing(&slide_words(0, 4), &mut ms);
    let gotos = run(&mut c, t0, &next);
    assert_eq!(
        gotos.iter().map(|g| (g.1, g.2)).collect::<Vec<_>>(),
        [(0, 4)]
    );
}

/// One line per input: ms, what was applied, the action, the state, blank.
fn trace(c: &mut Conductor, t0: Instant, script: &[(u64, String)]) -> Vec<String> {
    let mut out = Vec::new();
    for (ms, step) in script {
        let (ms, step) = (*ms, step.as_str());
        let now = t0 + Duration::from_millis(ms);
        let action = match step.split_once(' ') {
            Some(("step", d)) => c.manual_step(d.parse().unwrap(), now),
            Some(("heard", text)) => c.on_transcript(text, now),
            _ if step == "tick" => c.tick(now),
            _ if step == "external-blank" => {
                c.notify_external_blank();
                Action::Noop
            }
            _ => panic!("bad step {step}"),
        };
        let action = match action {
            Action::Goto {
                song_index,
                slide_in_song,
                ..
            } => format!("goto {song_index}/{slide_in_song}"),
            other => format!("{other:?}").to_lowercase(),
        };
        let state = format!("{:?}", c.state());
        out.push(format!(
            "{ms} {step:.12} -> {action} {state} {}",
            c.is_blank()
        ));
    }
    out
}

#[test]
fn machine_state_blanking_and_manual_step_behave_as_before() {
    let v = |s: usize, from: usize| slide_words(0, s)[from..from + 5].join(" ");
    let (a, b, c2) = (v(0, 0), v(0, 5), v(0, 10));
    let other = "seven eight nine ten eleven twelve";
    let script: Vec<(u64, String)> = vec![
        (0, "tick".into()),
        (500, "heard seven eight nine ten".into()),
        (1000, format!("heard {a}")),
        (2500, format!("heard {b}")),
        (3000, "tick".into()),
        (6000, "tick".into()),
        (8000, "tick".into()),
        (8500, "tick".into()),
        (9000, format!("heard {c2}")),
        (9100, "tick".into()),
        (9500, "step 1".into()),
        (9600, "step 9".into()),
        (9700, "step 1".into()),
        (9800, "step -4".into()),
        (9900, "step -1".into()),
        (10000, "external-blank".into()),
        (10500, format!("heard {a}")),
        (11000, "tick".into()),
        (12000, format!("heard {other}")),
        (14000, format!("heard {other}")),
        (16000, "tick".into()),
        (17500, "tick".into()),
        (18000, format!("heard {other}")),
        (18100, "tick".into()),
        (19000, format!("heard {a}")),
        (25000, "tick".into()),
        (26000, "tick".into()),
    ];
    let mut c = conductor();
    let got = trace(&mut c, Instant::now(), &script);
    // Recorded at 733c7e1, before the evidence tracker.
    let want = [
        "0 tick -> noop Listening true",
        "500 heard seven  -> noop Listening true",
        "1000 heard alpha  -> goto 0/0 Singing false",
        "2500 heard and ec -> noop Singing false",
        "3000 tick -> noop Singing false",
        "6000 tick -> noop InterVerseSilence false",
        "8000 tick -> blank BlankHold true",
        "8500 tick -> noop BlankHold true",
        "9000 heard hotel  -> goto 0/0 Singing false",
        "9100 tick -> noop Singing false",
        "9500 step 1 -> goto 0/1 Singing false",
        "9600 step 9 -> goto 0/4 Singing false",
        "9700 step 1 -> noop Singing false",
        "9800 step -4 -> goto 0/0 Singing false",
        "9900 step -1 -> noop Singing false",
        "10000 external-bla -> noop Singing true",
        "10500 heard alpha  -> goto 0/0 Singing false",
        "11000 tick -> noop Singing false",
        "12000 heard seven  -> noop Singing false",
        "14000 heard seven  -> noop Singing false",
        "16000 tick -> noop Singing false",
        "17500 tick -> noop InterVerseSilence false",
        "18000 heard seven  -> blank BlankHold true",
        "18100 tick -> noop BlankHold true",
        "19000 heard alpha  -> goto 0/0 Singing false",
        "25000 tick -> noop InterVerseSilence false",
        "26000 tick -> blank BlankHold true",
    ];
    assert_eq!(got, want, "\n{}", got.join("\n"));
}
