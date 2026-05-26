use anyhow::Result;
use enigo::{Enigo, Keyboard, Settings};

/// Insert text at the current cursor position using platform accessibility APIs.
///
/// Uses enigo for cross-platform keyboard simulation.
/// On macOS, this uses the Accessibility API.
/// On Linux, this uses XDG/X11 or Wayland protocols.
/// On Windows, this uses the SendInput API.
pub fn insert_text(text: &str) -> Result<()> {
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| anyhow::anyhow!("Failed to initialize enigo: {:?}", e))?;

    // Use text method for direct Unicode text insertion (no clipboard hack)
    enigo
        .text(text)
        .map_err(|e| anyhow::anyhow!("Failed to insert text: {:?}", e))?;

    log::info!("Inserted {} chars via accessibility API", text.len());
    Ok(())
}

/// Insert text with a small delay to let the target app process it.
pub fn insert_text_delayed(text: &str, delay_ms: u64) -> Result<()> {
    std::thread::sleep(std::time::Duration::from_millis(delay_ms));
    insert_text(text)
}
