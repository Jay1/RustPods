use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use btleplug::api::{
    BDAddr, Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter,
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::JoinHandle;
use tokio::time::sleep;
use futures::StreamExt;

use crate::bluetooth::scanner_config::ScanConfig;
use crate::bluetooth::events::{BleEvent, EventBroker, EventFilter};
use crate::airpods::{
    DetectedAirPods, detect_airpods, create_airpods_filter,
    AirPodsFilter
};

/// Custom error type for Bluetooth operations
#[derive(Debug, thiserror::Error)]
pub enum BleError {
    #[error("Failed to find a suitable Bluetooth adapter")]
    AdapterNotFound,
    
    #[error("Bluetooth operation failed: {0}")]
    BtlePlugError(#[from] btleplug::Error),
    
    #[error("Scanning is already in progress")]
    ScanInProgress,
    
    #[error("Device communication error: {0}")]
    DeviceError(String),
    
    #[error("Scanning is not supported on this adapter")]
    ScanningNotSupported,
    
    #[error("Scan was cancelled")]
    ScanCancelled,
    
    #[error("Scan cycle limit reached")]
    ScanCycleLimit,
}

/// Information about a discovered BLE device
#[derive(Debug, Clone)]
pub struct DiscoveredDevice {
    /// Device address
    pub address: BDAddr,
    /// Device name if available
    pub name: Option<String>,
    /// RSSI (signal strength) value
    pub rssi: Option<i16>,
    /// Manufacturer data (used for AirPods detection)
    pub manufacturer_data: HashMap<u16, Vec<u8>>,
    /// Whether this might be an AirPods device
    pub is_potential_airpods: bool,
    /// When the device was last seen
    pub last_seen: std::time::Instant,
}

impl DiscoveredDevice {
    /// Create a new discovered device from a peripheral
    pub async fn from_peripheral(peripheral: &Peripheral) -> Result<Self, BleError> {
        let properties = peripheral.properties().await?;
        let address = peripheral.address();
        
        // Ensure the properties exist
        let properties = match properties {
            Some(props) => props,
            None => return Err(BleError::DeviceError("No device properties available".to_string())),
        };
        
        // The manufacturer data is expected to be in properties if available
        let manufacturer_data = properties.manufacturer_data.clone();
        
        // Use the AirPods filter to check if this is a potential AirPods device
        let is_potential_airpods = create_airpods_filter()(&DiscoveredDevice {
            address,
            name: properties.local_name.clone(),
            rssi: properties.rssi,
            manufacturer_data: manufacturer_data.clone(),
            is_potential_airpods: false, // Not used in filter
            last_seen: std::time::Instant::now(),
        });
        
        Ok(Self {
            address,
            name: properties.local_name,
            rssi: properties.rssi,
            manufacturer_data,
            is_potential_airpods,
            last_seen: std::time::Instant::now(),
        })
    }
}

impl Default for DiscoveredDevice {
    fn default() -> Self {
        Self {
            address: BDAddr::default(),
            name: None,
            rssi: None,
            manufacturer_data: HashMap::new(),
            is_potential_airpods: false,
            last_seen: Instant::now(),
        }
    }
}

/// BLE scanner
pub struct BleScanner {
    /// The config for scanning
    config: ScanConfig,
    /// The Bluetooth adapter
    adapter: Option<Arc<Adapter>>,
    /// Whether scanning is in progress
    is_scanning: bool,
    /// Map of discovered devices by address
    devices: Arc<Mutex<HashMap<BDAddr, DiscoveredDevice>>>,
    /// Task handle for the scan
    scan_task: Option<JoinHandle<()>>,
    /// Channel for canceling the scan
    cancel_sender: Option<Sender<()>>,
    /// Channel for sending events
    event_sender: Option<Sender<BleEvent>>,
    /// Event broker for distribution
    event_broker: Option<EventBroker>,
    /// Current scan cycle count
    scan_cycles_completed: usize,
}

impl Clone for BleScanner {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            adapter: self.adapter.clone(),
            is_scanning: self.is_scanning,
            devices: self.devices.clone(),
            scan_task: None, // Don't clone the JoinHandle
            cancel_sender: None,
            event_sender: None,
            event_broker: self.event_broker.clone(),
            scan_cycles_completed: self.scan_cycles_completed,
        }
    }
}

