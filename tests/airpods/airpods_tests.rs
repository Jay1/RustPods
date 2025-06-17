#[cfg(test)]
mod tests {
    use crate::airpods::{AirPodsType, detector::identify_airpods_type};

    #[test]
    fn test_identify_airpods_type() {
        assert!(matches!(identify_airpods_type(&Some("AirPods 1".to_string()), &[0x07, 0x19, 0x01]).unwrap(), AirPodsType::AirPods1 | AirPodsType::AirPods2));
        assert_eq!(identify_airpods_type(&Some("AirPods Pro".to_string()), &[0x0E, 0x19, 0x01]).unwrap(), AirPodsType::AirPodsPro);
        assert_eq!(identify_airpods_type(&Some("AirPods Pro 2".to_string()), &[0x0F, 0x19, 0x01]).unwrap(), AirPodsType::AirPodsPro2);
        assert_eq!(identify_airpods_type(&Some("AirPods 3".to_string()), &[0x13, 0x19, 0x01]).unwrap(), AirPodsType::AirPods3);
        assert_eq!(identify_airpods_type(&Some("AirPods Max".to_string()), &[0x0A, 0x19, 0x01]).unwrap(), AirPodsType::AirPodsMax);
        assert_eq!(identify_airpods_type(&Some("Unknown Device".to_string()), &[0xFF, 0xFF, 0x01]).unwrap(), AirPodsType::Unknown);

        // Handle edge cases
        assert_eq!(identify_airpods_type(&Some("".to_string()), &[0x01, 0x00]).unwrap(), AirPodsType::Unknown);
        assert!(identify_airpods_type(&None, &[0x01, 0x00]).is_ok());
    }
} 
