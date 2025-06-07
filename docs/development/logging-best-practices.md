# RustPods Logging Best Practices

## Overview

RustPods implements a sophisticated configurable logging system that provides clean output by default while offering powerful selective debugging capabilities for developers. This guide outlines best practices for using the logging system effectively.

## Core Principles

### 1. Clean Default Experience
- Default application startup should show minimal output (warnings and errors only)
- Debug messages should only appear when explicitly requested via command-line flags
- Never use `println!()` for debug messages - always use the appropriate logging macros

### 2. Selective Debug Categories
- Debug output is organized into logical categories matching the application's architecture
- Developers can enable debug output for specific categories without noise from other components
- Categories align with the application's module structure

## Logging Macros and Functions

### Standard Log Levels (Always Visible When Enabled)
```rust
// Critical errors that prevent functionality
error!("Failed to initialize Bluetooth adapter: {}", error);

// Warnings about concerning but non-fatal issues  
warn!("Device connection unstable, RSSI: {}", rssi);

// Important application state changes
info!("Connected to AirPods: {}", device_name);

// General debug information (shows when --debug flag is used)
debug!("Processing {} devices from scanner", device_count);

// Very detailed tracing information
trace!("Raw BLE data: {:?}", raw_data);
```

### Conditional Debug Logging (Category-Specific)
```rust
// UI-related debug messages (--debug-ui)
crate::debug_log!("ui", "Window state changed to: {:?}", window_state);
crate::debug_log!("ui", "System tray clicked, showing window");

// Bluetooth operations (--debug-bluetooth)  
crate::debug_log!("bluetooth", "CLI scanner found {} devices", devices.len());
crate::debug_log!("bluetooth", "Adapter {} selected", adapter_name);

// AirPods-specific operations (--debug-airpods)
crate::debug_log!("airpods", "Battery levels: L:{}% R:{}% C:{}%", left, right, case);
crate::debug_log!("airpods", "Manufacturer data parsed: {:?}", mfg_data);

// Configuration operations (--debug-config)
crate::debug_log!("config", "Loading configuration from: {}", config_path);
crate::debug_log!("config", "Validation failed for setting: {}", setting_name);

// System-level operations (--debug-system)
crate::debug_log!("system", "Application lifecycle state: {:?}", state);
crate::debug_log!("system", "Persistence save completed");
```

## Debug Categories

### UI Category (`--debug-ui`)
**Purpose:** Window management, system tray operations, UI events, state transitions

**Modules:**
- `src/ui/state.rs` - UI state management
- `src/ui/system_tray.rs` - System tray operations  
- `src/ui/window_visibility.rs` - Window show/hide operations
- `src/ui/main_window.rs` - Main window rendering

**Example Usage:**
```rust
crate::debug_log!("ui", "AppState::update received message: {:?}", message);
crate::debug_log!("ui", "Main window rendering {} devices", device_count);
```

### Bluetooth Category (`--debug-bluetooth`)
**Purpose:** Bluetooth adapter discovery, device scanning, CLI scanner operations

**Modules:**
- `src/bluetooth/scanner.rs` - BLE scanning operations
- `src/bluetooth/cli_scanner.rs` - Native CLI scanner integration
- `src/bluetooth/adapter.rs` - Bluetooth adapter management

**Example Usage:**
```rust
crate::debug_log!("bluetooth", "Starting BLE scan with adapter: {}", adapter_id);
crate::debug_log!("bluetooth", "CLI scanner output: {}", scanner_output);
```

### AirPods Category (`--debug-airpods`)
**Purpose:** AirPods device detection, battery parsing, device-specific operations

**Modules:**
- `src/airpods/detector.rs` - AirPods device detection
- `src/airpods/battery.rs` - Battery data parsing
- `src/bluetooth/battery.rs` - Battery information handling

**Example Usage:**
```rust
crate::debug_log!("airpods", "Detected AirPods model: {:?}", model);
crate::debug_log!("airpods", "Battery update: L:{}% R:{}%", left_pct, right_pct);
```

### Config Category (`--debug-config`)
**Purpose:** Configuration loading, saving, validation, settings management

**Modules:**
- `src/config/mod.rs` - Configuration management
- `src/config/app_config.rs` - Application configuration
- `src/ui/form_validation.rs` - Settings validation

**Example Usage:**
```rust
crate::debug_log!("config", "Loading config from: {}", config_file.display());
crate::debug_log!("config", "Setting validation error: {}", error_msg);
```

### System Category (`--debug-system`)
**Purpose:** Application lifecycle, persistence, telemetry, diagnostics

**Modules:**
- `src/lifecycle_manager.rs` - Application lifecycle
- `src/state_persistence.rs` - State persistence
- `src/telemetry.rs` - Telemetry collection
- `src/diagnostics.rs` - System diagnostics

**Example Usage:**
```rust
crate::debug_log!("system", "Application shutdown initiated");
crate::debug_log!("system", "State persisted to: {}", state_file.display());
```

## Command Line Usage Examples

