# RustPods Asset Directory: Technical Reference and Integration Protocol

## Overview

This directory contains all static assets utilized by the RustPods application. Assets are embedded into the application binary at compile time to ensure deterministic deployment and operational integrity.

## Directory Structure

```
assets/
├── icons/
│   ├── app/          # Application icons
│   ├── tray/         # System tray icons
│   └── ui/           # User interface elements
└── README.md         # This document
```

## Asset Integration

Assets are integrated into the application using Rust's `include_bytes!` macro. The `src/assets.rs` module provides programmatic access to all embedded assets.

### Usage in Code

```rust
use crate::assets;
let logo_bytes = assets::app::LOGO;
let tray_icon = assets::tray::DARK_CONNECTED;
```

## Asset Specifications

### System Tray Icons
- **Format**: ICO
- **Size**: 16x16 pixels (standard Windows tray icon)
- **Variants**:
  - Dark theme (for light backgrounds)
  - Light theme (for dark backgrounds)
  - Connected state
  - Disconnected state

### Application Icons
- **Main Logo**: PNG, high resolution
- **Windows Icons**: ICO, multiple sizes (16x16, 32x32, 48x48, 256x256 recommended)

## Asset Management Protocol

### Adding New Assets
1. Place the asset in the appropriate directory.
2. Update `src/assets.rs` to include the new asset.
3. Reference the asset in code via the assets module.

### Updating Existing Assets
1. Replace the existing file with the new version.
2. Retain the original filename to preserve compatibility.
3. Rebuild the application to embed the updated asset.

## Icon Design Standards

- System tray icons must be legible and recognizable at small sizes.
- Connected/disconnected states must be visually distinct.
- All icons must conform to the application's design language and ensure appropriate contrast for both dark and light themes.

## Compliance

All asset modifications must adhere to the RustPods project's technical and design standards. Asset changes are subject to review for consistency, clarity, and operational suitability. 