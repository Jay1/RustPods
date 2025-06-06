# ⚠️ PUBLIC REPOSITORY NOTICE ⚠️

**IMPORTANT: THIS IS A PUBLIC REPOSITORY. ALWAYS EXERCISE EXTREME CAUTION WHEN PUSHING CODE.**

---

<p align="center">
  <img src="assets/icons/app/logo.png" alt="RustPods Logo" width="400">
</p>

<h1 align="center">RustPods</h1>

<p align="center">
  A simple, elegant battery monitor for Apple AirPods on Windows
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
    <img src="https://codecov.io/gh/Jay1/RustPods/branch/main/graph/badge.svg" alt="Code Coverage">
  </a>
  <a href="https://coderabbit.ai">
    <img src="https://img.shields.io/coderabbit/prs/github/Jay1/RustPods?utm_source=oss&utm_medium=github&utm_campaign=Jay1%2FRustPods&labelColor=171717&color=FF570A&link=https%3A%2F%2Fcoderabbit.ai&label=CodeRabbit+Reviews" alt="CodeRabbit Pull Request Reviews">
  </a>
</p>

---

Hi guys 👋 This little project came about because I wanted a simple, no-fuss way to check my AirPods battery on Windows. I found that many existing tools were either a bit complicated to set up, weren't free, or just didn't quite fit what I needed.

So, I built this: an open-source application crafted with Rust 🦀, designed to monitor Apple AirPods and Beats products easily and reliably on Windows.

## ✨ Features

- 🔋 **Real-time battery monitoring** for AirPods and case
- 🖥️ **Sleek UI** with Catppuccin Mocha theme (❤️)
- 🔍 **Automatic device detection** with Bluetooth LE
- 🔔 **System tray integration** for quick access
- ⚙️ **Customizable settings** for your preferences
- 🚀 **Lightweight and efficient** built with Rust
- 🍎 **Apple ecosystem focused** - AirPods and Beats products only

## 📥 Installation

### Option 1: Download Release (Recommended)

> **Note:** Official releases are coming soon! The repository is currently in active development.

