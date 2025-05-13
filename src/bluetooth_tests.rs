use crate::bluetooth::scanner::*;
use crate::bluetooth::ScanConfig;

#[tokio::test]
async fn test_scanner_new() {
    let scanner = BleScanner::new();
    assert!(!scanner.is_scanning());
    assert_eq!(scanner.get_scan_cycles(), 0);
    assert!(scanner.get_devices().await.is_empty());
}

#[test]
fn test_scanner_with_config() {
    let config = ScanConfig::default();
    let scanner = BleScanner::with_config(config);
    
    assert_eq!(scanner.get_scan_cycles(), 0);
    assert!(!scanner.is_scanning());
    // Note: Not testing get_devices().is_empty() here as it would require async
}

#[tokio::test]
async fn test_device_list_operations() {
    let mut scanner = BleScanner::new();
    
    // Initially empty
    assert!(scanner.get_devices().await.is_empty());
    
    // Clear should work even when empty
    scanner.clear_devices().await;
    assert!(scanner.get_devices().await.is_empty());
} 