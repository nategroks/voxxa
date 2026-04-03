use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    App, Emitter, Manager,
};

/// Create the system tray icon and menu.
pub fn create_tray(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let toggle = MenuItem::with_id(app, "toggle", "Start Recording", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let separator = MenuItem::with_id(app, "sep", "---", false, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Voxxa", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&toggle, &settings, &separator, &quit])?;

    TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("Voxxa - Voice Dictation")
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "toggle" => {
                log::info!("Toggle recording from tray");
                // Toggle recording state via app handle
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.emit("tray-toggle-recording", ());
                }
            }
            "settings" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
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
