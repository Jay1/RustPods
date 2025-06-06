#pragma once

#include <string>

/**
 * @brief Battery levels for AirPods components
 * 
 * Represents battery levels as percentages (0-100) for left earbud,
 * right earbud, and charging case.
 */
struct BatteryLevels {
    /// Left earbud battery percentage (0-100)
    int left;
    
    /// Right earbud battery percentage (0-100)
    int right;
    
    /// Charging case battery percentage (0-100)
    int case_;

    /**
     * @brief Default constructor
     * Initializes all battery levels to 0
     */
    BatteryLevels() : left(0), right(0), case_(0) {}

    /**
     * @brief Constructor with values
     * @param left Left earbud battery percentage
     * @param right Right earbud battery percentage
     * @param case_ Charging case battery percentage
     */
    BatteryLevels(int left, int right, int case_) 
        : left(left), right(right), case_(case_) {}
};

/**
 * @brief Charging state for AirPods components
 * 
 * Indicates whether each component is currently charging.
 */
struct ChargingState {
    /// True if left earbud is charging
    bool leftCharging;
    
    /// True if right earbud is charging
    bool rightCharging;
    
    /// True if charging case is charging
    bool caseCharging;

    /**
     * @brief Default constructor
     * Initializes all charging states to false
     */
    ChargingState() : leftCharging(false), rightCharging(false), caseCharging(false) {}

    /**
     * @brief Constructor with values
     * @param leftCharging Left earbud charging state
     * @param rightCharging Right earbud charging state
     * @param caseCharging Case charging state
     */
    ChargingState(bool leftCharging, bool rightCharging, bool caseCharging)
        : leftCharging(leftCharging), rightCharging(rightCharging), caseCharging(caseCharging) {}
};

/**
 * @brief Device state for AirPods components
 * 
 * Indicates the physical state of the earbuds and case.
 */
struct DeviceState {
    /// True if left earbud is in ear
    bool leftInEar;
    
    /// True if right earbud is in ear
    bool rightInEar;
    
    /// True if both earbuds are in the case
    bool bothInCase;
    
    /// True if the charging case lid is open
    bool lidOpen;

    /**
     * @brief Default constructor
     * Initializes all states to false
     */
    DeviceState() : leftInEar(false), rightInEar(false), bothInCase(false), lidOpen(false) {}

    /**
     * @brief Constructor with values
     * @param leftInEar Left earbud in ear state
     * @param rightInEar Right earbud in ear state
     * @param bothInCase Both earbuds in case state
     * @param lidOpen Case lid open state
     */
    DeviceState(bool leftInEar, bool rightInEar, bool bothInCase, bool lidOpen)
        : leftInEar(leftInEar), rightInEar(rightInEar), bothInCase(bothInCase), lidOpen(lidOpen) {}
};

/**
 * @brief Complete AirPods device information
 * 
 * This structure contains all parsed information from Apple Continuity Protocol
 * advertisements, including model identification, battery levels, charging states,
 * and device positioning information.
 */
struct AirPodsData {
    /// Human-readable model name (e.g., "AirPods Pro 2")
    std::string model;
    
    /// Model identifier as hex string (e.g., "0x2014")
    std::string modelId;
    
    /// Battery levels for all components
    BatteryLevels batteryLevels;
    
    /// Charging state for all components
    ChargingState chargingState;
    
    /// Device state information
    DeviceState deviceState;
    
    /// Which earbud is currently broadcasting ("left" or "right")
    std::string broadcastingEar;

    /**
     * @brief Default constructor
     */
    AirPodsData() = default;

    /**
     * @brief Constructor with all parameters
     * @param model Model name
     * @param modelId Model identifier
     * @param batteryLevels Battery information
     * @param chargingState Charging information
     * @param deviceState Device state information
     * @param broadcastingEar Broadcasting earbud
     */
    AirPodsData(
        const std::string& model,
        const std::string& modelId,
        const BatteryLevels& batteryLevels,
        const ChargingState& chargingState,
        const DeviceState& deviceState,
        const std::string& broadcastingEar
    ) : model(model)
      , modelId(modelId)
      , batteryLevels(batteryLevels)
      , chargingState(chargingState)
      , deviceState(deviceState)
      , broadcastingEar(broadcastingEar)
    {}

    /**
     * @brief Check if any component is charging
     * @return true if any component is currently charging
     */
    bool IsAnyCharging() const;

    /**
     * @brief Check if any earbud is in ear
     * @return true if either earbud is in ear
     */
    bool IsAnyInEar() const;

    /**
     * @brief Get the lowest battery level across all components
     * @return Lowest battery percentage
     */
    int GetLowestBatteryLevel() const;

    /**
     * @brief Get a summary string of battery levels
     * @return String like "L:70% R:80% C:50%"
     */
    std::string GetBatterySummary() const;
}; 