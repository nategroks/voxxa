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

/// The text of a workflow file (no YAML parser in the tree: plain text checks,
/// like `media_guard.rs` reads ci.yml).
fn workflow(name: &str) -> String {
    let path = format!("{}/../.github/workflows/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(path).unwrap().replace("\r\n", "\n")
}

#[test]
fn tag_builds_make_draft_releases_only() {
    let build = workflow("build.yml");
    assert!(
        build.contains("gh release create \"$TAG\""),
        "the draft job"
    );
    assert!(
        build.contains("args=(--draft "),
        "every release is created as a draft"
    );
    assert!(
        build.contains("args+=(--prerelease)"),
        "a tag with a - is a pre-release"
    );
    assert!(
        !build.contains("releaseDraft: false"),
        "tauri-action must not publish"
    );
    assert!(
        !build.contains("gh release edit"),
        "nothing publishes the draft"
    );
    assert!(build.contains("a published release is never rebuilt"));
    assert!(build.contains("git merge-base --is-ancestor \"$GITHUB_SHA\" origin/main"));
}

#[test]
fn windows_is_signed_before_its_updater_signatures_and_skips_cleanly() {
    let build = workflow("build.yml");
    let at = |needle: &str| {
        build
            .find(needle)
            .unwrap_or_else(|| panic!("build.yml lacks {needle:?}"))
    };
    // SignPath (preferred) and Azure (fallback) both come before the minisign step.
    let signpath = at("uses: signpath/github-action-submit-signing-request@");
    let azure = at("cargo install trusted-signing-cli --locked --version 0.11.0");
    let check = at("Get-AuthenticodeSignature");
    let minisign = at("npx tauri signer sign \"$f\"");
    let upload = at("gh release upload \"$TAG\" dist-win/* --clobber");
    assert!(azure < signpath && signpath < check && check < minisign && minisign < upload);
    // Each path runs only when chosen; without secrets the job warns and goes on.
    assert!(build.contains("if: steps.path.outputs.signing == 'signpath'"));
    assert!(build.contains("if: steps.path.outputs.signing == 'azure'"));
    assert!(build.contains("if: steps.path.outputs.updater == 'yes'"));
    assert!(build.contains("signing=none"));
    // createUpdaterArtifacts only through the override, only with the key.
    assert!(build.contains("if [ -n \"$TAURI_SIGNING_PRIVATE_KEY\" ]; then\n            echo '{\"bundle\":{\"createUpdaterArtifacts\":true}}'"));
    assert!(
        build.contains("includeUpdaterJson: false"),
        "latest.json has one writer"
    );
}

#[test]
fn the_build_matrix_and_globs_are_current() {
    let build = workflow("build.yml");
    assert!(!build.contains("platform: macos-13"), "retired runner");
    assert!(build.contains("platform: macos-15-intel"));
    assert!(
        !build.contains("*.nsis"),
        "matches nothing; NSIS is *-setup.exe"
    );
    assert!(build.contains("\"$BUNDLE\"/nsis/*-setup.exe \"$BUNDLE\"/msi/*.msi"));
    assert!(build.contains("toolchain: 1.99.1"), "as ci.yml");
    assert!(
        build.contains("CMAKE_PROJECT_INCLUDE:"),
        "portable ggml, #142"
    );
}

#[test]
fn winget_is_submitted_only_for_a_published_full_release_with_a_token() {
    let winget = workflow("winget.yml");
    assert!(winget.contains("types: [published]"));
    assert!(winget.contains("isDraft,isPrerelease"));
    assert!(winget.contains("if (-not $env:WINGET_TOKEN)"));
    assert!(winget.contains("winget-create/releases/download/v1.9.14.0/wingetcreate.exe"));
    assert!(
        winget.contains("Get-FileHash"),
        "the tool is pinned by hash"
    );
}
