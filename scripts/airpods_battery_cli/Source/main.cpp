// AirPods Battery CLI v6.0 - Modular Architecture
// Professional modular implementation using separated components

#include <iostream>
#include <string>
#include <vector>
#include <chrono>
#include <thread>
#include <iomanip>
#include <sstream>
#include <ctime>
#include <optional>

// Modular components
#include "ble/WinRtBleScanner.hpp"
#include "protocol/AppleContinuityParser.hpp"
#include "ble/BleDevice.hpp"

/**
 * @brief Convert byte vector to hexadecimal string
 * @param data Vector of bytes to convert
 * @return Hexadecimal string representation
 */
std::string to_hex_string(const std::vector<uint8_t>& data) {
    if (data.empty()) return "";
    std::stringstream ss;
    ss << std::hex << std::setfill('0');
    for (const auto& byte : data) {
        ss << std::setw(2) << static_cast<int>(byte);
    }
    return ss.str();
}

/**
 * @brief Convert AirPodsData to v5-compatible JSON format
 * @param airpods The AirPods data to convert
 * @return JSON string representation
 */
std::string airpods_to_json(const AirPodsData& airpods) {
    std::stringstream ss;
    ss << "            \"airpods_data\": {" << std::endl;
    ss << "                \"model\": \"" << airpods.model << "\"," << std::endl;
    ss << "                \"model_id\": \"" << airpods.modelId << "\"," << std::endl;
    ss << "                \"left_battery\": " << airpods.batteryLevels.left << "," << std::endl;
    ss << "                \"right_battery\": " << airpods.batteryLevels.right << "," << std::endl;
    ss << "                \"case_battery\": " << airpods.batteryLevels.case_ << "," << std::endl;
    ss << "                \"left_charging\": " << (airpods.chargingState.leftCharging ? "true" : "false") << "," << std::endl;
    ss << "                \"right_charging\": " << (airpods.chargingState.rightCharging ? "true" : "false") << "," << std::endl;
    ss << "                \"case_charging\": " << (airpods.chargingState.caseCharging ? "true" : "false") << "," << std::endl;
    ss << "                \"left_in_ear\": " << (airpods.deviceState.leftInEar ? "true" : "false") << "," << std::endl;
    ss << "                \"right_in_ear\": " << (airpods.deviceState.rightInEar ? "true" : "false") << "," << std::endl;
    ss << "                \"both_in_case\": " << (airpods.deviceState.bothInCase ? "true" : "false") << "," << std::endl;
    ss << "                \"lid_open\": " << (airpods.deviceState.lidOpen ? "true" : "false") << "," << std::endl;
    ss << "                \"broadcasting_ear\": \"" << airpods.broadcastingEar << "\"" << std::endl;
    ss << "            }";
    return ss.str();
}

/**
 * @brief Output devices in v5-compatible JSON format
 * @param devices Vector of discovered BLE devices
 */
void output_json(const std::vector<BleDevice>& devices) {
    auto timestamp = std::time(nullptr);
    AppleContinuityParser parser; // For parsing any unprocessed devices
    
    std::cout << "{" << std::endl;
    std::cout << "    \"scanner_version\": \"6.0\"," << std::endl;
    std::cout << "    \"scan_timestamp\": \"" << timestamp << "\"," << std::endl;
    std::cout << "    \"total_devices\": " << devices.size() << "," << std::endl;
    std::cout << "    \"devices\": [" << std::endl;
    
    int airpods_count = 0;
    bool first = true;
    
    for (const auto& device : devices) {
        if (!first) std::cout << "," << std::endl;
        first = false;
        
        std::cout << "        {" << std::endl;
        std::cout << "            \"device_id\": \"" << device.deviceId << "\"," << std::endl;
        std::cout << "            \"address\": \"" << device.GetFormattedAddress() << "\"," << std::endl;
        std::cout << "            \"rssi\": " << device.rssi << "," << std::endl;
        std::cout << "            \"manufacturer_data_hex\": \"" << device.GetManufacturerDataHex() << "\"," << std::endl;
        
        // Check if we have parsed AirPods data
        auto airpods_data = device.airpodsData;
        if (!airpods_data.has_value() && parser.CanParse(device.manufacturerData)) {
            // Try to parse with our modular parser if not already parsed
            airpods_data = parser.Parse(device.manufacturerData);
        }
        
        if (airpods_data.has_value()) {
            airpods_count++;
            std::cout << airpods_to_json(airpods_data.value()) << std::endl;
        } else {
            std::cout << "            \"airpods_data\": null" << std::endl;
        }
        
        std::cout << "        }";
    }
    
    std::cout << std::endl << "    ]," << std::endl;
    std::cout << "    \"airpods_count\": " << airpods_count << "," << std::endl;
    std::cout << "    \"status\": \"success\"," << std::endl;
    std::cout << "    \"note\": \"Modular AirPods Battery CLI v6.0 - Enhanced modular architecture\"" << std::endl;
    std::cout << "}" << std::endl;
}

