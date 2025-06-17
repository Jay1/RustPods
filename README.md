# RustPods

<p align="center">
  <img src="assets/icons/app/battery_ring_80_percent.svg" alt="RustPods Logo" width="200">
</p>

<p align="center">
  Advanced battery monitoring solution for Apple AirPods on Windows
</p>

<p align="center">
  <a href="https://github.com/Jay1/RustPods/actions/workflows/rust-ci.yml">
    <img src="https://github.com/Jay1/RustPods/actions/workflows/rust-ci.yml/badge.svg" alt="Rust CI">
  </a>
  <a href="https://opensource.org/licenses/MIT">
    <img src="https://img.shields.io/badge/License-MIT-yellow.svg" alt="License: MIT">
  </a>
  <img src="https://img.shields.io/badge/platform-windows-blue" alt="Platform: Windows">
  <img src="https://img.shields.io/badge/Powered%20by-Rust-orange" alt="Powered by: Rust">
  <a href="https://codecov.io/gh/Jay1/RustPods">
    <img src="https://codecov.io/gh/Jay1/RustPods/graph/badge.svg" alt="Code Coverage">
  </a>
  <a href="https://coderabbit.ai">
    <img src="https://img.shields.io/coderabbit/prs/github/Jay1/RustPods?utm_source=oss&utm_medium=github&utm_campaign=Jay1%2FRustPods&labelColor=171717&color=FF570A&link=https%3A%2F%2Fcoderabbit.ai&label=CodeRabbit+Reviews" alt="CodeRabbit Pull Request Reviews">
  </a>
</p>

---

RustPods is an open-source, production-grade application engineered for precise battery monitoring of Apple AirPods and Beats products on Windows platforms. The system leverages a hybrid C++/Rust architecture, implementing Apple's proprietary Continuity Protocol via Bluetooth Low Energy (BLE) for high-fidelity telemetry acquisition and robust system integration.

## Technical Capabilities

- **Battery Telemetry**: Real-time acquisition and display of individual AirPods and charging case battery levels
- **Bluetooth LE Integration**: Native Windows Setup API for reliable device enumeration and communication
- **Apple Continuity Protocol**: Complete protocol parsing for proprietary battery and status data extraction
- **System Integration**: Windows system tray interface for persistent, background operation
- **Configuration Management**: Persistent user settings and preference storage
- **Cross-Architecture Implementation**: Modular C++ CLI scanner and Rust application for performance and maintainability
- **Device Compatibility**: Exclusive support for Apple AirPods and Beats product lines

## System Requirements

- **Operating System**: Windows 10 (Build 1903) or later
- **Hardware**: Bluetooth Low Energy 4.0+ compatible adapter
- **Device**: Apple AirPods or Beats, paired via Windows Bluetooth configuration

## Architecture Overview

RustPods employs a hybrid architecture:

- **Native CLI Scanner (C++)**: Located in `scripts/airpods_battery_cli/`, provides Apple Continuity Protocol parsing and structured JSON output
- **Rust Application**: Modern UI built with the Iced framework, asynchronous Bluetooth integration, and robust state management
- **Data Exchange**: Structured JSON protocol between CLI scanner and Rust frontend
- **Build Automation**: Cross-platform build system for streamlined development and deployment

**Device Compatibility:**
- AirPods (1st, 2nd, 3rd generation)
- AirPods Pro (1st, 2nd generation, including USB-C variant)
- AirPods Max
- Beats Studio Buds, Fit Pro, Powerbeats Pro

> **Note:** Only devices implementing Apple's Continuity Protocol are supported. Third-party Bluetooth earbuds are not supported for battery telemetry.

## Build and Deployment

### Prerequisites

**Windows (Complete Functionality):**
- [Rust toolchain](https://rustup.rs/) (latest stable)
- [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) or Visual Studio 2019/2022 with C++ workload
- [CMake](https://cmake.org/download/) 3.16 or later
- [Git](https://git-scm.com/) (for submodule management)

**Linux/macOS (Limited Functionality):**
- [Rust toolchain](https://rustup.rs/) (latest stable)
- C++ compiler (gcc or clang)
- [CMake](https://cmake.org/download/) 3.16 or later
- [Git](https://git-scm.com/) (for submodule management)

### Automated Build

**Windows (PowerShell):**
```powershell
git clone https://github.com/Jay1/RustPods.git
cd RustPods
./scripts/build_all.ps1
```

**Linux/macOS (Bash):**
```bash
git clone https://github.com/Jay1/RustPods.git
cd RustPods
./scripts/build_all.sh
```

### Manual Build Sequence

1. **Submodule Initialization**
   ```bash
   git submodule update --init --recursive
   ```
2. **CLI Scanner Compilation**
   ```bash
   cd scripts/airpods_battery_cli
   cmake -B build -S . -G "Visual Studio 17 2022" -A x64  # Windows
   cmake -B build -S . -DCMAKE_BUILD_TYPE=Release         # Linux/macOS
   cmake --build build --config Release
   ```
3. **Rust Application Compilation**
   ```bash
   cd ../..
cargo build --release
   ```
4. **Application Execution**
   ```bash
   # Windows
   .\target\release\rustpods.exe
   # Linux/macOS
   ./target/release/rustpods
   ```

## Operational Modes

RustPods provides multiple execution modes for distinct operational requirements:

```
rustpods adapters    # Enumerate available Bluetooth adapters
rustpods scan        # Execute basic Bluetooth device scan
rustpods interval    # Initiate interval-based scanning protocol
rustpods airpods     # Execute AirPods-specific filtering demonstration
rustpods events      # Launch event system demonstration
rustpods ui          # Launch UI application (legacy state management)
rustpods stateui     # Launch UI application (modern state management, default)
rustpods help        # Display command reference
```

Default execution launches `stateui` mode with modern state management.

## Debug and Logging System

RustPods implements a sophisticated configurable logging system that provides clean output by default while offering powerful selective debugging capabilities for developers and troubleshooting.

**Log Levels:**
```sh
rustpods --quiet        # Errors only
rustpods               # Warnings and errors (default)
rustpods --info        # Info, warnings, and errors
rustpods --debug       # Debug, info, warnings, and errors
rustpods --trace       # All log messages
```

**Debug Categories:**
```sh
rustpods --debug-ui              # UI events, window management, system tray
rustpods --debug-bluetooth       # Bluetooth scanning, device discovery, CLI scanner
rustpods --debug-airpods         # AirPods detection, battery parsing
rustpods --debug-config          # Configuration loading, saving, validation
rustpods --debug-system          # System operations, lifecycle, persistence
rustpods --debug-all             # All debug categories
rustpods -v                      # Same as --debug-all
```

For detailed logging implementation, see [docs/development/logging-best-practices.md](docs/development/logging-best-practices.md).

## Documentation

Comprehensive technical documentation is available in the [docs](docs/index.md) directory, including:
- [User Guide](docs/user-guide/getting-started.md)
- [Development Guide](docs/development/assets.md)
- [Technical Documentation](docs/development/assets.md)

## Contribution Protocol

Contributions are accepted in accordance with the [Development Contribution Protocol](docs/CONTRIBUTING.md). All submissions must adhere to project standards for code quality, documentation, and testing.

## License

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Licensed under the MIT License - refer to [LICENSE](LICENSE) file for complete terms.

## Technical References

This project incorporates architectural foundations from the [AirPodsDesktop project](https://github.com/SpriteOvO/AirPodsDesktop) by SpriteOvO. The CLI scanner component extends their Apple Continuity Protocol parsing implementation. 