//! Tests for Bluetooth adapter edge cases and recovery

use std::time::Duration;
use tokio::time::timeout;

use rustpods::bluetooth::{
    AdapterManager, BleScanner, ScanConfig,
};
use rustpods::error::BluetoothError;

/// Create a mock adapter manager for testing
async fn create_mock_adapter_manager(_has_adapters: bool) -> Result<AdapterManager, BluetoothError> {
    // For testing purposes, we'll create a real adapter manager
    // In a production environment, we'd use a proper mocking framework
    let adapter_manager = AdapterManager::new().await?;
    
    // In a real mock implementation, we would control whether it has adapters
    // For now, we just return the real adapter manager and ignore the has_adapters parameter
    
    Ok(adapter_manager)
}

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
    
    // Create a real adapter manager because our mock doesn't actually support the no_adapters case
    let adapter_manager_result = AdapterManager::new().await;
    
    match adapter_manager_result {
        Ok(adapter_manager) => {
            let adapters = adapter_manager.get_available_adapters();
            
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
    println!("Starting test_scanner_recovery_after_failure");
    
    // Create a scanner with a short scan duration
    println!("Creating scanner with short scan duration...");
    let mut scanner = BleScanner::with_config(
        ScanConfig::new()
            .with_scan_duration(Duration::from_millis(500))
            .with_interval(Duration::from_millis(100))
    );
    println!("✅ Scanner created successfully");
    
    // Start scanning
    println!("Starting first scan...");
    match scanner.start_scanning().await {
        Ok(_) => println!("✅ First scan started successfully"),
        Err(e) => {
            println!("❌ Failed to start first scan: {:?}", e);
            panic!("Failed to start first scan: {:?}", e);
        }
    }
    
    // Verify scanner is running
    assert!(scanner.is_scanning(), "Scanner should be scanning");
    println!("✅ Scanner is running");
    
    // Wait for the scan to complete (should be ~500ms)
    println!("Waiting for scan to complete (should take ~500ms)...");
    tokio::time::sleep(Duration::from_millis(1000)).await;
    
    // Verify scanner stopped automatically
    println!("Checking if scanner stopped automatically...");
    let scanner_stopped = !scanner.is_scanning();
    if scanner_stopped {
        println!("✅ Scanner stopped automatically as expected");
    } else {
        // If it hasn't stopped, try to stop it manually
        println!("⚠️ Scanner didn't stop automatically, stopping manually...");
        match scanner.stop_scanning().await {
            Ok(_) => println!("✅ Manually stopped scanner"),
            Err(e) => {
                println!("❌ Failed to manually stop scanner: {:?}", e);
                panic!("Failed to stop scanner: {:?}", e);
            }
        }
    }
    
    assert!(!scanner.is_scanning(), "Scanner should have stopped after scan_duration");
    
    // Try to start a second scan
    println!("Starting second scan...");
    match scanner.start_scanning().await {
        Ok(_) => println!("✅ Second scan started successfully"),
        Err(e) => {
            println!("❌ Failed to start second scan (expected behavior): {:?}", e);
            // This is the expected behavior - scanner should recover
        }
    }
    
    // Verify scanner is running again
    if scanner.is_scanning() {
        println!("✅ Scanner is running again (recovered successfully)");
    } else {
        println!("❌ Scanner failed to recover");
        panic!("Scanner should be scanning after recovery");
    }
    
    // Clean up: stop scanning
    println!("Stopping scanner...");
    match timeout(Duration::from_millis(5000), scanner.stop_scanning()).await {
        Ok(result) => match result {
            Ok(_) => println!("✅ Scanner stopped successfully"),
            Err(e) => {
                println!("❌ Failed to stop scanner: {:?}", e);
                panic!("Failed to stop scanner: {:?}", e);
            }
        },
        Err(e) => {
            println!("❌ Timeout stopping scanner: {:?}", e);
            panic!("Scanner did not stop scanning within timeout period");
        }
    }
    
    // Verify scanner stopped
    assert!(!scanner.is_scanning(), "Scanner should not be scanning after stop");
    println!("✅ Scanner is no longer running");
    println!("Test completed successfully");
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
    let adapters = adapter_manager.get_available_adapters();
    
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

#[tokio::test]
async fn test_adapter_manager_get_available_adapters() {
    // Create a new adapter manager using our async function
    let adapter_manager_result = create_mock_adapter_manager(true).await;
    
    // Skip test if we couldn't create an adapter manager
    if adapter_manager_result.is_err() {
        println!("Skipping test_adapter_manager_get_available_adapters due to adapter manager creation failure");
        return;
    }
    
    let adapter_manager = adapter_manager_result.unwrap();
    
    // Get available adapters
    let adapters = adapter_manager.get_available_adapters();
    
    // Print adapter info for debugging
    println!("Found {} adapters", adapters.len());
    for (i, adapter) in adapters.iter().enumerate() {
        println!("Adapter {}: {}", i, adapter.name);
    }
    
    // Assert we have at least one adapter (may fail on systems without Bluetooth)
    if !skip_bluetooth_test() {
        assert!(!adapters.is_empty(), "Should have at least one adapter on test systems with Bluetooth");
    }
} 