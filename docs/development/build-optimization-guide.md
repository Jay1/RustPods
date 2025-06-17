# RustPods Build Optimization Guide: Technical Reference and Current Best Practices

## Preamble: Document Scope and Relevance

This guide documents the build and test optimization strategies for the RustPods project. It is intended for developers maintaining or extending the system, particularly where native C++ integration, custom build scripts, and a large test suite impact iteration speed. All recommendations and examples are current as of the latest project structure. Legacy or deprecated techniques are clearly marked.

## 1. Current Build System Overview

- **Hybrid Architecture**: RustPods employs a Rust application with a native C++ CLI scanner (built via CMake) for Apple Continuity Protocol parsing.
- **Custom build.rs**: The Rust build script (`build.rs`) manages native dependency compilation and artifact tracking.
- **Test Suite**: The project includes both fast unit tests and heavier integration tests, some of which are disabled by default for development speed.

## 2. Performance Optimization Techniques

### 2.1 Smart Native Dependency Tracking

- The `build.rs` script implements timestamp-based checks to avoid unnecessary C++ CLI scanner rebuilds.
- Only triggers a rebuild if source files or CMake configuration change.
- CMake cache is reused when possible.

**Example:**
```rust
// In build.rs
println!("cargo:rerun-if-changed=scripts/airpods_battery_cli/CMakeLists.txt");
println!("cargo:rerun-if-changed=scripts/airpods_battery_cli/Source/");
// ...timestamp comparison logic...
```

**Result:**
- Incremental Rust builds are not delayed by native scanner compilation unless required.

### 2.2 Test Suite Management

- Heavy or problematic integration tests are disabled by renaming to `.disabled` or using `#[ignore]` attributes.
- Fast unit tests are grouped for rapid feedback during development.
- Full test suite (including slow tests) is run before release or major merges.

**Example:**
```bash
cargo test --lib --no-default-features   # Fast unit tests only
cargo test --release                     # Full suite for pre-release
```

### 2.3 Feature Flag Utilization

- Use `--no-default-features` to skip optional or slow components during development.
- Enable all features for production builds and CI.

**Example:**
```bash
cargo build --no-default-features        # Fastest dev build
cargo build --release                    # Full production build
```

### 2.4 Cargo Watch for Continuous Feedback

- `cargo-watch` is recommended for automatic test or build execution on file changes.

**Example:**
```bash
cargo install cargo-watch
cargo watch -x "test --lib --no-default-features"
```

### 2.5 Compiler and Build Configuration

- Use LLD linker and incremental compilation for faster debug builds.
- Adjust codegen units for parallelism.

**Example:**
```toml
# .cargo/config.toml
[target.'cfg(all())']
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
[build]
incremental = true
jobs = 4
[profile.dev]
incremental = true
codegen-units = 256
```

### 2.6 IDE and Tooling Integration

- Configure VS Code or Cursor for optimal Rust analysis and build script support.

**Example:**
```json
// .vscode/settings.json
{
    "rust-analyzer.cargo.buildScripts.enable": true,
    "rust-analyzer.check.command": "clippy",
    "rust-analyzer.check.extraArgs": ["--no-default-features", "--lib"]
}
```

## 3. Legacy and Deprecated Techniques

- **Disabling Integration Tests**: Some async-heavy integration tests were previously disabled due to performance or stability issues. Review and re-enable only if they are relevant and stable in the current codebase.
- **Manual CMake/CLI Scanner Rebuilds**: The current build.rs logic supersedes the need for manual native rebuilds in most workflows.

## 4. Performance Monitoring and Verification

- Use provided scripts (e.g., `scripts/test_performance.ps1`) for benchmarking and regression detection.
- Track build and test times to identify performance regressions early.

**Example:**
```bash
./scripts/test_performance.ps1 -LibOnly
```

## 5. Recommended Development Workflow

- Use fast, incremental builds and unit tests for daily development.
- Run the full test suite and production build before merging or releasing.
- Monitor build and test times regularly.

**Typical Cycle:**
```bash
cargo check                                    # Fast compilation check
cargo test --lib --no-default-features         # Fast unit tests
cargo test --release                           # Full suite before release
cargo build --release                          # Production build
```

## 6. Key Takeaways

- Smart dependency tracking and feature flag usage are essential for rapid iteration.
- Maintain a clear separation between fast and slow tests.
- Regularly review and update build scripts and test organization to match project evolution.
- Remove or archive obsolete optimization techniques as the build system matures.

---

**This document is maintained as a living reference. Update or deprecate sections as the project's build system and test architecture evolve.** 