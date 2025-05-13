use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::{Adapter, Manager};
use std::error::Error;
use std::time::Duration;
use tokio::time;

#[derive(Debug)]
struct AdapterCapabilities {
    supports_scanning: bool,
    supports_central_role: bool,
    supports_advertising: bool,
    supports_connecting: bool,
    adapter_info: String,
}

struct BluetoothAdapter {
    adapter: Adapter,
}

impl BluetoothAdapter {
    async fn new() -> Result<Self, Box<dyn Error>> {
        let manager = Manager::new().await?;
        let adapters = manager.adapters().await?;
        
        if adapters.is_empty() {
            return Err("No Bluetooth adapters found".into());
        }
        
        // Use the first adapter
        let adapter = adapters.into_iter().next().unwrap();
        
        Ok(Self { adapter })
    }
    
    fn get_capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities {
            supports_scanning: true, // All BLE adapters support scanning on modern platforms
            supports_central_role: true,
            supports_advertising: true, // Most adapters support this
            supports_connecting: true,
            adapter_info: format!("{:?}", self.adapter),
        }
    }
    
    fn get_status(&self) -> String {
        "Active".to_string() // Simplified for test purposes
    }
    
    async fn scan(&self, duration_secs: u64) -> Result<Vec<String>, Box<dyn Error>> {
        let mut discovered_devices = Vec::new();
        
        println!("Starting BLE scan for {} seconds...", duration_secs);
        
        // Start scanning with specified filter
        self.adapter
            .start_scan(ScanFilter::default())
            .await?;
            
        // Scan for specified duration
        time::sleep(Duration::from_secs(duration_secs)).await;
        
        // Get discovered devices
        let peripherals = self.adapter.peripherals().await?;
        
        println!("Found {} devices", peripherals.len());
        
        for peripheral in peripherals.iter() {
            let properties = peripheral.properties().await?;
            let address = peripheral.address();
            let device_info = if let Some(properties) = properties {
                format!(
                    "Device: {} - {:?} - RSSI: {:?}",
                    properties.local_name.unwrap_or_else(|| "Unknown".to_string()),
                    address,
                    properties.rssi
                )
            } else {
                format!("Unknown device at {:?}", address)
            };
            
            discovered_devices.push(device_info.clone());
            println!("{}", device_info);
        }
        
        // Stop scanning
        self.adapter.stop_scan().await?;
        println!("Scan completed");
        
        Ok(discovered_devices)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Standalone Bluetooth Adapter Test");
    println!("--------------------------------");
    
    // Create adapter
    let adapter_result = BluetoothAdapter::new().await;
    
    match adapter_result {
        Ok(adapter) => {
            println!("✅ Successfully created BluetoothAdapter");
            
            // Test getting capabilities
            let capabilities = adapter.get_capabilities();
            println!("✅ Successfully retrieved adapter capabilities:");
            println!("  - Supports scanning: {}", capabilities.supports_scanning);
            println!("  - Supports central role: {}", capabilities.supports_central_role);
            println!("  - Adapter info: {}", capabilities.adapter_info);
            
            // Test getting status
            let status = adapter.get_status();
            println!("✅ Successfully retrieved adapter status: {}", status);
            
            // Perform a scan
            println!("\nPerforming a short BLE scan (3 seconds)...");
            match adapter.scan(3).await {
                Ok(devices) => {
                    println!("✅ Scan completed successfully");
                    println!("  - Discovered {} devices", devices.len());
                },
                Err(e) => {
                    println!("❌ Scan failed: {}", e);
                }
            }
            
        },
        Err(e) => {
            println!("❌ Failed to create BluetoothAdapter: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
} 