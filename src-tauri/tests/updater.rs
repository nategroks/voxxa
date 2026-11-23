//! ADR-020 §§ 1-2: the update comparator and the listening rule, as pure
//! functions. No network: nothing here builds an updater or an app. Where the
//! rules are enforced (the two commands, and that nothing else reaches the
//! plugin) is checked on the source text, as `release_config.rs` reads the
//! workflows: those paths need an app and the network to run.

use std::path::Path;

use voxxa_lib::updater::{check_error, refused, should_update, Outcome};

/// The code lines of a source file: CRLF folded, `//` comment lines dropped.
fn code(path: &Path) -> String {
    let text = (std::fs::read_to_string(path)).unwrap_or_else(|e| panic!("{path:?}: {e}"));
    (text.lines())
        .filter(|l| !l.trim_start().starts_with("//"))
        .map(|l| format!("{l}\n"))
        .collect()
}

fn src_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// `fn` from `signature` to its closing `}` at column 0.
fn body<'a>(src: &'a str, signature: &str) -> &'a str {
    let start = (src.find(signature)).unwrap_or_else(|| panic!("no {signature:?}"));
    let len = src[start..].find("\n}\n").expect("closing brace") + 3;
    &src[start..start + len]
}

/// Where `needle` first appears in `text` at or after `from`.
fn at(text: &str, needle: &str, from: usize) -> usize {
    (text[from..].find(needle)).unwrap_or_else(|| panic!("no {needle:?} after {from} in:\n{text}"))
        + from
}

const LISTENING_GUARD: &str =
    "if let Some(no) = refused(state.sessions.is_running()) {\n        return Ok(no);\n    }";

#[test]
fn both_commands_refuse_while_listening_before_the_network() {
    let src = code(&src_dir().join("updater/mod.rs"));
    // "Check for updates": the guard, then the one request.
    let check = body(&src, "pub async fn check_for_update(");
    assert!(
        at(check, LISTENING_GUARD, 0) < at(check, "latest(&app)", 0),
        "check_for_update must refuse before latest(&app):\n{check}"
    );
    // "Install and restart": the guard before the request, and again after the
    // download, before the installer can close Voxxa during a service.
    let install = body(&src, "pub async fn install_update(");
    let first = at(install, LISTENING_GUARD, 0);
    let request = at(install, "latest(&app)", 0);
    let download = at(install, ".download(", 0);
    let second = at(install, LISTENING_GUARD, download);
    let installs = at(install, "update.install(", 0);
    assert!(
        first < request && request < download && download < second && second < installs,
        "install_update must refuse before the request and before installing:\n{install}"
    );
}

/// Every `.rs` file under `dir`, recursively.
fn rust_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn nothing_outside_the_updater_module_reaches_the_updater() {
    // ADR-020 § 1: never at startup, never on a timer. Only src/updater/ may
    // touch the plugin; elsewhere the two commands appear only where lib.rs
    // registers them in generate_handler!.
    let mut files = Vec::new();
    rust_files(&src_dir(), &mut files);
    let updater_dir = src_dir().join("updater");
    let needles = [
        "tauri_plugin_updater",
        "UpdaterExt",
        "updater_builder",
        "updater(",
        "check_for_update",
        "install_update",
        "updater::latest",
    ];
    let registrations = ["updater::check_for_update,", "updater::install_update,"];
    let mut checked = 0;
    for file in files.iter().filter(|f| !f.starts_with(&updater_dir)) {
        let is_lib = file.ends_with("lib.rs") && file.parent() == Some(src_dir().as_path());
        for line in code(file).lines() {
            let line = line.trim();
            if is_lib && registrations.contains(&line) {
                continue;
            }
            for needle in needles {
                assert!(!line.contains(needle), "{file:?}: {line}");
            }
        }
        checked += 1;
    }
    assert!(checked > 20, "only {checked} files read");
    let lib = code(&src_dir().join("lib.rs"));
    let handler = &lib[at(&lib, "tauri::generate_handler![", 0)..];
    let handler = &handler[..at(handler, "])", 0)];
    for r in registrations {
        assert_eq!(lib.matches(r).count(), 1, "{r}");
        assert!(handler.contains(r), "{r} outside generate_handler!");
    }
}

