#!/usr/bin/env pwsh

<#
.SYNOPSIS
    Build script for RustPods and AirPods CLI Scanner
.DESCRIPTION
    This script automates the build process for both the Rust application
    and the native C++ CLI scanner, including all dependencies.
.PARAMETER Clean
    Clean build directories before building
.PARAMETER Release
    Build in release mode (default: debug)
.PARAMETER SkipCli
    Skip building the CLI scanner
.PARAMETER SkipRust
    Skip building the Rust application
#>

param(
    [switch]$Clean,
    [switch]$Release,
    [switch]$SkipCli,
    [switch]$SkipRust,
    [switch]$Help
)

# Color functions for better output
function Write-Success { param($Message) Write-Host $Message -ForegroundColor Green }
function Write-Info { param($Message) Write-Host $Message -ForegroundColor Cyan }
function Write-Warning { param($Message) Write-Host $Message -ForegroundColor Yellow }
function Write-Error { param($Message) Write-Host $Message -ForegroundColor Red }

function Show-Help {
    Write-Host @"
RustPods Build Script

Usage: .\scripts\build_all.ps1 [OPTIONS]

Options:
    -Clean      Clean build directories before building
    -Release    Build in release mode (default: debug)
    -SkipCli    Skip building the CLI scanner
    -SkipRust   Skip building the Rust application
    -Help       Show this help message

Examples:
    .\scripts\build_all.ps1                    # Debug build of everything
    .\scripts\build_all.ps1 -Release           # Release build of everything
    .\scripts\build_all.ps1 -Clean -Release    # Clean release build
    .\scripts\build_all.ps1 -SkipCli           # Only build Rust app

Prerequisites:
    - Rust toolchain (rustc, cargo)
    - Microsoft C++ Build Tools (MSVC)
    - CMake 3.16 or later
    - Git (for submodules)

"@ -ForegroundColor White
}

if ($Help) {
    Show-Help
    exit 0
}

# Configuration
$BuildMode = if ($Release) { "Release" } else { "Debug" }
$CargoProfile = if ($Release) { "--release" } else { "" }
$ProjectRoot = Split-Path -Parent $PSScriptRoot
$CliPath = Join-Path $ProjectRoot "scripts\airpods_battery_cli"
$BuildPath = Join-Path $CliPath "build"

Write-Info "=== RustPods Build Script ==="
Write-Info "Build Mode: $BuildMode"
Write-Info "Project Root: $ProjectRoot"

# Check prerequisites
function Test-Prerequisites {
    Write-Info "Checking prerequisites..."
    
    $errors = @()
    
    # Check Rust
    try {
        $rustVersion = rustc --version 2>$null
        Write-Success "✓ Rust: $rustVersion"
    } catch {
        $errors += "Rust toolchain not found. Install from https://rustup.rs/"
    }
    
    # Check Cargo
    try {
        $cargoVersion = cargo --version 2>$null
        Write-Success "✓ Cargo: $cargoVersion"
    } catch {
        $errors += "Cargo not found (should come with Rust)"
    }
    
    # Check CMake
    try {
        $cmakeVersion = cmake --version 2>$null | Select-Object -First 1
        Write-Success "✓ CMake: $cmakeVersion"
    } catch {
        $errors += "CMake not found. Install from https://cmake.org/download/"
    }
    
    # Check MSVC (try to find cl.exe)
    $msvcPaths = @(
        "${env:ProgramFiles}\Microsoft Visual Studio\2022\*\VC\Tools\MSVC\*\bin\Hostx64\x64\cl.exe",
        "${env:ProgramFiles}\Microsoft Visual Studio\2019\*\VC\Tools\MSVC\*\bin\Hostx64\x64\cl.exe",
        "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\*\VC\Tools\MSVC\*\bin\Hostx64\x64\cl.exe",
        "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2019\*\VC\Tools\MSVC\*\bin\Hostx64\x64\cl.exe"
    )
    
    $msvcFound = $false
    foreach ($path in $msvcPaths) {
        if (Get-ChildItem $path -ErrorAction SilentlyContinue) {
            Write-Success "✓ MSVC: Found Visual Studio C++ compiler"
            $msvcFound = $true
            break
        }
    }
    
    if (-not $msvcFound) {
        $errors += "MSVC not found. Install Visual Studio 2019/2022 with C++ workload or Build Tools for Visual Studio"
    }
    
    # Check Git
    try {
        $gitVersion = git --version 2>$null
        Write-Success "✓ Git: $gitVersion"
    } catch {
        $errors += "Git not found. Install from https://git-scm.com/"
    }
    
    if ($errors.Count -gt 0) {
        Write-Error "Prerequisites check failed:"
        $errors | ForEach-Object { Write-Error "  - $_" }
        exit 1
    }
    
    Write-Success "All prerequisites satisfied!"
}

# Initialize submodules
function Initialize-Submodules {
    Write-Info "Initializing Git submodules..."
    
    Push-Location $ProjectRoot
    try {
        git submodule update --init --recursive
        if ($LASTEXITCODE -ne 0) {
            throw "Git submodule initialization failed"
        }
        Write-Success "✓ Submodules initialized"
    } catch {
        Write-Error "Failed to initialize submodules: $_"
        exit 1
    } finally {
        Pop-Location
    }
}

