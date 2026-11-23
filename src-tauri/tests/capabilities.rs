//! Q2-09 (audit 7): without a capability file Tauri rejects every
//! `event.listen`, so the operator view, Stage Display, download progress and
//! the device-loss notice never update. The file grants exactly the event API
//! to both windows and nothing broader. The stage window config is checked
//! here too (Q2-04).

#[test]
fn both_windows_may_listen_to_events_and_nothing_more() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/capabilities/default.json");
    let text = std::fs::read_to_string(path).expect("capabilities/default.json is missing");
    let cap: serde_json::Value = serde_json::from_str(&text).unwrap();
    let list = |key: &str| -> Vec<String> {
        cap[key]
            .as_array()
            .unwrap_or_else(|| panic!("{key} must be a list"))
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect()
    };
    let mut windows = list("windows");
    windows.sort();
    assert_eq!(windows, ["main", "stage"]);
    assert_eq!(list("permissions"), ["core:event:default"]);
}

/// Tauri loads every file under capabilities/, and any capability inlined in
/// tauri.conf.json. A second file granting, say, "updater:default" would give
/// the webview the updater plugin directly, past Voxxa's comparator and the
/// listening rule (ADR-020 § 3). So default.json is the only grant there is.
#[test]
fn default_json_is_the_only_capability() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/capabilities");
    let mut names: Vec<String> = (std::fs::read_dir(dir).unwrap())
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(names, ["default.json"], "files in {dir}");
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tauri.conf.json");
    let conf: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let inline = &conf["app"]["security"]["capabilities"];
    assert!(inline.is_null(), "app.security.capabilities: {inline}");
}

/// Q2-04 (audit 3): the Stage Display is view-only and must never take the
/// keyboard from the presentation app, where the keystroke driver types.
/// Showing it goes through `commands::show_stage` (no activation).
#[test]
fn the_stage_window_never_takes_the_keyboard() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tauri.conf.json");
    let conf: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let stage = conf["app"]["windows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|w| w["label"] == "stage")
        .expect("stage window");
    // Without "focus": false, WebView2 takes the keyboard into the hidden
    // stage at launch.
    assert_eq!(stage["focus"], false, "no focus at launch");
    assert_eq!(stage["focusable"], false, "a click must not take the keys");
    assert_eq!(stage["visible"], false, "hidden until the operator asks");
}
