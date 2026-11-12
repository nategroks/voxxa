//! Regression tests for the defects fixed in Q2 (.10x/reviews/2026-11-11-ponytail-q0.md §4).
//! Each test names the defect it guards.

use std::path::{Path, PathBuf};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn rust_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// ADR-007: no copyrighted lyrics in the sources, test fixtures included.
#[test]
fn no_copyrighted_lyric_fixtures_in_sources() {
    // Split so this file does not match itself.
    let banned = [
        ["way", " maker"].concat(),
        ["how great", " is our god"].concat(),
    ];
    let mut files = Vec::new();
    rust_sources(&manifest_dir().join("src"), &mut files);
    for f in files {
        let text = std::fs::read_to_string(&f).unwrap().to_lowercase();
        for b in &banned {
            assert!(!text.contains(b.as_str()), "{} contains '{b}'", f.display());
        }
    }
}

/// A minimal PDF with one text line per page. `font` is the font name the
/// content stream uses; the page resources only define /F1.
fn pdf(pages: &[&str], font: &str, media_box: &str) -> Vec<u8> {
    let kids: Vec<String> = (0..pages.len())
        .map(|i| format!("{} 0 R", 4 + 2 * i))
        .collect();
    let mut objs = vec![
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        format!(
            "<< /Type /Pages /Kids [{}] /Count {} >>",
            kids.join(" "),
            pages.len()
        ),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string(),
    ];
    for (i, text) in pages.iter().enumerate() {
        let content = if text.is_empty() {
            String::new()
        } else {
            format!("BT /{font} 18 Tf 72 720 Td ({text}) Tj ET")
        };
        objs.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox {media_box} \
             /Resources << /Font << /F1 3 0 R >> >> /Contents {} 0 R >>",
            5 + 2 * i
        ));
        objs.push(format!(
            "<< /Length {} >>\nstream\n{content}\nendstream",
            content.len()
        ));
    }
    let mut out = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for (i, o) in objs.iter().enumerate() {
        offsets.push(out.len());
        out.extend(format!("{} 0 obj\n{o}\nendobj\n", i + 1).bytes());
    }
    let xref = out.len();
    out.extend(format!("xref\n0 {}\n0000000000 65535 f \n", objs.len() + 1).bytes());
    for off in offsets {
        out.extend(format!("{off:010} 00000 n \n").bytes());
    }
    out.extend(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objs.len() + 1
        )
        .bytes(),
    );
    out
}

const LETTER: &str = "[0 0 612 792]";

/// Q2-06: one slide per page, in page order.
#[test]
fn pdf_pages_become_slides_in_order() {
    let bytes = pdf(
        &[
            "Amazing grace how sweet the sound",
            "Twas grace that taught my heart to fear",
            "When we have been there ten thousand years",
        ],
        "F1",
        LETTER,
    );
    let song = voxxa_lib::pdf_parse(&bytes, "Amazing Grace").unwrap();
    let texts: Vec<&str> = song.slides.iter().map(|s| s.text.as_str()).collect();
    assert_eq!(texts.len(), 3, "{texts:?}");
    assert!(texts[0].contains("Amazing grace"));
    assert!(texts[1].contains("taught my heart"));
    assert!(texts[2].contains("ten thousand years"));
}

/// Q2-05/Q2-06: an empty page stays a slide so slide numbers match the deck.
#[test]
fn pdf_empty_page_is_kept_as_empty_slide() {
    let bytes = pdf(
        &["Be thou my vision", "", "Riches I heed not"],
        "F1",
        LETTER,
    );
    let song = voxxa_lib::pdf_parse(&bytes, "T").unwrap();
    assert_eq!(song.slides.len(), 3);
    assert!(song.slides[1].text.is_empty());
    assert!(song.slides[2].text.contains("Riches"));
}

/// Q2-01: malformed PDFs that make pdf-extract panic return an error instead.
#[test]
fn malformed_pdfs_return_err_instead_of_panicking() {
    let unknown_font = pdf(&["Holy holy holy"], "F9", LETTER);
    let short_media_box = pdf(&["Holy holy holy"], "F1", "[0 0 612]");
    for bytes in [unknown_font, short_media_box] {
        assert!(voxxa_lib::pdf_parse(&bytes, "T").is_err());
    }
}

fn example_conductor() -> voxxa_lib::Conductor {
    let json = std::fs::read_to_string(manifest_dir().join("../setlist.example.json")).unwrap();
    let setlist: voxxa_lib::Setlist = serde_json::from_str(&json).unwrap();
    voxxa_lib::Conductor::new(setlist.setlist, voxxa_lib::SmartConfig::default())
}

const AMAZING_GRACE_V1: &str = "amazing grace how sweet the sound that saved a wretch like me \
    i once was lost but now am found was blind but now i see";
const MS: fn(u64) -> std::time::Duration = std::time::Duration::from_millis;

/// Q2-02 (audit 1): after a song and a blank, a prayer must not bring the lyrics back.
#[test]
fn prayer_after_blank_does_not_reshow_lyrics() {
    use voxxa_lib::Action;
    let mut c = example_conductor();
    let t0 = std::time::Instant::now();
    assert!(matches!(
        c.on_transcript(AMAZING_GRACE_V1, t0),
        Action::Goto { .. }
    ));
    let blanked = (1..=50).any(|i| matches!(c.tick(t0 + MS(200 * i)), Action::Blank));
    assert!(blanked && c.is_blank(), "10 s of silence blanks");
    let action = c.on_transcript(
        "let us bow our heads and pray father we thank you for this morning",
        t0 + MS(11_000),
    );
    assert!(matches!(action, Action::Noop), "got {action:?}");
    assert!(c.is_blank());
}

/// Q2-02 (audit 1): continuous speech that matches no song blanks the screen.
#[test]
fn continuous_unmatched_speech_blanks() {
    use voxxa_lib::Action;
    let sermon = [
        "good morning church it is so good to see every one of you here today",
        "before we begin please turn to the person next to you and say hello",
        "this week the youth group meets on wednesday at six in the fellowship hall",
        "open your bibles with me to the gospel of mark chapter four verse thirty five",
        "jesus and his disciples got into a boat and a great storm came up on the lake",
    ];
    let mut c = example_conductor();
    let t0 = std::time::Instant::now();
    c.on_transcript(AMAZING_GRACE_V1, t0);
    let first_unmatched = 5_000;
    let limit =
        (voxxa_lib::SmartConfig::default().unrecognized_speech_to_blank_secs * 1000.0) as u64;
    let mut blank_at = None;
    for ms in (200..30_000).step_by(200) {
        let now = t0 + MS(ms);
        c.on_speech(now);
        if ms % 5_000 == 0 {
            c.on_transcript(sermon[(ms / 5_000) as usize % sermon.len()], now);
        }
        if matches!(c.tick(now), Action::Blank) {
            blank_at.get_or_insert(ms);
        }
    }
    let blank_at = blank_at.expect("unmatched speech never blanked");
    assert!(
        blank_at <= first_unmatched + limit + 200,
        "blanked at {blank_at} ms"
    );
    assert!(c.is_blank());
}
