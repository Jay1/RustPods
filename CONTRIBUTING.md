# Contributing to RustPods

Thank you for your interest in contributing to RustPods! This document provides guidelines and instructions for contributing to the project.

> **Note**: RustPods is focused exclusively on Apple AirPods and Beats products using Apple's proprietary Continuity Protocol. We do not support other Bluetooth earbuds (Sony, Bose, Samsung, etc.) due to technical limitations.

## Code of Conduct

Please be respectful and considerate of others when contributing to this project. We aim to foster an inclusive and welcoming community.

## Getting Started

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```sh
   git clone https://github.com/YOUR_USERNAME/RustPods.git
   cd RustPods
   ```
3. Add the original repository as an upstream remote:
   ```sh
   git remote add upstream https://github.com/ORIGINAL_OWNER/RustPods.git
   ```
4. Create a new branch for your changes:
   ```sh
   git checkout -b feature/your-feature-name
   ```

## Development Environment

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

> **Note:** Full AirPods battery monitoring requires Windows. Linux/macOS builds will have limited Bluetooth capabilities.

### Quick Setup

**Option 1: Automated Build (Recommended)**

Use our automated build scripts that handle everything:

```powershell
# Windows (PowerShell)
.\scripts\build_all.ps1

# Linux/macOS (Bash)  
./scripts/build_all.sh
```

**Option 2: Standard Cargo Build**

The project includes automated CLI scanner building:

```sh
# This will automatically build the CLI scanner and Rust app
cargo build

# For release builds
cargo build --release
```

**Option 3: Manual Build**

If the automated builds don't work:

```sh
# 1. Initialize submodules
git submodule update --init --recursive

# 2. Build CLI scanner manually
cd scripts/airpods_battery_cli
cmake -B build -S . -G "Visual Studio 17 2022" -A x64  # Windows
cmake -B build -S . -DCMAKE_BUILD_TYPE=Release         # Linux/macOS
cmake --build build --config Release

# 3. Build Rust application
cd ../..
cargo build --release
```

### Development Workflow

1. **Initial setup**:
   ```sh
   cargo build  # This builds everything automatically
   ```

2. **Run the application**:
   ```sh
   cargo run    # Launch with default UI
   cargo run -- help  # Show command options
   ```

3. **Run tests**:
   ```sh
   cargo test
   ```

4. **Check code quality**:
   ```sh
   cargo fmt     # Format code
   cargo clippy  # Run linter
   ```

### Build System Architecture

RustPods uses a hybrid build system:

- **Rust Application**: Built with Cargo as usual
- **CLI Scanner**: Native C++ component built with CMake
- **Automated Integration**: `build.rs` automatically builds the CLI scanner during `cargo build`
- **Build Scripts**: PowerShell/Bash scripts for complete automation

**Key Files:**
- `build.rs` - Rust build script that builds CLI scanner
- `scripts/build_all.ps1` - Windows automated build script
- `scripts/build_all.sh` - Linux/macOS automated build script
- `scripts/airpods_battery_cli/CMakeLists.txt` - CLI scanner build configuration

## Making Changes

### Code Organization

The project is structured as follows:

```
RustPods/
├── src/                           # Rust source code
│   ├── bluetooth/                 # Bluetooth functionality
│   │   ├── cli_scanner.rs         # CLI scanner integration
│   │   ├── battery_monitor.rs     # Battery monitoring
│   │   └── ...
│   ├── airpods/                   # AirPods-specific logic
│   ├── ui/                        # User interface components
│   └── config/                    # Application configuration
├── scripts/                       # Build scripts and tools
│   ├── airpods_battery_cli/       # Native CLI scanner (C++)
│   ├── build_all.ps1              # Windows build automation
│   └── build_all.sh               # Linux/macOS build automation
├── tests/                         # Integration tests
├── examples/                      # Example code
├── assets/                        # Icons and other assets
├── docs/                          # Documentation
└── third_party/                   # Git submodules (spdlog)
```

### Development Guidelines

1. **Follow Rust conventions**: Use `cargo fmt` and `cargo clippy`
2. **Add tests**: Write unit tests for new functionality
3. **Update documentation**: Keep README and docs current
4. **CLI Scanner Changes**: If modifying the CLI scanner, ensure it builds on Windows
5. **Cross-platform**: Consider Linux/macOS compatibility where possible

### Testing Your Changes

```sh
# Test Rust code
cargo test

# Test CLI scanner (Windows only)
cd scripts/airpods_battery_cli/build/Release
./airpods_battery_cli.exe

# Test full integration
cargo run -- airpods  # Test AirPods detection
cargo run             # Test full UI
```

