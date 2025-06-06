// AirPods Battery CLI v5.0 - Standalone AirPods Battery Monitor
// Professional refactoring of AirPodsDesktop BLE scanning for CLI use

#include <iostream>
#include <string>
#include <vector>
#include <unordered_map>
#include <chrono>
#include <thread>
#include <atomic>
#include <mutex>
#include <iomanip>
#include <sstream>
#include <ctime>
#include <optional>
#include <condition_variable>
#include <functional>

// Fix DirectX assertion issues (from AirPodsDesktop)
#define assert(expr) ((void)0)

// Windows WinRT includes (exact same as AirPodsDesktop)
#include <winrt/Windows.Foundation.h>
#include <winrt/Windows.Foundation.Collections.h>
#include <winrt/Windows.Storage.Streams.h>
#include <winrt/Windows.Devices.Enumeration.h>
#include <winrt/Windows.Devices.Bluetooth.Advertisement.h>
#include <winrt/Windows.Networking.h>

// Namespace aliases (exact same as AirPodsDesktop)
using namespace std::chrono_literals;
namespace WinrtFoundation = winrt::Windows::Foundation;
namespace WinrtBluetooth = winrt::Windows::Devices::Bluetooth;
namespace WinrtBluetoothAdv = winrt::Windows::Devices::Bluetooth::Advertisement;
namespace WinrtDevicesEnumeration = winrt::Windows::Devices::Enumeration;

std::string to_hex_string(const std::vector<uint8_t>& data) {
    if (data.empty()) return "";
    std::stringstream ss;
    ss << std::hex << std::setfill('0');
    for (const auto& byte : data) {
        ss << std::setw(2) << static_cast<int>(byte);
    }
    return ss.str();
}

struct AirPodsData {
    std::string model;
    std::string model_id;
    int left_battery;
    int right_battery;
    int case_battery;
    bool left_charging;
    bool right_charging;
    bool case_charging;
    bool left_in_ear;
    bool right_in_ear;
    bool both_in_case;
    bool lid_open;
    std::string broadcasting_ear;
};

std::optional<AirPodsData> parse_airpods_data(const std::vector<uint8_t>& data) {
    if (data.size() < 8) return std::nullopt;
    
    // The manufacturer data from WinRT does NOT include the company ID (0x4C 0x00)
    // It starts directly with the protocol type
    if (data[0] != 0x07) return std::nullopt;
    
    AirPodsData airpods;
    
    // Adjust indices since we removed the 0x4C 0x00 prefix (shifted by -2)
    uint16_t model_id = (data[4] << 8) | data[3];
    switch (model_id) {
        case 0x2014: airpods.model = "AirPods Pro 2"; break;
        case 0x200E: airpods.model = "AirPods Pro"; break;
        case 0x2013: airpods.model = "AirPods 3"; break;
        case 0x200F: airpods.model = "AirPods 2"; break;
        default: airpods.model = "Unknown AirPods"; break;
    }
    
    char model_hex[8];
    sprintf_s(model_hex, "0x%04X", model_id);
    airpods.model_id = model_hex;
    
    uint8_t status = data[5];
    uint8_t battery_data = data[6];
    uint8_t lid_data = data[7];
    
    airpods.case_battery = ((status & 0xF0) >> 4) * 10;
    airpods.left_battery = ((battery_data & 0xF0) >> 4) * 10;
    airpods.right_battery = (battery_data & 0x0F) * 10;
    
    airpods.case_charging = (status & 0x04) != 0;
    airpods.left_charging = (status & 0x02) != 0;
    airpods.right_charging = (status & 0x01) != 0;
    
    airpods.lid_open = (lid_data & 0x04) != 0;
    airpods.left_in_ear = (lid_data & 0x02) != 0;
    airpods.right_in_ear = (lid_data & 0x01) != 0;
    airpods.both_in_case = !airpods.left_in_ear && !airpods.right_in_ear;
    airpods.broadcasting_ear = "right";
    
    return airpods;
}

struct BLEDevice {
    std::string device_id;
    uint64_t address;
    int rssi;
    std::vector<uint8_t> manufacturer_data;
    std::optional<AirPodsData> airpods_data;
    WinrtFoundation::DateTime timestamp;
};

struct ReceivedData {
    int32_t rssi;
    WinrtFoundation::DateTime timestamp;
    uint64_t address;
    std::unordered_map<uint16_t, std::vector<uint8_t>> manufacturerDataMap;
};

class AdvertisementWatcher {
public:
    using Timestamp = WinrtFoundation::DateTime;

