use rustpods::airpods::detector::*;
use rustpods::AirPodsType;

#[test]
fn test_identify_airpods_type() {
    // Test various AirPods type identification
    assert!(matches!(identify_airpods_type(&Some("AirPods 1".to_string()), &[0x07, 0x19, 0x01]), AirPodsType::AirPods1 | AirPodsType::AirPods2));
    assert_eq!(identify_airpods_type(&Some("AirPods Pro".to_string()), &[0x0E, 0x19, 0x01]), AirPodsType::AirPodsPro);
    assert_eq!(identify_airpods_type(&Some("AirPods Pro 2".to_string()), &[0x0F, 0x19, 0x01]), AirPodsType::AirPodsPro2);
    assert_eq!(identify_airpods_type(&Some("AirPods 3".to_string()), &[0x13, 0x19, 0x01]), AirPodsType::AirPods3);
    assert_eq!(identify_airpods_type(&Some("AirPods Max".to_string()), &[0x0A, 0x19, 0x01]), AirPodsType::AirPodsMax);
    assert_eq!(identify_airpods_type(&Some("Unknown Device".to_string()), &[0xFF, 0xFF, 0x01]), AirPodsType::Unknown);
    
    // Test with data that's too short
    assert_eq!(identify_airpods_type(&Some("".to_string()), &[0x01]), AirPodsType::Unknown);
    assert_eq!(identify_airpods_type(&None, &[]), AirPodsType::Unknown);
} 