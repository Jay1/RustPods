#include "BleDevice.hpp"
#include "../protocol/AirPodsData.hpp"
#include <sstream>
#include <iomanip>

BleDevice::BleDevice(
    const std::string& deviceId,
    uint64_t address,
    int rssi,
    const std::vector<uint8_t>& manufacturerData
) : deviceId(deviceId)
  , address(address)
  , rssi(rssi)
  , manufacturerData(manufacturerData)
  , timestamp(std::chrono::system_clock::now())
{
}

bool BleDevice::HasAirPodsData() const {
    return airpodsData.has_value();
}

std::string BleDevice::GetFormattedAddress() const {
    std::stringstream ss;
    ss << std::hex << std::setfill('0') << std::uppercase;
    
    // Extract bytes from 64-bit address (assuming little-endian)
    for (int i = 5; i >= 0; --i) {
        if (i < 5) ss << ":";
        ss << std::setw(2) << ((address >> (i * 8)) & 0xFF);
    }
    
    return ss.str();
}

std::string BleDevice::GetManufacturerDataHex() const {
    if (manufacturerData.empty()) {
        return "";
    }
    
    std::stringstream ss;
    ss << std::hex << std::setfill('0') << std::nouppercase;
    
    for (const auto& byte : manufacturerData) {
        ss << std::setw(2) << static_cast<int>(byte);
    }
    
    return ss.str();
}

std::chrono::duration<double> BleDevice::GetAge() const {
    return std::chrono::system_clock::now() - timestamp;
}

bool operator==(const BleDevice& lhs, const BleDevice& rhs) {
    return lhs.address == rhs.address;
}

bool operator!=(const BleDevice& lhs, const BleDevice& rhs) {
    return !(lhs == rhs);
} 