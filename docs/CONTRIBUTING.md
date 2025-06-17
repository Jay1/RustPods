# RustPods Contribution Protocol: Technical Standards and Procedures

## Introduction

This document delineates the formal procedures and technical standards for contributing to the RustPods project. RustPods is engineered exclusively for Apple AirPods and Beats products utilizing Apple's proprietary Continuity Protocol. Contributions pertaining to unsupported Bluetooth devices (e.g., Sony, Bose, Samsung) are not within project scope due to technical constraints.

## Code of Conduct

Contributors are required to maintain a professional, respectful, and inclusive environment. All interactions must reflect the highest standards of collegiality and technical discourse.

## Repository Initialization

1. Fork the repository on GitHub.
2. Clone the forked repository:
   ```sh
   git clone https://github.com/YOUR_USERNAME/RustPods.git
   cd RustPods
   ```
3. Add the original repository as an upstream remote:
   ```sh
   git remote add upstream https://github.com/ORIGINAL_OWNER/RustPods.git
   ```
4. Create a dedicated feature branch:
   ```sh
   git checkout -b feature/your-feature-name
   ```

## Development Environment Requirements

### Windows (Full Functionality)
- Rust toolchain (latest stable)
- Microsoft C++ Build Tools or Visual Studio 2019/2022 with C++ workload
- CMake 3.16 or later
- Git (for submodules)

### Linux/macOS (Limited Functionality)
- Rust toolchain (latest stable)
- C++ compiler (gcc or clang)
- CMake 3.16 or later
- Git (for submodules)

> **Note:** Full AirPods battery monitoring is supported exclusively on Windows. Linux/macOS builds provide limited Bluetooth functionality.

## Build and Execution Procedures

### Automated Build (Recommended)

Utilize the provided automation scripts:
```powershell
# Windows
./scripts/build_all.ps1
# Linux/macOS
./scripts/build_all.sh
```

### Standard Cargo Build

```sh
cargo build           # Builds both CLI scanner and Rust application
cargo build --release # Release build
```

### Manual Build Sequence

```sh
# Initialize submodules
git submodule update --init --recursive
# Build CLI scanner
cd scripts/airpods_battery_cli
cmake -B build -S . -G "Visual Studio 17 2022" -A x64  # Windows
cmake -B build -S . -DCMAKE_BUILD_TYPE=Release         # Linux/macOS
cmake --build build --config Release
# Build Rust application
cd ../..
cargo build --release
```

## Project Structure

```
RustPods/
├── src/                           # Rust source code
│   ├── bluetooth/                 # Bluetooth functionality
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
└── docs/                          # Documentation
```

## Development Protocols

- Adhere to Rust conventions: execute `cargo fmt` and `cargo clippy` prior to submission.
- Implement comprehensive unit tests for all new functionality.
- Maintain and update documentation in parallel with code changes.
- For CLI scanner modifications, ensure successful Windows builds.
- Consider cross-platform compatibility in all contributions.

## Testing Procedures

```sh
cargo test                         # Execute Rust test suite
cd scripts/airpods_battery_cli/build/Release
./airpods_battery_cli.exe          # Test CLI scanner (Windows only)
cargo run -- airpods               # Test AirPods detection
cargo run                          # Launch full UI
```

## Troubleshooting and Diagnostics

- **Submodule Initialization:**
  ```sh
  git submodule update --init --recursive --force
  ```
- **CMake Installation:**
  - Windows: https://cmake.org/download/
  - Linux: `sudo apt install cmake`
  - macOS: `brew install cmake`
- **MSVC Installation:**
  - Install Visual Studio Build Tools or Visual Studio with C++ workload
- **CLI Scanner Build Failure:**
  ```sh
  cd scripts/airpods_battery_cli
  cmake -B build -S . --fresh
  cmake --build build --config Release --verbose
  ```
- **Rust Compilation Errors:**
  ```sh
  rustup update
  cargo clean
  cargo build
  ```

## Commit Standards

- Employ clear, imperative commit messages (e.g., "Add", "Fix", "Update").
- Reference issue numbers where applicable (e.g., "Fix #42: Resolve battery level display issue").
- Restrict commits to a single, focused change.

## Submission Workflow

1. Push changes to the feature branch:
   ```sh
   git push origin feature/your-feature-name
   ```
2. Open a pull request against the main repository.
3. Provide a precise description of the changes and reference relevant issues.
4. Respond promptly to review feedback and implement requested modifications.

## Pull Request Review Criteria

- Conformance to project style and conventions
- Documentation updates as required
- Inclusion of test evidence for UI changes
- Adequate test coverage for new or modified code
- Responsiveness to reviewer feedback

## Accepted Contribution Types

- Defect remediation (bug fixes)
- Feature enhancements
- Documentation improvements
- Performance optimizations
- User interface and accessibility improvements
- Test coverage expansion
- Support for additional AirPods/Beats models
- Build system and automation enhancements

## Code Style and Documentation

- Adhere to Rust's official style guide
- Employ descriptive, unambiguous identifiers
- Document complex logic and all public APIs
- Execute `cargo fmt` and ensure `cargo clippy` passes without warnings

## Testing and Validation

- Implement unit tests for all new code
- Ensure all tests pass (`cargo test`)
- For UI changes, provide functional verification
- For Bluetooth features, utilize mocks as appropriate
- Validate CLI scanner changes on Windows

## Documentation Protocols

- Update documentation to reflect all substantive changes
- Document public APIs with doc comments
- For major changes, update guides in the `docs/` directory
- Revise the README for new features or build instructions

## Performance and Platform Considerations

- Optimize for minimal system resource consumption
- Ensure UI responsiveness under all operational conditions
- Monitor and manage memory usage
- Optimize CLI scanner invocation and data parsing
- Windows is the primary target; ensure graceful degradation on Linux/macOS

## Security and Licensing

- Never commit sensitive or confidential data
- Ensure native code is free from security vulnerabilities
- Respect user privacy in all Bluetooth operations
- All contributions are licensed under the project's MIT license

## Support and Inquiries

For technical assistance or procedural clarification:
1. Consult this protocol document
2. Review existing issues and pull requests
3. Open a GitHub issue for further discussion
4. Contact project maintainers as necessary

Thank you for contributing to RustPods! 🦀🎧 