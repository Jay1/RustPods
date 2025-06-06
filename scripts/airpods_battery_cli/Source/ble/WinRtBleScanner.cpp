#include "WinRtBleScanner.hpp"
#include "../protocol/AppleContinuityParser.hpp"
#include <iostream>
#include <sstream>
#include <iomanip>

// Constant for Apple Company ID (from v5 scanner)
constexpr uint16_t APPLE_COMPANY_ID = 76;

WinRtBleScanner::WinRtBleScanner() 
    : lastStartTime_(std::chrono::steady_clock::now())
{
    // Set up WinRT event handlers using std::bind (from v5 scanner)
    bleWatcher_.Received(std::bind(&WinRtBleScanner::OnAdvertisementReceived, this, std::placeholders::_2));
    bleWatcher_.Stopped(std::bind(&WinRtBleScanner::OnScannerStopped, this, std::placeholders::_2));
}

WinRtBleScanner::~WinRtBleScanner() {
    if (!destroyRequested_) {
        destroyRequested_ = true;
        Stop();
        
        // Wait for cleanup with timeout (from v5 scanner)
        std::unique_lock<std::mutex> lock{conditionMutex_};
        destroyCondition_.wait_for(lock, std::chrono::seconds(1));
    }
}

bool WinRtBleScanner::Start() {
    try {
        stopRequested_ = false;
        lastStartTime_ = std::chrono::steady_clock::now();

        std::lock_guard<std::mutex> lock{devicesMutex_};
        bleWatcher_.Start();
        
        std::cout << "[INFO] Bluetooth AdvWatcher start succeeded." << std::endl;
        return true;
    }
    catch (const std::exception& ex) {
        std::cout << "[ERROR] Start adv watcher exception: " << ex.what() << std::endl;
        return false;
    }
}

bool WinRtBleScanner::Stop() {
    try {
        stopRequested_ = true;
        stopCondition_.notify_all();

        std::lock_guard<std::mutex> lock{devicesMutex_};
        bleWatcher_.Stop();
        
        std::cout << "[INFO] Bluetooth AdvWatcher stop succeeded." << std::endl;
        return true;
    }
    catch (const std::exception& ex) {
        std::cout << "[ERROR] Stop adv watcher exception: " << ex.what() << std::endl;
        return false;
    }
}

bool WinRtBleScanner::IsScanning() const {
    std::lock_guard<std::mutex> lock{devicesMutex_};
    return bleWatcher_.Status() == WinrtBluetoothAdv::BluetoothLEAdvertisementWatcherStatus::Started;
}

std::vector<BleDevice> WinRtBleScanner::GetDevices() const {
    std::lock_guard<std::mutex> lock{devicesMutex_};
    return devices_;
}

void WinRtBleScanner::RegisterCallback(DeviceCallback callback) {
    deviceCallback_ = std::move(callback);
}

void WinRtBleScanner::ClearDevices() {
    std::lock_guard<std::mutex> lock{devicesMutex_};
    devices_.clear();
}

size_t WinRtBleScanner::GetDeviceCount() const {
    std::lock_guard<std::mutex> lock{devicesMutex_};
    return devices_.size();
}

void WinRtBleScanner::OnAdvertisementReceived(
    const WinrtBluetoothAdv::BluetoothLEAdvertisementReceivedEventArgs& args
) {
    // Extract basic information (exactly as in v5 scanner)
    int32_t rssi = args.RawSignalStrengthInDBm();
    WinrtFoundation::DateTime timestamp = args.Timestamp();
    uint64_t address = args.BluetoothAddress();

    // Process manufacturer data (exactly as in v5 scanner)
    const auto& manufacturerDataArray = args.Advertisement().ManufacturerData();
    for (uint32_t i = 0; i < manufacturerDataArray.Size(); ++i) {
        const auto& manufacturerData = manufacturerDataArray.GetAt(i);
        const auto companyId = manufacturerData.CompanyId();
        const auto& data = manufacturerData.Data();

        // Convert WinRT data to std::vector (exactly as in v5 scanner)
        std::vector<uint8_t> stdData(data.data(), data.data() + data.Length());
        
        // Process manufacturer data
        ProcessManufacturerData(address, rssi, timestamp, stdData, companyId);
    }
}

