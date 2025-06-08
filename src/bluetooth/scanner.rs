use futures::{Future, Stream};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

use btleplug::api::{BDAddr, Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::JoinHandle;
use tokio::time::{interval, sleep};
use uuid::Uuid;

use crate::airpods::{create_airpods_filter, detect_airpods, DetectedAirPods};
use crate::bluetooth::events::{BleEvent, EventBroker, EventFilter};
use crate::bluetooth::scanner_config::ScanConfig;
use crate::config::{AppConfig, Configurable};

// Import new error types
use crate::error::{BluetoothError, ErrorContext, RecoveryAction};

/// Trait for providing Bluetooth adapter events and peripheral lookup, enabling dependency injection for testing.
#[allow(clippy::type_complexity)]
pub trait AdapterEventsProvider: Send + Sync {
    fn clone_box(&self) -> Box<dyn AdapterEventsProvider>;
    fn get_events<'a>(
        &'a self,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Pin<Box<dyn Stream<Item = CentralEvent> + Send>>,
                        BluetoothError,
                    >,
                > + Send
                + 'a,
        >,
    >;
    fn get_peripheral<'a>(
        &'a self,
        address: &BDAddr,
    ) -> Pin<Box<dyn Future<Output = Result<Peripheral, BluetoothError>> + Send + 'a>>;
}

impl Clone for Box<dyn AdapterEventsProvider> {
    fn clone(&self) -> Box<dyn AdapterEventsProvider> {
        self.clone_box()
    }
}

/// Real implementation of AdapterEventsProvider for the actual Bluetooth adapter.
pub struct RealAdapterEventsProvider {
    adapter: Arc<Adapter>,
}

impl RealAdapterEventsProvider {
    pub fn new(adapter: Arc<Adapter>) -> Self {
        Self { adapter }
    }
}

impl AdapterEventsProvider for RealAdapterEventsProvider {
    fn clone_box(&self) -> Box<dyn AdapterEventsProvider> {
        Box::new(Self {
            adapter: self.adapter.clone(),
        })
    }
    fn get_events<'a>(
        &'a self,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Pin<Box<dyn Stream<Item = CentralEvent> + Send>>,
                        BluetoothError,
                    >,
                > + Send
                + 'a,
        >,
    > {
        let adapter = self.adapter.clone();
        Box::pin(async move {
            let stream = adapter.events().await.map_err(BluetoothError::from)?;
            Ok(stream)
        })
    }
    fn get_peripheral<'a>(
        &'a self,
        address: &BDAddr,
    ) -> Pin<Box<dyn Future<Output = Result<Peripheral, BluetoothError>> + Send + 'a>> {
        let _adapter = self.adapter.clone();
        let _address = *address;
        Box::pin(async move {
            // TODO: Properly convert BDAddr to the required type for adapter.peripheral().
            panic!("get_peripheral: Conversion from BDAddr to the required type is not implemented. Update this code to support your platform.");
        })
    }
}

/// Configuration for Bluetooth scanner
#[derive(Debug, Clone)]
pub struct BleScannerConfig {
    /// Duration of each scan
    pub scan_duration: std::time::Duration,
    /// Interval between scans
    pub interval_between_scans: std::time::Duration,
    /// Whether to filter out known devices
    pub filter_known_devices: bool,
    /// Whether to only update RSSI for known devices
    pub update_rssi_only: bool,
    /// Interval for updating device data
    pub update_interval: std::time::Duration,
    /// Timeout for scanning
    pub scan_timeout: Option<std::time::Duration>,
    /// Maximum number of retries for operations
    pub max_retries: u8,
    /// Delay between retries
    pub retry_delay: std::time::Duration,
}

impl Default for BleScannerConfig {
    fn default() -> Self {
        Self {
            scan_duration: std::time::Duration::from_secs(10),
            interval_between_scans: std::time::Duration::from_secs(30),
            filter_known_devices: false,
            update_rssi_only: false,
            update_interval: std::time::Duration::from_secs(5),
            scan_timeout: None,
            max_retries: 3,
            retry_delay: std::time::Duration::from_millis(500),
        }
    }
}

/// A discovered Bluetooth device
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiscoveredDevice {
    /// Device address
    #[serde(with = "bdaddr_serde")]
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
    #[serde(skip, default = "std::time::Instant::now")]
    pub last_seen: std::time::Instant,
    /// Whether the device is connected
    pub is_connected: bool,
    /// Service data from the device
    pub service_data: HashMap<Uuid, Vec<u8>>,
    /// Services advertised by the device
    pub services: Vec<Uuid>,
    /// Transmit power level if available
    pub tx_power_level: Option<i16>,
}

// Custom serialization for BDAddr
mod bdaddr_serde {
    use btleplug::api::BDAddr;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(bdaddr: &BDAddr, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Convert BDAddr to string for serialization
        let addr_str = bdaddr.to_string();
        addr_str.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<BDAddr, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        // Deserialize from string
        let addr_str = String::deserialize(deserializer)?;

        // Parse back to BDAddr
        let bytes: Vec<&str> = addr_str.split(':').collect();

        if bytes.len() != 6 {
            return Err(D::Error::custom(format!(
                "Invalid BDAddr format: {}",
                addr_str
            )));
        }

        let mut addr = [0u8; 6];
        for (i, byte) in bytes.iter().enumerate() {
            addr[i] = u8::from_str_radix(byte, 16)
                .map_err(|e| D::Error::custom(format!("Invalid hex byte '{}': {}", byte, e)))?;
        }

        Ok(BDAddr::from(addr))
    }
}

