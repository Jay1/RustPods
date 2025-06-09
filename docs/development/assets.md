# Asset Management System Architecture

## System Overview

This document defines the asset management architecture for RustPods, establishing protocols for resource allocation, binary embedding, and runtime asset access within the application framework.

## Asset Organization Architecture

### Resource Directory Structure
```
assets/
├── icons/
│   ├── app/          # Application identity assets
│   │   ├── logo_ring.png                    # Primary brand identity (ring design)
│   │   ├── logo_ring.ico                    # Windows application icon (ring design)
│   │   ├── battery_ring_80_percent.svg      # Repository documentation asset
│   │   ├── icon_256.ico                     # High-resolution application icon
│   │   └── icon_128.ico                     # Standard-resolution application icon
│   ├── tray/         # System tray integration assets
│   │   ├── rustpods-tray-dark-connected.ico     # Dark UI theme, connected state
│   │   ├── rustpods-tray-dark-disconnected.ico  # Dark UI theme, disconnected state
│   │   ├── rustpods-tray-light-connected.ico    # Light UI theme, connected state
│   │   └── rustpods-tray-light-disconnected.ico # Light UI theme, disconnected state
│   └── ui/           # User interface component assets (SVG vector graphics)
```

## Binary Embedding Framework

### Asset Integration Implementation
RustPods employs Rust's compile-time asset embedding through the `include_bytes!` macro, ensuring zero-dependency resource deployment and eliminating external file dependencies.

```rust
// Asset Management Module: src/assets.rs
pub mod app {
    /// Primary application brand identity asset
    pub const LOGO: &[u8] = include_bytes!("../assets/icons/app/logo_ring.png");
    /// High-resolution Windows application icon
    pub const ICON_256: &[u8] = include_bytes!("../assets/icons/app/logo_ring.ico");
    /// Standard-resolution Windows application icon  
    pub const ICON_128: &[u8] = include_bytes!("../assets/icons/app/logo_ring.ico");
}

pub mod tray {
    /// System tray icon: Dark theme, connected state
    pub const DARK_CONNECTED: &[u8] = include_bytes!("../assets/icons/tray/rustpods-tray-dark-connected.ico");
    /// System tray icon: Dark theme, disconnected state
    pub const DARK_DISCONNECTED: &[u8] = include_bytes!("../assets/icons/tray/rustpods-tray-dark-disconnected.ico");
    /// System tray icon: Light theme, connected state
    pub const LIGHT_CONNECTED: &[u8] = include_bytes!("../assets/icons/tray/rustpods-tray-light-connected.ico");
    /// System tray icon: Light theme, disconnected state
    pub const LIGHT_DISCONNECTED: &[u8] = include_bytes!("../assets/icons/tray/rustpods-tray-light-disconnected.ico");
}

pub mod ui {
    // Vector graphics and UI component assets are defined here
}
```

## Runtime Asset Access Protocols

### Asset Consumption Framework
Application components access embedded assets through standardized import and reference patterns:

```rust
// Asset module import
use crate::assets;

// Application identity asset access
let logo_bytes = assets::app::LOGO;

// System tray asset access for state indication
let tray_icon = assets::tray::DARK_CONNECTED;
```

### System Tray Integration Implementation
System tray components utilize the asset management framework for dynamic icon state representation:

```rust
// System tray icon provisioning
IconSource::Data {
    data: crate::assets::tray::DARK_CONNECTED.to_vec(),
}
```

## Asset Specification Standards

### System Tray Icon Requirements
- **Format**: ICO (Windows Icon format)
- **Resolution**: 16×16 pixels (standard Windows system tray specification)
- **State Variations**: Connected/disconnected operational states
- **Theme Variations**: Dark/light theme compatibility for Windows UI integration

### Application Icon Requirements
- **Primary Logo**: PNG format, vector-scalable resolution
- **Windows Icons**: ICO format with multi-resolution embedding (16×16, 32×32, 48×48, 256×256)
- **Brand Consistency**: Maintains visual identity standards across all application touchpoints

## Asset Development Workflow

### New Asset Integration Procedure
1. **Asset Placement**: Deploy new assets within the appropriate directory structure
2. **Module Registration**: Update `src/assets.rs` with new asset constant definitions
3. **Implementation Integration**: Reference assets through the standardized assets module import pattern

### Asset Update Management
1. **File Replacement**: Replace existing assets while maintaining filename consistency for compatibility
2. **Binary Recompilation**: Execute build process to embed updated assets into application binary
3. **Validation**: Verify asset integration through application testing protocols

## Visual Design Standards

### Icon Design Framework
- **System Tray Icons**: Optimized for small-scale recognition and clarity at 16×16 pixel resolution
- **Connection State Indication**: Clear visual differentiation between connected and disconnected operational states
- **Theme Integration**: Appropriate contrast ratios for both Windows dark and light UI themes
- **Brand Cohesion**: Consistent design language alignment with overall application visual identity
- **Accessibility**: Adequate contrast ratios meeting accessibility standards for users with visual impairments

### Asset Quality Assurance
All assets undergo validation for technical compliance, visual consistency, and operational functionality prior to integration into the production build system. 