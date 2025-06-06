#!/usr/bin/env bash

set -euo pipefail

# RustPods Build Script (Linux/macOS)
# This script automates the build process for RustPods

# Colors for output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly CYAN='\033[0;36m'
readonly NC='\033[0m' # No Color

# Configuration
CLEAN=false
RELEASE=false
SKIP_CLI=false
SKIP_RUST=false

# Functions
error() { echo -e "${RED}Error: $*${NC}" >&2; }
success() { echo -e "${GREEN}$*${NC}"; }
info() { echo -e "${CYAN}$*${NC}"; }
warning() { echo -e "${YELLOW}$*${NC}"; }

show_help() {
    cat << EOF
RustPods Build Script (Linux/macOS)

Usage: $0 [OPTIONS]

Options:
    --clean      Clean build directories before building
    --release    Build in release mode (default: debug)
    --skip-cli   Skip building the CLI scanner
    --skip-rust  Skip building the Rust application
    --help       Show this help message

Examples:
    $0                    # Debug build of everything
    $0 --release          # Release build of everything
    $0 --clean --release  # Clean release build
    $0 --skip-cli         # Only build Rust app

Prerequisites:
    - Rust toolchain (rustc, cargo)
    - C++ compiler (gcc/clang)
    - CMake 3.16 or later
    - Git (for submodules)

Note: CLI scanner is Windows-specific and may not function on Linux/macOS.
The Rust application can be built but will have limited functionality.

EOF
}

check_command() {
    if ! command -v "$1" &> /dev/null; then
        error "$1 is not installed or not in PATH"
        return 1
    fi
    return 0
}

check_prerequisites() {
    info "Checking prerequisites..."
    
    local errors=0
    
    # Check Rust
    if check_command rustc; then
        success "✓ Rust: $(rustc --version)"
    else
        error "  - Rust toolchain not found. Install from https://rustup.rs/"
        ((errors++))
    fi
    
    # Check Cargo
    if check_command cargo; then
        success "✓ Cargo: $(cargo --version)"
    else
        error "  - Cargo not found (should come with Rust)"
        ((errors++))
    fi
    
    # Check CMake
    if check_command cmake; then
        success "✓ CMake: $(cmake --version | head -n1)"
    else
        error "  - CMake not found. Install from package manager or https://cmake.org/"
        ((errors++))
    fi
    
    # Check C++ compiler
    if check_command g++ || check_command clang++; then
        if command -v g++ &> /dev/null; then
            success "✓ C++ Compiler: $(g++ --version | head -n1)"
        else
            success "✓ C++ Compiler: $(clang++ --version | head -n1)"
        fi
    else
        error "  - C++ compiler not found. Install gcc or clang"
        ((errors++))
    fi
    
    # Check Git
    if check_command git; then
        success "✓ Git: $(git --version)"
    else
        error "  - Git not found. Install from package manager"
        ((errors++))
    fi
    
    if [[ $errors -gt 0 ]]; then
        error "Prerequisites check failed with $errors errors"
        exit 1
    fi
    
    success "All prerequisites satisfied!"
}

initialize_submodules() {
    info "Initializing Git submodules..."
    
    local project_root
    project_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
    
    cd "$project_root"
    git submodule update --init --recursive
    success "✓ Submodules initialized"
}