impl DiscoveredDevice {
    /// Create a new discovered device from a peripheral
    pub async fn from_peripheral(peripheral: &Peripheral) -> Result<Self, BluetoothError> {
        // Create error context
        let _ctx = ErrorContext::new("BleScanner", "from_peripheral")
            .with_metadata("peripheral_address", peripheral.address().to_string());

        // Use ? operator for error propagation with automatic conversion via From trait
        let properties = peripheral.properties().await.map_err(|e| {
            let bt_err: BluetoothError = e.into();
            bt_err
        })?;

        let address = peripheral.address();

        // Ensure the properties exist
        let properties = match properties {
            Some(props) => props,
            None => {
                return Err(BluetoothError::InvalidData(
                    "No device properties available".to_string(),
                ))
            }
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
            is_connected: false,
            service_data: HashMap::new(),
            services: Vec::new(),
            tx_power_level: properties.tx_power_level,
        });

        Ok(Self {
            address,
            name: properties.local_name,
            rssi: properties.rssi,
            manufacturer_data,
            is_potential_airpods,
            last_seen: std::time::Instant::now(),
            is_connected: false,
            service_data: HashMap::new(),
            services: Vec::new(),
            tx_power_level: properties.tx_power_level,
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
            is_connected: false,
            service_data: HashMap::new(),
            services: Vec::new(),
            tx_power_level: None,
        }
    }
}

/// BLE scanner
///
/// Example usage:
/// ```rust,no_run
/// use std::sync::Arc;
/// use rustpods::bluetooth::{BleScanner, ScanConfig};
/// use rustpods::bluetooth::scanner::{RealAdapterEventsProvider, MockAdapterEventsProvider};
///
/// // In production (example with mock since we can't actually get a real adapter in doctest):
/// let provider = Arc::new(MockAdapterEventsProvider);
/// let scanner = BleScanner::new(provider, ScanConfig::default());
///
/// // In tests:
/// let scanner = BleScanner::new(Arc::new(MockAdapterEventsProvider), ScanConfig::default());
/// ```
pub struct BleScanner {
    /// The config for scanning
    config: ScanConfig,
    /// The Bluetooth adapter
    adapter: Option<Arc<Adapter>>,
    /// Whether scanning is in progress
    is_scanning: bool,
    /// Map of discovered devices by address
    devices: Arc<tokio::sync::Mutex<HashMap<BDAddr, DiscoveredDevice>>>,
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
    /// Events provider
    events_provider: Arc<dyn AdapterEventsProvider + Send + Sync>,
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
            events_provider: self.events_provider.clone(),
        }
    }
}

impl BleScanner {
    /// Create a new BLE scanner with the specified events provider and configuration
    pub fn new(
        events_provider: Arc<dyn AdapterEventsProvider + Send + Sync>,
        config: ScanConfig,
    ) -> Self {
        Self {
            config,
            adapter: None,
            is_scanning: false,
            devices: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            scan_task: None,
            cancel_sender: None,
            event_sender: None,
            event_broker: None,
            scan_cycles_completed: 0,
            events_provider,
        }
    }

    /// Create a new BLE scanner with a Bluetooth adapter and BleScannerConfig
    pub fn with_adapter_config(adapter: Arc<Adapter>, config: BleScannerConfig) -> Self {
        let provider = Arc::new(RealAdapterEventsProvider {
            adapter: adapter.clone(),
        });
        let scan_config = ScanConfig::default()
            .with_scan_duration(config.scan_duration)
            .with_interval(config.interval_between_scans)
            .with_auto_stop(!config.filter_known_devices)
            .with_continuous(false);
        Self::new(provider, scan_config)
    }

    /// Create a new BLE scanner with the specified configuration
    pub fn with_config(config: ScanConfig) -> Self {
        let provider = Arc::new(MockAdapterEventsProvider);
        Self::new(provider, config)
    }

    /// Update the scanner configuration
    pub fn set_config(&mut self, config: ScanConfig) {
        self.config = config;
    }

    /// Get the current scanner configuration
    pub fn get_config(&self) -> &ScanConfig {
        &self.config
    }

