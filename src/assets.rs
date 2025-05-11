//! Assets module for embedding static files into the application binary

/// App icons and images
pub mod app {
    /// Main application logo (PNG format)
    pub const LOGO: &[u8] = include_bytes!("../assets/icons/app/logo.png");
    
    /// Application icon 256x256 (ICO format)
    pub const ICON_256: &[u8] = include_bytes!("../assets/icons/app/icon_256.ico");
    
    /// Application icon 128x128 (ICO format)
    pub const ICON_128: &[u8] = include_bytes!("../assets/icons/app/icon_128.ico");
}

/// System tray icons
pub mod tray {
    /// System tray icon for dark theme, connected state (ICO format)
    pub const DARK_CONNECTED: &[u8] = include_bytes!("../assets/icons/tray/rustpods-tray-dark-connected.ico");
    
    /// System tray icon for dark theme, disconnected state (ICO format)
    pub const DARK_DISCONNECTED: &[u8] = include_bytes!("../assets/icons/tray/rustpods-tray-dark-disconnected.ico");
    
    /// System tray icon for light theme, connected state (ICO format)
    pub const LIGHT_CONNECTED: &[u8] = include_bytes!("../assets/icons/tray/rustpods-tray-light-connected.ico");
    
    /// System tray icon for light theme, disconnected state (ICO format)
    pub const LIGHT_DISCONNECTED: &[u8] = include_bytes!("../assets/icons/tray/rustpods-tray-light-disconnected.ico");
}

/// UI element assets
pub mod ui {
    // UI-specific assets would go here
} 