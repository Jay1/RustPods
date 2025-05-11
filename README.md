# RustPods

A Rust application for monitoring Apple AirPods and other Bluetooth headphones.

## Project Overview

RustPods is designed to provide a cross-platform desktop application that:
- Displays real-time status of connected AirPods/Bluetooth headphones
- Monitors battery levels for AirPods devices
- Provides system tray integration for easy access
- Detects AirPods devices using Bluetooth Low Energy scanning

## Setup

1. Ensure you have Rust and Cargo installed. See https://www.rust-lang.org/tools/install
2. Clone this repository:
   ```sh
   git clone https://github.com/your-username/RustPods.git
   cd RustPods
   ```
3. The project uses the following dependencies (already configured in `Cargo.toml`):
   - btleplug - Bluetooth Low Energy library
   - iced - Modern GUI library for Rust
   - tray-item - System tray integration 
   - tokio - Async runtime
   - thiserror - Error handling
   - async-stream - Async stream utilities
4. Build the project:
   ```sh
   cargo build
   ```

## Usage

- Run the application:
  ```sh
  cargo run
  ```
- The application will initialize the system tray icon
- Use the system tray menu to:
  - Start/stop Bluetooth scanning
  - Open the main application window
  - Exit the application

## Project Structure

- `src/` - Main source code directory
  - `main.rs` - Application entry point
  - `app/` - Core application logic
  - `bluetooth/` - Bluetooth scanning and event system
    - `adapter.rs` - Bluetooth adapter management
    - `events.rs` - Event broker system for Bluetooth events
    - `scanner.rs` - BLE scanning functionality
    - `scanner_config.rs` - Configuration for BLE scanning
    - `examples.rs` - Example usages of BLE scanning
  - `airpods/` - AirPods-specific functionality
    - `detector.rs` - AirPods detection from BLE advertisements
    - `filter.rs` - Filtering functions for AirPods devices
    - `mod.rs` - Type definitions and data parsing
  - `config/` - Configuration and settings management
  - `ui/` - User interface implementation
  - `error.rs` - Error handling utilities

## Event System

The project implements a flexible event system for handling Bluetooth device events:
- `EventBroker` manages subscribers and distributes events
- `EventFilter` allows for filtering events by type, device, or custom criteria
- Subscribers can receive only the events they're interested in

## AirPods Detection

RustPods can detect various AirPods models:
- AirPods (1st and 2nd generation)
- AirPods Pro (1st and 2nd generation)
- AirPods 3
- AirPods Max

The detection system decodes manufacturer data from BLE advertisements to extract:
- Device type identification
- Battery levels for left, right, and case
- Charging status

## UI Implementation

This project uses iced for the user interface. Some key aspects of our implementation:

- **Declarative UI**: The interface is built by composing widgets in a functional, declarative style
- **Message-based Architecture**: All UI state changes are driven by messages processed in the update function
- **System Tray Integration**: A non-intrusive system tray icon provides access to application functions

Our implementation follows the patterns documented in our [icedPatterns.mdc](.cursor/rules/icedPatterns.mdc) file.

## Development Status

The project has implemented:
- Bluetooth device scanning and discovery
- AirPods device detection and identification
- Event broker system for Bluetooth events
- Basic UI elements and system tray integration

Active development continues with a focus on:
- Improving UI functionality
- Enhancing AirPods detection reliability
- Adding support for more Bluetooth headphone types
- Cross-platform testing and optimization

## License

MIT 