    /// Start scanning for devices
    #[tracing::instrument(name = "start_scanning", skip(self))]
    pub async fn start_scanning(&mut self) -> Result<Receiver<BleEvent>, BluetoothError> {
        println!("[BleScanner] start_scanning called");
        tracing::trace!(function = "start_scanning", "Entering start_scanning");
        let _ctx = ErrorContext::new("BleScanner", "start_scanning");
        info!("BleScanner::start_scanning called");

        // Check if scanning is already in progress
        if self.is_scanning {
            warn!(
                "{}Scan already in progress, returning existing channel",
                _ctx
            );

            // If there's an existing event broker, subscribe to it
            if let Some(event_broker) = &mut self.event_broker {
                let (_, rx) = event_broker.subscribe(EventFilter::All);
                tracing::trace!(function = "start_scanning", "Exiting start_scanning");
                info!("BleScanner::start_scanning returning");
                return Ok(rx);
            } else {
                return Err(BluetoothError::Other(
                    "Scanner is marked as scanning but has no event broker".to_string(),
                ));
            }
        }

        // Get or initialize the adapter
        let adapter = self.get_or_init_adapter().await?;
        println!("[BleScanner] Adapter initialized");

        // Create channels
        let (event_tx, _event_rx) = channel(100);
        let (cancel_tx, cancel_rx) = channel(1);

        // Store the transmitters
        self.event_sender = Some(event_tx.clone());
        self.cancel_sender = Some(cancel_tx);

        // Start the scan task
        match self.start_scan_task(event_tx, cancel_rx, adapter).await {
            Ok(task) => {
                // Store the task handle
                self.scan_task = Some(task);

                // Mark as scanning
                self.is_scanning = true;

                // Initialize event broker if it doesn't exist
                if self.event_broker.is_none() {
                    self.event_broker = Some(EventBroker::new());
                }

                // Start the event broker
                let event_broker = self.event_broker.as_mut().unwrap();
                event_broker.start();

                // Subscribe to events
                let (_, rx) = event_broker.subscribe(EventFilter::All);
                tracing::trace!(function = "start_scanning", "Exiting start_scanning");
                info!("BleScanner::start_scanning returning");
                Ok(rx)
            }
            Err(e) => {
                // Clean up
                self.event_sender = None;
                self.cancel_sender = None;

                // Return the error
                Err(e)
            }
        }
    }

    /// Get or initialize the Bluetooth adapter with retry logic
    #[tracing::instrument(name = "get_or_init_adapter", skip(self))]
    async fn get_or_init_adapter(&mut self) -> Result<Arc<Adapter>, BluetoothError> {
        tracing::trace!(
            function = "get_or_init_adapter",
            "Entering get_or_init_adapter"
        );
        let _ctx = ErrorContext::new("BleScanner", "get_or_init_adapter");

        info!("BleScanner::get_or_init_adapter called");

        // If we already have an adapter, return it
        if let Some(adapter) = &self.adapter {
            tracing::trace!(
                function = "get_or_init_adapter",
                "Exiting get_or_init_adapter"
            );
            info!("BleScanner::get_or_init_adapter returning");
            return Ok(adapter.clone());
        }

        // Try to initialize the adapter with retries
        let mut attempts = 0;
        let max_attempts = self.config.max_retries;

        while attempts < max_attempts {
            attempts += 1;
            debug!(
                "{}Initializing adapter (attempt {}/{})",
                _ctx, attempts, max_attempts
            );

            match self.try_initialize().await {
                Ok(()) => {
                    if let Some(adapter) = &self.adapter {
                        info!("{}Successfully initialized adapter", _ctx);
                        tracing::trace!(
                            function = "get_or_init_adapter",
                            "Exiting get_or_init_adapter"
                        );
                        info!("BleScanner::get_or_init_adapter returning");
                        return Ok(adapter.clone());
                    }
                }
                Err(e) => {
                    if self.is_error_retryable(&e) && attempts < max_attempts {
                        warn!("{}Adapter initialization failed with retryable error: {}. Retrying ({}/{})", 
                            _ctx, e, attempts, max_attempts);
                        sleep(self.config.retry_delay).await;
                        continue;
                    } else {
                        error!(
                            "{}Failed to initialize adapter after {} attempts: {}",
                            _ctx, attempts, e
                        );
                        tracing::trace!(
                            function = "get_or_init_adapter",
                            "Exiting get_or_init_adapter"
                        );
                        info!("BleScanner::get_or_init_adapter returning");
                        return Err(BluetoothError::AdapterNotAvailable {
                            reason: format!("Failed to initialize adapter: {}", e),
                            recovery: RecoveryAction::SelectDifferentAdapter,
                        });
                    }
                }
            }
        }

        // This should not be reached due to the returns in the loop
        Err(BluetoothError::NoAdapter)
    }

