//! ADR-020 §§ 1-2: the update comparator and the listening rule, as pure
//! functions. No network: nothing here builds an updater or an app.

use voxxa_lib::updater::{refused, should_update, Outcome};

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
            notes: None
        }),
        serde_json::json!({"status": "available", "current": "0.1.0", "version": "0.2.0", "notes": null})
    );
}
