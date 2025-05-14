use rustpods::airpods::AirPodsType;
use rustpods::error::AirPodsError;

fn main() {
    println!("Testing AirPods type identification");
    
    // Import locally for testing
    use rustpods::airpods::detector::identify_airpods_type;
    
    // Test with String type for various AirPods models
    let result1 = identify_airpods_type(&Some("AirPods 1".to_string()), &[0x07, 0x19, 0x01]);
    println!("AirPods 1 identified as: {:?}", result1);
    assert!(matches!(result1, Ok(AirPodsType::AirPods1)));
    
    let result2 = identify_airpods_type(&Some("AirPods Pro".to_string()), &[0x0E, 0x19, 0x01]);
    println!("AirPods Pro identified as: {:?}", result2);
    assert_eq!(result2.unwrap(), AirPodsType::AirPodsPro);
    
    let result3 = identify_airpods_type(&Some("AirPods Pro 2".to_string()), &[0x0F, 0x19, 0x01]);
    println!("AirPods Pro 2 identified as: {:?}", result3);
    assert_eq!(result3.unwrap(), AirPodsType::AirPodsPro2);
    
    let result4 = identify_airpods_type(&Some("AirPods 3".to_string()), &[0x13, 0x19, 0x01]);
    println!("AirPods 3 identified as: {:?}", result4);
    assert_eq!(result4.unwrap(), AirPodsType::AirPods3);
    
    let result5 = identify_airpods_type(&Some("AirPods Max".to_string()), &[0x0A, 0x19, 0x01]);
    println!("AirPods Max identified as: {:?}", result5);
    assert_eq!(result5.unwrap(), AirPodsType::AirPodsMax);
    
    // Test with unknown pattern
    let result6 = identify_airpods_type(&Some("Unknown".to_string()), &[0xFF, 0xFF, 0x01]);
    println!("Unknown pattern identified as: {:?}", result6);
    assert_eq!(result6.unwrap(), AirPodsType::Unknown);
    
    // Test with empty data
    let result7 = identify_airpods_type(&Some("".to_string()), &[]);
    println!("Empty data identified as: {:?}", result7);
    assert_eq!(result7.unwrap(), AirPodsType::Unknown);
    
    // Test with None name
    let result8 = identify_airpods_type(&None, &[0x07, 0x19, 0x01]);
    println!("None name with AirPods 1 data identified as: {:?}", result8);
    assert_eq!(result8.unwrap(), AirPodsType::AirPods1);
    
    println!("All tests passed!");
} 