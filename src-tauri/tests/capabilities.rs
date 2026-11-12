//! Q2-09 (audit 7): without a capability file Tauri rejects every
//! `event.listen`, so the operator view, Stage Display, download progress and
//! the device-loss notice never update. The file grants exactly the event API
//! to both windows and nothing broader.

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
