#pragma once

#include "IBleScanner.hpp"
#include "BleDevice.hpp"
#include <mutex>
#include <atomic>
#include <condition_variable>
#include <chrono>
#include <thread>

// Fix DirectX assertion issues (from v5 scanner)
#define assert(expr) ((void)0)

// Windows WinRT includes (exact same as v5 scanner)
#include <winrt/Windows.Foundation.h>
#include <winrt/Windows.Foundation.Collections.h>
#include <winrt/Windows.Storage.Streams.h>
#include <winrt/Windows.Devices.Enumeration.h>
#include <winrt/Windows.Devices.Bluetooth.Advertisement.h>
#include <winrt/Windows.Networking.h>

// Namespace aliases (exact same as v5 scanner)
namespace WinrtFoundation = winrt::Windows::Foundation;
namespace WinrtBluetooth = winrt::Windows::Devices::Bluetooth;
namespace WinrtBluetoothAdv = winrt::Windows::Devices::Bluetooth::Advertisement;
namespace WinrtDevicesEnumeration = winrt::Windows::Devices::Enumeration;

/**
 * @brief WinRT-based implementation of the BLE scanner interface
 * 
 * This implementation uses Windows Runtime Bluetooth LE Advertisement Watcher
 * to scan for BLE devices. It preserves the exact functionality of the v5 scanner
 * while providing a clean, modular interface.
 */
class WinRtBleScanner : public IBleScanner {
public:
    /**
     * @brief Constructor
     * Initializes the WinRT advertisement watcher and sets up callbacks
     */
    WinRtBleScanner();

    /**
     * @brief Destructor
     * Ensures proper cleanup of resources and stops scanning if active
     */
    ~WinRtBleScanner() override;

    // IBleScanner interface implementation
    bool Start() override;
    bool Stop() override;
    bool IsScanning() const override;
    std::vector<BleDevice> GetDevices() const override;
    void RegisterCallback(DeviceCallback callback) override;
    void ClearDevices() override;
    size_t GetDeviceCount() const override;

private:
    /// Retry interval for automatic restart (from v5 scanner)
    static constexpr auto RETRY_INTERVAL = std::chrono::seconds(3);

    /// WinRT Bluetooth LE advertisement watcher
    WinrtBluetoothAdv::BluetoothLEAdvertisementWatcher bleWatcher_;

    /// Mutex for thread-safe access to device collection
    mutable std::mutex devicesMutex_;

    /// Collection of discovered devices
    std::vector<BleDevice> devices_;

    /// Callback for device discovery events
    DeviceCallback deviceCallback_;

    /// Atomic flags for state management
    std::atomic<bool> stopRequested_{false};
    std::atomic<bool> destroyRequested_{false};
    std::atomic<std::chrono::steady_clock::time_point> lastStartTime_;

    /// Condition variables for synchronization
    std::mutex conditionMutex_;
    std::condition_variable stopCondition_;
    std::condition_variable destroyCondition_;

    /**
     * @brief Handle received BLE advertisement
     * @param args Advertisement event arguments from WinRT
     * 
     * This method preserves the exact logic from the v5 scanner for processing
     * BLE advertisements and extracting manufacturer data.
     */
    void OnAdvertisementReceived(
        const WinrtBluetoothAdv::BluetoothLEAdvertisementReceivedEventArgs& args
    );

    /**
     * @brief Handle scanner stopped event
     * @param args Stopped event arguments from WinRT
     * 
     * This method implements automatic restart logic from the v5 scanner.
     */
    void OnScannerStopped(
        const WinrtBluetoothAdv::BluetoothLEAdvertisementWatcherStoppedEventArgs& args
    );

    /**
     * @brief Process manufacturer data and create BLE device
     * @param address Bluetooth address
     * @param rssi Signal strength
     * @param timestamp Discovery timestamp
     * @param manufacturerData Raw manufacturer data
     * @param companyId Company identifier
     * 
     * This method handles the device creation and Apple device filtering
     * logic from the v5 scanner.
     */
    void ProcessManufacturerData(
        uint64_t address,
        int32_t rssi,
        WinrtFoundation::DateTime timestamp,
        const std::vector<uint8_t>& manufacturerData,
        uint16_t companyId
    );

    /**
     * @brief Add a device to the collection
     * @param device The device to add
     * 
     * This method handles thread-safe device storage and callback notification.
     */
    void AddDevice(const BleDevice& device);

    /**
     * @brief Convert WinRT DateTime to system_clock time_point
     * @param winrtTime WinRT DateTime value
     * @return Equivalent system_clock time_point
     */
    std::chrono::system_clock::time_point ConvertWinRtTime(
        WinrtFoundation::DateTime winrtTime
    ) const;
}; 