impl BleScanner {
    /// Create a new BLE scanner with default configuration
    pub fn new() -> Self {
        Self {
            config: ScanConfig::default(),
            adapter: None,
            is_scanning: false,
            devices: Arc::new(Mutex::new(HashMap::new())),
            scan_task: None,
            cancel_sender: None,
            event_sender: None,
            event_broker: None,
            scan_cycles_completed: 0,
        }
    }
    
    /// Create a new BLE scanner with the specified configuration
    pub fn with_config(config: ScanConfig) -> Self {
        Self {
            config,
            adapter: None,
            is_scanning: false,
            devices: Arc::new(Mutex::new(HashMap::new())),
            scan_task: None,
            cancel_sender: None,
            event_sender: None,
            event_broker: None,
            scan_cycles_completed: 0,
        }
    }
    
    /// Update the scanner configuration
    pub fn set_config(&mut self, config: ScanConfig) {
        self.config = config;
    }
    
    /// Get the current scanner configuration
    pub fn get_config(&self) -> &ScanConfig {
        &self.config
    }
    
    /// Initialize the scanner
    pub async fn initialize(&mut self) -> Result<(), BleError> {
        // Create a manager
        let manager = Manager::new().await.map_err(BleError::BtlePlugError)?;
        
        // Get the adapter list
        let adapters = manager.adapters().await.map_err(BleError::BtlePlugError)?;
        
        // Find the first adapter that can be used
        if let Some(adapter) = adapters.into_iter().next() {
            self.adapter = Some(Arc::new(adapter));
            Ok(())
        } else {
            Err(BleError::AdapterNotFound)
        }
    }
    
    /// Start scanning for devices
    pub async fn start_scanning(&mut self) -> Result<Receiver<BleEvent>, BleError> {
        // Check if a scan is already in progress
        if self.is_scanning {
            return Err(BleError::ScanInProgress);
        }
        
        // Make sure we have an adapter
        let adapter = if let Some(adapter) = &self.adapter {
            adapter.clone()
        } else {
            // Try to initialize
            self.initialize().await?;
            
            // Get the adapter
            self.adapter.as_ref().ok_or(BleError::AdapterNotFound)?.clone()
        };
        
        // Create channels for communication
        let (tx, rx) = channel(100);
        self.event_sender = Some(tx.clone());
        
        // Create a channel for cancellation
        let (cancel_tx, cancel_rx) = channel::<()>(1);
        self.cancel_sender = Some(cancel_tx);
        
        // Start scanning task
        let scan_task = self.start_scan_task(tx, cancel_rx, adapter).await?;
        self.scan_task = Some(scan_task);
        self.is_scanning = true;
        
        // Create the event broker if it doesn't exist
        if self.event_broker.is_none() {
            self.event_broker = Some(EventBroker::new());
        }
        
        Ok(rx)
    }
    
