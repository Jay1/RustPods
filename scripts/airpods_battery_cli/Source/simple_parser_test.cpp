#include <iostream>
#include <vector>
#include "protocol/AppleContinuityParser.hpp"

int main() {
    std::cout << "Simple parser test starting..." << std::endl;
    
    try {
        std::cout << "Creating parser..." << std::endl;
        AppleContinuityParser parser;
        std::cout << "Parser created!" << std::endl;
        
        std::cout << "Parser name: " << parser.GetParserName() << std::endl;
        std::cout << "Parser version: " << parser.GetParserVersion() << std::endl;
        
        // Test data from real v5 scanner
        std::vector<uint8_t> data = {0x07, 0x19, 0x01, 0x14, 0x20, 0x0b, 0x88, 0x8f};
        
        std::cout << "Data size: " << data.size() << std::endl;
        std::cout << "First byte: 0x" << std::hex << static_cast<int>(data[0]) << std::dec << std::endl;
        
        std::cout << "Testing CanParse..." << std::endl;
        bool canParse = parser.CanParse(data);
        std::cout << "CanParse result: " << (canParse ? "true" : "false") << std::endl;
        
        if (canParse) {
            std::cout << "Calling Parse..." << std::endl;
            auto result = parser.Parse(data);
            
            if (result.has_value()) {
                std::cout << "✓ Parse successful!" << std::endl;
                std::cout << "Model: " << result->model << std::endl;
                return 0;
            } else {
                std::cout << "❌ Parse returned nullopt" << std::endl;
                return 1;
            }
        } else {
            std::cout << "❌ CanParse returned false" << std::endl;
            return 1;
        }
    }
    catch (const std::exception& e) {
        std::cout << "❌ Exception: " << e.what() << std::endl;
        return 1;
    }
} 