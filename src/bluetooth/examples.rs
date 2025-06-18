use crate::error::BluetoothError;
use btleplug::api::{BDAddr, CentralEvent};
use btleplug::platform::Peripheral;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::bluetooth::{BleEvent, BleScanner, ScanConfig};
// Removed unused imports - now using CLI scanner directly
// Example-only mock provider for demo purposes (not for production use)
pub struct ExampleMockAdapterEventsProvider;
impl crate::bluetooth::scanner::AdapterEventsProvider for ExampleMockAdapterEventsProvider {
    fn clone_box(&self) -> Box<dyn crate::bluetooth::scanner::AdapterEventsProvider> {
        Box::new(ExampleMockAdapterEventsProvider)
    }
    fn get_events<'a>(
        &'a self,
    ) -> std::pin::Pin<
        Box<
            dyn futures::Future<
                    Output = Result<
                        std::pin::Pin<Box<dyn futures::Stream<Item = CentralEvent> + Send>>,
                        BluetoothError,
                    >,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async {
            Ok(Box::pin(futures::stream::empty())
                as std::pin::Pin<
                    Box<dyn futures::Stream<Item = CentralEvent> + Send>,
                >)
        })
    }
    fn get_peripheral<'a>(
        &'a self,
        _address: &BDAddr,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<Peripheral, BluetoothError>> + Send + 'a>,
    > {
        Box::pin(async {
            panic!("ExampleMockAdapterEventsProvider::get_peripheral not implemented")
        })
    }
}

/// Basic adapter discovery example
pub async fn discover_adapters() -> Result<(), Box<dyn std::error::Error>> {
    println!("Discovering Bluetooth adapters...");
    let mut scanner = BleScanner::new(
        Arc::new(ExampleMockAdapterEventsProvider),
        ScanConfig::default(),
    );

    // Initialize the scanner (which will find the first available adapter)
    scanner.initialize().await?;

    println!("Found adapter");

    Ok(())
}

