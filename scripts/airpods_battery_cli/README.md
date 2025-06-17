# AirPods Battery CLI Scanner: Technical Architecture and Operational Protocol

## Executive Summary

This command-line utility provides production-grade monitoring of Apple AirPods and Beats device battery levels using Windows Bluetooth Low Energy (BLE) capabilities. The implementation is modular, robust, and engineered for integration with the RustPods system.

## Table of Contents
- Overview
- Features
- System Requirements
- Installation and Build Procedures
- Usage and Integration
- Architecture
- Supported Devices
- JSON Output Schema
- Troubleshooting and Diagnostics
- Development Protocols
- Contribution and Licensing

## Overview

The CLI scanner implements Apple's Continuity Protocol to extract battery and status information from AirPods and Beats devices via BLE advertisements. The codebase is modular C++ and suitable for production deployment.

## Features

- Real-time battery monitoring for all major AirPods and Beats models
- Detailed status reporting: individual battery percentages, charging state, in-ear detection, case lid position, broadcasting earbud
- Structured JSON output for seamless integration
- Modular, interface-based architecture
- Comprehensive error handling and logging

## System Requirements

- **Operating System**: Windows 10 version 1809 or later, or Windows 11
- **Hardware**: Bluetooth 4.0+ adapter with BLE support, at least one paired AirPods or supported Beats device
- **Development**: Visual Studio 2019/2022 with C++ workload, Windows SDK 10.0.26100.0 or later, CMake 3.10+, Git

## Installation and Build Procedures

### Build from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/your-username/airpods-battery-cli.git
   cd airpods-battery-cli
   ```
2. Initialize submodules:
   ```bash
   git submodule update --init --recursive
   ```
3. Build using the provided script:
   ```powershell
   ..\..\scripts\build_all.ps1 -SkipRust
   ```
   Or build manually:
   ```bash
   mkdir build
   cd build
   cmake .. -G "Visual Studio 17 2022" -A x64
   cmake --build . --config Release
   ```
4. The executable will be available at:
   ```
   build/Release/airpods_battery_cli_v5.exe
   ```

## Usage and Integration

### Basic Usage
```bash
./airpods_battery_cli_v5.exe
```

### Example Output
```json
{
    "scanner_version": "v5",
    "timestamp": "2024-06-06T02:30:45Z",
    "devices": [ ... ]
}
```

### Integration
- Outputs structured JSON for direct consumption by RustPods or other applications
- See documentation for PowerShell and Python integration examples

## Architecture

- **BLE Scanner Module**: Abstract interface and Windows Runtime implementation
- **Protocol Parser Module**: Apple Continuity Protocol implementation and data structures
- **Build System**: Modular CMake configuration, static libraries, and test executables
- **Design Principles**: Interface-based design, thread safety, RAII, comprehensive error handling, and documentation

## Supported Devices

- AirPods (1st, 2nd, 3rd generation)
- AirPods Pro (1st, 2nd generation, including USB-C)
- AirPods Max
- Beats Studio Buds, Fit Pro, Powerbeats Pro, and other listed models

## JSON Output Schema

See documentation for the complete JSON schema, including all device and status fields.

## Troubleshooting and Diagnostics

- Ensure AirPods are paired and Bluetooth adapter is enabled
- Run as Administrator if access denied
- Verify all build dependencies and submodules are initialized
- Use provided diagnostic commands for device and adapter status

## Development Protocols

- Modular C++ architecture with interface-based design
- Comprehensive test suites for unit, integration, and modular validation
- All contributions must adhere to RustPods project standards for code quality, documentation, and testing

## Contribution and Licensing

- See [CONTRIBUTING.md](../../docs/CONTRIBUTING.md) for contribution protocols
- Licensed under the MIT License; see [LICENSE](../../LICENSE) for terms

## Acknowledgments

This project builds upon the architectural foundations of [AirPodsDesktop](https://github.com/SpriteOvO/AirPodsDesktop) by SpriteOvO, extending their Apple Continuity Protocol parsing implementation for standalone CLI use.
