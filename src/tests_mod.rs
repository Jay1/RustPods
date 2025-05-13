#[cfg(test)]
mod tests {
    use crate::airpods::detector::*;
    use crate::bluetooth::{scanner::*, ScanConfig};
    use crate::AirPodsType;

    #[test]
    fn test_identify_airpods_type_standalone() {
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