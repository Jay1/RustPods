# RustPods

<p align="center">
  <img src="battery_ring_80_percent.svg" alt="RustPods Logo" width="400">
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

RustPods is an open-source application designed for monitoring Apple AirPods and Beats products on Windows platforms. Built with Rust, it provides reliable battery monitoring through implementation of Apple's proprietary Continuity Protocol via Bluetooth Low Energy communication.

## Features

- **Real-time battery monitoring** for AirPods and case components
- **Native user interface** with Catppuccin Mocha theme
- **Automatic device detection** using Bluetooth LE scanning
- **System tray integration** for persistent monitoring
- **Configurable settings** for customization
- **High-performance architecture** leveraging Rust's memory safety
- **Apple ecosystem compatibility** - exclusively supports AirPods and Beats products

## Installation

### Binary Distribution

> **Note:** Official releases are currently in preparation. The repository is in active development.

1. Download the latest release from the [Releases](https://github.com/Jay1/RustPods/releases) page
2. Extract the archive to desired location
3. Execute `RustPods.exe` - no installation required

### Source Build

Refer to the [Building from Source](#building-from-source) section for compilation instructions.

For comprehensive usage documentation, consult the [Getting Started Guide](docs/user-guide/getting-started.md).

## System Requirements

RustPods requires paired AirPods connected to Windows. The application automatically detects discoverable devices when the AirPods case is opened or earbuds are removed.

## Architecture

RustPods implements a hybrid architecture combining native C++ scanning capabilities with a modern Rust frontend:

**Core Components:**
- **Native CLI Scanner (v3.1)**: C++ implementation in `scripts/airpods_battery_cli/` providing complete Apple Continuity Protocol parsing
- **Rust Application**: Modern UI built with Iced framework and asynchronous Bluetooth integration  
- **JSON Interface**: Structured data exchange protocol between CLI scanner and Rust application
- **Build Automation**: Cross-platform build system for streamlined development

The system uses Windows Setup API for BLE device enumeration and provides structured JSON output consumed by the Rust application layer.

**Device Compatibility (Apple Ecosystem Only):**
- **AirPods**: 1st, 2nd, 3rd generation
- **AirPods Pro**: 1st and 2nd generation (including USB-C variant)
- **AirPods Max**  
- **Beats Products**: Studio Buds, Fit Pro, Powerbeats Pro

> **Technical Note**: RustPods exclusively supports devices implementing Apple's proprietary Continuity Protocol. Third-party Bluetooth earbuds (Sony, Bose, Samsung, etc.) will be enumerated but will not provide battery telemetry.

## Building from Source

RustPods maintains full open-source buildability with comprehensive toolchain support.

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

> **Platform Note:** Complete AirPods battery monitoring functionality requires Windows. Linux/macOS builds provide limited Bluetooth capabilities.

### Automated Build Process

Automated build scripts handle complete compilation workflow:

**Windows (PowerShell):**
```powershell
# Repository acquisition
git clone https://github.com/Jay1/RustPods.git
cd RustPods

# Debug build
.\scripts\build_all.ps1

# Release build
.\scripts\build_all.ps1 -Release

# Clean release build
.\scripts\build_all.ps1 -Clean -Release

# Display build options
.\scripts\build_all.ps1 -Help
```

**Linux/macOS (Bash):**
```bash
# Repository acquisition
git clone https://github.com/Jay1/RustPods.git
cd RustPods

# Debug build
./scripts/build_all.sh

# Release build
./scripts/build_all.sh --release

# Display build options
./scripts/build_all.sh --help
```

### Manual Compilation

For manual build control or script troubleshooting:

#### 1. Submodule Initialization
```bash
git submodule update --init --recursive
```

#### 2. CLI Scanner Compilation
```bash
cd scripts/airpods_battery_cli

# CMake configuration
cmake -B build -S . -G "Visual Studio 17 2022" -A x64  # Windows
cmake -B build -S . -DCMAKE_BUILD_TYPE=Release         # Linux/macOS

# Compilation
cmake --build build --config Release
```

#### 3. Rust Application Compilation
```bash
# Execute from project root
cargo build --release
```

#### 4. Application Execution
```bash
# Windows
.\target\release\rustpods.exe

# Linux/macOS
./target/release/rustpods
```

### Build Troubleshooting

**Common Resolution Strategies:**

1. **Submodule initialization failures**: Execute `git submodule update --init --recursive`
2. **CMake detection issues**: Verify CMake installation and PATH configuration
3. **MSVC compiler absence (Windows)**: Install Visual Studio Build Tools with C++ workload
4. **Rust compilation errors**: Update to latest stable toolchain via `rustup update`

**Support Resources:**

For build-related issues:
1. Review [CONTRIBUTING.md](CONTRIBUTING.md) documentation
2. Search existing [GitHub Issues](https://github.com/Jay1/RustPods/issues)
3. Submit new issue with complete error output

## Command-Line Interface

RustPods provides multiple execution modes for different operational requirements:

```
Usage:
  rustpods adapters    - Enumerate available Bluetooth adapters
  rustpods scan        - Execute basic Bluetooth device scan
  rustpods interval    - Initiate interval-based scanning protocol
  rustpods airpods     - Execute AirPods-specific filtering demonstration
  rustpods events      - Launch event system demonstration
  rustpods ui          - Launch UI application with legacy state management
  rustpods stateui     - Launch UI application with modern state management (default)
  rustpods help        - Display command reference
```

Default execution without arguments launches `stateui` mode with modern state management architecture.

### Usage Examples:

```sh
# Launch primary UI application (default behavior)
rustpods

# Enumerate system Bluetooth adapters
rustpods adapters

# Execute AirPods device scan
rustpods airpods

# Display command reference
rustpods help
```

## Documentation

Comprehensive documentation available in the [docs](docs/index.md) directory:

- [User Guide](docs/user-guide/getting-started.md)
- [Development Guide](docs/development/assets.md)
- [Technical Documentation](docs/development/assets.md)

## Contributing

Contributions are accepted following established protocols. Review [CONTRIBUTING.md](CONTRIBUTING.md) for submission guidelines.

**Developer Notes:**
- Automated build scripts in `scripts/` provide complete project compilation
- Both Rust application and C++ CLI scanner maintain open-source licensing
- Development guidelines and project architecture documented in contributing guide

## Development Status

**Implemented Features:**
- Complete Apple Continuity Protocol parsing in native CLI scanner v3.1
- AirPods battery monitoring with charging state detection
- Comprehensive AirPods and Beats model support (Pro, Pro 2, Max, Studio Buds, etc.)
- Modern Rust UI implementation with Iced framework and dark theme
- System tray integration with configuration persistence
- JSON data interface between CLI scanner and Rust application
- Comprehensive build automation for open-source development

**Current Development Focus:**
- Real-time BLE advertisement scanning integration
- Performance optimization for battery polling protocols
- Enhanced error handling and user feedback systems

## License

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Licensed under the MIT License - refer to [LICENSE](LICENSE) file for complete terms.

## Acknowledgements

This project incorporates foundational work from the [AirPodsDesktop project](https://github.com/SpriteOvO/AirPodsDesktop) by SpriteOvO. The AirPods battery CLI scanner builds upon their open-source implementation of Apple Continuity Protocol parsing. 