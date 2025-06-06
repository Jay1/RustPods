# AirPods CLI Scanner - Clean Architecture Design

## Overview

This document outlines the clean architecture design for refactoring the working v5 AirPods CLI scanner prototype into a production-ready, maintainable codebase while preserving its exact functionality.

## Architecture Principles

1. **Separation of Concerns**: Each module has a single, well-defined responsibility
2. **Dependency Inversion**: High-level modules do not depend on low-level modules
3. **Interface Segregation**: Clients depend only on interfaces they use
4. **Single Responsibility**: Each class has one reason to change
5. **Open/Closed**: Open for extension, closed for modification

## Module Structure

### 1. Core Application Layer (`src/core/`)

**Application.hpp/cpp**
- Main application entry point and orchestration
- Command-line argument parsing and validation
- Error handling and user feedback
- Application lifecycle management

**Configuration.hpp/cpp**
- Scan duration settings
- Logging configuration
- Output format preferences
- Bluetooth adapter selection

### 2. BLE Scanner Module (`src/ble/`)

**IBleScanner.hpp**
```cpp
class IBleScanner {
public:
    virtual ~IBleScanner() = default;
    virtual bool Start() = 0;
    virtual bool Stop() = 0;
    virtual std::vector<BleDevice> GetDevices() const = 0;
    virtual void RegisterCallback(std::function<void(const BleDevice&)> callback) = 0;
};
```

**WinRtBleScanner.hpp/cpp**
- Concrete implementation using WinRT BluetoothLEAdvertisementWatcher
- Thread-safe device collection
- Advertisement event handling
- Resource management for Bluetooth connections

**BleDevice.hpp**
```cpp
struct BleDevice {
    std::string deviceId;
    uint64_t address;
    int rssi;
    std::vector<uint8_t> manufacturerData;
    std::chrono::system_clock::time_point timestamp;
    std::optional<AirPodsData> airpodsData;
};
```

### 3. Protocol Parser Module (`src/protocol/`)

**IProtocolParser.hpp**
```cpp
template<typename T>
class IProtocolParser {
public:
    virtual ~IProtocolParser() = default;
    virtual std::optional<T> Parse(const std::vector<uint8_t>& data) = 0;
    virtual bool CanParse(const std::vector<uint8_t>& data) const = 0;
};
```

**AppleContinuityParser.hpp/cpp**
- Implements IProtocolParser<AirPodsData>
- Apple Continuity Protocol parsing logic
- Model identification and validation
- Battery level extraction algorithms
- Charging state determination
- Device state parsing (in-ear, case, etc.)

**AirPodsData.hpp**
```cpp
struct AirPodsData {
    std::string model;
    std::string modelId;
    BatteryLevels batteryLevels;
    ChargingState chargingState;
    DeviceState deviceState;
    std::string broadcastingEar;
};

struct BatteryLevels {
    int left;
    int right;
    int case_;
};

struct ChargingState {
    bool leftCharging;
    bool rightCharging;
    bool caseCharging;
};

struct DeviceState {
    bool leftInEar;
    bool rightInEar;
    bool bothInCase;
    bool lidOpen;
};
```

### 4. Device Processing Module (`src/device/`)

**DeviceFilter.hpp/cpp**
- Company ID filtering (Apple = 76)
- Device type identification
- Duplicate device handling
- Signal strength filtering

**DeviceManager.hpp/cpp**
- Thread-safe device storage
- Device lifecycle management
- Device update notifications
- Historical data tracking

### 5. Output Module (`src/output/`)

**IOutputFormatter.hpp**
```cpp
class IOutputFormatter {
public:
    virtual ~IOutputFormatter() = default;
    virtual void OutputDevices(const std::vector<BleDevice>& devices) = 0;
    virtual void OutputError(const std::string& error) = 0;
};
```

**JsonOutputFormatter.hpp/cpp**
- Structured JSON output compatible with Rust integration
- Scanner version and timestamp metadata
- Device count and status information
- Error response formatting

**ConsoleLogger.hpp/cpp**
- Real-time device detection logging
- Configurable verbosity levels
- Error and status messages
- Progress indicators

### 6. Error Handling Module (`src/error/`)