    /// Start the scan task
    #[tracing::instrument(name = "start_scan_task", skip(self, event_tx, cancel_rx, adapter))]
    async fn start_scan_task(
        &self,
        event_tx: Sender<BleEvent>,
        mut cancel_rx: Receiver<()>,
        adapter: Arc<Adapter>,
    ) -> Result<JoinHandle<()>, BluetoothError> {
        tracing::trace!(function = "start_scan_task", "Entering start_scan_task");
        let _ctx = ErrorContext::new("BleScanner", "start_scan_task");
        debug!(
            "{}Starting scan task with interval {:?}",
            _ctx, self.config.scan_duration
        );
        println!("[BleScanner] start_scan_task: scan task starting");

        // Get a clone of the devices map
        let _devices = self.devices.clone();

        // Create a filter for scanning
        let filter = ScanFilter::default();

        // Create a clone of the config
        let config = self.config.clone();

        // Use a shared event stream
        let event_stream = match adapter.events().await {
            Ok(stream) => {
                println!("[BleScanner] Event stream created successfully");
                info!("[BleScanner] Event stream created successfully");
                stream
            }
            Err(e) => {
                error!("{}Failed to get event stream: {}", _ctx, e);
                return Err(BluetoothError::ApiError(format!(
                    "Failed to get event stream: {}",
                    e
                )));
            }
        };

        // When spawning the scan task:
        let task_span = tracing::info_span!("scan_task", function = "start_scan_task");
        let task = tokio::spawn(async move {
            let _enter = task_span.enter();
            info!("Scan task started");
            println!("[BleScanner] Scan task started");
            let mut interval = interval(config.scan_duration);
            let scan_timeout = config.scan_timeout;
            let mut scan_count = 0;
            let mut event_stream = event_stream;
            let inner_ctx = ErrorContext::new("BleScanner", "scan_task");
            loop {
                if cancel_rx.try_recv().is_ok() {
                    debug!("{}Scan task cancelled", inner_ctx);
                    println!("[BleScanner] Scan task cancelled");
                    break;
                }
                interval.tick().await;
                scan_count += 1;
                info!("Scan cycle {} started", scan_count);
                println!("[BleScanner] Scan cycle {} started", scan_count);
                let scan_result = match scan_timeout {
                    Some(timeout_duration) => {
                        match tokio::time::timeout(
                            timeout_duration,
                            adapter.start_scan(filter.clone()),
                        )
                        .await
                        {
                            Ok(result) => {
                                println!("[BleScanner] adapter.start_scan returned: {:?}", result);
                                info!("[BleScanner] adapter.start_scan returned: {:?}", result);
                                result
                            }
                            Err(_) => {
                                warn!("{}Scan timed out after {:?}", inner_ctx, timeout_duration);
                                info!("Scan cycle {} ended (timeout)", scan_count);
                                continue;
                            }
                        }
                    }
                    None => {
                        let result = adapter.start_scan(filter.clone()).await;
                        println!("[BleScanner] adapter.start_scan returned: {:?}", result);
                        info!("[BleScanner] adapter.start_scan returned: {:?}", result);
                        result
                    }
                };
                if let Err(_e) = scan_result {
                    error!("{}Failed to start scan: {}", inner_ctx, _e);
                    println!("[BleScanner] Failed to start scan: {}", _e);
                    let _ = event_tx
                        .send(BleEvent::Error(format!("Failed to start scan: {}", _e)))
                        .await;
                    println!(
                        "BleScanner::scan_task: sending event: {:?}",
                        BleEvent::Error(format!("Failed to start scan: {}", _e))
                    );
                    info!("Scan cycle {} ended (error)", scan_count);
                    continue;
                }
                debug!(
                    "{}Scan started successfully, processing events...",
                    inner_ctx
                );
                println!("[BleScanner] Scan started successfully, processing events...");
                while let Ok(Some(event)) =
                    tokio::time::timeout(Duration::from_millis(100), event_stream.next()).await
                {
                    match &event {
                        CentralEvent::DeviceDiscovered(address) => {
                            println!("[BleScanner] CentralEvent::DeviceDiscovered: {:?}", address);
                        }
                        CentralEvent::DeviceUpdated(address) => {
                            println!("[BleScanner] CentralEvent::DeviceUpdated: {:?}", address);
                        }
                        CentralEvent::DeviceConnected(address) => {
                            println!("[BleScanner] CentralEvent::DeviceConnected: {:?}", address);
                        }
                        CentralEvent::DeviceDisconnected(address) => {
                            println!(
                                "[BleScanner] CentralEvent::DeviceDisconnected: {:?}",
                                address
                            );
                        }
                        _ => {
                            println!("[BleScanner] CentralEvent (other): {:?}", event);
                        }
                    }
                }
                info!("Scan cycle {} ended", scan_count);
                if let Err(e) = adapter.stop_scan().await {
                    warn!("{}Failed to stop scan: {}", inner_ctx, e);
                }
            }
            debug!("{}Scan task ending, stopping scan...", inner_ctx);
            if let Err(e) = adapter.stop_scan().await {
                warn!("{}Failed to stop scan during cleanup: {}", inner_ctx, e);
            }
            let _ = event_tx.send(BleEvent::ScanStopped).await;
            info!("Event sent: {:?}", BleEvent::ScanStopped);
            println!("BleScanner::scan_task: sent event");
            info!("BleScanner::scan_task: sent event");
            debug!("{}Scan task completed", inner_ctx);
            info!("Scan task ended");
            tracing::trace!(
                function = "start_scan_task",
                "Exiting scan_task async block"
            );
        });

        tracing::trace!(function = "start_scan_task", "Exiting start_scan_task");
        Ok(task)
    }

    /// Stop scanning for devices
    #[tracing::instrument(name = "stop_scanning", skip(self))]
    pub async fn stop_scanning(&mut self) -> Result<(), BluetoothError> {
        tracing::trace!(function = "stop_scanning", "Entering stop_scanning");
        let _ctx = ErrorContext::new("BleScanner", "stop_scanning");

        if !self.is_scanning {
            debug!("{}Scan not started, ignoring stop_scanning request", _ctx);
            tracing::trace!(function = "stop_scanning", "Exiting stop_scanning");
            return Ok(());
        }

        // Send the cancel signal to the scan task
        if let Some(sender) = &self.cancel_sender {
            if sender.send(()).await.is_err() {
                // The scan task has already terminated
                debug!(
                    "{}Failed to send cancel signal, scan task likely already terminated",
                    _ctx
                );
            }
        }

        // Wait for the task to complete
        if let Some(task) = self.scan_task.take() {
            debug!("{}Waiting for scan task to complete", _ctx);
            match task.await {
                Ok(_) => {
                    debug!("{}Scan task completed successfully", _ctx);
                }
                Err(e) => {
                    warn!("{}Error waiting for scan task to complete: {}", _ctx, e);
                    // Continue anyway, as we want to clean up
                }
            }
        }

        // Reset the state
        self.is_scanning = false;
        self.cancel_sender = None;
        self.event_sender = None;

        // Publish a scan stopped event
        self.event_broker().publish_event(BleEvent::ScanStopped);

        info!("{}Scan successfully stopped", _ctx);
        tracing::trace!(function = "stop_scanning", "Exiting stop_scanning");
        Ok(())
    }

