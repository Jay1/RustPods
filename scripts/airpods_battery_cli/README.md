# AirPods Battery CLI Scanner

A professional, production-ready command-line utility for monitoring Apple AirPods and Beats device battery levels using Windows Bluetooth Low Energy (BLE) capabilities.

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [System Requirements](#system-requirements)
- [Installation](#installation)
- [Usage](#usage)
- [Architecture](#architecture)
- [Supported Devices](#supported-devices)
- [JSON Output Format](#json-output-format)
- [Troubleshooting](#troubleshooting)
- [Development](#development)
- [Contributing](#contributing)
- [License](#license)

## Overview

This CLI scanner implements Apple's Continuity Protocol to extract battery information from AirPods and Beats devices via Bluetooth Low Energy advertisements. The project features a clean, modular C++ architecture suitable for production deployment and open-source distribution.

## Features

- **Real-time Battery Monitoring**: Live detection of AirPods and Beats device battery levels
- **Comprehensive Device Support**: All major AirPods and Beats models (see [Supported Devices](#supported-devices))
- **Detailed Status Information**: 
  - Individual left/right earbud and case battery percentages
  - Charging state detection (earbuds and case)
  - In-ear detection status
  - Case lid position (open/closed)
  - Broadcasting earbud identification
- **Structured Output**: Clean JSON format for integration with other applications
- **Modular Architecture**: Professional codebase with clear separation of concerns
- **Production Ready**: Robust error handling, comprehensive logging, and thorough testing

## System Requirements

### Operating System
- Windows 10 version 1809 or later
- Windows 11 (all versions)

### Hardware
- Bluetooth 4.0+ adapter with BLE support
- At least one paired AirPods or supported Beats device

### Development Requirements
- Visual Studio 2019/2022 with C++ workload or Build Tools for Visual Studio
- Windows SDK 10.0.26100.0 or later
- CMake 3.10 or later
- Git (for submodule management)

## Installation

### Build from Source
1. Clone the repository:
   ```bash
   git clone https://github.com/your-username/airpods-battery-cli.git
        cd airpods-battery-cli
   ```

2. Initialize submodules (if not cloned recursively):
   ```bash
   git submodule update --init --recursive
   ```

3. Build using the provided script:
   ```powershell
   # Windows PowerShell
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

## Usage

### Basic Usage
```bash
# Scan for AirPods and display battery information
.\airpods_battery_cli_v5.exe
```

### Example Output
```json
{
    "scanner_version": "v5",
    "timestamp": "2024-06-06T02:30:45Z",
    "devices": [
        {
            "device_id": "fb4948d33d2f",
            "device_name": "Jay's AirPods Pro",
            "model": "AirPods Pro 2",
            "model_id": 8212,
            "rssi": -45,
            "manufacturer_data": "07190114200b778f...",
            "airpods_data": {
                "left_battery": 70,
                "right_battery": 70,
                "case_battery": 0,
                "left_charging": true,
                "right_charging": true,
                "case_charging": false,
                "left_in_ear": true,
                "right_in_ear": true,
                "both_in_case": false,
                "lid_open": true,
                "broadcasting_ear": "right"
            }
        }
    ]
}
```

### Integration with Other Applications
The scanner outputs structured JSON, making it easy to integrate with other applications:

```powershell
# PowerShell example
$result = .\airpods_battery_cli_v5.exe | ConvertFrom-Json
foreach ($device in $result.devices) {
    Write-Host "$($device.model): L:$($device.airpods_data.left_battery)% R:$($device.airpods_data.right_battery)% C:$($device.airpods_data.case_battery)%"
}
```

```python
# Python example
import subprocess
import json

result = subprocess.run(['airpods_battery_cli_v5.exe'], capture_output=True, text=True)
data = json.loads(result.stdout)

for device in data['devices']:
    airpods = device['airpods_data']
    print(f"{device['model']}: L:{airpods['left_battery']}% R:{airpods['right_battery']}% C:{airpods['case_battery']}%")
```

## Architecture

The CLI scanner features a modular architecture with clear separation of concerns:

### Core Components

1. **BLE Scanner Module** (`Source/ble/`)
   - `IBleScanner.hpp`: Abstract interface for BLE scanning
   - `WinRtBleScanner.cpp`: Windows Runtime BLE implementation
   - `BleDevice.hpp/cpp`: Device data structures and utilities

2. **Protocol Parser Module** (`Source/protocol/`)
   - `IProtocolParser.hpp`: Template interface for protocol parsers
   - `AppleContinuityParser.cpp`: Apple Continuity Protocol implementation
   - `AirPodsData.hpp/cpp`: Data structures for AirPods information

3. **Build System**
   - `CMakeLists.txt`: CMake configuration with modular targets
   - Static libraries: `protocol_parser.lib`, `ble_scanner.lib`
   - Test executables for validation

### Key Design Principles

- **Interface-based Design**: Clean abstractions for testability and extensibility
- **Thread Safety**: Proper synchronization for real-time operations
- **Resource Management**: RAII patterns for Bluetooth connections
- **Error Handling**: Comprehensive exception handling and user-friendly messages
- **Documentation**: Thorough code comments and API documentation

For detailed architecture information, see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Supported Devices

The scanner supports all major Apple AirPods and Beats models:

### AirPods Models
- **AirPods (1st generation)** - Model ID: 0x200E
- **AirPods (2nd generation)** - Model ID: 0x200F  
- **AirPods (3rd generation)** - Model ID: 0x2013
- **AirPods Pro** - Model ID: 0x200E
- **AirPods Pro (2nd generation)** - Model ID: 0x2014
- **AirPods Max** - Model ID: 0x200A

### Beats Models
- **Beats Solo Pro** - Model ID: 0x201B
- **Beats Studio Buds** - Model ID: 0x2017
- **Beats Fit Pro** - Model ID: 0x2019
- **PowerBeats Pro** - Model ID: 0x200B

### Detection Method
Devices are detected through Apple's Continuity Protocol using:
- **Apple Company ID**: 0x004C (76 decimal)
- **Protocol Type**: Proximity Pairing (0x07)
- **Manufacturer Data**: 27+ bytes for complete protocol information

## JSON Output Format

The scanner outputs structured JSON with the following schema:

```json
{
    "scanner_version": "string",
    "timestamp": "ISO8601 datetime",
    "devices": [
        {
            "device_id": "string (hex MAC address)",
            "device_name": "string (optional)",
            "model": "string (human-readable model name)",
            "model_id": "number (numeric model identifier)",
            "rssi": "number (signal strength in dBm)",
            "manufacturer_data": "string (hex-encoded raw data)",
            "airpods_data": {
                "left_battery": "number (0-100)",
                "right_battery": "number (0-100)",
                "case_battery": "number (0-100)",
                "left_charging": "boolean",
                "right_charging": "boolean", 
                "case_charging": "boolean",
                "left_in_ear": "boolean",
                "right_in_ear": "boolean",
                "both_in_case": "boolean",
                "lid_open": "boolean",
                "broadcasting_ear": "string (left|right|unknown)"
            }
        }
    ]
}
```

## Troubleshooting

### Common Issues

#### No Devices Found
**Problem**: Scanner runs but reports no devices
**Solutions**:
1. Ensure AirPods are paired with the computer
2. Check Bluetooth adapter is enabled and functioning
3. Verify AirPods are not in deep sleep (use them briefly to wake up)
4. Try re-pairing the AirPods if necessary

#### Access Denied Errors
**Problem**: Permission errors when accessing Bluetooth
**Solutions**:
1. Run as Administrator (right-click → "Run as administrator")
2. Check Windows Privacy settings for Bluetooth access
3. Verify Bluetooth service is running (`services.msc` → Bluetooth Support Service)

#### Compilation Errors
**Problem**: Build fails with compiler errors
**Solutions**:
1. Ensure Visual Studio 2019/2022 with C++ workload is installed
2. Verify Windows SDK 10.0.26100.0 or later is available
3. Check CMake version is 3.10 or later
4. Initialize git submodules: `git submodule update --init --recursive`

#### Missing spdlog Dependencies
**Problem**: Build fails with spdlog-related errors
**Solutions**:
1. Initialize submodules: `git submodule update --init --recursive`
2. Verify `third_party/spdlog/` directory exists and contains files
3. Clean and rebuild: `cmake --build build --target clean && cmake --build build --config Release`

### Diagnostic Commands

```bash
# Check Bluetooth adapter status
Get-PnpDevice -Class Bluetooth

# Verify BLE device enumeration
Get-PnpDevice | Where-Object {$_.FriendlyName -like "*AirPods*"}

# Test CLI scanner with verbose output
.\airpods_battery_cli_v5.exe 2>&1 | Tee-Object -FilePath debug.log
```

### Debug Mode
The scanner includes comprehensive logging. Check console output for detailed information about:
- Bluetooth adapter detection
- Device enumeration process  
- Protocol parsing results
- Error conditions and recovery attempts

## Development

### Project Structure
```
scripts/airpods_battery_cli/
├── CMakeLists.txt              # Build configuration
├── README.md                   # This file
├── Source/                     # Source code
│   ├── airpods_battery_cli_v5.cpp  # V5 reference implementation
│   ├── ble/                    # BLE scanning module
│   │   ├── IBleScanner.hpp     # BLE scanner interface
│   │   ├── WinRtBleScanner.*   # Windows Runtime implementation
│   │   └── BleDevice.*         # Device data structures
│   ├── protocol/               # Protocol parsing module
│   │   ├── IProtocolParser.hpp # Parser interface
│   │   ├── AppleContinuityParser.*  # Apple protocol implementation
│   │   └── AirPodsData.*       # AirPods data structures
│   └── test_*.cpp              # Test programs
├── third_party/                # External dependencies
│   └── spdlog/                 # Logging library (submodule)
└── docs/                       # Documentation
    ├── ARCHITECTURE.md         # Detailed architecture guide
    └── CONTRIBUTING.md         # Contributor guidelines
```

### Building for Development
```bash
# Debug build for development
cmake -B build -S . -G "Visual Studio 17 2022" -A x64
cmake --build build --config Debug

# Run tests
cd build/Debug
.\modular_parser_test.exe
.\test_protocol_parser.exe
```

### Testing
The project includes comprehensive test suites:
- **Unit Tests**: Individual component testing
- **Integration Tests**: Full protocol parsing validation
- **Modular Tests**: Architecture component verification
- **Reference Validation**: Comparison with V5 implementation

### Adding New Protocol Support
1. Implement `IProtocolParser<T>` interface in `protocol/` directory
2. Add protocol-specific data structures
3. Register parser in main scanner logic
4. Add comprehensive tests for the new protocol
5. Update documentation with supported devices

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](docs/CONTRIBUTING.md) for guidelines on:
- Code style and conventions
- Testing requirements
- Pull request process
- Issue reporting
- Development setup

### Quick Start for Contributors
1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes and add tests
4. Ensure all tests pass: `.\scripts\build_all.ps1`
5. Commit your changes: `git commit -m 'Add amazing feature'`
6. Push to your branch: `git push origin feature/amazing-feature`
7. Open a Pull Request

## Acknowledgments

This project builds upon the excellent work of the [AirPodsDesktop](https://github.com/SpriteOvO/AirPodsDesktop) project by SpriteOvO. We extend our sincere gratitude for:

- **Apple Continuity Protocol Implementation**: The core BLE advertisement parsing and Apple protocol decoding
- **AirPods Model Detection**: Comprehensive support for all AirPods and Beats variants
- **Windows BLE Integration**: Proven Windows Runtime Bluetooth LE implementation patterns
- **Protocol Documentation**: Detailed understanding of Apple's proprietary advertising format

While this CLI scanner has been significantly refactored and simplified for standalone use, the foundational protocol parsing logic originated from the AirPodsDesktop project. We encourage users to also check out the original full-featured desktop application.

**Original Project**: [AirPodsDesktop](https://github.com/SpriteOvO/AirPodsDesktop)  
**Original Author**: [@SpriteOvO](https://github.com/SpriteOvO)  
**License**: MIT License

## License

This project is part of the RustPods application and is distributed under the MIT License. See the main project [LICENSE](../../LICENSE) file for details.

---

**Project**: RustPods  
**Repository**: https://github.com/Jay1/RustPods  
**CLI Scanner Version**: v5.0 (Production)  
**Last Updated**: June 2025