    explicit AdvertisementWatcher() {
        _bleWatcher.Received(std::bind(&AdvertisementWatcher::OnReceived, this, std::placeholders::_2));
        _bleWatcher.Stopped(std::bind(&AdvertisementWatcher::OnStopped, this, std::placeholders::_2));
    }

    ~AdvertisementWatcher() {
        if (!_stop) {
            _destroy = true;
            Stop();
            std::unique_lock<std::mutex> lock{_conVarMutex};
            _destroyConVar.wait_for(lock, 1s);
        }
    }

    bool Start() {
        try {
            _stop = false;
            _lastStartTime = std::chrono::steady_clock::now();

            std::lock_guard<std::mutex> lock{_mutex};
            _bleWatcher.Start();
            std::cerr << "[INFO] Bluetooth AdvWatcher start succeeded." << std::endl;
            return true;
        }
        catch (const std::exception &ex) {
            std::cerr << "[ERROR] Start adv watcher exception: " << ex.what() << std::endl;
            return false;
        }
    }

    bool Stop() {
        try {
            _stop = true;
            _stopConVar.notify_all();

            std::lock_guard<std::mutex> lock{_mutex};
            _bleWatcher.Stop();
            std::cerr << "[INFO] Bluetooth AdvWatcher stop succeeded." << std::endl;
            return true;
        }
        catch (const std::exception &ex) {
            std::cerr << "[ERROR] Stop adv watcher exception: " << ex.what() << std::endl;
            return false;
        }
    }

    const std::vector<BLEDevice>& GetDevices() const {
        return _devices;
    }

private:
    static constexpr inline auto kRetryInterval = 3s;

    WinrtBluetoothAdv::BluetoothLEAdvertisementWatcher _bleWatcher;
    std::mutex _mutex;
    std::vector<BLEDevice> _devices;

    std::atomic<bool> _stop{false}, _destroy{false};
    std::atomic<std::chrono::steady_clock::time_point> _lastStartTime;
    std::mutex _conVarMutex;
    std::condition_variable _stopConVar, _destroyConVar;

    void OnReceived(const WinrtBluetoothAdv::BluetoothLEAdvertisementReceivedEventArgs &args) {
        ReceivedData receivedData;

        receivedData.rssi = args.RawSignalStrengthInDBm();
        receivedData.timestamp = args.Timestamp();
        receivedData.address = args.BluetoothAddress();

        const auto &manufacturerDataArray = args.Advertisement().ManufacturerData();
        for (uint32_t i = 0; i < manufacturerDataArray.Size(); ++i) {
            const auto &manufacturerData = manufacturerDataArray.GetAt(i);
            const auto companyId = manufacturerData.CompanyId();
            const auto &data = manufacturerData.Data();

            std::vector<uint8_t> stdData(data.data(), data.data() + data.Length());
            receivedData.manufacturerDataMap.try_emplace(companyId, std::move(stdData));
        }

        for (const auto& [companyId, data] : receivedData.manufacturerDataMap) {
            if (companyId == 76) {
                BLEDevice device;
                device.address = receivedData.address;
                device.rssi = receivedData.rssi;
                device.timestamp = receivedData.timestamp;
                device.manufacturer_data = data;
                
                char addr_str[32];
                sprintf_s(addr_str, "%012llx", device.address);
                device.device_id = addr_str;

                device.airpods_data = parse_airpods_data(device.manufacturer_data);
                
                if (device.airpods_data.has_value()) {
                    std::cerr << "[INFO] AirPods detected: " << device.airpods_data->model 
                              << " - Left:" << device.airpods_data->left_battery 
                              << "% Right:" << device.airpods_data->right_battery 
                              << "% Case:" << device.airpods_data->case_battery << "%" << std::endl;
                } else {
                    std::cerr << "[INFO] Apple device detected: " << to_hex_string(device.manufacturer_data) << std::endl;
                }
                
                std::lock_guard<std::mutex> lock{_mutex};
                _devices.push_back(device);
            }
        }
    }

    void OnStopped(const WinrtBluetoothAdv::BluetoothLEAdvertisementWatcherStoppedEventArgs &args) {
        std::unique_lock<std::mutex> lock{_mutex};
        auto status = _bleWatcher.Status();
        lock.unlock();

        std::cerr << "[INFO] BLE advertisement scan stopped." << std::endl;

        if (!_destroy) {
            do {
                std::unique_lock<std::mutex> lock{_conVarMutex};
                _stopConVar.wait_until(lock, _lastStartTime.load() + kRetryInterval);
            } while (!_stop && !Start());
        }
        else {
            _destroyConVar.notify_all();
        }
    }
};

