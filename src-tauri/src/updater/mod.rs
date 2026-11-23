//! Updates only when the operator asks (ADR-020 §§ 1-2).
//!
//! The GUI's "Check for updates" button (Advanced > Settings) calls
//! [`check_for_update`]; its "Install and restart" calls [`install_update`].
//! Nothing here runs at startup or on a timer, and the webview has no updater
//! permission: these two commands are the only way in. Both refuse while Voxxa
//! is listening, before any network call. Every release is judged by
//! [`should_update`], a pure function, never by the plugin's built-in check.

use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Runtime, State};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::AppState;

/// What a check found. Serialised as `{ "status": "...", ... }` for the GUI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Outcome {
    /// Refused because Voxxa is listening; no network call was made.
    Listening,
    /// The newest published release is not newer than this one.
    UpToDate { current: String },
    /// A newer release can be installed.
    Available { current: String, version: String },
}

/// True only when `remote` is a strictly newer semver version than
/// `installed`, by semver precedence: equal is no update, lower is refused,
/// `0.1.0-rc.1 < 0.1.0`, build metadata alone is no update, and a version
/// that does not parse (a leading `v` included) is refused.
pub fn should_update(installed: &str, remote: &str) -> bool {
    match (
        semver::Version::parse(installed),
        semver::Version::parse(remote),
    ) {
        (Ok(installed), Ok(remote)) => remote.cmp_precedence(&installed).is_gt(),
        _ => false,
    }
}

/// The rule that comes before any network call: while Voxxa is listening
/// (a service, or the first-run test) an update check is refused.
pub fn refused(listening: bool) -> Option<Outcome> {
    listening.then_some(Outcome::Listening)
}

/// The updater plugin with [`should_update`] as its default comparator, so no
/// path through the plugin falls back to its own version check.
pub fn plugin<R: Runtime>() -> tauri::plugin::TauriPlugin<R, tauri_plugin_updater::Config> {
    tauri_plugin_updater::Builder::new()
        .default_version_comparator(|current, release| {
            should_update(&current.to_string(), &release.version.to_string())
        })
        .build()
}

/// What the GUI shows when a check fails. tauri-plugin-updater 2.14.1 returns
/// `ReleaseNotFound` for any non-success HTTP status (`updater.rs:661`): 404
/// before the first full release, but also 5xx and a rate-limit 403. So it is
/// never "up to date", and it gets no internet hint: the connection worked.
pub fn check_error(e: tauri_plugin_updater::Error) -> String {
    match e {
        tauri_plugin_updater::Error::ReleaseNotFound => {
            "No update is published yet, or GitHub did not answer. Try again later.".into()
        }
        e => format!("Could not check for updates: {e}"),
    }
}

/// One request to the release endpoint, counted on the privacy card.
async fn latest(app: &AppHandle) -> Result<Option<Update>, String> {
    let updater = (app.updater_builder())
        .version_comparator(|current, release| {
            should_update(&current.to_string(), &release.version.to_string())
        })
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(check_error)?;
    crate::net_stats::record_request();
    updater.check().await.map_err(check_error)
}

fn outcome(app: &AppHandle, update: Option<&Update>) -> Outcome {
    let current = app.package_info().version.to_string();
    match update {
        None => Outcome::UpToDate { current },
        Some(u) => Outcome::Available {
            current,
            version: u.version.clone(),
        },
    }
}

/// "Check for updates": asks the release endpoint once.
#[tauri::command]
pub async fn check_for_update(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Outcome, String> {
    if let Some(no) = refused(state.sessions.is_running()) {
        return Ok(no);
    }
    let update = latest(&app).await?;
    Ok(outcome(&app, update.as_ref()))
}

/// "Install and restart", after the operator confirmed `version`. Checks
/// again, installs only that exact version, and refuses again if listening
/// started during the download. On Windows the installer closes Voxxa and
/// starts it again; elsewhere Voxxa restarts itself.
#[tauri::command]
pub async fn install_update(
    app: AppHandle,
    state: State<'_, AppState>,
    version: String,
) -> Result<Outcome, String> {
    if let Some(no) = refused(state.sessions.is_running()) {
        return Ok(no);
    }
    let Some(update) = latest(&app).await? else {
        return Ok(outcome(&app, None));
    };
    if update.version != version {
        return Err(format!(
            "The update on offer is now {}, not {version}. Check again.",
            update.version
        ));
    }
    crate::net_stats::record_request();
    let bytes = (update.download(|_, _| {}, || {}).await)
        .map_err(|e| format!("Could not download the update: {e}"))?;
    if let Some(no) = refused(state.sessions.is_running()) {
        return Ok(no);
    }
    (update.install(bytes)).map_err(|e| format!("Could not install the update: {e}"))?;
    app.restart()
}
