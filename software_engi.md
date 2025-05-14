# RustPods: AirPods Battery Monitor for Windows

## Project Overview

RustPods (working name: RustPodsMon) is a lightweight, native Windows application built in Rust for displaying the battery levels of connected Apple AirPods. The application monitors the left earbud, right earbud, and charging case battery levels through Bluetooth Low Energy (BLE) communication.

## Target Users

Windows users who own Apple AirPods and want an efficient way to monitor battery status directly on their desktop.

## Success Metrics

- Accurate and timely display of battery levels
- Low system resource consumption
- High user satisfaction regarding ease of use and stability
- Successful detection and battery reporting for common AirPods models

## Technical Architecture

### Core Components

1. **Bluetooth LE Interaction Layer** (Using `btleplug` crate)
   - Adapter discovery and selection
   - Device scanning and filtering
   - Accessing advertisement data
   - Handling Bluetooth events and disconnections

2. **AirPods Data Parsing Logic**
   - Interpreting BLE advertisement data format for AirPods battery status
   - Structures to hold battery state (L, R, Case)

3. **UI Layer** (Using `iced` framework)
   - Main window with battery indicators
   - Settings panel
   - Catppuccin Mocha theme implementation

4. **System Tray Integration** (Using platform-specific APIs)
   - System tray icon with battery indicators
   - Context menu and tooltips
   - Show/hide functionality

5. **Configuration Management** (Using `serde` with JSON)
   - Loading and saving user settings
   - Configuration validation

6. **State Management**
   - Centralized state store for application data
   - Message-based architecture for state updates
   - Proper state synchronization between UI and background tasks

## Key Features

### Core Bluetooth Functionality

- Continuous scanning for BLE devices with AirPods filtering
- Real-time battery level extraction and updates
- Support for multiple AirPods models (standard, Pro, Max)
- Efficient battery status polling with adaptive intervals

### User Interface

- Compact, non-intrusive main window with battery percentages
- Visual indicators with appropriate color coding (green/yellow/red)
- Battery level animations and charging status indicators
- Connection status display with troubleshooting guidance
- Dark theme support with Catppuccin Mocha color scheme

### System Integration

- System tray icon with battery level tooltips
- Context menu with quick actions
- Start with Windows option
- Window visibility management (show/hide, minimize to tray)
- Keyboard shortcuts for common actions

### Settings and Configuration

- Launch on startup option
- Refresh interval configuration
- UI theme settings
- Configuration persistence between sessions
- Window position memory

## Project Status

The project is under active development with several components already implemented:

1. ✅ **Project structure and dependencies** established
2. ✅ **Bluetooth LE scanning** implemented
3. ✅ **AirPods detection logic** implemented
4. ✅ **Battery data extraction** implemented
5. ✅ **Core application state management** implemented
6. ✅ **Configuration management** implemented
7. ✅ **Main UI window** designed and implemented
8. ✅ **Settings UI window** implemented
9. ✅ **System tray integration** implemented
10. ✅ **UI-Application state integration** completed
11. ✅ **Application entry point** implemented
12. ⏱️ **Error handling and logging** in progress
13. ⏱️ **Application packaging and installation** pending
14. ⏱️ **Automated testing** partially implemented
15. ⏱️ **Documentation and user guide** pending
16. ✅ **Catppuccin Mocha theme** implemented
17. ✅ **MVP stabilization** completed
18. ✅ **UI testing suite** implemented

## Technical Details

### Directory Structure

```
src/
├── main.rs                  // Application entry point
├── lib.rs                   // Library exports
├── app/                     // Main application logic
├── app_controller.rs        // Main application controller
├── app_state_controller.rs  // State controller
├── bluetooth/               // Bluetooth functionality
│   ├── adapter.rs           // Bluetooth adapter management
│   ├── battery.rs           // Battery information handling
│   ├── scanner.rs           // BLE scanning functionality
│   └── ...
├── airpods/                 // AirPods-specific logic
│   ├── detector.rs          // AirPods detection
│   └── filter.rs            // AirPods filtering
├── ui/                      // User interface components
│   ├── components/          // UI components
│   ├── state.rs             // Application state
│   ├── theme.rs             // Theming and styling
│   └── ...
├── config/                  // Application configuration
├── error.rs                 // Error handling
└── ...
```

### Key Technical Approaches

1. **Bluetooth Communication**
   - BLE scanning with device filtering
   - Advertisement data parsing for battery information
   - Battery level extraction and normalization (0-100%)
   - Charging status detection 

2. **Asynchronous Architecture**
   - Tokio runtime for async operations
   - Channel-based communication between UI and background tasks
   - Message-based state updates
   - Proper resource cleanup

3. **UI Framework**
   - Iced framework for native Windows UI
   - Custom components for battery visualization
   - Responsive layout design
   - Catppuccin Mocha theme integration

4. **State Management**
   - Reactive state management pattern
   - Clear data flow architecture
   - Thread-safe state access
   - State persistence

5. **Error Handling and Recovery**
   - Structured logging with different verbosity levels
   - Categorized error types for different subsystems
   - Automatic recovery procedures
   - Diagnostic mode for troubleshooting

## Current Challenges

1. **Bluetooth Device Compatibility**
   - Some Bluetooth adapters have different behaviors
   - Need to handle various AirPods models and firmware versions

2. **Battery Data Reliability**
   - Advertisement data from AirPods can be inconsistent
   - Need to implement data validation and smoothing

3. **UI Performance**
   - Ensuring UI remains responsive during intensive BLE operations
   - Balancing update frequency and resource usage

4. **Cross-Platform Considerations**
   - Primary focus on Windows, but design should allow for future expansion

## Testing Strategy

1. **Unit Testing**
   - Core functionality modules (bluetooth, configuration, state management)
   - UI component tests
   - Test fixtures and mocks for reliable testing

2. **Integration Testing**
   - End-to-end workflow tests
   - System tray and UI interaction tests
   - State propagation tests

3. **Manual Testing**
   - Bluetooth device interaction
   - System tray appearance on different platforms
   - UI responsiveness

## Future Enhancements (Post v1.0)

1. Support for other Bluetooth headphones/earbuds
2. Customizable UI themes/skins
3. Notifications for low battery levels
4. Displaying AirPods model/firmware version
5. More detailed connection status and troubleshooting information
6. Localization/Internationalization support

## Development Workflow

1. Task breakdown using Task Master
2. Prioritization based on core functionality
3. Comprehensive testing of critical components
4. Regular integration testing
5. Documentation updates in parallel with development

## Critical Dependencies

1. **btleplug** (v0.10) - Bluetooth LE functionality
2. **iced** (latest) - UI framework
3. **tokio** (v1.25+) - Async runtime
4. **serde** (v1.0) - Configuration serialization
5. Platform-specific libraries for system tray integration

## Technical Decisions

1. **Rust Language:** Chosen for performance, reliability, and strong type system
2. **Iced UI Framework:** Selected for native look and feel with Rust integration
3. **Tokio Runtime:** Used for efficient async operations
4. **Catppuccin Mocha Theme:** Applied for visual consistency and dark mode support
5. **Message-Based Architecture:** Implemented for clean state management and UI updates 