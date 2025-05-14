use std::time::{Duration, Instant};
use tokio::time::sleep;

use rustpods::airpods::{
    AirPodsType, AirPodsFilter,
    airpods_all_models_filter, airpods_pro_filter, airpods_nearby_filter
};
use rustpods::bluetooth::{BleScanner, BleEvent, ScanConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("AirPods Filter Test");
    println!("------------------");
    
    // Create a scanner with AirPods optimized configuration
    let config = ScanConfig::airpods_optimized();
    let mut scanner = BleScanner::with_config(config);
    
    // Initialize the scanner
    println!("Initializing Bluetooth scanner...");
    if let Err(e) = scanner.initialize().await {
        println!("Error initializing scanner: {}", e);
        return Ok(());
    }
    
    // Start scanning
    println!("Starting Bluetooth scan...");
    let events = match scanner.start_scanning().await {
        Ok(events) => events,
        Err(e) => {
            println!("Error starting scan: {}", e);
            return Ok(());
        }
    };
    
    // Set up test filters
    println!("\nFilters configured:");
    println!("1. All AirPods filter - detects any AirPods model");
    println!("2. AirPods Pro filter - only detects Pro models");
    println!("3. Nearby AirPods filter - stronger signal devices");
    println!("4. Custom filter - AirPods 3rd gen with min RSSI");
    
    // Run scan for a limited time
    let scan_duration = Duration::from_secs(30);
    let _start_time = Instant::now();
    
    println!("\nScanning for {} seconds...", scan_duration.as_secs());
    
    // Clone the scanner for use in the tokio::spawn task
    let scanner_clone = scanner.clone();

    // Process events until timeout
    tokio::spawn(async move {
        let mut events = events;
        while let Some(event) = events.recv().await {
            match event {
                BleEvent::DeviceDiscovered(device) => {
                    if device.is_potential_airpods {
                        println!("Potential AirPods: {} (RSSI: {:?})", 
                            device.name.as_deref().unwrap_or("Unnamed"), 
                            device.rssi);
                    }
                    
                    // Use the cloned scanner here - properly await
                    let _all_airpods = scanner_clone.get_filtered_airpods(&airpods_all_models_filter()).await;
                }
                BleEvent::AirPodsDetected(airpods) => {
                    println!("AirPods detected: {:?} ({:?})", 
                        airpods.name.unwrap_or_else(|| "Unknown".to_string()), 
                        airpods.device_type);
                    
                    // Print battery info if available
                    if let Some(battery) = &airpods.battery {
                        if let Some(left) = battery.left {
                            println!("  Left battery: {}%", left);
                        }
                        if let Some(right) = battery.right {
                            println!("  Right battery: {}%", right);
                        }
                        if let Some(case) = battery.case {
                            println!("  Case battery: {}%", case);
                        }
                    }
                }
                BleEvent::ScanCycleCompleted { devices_found } => {
                    println!("\nScan cycle completed, {} devices found", devices_found);
                    
                    // Get filtered devices - properly await each call
                    let _all_airpods = scanner_clone.get_filtered_airpods(&airpods_all_models_filter()).await;
                    let pro_airpods = scanner_clone.get_filtered_airpods(&airpods_pro_filter()).await;
                    let nearby_airpods = scanner_clone.get_filtered_airpods(&airpods_nearby_filter(-70)).await;
                    
                    // Custom filter: AirPods 3 with strong signal
                    let custom_filter = rustpods::airpods::AirPodsFilterOptions::new()
                        .with_models(vec![AirPodsType::AirPods3])
                        .with_min_rssi(-70)
                        .create_filter_function();
                    
                    let custom_filtered = scanner_clone.get_filtered_airpods(&custom_filter).await;
                    
                    // Display filter results
                    println!("  All AirPods: {} devices", _all_airpods.len());
                    println!("  Pro models: {} devices", pro_airpods.len());
                    println!("  Nearby AirPods: {} devices", nearby_airpods.len());
                    println!("  AirPods 3 (custom): {} devices", custom_filtered.len());
                    
                    // Show details for nearby devices
                    if !nearby_airpods.is_empty() {
                        println!("\n  Nearby AirPods details:");
                        for (i, device) in nearby_airpods.iter().enumerate() {
                            println!("    {}. {} (RSSI: {:?})", 
                                i+1, 
                                device.name.as_deref().unwrap_or("Unknown"), 
                                device.rssi);
                        }
                    }
                }
                BleEvent::Error(e) => {
                    println!("Error: {}", e);
                }
                _ => {}
            }
        }
    });
    
    // Wait for scan to complete
    sleep(scan_duration).await;
    
    // Stop the scanner
    println!("\nScan complete. Stopping...");
    if let Err(e) = scanner.stop_scanning().await {
        println!("Error stopping scan: {}", e);
    }
    
    Ok(())
} 