# Build CLI scanner
function Build-CliScanner {
    if ($SkipCli) {
        Write-Info "Skipping CLI scanner build"
        return
    }
    
    Write-Info "Building AirPods CLI Scanner..."
    
    Push-Location $CliPath
    try {
        # Clean if requested
        if ($Clean -and (Test-Path $BuildPath)) {
            Write-Info "Cleaning CLI scanner build directory..."
            Remove-Item $BuildPath -Recurse -Force
        }
        
        # Create build directory
        if (-not (Test-Path $BuildPath)) {
            New-Item -ItemType Directory -Path $BuildPath | Out-Null
        }
        
        # Configure with CMake
        Write-Info "Configuring CLI scanner with CMake..."
        cmake -B build -S . -G "Visual Studio 17 2022" -A x64
        if ($LASTEXITCODE -ne 0) {
            throw "CMake configuration failed"
        }
        
        # Build
        Write-Info "Building CLI scanner ($BuildMode mode)..."
        cmake --build build --config $BuildMode
        if ($LASTEXITCODE -ne 0) {
            throw "CLI scanner build failed"
        }
        
        # Verify executables exist (both v5 reference and modular architecture)
        $v5ExePath = Join-Path $BuildPath "$BuildMode\airpods_battery_cli_v5.exe"
        $modularTestPath = Join-Path $BuildPath "$BuildMode\modular_parser_test.exe"
        
        if (-not (Test-Path $v5ExePath)) {
            throw "V5 reference scanner not found at: $v5ExePath"
        }
        
        if (-not (Test-Path $modularTestPath)) {
            throw "Modular test executable not found at: $modularTestPath"
        }
        
        Write-Success "✓ CLI scanner built successfully:"
        Write-Success "  - V5 Reference: $v5ExePath"
        Write-Success "  - Modular Test: $modularTestPath"
        Write-Success "  - Protocol Parser Library: protocol_parser.lib"
        Write-Success "  - BLE Scanner Library: ble_scanner.lib"
        
        # Test the modular parser
        Write-Info "Testing modular parser..."
        Push-Location (Split-Path $modularTestPath)
        try {
            $testOutput = & $modularTestPath 2>&1
            if ($LASTEXITCODE -eq 0) {
                Write-Success "✓ Modular parser test passed"
            } else {
                Write-Warning "⚠ Modular parser test returned non-zero exit code"
            }
        } finally {
            Pop-Location
        }
        
    } catch {
        Write-Error "CLI scanner build failed: $_"
        exit 1
    } finally {
        Pop-Location
    }
}

# Build Rust application
function Build-RustApp {
    if ($SkipRust) {
        Write-Info "Skipping Rust application build"
        return
    }
    
    Write-Info "Building RustPods Rust application..."
    
    Push-Location $ProjectRoot
    try {
        # Clean if requested
        if ($Clean) {
            Write-Info "Cleaning Rust build artifacts..."
            cargo clean
        }
        
        # Build
        Write-Info "Building Rust application ($BuildMode mode)..."
        if ($Release) {
            cargo build --release
        } else {
            cargo build
        }
        
        if ($LASTEXITCODE -ne 0) {
            throw "Rust build failed"
        }
        
        # Verify executable exists
        $targetDir = if ($Release) { "release" } else { "debug" }
        $exePath = Join-Path $ProjectRoot "target\$targetDir\rustpods.exe"
        if (-not (Test-Path $exePath)) {
            throw "RustPods executable not found at: $exePath"
        }
        
        Write-Success "✓ RustPods built successfully: $exePath"
        
        # Run tests
        Write-Info "Running Rust tests..."
        cargo test $CargoProfile
        if ($LASTEXITCODE -eq 0) {
            Write-Success "✓ All Rust tests passed"
        } else {
            Write-Warning "⚠ Some Rust tests failed"
        }
        
    } catch {
        Write-Error "Rust build failed: $_"
        exit 1
    } finally {
        Pop-Location
    }
}

# Main execution
function Main {
    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    
    try {
        Test-Prerequisites
        Initialize-Submodules
        Build-CliScanner
        Build-RustApp
        
        $stopwatch.Stop()
        Write-Success ""
        Write-Success "=== Build Completed Successfully ==="
        Write-Success "Total time: $($stopwatch.Elapsed.ToString('mm\:ss'))"
        Write-Success ""
        Write-Info "Built components:"
        if (-not $SkipCli) {
            Write-Info "  • AirPods CLI Scanner (V5): scripts\airpods_battery_cli\build\$BuildMode\airpods_battery_cli_v5.exe"
            Write-Info "  • Modular Architecture: protocol_parser.lib, ble_scanner.lib"
            Write-Info "  • Test Executables: modular_parser_test.exe, test_protocol_parser.exe"
        }
        if (-not $SkipRust) {
            $targetDir = if ($Release) { "release" } else { "debug" }
            Write-Info "  • RustPods Application: target\$targetDir\rustpods.exe"
        }
        Write-Info ""
        Write-Info "To run RustPods:"
        $targetDir = if ($Release) { "release" } else { "debug" }
        Write-Info "  .\target\$targetDir\rustpods.exe"
        
    } catch {
        $stopwatch.Stop()
        Write-Error ""
        Write-Error "=== Build Failed ==="
        Write-Error "Error: $_"
        Write-Error "Time elapsed: $($stopwatch.Elapsed.ToString('mm\:ss'))"
        exit 1
    }
}

# Run the main function
Main 