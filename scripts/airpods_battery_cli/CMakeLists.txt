cmake_minimum_required(VERSION 3.10)

project(airpods_battery_cli)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Find required packages (optional for standalone build)
# find_package(spdlog CONFIG REQUIRED)
# find_package(nlohmann_json CONFIG REQUIRED)

# Common compile options for Windows/MSVC
set(COMMON_COMPILE_OPTIONS
    "$<$<CXX_COMPILER_ID:MSVC>:/MP>"        # Multi-processor compilation
    "$<$<CXX_COMPILER_ID:MSVC>:/await>"     # Enable legacy coroutine support for WinRT
    "$<$<CXX_COMPILER_ID:MSVC>:/utf-8>"     # UTF-8 support
)

set(COMMON_COMPILE_DEFINITIONS
    NOMINMAX
    WIN32_LEAN_AND_MEAN
)

# ===== V5 Scanner - Reference Implementation =====
add_executable(airpods_battery_cli_v5 Source/airpods_battery_cli_v5.cpp)

set_target_properties(airpods_battery_cli_v5 PROPERTIES
    CXX_STANDARD 20
    CXX_STANDARD_REQUIRED YES
)

target_compile_options(airpods_battery_cli_v5 PRIVATE ${COMMON_COMPILE_OPTIONS})
target_compile_definitions(airpods_battery_cli_v5 PRIVATE ${COMMON_COMPILE_DEFINITIONS})
target_link_libraries(airpods_battery_cli_v5 windowsapp)

message(STATUS "V5 Scanner configured - Reference implementation")

# ===== Modular Architecture Libraries =====

# Protocol Parser Library
add_library(protocol_parser STATIC
    Source/protocol/AirPodsData.cpp
    Source/protocol/AppleContinuityParser.cpp
)

set_target_properties(protocol_parser PROPERTIES
    CXX_STANDARD 20
    CXX_STANDARD_REQUIRED YES
)

target_compile_options(protocol_parser PRIVATE ${COMMON_COMPILE_OPTIONS})
target_compile_definitions(protocol_parser PRIVATE ${COMMON_COMPILE_DEFINITIONS})

target_include_directories(protocol_parser 
    PUBLIC Source
    PUBLIC Source/protocol
)

# BLE Scanner Library  
add_library(ble_scanner STATIC
    Source/ble/BleDevice.cpp
    Source/ble/WinRtBleScanner.cpp
)

set_target_properties(ble_scanner PROPERTIES
    CXX_STANDARD 20
    CXX_STANDARD_REQUIRED YES
)

target_compile_options(ble_scanner PRIVATE ${COMMON_COMPILE_OPTIONS})
target_compile_definitions(ble_scanner PRIVATE ${COMMON_COMPILE_DEFINITIONS})

target_include_directories(ble_scanner 
    PUBLIC Source
    PUBLIC Source/ble
    PUBLIC Source/protocol
)

target_link_libraries(ble_scanner 
    PUBLIC protocol_parser
    PUBLIC windowsapp
)

# ===== Test Executables =====

# Protocol Parser Test
add_executable(test_protocol_parser Source/test_protocol_parser.cpp)
set_target_properties(test_protocol_parser PROPERTIES CXX_STANDARD 20)
target_compile_options(test_protocol_parser PRIVATE ${COMMON_COMPILE_OPTIONS})
target_compile_definitions(test_protocol_parser PRIVATE ${COMMON_COMPILE_DEFINITIONS})
target_link_libraries(test_protocol_parser protocol_parser)

# Modular Parser Test (File Output)
add_executable(modular_parser_test Source/modular_parser_test.cpp)
set_target_properties(modular_parser_test PROPERTIES CXX_STANDARD 20)
target_compile_options(modular_parser_test PRIVATE ${COMMON_COMPILE_OPTIONS})
target_compile_definitions(modular_parser_test PRIVATE ${COMMON_COMPILE_DEFINITIONS})
target_link_libraries(modular_parser_test protocol_parser)

# Simple Parser Test
add_executable(simple_parser_test Source/simple_parser_test.cpp)
set_target_properties(simple_parser_test PROPERTIES CXX_STANDARD 20)
target_compile_options(simple_parser_test PRIVATE ${COMMON_COMPILE_OPTIONS})
target_compile_definitions(simple_parser_test PRIVATE ${COMMON_COMPILE_DEFINITIONS})
target_link_libraries(simple_parser_test protocol_parser)

# Minimal Test
add_executable(minimal_test Source/minimal_test.cpp)
set_target_properties(minimal_test PROPERTIES CXX_STANDARD 20)
target_compile_options(minimal_test PRIVATE ${COMMON_COMPILE_OPTIONS})
target_compile_definitions(minimal_test PRIVATE ${COMMON_COMPILE_DEFINITIONS})

# ===== Production CLI Scanner (Future) =====
# TODO: Complete integration of modular components with v5 functionality
# add_executable(airpods_battery_cli Source/main.cpp)
# set_target_properties(airpods_battery_cli PROPERTIES
#     CXX_STANDARD 20
#     CXX_STANDARD_REQUIRED YES
# )
# target_compile_options(airpods_battery_cli PRIVATE ${COMMON_COMPILE_OPTIONS})
# target_compile_definitions(airpods_battery_cli PRIVATE ${COMMON_COMPILE_DEFINITIONS})
# target_link_libraries(airpods_battery_cli ble_scanner protocol_parser windowsapp)

message(STATUS "Modular architecture configured:")
message(STATUS "  - protocol_parser: Static library for Apple Continuity Protocol parsing")
message(STATUS "  - ble_scanner: Static library for BLE advertisement scanning")
message(STATUS "  - Test executables: test_protocol_parser, modular_parser_test, simple_parser_test, minimal_test")
message(STATUS "  - V5 reference: airpods_battery_cli_v5 (preserved as gold standard)")
