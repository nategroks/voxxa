//! Q7 (user decision 2026-11-18): the church installers carry the app only,
//! not the developer tools `voxxad` and `voxxa-eval`.
//!
//! tauri-cli 2.10 picks the binaries to bundle in `get_binaries`
//! (tauri-cli `src/interface/rust.rs`): every `[[bin]]` entry whose
//! `required-features` are all enabled for `tauri build`, plus every entry
//! of `src/bin/` it has not seen yet. So a tool is left out only when it
//! needs a feature that `tauri build` does not enable AND it lives outside
//! `src/bin/`. These checks keep both conditions true. The proof that the
//! NSIS installer then lists no tool is in the Q7 DevOps record.

use std::path::Path;

const MANIFEST: &str = include_str!("../Cargo.toml");

/// The `key = value` lines of each `[[bin]]` table, in file order.
fn bin_tables() -> Vec<Vec<(String, String)>> {
    let mut tables = Vec::new();
    let mut current: Option<Vec<(String, String)>> = None;
    for line in MANIFEST.lines().map(str::trim) {
        if line.starts_with('[') {
            tables.extend(current.take());
            if line == "[[bin]]" {
                current = Some(Vec::new());
            }
        } else if let (Some(table), Some((key, value))) = (&mut current, line.split_once('=')) {
            table.push((key.trim().to_string(), value.trim().to_string()));
        }
    }
    tables.extend(current);
    tables
}

fn get<'a>(table: &'a [(String, String)], key: &str) -> Option<&'a str> {
    (table.iter())
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}

#[test]
fn both_tools_need_the_cli_feature_and_live_outside_src_bin() {
    let tables = bin_tables();
    for tool in ["\"voxxad\"", "\"voxxa-eval\""] {
        let table = (tables.iter())
            .find(|t| get(t, "name") == Some(tool))
            .unwrap_or_else(|| panic!("no [[bin]] entry for {tool}"));
        assert_eq!(
            get(table, "required-features"),
            Some(r#"["cli"]"#),
            "{tool}"
        );
        let path = get(table, "path").unwrap_or_else(|| panic!("{tool} has no path"));
        assert!(path.starts_with("\"src/cli/"), "{tool}: {path}");
    }
    // The app itself needs no feature: it is the installer's main binary.
    let app = (tables.iter())
        .find(|t| get(t, "name") == Some("\"voxxa\""))
        .expect("the app's [[bin]] entry");
    assert_eq!(get(app, "required-features"), None);
}

#[test]
fn the_cli_feature_is_off_by_default() {
    let features = (MANIFEST.split("\n[features]").nth(1))
        .expect("a [features] table")
        .split("\n[")
        .next()
        .unwrap();
    let default = (features.lines())
        .map(str::trim)
        .find(|l| l.starts_with("default"))
        .expect("a default feature list");
    assert!(!default.contains("cli"), "{default}");
}

#[test]
fn nothing_lives_in_src_bin() {
    // tauri-cli bundles every entry of src/bin/ that no [[bin]] entry
    // already names, whatever its features.
    let bin = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("bin");
    assert!(!bin.exists(), "{} must not exist", bin.display());
}
