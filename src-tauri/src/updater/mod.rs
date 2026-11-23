//! Update check and install, started by the operator only (ADR-020): the
//! "Check for updates" button in Advanced → Settings. Nothing here runs on its
//! own or at startup, and the webview has no updater plugin permission; it
//! calls these two commands. Refused while Voxxa is listening, before any
//! network call. Downloads are verified with `plugins.updater.pubkey`.

use crate::AppState;
use semver::Version;
use tauri::{AppHandle, State};
use tauri_plugin_updater::{Update, UpdaterExt};

pub const LISTENING: &str = "Stop listening before checking for updates.";

/// The rule for both commands: nothing while a service is being followed.
pub fn refuse_while_listening(listening: bool) -> Result<(), String> {
    if listening {
        return Err(LISTENING.into());
    }
    Ok(())
}

/// True only when `remote` is strictly newer than `installed` by semver
/// precedence: equal or lower is no update, a pre-release sorts before its
/// release, build metadata is ignored, and a malformed version is refused.
pub fn is_update(installed: &str, remote: &str) -> bool {
    match (Version::parse(installed), Version::parse(remote)) {
        (Ok(installed), Ok(remote)) => remote.cmp_precedence(&installed).is_gt(),
        _ => false,
    }
}

async fn find(app: &AppHandle, state: &AppState) -> Result<Option<Update>, String> {
    refuse_while_listening(state.sessions.is_running())?;
    let updater = (app.updater_builder())
        .version_comparator(|v, release| is_update(&v.to_string(), &release.version.to_string()))
        .build()
        .map_err(|e| e.to_string())?;
    crate::net_stats::record_request();
    let found = updater.check().await;
    found.map_err(|e| format!("Could not check for updates: {e}"))
}

/// The newer version on offer, or `None` when Voxxa is up to date.
#[tauri::command]
pub async fn check_for_update(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    Ok(find(&app, &state).await?.map(|u| u.version))
}

/// Install `version`, the one the operator confirmed, then restart Voxxa.
/// On Windows the installer closes Voxxa and starts it again itself.
#[tauri::command]
pub async fn install_update(
    app: AppHandle,
    state: State<'_, AppState>,
    version: String,
) -> Result<(), String> {
    let update = find(&app, &state).await?;
    let update = update.filter(|u| u.version == version);
    let update = update.ok_or("The update on offer changed. Check for updates again.")?;
    let installed = update.download_and_install(|_, _| {}, || {}).await;
    installed.map_err(|e| format!("Could not install the update: {e}"))?;
    app.restart()
}