    /// Get a list of currently known devices
    pub async fn get_devices(&self) -> Vec<DiscoveredDevice> {
        let devices = self.devices.lock().await;
        devices.values().cloned().collect()
    }

    /// Get potential AirPods devices from discovered devices
    pub async fn get_potential_airpods(&self) -> Vec<DiscoveredDevice> {
        let devices = self.devices.lock().await;
        devices
            .values()
            .filter(|d| d.is_potential_airpods)
            .cloned()
            .collect()
    }

    /// Get fully detected AirPods devices with battery information
    pub async fn get_detected_airpods(&self) -> Vec<DetectedAirPods> {
        let devices = self.devices.lock().await;
        devices
            .values()
            .filter_map(|d| detect_airpods(d).ok().flatten())
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
    pub async fn clear_devices(&mut self) {
        let mut devices = self.devices.lock().await;
        devices.clear();
    }

    /// Get a device by address if it exists
    pub async fn get_device(&self, address: &BDAddr) -> Option<DiscoveredDevice> {
        let devices = self.devices.lock().await;
        devices.get(address).cloned()
    }

    /// Process a discovered device (static version)
    #[tracing::instrument(name = "process_discovered_device", skip(devices, event_tx, _config), fields(address = %device.address))]
    async fn process_discovered_device(
        device: &DiscoveredDevice,
        devices: &Arc<tokio::sync::Mutex<HashMap<BDAddr, DiscoveredDevice>>>,
        event_tx: &Sender<BleEvent>,
        _config: &ScanConfig,
    ) -> Result<(), BluetoothError> {
        tracing::trace!(function = "process_discovered_device", address = %device.address, "Entering process_discovered_device");
        let _ctx = ErrorContext::new("BleScanner", "process_discovered_device")
            .with_metadata("address", device.address.to_string());

        // Lock the devices map
        let mut devices_map = devices.lock().await;

        // Check if this is a new device or an update
        let is_new = !devices_map.contains_key(&device.address);

        // Determine if we should send an update event
        let send_event = if is_new {
            // Always send event for new devices
            true
        } else if let Some(existing) = devices_map.get(&device.address) {
            // For existing devices, check if we should update
            // Only update if RSSI or manufacturer data changes
            device.rssi != existing.rssi || device.manufacturer_data != existing.manufacturer_data
        } else {
            false
        };

        // Update the device in our map
        devices_map.insert(device.address, device.clone());

        // Send the appropriate event
        if send_event {
            debug!("{}Sending device event for {}", _ctx, device.address);
            if is_new {
                event_tx
                    .send(BleEvent::DeviceDiscovered(device.clone()))
                    .await
                    .map_err(|_| {
                        BluetoothError::Other("Failed to send device discovered event".to_string())
                    })?;
                println!("BleScanner::process_discovered_device: sending event");
                info!("BleScanner::process_discovered_device: sending event");
            } else {
                event_tx
                    .send(BleEvent::DeviceUpdated(device.clone()))
                    .await
                    .map_err(|_| {
                        BluetoothError::Other("Failed to send device updated event".to_string())
                    })?;
                println!("BleScanner::process_discovered_device: sending event");
                info!("BleScanner::process_discovered_device: sending event");
            }
        }

        tracing::trace!(function = "process_discovered_device", address = %device.address, "Exiting process_discovered_device");
        Ok(())
    }

    /// Get filtered AirPods devices matching a filter
    pub async fn get_filtered_airpods(
        &self,
        filter: &crate::airpods::AirPodsFilter,
    ) -> Vec<DiscoveredDevice> {
        let devices = self.get_devices().await;
        devices.into_iter().filter(|d| filter(d)).collect()
    }

    /// Get detected AirPods devices matching a filter
    pub async fn get_filtered_detected_airpods(
        &self,
        filter: &crate::airpods::AirPodsFilter,
    ) -> Vec<DetectedAirPods> {
        let devices = self.get_devices().await;
        devices
            .into_iter()
            .filter(|d| filter(d))
            .filter_map(|d| detect_airpods(&d).ok().flatten())
            .collect()
    }

    /// Check if there are any AirPods matching the filter
    pub async fn has_airpods_matching(&self, filter: &crate::airpods::AirPodsFilter) -> bool {
        let devices = self.get_devices().await;
        devices.iter().any(filter)
    }

    /// Filter devices using an AirPods filter
    pub async fn filter_devices(
        &self,
        filter: &crate::airpods::AirPodsFilter,
    ) -> Vec<DiscoveredDevice> {
        let devices = self.get_devices().await;
        devices.into_iter().filter(|d| filter(d)).collect()
    }

    /// Get or create an event broker
    fn event_broker(&mut self) -> &mut EventBroker {
        if self.event_broker.is_none() {
            self.event_broker = Some(EventBroker::new());
        }
        self.event_broker.as_mut().unwrap()
    }

    /// Subscribe to all events
    pub fn subscribe_all(&mut self) -> Receiver<BleEvent> {
        let (_, rx) = self.event_broker().subscribe(EventFilter::All);
        rx
    }

    /// Subscribe to events with a filter
    pub fn subscribe(&mut self, filter: EventFilter) -> Receiver<BleEvent> {
        let (_, rx) = self.event_broker().subscribe(filter);
        rx
    }

    /// Subscribe to AirPods events only
    pub fn subscribe_airpods(&mut self) -> Receiver<BleEvent> {
        let (_, rx) = self.event_broker().subscribe(EventFilter::airpods_only());
        rx
    }

    /// Get peripherals by Bluetooth address
    pub async fn get_peripherals_by_address(
        &self,
        address: &BDAddr,
    ) -> Result<Vec<Peripheral>, BluetoothError> {
        let _ctx = ErrorContext::new("BleScanner", "get_peripherals_by_address")
            .with_metadata("address", address.to_string());

        // Make sure we have an adapter
        let adapter = if let Some(adapter) = &self.adapter {
            adapter.clone()
        } else {
            log::error!("{}No adapter initialized", _ctx);
            return Err(BluetoothError::NoAdapter);
        };

        // Get all peripherals with proper error handling
        let peripherals = adapter.peripherals().await.map_err(|e| {
            log::error!("{}Failed to get peripherals: {}", _ctx, e);
            BluetoothError::from(e)
        })?;

        // Filter by address
        let matching_devices = peripherals
            .into_iter()
            .filter(|peripheral| peripheral.address() == *address)
            .collect::<Vec<_>>();

        Ok(matching_devices)
    }

    /// Start scanning with a specific configuration
    pub async fn start_scanning_with_config(
        &mut self,
        config: ScanConfig,
    ) -> Result<Receiver<BleEvent>, BluetoothError> {
        let _ctx = ErrorContext::new("BleScanner", "start_scanning_with_config");

        // Similar to start_scanning but with custom config
        if self.scan_task.is_some() {
            log::warn!("{}Scan already in progress", _ctx);
            return Err(BluetoothError::ScanFailed(
                "Scan already in progress".to_string(),
            ));
        }

        // Make sure we have an adapter
        let adapter = if let Some(adapter) = &self.adapter {
            adapter.clone()
        } else {
            // Try to initialize
            log::info!("{}No adapter available, attempting to initialize", _ctx);
            self.initialize().await?;

            // Get the adapter
            self.adapter
                .as_ref()
                .ok_or_else(|| {
                    log::error!("{}Failed to initialize adapter", _ctx);
                    BluetoothError::NoAdapter
                })?
                .clone()
        };

        // Create channels for communication
        let (tx, rx) = channel(100);
        self.event_sender = Some(tx.clone());

        // Create a channel for cancellation
        let (cancel_tx, cancel_rx) = channel::<()>(1);
        self.cancel_sender = Some(cancel_tx);

        // Use the provided config
        self.config = config.clone();

        // Start scanning task with proper error handling
        let scan_task = self.start_scan_task(tx, cancel_rx, adapter).await?;
        self.scan_task = Some(scan_task);
        self.is_scanning = true;

        log::info!(
            "{}Scanning started successfully with custom configuration",
            _ctx
        );
        Ok(rx)
    }

    /// Get a reference to this scanner as a Configurable trait object
    pub fn as_configurable(&mut self) -> &mut dyn Configurable {
        self
    }

    /// Initialize the scanner
    #[tracing::instrument(name = "initialize", skip(self))]
    pub async fn initialize(&mut self) -> Result<(), BluetoothError> {
        tracing::trace!(function = "initialize", "Entering initialize");
        let _ctx = ErrorContext::new("BleScanner", "initialize");
        let max_retries = 3; // Default retry count
        let retry_delay = Duration::from_millis(500); // Default delay between retries

        // Try to initialize with retries for transient failures
        for attempt in 0..=max_retries {
            match self.try_initialize().await {
                Ok(_) => {
                    if attempt > 0 {
                        debug!(
                            "Successfully initialized Bluetooth adapter after {} retries",
                            attempt
                        );
                    }
                    tracing::trace!(function = "initialize", "Exiting initialize");
                    return Ok(());
                }
                Err(e) if attempt < max_retries && self.is_error_retryable(&e) => {
                    warn!(
                        "Bluetooth initialization attempt {} failed: {}. Retrying...",
                        attempt + 1,
                        e
                    );
                    sleep(retry_delay).await;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }

        // Should never reach here due to the loop structure, but the compiler needs this
        Err(BluetoothError::NoAdapter)
    }

    // Helper method that actually attempts the initialization
    #[tracing::instrument(name = "try_initialize", skip(self))]
    async fn try_initialize(&mut self) -> Result<(), BluetoothError> {
        tracing::trace!(function = "try_initialize", "Entering try_initialize");
        let _ctx = ErrorContext::new("BleScanner", "try_initialize");

        // Create a manager
        let manager = Manager::new().await.map_err(BluetoothError::from)?;

        // Get the adapter list
        let adapters = manager.adapters().await.map_err(BluetoothError::from)?;

        // Log all discovered adapters
        println!("[BleScanner] Discovered {} adapters", adapters.len());
        info!("[BleScanner] Discovered {} adapters", adapters.len());
        for (i, adapter) in adapters.iter().enumerate() {
            let addr = adapter
                .adapter_info()
                .await
                .unwrap_or_else(|_| "<unknown>".to_string());
            println!("[BleScanner] Adapter {}: info={}", i, addr);
            info!("[BleScanner] Adapter {}: info={}", i, addr);
        }

        // Find the first adapter that can be used
        if let Some(adapter) = adapters.into_iter().next() {
            self.adapter = Some(Arc::new(adapter));

            // Initialize the event broker if needed
            if self.event_broker.is_none() {
                self.event_broker = Some(EventBroker::new());
            }

            Ok(())
        } else {
            Err(BluetoothError::NoAdapter)
        }
    }

    // Helper method to check if an error is retryable
    fn is_error_retryable(&self, error: &BluetoothError) -> bool {
        match error {
            BluetoothError::ConnectionFailed(_) => true,
            BluetoothError::ScanFailed(_) => true,
            BluetoothError::DeviceDisconnected(_) => true,
            BluetoothError::NoAdapter => false, // Adapter missing is not retryable without user action
            BluetoothError::ApiError(_) => true, // API errors might be transient
            BluetoothError::InvalidData(_) => false, // Data validation errors aren't retryable
            BluetoothError::DeviceNotFound(_) => false, // Missing device not retryable
            BluetoothError::PermissionDenied(_) => false, // Permission issues need user intervention
            BluetoothError::Timeout(_) => true,
            BluetoothError::Other(_) => true, // Generic errors, attempt to retry
            BluetoothError::AdapterRefreshFailed { .. } => true,
            BluetoothError::AdapterNotAvailable { .. } => false, // Adapter unavailable needs user action
            BluetoothError::AdapterScanFailed { .. } => true,
        }
    }

    /// Create a dummy scanner that does nothing (for non-BLE builds)
    pub fn dummy() -> Self {
        struct DummyProvider;
        impl AdapterEventsProvider for DummyProvider {
            fn clone_box(&self) -> Box<dyn AdapterEventsProvider> {
                Box::new(DummyProvider)
            }
            fn get_events<'a>(
                &'a self,
            ) -> std::pin::Pin<
                Box<
                    dyn futures::Future<
                            Output = Result<
                                std::pin::Pin<
                                    Box<
                                        dyn futures::Stream<Item = btleplug::api::CentralEvent>
                                            + Send,
                                    >,
                                >,
                                BluetoothError,
                            >,
                        > + Send
                        + 'a,
                >,
            > {
                Box::pin(async {
                    Ok(Box::pin(futures::stream::empty())
                        as std::pin::Pin<
                            Box<dyn futures::Stream<Item = btleplug::api::CentralEvent> + Send>,
                        >)
                })
            }
            fn get_peripheral<'a>(
                &'a self,
                _address: &btleplug::api::BDAddr,
            ) -> std::pin::Pin<
                Box<
                    dyn futures::Future<
                            Output = Result<btleplug::platform::Peripheral, BluetoothError>,
                        > + Send
                        + 'a,
                >,
            > {
                Box::pin(async { panic!("DummyProvider::get_peripheral not implemented") })
            }
        }
        let provider = std::sync::Arc::new(DummyProvider);
        Self::new(
            provider,
            crate::bluetooth::scanner_config::ScanConfig::default(),
        )
    }
}

impl Configurable for BleScanner {
    fn apply_config(&mut self, config: &AppConfig) {
        // Apply bluetooth-related settings to our ScanConfig
        let mut scan_config = ScanConfig::default();
        scan_config = scan_config.with_scan_duration(config.bluetooth.scan_duration);
        scan_config = scan_config.with_interval(config.bluetooth.scan_interval);
        scan_config = scan_config.with_min_rssi(config.bluetooth.min_rssi);

        // Set a reasonable default for max cycles
        scan_config = scan_config.with_max_cycles(Some(5));

        self.set_config(scan_config);
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

/// Parse a BDAddr from a string
#[allow(dead_code)]
pub fn parse_bdaddr(s: &str) -> Result<BDAddr, String> {
    let bytes: Vec<&str> = s.split(':').collect();

    if bytes.len() != 6 {
        return Err(format!("Invalid BDAddr format: {}", s));
    }

    let mut addr = [0u8; 6];
    for (i, byte) in bytes.iter().enumerate() {
        addr[i] = u8::from_str_radix(byte, 16)
            .map_err(|e| format!("Invalid hex byte '{}': {}", byte, e))?;
    }

    Ok(BDAddr::from(addr))
}

pub struct MockAdapterEventsProvider;

impl AdapterEventsProvider for MockAdapterEventsProvider {
    fn clone_box(&self) -> Box<dyn AdapterEventsProvider> {
        Box::new(MockAdapterEventsProvider)
    }
    fn get_events<'a>(
        &'a self,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Pin<Box<dyn Stream<Item = CentralEvent> + Send>>,
                        BluetoothError,
                    >,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async {
            Ok(Box::pin(futures::stream::empty())
                as Pin<Box<dyn Stream<Item = CentralEvent> + Send>>)
        })
    }
    fn get_peripheral<'a>(
        &'a self,
        _address: &BDAddr,
    ) -> Pin<Box<dyn Future<Output = Result<Peripheral, BluetoothError>> + Send + 'a>> {
        Box::pin(async { panic!("MockAdapterEventsProvider::get_peripheral not implemented") })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bluetooth::scanner_config::ScanConfig;

