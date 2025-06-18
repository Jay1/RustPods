//! Assets module for embedding static files into the application binary

/// App icons and images
pub mod app {
    /// Main application logo (PNG format)
    pub const LOGO: &[u8] = include_bytes!("../assets/icons/app/logo_ring.png");

    /// Main application logo SVG (vector format for crisp scaling)
    pub const LOGO_SVG: &[u8] = include_bytes!("../assets/icons/app/logo_ring.svg");

    /// Application icon 256x256 (ICO format)
    pub const ICON_256: &[u8] = include_bytes!("../assets/icons/app/logo_ring.ico");

    /// Application icon 128x128 (ICO format)
    pub const ICON_128: &[u8] = include_bytes!("../assets/icons/app/logo_ring.ico");
}

/// System tray icons
pub mod tray {
    /// System tray icon for dark theme, connected state (ICO format)
    pub const DARK_CONNECTED: &[u8] =
        include_bytes!("../assets/icons/tray/rustpods-tray-dark-connected.ico");

    /// System tray icon for dark theme, disconnected state (ICO format)
    pub const DARK_DISCONNECTED: &[u8] =
        include_bytes!("../assets/icons/tray/rustpods-tray-dark-disconnected.ico");

    /// System tray icon for light theme, connected state (ICO format)
    pub const LIGHT_CONNECTED: &[u8] =
        include_bytes!("../assets/icons/tray/rustpods-tray-light-connected.ico");

    /// System tray icon for light theme, disconnected state (ICO format)
    pub const LIGHT_DISCONNECTED: &[u8] =
        include_bytes!("../assets/icons/tray/rustpods-tray-light-disconnected.ico");
}

/// UI element assets
pub mod ui {
    /// Close icon SVG for buttons and dialogs
    pub const CLOSE_ICON: &[u8] = include_bytes!("../assets/icons/close.svg");

    /// Charging bolt icon SVG for battery indicators
    pub const CHARGING_ICON: &[u8] = include_bytes!("../assets/icons/charging.svg");

    /// Settings gear icon SVG for settings buttons
    pub const SETTINGS_ICON: &[u8] = include_bytes!("../assets/icons/settings.svg");
}

/// Font assets
pub mod fonts {
    /// SpaceMono Nerd Font Regular (TTF format)
    pub const SPACE_MONO_NERD_FONT: &[u8] =
        include_bytes!("../assets/fonts/SpaceMonoNerdFont-Regular.ttf");
}
