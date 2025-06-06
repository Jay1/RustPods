#include <iostream>
#include <fstream>
#include <vector>
#include "protocol/AppleContinuityParser.hpp"

int main() {
    std::ofstream logFile("modular_test_output.log");
    
    if (!logFile.is_open()) {
        return 1;
    }
    
    logFile << "=== Modular Parser Test ===" << std::endl;
    
    try {
        logFile << "Creating AppleContinuityParser..." << std::endl;
        AppleContinuityParser parser;
        
        logFile << "Parser created successfully!" << std::endl;
        logFile << "Parser name: " << parser.GetParserName() << std::endl;
        logFile << "Parser version: " << parser.GetParserVersion() << std::endl;
        
        // Test data from real v5 scanner (AirPods Pro 2)
        std::vector<uint8_t> data = {0x07, 0x19, 0x01, 0x14, 0x20, 0x0b, 0x77, 0x8f};
        
        logFile << "Testing with real AirPods data..." << std::endl;
        logFile << "Data size: " << data.size() << " bytes" << std::endl;
        
        logFile << "Testing CanParse..." << std::endl;
        bool canParse = parser.CanParse(data);
        logFile << "CanParse result: " << (canParse ? "true" : "false") << std::endl;
        
        if (canParse) {
            logFile << "Calling Parse method..." << std::endl;
            auto result = parser.Parse(data);
            
            if (result.has_value()) {
                logFile << "✓ Parse successful!" << std::endl;
                const auto& airpods = result.value();
                
                logFile << "Model: " << airpods.model << std::endl;
                logFile << "Model ID: " << airpods.modelId << std::endl;
                logFile << "Battery Summary: " << airpods.GetBatterySummary() << std::endl;
                
                logFile << "Detailed Battery Info:" << std::endl;
                logFile << "  Left: " << airpods.batteryLevels.left << "%" << std::endl;
                logFile << "  Right: " << airpods.batteryLevels.right << "%" << std::endl;
                logFile << "  Case: " << airpods.batteryLevels.case_ << "%" << std::endl;
                
                logFile << "Charging State:" << std::endl;
                logFile << "  Left charging: " << (airpods.chargingState.leftCharging ? "true" : "false") << std::endl;
                logFile << "  Right charging: " << (airpods.chargingState.rightCharging ? "true" : "false") << std::endl;
                logFile << "  Case charging: " << (airpods.chargingState.caseCharging ? "true" : "false") << std::endl;
                
                logFile << "Device State:" << std::endl;
                logFile << "  Left in ear: " << (airpods.deviceState.leftInEar ? "true" : "false") << std::endl;
                logFile << "  Right in ear: " << (airpods.deviceState.rightInEar ? "true" : "false") << std::endl;
                logFile << "  Both in case: " << (airpods.deviceState.bothInCase ? "true" : "false") << std::endl;
                logFile << "  Lid open: " << (airpods.deviceState.lidOpen ? "true" : "false") << std::endl;
                
                logFile << "Broadcasting ear: " << airpods.broadcastingEar << std::endl;
                
                // Validate against expected v5 scanner results
                if (airpods.model == "AirPods Pro 2" && 
                    airpods.batteryLevels.left == 70 && 
                    airpods.batteryLevels.right == 70 &&
                    airpods.batteryLevels.case_ == 0) {
                    logFile << "🎉 SUCCESS! Modular parser matches v5 scanner exactly!" << std::endl;
                } else {
                    logFile << "❌ MISMATCH! Results don't match v5 scanner" << std::endl;
                }
                
            } else {
                logFile << "❌ Parse returned nullopt" << std::endl;
            }
        } else {
            logFile << "❌ CanParse returned false" << std::endl;
        }
        
    } catch (const std::exception& e) {
        logFile << "❌ Exception caught: " << e.what() << std::endl;
    }
    
    logFile << "Modular parser test completed." << std::endl;
    logFile.close();
    
    // Also try console output
    std::cout << "Modular parser test completed - check modular_test_output.log" << std::endl;
    
    return 0;
} 