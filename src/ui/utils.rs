//! UI utility functions

use iced::window::Icon;

/// Load the window icon with proper error handling and fallbacks
pub fn load_window_icon() -> Option<Icon> {
    // Try different icon sources in order of preference
    let icon_attempts = [
        // Primary icon - 128x128 ICO
        (
            "ICO 128x128",
            include_bytes!("../../assets/icons/app/icon_128.ico") as &[u8],
        ),
        // Fallback - 256x256 ICO
        (
            "ICO 256x256",
            include_bytes!("../../assets/icons/app/icon_256.ico") as &[u8],
        ),
    ];

    for (description, icon_data) in icon_attempts.iter() {
        crate::debug_log!("ui", "Attempting to load window icon: {}", description);

        match iced::window::icon::from_file_data(icon_data, None) {
            Ok(icon) => {
                log::info!("Successfully loaded window icon: {}", description);
                return Some(icon);
            }
            Err(e) => {
                log::warn!("Failed to load window icon {}: {}", description, e);
            }
        }
    }

    log::error!("Failed to load any window icon, application will use default system icon");
    None
}
