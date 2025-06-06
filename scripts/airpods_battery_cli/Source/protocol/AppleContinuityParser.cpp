#include "AppleContinuityParser.hpp"
#include <sstream>
#include <iomanip>

std::optional<AirPodsData> AppleContinuityParser::Parse(const std::vector<uint8_t>& data) {
    // Validate minimum data length (exactly as in v5 scanner)
    if (data.size() < MIN_DATA_LENGTH) {
        return std::nullopt;
    }
    
    // Check protocol type - must be proximity pairing (exactly as in v5 scanner)
    // Note: The manufacturer data from WinRT does NOT include the company ID (0x4C 0x00)
    // It starts directly with the protocol type
    if (data[0] != PROXIMITY_PAIRING_TYPE) {
        return std::nullopt;
    }
    
    // Extract model ID (exactly as in v5 scanner)
    // Adjust indices since we removed the 0x4C 0x00 prefix (shifted by -2)
    uint16_t modelId = (data[4] << 8) | data[3];
    
    // Extract data bytes (exactly as in v5 scanner)
    uint8_t statusByte = data[5];
    uint8_t batteryData = data[6];
    uint8_t lidData = data[7];
    
    // Parse all components
    std::string model = ParseModelName(modelId);
    std::string modelIdStr = FormatModelId(modelId);
    BatteryLevels batteryLevels = ExtractBatteryLevels(batteryData, statusByte);
    ChargingState chargingState = ExtractChargingState(statusByte);
    DeviceState deviceState = ExtractDeviceState(lidData);
    std::string broadcastingEar = DetermineBroadcastingEar();
    
    // Create and return AirPods data
    return AirPodsData(
        model,
        modelIdStr,
        batteryLevels,
        chargingState,
        deviceState,
        broadcastingEar
    );
}

bool AppleContinuityParser::CanParse(const std::vector<uint8_t>& data) const {
    // Check minimum length and protocol type
    return data.size() >= MIN_DATA_LENGTH && data[0] == PROXIMITY_PAIRING_TYPE;
}

std::string AppleContinuityParser::GetParserName() const {
    return "Apple Continuity Protocol Parser";
}

std::string AppleContinuityParser::GetParserVersion() const {
    return "1.0 (v5 scanner compatible)";
}

std::string AppleContinuityParser::ParseModelName(uint16_t modelId) const {
    // Exact model detection logic from v5 scanner
    switch (modelId) {
        case 0x2014: return "AirPods Pro 2";
        case 0x200E: return "AirPods Pro";
        case 0x2013: return "AirPods 3";
        case 0x200F: return "AirPods 2";
        default: return "Unknown AirPods";
    }
}

std::string AppleContinuityParser::FormatModelId(uint16_t modelId) const {
    // Format as hex string (exactly as in v5 scanner)
    std::stringstream ss;
    ss << "0x" << std::hex << std::uppercase << std::setfill('0') << std::setw(4) << modelId;
    return ss.str();
}

BatteryLevels AppleContinuityParser::ExtractBatteryLevels(uint8_t batteryData, uint8_t statusByte) const {
    // Exact battery calculation logic from v5 scanner
    int caseBattery = ((statusByte & 0xF0) >> 4) * 10;
    int leftBattery = ((batteryData & 0xF0) >> 4) * 10;
    int rightBattery = (batteryData & 0x0F) * 10;
    
    return BatteryLevels(leftBattery, rightBattery, caseBattery);
}

ChargingState AppleContinuityParser::ExtractChargingState(uint8_t statusByte) const {
    // Exact charging state parsing from v5 scanner
    bool caseCharging = (statusByte & 0x04) != 0;
    bool leftCharging = (statusByte & 0x02) != 0;
    bool rightCharging = (statusByte & 0x01) != 0;
    
    return ChargingState(leftCharging, rightCharging, caseCharging);
}

DeviceState AppleContinuityParser::ExtractDeviceState(uint8_t lidData) const {
    // Exact device state parsing from v5 scanner
    bool lidOpen = (lidData & 0x04) != 0;
    bool leftInEar = (lidData & 0x02) != 0;
    bool rightInEar = (lidData & 0x01) != 0;
    bool bothInCase = !leftInEar && !rightInEar;
    
    return DeviceState(leftInEar, rightInEar, bothInCase, lidOpen);
}

std::string AppleContinuityParser::DetermineBroadcastingEar() const {
    // Default from v5 scanner - could be enhanced in future versions
    return "right";
} 