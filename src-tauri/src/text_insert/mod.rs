use anyhow::Result;
use enigo::{Enigo, Key, Keyboard, Settings};

/// Send Right arrow key to advance slide in presentation software.
pub fn send_next_slide() -> Result<()> {
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| anyhow::anyhow!("Failed to initialize enigo: {:?}", e))?;

    enigo
        .key(Key::RightArrow, enigo::Direction::Click)
        .map_err(|e| anyhow::anyhow!("Failed to send key: {:?}", e))?;

    log::info!("[CTRL] Sent Right arrow key");
    Ok(())
}

/// Send Left arrow key to go back a slide.
pub fn send_prev_slide() -> Result<()> {
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| anyhow::anyhow!("Failed to initialize enigo: {:?}", e))?;

    enigo
        .key(Key::LeftArrow, enigo::Direction::Click)
        .map_err(|e| anyhow::anyhow!("Failed to send key: {:?}", e))?;

    log::info!("[CTRL] Sent Left arrow key");
    Ok(())
}
