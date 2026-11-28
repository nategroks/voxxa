//! V5a (plan V5 step 9; spec § 4.5; ADR-022 § 7): the `.txt` importer keeps
//! each section label as a `sections` entry covering the slides that follow
//! it, marks a slide whose first line starts with `Credit:` as `kind:
//! "credit"`, and changes nothing already committed: every setlist loads and
//! serialises to the same JSON, and every session log replays to its own
//! action lines. Made-up words only.

mod common;

use common::committed_logs;
use std::path::Path;
use std::process::Command;
use voxxa_lib::evidence::{check_sections, Section, SectionKind};
use voxxa_lib::{txt_parse, Setlist, Slide};

/// Verse 1 spans two slides (the second block has no label), a `Bridge`
/// label has no slide after it, Verse 2 is split by `---`, and the credit
/// slide comes last.
const LANTERN: &str = "Lantern Song\n\n\
    Verse 1\nZeloma tarven osk velin\nBrindel varo mest alune\n\n\
    Tamsel unda ravi kelo\n\n\
    Chorus\nOllu ollu vessa trane\n\n\
    Bridge\n\n\
    Verse 2\nQuorra delin sefa morrow\n---\nPelor ammit ollu ane\n\n\
    Credit: Lantern Song, A. Writer, example.org/lantern, CC BY 4.0\n";

fn section(kind: SectionKind, slides: &[usize]) -> Section {
    Section {
        kind,
        slides: slides.to_vec(),
    }
}

#[test]
fn txt_section_labels_fill_sections() {
    let song = txt_parse(LANTERN, "fallback").unwrap();
    assert_eq!(song.title, "Lantern Song");
    assert_eq!(song.slides.len(), 6);
    // Each label covers the slides up to the next label; the label with no
    // slide after it (Bridge) is dropped; the credit slide is in no section.
    assert_eq!(
        song.sections,
        [
            section(SectionKind::Verse, &[0, 1]),
            section(SectionKind::Chorus, &[2]),
            section(SectionKind::Verse, &[3, 4]),
        ]
    );
    check_sections(&song).unwrap();
    // The label stays off the slide text, as before.
    for slide in &song.slides {
        for label in ["Verse", "Chorus", "Bridge"] {
            assert!(!slide.text.contains(label), "slide {}", slide.id);
        }
    }
    // Labels map the way a setlist's `sections[].kind` is read (ADR-016):
    // `refrain` is a chorus and any other accepted label is `other`.
    let kinds = "T\n\nRefrain\nabc\n\nPre-Chorus\ndef\n\nTag\nghi\n\nOutro\njkl\n";
    let song = txt_parse(kinds, "fallback").unwrap();
    let kinds: Vec<SectionKind> = song.sections.iter().map(|s| s.kind).collect();
    use SectionKind::*;
    assert_eq!(kinds, [Chorus, Other, Tag, Other]);
    // A song without labels has no sections, as before.
    let plain = txt_parse("T\n\nabc def\n\nghi jkl\n", "fallback").unwrap();
    assert!(plain.sections.is_empty());
    // Slides before the first label are in no section; a label may end in a
    // colon and carry a number.
    let lead = txt_parse(
        "T\n\nabc def\n\nVerse 1\nghi jkl\n\nmno pqr\n\nRefrain 2:\nstu vwx\n",
        "fallback",
    )
    .unwrap();
    assert_eq!(lead.slides.len(), 4);
    assert_eq!(
        lead.sections,
        [
            section(SectionKind::Verse, &[1, 2]),
            section(SectionKind::Chorus, &[3]),
        ]
    );
}

#[test]
fn credit_slide_kind_detected() {
    let song = txt_parse(LANTERN, "fallback").unwrap();
    let credit = &song.slides[5];
    assert_eq!(credit.kind.as_deref(), Some("credit"));
    assert!(
        credit.text.starts_with("Credit:"),
        "the line stays on the slide"
    );
    assert!(song.slides[..5].iter().all(|s| s.kind.is_none()));
    // Written only when set: a lyric slide's JSON has no `kind` key.
    let lyric = serde_json::to_string(&song.slides[0]).unwrap();
    assert!(!lyric.contains("kind"), "{lyric}");
    let credit = serde_json::to_string(credit).unwrap();
    assert!(credit.contains(r#""kind":"credit""#), "{credit}");
    // Only a first line that starts with `Credit:` counts.
    let later = txt_parse("T\n\nabc\nCredit: x\n\ncredit: y\n", "fallback").unwrap();
    assert!(later.slides.iter().all(|s| s.kind.is_none()));
    // A slide written before the field existed reads back with no kind.
    let old: Slide = serde_json::from_str(r#"{"id":0,"text":"zeloma"}"#).unwrap();
    assert!(old.kind.is_none());
}

/// The binary is built only with `--features cli` (see replay_determinism).
fn voxxad_exe() -> &'static str {
    if !cfg!(feature = "cli") {
        panic!("run: cargo test --features cli");
    }
    env!("CARGO_BIN_EXE_voxxad")
}

fn action_lines(text: &str) -> Vec<&str> {
    (text.lines())
        .filter(|l| l.contains(r#""type":"action""#))
        .collect()
}

/// A committed setlist loads into today's types and serialises to the same
/// JSON: no `kind`, no `sections` appears that the file does not carry. The
/// one difference that is meant to be there stays: a `refrain` section kind
/// reads as a chorus (ADR-016) and is written back as `chorus`.
fn same_json(path: &Path) -> Result<(), String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut before: serde_json::Value = serde_json::from_str(&text).unwrap();
    for song in before["setlist"].as_array_mut().unwrap() {
        if let Some(sections) = song.get_mut("sections").and_then(|s| s.as_array_mut()) {
            for section in sections.iter_mut().filter(|s| s["kind"] == "refrain") {
                section["kind"] = "chorus".into();
            }
        }
    }
    let typed: Setlist = serde_json::from_str(&text).unwrap();
    let after = serde_json::to_value(&typed).unwrap();
    (after == before)
        .then_some(())
        .ok_or_else(|| format!("{} serialises differently", path.display()))
}

#[test]
fn old_setlists_replay_unchanged() {
    let logs = committed_logs();
    assert!(logs.len() >= 7, "{logs:?}");
    let mut failures = Vec::new();
    let mut setlists: Vec<_> = logs.iter().map(|(_, _, s)| s.clone()).collect();
    setlists.push("../setlist.example.json".into());
    for setlist in &setlists {
        if let Err(e) = same_json(setlist) {
            failures.push(e);
        }
    }
    for (name, log, setlist) in &logs {
        let out = Command::new(voxxad_exe())
            .arg("--replay")
            .arg(log)
            .arg("--setlist")
            .arg(setlist)
            .output()
            .unwrap();
        if !out.status.success() {
            failures.push(format!("{name}: replay failed"));
            continue;
        }
        let replayed = String::from_utf8(out.stdout).unwrap();
        let recorded = std::fs::read_to_string(log).unwrap();
        let (again, live) = (action_lines(&replayed), action_lines(&recorded));
        if again != live {
            // Counts only: never the text.
            failures.push(format!(
                "{name}: replay printed {} action lines, the log holds {}",
                again.len(),
                live.len()
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