void output_json(const std::vector<BLEDevice>& devices) {
    auto timestamp = std::time(nullptr);
    
    std::cout << "{" << std::endl;
    std::cout << "    \"scanner_version\": \"5.0\"," << std::endl;
    std::cout << "    \"scan_timestamp\": \"" << timestamp << "\"," << std::endl;
    std::cout << "    \"total_devices\": " << devices.size() << "," << std::endl;
    std::cout << "    \"devices\": [" << std::endl;
    
    int airpods_count = 0;
    bool first = true;
    
    for (const auto& device : devices) {
        if (!first) std::cout << "," << std::endl;
        first = false;
        
        std::cout << "        {" << std::endl;
        std::cout << "            \"device_id\": \"" << device.device_id << "\"," << std::endl;
        std::cout << "            \"address\": \"" << device.address << "\"," << std::endl;
        std::cout << "            \"rssi\": " << device.rssi << "," << std::endl;
        std::cout << "            \"manufacturer_data_hex\": \"" << to_hex_string(device.manufacturer_data) << "\"," << std::endl;
        
        if (device.airpods_data.has_value()) {
            airpods_count++;
            const auto& airpods = device.airpods_data.value();
            
            std::cout << "            \"airpods_data\": {" << std::endl;
            std::cout << "                \"model\": \"" << airpods.model << "\"," << std::endl;
            std::cout << "                \"model_id\": \"" << airpods.model_id << "\"," << std::endl;
            std::cout << "                \"left_battery\": " << airpods.left_battery << "," << std::endl;
            std::cout << "                \"right_battery\": " << airpods.right_battery << "," << std::endl;
            std::cout << "                \"case_battery\": " << airpods.case_battery << "," << std::endl;
            std::cout << "                \"left_charging\": " << (airpods.left_charging ? "true" : "false") << "," << std::endl;
            std::cout << "                \"right_charging\": " << (airpods.right_charging ? "true" : "false") << "," << std::endl;
            std::cout << "                \"case_charging\": " << (airpods.case_charging ? "true" : "false") << "," << std::endl;
            std::cout << "                \"left_in_ear\": " << (airpods.left_in_ear ? "true" : "false") << "," << std::endl;
            std::cout << "                \"right_in_ear\": " << (airpods.right_in_ear ? "true" : "false") << "," << std::endl;
            std::cout << "                \"both_in_case\": " << (airpods.both_in_case ? "true" : "false") << "," << std::endl;
            std::cout << "                \"lid_open\": " << (airpods.lid_open ? "true" : "false") << "," << std::endl;
            std::cout << "                \"broadcasting_ear\": \"" << airpods.broadcasting_ear << "\"" << std::endl;
            std::cout << "            }" << std::endl;
        } else {
            std::cout << "            \"airpods_data\": null" << std::endl;
        }
        
        std::cout << "        }";
    }
    
    std::cout << std::endl << "    ]," << std::endl;
    std::cout << "    \"airpods_count\": " << airpods_count << "," << std::endl;
    std::cout << "    \"status\": \"success\"," << std::endl;
    std::cout << "    \"note\": \"Standalone AirPods Battery CLI v5.0 - Real BLE advertisement capture\"" << std::endl;
    std::cout << "}" << std::endl;
}

int main() {
    try {
        std::cerr << "AirPods Battery CLI v5.0 - Standalone Battery Monitor" << std::endl;
        
        AdvertisementWatcher watcher;
        
        if (watcher.Start()) {
            std::cerr << "[INFO] Scanning for 10 seconds..." << std::endl;
            std::this_thread::sleep_for(std::chrono::seconds(10));
            
            watcher.Stop();
            
            // Give a small delay to ensure the stop event is fully processed
            std::this_thread::sleep_for(std::chrono::milliseconds(100));
            
            const auto& devices = watcher.GetDevices();
            output_json(devices);
        } else {
            std::cout << "{\"scanner_version\":\"5.0\",\"status\":\"error\",\"error\":\"Failed to start BLE scan\",\"total_devices\":0,\"devices\":[],\"airpods_count\":0}" << std::endl;
            return 1;
        }
        
        return 0;
    }
    catch (const std::exception& e) {
        std::cout << "{\"scanner_version\":\"5.0\",\"status\":\"error\",\"error\":\"" << e.what() << "\",\"total_devices\":0,\"devices\":[],\"airpods_count\":0}" << std::endl;
        return 1;
    }
}