void WinRtBleScanner::OnScannerStopped(
    const WinrtBluetoothAdv::BluetoothLEAdvertisementWatcherStoppedEventArgs& args
) {
    std::unique_lock<std::mutex> lock{devicesMutex_};
    auto status = bleWatcher_.Status();
    lock.unlock();

    std::cout << "[INFO] BLE advertisement scan stopped." << std::endl;

    // Automatic restart logic (exactly as in v5 scanner)
    if (!destroyRequested_) {
        do {
            std::unique_lock<std::mutex> lock{conditionMutex_};
            stopCondition_.wait_until(lock, lastStartTime_.load() + RETRY_INTERVAL);
        } while (!stopRequested_ && !Start());
    }
    else {
        destroyCondition_.notify_all();
    }
}

void WinRtBleScanner::ProcessManufacturerData(
    uint64_t address,
    int32_t rssi,
    WinrtFoundation::DateTime timestamp,
    const std::vector<uint8_t>& manufacturerData,
    uint16_t companyId
) {
    // Only process Apple devices (exactly as in v5 scanner)
    if (companyId == APPLE_COMPANY_ID) {
        // Create device ID as hex string (exactly as in v5 scanner)
        std::stringstream ss;
        ss << std::hex << std::setfill('0') << std::setw(12) << address;
        std::string deviceId = ss.str();

        // Create BLE device
        BleDevice device(deviceId, address, rssi, manufacturerData);
        device.timestamp = ConvertWinRtTime(timestamp);

        // Parse AirPods data using the protocol parser
        AppleContinuityParser parser;
        device.airpodsData = parser.Parse(manufacturerData);
        
        // Log detection (exactly as in v5 scanner)
        if (device.airpodsData.has_value()) {
            const auto& airpods = device.airpodsData.value();
            std::cout << "[INFO] AirPods detected: " << airpods.model 
                      << " - Left:" << airpods.batteryLevels.left 
                      << "% Right:" << airpods.batteryLevels.right 
                      << "% Case:" << airpods.batteryLevels.case_ << "%" << std::endl;
        } else {
            std::cout << "[INFO] Apple device detected: " << device.GetManufacturerDataHex() << std::endl;
        }
        
        // Add device to collection
        AddDevice(device);
    }
}

void WinRtBleScanner::AddDevice(const BleDevice& device) {
    {
        std::lock_guard<std::mutex> lock{devicesMutex_};
        devices_.push_back(device);
    }
    
    // Notify callback if registered
    if (deviceCallback_) {
        deviceCallback_(device);
    }
}

std::chrono::system_clock::time_point WinRtBleScanner::ConvertWinRtTime(
    WinrtFoundation::DateTime winrtTime
) const {
    // Convert WinRT DateTime to system_clock time_point
    // WinRT DateTime is in 100-nanosecond intervals since January 1, 1601
    constexpr int64_t EPOCH_DIFFERENCE = 11644473600LL; // Seconds between 1601 and 1970
    constexpr int64_t TICKS_PER_SECOND = 10000000LL;    // 100-nanosecond ticks per second
    
    int64_t ticks = winrtTime.time_since_epoch().count();
    int64_t seconds = (ticks / TICKS_PER_SECOND) - EPOCH_DIFFERENCE;
    int64_t nanoseconds = (ticks % TICKS_PER_SECOND) * 100;
    
    return std::chrono::system_clock::time_point(
        std::chrono::duration_cast<std::chrono::system_clock::duration>(
            std::chrono::seconds(seconds) + std::chrono::nanoseconds(nanoseconds)
        )
    );
} 