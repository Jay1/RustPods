#pragma once

#include <vector>
#include <functional>
#include <memory>

// Forward declarations
struct BleDevice;

/**
 * @brief Interface for Bluetooth Low Energy device scanning
 * 
 * This interface provides a clean abstraction for BLE advertisement scanning,
 * allowing for different implementations (WinRT, native, mock) while maintaining
 * consistent behavior and thread safety.
 */
class IBleScanner {
public:
    /// Callback function signature for device discovery events
    using DeviceCallback = std::function<void(const BleDevice& device)>;

    virtual ~IBleScanner() = default;

    /**
     * @brief Start the BLE advertisement scanning process
     * @return true if scanning started successfully, false otherwise
     */
    virtual bool Start() = 0;

    /**
     * @brief Stop the BLE advertisement scanning process
     * @return true if scanning stopped successfully, false otherwise
     */
    virtual bool Stop() = 0;

    /**
     * @brief Check if the scanner is currently active
     * @return true if scanning is in progress, false otherwise
     */
    virtual bool IsScanning() const = 0;

    /**
     * @brief Get all discovered devices
     * @return Vector of discovered BLE devices
     */
    virtual std::vector<BleDevice> GetDevices() const = 0;

    /**
     * @brief Register a callback for real-time device discovery
     * @param callback Function to call when a new device is discovered
     */
    virtual void RegisterCallback(DeviceCallback callback) = 0;

    /**
     * @brief Clear all discovered devices from the internal storage
     */
    virtual void ClearDevices() = 0;

    /**
     * @brief Get the number of devices discovered so far
     * @return Number of discovered devices
     */
    virtual size_t GetDeviceCount() const = 0;
};

/// Smart pointer type for BLE scanner instances
using BleScannerPtr = std::unique_ptr<IBleScanner>; 