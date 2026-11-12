//! ADR-007: Voxxa ships no copyrighted lyrics, and that includes test fixtures.
//! The conductor tests once used lines from two CCLI-licensed worship songs;
//! they now use public-domain hymns. This test keeps those titles out of
//! src-tauri/ for good. The needles are split so this file does not match itself.

use std::fs;
use std::path::Path;

fn scan(dir: &Path, needles: &[String], hits: &mut Vec<String>) {
    for entry in fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            // Build output (CI keeps it in src-tauri/target) and generated schemas.
            if name != "target" && name != "gen" && !name.starts_with('.') {
                scan(&path, needles, hits);
            }
            continue;
        }
        let Ok(bytes) = fs::read(&path) else { continue };
        let text = String::from_utf8_lossy(&bytes).to_lowercase();
        for needle in needles {
            if text.contains(needle.as_str()) {
                hits.push(format!("{} contains \"{needle}\"", path.display()));
            }
        }
    }
}

#[test]
fn no_copyrighted_song_titles_in_src_tauri() {
    let needles = [
        ["way", " maker"].concat(),
        ["how great", " is our god"].concat(),
    ];
    let mut hits = Vec::new();
    scan(Path::new(env!("CARGO_MANIFEST_DIR")), &needles, &mut hits);
    assert!(hits.is_empty(), "ADR-007 fixtures: {hits:#?}");
}
