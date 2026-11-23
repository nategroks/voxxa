//! ADR-020 §§ 2, 4 and 6: what `tauri.conf.json` must keep. The CSP stays
//! strict, the updater can never be told to accept a downgrade or plain HTTP,
//! and the Windows installers are per-user NSIS with the WebView2 bootstrapper
//! inside. A build without the updater key must still succeed, so
//! `createUpdaterArtifacts` is never on in the committed file (the release
//! workflow turns it on only when the key exists).

fn conf() -> serde_json::Value {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tauri.conf.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// `directive -> sources` of a CSP string.
fn directives(csp: &str) -> Vec<(String, Vec<String>)> {
    (csp.split(';'))
        .map(str::trim)
        .filter(|d| !d.is_empty())
        .map(|d| {
            let mut words = d.split_whitespace().map(str::to_string);
            (words.next().unwrap(), words.collect())
        })
        .collect()
}

fn sources<'a>(csp: &'a [(String, Vec<String>)], name: &str) -> &'a [String] {
    (csp.iter())
        .find(|(d, _)| d == name)
        .map(|(_, s)| s.as_slice())
        .unwrap_or_else(|| panic!("the CSP has no {name}"))
}

#[test]
fn the_csp_is_strict() {
    let conf = conf();
    let csp = conf["app"]["security"]["csp"]
        .as_str()
        .expect("app.security.csp must be a string, not null");
    let csp = directives(csp);
    for unsafe_word in [
        "'unsafe-eval'",
        "'unsafe-inline'",
        "'unsafe-hashes'",
        "*",
        "data:",
        "blob:",
        "http:",
        "https:",
    ] {
        for (name, list) in &csp {
            assert!(
                !list.iter().any(|s| s == unsafe_word),
                "{name} allows {unsafe_word}"
            );
        }
    }
    for name in [
        "default-src",
        "script-src",
        "style-src",
        "img-src",
        "font-src",
    ] {
        assert_eq!(sources(&csp, name), ["'self'"], "{name}");
    }
    // Tauri's IPC only: the page itself never reaches the network (ADR-020 § 4).
    assert_eq!(
        sources(&csp, "connect-src"),
        ["ipc:", "http://ipc.localhost"]
    );
    assert_eq!(sources(&csp, "object-src"), ["'none'"]);
}

#[test]
fn the_dev_csp_differs_only_for_vite() {
    let conf = conf();
    let dev = conf["app"]["security"]["devCsp"].as_str().expect("devCsp");
    let dev = directives(dev);
    assert!(!(dev.iter()).any(|(_, l)| l.iter().any(|s| s == "'unsafe-eval'")));
    assert_eq!(sources(&dev, "script-src"), ["'self'"]);
    assert_eq!(
        sources(&dev, "connect-src"),
        ["ipc:", "http://ipc.localhost", "ws://localhost:5173"]
    );
}

#[test]
fn the_updater_refuses_downgrades_and_plain_http() {
    let conf = conf();
    let updater = conf["plugins"]["updater"].as_object().expect("updater");
    for key in updater.keys() {
        assert!(
            !key.starts_with("dangerous") && key != "allowDowngrades",
            "plugins.updater.{key} must not be set (ADR-020 § 2)"
        );
    }
    let endpoints = updater["endpoints"].as_array().unwrap();
    assert!(!endpoints.is_empty());
    for e in endpoints {
        let e = e.as_str().unwrap();
        assert!(
            e.starts_with("https://github.com/nategroks/voxxa/releases/"),
            "{e}"
        );
        // "latest" never points at a draft or a pre-release (ADR-020 § 1).
        assert!(e.contains("/releases/latest/download/"), "{e}");
    }
    let pubkey = updater["pubkey"].as_str().unwrap();
    assert!(pubkey.len() > 100, "the minisign public key is missing");
    assert_eq!(updater["windows"]["installMode"], "passive");
}

#[test]
fn windows_installers_are_per_user_nsis_with_the_webview2_bootstrapper() {
    let conf = conf();
    let win = &conf["bundle"]["windows"];
    assert_eq!(win["nsis"]["installMode"], "currentUser");
    assert_eq!(win["webviewInstallMode"]["type"], "embedBootstrapper");
    assert_eq!(win["webviewInstallMode"]["silent"], true);
    assert_eq!(win["digestAlgorithm"], "sha256");
    assert_eq!(win["tsp"], true, "RFC 3161 timestamps");
    // No certificate on disk, no signing command in the committed file: the
    // release workflow injects one only when the Azure secrets exist.
    assert!(win["certificateThumbprint"].is_null());
    assert!(win.get("signCommand").is_none());
}

#[test]
fn a_build_without_the_updater_key_still_succeeds() {
    // The Tauri CLI fails a build with createUpdaterArtifacts on and no
    // TAURI_SIGNING_PRIVATE_KEY; build.yml injects it only when the key exists.
    let conf = conf();
    let on = &conf["bundle"]["createUpdaterArtifacts"];
    assert!(on.is_null() || on == false, "createUpdaterArtifacts: {on}");
}
