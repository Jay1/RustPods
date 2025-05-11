//! Tests for Bluetooth adapter edge cases and recovery

use std::time::Duration;
use tokio::time::sleep;
use btleplug::api::BDAddr;
use rustpods::bluetooth::{
    AdapterManager, BleScanner, BleEvent, ScanConfig, 
    receiver_to_stream, DiscoveredDevice
};
use futures::{StreamExt, pin_mut};

/// Helper to determine if we should skip Bluetooth tests
fn skip_bluetooth_test() -> bool {
    std::env::var("CI").is_ok() || std::env::var("SKIP_BT_TESTS").is_ok()
}

#[tokio::test]
async fn test_adapter_manager_no_adapters() {
    // Skip in CI or if user wants to skip Bluetooth tests
    if skip_bluetooth_test() {
        return;
    }
    
    // Create a mock adapter manager (this would ideally use a proper mock in a real test)
    // Here we're just testing the real adapter manager's fallback behavior
    let adapter_manager_result = AdapterManager::new().await;
    
    match adapter_manager_result {
        Ok(adapter_manager) => {
            let adapters = adapter_manager.get_available_adapters().clone();
            
            // Check the fallback behavior - this will differ on systems with/without Bluetooth
            if adapters.is_empty() {
                println!("No adapters found (expected on systems without Bluetooth)");
            } else {
                println!("Found {} adapters", adapters.len());
                
                // Try to select the first adapter
                if let Some(first_adapter) = adapters.first() {
                    // Create a new instance for select_adapter call to avoid borrow conflicts
                    let adapter_manager_result2 = AdapterManager::new().await;
                    if let Ok(mut manager_with_adapter) = adapter_manager_result2 {
                        let result = manager_with_adapter.select_adapter(first_adapter.index);
                        assert!(result.is_ok(), "Should be able to select first adapter");
                    }
                }
            }
        },
        Err(e) => {
            println!("AdapterManager creation failed (expected without Bluetooth): {:?}", e);
            // This is also an acceptable outcome on systems without Bluetooth
        }
    }
}

#[tokio::test]
async fn test_scanner_recovery_after_failure() {
    // Skip in CI or if user wants to skip Bluetooth tests
    if skip_bluetooth_test() {
        return;
    }
    
    // Create a scanner with a very short scan duration
    let config = ScanConfig::one_time_scan(Duration::from_millis(500))
        .with_auto_stop(true);
    let mut scanner = BleScanner::with_config(config);
    
    // Skip if we can't initialize the scanner
    if let Err(e) = scanner.initialize().await {
        println!("Scanner initialization failed (expected in CI): {:?}", e);
        return;
    }
    
    // First scan
    let scan_result1 = scanner.start_scanning().await;
    if let Err(e) = scan_result1 {
        println!("Failed to start first scan: {:?}", e);
        return;
    }
    
    // Wait for first scan to complete
    sleep(Duration::from_secs(1)).await;
    assert!(!scanner.is_scanning(), "Scanner should have stopped after scan_duration");
    
    // Try a second scan to test recovery
    let scan_result2 = scanner.start_scanning().await;
    if let Err(e) = scan_result2 {
        panic!("Failed to start second scan after recovery: {:?}", e);
    }
    
    // Wait for second scan to complete
    sleep(Duration::from_secs(1)).await;
    assert!(!scanner.is_scanning(), "Scanner should have stopped after second scan");
}

#[tokio::test]
async fn test_scanner_initialization_multiple_times() {
    // Skip in CI or if user wants to skip Bluetooth tests
    if skip_bluetooth_test() {
        return;
    }
    
    // Create a scanner
    let mut scanner = BleScanner::new();
    
    // Try to initialize multiple times (should be idempotent)
    let init1 = scanner.initialize().await;
    if let Err(e) = init1 {
        println!("First initialization failed (expected in CI): {:?}", e);
        return;
    }
    
    // Second initialization should succeed or be a no-op
    let init2 = scanner.initialize().await;
    assert!(init2.is_ok(), "Second initialization should succeed");
    
    // Start and stop a scan to verify everything works
    let scan_result = scanner.start_scanning().await;
    if let Err(e) = scan_result {
        println!("Failed to start scanning: {:?}", e);
        return;
    }
    
    // Stop manually instead of waiting
    let stop_result = scanner.stop_scanning().await;
    assert!(stop_result.is_ok(), "Stopping scan should succeed");
    assert!(!scanner.is_scanning(), "Scanner should not be scanning after stop");
}

#[tokio::test]
async fn test_adapter_selection() {
    // Skip in CI or if user wants to skip Bluetooth tests
    if skip_bluetooth_test() {
        return;
    }
    
    // Create adapter manager
    let adapter_manager_result = AdapterManager::new().await;
    if let Err(e) = adapter_manager_result {
        println!("AdapterManager creation failed (expected without Bluetooth): {:?}", e);
        return;
    }
    
    // Get the list of adapters first
    let adapter_manager = adapter_manager_result.unwrap();
    let adapters = adapter_manager.get_available_adapters().clone();
    
    if adapters.is_empty() {
        println!("No adapters found, skipping adapter selection test");
        return;
    }
    
    // Get the first adapter
    let first_adapter_info = adapters[0].clone();
    
    // Create a new adapter manager instance to avoid borrow conflicts
    let adapter_manager_result2 = AdapterManager::new().await;
    if let Ok(mut adapter_manager2) = adapter_manager_result2 {
        // If there's only one adapter, select_adapter should either succeed or be a no-op
        let select_adapter_result = adapter_manager2.select_adapter(first_adapter_info.index);
        assert!(select_adapter_result.is_ok(), "Using available adapter should succeed");
        
        // Verify selected adapter index
        let selected_adapter = adapter_manager2.get_selected_adapter_info();
        if let Some(info) = selected_adapter {
            assert_eq!(info.index, first_adapter_info.index, 
                    "Selected adapter should match the one we selected");
        }
        
        // Try selecting a non-existent adapter
        let bad_index = 999; // Unlikely to exist
        let bad_adapter_result = adapter_manager2.select_adapter(bad_index);
        assert!(bad_adapter_result.is_err(), "Using non-existent adapter should fail");
    }
}

#[tokio::test]
async fn test_scanner_error_handling() {
    // Skip in CI or if user wants to skip Bluetooth tests
    if skip_bluetooth_test() {
        return;
    }
    
    // Create a scanner
    let mut scanner = BleScanner::new();
    
    // Try to start scanning without initialization
    // This should fail gracefully
    let start_result = scanner.start_scanning().await;
    
    if let Err(e) = start_result {
        println!("Expected error when starting without initialization: {:?}", e);
        // This is the expected path
    } else {
        println!("Scan started without explicit initialization (adapter auto-initialized)");
        // This might happen if the scanner auto-initializes
        
        // Stop the scan to clean up
        let _ = scanner.stop_scanning().await;
    }
    
    // Now initialize properly
    let init_result = scanner.initialize().await;
    if let Err(e) = init_result {
        println!("Initialization failed (expected in CI): {:?}", e);
        return;
    }
    
    // Start scan
    let scan_result = scanner.start_scanning().await;
    if let Err(e) = scan_result {
        println!("Failed to start scanning: {:?}", e);
        return;
    }
    
    // Try to start again while already scanning (should fail)
    let second_start = scanner.start_scanning().await;
    assert!(second_start.is_err(), "Starting scan while already scanning should fail");
    
    // Stop scan
    let _ = scanner.stop_scanning().await;
} 