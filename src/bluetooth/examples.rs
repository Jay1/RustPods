use std::time::{Duration, Instant};
// use btleplug::api::BDAddr;

use crate::bluetooth::{BleScanner, BleEvent, ScanConfig};
use crate::airpods::{airpods_all_models_filter, airpods_pro_filter, airpods_nearby_filter};

/// Basic adapter discovery example
pub async fn discover_adapters() -> Result<(), Box<dyn std::error::Error>> {
    println!("Discovering Bluetooth adapters...");
    let mut scanner = BleScanner::new();
    
    // Initialize the scanner (which will find the first available adapter)
    scanner.initialize().await?;
    
    println!("Found adapter");
    
    Ok(())
}

/// Basic scanning example with a specific adapter
pub async fn scan_with_adapter() -> Result<(), Box<dyn std::error::Error>> {
    println!("Scanning with adapter...");
    
    // Create scanner and initialize
    let mut scanner = BleScanner::new();
    scanner.initialize().await?;
    
    // Start scanning
    let mut events = scanner.start_scanning().await?;
    
    // Set a timeout for scanning
    let timeout = tokio::time::sleep(Duration::from_secs(5));
    tokio::pin!(timeout);
    
    // Process events
    println!("Scanning for 5 seconds...");
    loop {
        tokio::select! {
            Some(event) = events.recv() => {
                match event {
                    BleEvent::DeviceDiscovered(device) => {
                        println!(
                            "Found device: {} ({}), RSSI: {}, AirPods: {}", 
                            device.name.as_deref().unwrap_or("Unnamed"), 
                            device.address,
                            device.rssi.unwrap_or(0),
                            if device.is_potential_airpods { "Yes" } else { "No" }
                        );
                    },
                    BleEvent::DeviceLost(addr) => {
                        println!("Lost device: {}", addr);
                    },
                    BleEvent::Error(err) => {
                        println!("Error: {}", err);
                    },
                    BleEvent::AdapterChanged(info) => {
                        println!("Adapter changed: {}", info);
                    },
                    BleEvent::ScanCycleCompleted { devices_found } => {
                        println!("Scan cycle completed. Found {} devices.", devices_found);
                    },
                    BleEvent::ScanningCompleted => {
                        println!("Scanning completed.");
                        break;
                    },
                    BleEvent::AirPodsDetected(airpods) => {
                        println!("AirPods detected: {:?} - Battery: L:{}% R:{}% Case:{}%", 
                            airpods.device_type,
                            airpods.battery.left.unwrap_or(0),
                            airpods.battery.right.unwrap_or(0),
                            airpods.battery.case.unwrap_or(0));
                    }
                }
            },
            _ = &mut timeout => {
                println!("Scan timeout reached.");
                break;
            }
        }
    }
    
    // Stop scanning
    scanner.stop_scanning().await?;
    
    Ok(())
}

/// Interval-based scanning example
pub async fn interval_scanning() -> Result<(), Box<dyn std::error::Error>> {
    println!("Interval-based scanning...");
    
    // Create a scanner with a custom config for interval scanning
    let config = ScanConfig {
        scan_duration: Duration::from_secs(3),
        interval_between_scans: Duration::from_secs(2),
        max_scan_cycles: Some(3),
        ..ScanConfig::default()
    };
    
    let mut scanner = BleScanner::with_config(config);
    scanner.initialize().await?;
    
    // Start scanning
    let mut events = scanner.start_scanning().await?;
    
    // Receive events until scanning is completed
    println!("Starting interval-based scanning...");
    let _start_time = Instant::now();
    
    while let Some(event) = events.recv().await {
        match event {
            BleEvent::DeviceDiscovered(device) => {
                println!("  - {} ({:?}, RSSI: {:?})", 
                         device.name.as_deref().unwrap_or("Unnamed"),
                         device.address,
                         device.rssi);
            },
            BleEvent::DeviceLost(addr) => {
                println!("  - Device lost: {}", addr);
            },
            BleEvent::Error(e) => {
                println!("  - Error: {}", e);
            },
            BleEvent::AdapterChanged(info) => {
                println!("  - Adapter changed: {}", info);
            },
            BleEvent::ScanCycleCompleted { devices_found } => {
                println!("Scan cycle completed, found {} devices.", devices_found);
                println!("   Waiting for next scan cycle...");
            },
            BleEvent::ScanningCompleted => {
                println!("Scanning completed.");
                break;
            },
            BleEvent::AirPodsDetected(airpods) => {
                println!("  - AirPods detected: {:?} - Battery: L:{}% R:{}% Case:{}%", 
                         airpods.device_type,
                         airpods.battery.left.unwrap_or(0),
                         airpods.battery.right.unwrap_or(0),
                         airpods.battery.case.unwrap_or(0));
            }
        }
    }
    
    // Stop the scanner when we're done
    println!("Example finished. Stopping scanner...");
    scanner.stop_scanning().await?;
    
    Ok(())
}

/// AirPods filtering example
pub async fn airpods_filtering() -> Result<(), Box<dyn std::error::Error>> {
    println!("AirPods filtering demo...");
    
    // Create scanner with extended scan time
    let config = ScanConfig {
        scan_duration: Duration::from_secs(10),
        auto_stop_scan: true,
        ..ScanConfig::default()
    };
    
    let mut scanner = BleScanner::with_config(config);
    scanner.initialize().await?;
    
    // Start scanning
    let events = scanner.subscribe_all();
    scanner.start_scanning().await?;
    
    // Set a timeout
    let timeout = tokio::time::sleep(Duration::from_secs(12)); // Slightly longer than scan_duration
    tokio::pin!(timeout);
    
    // Track discovered AirPods
    println!("Scanning for AirPods...\n");
    
    // Create a clone for filters
    let scanner_clone = scanner.clone();
    
    tokio::spawn(async move {
        let mut events = events;
        while let Some(event) = events.recv().await {
            match event {
                BleEvent::AirPodsDetected(airpods) => {
                    println!("🎧 Detected AirPods: {:?}", airpods.device_type);
                    println!("    Battery: L:{}% R:{}% Case:{}%",
                        airpods.battery.left.unwrap_or(0),
                        airpods.battery.right.unwrap_or(0),
                        airpods.battery.case.unwrap_or(0));
                    
                    // Get filtered devices
                    let all_airpods = scanner_clone.get_filtered_airpods(&airpods_all_models_filter());
                    let pro_airpods = scanner_clone.get_filtered_airpods(&airpods_pro_filter());
                    let nearby_airpods = scanner_clone.get_filtered_airpods(&airpods_nearby_filter(-70));
                    
                    // Custom filter: AirPods with strong signal
                    let custom_airpods_filter = crate::airpods::AirPodsFilter::new()
                        .with_min_rssi(-70); // Strong signal filter
                    
                    let custom_filtered = scanner_clone.get_filtered_airpods(&custom_airpods_filter);
                    
                    // Display filter results
                    println!();
                    println!("🔍 Filtered results:");
                    println!("  - All AirPods: {} device(s)", all_airpods.len());
                    println!("  - Pro models only: {} device(s)", pro_airpods.len());
                    println!("  - Nearby AirPods (-60 RSSI): {} device(s)", nearby_airpods.len());
                    println!("  - Custom filter: {} device(s)", custom_filtered.len());
                    println!();
                },
                _ => {} // Ignore other events
            }
        }
    });
    
    // Wait for timeout
    timeout.await;
    
    // Stop scanning
    scanner.stop_scanning().await?;
    
    Ok(())
} 