    /// Start the background scanning task
    async fn start_scan_task(
        &self, 
        event_tx: Sender<crate::bluetooth::events::BleEvent>, 
        mut cancel_rx: Receiver<()>,
        adapter: Arc<Adapter>
    ) -> Result<JoinHandle<()>, BleError> {
        // Clone necessary values for the task
        let devices_clone = self.devices.clone();
        let config = self.config.clone();
        
        // Start the background scanning task
        let scan_task = tokio::spawn(async move {
            let mut cycle_count = 0;
            
            // Start scanning loop
            'scan_loop: loop {
                // Check if we've reached the maximum number of cycles
                if let Some(max_cycles) = config.max_scan_cycles {
                    if cycle_count >= max_cycles {
                        let _ = event_tx.send(BleEvent::Error("Scan cycle limit reached".to_string())).await;
                        let _ = event_tx.send(BleEvent::ScanningCompleted).await;
                        break;
                    }
                }
                
                // Set timeout for this scan cycle
                let scan_timeout = tokio::time::sleep(config.scan_duration);
                tokio::pin!(scan_timeout);
                
                // Start scan cycle
                if let Err(e) = adapter.start_scan(ScanFilter::default()).await {
                    let _ = event_tx.send(BleEvent::Error(format!("Failed to start scan: {}", e))).await;
                    let _ = event_tx.send(BleEvent::ScanningCompleted).await;
                    break;
                }
                
                // Process events during the scan
                loop {
                    tokio::select! {
                        // Process events from the adapter - handle the future->stream conversion properly
                        Some(event_result) = async {
                            match adapter.events().await {
                                Ok(mut event_stream) => event_stream.next().await,
                                Err(_) => None,
                            }
                        } => {
                            match event_result {
                                CentralEvent::DeviceDiscovered(peripheral_id) => {
                                    // Get the peripheral
                                    match adapter.peripheral(&peripheral_id).await {
                                        Ok(peripheral) => {
                                            // Create a device from the peripheral
                                            match DiscoveredDevice::from_peripheral(&peripheral).await {
                                                Ok(device) => {
                                                    // Store the device
                                                    {
                                                        let mut devices = devices_clone.lock().unwrap();
                                                        devices.insert(device.address, device.clone());
                                                    }
                                                    
                                                    // Notify listeners
                                                    let _ = event_tx.send(BleEvent::DeviceDiscovered(device.clone())).await;
                                                    
                                                    // Check for AirPods and notify if found
                                                    if device.is_potential_airpods {
                                                        if let Some(airpods) = detect_airpods(&device) {
                                                            let _ = event_tx.send(BleEvent::AirPodsDetected(airpods)).await;
                                                        }
                                                    }
                                                },
                                                Err(e) => {
                                                    let _ = event_tx.send(BleEvent::Error(format!("Failed to create device: {}", e))).await;
                                                }
                                            }
                                        },
                                        Err(e) => {
                                            let _ = event_tx.send(BleEvent::Error(format!("Failed to get peripheral: {}", e))).await;
                                        }
                                    }
                                },
                                CentralEvent::ManufacturerDataAdvertisement{id: _id, manufacturer_data: _} => {
                                    // Ignore - devices are processed in DeviceDiscovered
                                },
                                CentralEvent::DeviceDisconnected(_peripheral_id) => {
                                    // Handle device disconnected event
                                },
                                _ => {} // Ignore other events for now
                            }
                        },
                        
                        // Scan duration timeout
                        _ = &mut scan_timeout, if config.auto_stop_scan => {
                            break;
                        },
                        
                        // Cancellation request
                        Some(_) = cancel_rx.recv() => {
                            let _ = adapter.stop_scan().await;
                            let _ = event_tx.send(BleEvent::ScanningCompleted).await;
                            break 'scan_loop;
                        }
                    }
                }
                
                // End of scan cycle
                let _ = adapter.stop_scan().await;
                
                // Increment cycle count and notify listeners
                cycle_count += 1;
                let devices_count = {
                    let devices = devices_clone.lock().unwrap();
                    devices.len()
                };
                let _ = event_tx.send(BleEvent::ScanCycleCompleted { devices_found: devices_count }).await;
                
                // If there are no more cycles or interval is zero, break
                if let Some(max_cycles) = config.max_scan_cycles {
                    if cycle_count >= max_cycles {
                        let _ = event_tx.send(BleEvent::ScanningCompleted).await;
                        break;
                    }
                }
                
                // Wait for the configured interval before the next scan cycle
                if config.interval_between_scans.as_secs() > 0 {
                    // Check for cancellation during the interval
                    tokio::select! {
                        _ = sleep(config.interval_between_scans) => {},
                        Some(_) = cancel_rx.recv() => {
                            let _ = event_tx.send(BleEvent::ScanningCompleted).await;
                            break;
                        }
                    }
                }
            }
        });
        
