use std::error::Error;
use rustpods::bluetooth::adapter::BluetoothAdapter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Testing BluetoothAdapter functionality");
    
    // Create a new adapter
    let adapter_result = BluetoothAdapter::new().await;
    
    match adapter_result {
        Ok(adapter) => {
            println!("✓ Successfully created BluetoothAdapter");
            
            // Test getting capabilities
            let capabilities = adapter.get_capabilities();
            println!("✓ Successfully retrieved adapter capabilities");
            println!("  Supports scanning: {}", capabilities.supports_scanning);
            println!("  Supports central role: {}", capabilities.supports_central_role);
            println!("  Supports advertising: {}", capabilities.supports_advertising);
            println!("  Supports connecting: {}", capabilities.supports_connecting);
            println!("  Is powered on: {}", capabilities.is_powered_on);
            println!("  Max connections: {}", capabilities.max_connections);
            println!("  Adapter status: {:?}", capabilities.status);
            
            // Test getting status
            let status = adapter.get_status();
            println!("✓ Successfully retrieved adapter status: {:?}", status);
            
            // Test getting address - this will likely fail on most platforms due to permission issues
            let address_result = adapter.get_address().await;
            match address_result {
                Ok(Some(addr)) => {
                    println!("✓ Successfully retrieved adapter address: {}", addr);
                },
                Ok(None) => {
                    println!("⚠️ Could not retrieve adapter address (None)");
                },
                Err(e) => {
                    println!("⚠️ Error retrieving adapter address: {}", e);
                }
            }
            
            // Test starting a scan
            match adapter.start_scan().await {
                Ok(_) => {
                    println!("✓ Successfully started scan");
                    
                    // Wait a bit to allow some device discovery
                    println!("Scanning for devices for 5 seconds...");
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    
                    // Get discovered devices
                    match adapter.get_discovered_devices().await {
                        Ok(devices) => {
                            println!("✓ Found {} device(s)", devices.len());
                            for (i, device) in devices.iter().enumerate() {
                                println!("  Device {}: {}", i+1, device.address);
                                if let Some(name) = &device.name {
                                    println!("    Name: {}", name);
                                }
                                if let Some(rssi) = device.rssi {
                                    println!("    RSSI: {}", rssi);
                                }
                            }
                        },
                        Err(e) => {
                            println!("❌ Error getting discovered devices: {}", e);
                        }
                    }
                    
                    // Stop scanning
                    if let Err(e) = adapter.stop_scan().await {
                        println!("❌ Error stopping scan: {}", e);
                    } else {
                        println!("✓ Successfully stopped scan");
                    }
                },
                Err(e) => {
                    println!("❌ Error starting scan: {}", e);
                }
            }
            
            println!("Test completed!");
            Ok(())
        },
        Err(e) => {
            println!("❌ Error creating adapter: {}", e);
            Err(e.into())
        }
    }
} 