    #[test]
    fn test_parse_bdaddr_valid() {
        let addr_str = "12:34:56:78:9A:BC";
        let result = parse_bdaddr(addr_str);
        assert!(result.is_ok());

        let addr = result.unwrap();
        assert_eq!(addr, BDAddr::from([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]));
    }

    #[test]
    fn test_parse_bdaddr_invalid_format() {
        // Too few segments
        let result = parse_bdaddr("12:34:56:78:9A");
        assert!(result.is_err());

        // Invalid hex
        let result = parse_bdaddr("12:34:56:78:9A:ZZ");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_scanner_new() {
        let scanner = BleScanner::new(Arc::new(MockAdapterEventsProvider), ScanConfig::default());
        assert!(!scanner.is_scanning());
        assert_eq!(scanner.get_scan_cycles(), 0);
        assert!(scanner.get_devices().await.is_empty());
    }

    #[test]
    fn test_scanner_with_config() {
        let config = ScanConfig::default();
        let scanner = BleScanner::new(Arc::new(MockAdapterEventsProvider), config);
        assert_eq!(scanner.get_scan_cycles(), 0);
        assert!(!scanner.is_scanning());
        // Note: Not testing get_devices().is_empty() here as it would require async
    }

    #[tokio::test]
    async fn test_device_list_operations() {
        let mut scanner =
            BleScanner::new(Arc::new(MockAdapterEventsProvider), ScanConfig::default());

        // Initially empty
        assert!(scanner.get_devices().await.is_empty());

        // Clear should work even when empty
        scanner.clear_devices().await;
        assert!(scanner.get_devices().await.is_empty());
    }

    #[test]
    fn test_error_context_creation() {
        let ctx = ErrorContext::new("TestComponent", "test_operation");
        assert_eq!(ctx.component, "TestComponent");
        assert_eq!(ctx.operation, "test_operation");

        let ctx_with_data = ctx.with_metadata("test_key", "test_value");
        assert!(ctx_with_data.metadata.contains_key("test_key"));
        assert_eq!(
            ctx_with_data.metadata.get("test_key").unwrap(),
            "test_value"
        );
    }

    #[test]
    fn test_recovery_action_properties() {
        let recovery = RecoveryAction::Retry;
        assert_eq!(recovery.description(), "Retry the operation");

        let recovery = RecoveryAction::RestartApplication;
        assert_eq!(recovery.description(), "Restart the application");
    }

    #[tokio::test]
    async fn test_retry_logic_for_transient_errors() {
        use std::sync::{Arc, Mutex};
        use std::time::Duration;

        // Create a simple mock struct to track retry attempts
        struct RetryTracker {
            attempts: usize,
            should_succeed_after: usize,
            succeeded: bool,
        }

        impl RetryTracker {
            fn new(succeed_after: usize) -> Self {
                Self {
                    attempts: 0,
                    should_succeed_after: succeed_after,
                    succeeded: false,
                }
            }

            fn attempt_operation(&mut self) -> Result<(), BluetoothError> {
                self.attempts += 1;

                if self.attempts >= self.should_succeed_after {
                    self.succeeded = true;
                    Ok(())
                } else {
                    Err(BluetoothError::Timeout(Duration::from_millis(100)))
                }
            }
        }

        // Test case 1: Operation succeeds after retries
        let tracker = Arc::new(Mutex::new(RetryTracker::new(3)));
        let tracker_clone = tracker.clone();

        // Create a function that will use our retry mechanism
        async fn operation_with_retry(
            tracker: Arc<Mutex<RetryTracker>>,
            max_retries: usize,
        ) -> Result<(), BluetoothError> {
            let mut attempts = 0;
            let retry_delay = Duration::from_millis(10); // Short delay for testing

            loop {
                match tracker.lock().unwrap().attempt_operation() {
                    Ok(()) => return Ok(()),
                    Err(e) => {
                        attempts += 1;
                        if attempts >= max_retries {
                            return Err(e);
                        }
                        tokio::time::sleep(retry_delay).await;
                    }
                }
            }
        }

        // The operation should succeed with enough retries
        let result = operation_with_retry(tracker_clone, 5).await;
        assert!(result.is_ok());
        assert!(tracker.lock().unwrap().succeeded);
        assert_eq!(tracker.lock().unwrap().attempts, 3);

        // Test case 2: Operation fails if max retries is too low
        let tracker = Arc::new(Mutex::new(RetryTracker::new(5)));
        let tracker_clone = tracker.clone();

        let result = operation_with_retry(tracker_clone, 3).await;
        assert!(result.is_err());

        // Should have made exactly the max number of attempts
        assert_eq!(tracker.lock().unwrap().attempts, 3);
        assert!(!tracker.lock().unwrap().succeeded);

        // Verify we got the correct error type
        match result {
            Err(BluetoothError::Timeout(_)) => { /* Success */ }
            _ => panic!("Wrong error type returned"),
        }
    }
}
