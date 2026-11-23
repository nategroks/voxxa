//! Q10 (founder decisions 2026-11-23, ADR-008, ADR-020): build.yml makes a
//! draft release, marks a tag with "-" as a pre-release, fills every winget
//! placeholder, and uploads the NSIS installer by a glob that matches it.
//! Text checks only: the workflow itself runs on GitHub.

const BUILD: &str = include_str!("../../.github/workflows/build.yml");
const WINGET: [&str; 3] = [
    include_str!("../../packaging/winget/nategroks.Voxxa.yaml"),
    include_str!("../../packaging/winget/nategroks.Voxxa.installer.yaml"),
    include_str!("../../packaging/winget/nategroks.Voxxa.locale.en-US.yaml"),
];

#[test]
fn releases_are_drafts_and_a_dash_tag_is_a_pre_release() {
    assert!(BUILD.contains("-F draft=true"));
    assert!(BUILD.contains(r#"case "$TAG" in *-*) pre=true ;; esac"#));
    for never in ["draft=false", "releaseDraft: false", "tagName:"] {
        assert!(!BUILD.contains(never), "{never}");
    }
}

#[test]
fn the_runners_and_upload_globs_exist() {
    assert!(!BUILD.contains("macos-13"), "retired runner");
    assert!(!BUILD.contains("*.nsis"), "matches nothing");
    assert!(BUILD.contains("release/bundle/nsis/*-setup.exe"));
}

#[test]
fn every_winget_placeholder_is_filled_by_the_release_step() {
    for text in WINGET {
        for part in text.split("{{").skip(1) {
            let name = part.split("}}").next().unwrap();
            assert!(
                BUILD.contains(&format!("s|{{{{{name}}}}}|")),
                "{name} is never filled"
            );
        }
    }
    let installer = WINGET[1];
    assert!(installer.contains("InstallerType: nullsoft") && installer.contains("Scope: user"));
}
