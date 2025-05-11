# RustPods

A Rust application for monitoring and controlling Apple AirPods and other Bluetooth headphones.

## Project Overview

RustPods is designed to provide a cross-platform desktop application that:
- Displays real-time status of connected AirPods/Bluetooth headphones
- Allows controlling device features like noise cancellation and transparency mode
- Monitors battery levels and provides notifications
- Integrates with the system tray for quick access to common controls

## Setup

1. Ensure you have Rust and Cargo installed. See https://www.rust-lang.org/tools/install
2. Clone this repository:
   ```sh
   git clone https://github.com/Jay1/RustPods.git
   cd RustPods
   ```
3. The project uses the following dependencies (already configured in `Cargo.toml`):
   - btleplug = "0.10" - Bluetooth Low Energy library
   - iced = "0.13" - Modern GUI library for Rust
   - tray-item = "0.7" - System tray integration 
   - serde = "1.0" with derive features - For data serialization/deserialization
   - serde_json = "1.0" - JSON support
   - tokio = "1.0" with full features - Async runtime
4. Build the project:
   ```sh
   cargo build
   ```

## Usage

- Run the application:
  ```sh
  cargo run
  ```
- The application will show a simple UI and create a system tray icon
- Connect your AirPods or other Bluetooth headphones to your computer first
- Use the UI or system tray menu to interact with your headphones

## Project Structure

- `src/` - Main source code directory
  - `main.rs` - Application entry point
  - `app/` - Core application logic
  - `bluetooth/` - Bluetooth communication module
  - `airpods/` - AirPods-specific functionality
  - `config/` - Configuration and settings management
  - `ui/` - User interface implementation
  - `error.rs` - Error handling utilities

## UI Implementation

This project uses iced 0.13 for the user interface. Some key aspects of our implementation:

- **Declarative UI**: The interface is built by composing widgets in a functional, declarative style
- **Message-based Updates**: All UI state changes are driven by messages processed in the update function
- **Task-based Asynchronous Operations**: Long-running operations use iced's Task system
- **Centralized UI Management**: The `UiManager` struct provides a simple interface for launching the UI

Our implementation follows the pattern shown in the `src/ui/mod.rs` file, which serves as the entry point for the UI subsystem. For more details on iced patterns, see our [iced_patterns.mdc](.cursor/rules/iced_patterns.mdc) file.

### Minimal Example

A simple example implementation is provided in the `iced_minimal` directory, demonstrating the basic patterns used throughout the application. This serves as a reference for understanding iced 0.13's API structure.

## Development Status

Currently, the project has completed its initial setup phase (Task 1) including:
- Project initialization with dependencies
- Module structure establishment
- Basic configuration module
- Core application structure
- Minimal UI shell with system tray integration

Next phases will focus on implementing Bluetooth device discovery and connection, specific AirPods functionality, and the complete user interface.

## License

MIT 