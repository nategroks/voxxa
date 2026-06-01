use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    App, AppHandle, Emitter, Manager,
};

use crate::AppState;

/// Create the system tray icon and menu.
///
/// Tray items are emergency-grade operator controls: anything the worship
/// operator might need to do mid-service without unminimizing the main window.
/// Per §6.3 of the plan, Blank is the panic button — it bypasses the conductor
/// and hits the active presenter directly.
pub fn create_tray(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let toggle = MenuItem::with_id(app, "toggle", "Start / Stop listening", true, None::<&str>)?;
    let blank = MenuItem::with_id(app, "blank", "Blank output", true, None::<&str>)?;
    let stage = MenuItem::with_id(app, "stage", "Show / hide Stage Display", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let show_main = MenuItem::with_id(app, "show", "Show Voxxa window", true, None::<&str>)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Voxxa", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[&toggle, &blank, &stage, &sep1, &show_main, &sep2, &quit],
    )?;

    TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("Voxxa")
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "toggle" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.emit("tray-toggle-recording", ());
                }
            }
            "blank" => {
                tray_blank(app);
            }
            "stage" => {
                tray_toggle_stage(app);
            }
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}

/// Spawn an async task that blanks the active presenter. Tray menu callbacks
/// are sync; we hop into the Tauri async runtime to do the actual work.
fn tray_blank(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        // Pre-clone the Arcs we need before any await so the tauri::State
        // guard (which is !Send) doesn't try to cross an await point.
        let (presenter, conductor) = {
            let state = app.state::<AppState>();
            (state.presenter.clone(), state.conductor.clone())
        };
        {
            let p = presenter.lock().await;
            if let Err(e) = p.blank().await {
                log::warn!("[TRAY] blank failed: {e}");
                return;
            }
        }
        // Keep the conductor's is_blank in sync — without this, the next lyric
        // match wouldn't fire an unblank because the conductor thinks output
        // is still showing.
        let mut cl = conductor.lock().await;
        if let Some(c) = cl.as_mut() {
            c.notify_external_blank();
        }
    });
}

fn tray_toggle_stage(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("stage") {
        match win.is_visible() {
            Ok(true) => {
                let _ = win.hide();
            }
            _ => {
                let _ = win.show();
                let _ = win.set_focus();
            }
        }
    }
}
