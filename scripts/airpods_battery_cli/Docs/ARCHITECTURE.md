# CLI Scanner Architecture Documentation

This document provides a comprehensive overview of the AirPods Battery CLI Scanner architecture, detailing the modular design principles, component interactions, and implementation patterns.

## Table of Contents

- [Architecture Overview](#architecture-overview)
- [Core Components](#core-components)
- [Design Principles](#design-principles)
- [Module Interfaces](#module-interfaces)
- [Data Flow](#data-flow)
- [Threading Model](#threading-model)
- [Error Handling Strategy](#error-handling-strategy)
- [Build System](#build-system)
- [Performance Considerations](#performance-considerations)
- [Future Extensibility](#future-extensibility)

## Architecture Overview

The CLI scanner follows a **layered, modular architecture** with clear separation of concerns. The design prioritizes:

- **Maintainability**: Clean interfaces and modular components
- **Testability**: Interface-based design enables unit testing
- **Extensibility**: Template-based patterns for adding new protocols
- **Performance**: Efficient resource management and threading
- **Reliability**: Comprehensive error handling and logging

### High-Level Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│                    CLI Application                      │
│                 (Entry Point & Config)                 │
└─────────────────────┬───────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────┐
│                 Device Processing                      │
│             (Filtering & Management)                   │
└─────────────┬───────────────────────────┬───────────────┘
              │                           │
┌─────────────▼───────────┐   ┌───────────▼───────────────┐
│    BLE Scanner Module   │   │   Protocol Parser Module  │
│  ┌─────────────────────┐│   │ ┌─────────────────────────┐│
│  │   IBleScanner       ││   │ │  IProtocolParser<T>     ││
│  │   Interface         ││   │ │  Template Interface     ││
│  └─────────────────────┘│   │ └─────────────────────────┘│
│  ┌─────────────────────┐│   │ ┌─────────────────────────┐│
│  │  WinRtBleScanner    ││   │ │ AppleContinuityParser   ││
│  │  Implementation     ││   │ │ Implementation          ││
│  └─────────────────────┘│   │ └─────────────────────────┘│
│  ┌─────────────────────┐│   │ ┌─────────────────────────┐│
│  │    BleDevice        ││   │ │    AirPodsData          ││
│  │  Data Structure     ││   │ │   Data Structures       ││
│  └─────────────────────┘│   │ └─────────────────────────┘│
└─────────────────────────┘   └─────────────────────────────┘
              │                           │
              └─────────────┬─────────────┘
                           │
┌─────────────────────────▼─────────────────────────────────┐
│                   Output Module                          │
│            (JSON Formatting & Logging)                   │
└───────────────────────────────────────────────────────────┘
```

## Core Components

### 1. BLE Scanner Module (`Source/ble/`)

**Purpose**: Handles Bluetooth Low Energy device discovery and advertisement capture.

#### Key Files:
- **`IBleScanner.hpp`**: Abstract interface defining scanner contract
- **`WinRtBleScanner.hpp/.cpp`**: Windows Runtime implementation  
- **`BleDevice.hpp/.cpp`**: Device data structures and utilities

#### Responsibilities:
- Bluetooth adapter management
- BLE advertisement scanning
- Device discovery and filtering
- Thread-safe device collection
- Real-time event handling

#### Key Classes:

```cpp
/**
 * @brief Abstract interface for BLE scanning functionality
 */
class IBleScanner {
public:
    using DeviceCallback = std::function<void(const BleDevice&)>;
    
    virtual ~IBleScanner() = default;
    virtual bool Start() = 0;
    virtual bool Stop() = 0;
    virtual bool IsScanning() const = 0;
    virtual std::vector<BleDevice> GetDevices() const = 0;
    virtual void RegisterCallback(DeviceCallback callback) = 0;
    virtual void ClearDevices() = 0;
    virtual size_t GetDeviceCount() const = 0;
};

/**
 * @brief Represents a discovered BLE device with parsed data
 */
struct BleDevice {
    std::string deviceId;
    uint64_t address;
    int rssi;
    std::vector<uint8_t> manufacturerData;
    std::chrono::system_clock::time_point timestamp;
    std::optional<AirPodsData> airpodsData;
    
    // Utility methods
    bool HasAirPodsData() const;
    std::string GetFormattedAddress() const;
    std::string GetManufacturerDataHex() const;
    std::chrono::duration<double> GetAge() const;
};
```

### 2. Protocol Parser Module (`Source/protocol/`)

**Purpose**: Parses manufacturer-specific data according to protocol specifications.

#### Key Files:
- **`IProtocolParser.hpp`**: Template interface for protocol parsers
- **`AppleContinuityParser.hpp/.cpp`**: Apple Continuity Protocol implementation
- **`AirPodsData.hpp/.cpp`**: AirPods-specific data structures

#### Responsibilities:
- Protocol data validation
- Binary data parsing
- Device identification
- Battery level extraction
- Status information parsing

#### Key Classes:

```cpp
/**
 * @brief Template interface for protocol parsers
 */
template<typename T>
class IProtocolParser {
public:
    virtual ~IProtocolParser() = default;
    virtual bool CanParse(const std::vector<uint8_t>& data) const = 0;
    virtual std::optional<T> Parse(const std::vector<uint8_t>& data) const = 0;
    virtual std::string GetParserName() const = 0;
    virtual std::string GetParserVersion() const = 0;
};

/**
 * @brief Complete AirPods device information
 */
struct AirPodsData {
    AirPodsModel model;
    BatteryLevels battery;
    ChargingState charging;
    DeviceState device;
    
    // Utility methods
    std::string GetModelName() const;
    std::string GetBatterySummary() const;
    bool IsCharging() const;
    bool IsInUse() const;
};
```

### 3. Build System (`CMakeLists.txt`)

**Purpose**: Modular build configuration with static libraries and test targets.

#### Library Targets:
- **`protocol_parser`**: Static library containing protocol parsing logic
- **`ble_scanner`**: Static library containing BLE scanning functionality  
- **`airpods_battery_cli_v5`**: Reference implementation executable

#### Test Targets:
- **`test_protocol_parser`**: Unit tests for protocol parsing
- **`modular_parser_test`**: Integration test for modular parser
- **`minimal_test`**: Basic functionality verification
- **`simple_parser_test`**: Simplified parser testing

## Design Principles

### 1. Interface Segregation

Each module exposes only the functionality needed by its clients:

```cpp
// BLE Scanner only exposes scanning functionality
class IBleScanner {
    // No protocol-specific methods
};

// Protocol Parser only exposes parsing functionality  
template<typename T>
class IProtocolParser {
    // No device management methods
};
```

### 2. Dependency Inversion

High-level modules depend on abstractions, not concrete implementations:

```cpp
// Device Processing depends on interfaces, not implementations
class DeviceProcessor {
    std::unique_ptr<IBleScanner> scanner_;
    std::unique_ptr<IProtocolParser<AirPodsData>> parser_;
};
```

### 3. Single Responsibility

Each class has a single, well-defined responsibility:

- `BleDevice`: Represents device data only
- `WinRtBleScanner`: Handles BLE scanning only
- `AppleContinuityParser`: Parses Apple protocol only
- `AirPodsData`: Contains AirPods information only

### 4. Template-Based Extensibility

Protocol parsers use templates to enable future protocol support:

```cpp
// Easy to add new protocols
class BeatsProtocolParser : public IProtocolParser<BeatsData> {
    // Implementation for Beats-specific protocol
};
```

## Module Interfaces

### BLE Scanner Interface Contract

```cpp
class IBleScanner {
public:
    // Lifecycle management
    virtual bool Start() = 0;
    virtual bool Stop() = 0;
    virtual bool IsScanning() const = 0;
    
    // Device access
    virtual std::vector<BleDevice> GetDevices() const = 0;
    virtual size_t GetDeviceCount() const = 0;
    virtual void ClearDevices() = 0;
    
    // Event handling
    virtual void RegisterCallback(DeviceCallback callback) = 0;
};
```

**Threading Requirements**:
- All methods must be thread-safe
- Callbacks executed on scanner thread
- Device collection protected by mutex

**Error Handling**:
- Return `false` for operation failures
- Throw exceptions for critical errors
- Log detailed error information

### Protocol Parser Interface Contract

```cpp
template<typename T>
class IProtocolParser {
public:
    // Validation
    virtual bool CanParse(const std::vector<uint8_t>& data) const = 0;
    
    // Parsing
    virtual std::optional<T> Parse(const std::vector<uint8_t>& data) const = 0;
    
    // Metadata
    virtual std::string GetParserName() const = 0;
    virtual std::string GetParserVersion() const = 0;
};
```

**Contract Requirements**:
- `CanParse()` must be fast and stateless
- `Parse()` returns `std::nullopt` for invalid data
- No exceptions for malformed data
- Thread-safe implementation required

## Data Flow

### Device Discovery Flow

```
1. WinRtBleScanner receives BLE advertisement
   ↓
2. Extract manufacturer data and device info
   ↓
3. Create BleDevice with raw data
   ↓
4. Check if AppleContinuityParser CanParse() data
   ↓
5. If yes, call Parse() to extract AirPodsData
   ↓
6. Store parsed data in BleDevice.airpodsData
   ↓
7. Add device to thread-safe collection
   ↓
8. Trigger registered callbacks with new device
   ↓
9. Output JSON with device information
```

### Protocol Parsing Flow

```
Raw Manufacturer Data (bytes)
   ↓
Company ID Check (0x004C for Apple)
   ↓
Protocol Type Check (0x07 for Proximity Pairing)
   ↓
Data Length Validation (≥8 bytes required)
   ↓
Model ID Extraction (bytes 5-6, little-endian)
   ↓
Battery Data Parsing (nibble extraction)
   ↓
Charging State Analysis (bit manipulation)
   ↓
Device State Parsing (in-ear, case, lid)
   ↓
AirPodsData Structure Population
```

## Threading Model

### Thread Safety Strategy

The scanner implements a **producer-consumer pattern** with thread safety:

#### Producer Thread (BLE Scanner)
- Windows Runtime BLE advertisement callbacks
- Device discovery and data extraction
- Protected device collection updates

#### Consumer Thread (Main Application)
- Device enumeration and access
- JSON output generation
- User interface updates

#### Synchronization Mechanisms

```cpp
class WinRtBleScanner {
private:
    mutable std::mutex devicesMutex_;           // Protects device collection
    std::vector<BleDevice> discoveredDevices_;  // Thread-safe collection
    std::vector<DeviceCallback> callbacks_;     // Callback registry
    
public:
    // Thread-safe device access
    std::vector<BleDevice> GetDevices() const {
        std::lock_guard<std::mutex> lock(devicesMutex_);
        return discoveredDevices_;
    }
};
```

#### Thread Safety Guidelines

1. **Mutex Protection**: All shared data protected by mutexes
2. **Callback Isolation**: Callbacks executed with data copies
3. **RAII Management**: Automatic resource cleanup
4. **Exception Safety**: Strong exception guarantees

## Error Handling Strategy

### Error Categories

1. **System Errors**: Bluetooth adapter issues, permissions
2. **Protocol Errors**: Malformed data, unknown formats  
3. **Resource Errors**: Memory allocation, file access
4. **Logic Errors**: Programming mistakes, invalid parameters

### Error Handling Patterns

```cpp
// System errors - exceptions for critical failures
try {
    scanner->Start();
} catch (const BluetoothException& e) {
    std::cerr << "Bluetooth error: " << e.what() << std::endl;
    return EXIT_FAILURE;
}

// Protocol errors - graceful degradation
auto result = parser->Parse(data);
if (!result) {
    // Continue processing other devices
    continue;
}

// Resource errors - RAII cleanup
class WinRtBleScanner {
    ~WinRtBleScanner() {
        // Automatic cleanup in destructor
        Stop();
    }
};
```

### Logging Strategy

The scanner uses structured logging via spdlog:

```cpp
// Different log levels for different scenarios
spdlog::info("Scanner started successfully");
spdlog::warn("Unknown device model: {:#x}", modelId);
spdlog::error("Failed to start Bluetooth scanner: {}", error);
spdlog::debug("Parsed battery data: L:{}% R:{}% C:{}%", left, right, case_);
```

## Build System

### Modular Libraries

The CMake configuration creates modular static libraries:

```cmake
# Protocol parser library
add_library(protocol_parser STATIC
    Source/protocol/AirPodsData.cpp
    Source/protocol/AppleContinuityParser.cpp
)

# BLE scanner library (depends on protocol_parser)
add_library(ble_scanner STATIC
    Source/ble/BleDevice.cpp
    Source/ble/WinRtBleScanner.cpp
)
target_link_libraries(ble_scanner protocol_parser)
```

### Dependency Chain

```
airpods_battery_cli_v5.exe
    ↓
ble_scanner.lib
    ↓
protocol_parser.lib
    ↓
spdlog (submodule)
    ↓
Windows SDK libraries
```

### Common Build Configuration

```cmake
# Shared compile options
set(COMMON_COMPILE_OPTIONS
    "$<$<CXX_COMPILER_ID:MSVC>:/MP>"        # Multi-processor compilation
    "$<$<CXX_COMPILER_ID:MSVC>:/await>"     # WinRT coroutine support
    "$<$<CXX_COMPILER_ID:MSVC>:/utf-8>"     # UTF-8 support for spdlog
)

# Shared definitions
set(COMMON_COMPILE_DEFINITIONS
    NOMINMAX                                # Prevent min/max macro conflicts
    WIN32_LEAN_AND_MEAN                    # Reduce Windows header size
)
```

## Performance Considerations

### Memory Management

1. **RAII Patterns**: Automatic resource cleanup
2. **Smart Pointers**: Managed memory allocation
3. **Move Semantics**: Efficient data transfer
4. **Minimal Copying**: Reference passing where possible

### CPU Efficiency

1. **Lazy Evaluation**: Parse only when needed
2. **Caching**: Store parsed results to avoid recomputation
3. **Fast Path**: Quick validation before expensive operations
4. **Thread Pool**: Reuse threads for callbacks

### I/O Optimization

1. **Batch Operations**: Group Bluetooth operations
2. **Asynchronous I/O**: Non-blocking network operations
3. **Buffer Management**: Efficient data structures
4. **Minimal Allocations**: Reuse buffers where possible

## Future Extensibility

### Adding New Protocols

To add support for new device protocols:

1. **Define Data Structures**:
```cpp
struct NewProtocolData {
    // Protocol-specific fields
};
```

2. **Implement Parser Interface**:
```cpp
class NewProtocolParser : public IProtocolParser<NewProtocolData> {
    bool CanParse(const std::vector<uint8_t>& data) const override;
    std::optional<NewProtocolData> Parse(const std::vector<uint8_t>& data) const override;
};
```

3. **Register Parser**:
```cpp
// Add to device processing logic
auto newParser = std::make_unique<NewProtocolParser>();
// Registration logic
```

### Adding New Scanner Backends

To support different scanning technologies:

1. **Implement Scanner Interface**:
```cpp
class NewBleScanner : public IBleScanner {
    // Implementation for new scanning technology
};
```

2. **Factory Pattern**:
```cpp
std::unique_ptr<IBleScanner> CreateScanner(ScannerType type) {
    switch (type) {
        case ScannerType::WinRT: return std::make_unique<WinRtBleScanner>();
        case ScannerType::NewTech: return std::make_unique<NewBleScanner>();
    }
}
```

### Configuration System

Future configuration support can be added through:

1. **Configuration Classes**:
```cpp
struct ScannerConfig {
    std::chrono::seconds scanInterval{5};
    bool enableLogging{true};
    LogLevel logLevel{LogLevel::Info};
};
```

2. **Builder Pattern**:
```cpp
auto scanner = ScannerBuilder()
    .withConfig(config)
    .withParser<AirPodsData>(std::make_unique<AppleContinuityParser>())
    .build();
```

---

This architecture provides a solid foundation for the CLI scanner while maintaining flexibility for future enhancements and protocol support. 