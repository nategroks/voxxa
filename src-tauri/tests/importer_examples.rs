//! Integration tests that run the importers against the example files
//! shipped in `examples/`. These exercise the parsers on real-world-shaped
//! content (multi-verse OpenLyrics with `<br/>`, ChordPro with `{key:}` /
//! comment lines / nested chord brackets, plain text with numbered verse
//! labels) rather than the synthetic snippets in the per-importer unit
//! tests.
//!
//! If you add a new file under `examples/`, add a test here too — these
//! double as smoke tests for whatever Voxxa actually ships beta operators.

use std::fs;
use std::path::PathBuf;

fn example(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("examples")
        .join(name)
}

#[test]
fn parses_amazing_grace_txt() {
    let text = fs::read_to_string(example("amazing-grace.txt")).unwrap();
    let song = voxxa_lib::txt_parse(&text, "Untitled").unwrap();
    assert_eq!(song.title, "Amazing Grace");
    assert_eq!(song.slides.len(), 4, "four verses, one per slide");
    assert!(song.slides[0].text.contains("Amazing grace"));
    assert!(!song.slides[0].text.contains("Verse"), "label was stripped");
    assert!(song.slides[3].text.contains("As long as life endures"));
}

#[test]
fn parses_how_great_thou_art_openlyrics() {
    let xml = fs::read_to_string(example("how-great-thou-art.openlyrics")).unwrap();
    let song = voxxa_lib::openlyrics_parse(&xml, "Untitled").unwrap();
    assert_eq!(song.title, "How Great Thou Art");
    // 4 verses + chorus = 5 slides.
    assert_eq!(song.slides.len(), 5);
    // Chorus should appear with `<br/>` rendered as newline.
    assert!(song.slides[1].text.contains("Then sings my soul"));
    assert!(
        song.slides[1].text.matches('\n').count() >= 3,
        "<br/> tags become newlines"
    );
}

#[test]
fn parses_be_thou_my_vision_chordpro() {
    let text = fs::read_to_string(example("be-thou-my-vision.chordpro")).unwrap();
    let song = voxxa_lib::chordpro_parse(&text, "Untitled").unwrap();
    assert_eq!(song.title, "Be Thou My Vision");
    assert_eq!(song.slides.len(), 4, "four verses");
    // Chord brackets stripped.
    assert!(!song.slides[0].text.contains("["));
    assert!(!song.slides[0].text.contains("]"));
    // Subtitle/key directives ignored, not present in slide body.
    assert!(!song.slides[0].text.contains("subtitle"));
    assert!(!song.slides[0].text.contains("Public Domain"));
}

#[test]
fn example_setlist_json_loads_cleanly() {
    let json = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("setlist.example.json"),
    )
    .unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let setlist = parsed["setlist"].as_array().expect("setlist is an array");
    assert_eq!(setlist.len(), 3, "three songs");
    for song in setlist {
        let slides = song["slides"].as_array().expect("slides is an array");
        assert!(!slides.is_empty(), "every example song has slides");
    }
}