1. Download the latest release from the [Releases](https://github.com/Jay1/RustPods/releases) page (coming soon)
2. Extract the ZIP file to any location
3. Run `RustPods.exe` - no installation required!

### Option 2: Build from Source

See the [Building from Source](#-building-from-source) section below for detailed instructions.

*For detailed usage instructions, see our [Getting Started Guide](docs/user-guide/getting-started.md).*

## 🚀 Quick Start

1. Launch RustPods - it will appear in your system tray
2. Make sure your AirPods are paired and connected with Windows
3. Open the AirPods case or take them out to make them discoverable
4. RustPods will use a native CLI scanner to detect your AirPods and show battery levels

## 🔧 How It Works

RustPods uses a **native C++ CLI scanner** (`scripts/airpods_battery_cli/`) that implements Apple's Continuity Protocol for reliable AirPods detection and battery monitoring. The scanner uses Windows Setup API for BLE device enumeration and provides structured JSON output that the Rust application consumes.

**Key Components:**
- **Native CLI Scanner (v3.1)**: C++ implementation in `scripts/airpods_battery_cli/` with complete Apple Continuity Protocol parsing
- **Rust Application**: Modern UI built with Iced framework and async Bluetooth integration
- **JSON Interface**: Structured data exchange between CLI scanner and Rust app
- **Build Automation**: Cross-platform build scripts for seamless development

**Supported Devices (Apple Ecosystem Only):**
- **AirPods**: 1st, 2nd, 3rd generation
- **AirPods Pro**: 1st and 2nd generation (including USB-C variant)
- **AirPods Max**
- **Beats Products**: Studio Buds, Fit Pro, Powerbeats Pro

> **Note**: RustPods uses Apple's proprietary Continuity Protocol and only supports Apple and Beats devices. Other Bluetooth earbuds (Sony, Bose, Samsung, etc.) will appear in device lists but will not show battery information.

## 🔧 Building from Source

RustPods is designed to be fully open source and buildable by anyone. The project consists of two main components that work together.

### Prerequisites

**Windows (Full Functionality):**
- [Rust toolchain](https://rustup.rs/) (latest stable)
- [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) or Visual Studio 2019/2022 with C++ workload
- [CMake](https://cmake.org/download/) 3.16 or later
- [Git](https://git-scm.com/) (for submodules)

**Linux/macOS (Limited Functionality):**
- [Rust toolchain](https://rustup.rs/) (latest stable)
- C++ compiler (gcc or clang)
- [CMake](https://cmake.org/download/) 3.16 or later
- [Git](https://git-scm.com/) (for submodules)

> **Note:** Full AirPods battery monitoring functionality requires Windows. Linux/macOS builds will have limited Bluetooth capabilities.

### Automated Build (Recommended)

We provide automated build scripts that handle the entire build process:

**Windows (PowerShell):**
```powershell
# Clone the repository
git clone https://github.com/Jay1/RustPods.git
cd RustPods

# Build everything (debug mode)
.\scripts\build_all.ps1

# Build in release mode
.\scripts\build_all.ps1 -Release

# Clean release build
.\scripts\build_all.ps1 -Clean -Release

# Show help for more options
.\scripts\build_all.ps1 -Help
```

**Linux/macOS (Bash):**
```bash
# Clone the repository
git clone https://github.com/Jay1/RustPods.git
cd RustPods

# Build everything (debug mode)
./scripts/build_all.sh

# Build in release mode
./scripts/build_all.sh --release

# Show help for more options
./scripts/build_all.sh --help
```

### Manual Build Steps

If you prefer to build manually or the automated scripts don't work:

#### 1. Initialize Submodules
```bash
git submodule update --init --recursive
```

#### 2. Build the CLI Scanner
```bash
cd scripts/airpods_battery_cli

# Configure with CMake
cmake -B build -S . -G "Visual Studio 17 2022" -A x64  # Windows
cmake -B build -S . -DCMAKE_BUILD_TYPE=Release         # Linux/macOS

# Build
cmake --build build --config Release
```

#### 3. Build the Rust Application
```bash
# From project root
cargo build --release
```

#### 4. Run RustPods
```bash
# Windows
.\target\release\rustpods.exe

# Linux/macOS
./target/release/rustpods
```

### Build Troubleshooting

**Common Issues:**

1. **Submodule errors**: Make sure Git submodules are initialized with `git submodule update --init --recursive`

2. **CMake not found**: Install CMake and ensure it's in your PATH

3. **MSVC not found (Windows)**: Install Visual Studio Build Tools or Visual Studio with C++ workload

4. **Compilation errors**: Ensure you have the latest Rust stable toolchain with `rustup update`

**Getting Help:**

If you encounter build issues:
1. Check the [CONTRIBUTING.md](CONTRIBUTING.md) guide
2. Search existing [GitHub Issues](https://github.com/Jay1/RustPods/issues)
3. Create a new issue with your build error output

## 🖥️ Command-Line Interface

RustPods can be run in different modes through the command line:

```
Usage:
  rustpods adapters    - Discover Bluetooth adapters
  rustpods scan        - Run a basic Bluetooth scan
  rustpods interval    - Run interval-based scanning
  rustpods airpods     - Run AirPods filtering demo
  rustpods events      - Run event system demo (use cargo run --example event_system)
  rustpods ui          - Launch the UI application with original state management
  rustpods stateui     - Launch the UI application with new state management (default)
  rustpods help        - Show this help message
```

When run without any arguments, RustPods defaults to `stateui` mode, launching the main application with the new state management system.

### Examples:

```sh
# Launch the main UI application (default)
rustpods

# Show all available Bluetooth adapters on your system
rustpods adapters

# Run a scan for AirPods devices
rustpods airpods

# Show command-line help
rustpods help
```

## 📖 Documentation

Visit our [documentation](docs/index.md) for detailed guides:

- [User Guide](docs/user-guide/getting-started.md)
- [Development Guide](docs/development/assets.md)
- [Technical Documentation](docs/development/assets.md)

## 🤝 Contributing

Contributions are welcome! Check out our [CONTRIBUTING.md](CONTRIBUTING.md) guide to get started.

**For Contributors:**
- The automated build scripts in `scripts/` make it easy to build the entire project
- Both the Rust application and C++ CLI scanner are fully open source
- See our coding guidelines and project structure in the contributing guide

## 🛠️ Development Status

The project currently implements:
- ✅ **Complete Apple Continuity Protocol parsing** in native CLI scanner v3.1
- ✅ **AirPods battery monitoring** with charging state detection
- ✅ **All AirPods and Beats models** supported (Pro, Pro 2, Max, Studio Buds, etc.)
- ✅ **Modern Rust UI** with Iced framework and dark theme
- ✅ **System tray integration** and configuration persistence
- ✅ **JSON data interface** between CLI scanner and Rust app
- ✅ **Comprehensive build automation** for open source development

**Current Focus:**
- Real-time BLE advertisement scanning integration
- Performance optimization for battery polling
- Enhanced error handling and user feedback

## 📄 License

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgements

Special thanks to the [AirPodsDesktop project](https://github.com/SpriteOvO/AirPodsDesktop) by SpriteOvO. Our AirPods battery CLI scanner is based on their excellent open-source work and provides the foundation for Apple Continuity Protocol parsing. 