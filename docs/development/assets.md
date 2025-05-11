# Asset Management in RustPods

This guide explains how assets are organized and used in the RustPods application.

## Directory Structure

```
assets/
├── icons/
│   ├── app/          - Application icons
│   │   ├── logo.png  - Main application logo
│   │   ├── icon_256.ico - 256x256 app icon
│   │   └── icon_128.ico - 128x128 app icon
│   ├── tray/         - System tray icons
│   │   ├── rustpods-tray-dark-connected.ico    - Dark theme, connected
│   │   ├── rustpods-tray-dark-disconnected.ico - Dark theme, disconnected
│   │   ├── rustpods-tray-light-connected.ico   - Light theme, connected
│   │   └── rustpods-tray-light-disconnected.ico - Light theme, disconnected
│   └── ui/           - UI elements
└── README.md         - Asset documentation
```

## Asset Implementation

Assets are embedded into the application binary using Rust's `include_bytes!` macro. The `src/assets.rs` module provides access to all assets:

```rust
// src/assets.rs
pub mod app {
    pub const LOGO: &[u8] = include_bytes!("../assets/icons/app/logo.png");
    pub const ICON_256: &[u8] = include_bytes!("../assets/icons/app/icon_256.ico");
    pub const ICON_128: &[u8] = include_bytes!("../assets/icons/app/icon_128.ico");
}

pub mod tray {
    pub const DARK_CONNECTED: &[u8] = include_bytes!("../assets/icons/tray/rustpods-tray-dark-connected.ico");
    pub const DARK_DISCONNECTED: &[u8] = include_bytes!("../assets/icons/tray/rustpods-tray-dark-disconnected.ico");
    pub const LIGHT_CONNECTED: &[u8] = include_bytes!("../assets/icons/tray/rustpods-tray-light-connected.ico");
    pub const LIGHT_DISCONNECTED: &[u8] = include_bytes!("../assets/icons/tray/rustpods-tray-light-disconnected.ico");
}

pub mod ui {
    // UI-specific assets would go here
}
```

## Using Assets in Code

Assets can be accessed from any part of the application as follows:

```rust
// Import the assets module
use crate::assets;

// Access application logo
let logo_bytes = assets::app::LOGO;

// Access tray icons
let tray_icon = assets::tray::DARK_CONNECTED;
```

In the SystemTray implementation, icons are provided to the tray-item crate:

```rust
IconSource::Data {
    data: crate::assets::tray::DARK_CONNECTED.to_vec(),
}
```

## Asset Requirements

### System Tray Icons
- Format: ICO
- Size: 16x16 pixels for standard Windows tray icons
- Variations needed:
  - Dark theme (for light backgrounds)
  - Light theme (for dark backgrounds)
  - Connected state
  - Disconnected state

### Application Icons
- Main logo: PNG format, high resolution
- Windows icons: ICO format, multiple sizes (recommended: 16x16, 32x32, 48x48, 256x256)

## Adding New Assets

1. Place new assets in the appropriate directory
2. Update `src/assets.rs` to include the new asset
3. Use the asset in your code by referencing it from the assets module

## Updating Existing Assets

1. Replace the existing file with the new version
2. Keep the same filename to maintain compatibility
3. Rebuild the application to embed the updated asset

## Icon Design Guidelines

- System tray icons should be simple and recognizable at small sizes
- Connected icons should clearly indicate a connection status
- Disconnected icons should use more muted colors
- All icons should follow the overall application design language
- For dark/light themes, ensure appropriate contrast 