# RustPods Logging Best Practices: Technical Reference and Implementation Protocol

## Preamble: Document Scope and Relevance

This document defines the technical standards and operational protocols for logging and debugging within the RustPods application. It is intended for developers and maintainers seeking to ensure consistent, performant, and professional logging throughout the system. All recommendations and examples are current as of the latest implementation. Legacy or deprecated techniques are clearly marked.

## 1. Logging System Architecture

- **Custom Logger**: RustPods implements a custom logging system with category-based filtering and atomic flag checking.
- **Default Output**: Only warnings and errors are displayed by default, ensuring a professional user experience.
- **Selective Debugging**: Developers can enable debug output for specific categories via command-line flags.
- **Performance**: The system is optimized for minimal overhead when debug output is disabled.

## 2. Core Logging Principles

### 2.1 Clean Default Experience
- Application startup and normal operation display only warnings and errors.
- Debug messages are suppressed unless explicitly enabled.
- All debug output must use the appropriate logging macros; `println!()` is prohibited for debug purposes.

### 2.2 Category-Based Debugging
- Debug output is organized into logical categories aligned with the application's architecture.
- Categories include: `ui`, `bluetooth`, `airpods`, `config`, `system`.
- Developers can enable one or more categories as needed for targeted diagnostics.

## 3. Logging Macros and Usage

### 3.1 Standard Log Levels
```rust
error!("Failed to initialize Bluetooth adapter: {}", error);
warn!("Device connection unstable, RSSI: {}", rssi);
info!("Connected to AirPods: {}", device_name);
debug!("Processing {} devices from scanner", device_count);
trace!("Raw BLE data: {:?}", raw_data);
```

### 3.2 Category-Specific Debug Logging
```rust
crate::debug_log!("ui", "Window state changed to: {:?}", window_state);
crate::debug_log!("bluetooth", "CLI scanner found {} devices", devices.len());
crate::debug_log!("airpods", "Battery levels: L:{}% R:{}% C:{}%", left, right, case);
crate::debug_log!("config", "Loading configuration from: {}", config_path);
crate::debug_log!("system", "Application lifecycle state: {:?}", state);
```

## 4. Debug Categories and Module Mapping

| Category    | Purpose                                      | Modules                                      |
|-------------|----------------------------------------------|----------------------------------------------|
| `ui`        | UI events, window management, system tray    | `src/ui/state.rs`, `src/ui/system_tray.rs`   |
| `bluetooth` | BLE scanning, CLI scanner, adapter mgmt      | `src/bluetooth/scanner.rs`, `src/bluetooth/cli_scanner.rs`, `src/bluetooth/adapter.rs` |
| `airpods`   | AirPods detection, battery parsing           | `src/airpods/detector.rs`, `src/airpods/battery.rs`, `src/bluetooth/battery.rs` |
| `config`    | Config loading, validation, settings          | `src/config/mod.rs`, `src/config/app_config.rs`, `src/ui/form_validation.rs` |
| `system`    | Lifecycle, persistence, telemetry, diagnostics| `src/lifecycle_manager.rs`, `src/state_persistence.rs`, `src/telemetry.rs`, `src/diagnostics.rs` |

## 5. Command-Line Usage Protocol

- `rustpods` (default): Only warnings and errors are shown.
- `rustpods --quiet`: Errors only.
- `rustpods --debug-ui`: UI debug output enabled.
- `rustpods --debug-bluetooth`: Bluetooth debug output enabled.
- `rustpods --debug-airpods`: AirPods debug output enabled.
- `rustpods --debug-config`: Config debug output enabled.
- `rustpods --debug-system`: System debug output enabled.
- `rustpods --debug-all` or `-v`: All debug categories enabled.

## 6. Implementation Requirements

### 6.1 Required Practices
- Use `debug_log!` macro for all category-specific debug output.
- Use `info!`, `warn!`, and `error!` for always-visible messages.
- Include contextual information in all log messages.
- Prefer structured data over string concatenation.
- Use lazy evaluation for expensive debug computations.

### 6.2 Prohibited Practices
- Do not use `println!()` for debug output.
- Do not use `log::debug!` directly for category-specific output.
- Avoid excessive debug output in tight loops.

## 7. Testing and Verification Protocols

- Test default output to ensure only warnings and errors are visible.
- Test each debug category flag to verify correct filtering.
- Confirm that debug messages are informative and context-rich.
- Use provided test commands to validate logging behavior in all operational modes.

## 8. Migration and Maintenance

- Replace all legacy `println!` and `log::debug!` calls with `debug_log!` macro.
- Update category mappings as new modules are added.
- Monitor performance impact of debug flag checking.
- Maintain documentation and help text for all debug flags.

## 9. Performance Considerations

- The `debug_log!` macro is optimized for minimal overhead when debug output is disabled.
- Atomic flag checking ensures fast runtime performance.
- String formatting and expensive computations are only performed when output is enabled.

## 10. Logging for Battery Intelligence System

The [Battery Intelligence System](battery_intelligence.md) is a critical subsystem requiring rigorous logging and observability. All event logging, error reporting, and diagnostic output for battery analytics must conform to the standards in this document.

### Logging Strategies for Battery Intelligence
- Use the `debug_log!` macro with the `airpods` or `system` category for all significant battery events, model updates, and prediction anomalies.
- Log contextual information: device ID, event type, battery levels, timestamps, computed rates, and confidence metrics.
- Focus on meaningful state transitions, errors, and prediction results.

### Example Log Messages
```rust
crate::debug_log!("airpods", "Battery event: 10% drop detected, device: {}, from {}% to {}%, elapsed: {} min", device_id, prev_level, curr_level, elapsed_min);
crate::debug_log!("airpods", "Depletion rate buffer updated: {} samples, median rate: {:.2} min/10%", buffer_len, median_rate);
crate::debug_log!("airpods", "Prediction: {}% remaining, estimated time-to-empty: {} min, confidence: {}", est_level, time_to_empty, confidence);
crate::debug_log!("system", "Battery intelligence model error: {}", error_msg);
```

## 11. Future Enhancements (Planned)

- Configuration file support for debug flags.
- Runtime toggling of debug categories.
- Per-category log levels.
- Structured (e.g., JSON) log output for automated analysis.
- Integrated performance metrics in debug output.

## 12. Production Readiness and Verification

- The logging system is production-ready, providing a clean user experience and powerful debugging capabilities.
- All debug categories and flag combinations have been verified for correctness and performance.
- Documentation and help text are maintained for all logging features.

---

**This document is maintained as a living reference. Update as the logging system evolves or as new best practices are established.** 