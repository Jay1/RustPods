#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AirPodsType {
    Unknown,
    AirPods1,
    AirPods2,
    AirPods3,
    AirPodsPro,
    AirPodsPro2,
    AirPodsMax,
    PowerbeatsPro,
    BeatsFitPro,
    BeatsStudio,
    BeatsX,
}

/// Identify the type of AirPods from the advertisement data
fn identify_airpods_type(device_name: &Option<String>, manufacturer_data: &[u8]) -> AirPodsType {
    // Check if we have the right manufacturer data format
    if manufacturer_data.len() < 3 {
        return AirPodsType::Unknown;
    }

    // AirPods identification is based on bytes in the manufacturer data
    // Different models have different identification bytes
    match (manufacturer_data[0], manufacturer_data[1], manufacturer_data[2]) {
        // AirPods 1st generation
        (0x07, 0x19, 0x01) => AirPodsType::AirPods1,
        
        // AirPods 2nd generation
        (0x0F, 0x19, 0x01) | (0x09, 0x19, 0x01) | (0x01, 0x19, 0x01) => AirPodsType::AirPods2,
        
        // AirPods 3rd generation
        (0x13, 0x19, 0x01) => AirPodsType::AirPods3,
        
        // AirPods Pro
        (0x0E, 0x19, 0x01) => AirPodsType::AirPodsPro,
        
        // AirPods Pro 2
        (0x14, 0x19, 0x01) => AirPodsType::AirPodsPro2,
        
        // AirPods Max
        (0x0A, 0x19, 0x01) => AirPodsType::AirPodsMax,
        
        // Beats Powerbeats Pro
        (0x0B, 0x19, 0x01) => AirPodsType::PowerbeatsPro,
        
        // Beats Fit Pro
        (0x11, 0x19, 0x01) => AirPodsType::BeatsFitPro,
        
        // Beats Studio Buds
        (0x10, 0x19, 0x01) => AirPodsType::BeatsStudio,
        
        // BeatsX
        (0x05, 0x19, 0x01) => AirPodsType::BeatsX,
        
        // Unknown model
        _ => {
            // Check device name if available
            if let Some(name) = device_name {
                if name.contains("AirPods Pro") {
                    return AirPodsType::AirPodsPro;
                } else if name.contains("AirPods") {
                    // Can't determine which AirPods model exactly from the name alone
                    return AirPodsType::AirPods2; // Default to most common
                }
            }
            AirPodsType::Unknown
        }
    }
}

fn main() {
    println!("Testing AirPods Type Identification");
    println!("-----------------------------------");
    
    // Test cases for different AirPods models
    let test_cases = [
        (Some("AirPods".to_string()), vec![0x07, 0x19, 0x01], AirPodsType::AirPods1, "AirPods 1"),
        (Some("AirPods".to_string()), vec![0x09, 0x19, 0x01], AirPodsType::AirPods2, "AirPods 2"),
        (Some("AirPods".to_string()), vec![0x13, 0x19, 0x01], AirPodsType::AirPods3, "AirPods 3"),
        (Some("AirPods Pro".to_string()), vec![0x0E, 0x19, 0x01], AirPodsType::AirPodsPro, "AirPods Pro"),
        (Some("AirPods Pro".to_string()), vec![0x14, 0x19, 0x01], AirPodsType::AirPodsPro2, "AirPods Pro 2"),
        (Some("AirPods Max".to_string()), vec![0x0A, 0x19, 0x01], AirPodsType::AirPodsMax, "AirPods Max"),
        (Some("Powerbeats Pro".to_string()), vec![0x0B, 0x19, 0x01], AirPodsType::PowerbeatsPro, "Powerbeats Pro"),
        (Some("Beats Fit Pro".to_string()), vec![0x11, 0x19, 0x01], AirPodsType::BeatsFitPro, "Beats Fit Pro"),
        (Some("Beats Studio Buds".to_string()), vec![0x10, 0x19, 0x01], AirPodsType::BeatsStudio, "Beats Studio"),
        (Some("BeatsX".to_string()), vec![0x05, 0x19, 0x01], AirPodsType::BeatsX, "BeatsX"),
        (None, vec![0x07, 0x19, 0x01], AirPodsType::AirPods1, "AirPods 1 (no name)"),
        (Some("Unknown Device".to_string()), vec![0xFF, 0xFF, 0xFF], AirPodsType::Unknown, "Unknown device"),
        (Some("AirPods".to_string()), vec![0xFF, 0xFF, 0xFF], AirPodsType::AirPods2, "Fallback to name matching"),
    ];
    
    let mut passed = 0;
    let total = test_cases.len();
    
    for (i, (name, data, expected, description)) in test_cases.iter().enumerate() {
        let result = identify_airpods_type(name, data);
        let success = result == *expected;
        
        if success {
            passed += 1;
            println!("✅ Test {}: {} - PASSED", i + 1, description);
        } else {
            println!("❌ Test {}: {} - FAILED", i + 1, description);
            println!("   Expected: {:?}, Got: {:?}", expected, result);
        }
    }
    
    println!("\nTest Results: {}/{} passed", passed, total);
    
    if passed == total {
        println!("All tests passed successfully!");
    } else {
        println!("Some tests failed.");
    }
} 