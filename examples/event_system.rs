use std::time::{Duration, Instant};
use tokio::time::sleep;
use futures::StreamExt;

use rustpods::bluetooth::{
    BleScanner, BleEvent, ScanConfig, EventFilter, receiver_to_stream
};
use rustpods::bluetooth::events::EventType;
use rustpods::airpods::AirPodsType;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Bluetooth Event System Demo");
    println!("==========================");
    
    // Create a scanner with AirPods optimized configuration
    let mut scanner = BleScanner::with_config(ScanConfig::airpods_optimized());
    
    // Initialize the scanner
    println!("Initializing scanner...");
    scanner.initialize().await?;
    
    // Create multiple subscribers with different filters
    println!("Setting up subscribers...");
    
    // Subscriber 1: All events
    let all_events_rx = scanner.subscribe_all();
    
    // Subscriber 2: Only AirPods events
    let airpods_rx = scanner.subscribe_airpods();
    
    // Subscriber 3: Custom filter for AirPods Pro only with strong signal
    let airpods_pro_rx = scanner.subscribe(EventFilter::custom(|event| {
        if let BleEvent::AirPodsDetected(airpods) = event {
            // Check for Pro models and strong signal
            let is_pro = matches!(airpods.device_type, 
                AirPodsType::AirPodsPro | AirPodsType::AirPodsPro2);
            let strong_signal = airpods.rssi.map_or(false, |rssi| rssi > -65);
            
            return is_pro && strong_signal;
        }
        false
    }));
    
    // Subscriber 4: Just scan control events (errors, cycle completions, etc.)
    let control_rx = scanner.subscribe(EventFilter::event_types(vec![
        EventType::Error, 
        EventType::ScanCycleCompleted,
        EventType::ScanningCompleted,
        EventType::AdapterChanged
    ]));
    
    // Start listeners for all subscribers
    println!("Starting event listeners...");
    
    // Listener 1: All events
    let all_events_stream = receiver_to_stream(all_events_rx);
    let mut all_events_stream = Box::pin(all_events_stream);
    
    tokio::spawn(async move {
        let mut count = 0;
        while let Some(event) = all_events_stream.next().await {
            count += 1;
            if count % 5 == 0 {
                println!("[ALL] Received {} events...", count);
            }
            
            // Print out AirPods discoveries
            match &event {
                BleEvent::AirPodsDetected(airpods) => {
                    println!("[ALL] AirPods: {:?} ({})", 
                             airpods.device_type, 
                             airpods.name.as_deref().unwrap_or("Unnamed"));
                }
                _ => {}
            }
        }
    });
    
    // Listener 2: AirPods events
    let airpods_stream = receiver_to_stream(airpods_rx);
    let mut airpods_stream = Box::pin(airpods_stream);
    
    tokio::spawn(async move {
        while let Some(event) = airpods_stream.next().await {
            if let BleEvent::AirPodsDetected(airpods) = &event {
                println!("[AIRPODS] Detected: {:?} with battery L:{}% R:{}% Case:{}%", 
                         airpods.device_type,
                         airpods.battery.left.unwrap_or(0),
                         airpods.battery.right.unwrap_or(0),
                         airpods.battery.case.unwrap_or(0));
            }
        }
    });
    
    // Listener 3: AirPods Pro only
    let airpods_pro_stream = receiver_to_stream(airpods_pro_rx);
    let mut airpods_pro_stream = Box::pin(airpods_pro_stream);
    
    tokio::spawn(async move {
        while let Some(event) = airpods_pro_stream.next().await {
            if let BleEvent::AirPodsDetected(airpods) = &event {
                println!("[PRO] Nearby AirPods Pro: {} (RSSI: {:?})", 
                         airpods.name.as_deref().unwrap_or("Unnamed"),
                         airpods.rssi);
            }
        }
    });
    
    // Listener 4: Control events
    let control_stream = receiver_to_stream(control_rx);
    let mut control_stream = Box::pin(control_stream);
    
    tokio::spawn(async move {
        while let Some(event) = control_stream.next().await {
            match &event {
                BleEvent::Error(err) => {
                    println!("[CONTROL] Error: {}", err);
                }
                BleEvent::ScanCycleCompleted { devices_found } => {
                    println!("[CONTROL] Scan cycle completed, found {} devices", devices_found);
                }
                BleEvent::ScanningCompleted => {
                    println!("[CONTROL] Scanning completed");
                }
                BleEvent::AdapterChanged(info) => {
                    println!("[CONTROL] Adapter changed: {}", info);
                }
                _ => {}
            }
        }
    });
    
    // Start scanning
    println!("Starting Bluetooth scan...");
    scanner.start_scanning().await?;
    
    // Run for a fixed duration
    let scan_duration = Duration::from_secs(30);
    println!("Running scan for {} seconds...", scan_duration.as_secs());
    
    sleep(scan_duration).await;
    
    // Stop scanning
    println!("Stopping scan...");
    scanner.stop_scanning().await?;
    
    // Allow time for events to be processed
    sleep(Duration::from_secs(1)).await;
    
    println!("Event system demo completed.");
    
    Ok(())
} 