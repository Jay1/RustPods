# Asset Management System Architecture: Technical Reference and Integration Protocol

## Executive Summary

This document defines the authoritative asset management architecture for RustPods, specifying protocols for resource organization, binary embedding, and runtime access. All procedures and standards herein are aligned with the current project structure and technical requirements.

## Asset Directory Structure

```
assets/
├── fonts/
│   └── SpaceMonoNerdFont-Regular.ttf           # Application font (UI embedding)
├── icons/
│   ├── app/
│   │   ├── battery_ring_80_percent.svg        # Documentation and UI asset
│   │   ├── logo_ring.ico                      # Windows application icon (multi-resolution)
│   │   ├── logo_ring.png                      # High-resolution PNG logo
│   │   └── logo_ring.svg                      # Vector logo (scalable)
│   ├── charging.svg                           # Charging state icon
│   ├── close.svg                              # UI close button icon
│   ├── hw/
│   │   ├── airpods.png                        # AirPods hardware illustration
│   │   ├── airpodscase.png                    # AirPods case illustration
│   │   ├── AirpodsMax.png                     # AirPods Max illustration
│   │   ├── airpodspro.png                     # AirPods Pro illustration
│   │   ├── airpodsprocase.png                 # AirPods Pro case illustration
│   │   ├── beats.png                          # Beats hardware illustration
│   │   └── beatscase.png                      # Beats case illustration
│   ├── settings.svg                           # Settings icon
│   ├── tray/
│   │   ├── rustpods-tray-dark-connected.ico   # System tray, dark theme, connected
│   │   ├── rustpods-tray-dark-disconnected.ico# System tray, dark theme, disconnected
│   │   ├── rustpods-tray-light-connected.ico  # System tray, light theme, connected
│   │   └── rustpods-tray-light-disconnected.ico# System tray, light theme, disconnected
│   └── ui/                                    # UI-specific icons (future expansion)
└── README.md                                  # Asset directory reference
```

## Binary Embedding Protocol

All assets are embedded at compile time using Rust's `include_bytes!` macro. The `src/assets.rs` module exposes a structured API for asset access, ensuring deterministic deployment and eliminating runtime file dependencies.

### Example: Asset Module Structure
```rust
// src/assets.rs
pub mod app {
    pub const LOGO: &[u8] = include_bytes!("../assets/icons/app/logo_ring.png");
    pub const ICON: &[u8] = include_bytes!("../assets/icons/app/logo_ring.ico");
}
pub mod tray {
    pub const DARK_CONNECTED: &[u8] = include_bytes!("../assets/icons/tray/rustpods-tray-dark-connected.ico");
    pub const LIGHT_CONNECTED: &[u8] = include_bytes!("../assets/icons/tray/rustpods-tray-light-connected.ico");
    // ... other tray icons
}
```

## Runtime Asset Access

Application components import and reference assets via the `assets` module:
```rust
use crate::assets;
let logo = assets::app::LOGO;
let tray_icon = assets::tray::DARK_CONNECTED;
```

## Asset Specification Standards

### System Tray Icons
- **Format**: ICO (Windows icon format)
- **Resolution**: 16×16 pixels (standard system tray)
- **Variants**: Connected/disconnected, dark/light theme

### Application Icons
- **Format**: ICO (multi-resolution), PNG, SVG
- **Usage**: Application branding, installer, documentation

### Hardware Illustrations
- **Format**: PNG
- **Purpose**: UI and documentation, device identification

### UI Icons
- **Format**: SVG
- **Purpose**: Scalable vector graphics for UI components

### Fonts
- **Format**: TTF
- **Usage**: Embedded for UI rendering consistency

## Asset Integration Workflow

### Adding New Assets
1. Place the asset in the appropriate directory.
2. Register the asset in `src/assets.rs`.
3. Reference the asset in code via the assets module.
4. Rebuild the application to embed the asset.

### Updating Assets
1. Replace the file, maintaining the original filename for compatibility.
2. Rebuild the application to update the embedded asset.
3. Validate integration through application testing.

## Visual and Technical Design Standards

- All icons must be legible at target resolution and maintain brand consistency.
- System tray icons must provide clear state indication and theme compatibility.
- Hardware illustrations must accurately represent supported devices.
- All assets must meet accessibility standards for contrast and clarity.
- Asset changes are subject to review for technical and visual compliance.

## Compliance and Quality Assurance

All assets are validated for technical correctness, visual consistency, and operational suitability prior to production integration. Asset management protocols are enforced to ensure deterministic builds and maintainable resource allocation. 