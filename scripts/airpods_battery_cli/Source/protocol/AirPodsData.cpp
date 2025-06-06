#include "AirPodsData.hpp"
#include <algorithm>
#include <sstream>

bool AirPodsData::IsAnyCharging() const {
    return chargingState.leftCharging || 
           chargingState.rightCharging || 
           chargingState.caseCharging;
}

bool AirPodsData::IsAnyInEar() const {
    return deviceState.leftInEar || deviceState.rightInEar;
}

int AirPodsData::GetLowestBatteryLevel() const {
    return std::min({
        batteryLevels.left,
        batteryLevels.right,
        batteryLevels.case_
    });
}

std::string AirPodsData::GetBatterySummary() const {
    std::stringstream ss;
    ss << "L:" << batteryLevels.left << "% "
       << "R:" << batteryLevels.right << "% "
       << "C:" << batteryLevels.case_ << "%";
    return ss.str();
} 