#pragma once

#include "IProtocolParser.hpp"
#include "AirPodsData.hpp"
#include <string>

/**
 * @brief Parser for Apple Continuity Protocol advertisements
 * 
 * This parser implements the Apple Continuity Protocol for AirPods and Beats devices.
 * It preserves the exact parsing logic from the v5 scanner to ensure compatibility
 * and maintain the proven functionality for battery level extraction and device
 * state determination.
 */
class AppleContinuityParser : public IProtocolParser<AirPodsData> {
public:
    /**
     * @brief Constructor
     */
    AppleContinuityParser() = default;

    /**
     * @brief Destructor
     */
    ~AppleContinuityParser() override = default;

    // IProtocolParser interface implementation
    std::optional<AirPodsData> Parse(const std::vector<uint8_t>& data) override;
    bool CanParse(const std::vector<uint8_t>& data) const override;
    std::string GetParserName() const override;
    std::string GetParserVersion() const override;

private:
    /// Protocol type identifier for proximity pairing (from v5 scanner)
    static constexpr uint8_t PROXIMITY_PAIRING_TYPE = 0x07;
    
    /// Minimum data length required for valid AirPods advertisement
    static constexpr size_t MIN_DATA_LENGTH = 8;

    /**
     * @brief Parse model ID and return human-readable model name
     * @param modelId 16-bit model identifier
     * @return Human-readable model name
     * 
     * This method preserves the exact model detection logic from the v5 scanner.
     */
    std::string ParseModelName(uint16_t modelId) const;

    /**
     * @brief Format model ID as hex string
     * @param modelId 16-bit model identifier
     * @return Hex string representation (e.g., "0x2014")
     */
    std::string FormatModelId(uint16_t modelId) const;

    /**
     * @brief Extract battery levels from battery data byte
     * @param batteryData Raw battery data byte
     * @param statusByte Status byte containing case battery info
     * @return Battery levels structure
     * 
     * This method preserves the exact battery calculation logic from the v5 scanner:
     * - Battery levels are stored as 4-bit nibbles
     * - Each nibble represents battery level in 10% increments (0-10 scale)
     * - Final values are multiplied by 10 to get percentages (0-100)
     */
    BatteryLevels ExtractBatteryLevels(uint8_t batteryData, uint8_t statusByte) const;

    /**
     * @brief Extract charging states from status byte
     * @param statusByte Raw status byte
     * @return Charging state structure
     * 
     * This method preserves the exact charging state parsing from the v5 scanner:
     * - Bit 2 (0x04): Case charging
     * - Bit 1 (0x02): Left earbud charging
     * - Bit 0 (0x01): Right earbud charging
     */
    ChargingState ExtractChargingState(uint8_t statusByte) const;

    /**
     * @brief Extract device states from lid data byte
     * @param lidData Raw lid data byte
     * @return Device state structure
     * 
     * This method preserves the exact device state parsing from the v5 scanner:
     * - Bit 2 (0x04): Lid open
     * - Bit 1 (0x02): Left earbud in ear
     * - Bit 0 (0x01): Right earbud in ear
     * - Both in case is calculated as !leftInEar && !rightInEar
     */
    DeviceState ExtractDeviceState(uint8_t lidData) const;

    /**
     * @brief Determine which earbud is broadcasting
     * @return Broadcasting earbud identifier
     * 
     * This method preserves the v5 scanner default of "right" for compatibility.
     * In future versions, this could be enhanced to detect the actual broadcasting ear.
     */
    std::string DetermineBroadcastingEar() const;
}; 