### End User (Clean Experience)
```bash
# Normal operation - only shows warnings and errors
rustpods

# Quiet mode - only shows errors
rustpods --quiet
```

### Developer Debugging
```bash
# Debug specific component
rustpods --debug-ui               # UI issues only
rustpods --debug-bluetooth        # Bluetooth issues only
rustpods --debug-airpods          # AirPods detection issues only

# Debug multiple categories
rustpods --debug-ui --debug-bluetooth

# Full debug mode (all categories)
rustpods --debug-all
rustpods -v

# Combine with commands
rustpods --debug-bluetooth adapters    # Debug bluetooth while listing adapters
rustpods --debug-ui                    # Debug UI in normal mode
```

## Implementation Requirements

### DO ✅

**Use appropriate logging macros:**
```rust
// For conditional debug output
crate::debug_log!("category", "Message with data: {}", value);

// For always-visible important information
info!("Application started successfully");
warn!("Low battery detected: {}%", battery_level);
error!("Critical failure: {}", error);
```

**Include context in log messages:**
```rust
crate::debug_log!("bluetooth", "Device scan completed: {} devices found, scan duration: {}ms", 
                  device_count, scan_duration);
```

**Use structured information:**
```rust
crate::debug_log!("airpods", "Battery update - Device: {}, L:{}%, R:{}%, C:{}%, RSSI: {}", 
                  device_name, left, right, case, rssi);
```

### DON'T ❌

**Never use println! for debug output:**
```rust
// ❌ This always shows regardless of debug flags
println!("[DEBUG] Processing device: {}", device_id);
```

**Don't use log::debug! directly for category-specific output:**
```rust
// ❌ This shows for any --debug flag, not category-specific
log::debug!("UI update completed");
```

**Don't use excessive debug output in tight loops:**
```rust
// ❌ This can flood the output
for device in devices {
    crate::debug_log!("bluetooth", "Processing device: {}", device.id); // Too verbose
}

// ✅ Better approach
crate::debug_log!("bluetooth", "Processing {} devices", devices.len());
for device in devices {
    // Only log significant events or errors
    if let Err(e) = process_device(device) {
        crate::debug_log!("bluetooth", "Failed to process device {}: {}", device.id, e);
    }
}
```

## Testing Your Logging

### Local Testing
```bash
# Test default clean output
cargo run

# Test category-specific debugging
cargo run -- --debug-ui
cargo run -- --debug-bluetooth adapters
cargo run -- --debug-airpods

# Test full debug mode
cargo run -- --debug-all
```

### Verify Debug Output
1. **Default mode should show minimal output** - only configuration messages, warnings, and errors
2. **Category flags should show only relevant debug messages** - no cross-contamination between categories
3. **Debug messages should be informative** - include context, values, and operation status

## Migration Guide

### Converting Existing Debug Code

**Replace println! debug statements:**
```rust
// OLD
println!("[DEBUG] Device found: {}", device.name);

// NEW  
crate::debug_log!("bluetooth", "Device found: {}", device.name);
```

**Replace log::debug! calls:**
```rust
// OLD
log::debug!("UI state updated");

// NEW
crate::debug_log!("ui", "UI state updated");
```

**Choose appropriate categories:**
- Look at the module path to determine the appropriate category
- UI modules → `"ui"`
- Bluetooth modules → `"bluetooth"`  
- AirPods modules → `"airpods"`
- Config modules → `"config"`
- System modules → `"system"`

## Performance Considerations

### Debug Macro Efficiency
The `debug_log!` macro includes compile-time optimizations:
- Debug flag checking is fast (atomic read)
- String formatting only occurs when debug output is enabled
- Minimal overhead when debug categories are disabled

### Best Practices
- **Prefer structured data over string concatenation** in debug messages
- **Use lazy evaluation** for expensive debug computations
- **Avoid debug output in performance-critical paths** unless essential

```rust
// ✅ Efficient - only formats when needed
crate::debug_log!("bluetooth", "Scan results: {} devices, took {}ms", 
                  devices.len(), duration.as_millis());

// ❌ Inefficient - always computes expensive_computation()
crate::debug_log!("system", "Complex data: {}", expensive_computation());

// ✅ Better - use lazy evaluation
if crate::logging::should_log_debug(module_path!()) {
    let complex_data = expensive_computation();
    crate::debug_log!("system", "Complex data: {}", complex_data);
}
```

## Future Enhancements

### Planned Improvements
1. **Configuration file support** - Allow debug flags in configuration files
2. **Runtime toggle** - Enable/disable debug categories during application runtime  
3. **Log level per category** - Different log levels for different categories
4. **Performance metrics** - Timing information in debug output
5. **Structured logging** - JSON output for automated log analysis

### Maintenance Tasks
- Monitor module path changes that might affect category detection
- Update category mappings when new modules are added
- Review performance impact of debug flag checking
- Ensure backward compatibility with existing logging calls

This logging system provides a professional user experience while maintaining powerful debugging capabilities for development and troubleshooting. Follow these guidelines to ensure consistent, useful, and performant logging throughout the RustPods application. 