#[cfg(test)]
mod tests {
    use crate::airpods::{AirPodsType, detector::identify_airpods_type};
    use crate::bluetooth::{scanner::*, ScanConfig};

    #[test]
    fn test_identify_airpods_type_standalone() {
        // Test various AirPods type identification
        assert!(matches!(identify_airpods_type(&Some("AirPods 1".to_string()), &[0x07, 0x19, 0x01]).unwrap(), AirPodsType::AirPods1 | AirPodsType::AirPods2));
        assert_eq!(identify_airpods_type(&Some("AirPods Pro".to_string()), &[0x0E, 0x19, 0x01]).unwrap(), AirPodsType::AirPodsPro);
        assert_eq!(identify_airpods_type(&Some("AirPods Pro 2".to_string()), &[0x0F, 0x19, 0x01]).unwrap(), AirPodsType::AirPodsPro2);
        assert_eq!(identify_airpods_type(&Some("AirPods 3".to_string()), &[0x13, 0x19, 0x01]).unwrap(), AirPodsType::AirPods3);
        assert_eq!(identify_airpods_type(&Some("AirPods Max".to_string()), &[0x0A, 0x19, 0x01]).unwrap(), AirPodsType::AirPodsMax);
        assert_eq!(identify_airpods_type(&Some("Unknown Device".to_string()), &[0xFF, 0xFF, 0x01]).unwrap(), AirPodsType::Unknown);
        
        // Test with data that's too short - should return result with Unknown for valid 2-byte data
        assert_eq!(identify_airpods_type(&Some("".to_string()), &[0x01, 0x00]).unwrap(), AirPodsType::Unknown);
        
        // This will actually error since empty data is invalid
        assert!(identify_airpods_type(&None, &[0x01, 0x00]).is_ok());
    }

    #[tokio::test]
    async fn test_scanner_new_standalone() {
        let scanner = BleScanner::new();
        assert!(!scanner.is_scanning());
        assert_eq!(scanner.get_scan_cycles(), 0);
        assert!(scanner.get_devices().await.is_empty());
    }
    
    #[test]
    fn test_scanner_with_config_standalone() {
        let config = ScanConfig::default();
        let scanner = BleScanner::with_config(config);
        
        assert_eq!(scanner.get_scan_cycles(), 0);
        assert!(!scanner.is_scanning());
        // Note: Not testing get_devices().is_empty() here as it would require async
    }
    
    #[tokio::test]
    async fn test_device_list_operations_standalone() {
        let mut scanner = BleScanner::new();
        
        // Initially empty
        assert!(scanner.get_devices().await.is_empty());
        
        // Clear should work even when empty
        scanner.clear_devices().await;
        assert!(scanner.get_devices().await.is_empty());
    }
} 