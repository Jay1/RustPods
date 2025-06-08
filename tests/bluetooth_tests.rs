use rustpods::bluetooth::{scanner::*, ScanConfig};

#[tokio::test]
async fn test_scanner_new() {
    let scanner = BleScanner::with_config(ScanConfig::default());
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
    let mut scanner = BleScanner::with_config(ScanConfig::default());

    // Initially empty
    assert!(scanner.get_devices().await.is_empty());

    // Clear should work even when empty
    scanner.clear_devices().await;
    assert!(scanner.get_devices().await.is_empty());
}

#[test]
fn test_parse_bdaddr() {
    let addr_str = "12:34:56:78:9A:BC";
    let result = parse_bdaddr(addr_str);
    assert!(result.is_ok());

    let addr = result.unwrap();
    assert_eq!(
        addr,
        btleplug::api::BDAddr::from([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC])
    );

    // Test invalid address format
    let result = parse_bdaddr("12:34:56:78:9A");
    assert!(result.is_err());

    // Test invalid hex
    let result = parse_bdaddr("12:34:56:78:9A:ZZ");
    assert!(result.is_err());
}
