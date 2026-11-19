//! ADR-016 keeps `MachineState`, blanking and `manual_step` as they were.
//! These scripts never sing one slide into the next, so position tracking
//! plays no part; their expected traces were produced by the conductor
//! before Q6 (733c7e1) and must not change. They use only API that existed
//! then.

use std::time::{Duration, Instant};
use voxxa_lib::listen::Input;
use voxxa_lib::{Action, Conductor, Setlist, SmartConfig};

/// Amazing Grace is song 0 (public domain).
const SETLIST: &str = "../setlist.example.json";

fn name(action: &Action) -> String {
    match action {
        Action::Noop => "noop".into(),
        Action::Blank => "blank".into(),
        Action::Unblank => "unblank".into(),
        Action::Goto {
            song_index,
            slide_in_song,
            ..
        } => format!("goto {song_index}/{slide_in_song}"),
    }
}

#[test]
fn machine_state_blanking_and_manual_step_behave_as_before() {
    let text = std::fs::read_to_string(SETLIST).unwrap();
    let songs = serde_json::from_str::<Setlist>(&text).unwrap().setlist;
    let lines: Vec<String> = songs[0].slides[0]
        .text
        .lines()
        .map(str::to_lowercase)
        .collect();
    let here: Vec<String> = songs[0].slides[2]
        .text
        .lines()
        .map(str::to_lowercase)
        .collect();
    let next_song = songs[1].slides[0].text.lines().next().unwrap();
    let next_song = next_song.to_lowercase();
    let mut c = Conductor::new(songs, SmartConfig::default());
    let t0 = Instant::now();
    let at = |ms: u64| t0 + Duration::from_millis(ms);
    let mut trace = Vec::new();
    let mut log = |c: &Conductor, step: &str, action: Action| {
        let state = format!("{:?}", c.state());
        trace.push(format!(
            "{step}: {} {state} blank={}",
            name(&action),
            c.is_blank()
        ));
    };
    let prayer = [
        "let us bow our heads and pray together now",
        "father we thank you for this morning and everyone",
        "quiet our hearts as we listen to your word",
    ];

    let a = c.apply(&Input::Tick, at(200));
    log(&c, "tick before anything", a);
    let a = c.apply(&Input::Heard(lines[0].clone()), at(2000));
    log(&c, "first line", a);
    let a = c.apply(&Input::Heard(lines[1].clone()), at(4000));
    log(&c, "second line", a);
    let a = c.apply(&Input::Tick, at(4200));
    log(&c, "tick while singing", a);
    let a = c.apply(&Input::Tick, at(7500));
    log(&c, "3.5 s of silence", a);
    let a = c.apply(&Input::Tick, at(10000));
    log(&c, "6 s of silence", a);
    let a = c.apply(&Input::Heard(lines[2].clone()), at(11000));
    log(&c, "third line after the blank", a);
    let a = c.manual_step(1, at(12000));
    log(&c, "step +1", a);
    let a = c.manual_step(5, at(12500));
    log(&c, "step +5", a);
    let a = c.manual_step(1, at(13000));
    log(&c, "step +1 at the end", a);
    let a = c.manual_step(-1, at(13500));
    log(&c, "step -1", a);
    let a = c.manual_step(-10, at(14000));
    log(&c, "step -10", a);
    let a = c.apply(&Input::Speech, at(14500));
    log(&c, "speech", a);
    let a = c.apply(&Input::Heard(prayer[0].into()), at(16000));
    log(&c, "prayer 1", a);
    let a = c.apply(&Input::Heard(prayer[1].into()), at(19000));
    log(&c, "prayer 2", a);
    let a = c.apply(&Input::Speech, at(21000));
    log(&c, "speech", a);
    let a = c.apply(&Input::Tick, at(21200));
    log(&c, "tick 5.2 s into unmatched speech", a);
    let a = c.apply(&Input::Heard(prayer[2].into()), at(23000));
    log(&c, "prayer 3", a);
    let a = c.apply(&Input::Tick, at(23200));
    log(&c, "tick while blank", a);
    let a = c.apply(&Input::Heard(lines[3].clone()), at(25000));
    log(&c, "fourth line", a);
    c.notify_external_blank();
    let a = c.apply(&Input::Tick, at(25200));
    log(&c, "tick after an external blank", a);
    let a = c.apply(&Input::Heard(lines[0].clone()), at(27000));
    log(&c, "first line again", a);
    let a = c.manual_step(0, at(27500));
    log(&c, "step 0", a);
    // A step clears a blank, whoever blanked.
    c.notify_external_blank();
    let a = c.manual_step(1, at(28000));
    log(&c, "step +1 after an external blank", a);
    let a = c.apply(&Input::Tick, at(31000));
    log(&c, "4 s of silence", a);
    let a = c.apply(&Input::Tick, at(33000));
    log(&c, "6 s of silence after it", a);
    let a = c.manual_step(1, at(33500));
    log(&c, "step +1 in the blank hold", a);
    // A step clears the 40-word buffer: one line of the next song, right
    // after it, is read on its own and starts that song. With the slide's
    // words still in the buffer, the line would be diluted.
    let a = c.apply(&Input::Heard(here[0].clone()), at(35000));
    log(&c, "the slide's first line", a);
    let a = c.apply(&Input::Heard(here[1].clone()), at(37000));
    log(&c, "the slide's second line", a);
    let a = c.manual_step(1, at(37500));
    log(&c, "step +1 to the last slide", a);
    let a = c.apply(&Input::Heard(next_song.clone()), at(39000));
    log(&c, "the next song's first line", a);

    let expected = [
        "tick before anything: noop Listening blank=true",
        "first line: goto 0/0 Singing blank=false",
        "second line: noop Singing blank=false",
        "tick while singing: noop Singing blank=false",
        "3.5 s of silence: noop InterVerseSilence blank=false",
        "6 s of silence: blank BlankHold blank=true",
        "third line after the blank: goto 0/0 Singing blank=false",
        "step +1: goto 0/1 Singing blank=false",
        "step +5: goto 0/3 Singing blank=false",
        "step +1 at the end: noop Singing blank=false",
        "step -1: goto 0/2 Singing blank=false",
        "step -10: goto 0/0 Singing blank=false",
        "speech: noop Singing blank=false",
        "prayer 1: noop Singing blank=false",
        "prayer 2: noop Singing blank=false",
        "speech: noop Singing blank=false",
        "tick 5.2 s into unmatched speech: blank BlankHold blank=true",
        "prayer 3: noop BlankHold blank=true",
        "tick while blank: noop BlankHold blank=true",
        "fourth line: goto 0/0 Singing blank=false",
        "tick after an external blank: noop Singing blank=true",
        "first line again: goto 0/0 Singing blank=false",
        "step 0: noop Singing blank=false",
        "step +1 after an external blank: goto 0/1 Singing blank=false",
        "4 s of silence: noop InterVerseSilence blank=false",
        "6 s of silence after it: blank BlankHold blank=true",
        "step +1 in the blank hold: goto 0/2 Singing blank=false",
        "the slide's first line: noop Singing blank=false",
        "the slide's second line: noop Singing blank=false",
        "step +1 to the last slide: goto 0/3 Singing blank=false",
        "the next song's first line: goto 1/0 Singing blank=false",
    ];
    assert_eq!(trace, expected, "{trace:#?}");
}