        Ok(scan_task)
    }
    
    /// Stop the current scan if one is running
    pub async fn stop_scanning(&mut self) -> Result<(), BleError> {
        if !self.is_scanning {
            return Ok(());
        }
        
        // Send cancellation signal if available
        if let Some(tx) = &self.cancel_sender {
            let _ = tx.send(()).await;
        }
        
        // Stop the scan directly as well (belt and suspenders)
        if let Some(adapter) = &self.adapter {
            let _ = adapter.stop_scan().await;
        }
        
        // Abort the background task if it's still running
        if let Some(task) = self.scan_task.take() {
            task.abort();
        }
        
        self.is_scanning = false;
        Ok(())
    }
    
    /// Get a list of currently known devices
    pub fn get_devices(&self) -> Vec<DiscoveredDevice> {
        let devices = self.devices.lock().unwrap();
        devices.values().cloned().collect()
    }
    
    /// Get potential AirPods devices from discovered devices
    pub fn get_potential_airpods(&self) -> Vec<DiscoveredDevice> {
        let devices = self.devices.lock().unwrap();
        devices
            .values()
            .filter(|d| d.is_potential_airpods)
            .cloned()
            .collect()
    }
    
    /// Get fully detected AirPods devices with battery information
    pub fn get_detected_airpods(&self) -> Vec<DetectedAirPods> {
        let devices = self.devices.lock().unwrap();
        devices
            .values()
            .filter_map(|device| detect_airpods(device))
            .collect()
    }
    
    /// Check if scanning is currently in progress
    pub fn is_scanning(&self) -> bool {
        self.is_scanning
    }
    
    /// Get the number of completed scan cycles
    pub fn get_scan_cycles(&self) -> usize {
        self.scan_cycles_completed
    }
    
    /// Clear the device list
    pub fn clear_devices(&mut self) {
        let mut devices = self.devices.lock().unwrap();
        devices.clear();
    }
    
    /// Get a device by address if it exists
    pub fn get_device(&self, address: &BDAddr) -> Option<DiscoveredDevice> {
        let devices = self.devices.lock().unwrap();
        devices.get(address).cloned()
    }
    
    /// Process a discovered device, applying filters and sending events
    #[allow(dead_code)]
    async fn process_discovered_device(
        &self,
        device: DiscoveredDevice,
        event_tx: &Sender<BleEvent>,
    ) -> Result<(), BleError> {
        // Apply RSSI filtering if configured
        if let Some(min_rssi) = self.config.min_rssi {
            if let Some(rssi) = device.rssi {
                if rssi < min_rssi {
                    // Skip this device as it's too far away
                    return Ok(());
                }
            }
        }
        
        // Store the device
        {
            let mut devices = self.devices.lock().unwrap();
            devices.insert(device.address, device.clone());
        }
        
        // Notify listeners
        let _ = event_tx.send(BleEvent::DeviceDiscovered(device.clone())).await;
        
        // Check for AirPods and notify if found
        if device.is_potential_airpods {
            if let Some(airpods) = detect_airpods(&device) {
                let _ = event_tx.send(BleEvent::AirPodsDetected(airpods)).await;
            }
        }
        
        Ok(())
    }
    
    /// Get filtered AirPods devices based on a custom filter
    pub fn get_filtered_airpods(&self, filter: &AirPodsFilter) -> Vec<DiscoveredDevice> {
        let devices = self.get_devices();
        filter.apply_filter(&devices)
    }
    
    /// Get detected AirPods with full details matching a filter
    pub fn get_filtered_detected_airpods(&self, filter: &AirPodsFilter) -> Vec<DetectedAirPods> {
        // First filter the devices
        let filtered_devices = self.get_filtered_airpods(filter);
        
        // Then detect AirPods from the filtered devices
        filtered_devices
            .iter()
            .filter_map(|device| detect_airpods(device))
            .collect()
    }
    
    /// Check if any AirPods matching the filter are present
    pub fn has_airpods_matching(&self, filter: &AirPodsFilter) -> bool {
        !self.get_filtered_airpods(filter).is_empty()
    }
    
    /// Get or create the event broker
    fn event_broker(&mut self) -> &mut EventBroker {
        if self.event_broker.is_none() {
            self.event_broker = Some(EventBroker::new());
        }
        self.event_broker.as_mut().unwrap()
    }
    
    /// Subscribe to all events
    pub fn subscribe_all(&mut self) -> Receiver<BleEvent> {
        self.subscribe(EventFilter::All)
    }
    
    /// Subscribe to scanner events with custom filter
    pub fn subscribe(&mut self, filter: EventFilter) -> Receiver<BleEvent> {
        let (_, rx) = self.event_broker().subscribe(filter);
        rx
    }
    
    /// Subscribe to AirPods events only
    pub fn subscribe_airpods(&mut self) -> Receiver<BleEvent> {
        self.subscribe(EventFilter::airpods_only())
    }
}

impl Drop for BleScanner {
    fn drop(&mut self) {
        // If we have a running task, abort it to avoid leaking resources
        if let Some(task) = self.scan_task.take() {
            task.abort();
        }
    }
} 