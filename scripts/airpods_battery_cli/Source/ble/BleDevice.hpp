#pragma once

#include <string>
#include <vector>
#include <optional>
#include <chrono>
#include <cstdint>

// Include AirPods data structures for std::optional support
#include "protocol/AirPodsData.hpp"

/**
 * @brief Represents a Bluetooth Low Energy device discovered during scanning
 * 
 * This structure contains all relevant information about a BLE device,
 * including raw advertisement data and parsed protocol-specific information.
 */
struct BleDevice {
    /// Unique device identifier (typically MAC address as hex string)
    std::string deviceId;
    
    /// Raw Bluetooth address as 64-bit integer
    uint64_t address;
    
    /// Received Signal Strength Indicator in dBm
    int rssi;
    
    /// Raw manufacturer-specific data from BLE advertisement
    std::vector<uint8_t> manufacturerData;
    
    /// Timestamp when the device was discovered
    std::chrono::system_clock::time_point timestamp;
    
    /// Parsed AirPods data (if the device is an AirPods device)
    std::optional<AirPodsData> airpodsData;

    /**
     * @brief Default constructor
     */
    BleDevice() = default;

    /**
     * @brief Constructor with basic device information
     * @param deviceId Unique device identifier
     * @param address Raw Bluetooth address
     * @param rssi Signal strength in dBm
     * @param manufacturerData Raw manufacturer data
     */
    BleDevice(
        const std::string& deviceId,
        uint64_t address,
        int rssi,
        const std::vector<uint8_t>& manufacturerData
    );

    /**
     * @brief Check if this device has valid AirPods data
     * @return true if airpodsData is present and valid
     */
    bool HasAirPodsData() const;

    /**
     * @brief Get a formatted address string
     * @return MAC address formatted as XX:XX:XX:XX:XX:XX
     */
    std::string GetFormattedAddress() const;

    /**
     * @brief Get manufacturer data as hex string
     * @return Hex representation of manufacturer data
     */
    std::string GetManufacturerDataHex() const;

    /**
     * @brief Get the age of this device record
     * @return Duration since the device was discovered
     */
    std::chrono::duration<double> GetAge() const;
};

/**
 * @brief Compare two BLE devices for equality based on address
 * @param lhs First device
 * @param rhs Second device
 * @return true if devices have the same address
 */
bool operator==(const BleDevice& lhs, const BleDevice& rhs);

/**
 * @brief Compare two BLE devices for inequality
 * @param lhs First device
 * @param rhs Second device
 * @return true if devices have different addresses
 */
bool operator!=(const BleDevice& lhs, const BleDevice& rhs); 