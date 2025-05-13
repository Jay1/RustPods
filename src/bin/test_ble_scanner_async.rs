use std::collections::HashMap;

struct ScanConfig {
    scan_duration_ms: u64,
    filter_duplicates: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            scan_duration_ms: 1000,
            filter_duplicates: true,
        }
    }
}

struct BleScanner {
    is_scanning: bool,
    scan_cycles: usize,
    devices: HashMap<String, String>,
}

impl BleScanner {
    fn new() -> Self {
        Self {
            is_scanning: false,
            scan_cycles: 0,
            devices: HashMap::new(),
        }
    }
    
    fn with_config(config: ScanConfig) -> Self {
        Self {
            is_scanning: false,
            scan_cycles: 0,
            devices: HashMap::new(),
        }
    }
    
    fn is_scanning(&self) -> bool {
        self.is_scanning
    }
    
    fn get_scan_cycles(&self) -> usize {
        self.scan_cycles
    }
    
    async fn get_devices(&self) -> HashMap<String, String> {
        self.devices.clone()
    }
    
    async fn clear_devices(&mut self) {
        self.devices.clear();
    }
}

#[tokio::main]
async fn main() {
    println!("Testing BLE Scanner:");
    
    // Test scanner_new
    let scanner = BleScanner::new();
    assert!(!scanner.is_scanning());
    assert_eq!(scanner.get_scan_cycles(), 0);
    assert!(scanner.get_devices().await.is_empty());
    println!("  ✓ New scanner initialized correctly");
    
    // Test scanner_with_config
    let config = ScanConfig::default();
    let scanner = BleScanner::with_config(config);
    assert_eq!(scanner.get_scan_cycles(), 0);
    assert!(!scanner.is_scanning());
    println!("  ✓ Scanner with config initialized correctly");
    
    // Test device_list_operations
    let mut scanner = BleScanner::new();
    assert!(scanner.get_devices().await.is_empty());
    scanner.clear_devices().await;
    assert!(scanner.get_devices().await.is_empty());
    println!("  ✓ Device list operations work correctly");
    
    println!("\nAll tests passed successfully!");
} 