## Troubleshooting Build Issues

### Common Problems

**1. Git Submodule Issues**
```sh
# Fix: Re-initialize submodules
git submodule update --init --recursive --force
```

**2. CMake Not Found**
```sh
# Windows: Install from https://cmake.org/download/
# Linux: sudo apt install cmake
# macOS: brew install cmake
```

**3. MSVC Not Found (Windows)**
- Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- Or install Visual Studio 2019/2022 with "Desktop development with C++" workload

**4. CLI Scanner Build Fails**
```sh
# Try manual build
cd scripts/airpods_battery_cli
cmake -B build -S . --fresh  # Fresh configuration
cmake --build build --config Release --verbose
```

**5. Rust Compilation Errors**
```sh
# Update Rust toolchain
rustup update

# Clean and rebuild
cargo clean
cargo build
```

### Getting Help

If you encounter issues:

1. **Check the automated build scripts**: They often provide more detailed error messages
2. **Search existing [GitHub Issues](https://github.com/Jay1/RustPods/issues)**
3. **Create a new issue** with:
   - Your operating system and version
   - Rust version (`rustc --version`)
   - CMake version (`cmake --version`)
   - Complete error output
   - Steps you tried

## Commit Guidelines

- Use clear, descriptive commit messages
- Start with a capitalized, imperative verb (e.g., "Add", "Fix", "Update")
- Reference issue numbers if applicable (e.g., "Fix #42: Resolve battery level display issue")
- Keep commits focused on a single change

## Submitting Changes

1. Push your changes to your fork:
   ```sh
   git push origin feature/your-feature-name
   ```
2. Open a pull request against the main repository
3. Provide a clear description of the changes and any relevant issue numbers
4. Be responsive to feedback and be prepared to make additional changes if requested

## Pull Request Process

1. Ensure your code follows the project's style and conventions
2. Update documentation as needed
3. Include screenshots or GIFs for UI changes
4. Add appropriate tests for your changes
5. Your pull request will be reviewed by maintainers
6. Address any feedback or suggestions from the review

## Types of Contributions

We welcome the following types of contributions:

- **Bug fixes** - Especially cross-platform compatibility issues
- **Feature enhancements** - New AirPods features, UI improvements
- **Documentation improvements** - Build guides, API docs, examples
- **Performance optimizations** - Battery polling efficiency, UI responsiveness
- **UI/UX improvements** - Better user experience, accessibility
- **Test coverage improvements** - Unit tests, integration tests
- **Support for additional models** - More AirPods variants, additional Beats products
- **Build system improvements** - Better automation, cross-platform support

## Code Style

- Follow Rust's official style guide
- Use meaningful variable and function names
- Write comments for complex logic
- Include documentation comments for public APIs
- Run `cargo fmt` before submitting your code
- Ensure your code passes `cargo clippy` with no warnings

## Testing

- Write unit tests for your code
- Ensure all tests pass with `cargo test`
- For UI changes, include tests that verify the functionality
- For Bluetooth functionality, provide mocks where appropriate
- Test CLI scanner changes on Windows when possible

## Documentation

- Update documentation to reflect your changes
- Document public APIs with doc comments
- For significant changes, update the relevant guides in the `docs/` directory
- Update the README if adding new features or changing build instructions

## Performance Considerations

- **Battery Polling**: Consider impact on system resources
- **UI Responsiveness**: Ensure UI remains responsive during Bluetooth operations
- **Memory Usage**: Monitor memory usage, especially for long-running operations
- **CLI Scanner Efficiency**: Optimize CLI scanner calls and JSON parsing

## Platform-Specific Notes

### Windows Development
- Full functionality available
- Test CLI scanner integration thoroughly
- Ensure proper error handling for Windows APIs

### Linux/macOS Development
- Limited Bluetooth functionality
- Focus on UI components and core Rust code
- Test build scripts on these platforms

## Security Considerations

- **Public Repository**: Never commit sensitive data
- **CLI Scanner**: Ensure no security vulnerabilities in native code
- **User Privacy**: Respect user privacy in Bluetooth operations

## Licensing

By contributing to this project, you agree that your contributions will be licensed under the project's MIT license.

## Questions?

If you have any questions or need help with contributing, please:

1. Check this CONTRIBUTING.md guide
2. Look at existing issues and pull requests
3. Open a GitHub issue for discussion
4. Reach out to the maintainers

Thank you for contributing to RustPods! 🦀🎧 