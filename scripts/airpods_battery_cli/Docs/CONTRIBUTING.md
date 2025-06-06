# Contributing to AirPods Battery CLI Scanner

Thank you for your interest in contributing to the AirPods Battery CLI Scanner! This document provides guidelines and information for contributors to help maintain code quality and development standards.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Environment](#development-environment)
- [Code Standards](#code-standards)
- [Testing Guidelines](#testing-guidelines)
- [Submission Process](#submission-process)
- [Architecture Guidelines](#architecture-guidelines)
- [Protocol Implementation](#protocol-implementation)
- [Documentation Standards](#documentation-standards)
- [Community Guidelines](#community-guidelines)

## Getting Started

### Prerequisites

Before contributing, ensure you have:

- **Windows 10/11** with Bluetooth hardware
- **Visual Studio 2019/2022** with C++ workload
- **Windows SDK 10.0.26100.0** or later
- **CMake 3.10** or later
- **Git** with submodule support
- **AirPods or Beats device** for testing (recommended)

### Quick Setup

1. **Fork and Clone**:
   ```bash
   git clone --recursive https://github.com/[your-username]/RustPods.git
   cd RustPods/scripts/airpods_battery_cli
   ```

2. **Build and Test**:
   ```bash
   # Build all targets
   cmake -B build -S . -G "Visual Studio 17 2022" -A x64
   cmake --build build --config Release
   
   # Run tests to verify setup
   cd build/Release
   ./modular_parser_test.exe
   ```

3. **Verify V5 Reference**:
   ```bash
   # Test the working V5 scanner
   ./airpods_battery_cli_v5.exe
   ```

### Development Workflow

1. **Create Feature Branch**:
   ```bash
   git checkout -b feature/amazing-improvement
   ```

2. **Make Changes**: Follow the guidelines in this document

3. **Test Thoroughly**: Run all tests and verify functionality

4. **Update Documentation**: Ensure docs reflect your changes

5. **Submit Pull Request**: Following the submission process below

## Development Environment

### IDE Configuration

#### Visual Studio Setup
- **C++ Standard**: C++20
- **Platform**: x64 (Windows 64-bit)
- **Configuration**: Both Debug and Release
- **Extensions**: 
  - CMake Tools for Visual Studio
  - Git for Visual Studio

#### Code Formatting
We use consistent formatting to maintain readability:

```cpp
// Indentation: 4 spaces (no tabs)
// Brace style: Allman/BSD style
// Line length: 120 characters maximum

class ExampleClass 
{
public:
    ExampleClass(int value)
        : value_(value)
    {
        // Constructor implementation
    }

    void DoSomething() 
    {
        if (condition) 
        {
            // Action
        }
    }

private:
    int value_;
};
```

### Build Configuration

#### CMake Best Practices
- Always use out-of-source builds (`build/` directory)
- Support both Debug and Release configurations
- Link libraries properly through `target_link_libraries`
- Use generator expressions for MSVC-specific flags

#### Preprocessor Definitions
Standard definitions for all targets:
```cmake
target_compile_definitions(${TARGET_NAME} PRIVATE
    NOMINMAX                    # Prevent min/max macro conflicts
    WIN32_LEAN_AND_MEAN        # Reduce Windows header size
    UNICODE _UNICODE           # Unicode support
)
```

## Code Standards

### Naming Conventions

#### Classes and Structs
```cpp
// PascalCase for types
class BleScanner { };
struct AirPodsData { };
enum class ScannerState { };
```

#### Functions and Variables
```cpp
// camelCase for functions and variables
void startScanning();
int deviceCount;
bool isConnected;

// Private member variables with trailing underscore
class Scanner 
{
private:
    int deviceCount_;
    std::string scannerName_;
};
```

#### Constants and Enums
```cpp
// ALL_CAPS for constants
const int MAX_DEVICES = 100;
const std::string APPLE_COMPANY_ID = "004C";

// PascalCase for enum values
enum class DeviceState 
{
    Disconnected,
    Connecting,
    Connected,
    Scanning
};
```

### Interface Design

#### Abstract Interfaces
All interfaces should be pure virtual with virtual destructor:

```cpp
class IProtocolParser 
{
public:
    virtual ~IProtocolParser() = default;
    virtual bool CanParse(const std::vector<uint8_t>& data) const = 0;
    virtual std::optional<ParsedData> Parse(const std::vector<uint8_t>& data) const = 0;
    virtual std::string GetParserName() const = 0;
};
```

#### Resource Management
Use RAII patterns consistently:

```cpp
class ResourceManager 
{
public:
    ResourceManager() : resource_(AcquireResource()) { }
    ~ResourceManager() { ReleaseResource(resource_); }
    
    // Delete copy constructor and assignment
    ResourceManager(const ResourceManager&) = delete;
    ResourceManager& operator=(const ResourceManager&) = delete;
    
    // Allow move operations
    ResourceManager(ResourceManager&& other) noexcept 
        : resource_(std::exchange(other.resource_, nullptr)) { }
    
private:
    Resource* resource_;
};
```

### Error Handling

#### Exception Policy
- Use exceptions for **critical system failures** only
- Return `std::optional` or boolean for **recoverable errors**
- Log all errors with appropriate severity

```cpp
// Critical system failure - throw exception
void InitializeBluetooth() 
{
    if (!BluetoothAvailable()) 
    {
        throw BluetoothException("Bluetooth adapter not found");
    }
}

// Recoverable error - return optional
std::optional<DeviceInfo> ParseDevice(const std::vector<uint8_t>& data) 
{
    if (!ValidateData(data)) 
    {
        spdlog::warn("Invalid device data received");
        return std::nullopt;
    }
    return ParseValidData(data);
}
```

#### Logging Guidelines
Use structured logging with appropriate levels:

```cpp
#include <spdlog/spdlog.h>

// Error: System failures, exceptions
spdlog::error("Failed to initialize scanner: {}", error_message);

// Warning: Recoverable issues, deprecated usage
spdlog::warn("Device data validation failed for device {}", device_id);

// Info: Normal operation, important events
spdlog::info("Scanner started successfully, found {} adapters", adapter_count);

// Debug: Detailed information for troubleshooting
spdlog::debug("Parsing battery data: raw={}, left={}%, right={}%", 
              raw_data, left_battery, right_battery);
```

## Testing Guidelines

### Test Structure

#### Unit Tests
Test individual components in isolation:

```cpp
#include <gtest/gtest.h>
#include "AppleContinuityParser.hpp"

class AppleContinuityParserTest : public ::testing::Test 
{
protected:
    void SetUp() override 
    {
        parser_ = std::make_unique<AppleContinuityParser>();
    }

    std::unique_ptr<AppleContinuityParser> parser_;
};

TEST_F(AppleContinuityParserTest, CanParseValidData) 
{
    // Test valid Apple manufacturer data
    std::vector<uint8_t> valid_data = {0x4C, 0x00, 0x07, 0x19, 0x01, 0x14, 0x20, 0x0B};
    EXPECT_TRUE(parser_->CanParse(valid_data));
}

TEST_F(AppleContinuityParserTest, RejectsInvalidData) 
{
    // Test non-Apple manufacturer data
    std::vector<uint8_t> invalid_data = {0xFF, 0xFF, 0x00, 0x00};
    EXPECT_FALSE(parser_->CanParse(invalid_data));
}
```

#### Integration Tests
Test component interactions:

```cpp
TEST(IntegrationTest, ScannerParsesAirPodsData) 
{
    auto scanner = std::make_unique<WinRtBleScanner>();
    auto parser = std::make_unique<AppleContinuityParser>();
    
    // Mock or use real device data
    std::vector<uint8_t> airpods_data = GetRealAirPodsData();
    
    auto parsed = parser->Parse(airpods_data);
    ASSERT_TRUE(parsed.has_value());
    EXPECT_EQ(parsed->model, AirPodsModel::AirPodsPro2);
}
```

### Test Data

#### Real Device Data
Use real AirPods data for validation:

```cpp
// Known good AirPods Pro 2 data
const std::vector<uint8_t> AIRPODS_PRO2_DATA = {
    0x07, 0x19, 0x01, 0x14, 0x20, 0x0B, 0x77, 0x8F,
    // ... complete manufacturer data
};

// Expected parsing results
const AirPodsData EXPECTED_AIRPODS_PRO2 = {
    .model = AirPodsModel::AirPodsPro2,
    .battery = {.left = 70, .right = 70, .case_battery = 0},
    .charging = {.left = true, .right = true, .case_charging = false},
    // ... other expected values
};
```

### Performance Testing

#### Benchmarks
Measure performance of critical operations:

```cpp
#include <chrono>

TEST(PerformanceTest, ParsingSpeed) 
{
    AppleContinuityParser parser;
    std::vector<uint8_t> test_data = GetLargeTestDataSet();
    
    auto start = std::chrono::high_resolution_clock::now();
    
    for (int i = 0; i < 10000; ++i) 
    {
        parser.Parse(test_data);
    }
    
    auto end = std::chrono::high_resolution_clock::now();
    auto duration = std::chrono::duration_cast<std::chrono::microseconds>(end - start);
    
    // Ensure parsing is fast enough
    EXPECT_LT(duration.count(), 1000000); // < 1 second for 10k operations
}
```

## Submission Process

### Pull Request Requirements

#### Before Submitting
1. **All tests pass**: Run complete test suite
2. **No build warnings**: Clean compilation in Release mode
3. **Documentation updated**: Reflect all changes in docs
4. **Code formatted**: Follow formatting standards
5. **Functionality verified**: Test with real devices if possible

#### Pull Request Template
```markdown
## Summary
Brief description of changes and motivation.

## Changes Made
- List specific changes
- Include any breaking changes
- Mention any new dependencies

## Testing
- [ ] All existing tests pass
- [ ] New tests added for new functionality
- [ ] Tested with real AirPods device
- [ ] Performance impact assessed

## Documentation
- [ ] README.md updated if needed
- [ ] Architecture docs updated
- [ ] Code comments added/updated
- [ ] API changes documented

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Breaking changes noted
- [ ] Backward compatibility considered
```

### Review Process

#### What Reviewers Look For
1. **Code Quality**: Adherence to standards and best practices
2. **Architecture**: Proper use of interfaces and design patterns
3. **Testing**: Comprehensive test coverage
4. **Documentation**: Clear and complete documentation
5. **Performance**: No performance regressions
6. **Thread Safety**: Proper synchronization in concurrent code

#### Response to Feedback
- Address all reviewer comments
- Ask questions if feedback is unclear
- Update documentation based on review insights
- Test all requested changes thoroughly

## Architecture Guidelines

### Adding New Protocol Support

#### Protocol Parser Implementation
1. **Define Data Structures**:
```cpp
// Create protocol-specific data structures
struct NewProtocolData 
{
    DeviceModel model;
    BatteryInfo battery;
    // ... protocol-specific fields
};
```

2. **Implement Parser Interface**:
```cpp
class NewProtocolParser : public IProtocolParser<NewProtocolData> 
{
public:
    bool CanParse(const std::vector<uint8_t>& data) const override 
    {
        // Fast validation logic
        return data.size() >= MinDataSize && 
               data[0] == ExpectedCompanyId;
    }
    
    std::optional<NewProtocolData> Parse(const std::vector<uint8_t>& data) const override 
    {
        // Detailed parsing implementation
    }
};
```

3. **Add Comprehensive Tests**:
```cpp
TEST(NewProtocolParserTest, ParsesValidData) 
{
    // Test with known good data
}

TEST(NewProtocolParserTest, HandlesCorruptedData) 
{
    // Test error handling
}
```

### Scanner Backend Implementation

#### Interface Implementation
```cpp
class NewScannerBackend : public IBleScanner 
{
public:
    bool Start() override 
    {
        // Initialize scanning hardware/API
        // Set up event callbacks
        // Return success/failure
    }
    
    void RegisterCallback(DeviceCallback callback) override 
    {
        std::lock_guard<std::mutex> lock(callbackMutex_);
        callbacks_.push_back(callback);
    }
    
private:
    void OnDeviceDiscovered(const RawDeviceData& raw) 
    {
        // Convert to BleDevice
        // Trigger callbacks
    }
};
```

## Protocol Implementation

### Apple Continuity Protocol

#### Data Structure Understanding
```cpp
// Apple manufacturer data structure:
// Bytes 0-1: Company ID (0x004C little-endian)
// Byte 2: Advertisement type (0x07 for proximity pairing)
// Byte 3: Advertisement flags
// Bytes 4-5: Model ID (little-endian)
// Bytes 6-7: Battery data
// Bytes 8+: Additional device-specific data
```

#### Bit Manipulation Patterns
```cpp
// Extract battery level from nibble (0-10 scale to 0-100%)
uint8_t ExtractBatteryLevel(uint8_t nibble) 
{
    uint8_t raw_level = nibble & 0x0F;
    if (raw_level > 10) return 0; // Invalid data
    return raw_level * 10; // Convert to percentage
}

// Extract charging state from bit
bool ExtractChargingState(uint8_t byte, int bit_position) 
{
    return (byte >> bit_position) & 0x01;
}
```

### Adding New Device Support

#### Model Identification
1. **Capture Real Data**: Use existing scanner to capture manufacturer data
2. **Analyze Pattern**: Identify model ID and data structure
3. **Add to Parser**: Extend existing parser or create new one
4. **Comprehensive Testing**: Test with multiple device states

```cpp
// Add new model to existing enum
enum class AirPodsModel : uint16_t 
{
    AirPods1 = 0x200E,
    AirPods2 = 0x200F,
    AirPods3 = 0x2013,
    AirPodsPro = 0x200E,  // Same as AirPods1, differentiated by other data
    AirPodsPro2 = 0x2014,
    AirPodsMax = 0x200A,
    NewAirPodsModel = 0x2020,  // New model ID discovered
};
```

## Documentation Standards

### Code Documentation

#### Header Comments
```cpp
/**
 * @file BleScanner.hpp
 * @brief Bluetooth Low Energy device scanning functionality
 * @author Contributor Name
 * @date YYYY-MM-DD
 * @version X.Y.Z
 * 
 * This file contains the interface and implementation for BLE device
 * scanning, including real-time advertisement capture and device
 * filtering capabilities.
 */
```

#### Class Documentation
```cpp
/**
 * @class AppleContinuityParser
 * @brief Parser for Apple's Continuity Protocol BLE advertisements
 * 
 * This class implements parsing of Apple's proprietary Continuity Protocol
 * used by AirPods and other Apple devices to broadcast battery and status
 * information via BLE advertisements.
 * 
 * @details The parser supports:
 * - All major AirPods models (1st gen through Pro 2)
 * - Battery level extraction (0-100%)
 * - Charging state detection
 * - In-ear detection status
 * - Case lid position
 * 
 * @example
 * @code
 * AppleContinuityParser parser;
 * std::vector<uint8_t> data = GetBluetoothAdvertisementData();
 * 
 * if (parser.CanParse(data)) {
 *     auto result = parser.Parse(data);
 *     if (result) {
 *         std::cout << "Battery: " << result->battery.left << "%" << std::endl;
 *     }
 * }
 * @endcode
 * 
 * @see IProtocolParser
 * @see AirPodsData
 */
class AppleContinuityParser : public IProtocolParser<AirPodsData>
{
    // Implementation...
};
```

---

Thank you for contributing to the AirPods Battery CLI Scanner! Your efforts help make this project better for everyone.
