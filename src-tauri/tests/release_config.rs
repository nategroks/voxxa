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
    // An empty id makes tauri-action skip every upload ("No releaseId or
    // tagName provided"); the draft job must fail instead (rc.1 run 45258000510).
    assert!(
        build.contains("if ! [[ \"${id:-}\" =~ ^[0-9]+$ ]]; then"),
        "the draft job refuses an empty release id"
    );
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
    // Azure only when all six values exist; a partial setup warns and goes unsigned.
    assert!(build.contains("count AZURE_CLIENT_ID AZURE_CLIENT_SECRET AZURE_TENANT_ID AZURE_CODE_SIGNING_ENDPOINT AZURE_CODE_SIGNING_ACCOUNT AZURE_CODE_SIGNING_CERTIFICATE_PROFILE\n            if [ \"$found\" -eq 6 ]; then\n              signing=azure"));
    assert!(build.contains("Azure Trusted Signing is only partly configured (missing:$missing)"));
    // createUpdaterArtifacts only through the override, only with the key.
    assert!(build.contains("if [ -n \"$TAURI_SIGNING_PRIVATE_KEY\" ]; then\n            echo '{\"bundle\":{\"createUpdaterArtifacts\":true}}'"));
    assert!(
        build.contains("includeUpdaterJson: false"),
        "latest.json has one writer"
    );
    // The latest.json job learns only whether the key exists, never the key.
    assert!(build.contains("      updater: ${{ steps.path.outputs.updater }}"));
    let manifest_job = &build[at("  updater-manifest:")..];
    assert!(manifest_job.contains("if: needs.windows.outputs.updater == 'yes'"));
    assert!(
        !manifest_job.contains("TAURI_SIGNING_PRIVATE_KEY:"),
        "the private key must not be in the latest.json job"
    );
}

/// The text of one step of a workflow, from its `- name:` line to the next.
fn step<'a>(workflow: &'a str, name: &str) -> &'a str {
    let head = format!("      - name: {name}\n");
    let start = (workflow.find(&head)).unwrap_or_else(|| panic!("no step {name:?}"));
    let rest = &workflow[start + head.len()..];
    let end = rest.find("\n      - name: ").map_or(rest.len(), |i| i + 1);
    &workflow[start..start + head.len() + end]
}

#[test]
fn macos_builds_unsigned_when_the_apple_secrets_are_missing() {
    // The Tauri CLI reads APPLE_SIGNING_IDENTITY whenever the variable exists,
    // even empty, and it overrides tauri.conf.json's ad-hoc identity "-": an
    // empty identity reaches codesign and fails the macOS jobs. So no APPLE_*
    // value is set on the build step; the step before exports each one only
    // when its secret is not empty.
    let build = workflow("build.yml");
    let tauri = step(&build, "Build Tauri app");
    assert!(tauri.contains("uses: tauri-apps/tauri-action@"), "{tauri}");
    assert!(
        !tauri.contains("APPLE_"),
        "APPLE_* on the build step:\n{tauri}"
    );
    let values = step(&build, "Apple signing values");
    assert!(values.contains("if: runner.os == 'macOS'"), "{values}");
    assert!(values.contains("shell: bash"), "${{!name}} is bash");
    let names = "APPLE_SIGNING_IDENTITY APPLE_API_ISSUER APPLE_API_KEY APPLE_TEAM_ID";
    assert!(
        values.contains(&format!("for name in {names}; do")),
        "{values}"
    );
    assert!(
        values.contains("if [ -n \"${!name}\" ]; then\n              echo \"$name=${!name}\" >> \"$GITHUB_ENV\""),
        "{values}"
    );
    for name in names.split(' ') {
        assert!(
            values.contains(&format!("{name}: ${{{{ secrets.")),
            "{name}"
        );
    }
    assert!(
        build.find("- name: Apple signing values") < build.find("- name: Build Tauri app"),
        "the values are exported before the build"
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
fn windows_installers_build_whisper_cpp_with_o2() {
    // #59 (ADR-021 § 1): the cmake crate writes the MSVC release flags without
    // /O2. Two env lines in the Windows job only (-Ob2 breaks gcc and clang), a
    // new cache key (a cached unoptimised whisper-rs-sys build is never rebuilt),
    // and the CMakeCache check right after the installer build.
    let build = workflow("build.yml");
    let at = |needle: &str| {
        build
            .find(needle)
            .unwrap_or_else(|| panic!("build.yml lacks {needle:?}"))
    };
    let windows = &build[at("\n  windows:\n")..at("\n  updater-manifest:\n")];
    for var in ["CMAKE_C_FLAGS_RELEASE", "CMAKE_CXX_FLAGS_RELEASE"] {
        let line = format!("\n      {var}: -MD -O2 -Ob2 -DNDEBUG\n");
        assert!(
            windows.contains(&line),
            "the Windows job's env lacks {line:?}"
        );
        assert_eq!(
            build.matches(&format!("{var}:")).count(),
            1,
            "{var} must be set in the Windows job's env only"
        );
    }
    assert!(
        windows.contains("\n          key: release-portable-ggml-o2\n"),
        "the Windows cache key went back to one that may hold an unoptimised build"
    );
    let built = step(&build, "Build the installers (NSIS per-user and MSI)");
    let check = "      - name: Check that whisper.cpp was built with -O2\n";
    assert!(
        build[at(built) + built.len()..].starts_with(check),
        "the flag check must come right after the installer build"
    );
    let check = step(&build, "Check that whisper.cpp was built with -O2");
    assert!(
        check.contains("        shell: bash\n")
            && check.contains(
                "run: bash scripts/ci/check-msvc-release-flags.sh \"src-tauri/target/$TARGET/release\"\n"
            ),
        "{check}"
    );
    assert!(
        !check.contains("if:") && !check.contains("continue-on-error"),
        "{check}"
    );
}

#[test]
fn release_actions_are_pinned_to_full_commit_shas() {
    // These jobs hold the updater key, the signing secrets and a
    // contents:write token: a moved tag must not change what runs (Q2-28,
    // ADR-020 § 6). Every `uses:` is `owner/repo@<40 hex> # <version>`.
    for name in ["build.yml", "winget.yml"] {
        let text = workflow(name);
        let uses: Vec<&str> = (text.lines())
            .map(|l| l.trim_start().trim_start_matches("- "))
            .filter_map(|l| l.strip_prefix("uses: "))
            .collect();
        assert!(uses.len() > 1, "{name}: no uses: found");
        for u in uses {
            let reference = u.split_once('@').map_or("", |(_, r)| r);
            let (sha, comment) = reference.split_once(" # ").unwrap_or((reference, ""));
            assert!(
                sha.len() == 40 && sha.bytes().all(|b| b.is_ascii_hexdigit()),
                "{name}: {u} is not pinned to a commit SHA"
            );
            assert!(!comment.trim().is_empty(), "{name}: {u} lacks # version");
        }
    }
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
