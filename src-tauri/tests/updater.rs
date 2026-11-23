//! Q10 / ADR-020: the updater's version comparator and its listening rule,
//! as pure functions. No network, no window: the commands themselves call
//! these before anything else.

use voxxa_lib::updater::{is_update, refuse_while_listening, LISTENING};

#[test]
fn only_a_strictly_newer_version_is_an_update() {
    assert!(is_update("0.1.0", "0.1.1"));
    assert!(is_update("0.1.0", "0.2.0"));
    assert!(is_update("0.9.9", "1.0.0"));
    assert!(!is_update("0.1.0", "0.1.0"), "equal is no update");
    assert!(!is_update("0.2.0", "0.1.9"), "lower is refused");
    assert!(!is_update("1.0.0", "0.99.99"), "lower is refused");
}

#[test]
fn pre_releases_follow_semver_precedence() {
    assert!(is_update("0.1.0-rc.1", "0.1.0"), "rc.1 < release");
    assert!(is_update("0.1.0-rc.1", "0.1.0-rc.2"));
    assert!(is_update("0.1.0-rc.9", "0.1.0-rc.10"), "numeric, not text");
    assert!(is_update("0.1.0-alpha", "0.1.0-beta"));
    assert!(
        !is_update("0.1.0", "0.1.0-rc.1"),
        "a release is never downgraded to its rc"
    );
    assert!(!is_update("0.1.0-rc.2", "0.1.0-rc.1"));
    assert!(
        is_update("0.1.0", "0.1.1-rc.1"),
        "a newer patch's rc is newer"
    );
}

#[test]
fn build_metadata_does_not_make_a_version_newer() {
    assert!(!is_update("0.1.0", "0.1.0+ci.7"));
    assert!(!is_update("0.1.0+ci.7", "0.1.0+ci.8"));
    assert!(is_update("0.1.0+ci.7", "0.1.1"));
}

#[test]
fn a_malformed_version_is_refused() {
    for bad in [
        "", "v0.2.0", "0.2", "0.2.0.1", "latest", "0.2.0-", "01.2.0", " 0.2.0",
    ] {
        assert!(!is_update("0.1.0", bad), "remote {bad:?}");
        assert!(!is_update(bad, "9.9.9"), "installed {bad:?}");
    }
}

#[test]
fn nothing_happens_while_listening() {
    assert_eq!(refuse_while_listening(true), Err(LISTENING.to_string()));
    assert_eq!(refuse_while_listening(false), Ok(()));
    assert_eq!(LISTENING, "Stop listening before checking for updates.");
}