/// `blank_hold` clears the 40-word buffer, and re-showing the lyrics after
/// it (the current song sung again) does not. So once the lyrics are back,
/// song detection reads only the words since the blank: the next song
/// starts on its second line. With the verse sung before the blank still in
/// the buffer it would need a third.
#[test]
fn a_blank_hold_forgets_the_words_sung_before_it() {
    let text = std::fs::read_to_string(SETLIST).unwrap();
    let songs = serde_json::from_str::<Setlist>(&text).unwrap().setlist;
    let lower = |song: usize| -> Vec<String> {
        songs[song].slides[0]
            .text
            .lines()
            .map(str::to_lowercase)
            .collect()
    };
    let (first, next) = (lower(0), lower(1));
    let mut c = Conductor::new(songs, SmartConfig::default());
    let t0 = Instant::now();
    let at = |ms: u64| t0 + Duration::from_millis(ms);
    let mut trace = Vec::new();
    let mut ms = 0;
    for line in &first {
        ms += 2000;
        trace.push(name(&c.apply(&Input::Heard(line.clone()), at(ms))));
    }
    trace.push(name(&c.apply(&Input::Tick, at(ms + 3500))));
    trace.push(name(&c.apply(&Input::Tick, at(ms + 6000))));
    ms += 8000;
    trace.push(name(&c.apply(&Input::Heard(first[3].clone()), at(ms))));
    for line in &next {
        ms += 2000;
        trace.push(name(&c.apply(&Input::Heard(line.clone()), at(ms))));
    }
    let expected = [
        "goto 0/0", // the verse, line by line
        "noop", "noop", "noop", "noop",     // 3.5 s of silence: inter-verse silence
        "blank",    // 6 s: blank hold
        "goto 0/0", // its last line again: the lyrics are back
        "noop",     // the next song's first line
        "goto 1/0", // its second line
        "noop", "noop",
    ];
    assert_eq!(trace, expected, "{trace:#?}");
}
