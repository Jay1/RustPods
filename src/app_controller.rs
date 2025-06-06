use tokio::runtime::Runtime;
use tokio::runtime::Handle;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::bluetooth::{
    BleScanner, EventBroker, EventFilter, BleEvent, 
    ScanConfig
};
use crate::bluetooth::battery::AirPodsBatteryStatus;
use crate::airpods::DetectedAirPods;
use crate::ui::Message;

/// Main application controller that manages Bluetooth connections and UI state
pub struct AppController {
    /// Runtime handle for async tasks
    runtime_handle: Arc<Handle>,
    /// Runtime for async tasks (optional, only created if needed)
    runtime: Option<Arc<Runtime>>,
    /// Bluetooth event broker
    event_broker: EventBroker,
    /// Bluetooth scanner
    scanner: Arc<Mutex<Option<BleScanner>>>,
    /// Scan configuration
    scan_config: ScanConfig,
    /// Scan task handle
    scan_task: Option<JoinHandle<()>>,
    /// Message sender to UI
    ui_sender: mpsc::UnboundedSender<Message>,
    /// Currently connected AirPods device
    connected_device: Arc<Mutex<Option<DetectedAirPods>>>,
    /// Battery monitoring task
    battery_task: Option<JoinHandle<()>>,
}

impl AppController {
    /// Create a new application controller
    pub fn new(ui_sender: mpsc::UnboundedSender<Message>) -> Self {
        // Try to get the existing runtime handle or create a new runtime if needed
        let (runtime, runtime_handle) = match Handle::try_current() {
            Ok(handle) => {
                // Already inside a Tokio runtime, use its handle
                (None, Arc::new(handle))
            },
            Err(_) => {
                // No runtime exists, create one
                let runtime = Arc::new(Runtime::new().expect("Failed to create tokio runtime"));
                let handle = Arc::new(runtime.handle().clone());
                (Some(runtime), handle)
            }
        };
        
        // Create event broker for Bluetooth events
        let mut event_broker = EventBroker::new();
        event_broker.set_inactive_timeout(Some(Duration::from_secs(300))); // 5 minute timeout
        
        // Default scan configuration
        let scan_config = ScanConfig::default()
            .with_scan_duration(Duration::from_secs(10))
            .with_interval(Duration::from_secs(30))
            .with_min_rssi(Some(-70));
        
        Self {
            runtime,
            runtime_handle,
            event_broker,
            scanner: Arc::new(Mutex::new(None)),
            scan_config,
            scan_task: None,
            ui_sender,
            connected_device: Arc::new(Mutex::new(None)),
            battery_task: None,
        }
    }
    
    /// Initialize the application controller
    pub async fn initialize(&mut self) -> Result<(), String> {
        // Get clones of the needed data to avoid borrowing self
        let mut event_broker = self.event_broker.clone();
        let ui_sender = self.ui_sender.clone();
        let connected_device = self.connected_device.clone();
        
        // Start the event broker
        event_broker.start();
        
        // Subscribe to AirPods events
        let mut broker_clone = event_broker.clone();
        let (_id, mut rx_airpods) = broker_clone.subscribe(EventFilter::airpods_only());
        
        // Clone UI sender to move into task
        let ui_sender_clone = ui_sender.clone();
        let device_clone = connected_device.clone();
        
        // Spawn task to handle events
        tokio::spawn(async move {
            while let Some(event) = rx_airpods.recv().await {
                if let BleEvent::AirPodsDetected(airpods) = event {
                    // Send AirPods discovered event to UI
                    ui_sender_clone.send(Message::AirPodsConnected(airpods.clone())).ok();
                    
                    // Store connected device
                    let mut device = device_clone.lock().unwrap();
                    *device = Some(airpods);
                }
            }
        });
        
        // Subscribe to all device events
        let (_id, mut rx_device) = broker_clone.subscribe(EventFilter::event_types(vec![
            crate::bluetooth::events::EventType::DeviceDiscovered,
            crate::bluetooth::events::EventType::DeviceLost,
        ]));
        
        // Spawn task to handle events
        tokio::spawn(async move {
            while let Some(event) = rx_device.recv().await {
                match event {
                    BleEvent::DeviceDiscovered(device) => {
                        ui_sender.send(Message::DeviceDiscovered(device)).ok();
                    }
                    BleEvent::DeviceLost(address) => {
                        // Developer log and optional user feedback
                        log::info!("Device lost: {}", address);
                        let _ = ui_sender.send(Message::ShowToast(format!("Device lost: {}", address)));
                    }
                    _ => {} // Ignore other events
                }
            }
        });
        
        Ok(())
    }
    
    // Helper method to run async code from a synchronous context
    fn run_async<F, T>(&self, f: F) -> T
    where
        F: std::future::Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        // Always use a channel-based approach to avoid runtime nesting issues
        let (tx, rx) = std::sync::mpsc::channel();
        
        // Spawn the future onto the runtime
        let handle = self.runtime_handle.clone();
        handle.spawn(async move {
            let result = f.await;
            tx.send(result).expect("Failed to send result");
        });
        
