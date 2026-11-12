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