build_cli_scanner() {
    if [[ "$SKIP_CLI" == true ]]; then
        info "Skipping CLI scanner build"
        return
    fi
    
    warning "Building CLI scanner on Linux/macOS (limited functionality)..."
    
    local project_root
    project_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
    local cli_path="$project_root/scripts/airpods_battery_cli"
    local build_path="$cli_path/build"
    local build_mode
    build_mode=$([ "$RELEASE" == true ] && echo "Release" || echo "Debug")
    
    cd "$cli_path"
    
    # Clean if requested
    if [[ "$CLEAN" == true ]] && [[ -d "$build_path" ]]; then
        info "Cleaning CLI scanner build directory..."
        rm -rf "$build_path"
    fi
    
    # Create build directory
    mkdir -p "$build_path"
    
    # Configure with CMake
    info "Configuring CLI scanner with CMake..."
    cmake -B build -S . -DCMAKE_BUILD_TYPE="$build_mode"
    
    # Build
    info "Building CLI scanner ($build_mode mode)..."
    cmake --build build --config "$build_mode"
    
    # Check if executables were created (modular architecture)
    local v5_exe_path="$build_path/airpods_battery_cli_v5"
    local modular_test_path="$build_path/modular_parser_test"
    
    if [[ -f "$v5_exe_path" ]]; then
        success "✓ V5 reference scanner built: $v5_exe_path"
    else
        warning "⚠ V5 scanner executable not found (Windows-specific functionality)"
    fi
    
    if [[ -f "$modular_test_path" ]]; then
        success "✓ Modular test executable built: $modular_test_path"
        
        # Test the modular parser
        info "Testing modular parser..."
        cd "$build_path"
        if "$modular_test_path" &> /dev/null; then
            success "✓ Modular parser test passed"
        else
            warning "⚠ Modular parser test returned non-zero exit code (expected on non-Windows)"
        fi
    else
        warning "⚠ Modular test executable not found"
    fi
    
    # Check for libraries
    if [[ -f "$build_path/libprotocol_parser.a" ]]; then
        success "✓ Protocol parser library built: libprotocol_parser.a"
    fi
    
    if [[ -f "$build_path/libble_scanner.a" ]]; then
        success "✓ BLE scanner library built: libble_scanner.a"
    fi
}

build_rust_app() {
    if [[ "$SKIP_RUST" == true ]]; then
        info "Skipping Rust application build"
        return
    fi
    
    info "Building RustPods Rust application..."
    
    local project_root
    project_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
    
    cd "$project_root"
    
    # Clean if requested
    if [[ "$CLEAN" == true ]]; then
        info "Cleaning Rust build artifacts..."
        cargo clean
    fi
    
    # Build
    local build_mode
    build_mode=$([ "$RELEASE" == true ] && echo "release" || echo "debug")
    info "Building Rust application ($build_mode mode)..."
    
    if [[ "$RELEASE" == true ]]; then
        cargo build --release
    else
        cargo build
    fi
    
    # Verify executable exists
    local target_dir
    target_dir=$([ "$RELEASE" == true ] && echo "release" || echo "debug")
    local exe_path="$project_root/target/$target_dir/rustpods"
    
    if [[ -f "$exe_path" ]]; then
        success "✓ RustPods built successfully: $exe_path"
        
        # Run tests
        info "Running Rust tests..."
        if cargo test $([ "$RELEASE" == true ] && echo "--release"); then
            success "✓ All Rust tests passed"
        else
            warning "⚠ Some Rust tests failed"
        fi
    else
        error "RustPods executable not found at: $exe_path"
        exit 1
    fi
}

main() {
    local start_time
    start_time=$(date +%s)
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --clean)
                CLEAN=true
                shift
                ;;
            --release)
                RELEASE=true
                shift
                ;;
            --skip-cli)
                SKIP_CLI=true
                shift
                ;;
            --skip-rust)
                SKIP_RUST=true
                shift
                ;;
            --help)
                show_help
                exit 0
                ;;
            *)
                error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
    
    info "=== RustPods Build Script (Linux/macOS) ==="
    local build_mode
    build_mode=$([ "$RELEASE" == true ] && echo "Release" || echo "Debug")
    info "Build Mode: $build_mode"
    
    check_prerequisites
    initialize_submodules
    build_cli_scanner
    build_rust_app
    
    local end_time
    end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    success ""
    success "=== Build Completed Successfully ==="
    success "Total time: ${duration}s"
    success ""
    info "Built components:"
    if [[ "$SKIP_CLI" != true ]]; then
        info "  • AirPods CLI Scanner: scripts/airpods_battery_cli/build/airpods_battery_cli"
    fi
    if [[ "$SKIP_RUST" != true ]]; then
        local target_dir
        target_dir=$([ "$RELEASE" == true ] && echo "release" || echo "debug")
        info "  • RustPods Application: target/$target_dir/rustpods"
    fi
    info ""
    info "To run RustPods:"
    local target_dir
    target_dir=$([ "$RELEASE" == true ] && echo "release" || echo "debug")
    info "  ./target/$target_dir/rustpods"
    info ""
    warning "Note: Full functionality requires Windows for AirPods battery monitoring"
}

# Run main function
main "$@" 