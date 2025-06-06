#pragma once

#include <vector>
#include <optional>
#include <cstdint>
#include <string>

/**
 * @brief Template interface for protocol parsers
 * 
 * This interface provides a generic abstraction for parsing various types of
 * manufacturer data from BLE advertisements. The template parameter T represents
 * the type of data structure that the parser produces.
 * 
 * @tparam T The data type produced by the parser (e.g., AirPodsData)
 */
template<typename T>
class IProtocolParser {
public:
    virtual ~IProtocolParser() = default;

    /**
     * @brief Parse manufacturer data into structured information
     * @param data Raw manufacturer data bytes
     * @return Parsed data structure if successful, nullopt if parsing fails
     */
    virtual std::optional<T> Parse(const std::vector<uint8_t>& data) = 0;

    /**
     * @brief Check if the parser can handle the given data
     * @param data Raw manufacturer data bytes
     * @return true if this parser can handle the data format
     */
    virtual bool CanParse(const std::vector<uint8_t>& data) const = 0;

    /**
     * @brief Get the name of this parser
     * @return Human-readable parser name
     */
    virtual std::string GetParserName() const = 0;

    /**
     * @brief Get the version of this parser
     * @return Parser version string
     */
    virtual std::string GetParserVersion() const = 0;
}; 