        // Wait for the result
        rx.recv().expect("Failed to receive result")
    }
    
    /// Connect to a selected device by address
    pub fn connect_device(&mut self, address: String) -> Result<(), String> {
        // Clone necessary values to use in the async block
        let ui_sender = self.ui_sender.clone();
        let connected_device = self.connected_device.clone();
        let runtime_handle = self.runtime_handle.clone();
        
        let result = self.run_async(async move {
            // Developer log
            log::info!("Selected device with address: {}", address);
            
            // For now, we just set the connection timestamp and let the UI handle the rest
            // In a real implementation, we would actually connect to the device
            ui_sender.send(Message::SelectDevice(address)).ok();
            
            // Start battery monitoring
            let task = start_battery_monitoring(connected_device, ui_sender, runtime_handle);
            
            Ok(task)
        });
        
        // Store the battery task if successful
        if let Ok(task) = result {
            self.battery_task = Some(task);
            Ok(())
        } else {
            result.map(|_| ())
        }
    }
    
    /// Helper function to start battery monitoring (extracted to avoid self capture)
    fn start_battery_monitoring(&mut self) {
        // Call the extracted function with cloned values
        let task = start_battery_monitoring(
            self.connected_device.clone(),
            self.ui_sender.clone(),
            self.runtime_handle.clone()
        );
        
        self.battery_task = Some(task);
    }
    
    /// Disconnect from the current device
    pub fn disconnect_device(&mut self) -> Result<(), String> {
        // Clear connected device
        let mut device = self.connected_device.lock().unwrap();
        *device = None;
        
        // Stop battery monitoring task
        if let Some(task) = self.battery_task.take() {
            task.abort();
        }
        
        Ok(())
    }
    
    /// Shutdown the application controller and cleanup resources
    pub fn shutdown(&mut self) -> Result<(), String> {
        // Clone necessary values to use in the async block
        let scanner_arc = self.scanner.clone();
        let mut event_broker = self.event_broker.clone();
        
        // Get the current battery task to abort it outside the async block
        let battery_task = self.battery_task.take();
        let scan_task = self.scan_task.take();
        
        self.run_async(async move {
            // Check if scanning
            let is_scanning = scanner_arc.lock().unwrap().is_some();
            
            if is_scanning {
                // Take the scanner out to avoid holding the guard across await
                let mut scanner_option: Option<BleScanner> = std::mem::take(&mut *scanner_arc.lock().unwrap());
                
                // Stop scanner if we have one
                if let Some(scanner) = scanner_option.as_mut() {
                    let _ = scanner.stop_scanning().await;
                }
                
                // We're done with the scanner, just leave it as None
            }
            
            // Abort tasks (outside the async block if possible)
            if let Some(task) = battery_task {
                task.abort();
            }
            
            if let Some(task) = scan_task {
                task.abort();
            }
            
            // Shutdown event broker
            event_broker.shutdown().await;
            
            Ok(())
        })
    }

    /// Start the controller
    pub fn start(&mut self) -> Result<(), String> {
        // Since we can't use run_async with self.initialize() without borrowing issues,
        // create a blocking runtime for the initialization only
        let rt = Runtime::new().expect("Failed to create blocking runtime");
        rt.block_on(self.initialize())
    }
}

/// Standalone function to start battery monitoring for the connected device
fn start_battery_monitoring(
    connected_device: Arc<Mutex<Option<DetectedAirPods>>>,
    ui_sender: mpsc::UnboundedSender<Message>,
    runtime_handle: Arc<Handle>
) -> JoinHandle<()> {
    // Create the battery monitoring task
    runtime_handle.spawn(async move {
        // Loop while we have a connected device
        loop {
            // Sleep for some time
            tokio::time::sleep(Duration::from_secs(5)).await;
            
            // Check if we have a connected device
            let device = connected_device.lock().unwrap().clone();
            if let Some(_airpods) = device {
                // Simulate getting battery status
                // In a real app, we would read from the device
                let battery_level = crate::airpods::AirPodsBattery {
                    left: Some((80 + (tokio::time::Instant::now().elapsed().as_secs() % 20) as u8).min(100)),
                    right: Some((75 + (tokio::time::Instant::now().elapsed().as_secs() % 20) as u8).min(100)),
                    case: Some((90 + (tokio::time::Instant::now().elapsed().as_secs() % 10) as u8).min(100)),
                    charging: Some(if (tokio::time::Instant::now().elapsed().as_secs() % 60) < 30 {
                        crate::airpods::AirPodsChargingState::CaseCharging
                    } else {
                        crate::airpods::AirPodsChargingState::NotCharging
                    }),
                };
                
                // Create battery status
                let status = AirPodsBatteryStatus::new(battery_level);
                
                // Send to UI
                ui_sender.send(Message::BatteryStatusUpdated(status)).ok();
            } else {
                // If no device is connected, exit the loop
                break;
            }
        }
    })
}

impl Drop for AppController {
    fn drop(&mut self) {
        // Manually handle cleanup since we can't safely call shutdown() in Drop
        // This is to avoid the nested runtime issue if we're already in a runtime context
        
        // Only abort tasks, don't block on async operations in drop
        if let Some(task) = self.battery_task.take() {
            task.abort();
        }
        
        if let Some(task) = self.scan_task.take() {
            task.abort();
        }
    }
} 