/**
 * @brief Check if any AirPods devices have been found
 * @param devices Vector of discovered devices
 * @param parser Protocol parser for checking unprocessed devices
 * @return True if AirPods found, false otherwise
 */
bool has_airpods(const std::vector<BleDevice>& devices, AppleContinuityParser& parser) {
    for (const auto& device : devices) {
        // Check if device already has AirPods data
        if (device.airpodsData.has_value()) {
            return true;
        }
        
        // Check if device can be parsed as AirPods
        if (parser.CanParse(device.manufacturerData)) {
            auto parsed = parser.Parse(device.manufacturerData);
            if (parsed.has_value()) {
                return true;
            }
        }
    }
    return false;
}

/**
 * @brief Main application entry point
 * @param argc Number of command line arguments
 * @param argv Array of command line arguments
 * @return Exit code (0 for success, 1 for error)
 */
int main(int argc, char* argv[]) {
    try {
        std::cerr << "AirPods Battery CLI v6.0 - Modular Architecture" << std::endl;
        
        // Parse command line arguments (same as v5)
        int scan_duration = 4; // Default reduced from 10 to 4 seconds (60% faster)
        bool early_exit = false;
        bool continuous_until_found = false; // Continuous mode
        
        for (int i = 1; i < argc; i++) {
            std::string arg = argv[i];
            if (arg == "--duration" && i + 1 < argc) {
                scan_duration = std::stoi(argv[++i]);
                if (scan_duration < 1 || scan_duration > 30) {
                    scan_duration = 4; // Reset to default if invalid
                }
            } else if (arg == "--fast" || arg == "-f") {
                scan_duration = 2; // Ultra-fast 2-second scan
                early_exit = true;
            } else if (arg == "--quick" || arg == "-q") {
                scan_duration = 3; // Quick 3-second scan
                early_exit = true;
            } else if (arg == "--continuous" || arg == "-c") {
                continuous_until_found = true; // Scan continuously until AirPods found
                scan_duration = 30; // Maximum scan time as safety net
                std::cerr << "[INFO] Continuous scanning mode - will stop when AirPods found" << std::endl;
            } else if (arg == "--early-exit") {
                early_exit = true;
            }
        }
        
        // Initialize modular components
        WinRtBleScanner scanner;
        AppleContinuityParser parser;
        
        std::cerr << "[INFO] Using modular components:" << std::endl;
        std::cerr << "[INFO]   - " << parser.GetParserName() << " v" << parser.GetParserVersion() << std::endl;
        std::cerr << "[INFO]   - WinRT BLE Scanner (modular)" << std::endl;
        
        if (scanner.Start()) {
            if (continuous_until_found) {
                std::cerr << "[INFO] Scanning continuously until AirPods found (max " << scan_duration << " seconds)..." << std::endl;
                
                // Check for AirPods every 200ms for responsiveness
                int max_checks = scan_duration * 5; // 200ms intervals
                bool airpods_found = false;
                
                for (int i = 0; i < max_checks && !airpods_found; i++) {
                    std::this_thread::sleep_for(std::chrono::milliseconds(200));
                    
                    const auto& devices = scanner.GetDevices();
                    if (has_airpods(devices, parser)) {
                        airpods_found = true;
                        std::cerr << "[INFO] AirPods found after " << ((i + 1) * 0.2) << " seconds - stopping scan" << std::endl;
                        break;
                    }
                }
                
                if (!airpods_found) {
                    std::cerr << "[INFO] No AirPods found within " << scan_duration << " seconds - stopping scan" << std::endl;
                }
            } else {
                std::cerr << "[INFO] Scanning for " << scan_duration << " seconds..." << std::endl;
                
                if (early_exit) {
                    // Check for AirPods every 500ms and exit early if found
                    int checks = scan_duration * 2; // 500ms intervals
                    for (int i = 0; i < checks; i++) {
                        std::this_thread::sleep_for(std::chrono::milliseconds(500));
                        
                        const auto& devices = scanner.GetDevices();
                        if (has_airpods(devices, parser)) {
                            std::cerr << "[INFO] AirPods found, stopping early after " << ((i + 1) * 0.5) << " seconds" << std::endl;
                            break;
                        }
                    }
                } else {
                    // Regular fixed-duration scan
                    std::this_thread::sleep_for(std::chrono::seconds(scan_duration));
                }
            }
            
            scanner.Stop();
            
            // Give a small delay to ensure the stop event is fully processed
            std::this_thread::sleep_for(std::chrono::milliseconds(100));
            
            const auto& devices = scanner.GetDevices();
            output_json(devices);
        } else {
            std::cout << "{\"scanner_version\":\"6.0\",\"status\":\"error\",\"error\":\"Failed to start BLE scan\",\"total_devices\":0,\"devices\":[],\"airpods_count\":0}" << std::endl;
            return 1;
        }
        
        return 0;
    }
    catch (const std::exception& e) {
        std::cout << "{\"scanner_version\":\"6.0\",\"status\":\"error\",\"error\":\"" << e.what() << "\",\"total_devices\":0,\"devices\":[],\"airpods_count\":0}" << std::endl;
        return 1;
    }
} 