use rustpods::airpods::detector::*;
use rustpods::AirPodsType;
use rustpods::error::{AirPodsError, Result};

#[test]
fn test_identify_airpods_type() {
    // Test various AirPods type identification
    assert!(matches!(identify_airpods_type(&Some("AirPods 1".to_string()), &[0x07, 0x19, 0x01]), Ok(AirPodsType::AirPods1) | Ok(AirPodsType::AirPods2)));
    assert!(matches!(identify_airpods_type(&Some("AirPods Pro".to_string()), &[0x0E, 0x19, 0x01]), Ok(AirPodsType::AirPodsPro)));
    assert!(matches!(identify_airpods_type(&Some("AirPods Pro 2".to_string()), &[0x0F, 0x19, 0x01]), Ok(AirPodsType::AirPodsPro2)));
    assert!(matches!(identify_airpods_type(&Some("AirPods 3".to_string()), &[0x13, 0x19, 0x01]), Ok(AirPodsType::AirPods3)));
    assert!(matches!(identify_airpods_type(&Some("AirPods Max".to_string()), &[0x0A, 0x19, 0x01]), Ok(AirPodsType::AirPodsMax)));
    assert!(matches!(identify_airpods_type(&Some("Unknown Device".to_string()), &[0xFF, 0xFF, 0x01]), Ok(AirPodsType::Unknown)));
    
    // Test with data that's too short - should return an error
    assert!(matches!(identify_airpods_type(&Some("".to_string()), &[0x01]), Err(AirPodsError::InvalidData(_))));
    assert!(matches!(identify_airpods_type(&None, &[]), Err(AirPodsError::InvalidData(_))));
} 