**ErrorHandler.hpp/cpp**
- Centralized error handling
- Error code definitions
- User-friendly error messages
- Exception translation

**ScannerException.hpp**
```cpp
class ScannerException : public std::exception {
public:
    enum class ErrorCode {
        BluetoothUnavailable,
        ScanStartFailed,
        ScanStopFailed,
        InvalidArguments,
        ConfigurationError
    };
    
    ScannerException(ErrorCode code, const std::string& message);
    ErrorCode GetErrorCode() const;
    const char* what() const noexcept override;
};
```

## Data Flow Architecture

1. **Initialization**: Application parses arguments and configures components
2. **Scanner Start**: BleScanner begins advertisement monitoring
3. **Advertisement Processing**: Received advertisements are filtered and processed
4. **Protocol Parsing**: Apple device data is parsed using AppleContinuityParser
5. **Device Management**: Parsed devices are stored and managed by DeviceManager
6. **Output Generation**: Results are formatted and output via IOutputFormatter
7. **Cleanup**: Resources are properly released on shutdown

## Threading Model

- **Main Thread**: Application control, argument parsing, output generation
- **BLE Callback Thread**: Advertisement processing (WinRT managed)
- **Device Manager**: Thread-safe storage with mutex protection
- **Output Thread**: Asynchronous logging (optional, for performance)

## Error Handling Strategy

1. **Exception Safety**: All operations provide basic exception safety guarantee
2. **Resource Management**: RAII for all system resources
3. **Graceful Degradation**: Continue operation when possible
4. **User Feedback**: Clear error messages for common scenarios
5. **Logging**: Comprehensive error logging for debugging

## Coding Standards

### Naming Conventions
- Classes: PascalCase (e.g., `BleDevice`)
- Methods: PascalCase (e.g., `GetDevices()`)
- Variables: camelCase (e.g., `deviceId`)
- Constants: UPPER_SNAKE_CASE (e.g., `APPLE_COMPANY_ID`)
- Files: PascalCase matching primary class name

### Documentation Requirements
- All public APIs documented with Doxygen comments
- Complex algorithms explained with inline comments
- Usage examples in header files
- Architecture decisions documented

### Code Organization
- One class per file (except small utility classes)
- Header guards using `#pragma once`
- Include order: standard library, third-party, project headers
- Const correctness enforced throughout

## Build System Integration

### CMake Structure
```cmake
add_library(airpods_core STATIC
    src/core/Application.cpp
    src/core/Configuration.cpp
)

add_library(airpods_ble STATIC
    src/ble/WinRtBleScanner.cpp
)

add_library(airpods_protocol STATIC
    src/protocol/AppleContinuityParser.cpp
)

add_executable(airpods_battery_cli
    src/main.cpp
)

target_link_libraries(airpods_battery_cli
    airpods_core
    airpods_ble
    airpods_protocol
    windowsapp
)
```

### Compiler Requirements
- C++20 standard
- MSVC with `/await` flag for WinRT support
- `/utf-8` for Unicode support
- Warnings as errors for code quality

## Testing Strategy

### Unit Tests
- Each module tested in isolation
- Mock interfaces for dependencies
- Code coverage targeting 90%+
- Performance regression tests

### Integration Tests
- End-to-end functionality validation
- Real hardware testing with AirPods
- Cross-platform compatibility verification
- Error condition testing

### Validation Framework
- Functional equivalence with v5 prototype
- Performance benchmarking
- Memory leak detection
- Static analysis integration

## Migration Strategy

1. **Phase 1**: Create modular structure, migrate core data types
2. **Phase 2**: Implement BLE scanner module with WinRT wrapper
3. **Phase 3**: Refactor protocol parsing with clean interfaces
4. **Phase 4**: Add output formatting and error handling
5. **Phase 5**: Integration testing and performance optimization
6. **Phase 6**: Documentation and final validation

## Backwards Compatibility

- Command-line interface remains unchanged
- JSON output format preserved exactly
- Exit codes maintained for script compatibility
- Performance characteristics match or exceed v5

This architecture ensures maintainability, testability, and extensibility while preserving the exact functionality that makes the v5 scanner successful. 