/// Basic scanning example with a specific adapter
pub async fn scan_with_adapter() -> Result<(), Box<dyn std::error::Error>> {
    println!("Scanning with adapter...");

    // Create scanner and initialize
    let mut scanner = BleScanner::new(
        Arc::new(ExampleMockAdapterEventsProvider),
        ScanConfig::default(),
    );
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
                    BleEvent::DeviceUpdated(device) => {
                        println!(
                            "Updated device: {} ({}), RSSI: {}, AirPods: {}",
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
                    BleEvent::ScanStarted => {
                        println!("Scanning started.");
                    },
                    BleEvent::ScanStopped => {
                        println!("Scanning stopped.");
                        break;
                    },
                    BleEvent::AirPodsDetected(airpods) => {
                        println!("AirPods detected: {:?} - Battery: L:{}% R:{}% Case:{}%",
                            airpods.device_type,
                            airpods.battery.as_ref().and_then(|b| b.left).unwrap_or(0),
                            airpods.battery.as_ref().and_then(|b| b.right).unwrap_or(0),
                            airpods.battery.as_ref().and_then(|b| b.case).unwrap_or(0));
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
    let _config = ScanConfig {
        scan_duration: Duration::from_secs(3),
        interval_between_scans: Duration::from_secs(2),
        max_scan_cycles: Some(3),
        ..ScanConfig::default()
    };

    let mut scanner = BleScanner::new(
        Arc::new(ExampleMockAdapterEventsProvider),
        ScanConfig::default(),
    );
    scanner.initialize().await?;

    // Start scanning
    let mut events = scanner.start_scanning().await?;

    // Receive events until scanning is completed
    println!("Starting interval-based scanning...");
    let _start_time = Instant::now();

    while let Some(event) = events.recv().await {
        match event {
            BleEvent::DeviceDiscovered(device) => {
                println!(
                    "  - {} ({:?}, RSSI: {:?})",
                    device.name.as_deref().unwrap_or("Unnamed"),
                    device.address,
                    device.rssi
                );
            }
            BleEvent::DeviceUpdated(device) => {
                println!(
                    "  - Updated: {} ({:?}, RSSI: {:?})",
                    device.name.as_deref().unwrap_or("Unnamed"),
                    device.address,
                    device.rssi
                );
            }
            BleEvent::DeviceLost(addr) => {
                println!("  - Device lost: {}", addr);
            }
            BleEvent::Error(e) => {
                println!("  - Error: {}", e);
            }
            BleEvent::AdapterChanged(info) => {
                println!("  - Adapter changed: {}", info);
            }
            BleEvent::ScanCycleCompleted { devices_found } => {
                println!("Scan cycle completed, found {} devices.", devices_found);
                println!("   Waiting for next scan cycle...");
            }
            BleEvent::ScanningCompleted => {
                println!("Scanning completed.");
                break;
            }
            BleEvent::ScanStarted => {
                println!("Scanning started.");
            }
            BleEvent::ScanStopped => {
                println!("Scanning stopped.");
                break;
            }
            BleEvent::AirPodsDetected(airpods) => {
                println!(
                    "  - AirPods detected: {:?} - Battery: L:{}% R:{}% Case:{}%",
                    airpods.device_type,
                    airpods.battery.as_ref().and_then(|b| b.left).unwrap_or(0),
                    airpods.battery.as_ref().and_then(|b| b.right).unwrap_or(0),
                    airpods.battery.as_ref().and_then(|b| b.case).unwrap_or(0)
                );
            }
        }
    }

    // Stop the scanner when we're done
    println!("Example finished. Stopping scanner...");
    scanner.stop_scanning().await?;

    Ok(())
}

/// AirPods filtering example using CLI scanner
pub async fn airpods_filtering() -> Result<(), Box<dyn std::error::Error>> {
    println!("AirPods filtering demo...");

    // Use the CLI scanner directly for fast, reliable AirPods detection
    let cli_path = "scripts/airpods_battery_cli/build/Release/airpods_battery_cli.exe";

    println!("Using CLI scanner: {}", cli_path);
    println!("Scanning for AirPods...\n");

    // Execute the CLI scanner with fast mode
    let mut command = tokio::process::Command::new(cli_path);
    command.arg("--fast"); // Use fast 2-second scan with early exit

    // Hide console window on Windows in release builds
    #[cfg(all(windows, not(debug_assertions)))]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    let output = command.output().await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("CLI scanner failed: {}", stderr).into());
    }

    // Parse the JSON output (extract only the JSON part from stdout)
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Find the JSON part (starts with '{' and ends with '}')
    let json_start = stdout.find('{').ok_or("No JSON found in CLI output")?;
    let json_end = stdout.rfind('}').ok_or("No JSON end found in CLI output")? + 1;
    let json_str = &stdout[json_start..json_end];

    let scanner_result: crate::bluetooth::cli_scanner::CliScannerResult =
        serde_json::from_str(json_str)?;

    println!("🔍 CLI Scanner Results:");
    println!("  - Scanner Version: {}", scanner_result.scanner_version);
    println!("  - Total Devices Found: {}", scanner_result.total_devices);
    println!("  - AirPods Devices: {}", scanner_result.airpods_count);
    println!("  - Status: {}", scanner_result.status);

    if scanner_result.airpods_count > 0 {
        println!("\n🎧 AirPods Detected:");

        for device in &scanner_result.devices {
            if let Some(airpods_data) = &device.airpods_data {
                println!("  Device: {} ({})", device.device_id, device.address);
                println!(
                    "    Model: {} ({})",
                    airpods_data.model, airpods_data.model_id
                );
                println!(
                    "    Battery: L:{}% R:{}% Case:{}%",
                    airpods_data.left_battery,
                    airpods_data.right_battery,
                    airpods_data.case_battery
                );
                println!(
                    "    Charging: L:{} R:{} Case:{}",
                    airpods_data.left_charging,
                    airpods_data.right_charging,
                    airpods_data.case_charging
                );
                println!(
                    "    In Ear: L:{} R:{}",
                    airpods_data.left_in_ear, airpods_data.right_in_ear
                );
                println!(
                    "    Case Status: Lid Open:{}, Both in Case:{}",
                    airpods_data.lid_open, airpods_data.both_in_case
                );
                println!("    Broadcasting Ear: {}", airpods_data.broadcasting_ear);
                println!("    RSSI: {} dBm", device.rssi);
                println!();
            }
        }

        // Demonstrate filtering capabilities
        println!("🔍 Filter Examples:");

        // Filter by model type
        let pro_models: Vec<_> = scanner_result
            .devices
            .iter()
            .filter(|d| {
                d.airpods_data
                    .as_ref()
                    .map(|a| a.model.contains("Pro"))
                    .unwrap_or(false)
            })
            .collect();
        println!("  - Pro models only: {} device(s)", pro_models.len());

        // Filter by signal strength
        let strong_signal: Vec<_> = scanner_result
            .devices
            .iter()
            .filter(|d| d.rssi > -70)
            .collect();
        println!(
            "  - Strong signal (>-70 dBm): {} device(s)",
            strong_signal.len()
        );

        // Filter by charging status
        let charging: Vec<_> = scanner_result
            .devices
            .iter()
            .filter(|d| {
                d.airpods_data
                    .as_ref()
                    .map(|a| a.left_charging || a.right_charging || a.case_charging)
                    .unwrap_or(false)
            })
            .collect();
        println!("  - Currently charging: {} device(s)", charging.len());

        // Filter by in-ear status
        let in_ear: Vec<_> = scanner_result
            .devices
            .iter()
            .filter(|d| {
                d.airpods_data
                    .as_ref()
                    .map(|a| a.left_in_ear || a.right_in_ear)
                    .unwrap_or(false)
            })
            .collect();
        println!("  - In ear: {} device(s)", in_ear.len());
    } else {
        println!("\n❌ No AirPods found");
        println!("   Make sure your AirPods are:");
        println!("   - Powered on and nearby");
        println!("   - Not in deep sleep mode");
        println!("   - Have Bluetooth enabled");

        if scanner_result.total_devices > 0 {
            println!("\n📱 Other Apple devices found:");
            for device in &scanner_result.devices {
                if device.airpods_data.is_none() {
                    println!("  - {} (RSSI: {} dBm)", device.device_id, device.rssi);
                }
            }
        }
    }

    println!("\n✅ AirPods filtering demo completed using v6 modular CLI scanner");

    Ok(())
}