#[test]
fn only_the_two_commands_make_the_request() {
    let src = code(&src_dir().join("updater/mod.rs"));
    let latest = body(&src, "async fn latest(");
    // The plugin's one check and its builder live in latest() alone.
    for needle in [".check(", "updater_builder", ".updater("] {
        assert_eq!(
            src.matches(needle).count(),
            latest.matches(needle).count(),
            "{needle}"
        );
    }
    assert_eq!(latest.matches(".check(").count(), 1);
    // latest() is called from the two commands only; the download, the
    // install and the restart happen in install_update only.
    let check = body(&src, "pub async fn check_for_update(");
    let install = body(&src, "pub async fn install_update(");
    let calls = check.matches("latest(").count() + install.matches("latest(").count();
    assert_eq!(src.matches("latest(").count(), 1 + calls);
    assert_eq!(check.matches("latest(&app)").count(), 1);
    assert_eq!(install.matches("latest(&app)").count(), 1);
    for needle in [".download(", ".install(", "restart()"] {
        assert_eq!(src.matches(needle).count(), 1, "{needle}");
        assert_eq!(install.matches(needle).count(), 1, "{needle}");
    }
    // Neither command is called from inside the module either.
    for name in ["check_for_update", "install_update"] {
        assert_eq!(
            src.matches(name).count(),
            1,
            "{name} besides its definition"
        );
    }
}

#[test]
fn only_a_strictly_newer_version_is_an_update() {
    assert!(should_update("0.1.0", "0.1.1"));
    assert!(should_update("0.1.0", "0.2.0"));
    assert!(should_update("0.9.9", "1.0.0"));
    assert!(
        should_update("1.2.3", "1.10.0"),
        "numeric, not string, order"
    );
}

#[test]
fn the_same_version_is_no_update() {
    assert!(!should_update("0.1.0", "0.1.0"));
    assert!(!should_update("1.0.0-rc.1", "1.0.0-rc.1"));
}

#[test]
fn a_lower_version_is_refused() {
    assert!(!should_update("0.2.0", "0.1.9"));
    assert!(!should_update("1.0.0", "0.99.99"));
    assert!(!should_update("1.10.0", "1.9.0"));
    assert!(
        !should_update("0.1.0", "0.1.0-rc.1"),
        "a release is newer than its rc"
    );
}

#[test]
fn pre_releases_order_by_semver() {
    assert!(should_update("0.1.0-rc.1", "0.1.0"));
    assert!(should_update("0.1.0-rc.1", "0.1.0-rc.2"));
    assert!(
        should_update("0.1.0-rc.9", "0.1.0-rc.10"),
        "numeric identifiers"
    );
    assert!(should_update("0.1.0-alpha", "0.1.0-beta"));
    assert!(should_update("0.1.0-alpha.1", "0.1.0-alpha.beta"));
    assert!(should_update("0.1.0", "0.1.1-rc.1"));
    assert!(!should_update("0.1.0-rc.2", "0.1.0-rc.1"));
}

#[test]
fn build_metadata_alone_is_no_update() {
    // semver 2.0 § 10: build metadata does not count for precedence, though
    // semver::Version's Ord would rank 0.1.0+b2 above 0.1.0+b1.
    assert!(!should_update("0.1.0", "0.1.0+build.2"));
    assert!(!should_update("0.1.0+build.2", "0.1.0+build.10"));
    assert!(!should_update("0.1.0+zzz", "0.1.0"));
    assert!(should_update("0.1.0+build.9", "0.1.1"));
}

#[test]
fn a_malformed_version_is_refused() {
    for bad in [
        "",
        "latest",
        "v0.2.0",
        "0.2",
        "0.2.0.1",
        "01.2.0",
        "0.2.0-",
        " 0.2.0",
        "9999999999999999999999.0.0",
    ] {
        assert!(!should_update("0.1.0", bad), "remote {bad:?}");
        assert!(!should_update(bad, "0.2.0"), "installed {bad:?}");
    }
}

#[test]
fn listening_refuses_before_any_network_call() {
    assert_eq!(refused(true), Some(Outcome::Listening));
    assert_eq!(refused(false), None);
}

#[test]
fn a_release_not_found_is_neither_up_to_date_nor_a_network_failure() {
    // The plugin returns ReleaseNotFound for any non-success status: 404
    // before the first full release, but also 5xx and a rate-limit 403.
    let text = check_error(tauri_plugin_updater::Error::ReleaseNotFound);
    assert_eq!(
        text,
        "No update is published yet, or GitHub did not answer. Try again later."
    );
    assert!(
        !text.starts_with("Could not"),
        "no internet hint in the GUI"
    );
    let other = check_error(tauri_plugin_updater::Error::EmptyEndpoints);
    assert!(
        other.starts_with("Could not check for updates: "),
        "{other}"
    );
}

#[test]
fn the_gui_reads_a_status_field() {
    let json = |o: &Outcome| serde_json::to_value(o).unwrap();
    assert_eq!(
        json(&Outcome::Listening),
        serde_json::json!({"status": "listening"})
    );
    assert_eq!(
        json(&Outcome::UpToDate {
            current: "0.1.0".into()
        }),
        serde_json::json!({"status": "up_to_date", "current": "0.1.0"})
    );
    assert_eq!(
        json(&Outcome::Available {
            current: "0.1.0".into(),
            version: "0.2.0".into(),
        }),
        serde_json::json!({"status": "available", "current": "0.1.0", "version": "0.2.0"})
    );
}
