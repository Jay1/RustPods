use std::sync::Arc;
use std::fmt;
use std::default::Default;
use std::collections::HashMap;
use std::time::Duration;

use btleplug::api::{
    BDAddr, Central, Manager as _, 
    ScanFilter, Peripheral as _,
};
use btleplug::platform::{Adapter, Manager};
use tokio::time::sleep;

use crate::bluetooth::BleError;
use crate::bluetooth::scanner::DiscoveredDevice;

/// Adapter capabilities
#[derive(Debug, Clone, Default)]
pub struct AdapterCapabilities {
    /// Whether the adapter supports scanning
    pub supports_scanning: bool,
    
    /// Whether the adapter supports the central role
    pub supports_central_role: bool,
    
    /// Whether the adapter supports advertising
    pub supports_advertising: bool,
    
    /// Whether the adapter supports connecting to peripherals
    pub supports_connecting: bool,
    
    /// Whether the adapter is powered on
    pub is_powered_on: bool,
    
    /// Maximum number of connections supported
    pub max_connections: u8,
    
    /// When the capabilities were last checked
    pub last_checked: Option<std::time::Instant>,
    
    /// Adapter status
    pub status: AdapterStatus,
    
    /// Adapter information string
    pub adapter_info: String,
}

/// Adapter status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterStatus {
    /// Normal operation
    Normal,
    /// Adapter is ready
    Ready,
    /// Adapter is in error state
    Error,
    /// Adapter is disabled
    Disabled,
    /// Adapter is busy
    Busy,
}

impl Default for AdapterStatus {
    fn default() -> Self {
        Self::Normal
    }
}

/// Information about a Bluetooth adapter
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct AdapterInfo {
    /// Adapter index
    pub index: usize,
    /// Adapter address if available
    pub address: Option<BDAddr>,
    /// Adapter name or identifier
    pub name: String,
    /// Whether this is the default adapter
    pub is_default: bool,
    /// Adapter capabilities
    pub capabilities: AdapterCapabilities,
    /// Adapter vendor/manufacturer if available
    pub vendor: Option<String>,
}

impl fmt::Display for AdapterInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}#{}: {}{}{}",
            self.name,
            self.index,
            match self.address {
                Some(addr) => addr.to_string(),
                None => "unknown address".to_string(),
            },
            if self.is_default { " (default)" } else { "" },
            match self.capabilities.status {
                AdapterStatus::Normal => "",
                AdapterStatus::Ready => " (ready)",
                AdapterStatus::Error => " (error)",
                AdapterStatus::Disabled => " (disabled)",
                AdapterStatus::Busy => " (busy)",
            }
        )
    }
}


/// Handle adapter discovery and selection
#[derive(Clone)]
pub struct AdapterManager {
    /// The system's Bluetooth manager
    manager: Arc<Manager>,
    /// List of available adapter info
    available_adapters: Vec<AdapterInfo>,
    /// Currently selected adapter index
    selected_index: Option<usize>,
    /// Adapter status history (address -> status history)
    adapter_history: HashMap<String, Vec<(std::time::Instant, AdapterStatus)>>,
}

impl AdapterManager {
    /// Create a new adapter manager
    pub async fn new() -> Result<Self, BleError> {
        let manager = Arc::new(Manager::new().await?);
        
        let mut adapter_manager = Self {
            manager,
            available_adapters: Vec::new(),
            selected_index: None,
            adapter_history: HashMap::new(),
        };
        
        // Discover available adapters
        adapter_manager.refresh_adapters().await?;
        
        // Auto-select the first adapter if available
        if !adapter_manager.available_adapters.is_empty() {
            adapter_manager.selected_index = Some(0);
        }
        
        Ok(adapter_manager)
    }
    
    /// Refresh the list of available adapters with retry logic
    pub async fn refresh_adapters(&mut self) -> Result<(), BleError> {
        // Retry up to 3 times with increasing delays
        let mut attempt = 0;
        let max_attempts = 3;
        let mut last_error = None;
        
        while attempt < max_attempts {
            match self.try_refresh_adapters().await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    attempt += 1;
                    last_error = Some(e);
                    
                    // Don't sleep on the last attempt
                    if attempt < max_attempts {
                        // Exponential backoff: 100ms, 500ms, 1000ms
                        let delay = Duration::from_millis(100 * (5_u64.pow(attempt)));
                        sleep(delay).await;
                    }
                }
            }
        }
        
        // If we get here, all attempts failed
        Err(last_error.unwrap_or(BleError::AdapterNotFound))
    }
    
    /// Try to refresh adapter list once
    async fn try_refresh_adapters(&mut self) -> Result<(), BleError> {
        let adapters = self.manager.adapters().await?;
        let mut adapter_infos = Vec::new();
        
        for (index, adapter) in adapters.iter().enumerate() {
            // Try to get the adapter's address (may not be available on all platforms)
            let address = Self::get_adapter_address(adapter).await;
            
            // Generate a unique identifier for history tracking
            let adapter_id = if let Some(addr) = address {
                addr.to_string()
            } else {
                format!("adapter_{}", index)
            };
            
            // Check adapter capabilities
            let capabilities = self.check_adapter_capabilities(adapter, &adapter_id).await;
            
            // Try to get vendor information
            let vendor = Self::get_adapter_vendor(adapter).await;
            
            let info = AdapterInfo {
                index,
                address,
                name: Self::get_adapter_name(adapter, index).await,
                is_default: index == 0, // First adapter is considered default
                capabilities,
                vendor,
            };
            
            adapter_infos.push(info);
        }
        
        self.available_adapters = adapter_infos;
        
        // Reset selection if the selected adapter is no longer available
        if let Some(index) = self.selected_index {
            if index >= self.available_adapters.len() {
                self.selected_index = None;
            }
        }
        
        Ok(())
    }
    
    /// Check adapter capabilities
    async fn check_adapter_capabilities(&mut self, adapter: &Adapter, adapter_id: &str) -> AdapterCapabilities {
        let now = std::time::Instant::now();
        let mut caps = AdapterCapabilities::default();
        caps.last_checked = Some(now);
        
        // Check if scanning is supported
        let scanning_supported = self.check_scanning_capability(adapter).await.unwrap_or(false);
        caps.supports_scanning = scanning_supported;
        
        // Check if connecting is supported by trying to get a list of connected peripherals
        match adapter.peripherals().await {
            Ok(_) => caps.supports_connecting = true,
            Err(_) => caps.supports_connecting = false,
        }
        
        // Try to determine power state (this is platform-dependent)
        caps.is_powered_on = true; // Assume powered on since we got this far
        
        // Determine adapter status based on capabilities
        let status = if !caps.supports_scanning {
            AdapterStatus::Error
        } else if !caps.is_powered_on {
            AdapterStatus::Disabled
        } else {
            AdapterStatus::Normal
        };
        
        caps.status = status;
        
        // Update adapter history
        self.update_adapter_history(adapter_id, status, now);
        
        caps
    }
    
    /// Update adapter status history
    fn update_adapter_history(&mut self, adapter_id: &str, status: AdapterStatus, timestamp: std::time::Instant) {
        let history = self.adapter_history.entry(adapter_id.to_string()).or_default();
        
        // Only add entry if status changed or it's been a while since the last update
        if history.is_empty() || 
           history.last().map(|(_, last_status)| *last_status != status).unwrap_or(true) ||
           history.last().map(|(last_time, _)| timestamp.duration_since(*last_time) > Duration::from_secs(300)).unwrap_or(true) {
            history.push((timestamp, status));
            
            // Keep history at a reasonable size
            if history.len() > 10 {
                history.remove(0);
            }
        }
    }
    
    /// Get the address of an adapter (if available)
    async fn get_adapter_address(_adapter: &Adapter) -> Option<BDAddr> {
        // This is not directly supported by btleplug in a cross-platform way
        // For Windows, we can try to get this from the adapter info in the future
        // For now, we return None as a proper implementation would be platform-specific
        None
    }
    
    /// Try to determine adapter vendor information
    async fn get_adapter_vendor(_adapter: &Adapter) -> Option<String> {
        // This information is not available through btleplug
        // Platform-specific code could be added here
        None
    }
    
    /// Get a name for the adapter based on platform-specific information
    async fn get_adapter_name(adapter: &Adapter, index: usize) -> String {
        // Try to get a better name for the adapter if possible
        // The adapter.identifier() method might provide useful information on some platforms
        if let Ok(info) = adapter.adapter_info().await {
            // Different BLE libraries might have different field names for the identifier
            // Using info as a string directly instead of trying to access a specific field
            return info.to_string();
        }
        
        // Fallback: use a generic name with the index
        format!("BluetoothAdapter{}", index)
    }
    
    /// Get the list of available adapters
    pub fn get_available_adapters(&self) -> &[AdapterInfo] {
        &self.available_adapters
    }
    
    /// Select an adapter by index
    pub fn select_adapter(&mut self, index: usize) -> Result<(), BleError> {
        if index >= self.available_adapters.len() {
            return Err(BleError::AdapterNotFound);
        }
        
        self.selected_index = Some(index);
        Ok(())
    }
    
    /// Select the best available adapter based on capabilities
    pub fn select_best_adapter(&mut self) -> Result<(), BleError> {
        if self.available_adapters.is_empty() {
            return Err(BleError::AdapterNotFound);
        }
        
        // Find the first adapter with normal status that supports scanning
        for (index, info) in self.available_adapters.iter().enumerate() {
            if info.capabilities.status == AdapterStatus::Normal && 
               info.capabilities.supports_scanning {
                self.selected_index = Some(index);
                return Ok(());
            }
        }
        
        // If no ideal adapter found, use the first one that supports scanning
        for (index, info) in self.available_adapters.iter().enumerate() {
            if info.capabilities.supports_scanning {
                self.selected_index = Some(index);
                return Ok(());
            }
        }
        
        // Last resort: use the first adapter regardless of capabilities
        self.selected_index = Some(0);
        Ok(())
    }
    
    /// Get the currently selected adapter
    pub async fn get_selected_adapter(&self) -> Result<Adapter, BleError> {
        let index = self.selected_index.ok_or(BleError::AdapterNotFound)?;
        
        let adapters = self.manager.adapters().await?;
        adapters.into_iter().nth(index).ok_or(BleError::AdapterNotFound)
    }
    
    /// Get info about the currently selected adapter
    pub fn get_selected_adapter_info(&self) -> Option<&AdapterInfo> {
        self.selected_index.and_then(|index| self.available_adapters.get(index))
    }
    
    /// Get adapter history for a specific adapter
    pub fn get_adapter_history(&self, adapter_id: &str) -> Option<&[(std::time::Instant, AdapterStatus)]> {
        self.adapter_history.get(adapter_id).map(|h| h.as_slice())
    }
    
    /// Check if the adapter supports scanning
    pub async fn check_scanning_capability(&self, adapter: &Adapter) -> Result<bool, BleError> {
        // Try to start a very brief scan to see if scanning is supported
        let result = adapter.start_scan(ScanFilter::default()).await;
        
        // If the scan started successfully, stop it immediately
        if result.is_ok() {
            let _ = adapter.stop_scan().await;
            return Ok(true);
        }
        
        // Check if the error indicates scanning is not supported
        match result {
            Err(btleplug::Error::NotSupported(_)) => Ok(false),
            Err(e) => Err(BleError::BtlePlugError(e.to_string())),
            _ => Ok(true),
        }
    }
    
    /// Attempt to recover an adapter that's in a troubled state
    pub async fn try_recover_adapter(&mut self, index: usize) -> Result<bool, BleError> {
        if index >= self.available_adapters.len() {
            return Err(BleError::AdapterNotFound);
        }
        
        let adapters = self.manager.adapters().await?;
        let adapter = adapters.into_iter().nth(index).ok_or(BleError::AdapterNotFound)?;
        
        // First try to check scanning capability again (it might have recovered on its own)
        let supports_scanning = self.check_scanning_capability(&adapter).await?;
        
        if supports_scanning {
            // Update adapter info with recovered status
            if let Some(info) = self.available_adapters.get_mut(index) {
                info.capabilities.supports_scanning = true;
                info.capabilities.status = AdapterStatus::Normal;
                info.capabilities.last_checked = Some(std::time::Instant::now());
                
                // Get a copy of the adapter ID before we need to borrow self mutably again
                let adapter_id = if let Some(addr) = info.address {
                    addr.to_string()
                } else {
                    format!("adapter_{}", index)
                };
                
                // Get a copy of the last_checked timestamp before releasing the borrow
                let timestamp = info.capabilities.last_checked.unwrap();
                
                // Now update adapter history with the copied values
                self.update_adapter_history(&adapter_id, AdapterStatus::Normal, timestamp);
            }
            
            return Ok(true);
        }
        
        // If checking the capability didn't resolve the issue, we'd need more advanced recovery
        // techniques that are platform-specific and may require system-level permissions.
        // For now, we'll just return false to indicate failure.
        
        Ok(false)
    }
    
    /// Get the adapter for the specified address
    pub async fn get_adapter_by_address(&self, address: BDAddr) -> Result<Adapter, BleError> {
        // Get the list of adapters
        let adapters = self.manager.adapters().await?;
        
        // Find the adapter with the specified address
        for adapter in adapters {
            if let Some(addr) = Self::get_adapter_address(&adapter).await {
                if addr == address {
                    return Ok(adapter);
                }
            }
        }
        
        // If we get here, no adapter was found
        Err(BleError::AdapterNotFound)
    }
    
    /// Check if Bluetooth is available
    pub async fn is_bluetooth_available(&self) -> Result<bool, BleError> {
        // Get the list of adapters
        let adapters = self.manager.adapters().await?;
        
        // Check if there are any adapters
        Ok(!adapters.is_empty())
    }
}

/// Bluetooth adapter event
#[derive(Debug, Clone)]
pub enum BleAdapterEvent {
    /// Device discovered event
    DeviceDiscovered(DiscoveredDevice),
    /// Device updated event
    DeviceUpdated(DiscoveredDevice),
    /// Scan started event
    ScanStarted,
    /// Scan stopped event
    ScanStopped,
    /// Error event
    Error(String),
}

/// Wrapper around btleplug Adapter for easier use
#[derive(Clone)]
pub struct BluetoothAdapter {
    /// The underlying adapter
    adapter: Arc<Adapter>,
    /// Last known status
    status: AdapterStatus,
    /// Adapter capabilities
    capabilities: AdapterCapabilities,
}

impl BluetoothAdapter {
    /// Create a new BluetoothAdapter
    pub async fn new() -> Result<Self, BleError> {
        // Create a manager
        let manager = Manager::new().await?;
        
        // Get the adapter list
        let adapters = manager.adapters().await?;
        
        // Find the first adapter
        let adapter = adapters.into_iter().next()
            .ok_or(BleError::AdapterNotFound)?;
        
        // Check if scanning is supported
        let mut capabilities = AdapterCapabilities::default();
        
        // Set capabilities (async scanning check is not ideal but we keep it simple)
        capabilities.supports_scanning = true; // Assume it supports scanning
        capabilities.supports_central_role = true;
        capabilities.supports_advertising = false; // Not using advertising for now
        capabilities.supports_connecting = true;
        capabilities.is_powered_on = true;
        capabilities.max_connections = 5; // Default value
        capabilities.last_checked = Some(std::time::Instant::now());
        capabilities.status = AdapterStatus::Normal;
        capabilities.adapter_info = format!("{:?}", adapter);
        
        Ok(Self {
            adapter: Arc::new(adapter),
            capabilities,
            status: AdapterStatus::Normal,
        })
    }
    
    /// Get the Bluetooth adapter address
    ///
    /// This method returns the address of the Bluetooth adapter if available
    pub async fn get_address(&self) -> Result<Option<BDAddr>, BleError> {
        // This is not directly supported by btleplug in a cross-platform way
        // For Windows, we can get this from the adapter properties
        // For other platforms, we might need different approaches
        
        // We just return None for now as a proper implementation would be platform-specific
        // A more robust implementation would use platform-specific code to get the address
        Ok(None)
    }
    
    /// Get the adapter capabilities
    pub fn get_capabilities(&self) -> &AdapterCapabilities {
        &self.capabilities
    }
    
    /// Get the status
    pub fn get_status(&self) -> AdapterStatus {
        self.status
    }
    
    /// Get the underlying adapter
    pub fn get_adapter(&self) -> Arc<Adapter> {
        self.adapter.clone()
    }
    
    /// Start scanning for devices
    pub async fn start_scan(&self) -> Result<Vec<DiscoveredDevice>, BleError> {
        // Start scanning
        self.adapter.start_scan(ScanFilter::default()).await?;
        
        // Return empty list initially
        Ok(Vec::new())
    }
    
    /// Stop scanning for devices
    pub async fn stop_scan(&self) -> Result<(), BleError> {
        self.adapter.stop_scan().await?;
        Ok(())
    }
    
    /// Get discovered devices
    pub async fn get_discovered_devices(&self) -> Result<Vec<DiscoveredDevice>, BleError> {
        let peripherals = self.adapter.peripherals().await?;
        
        // Convert peripherals to discovered devices
        let mut devices = Vec::new();
        
        for peripheral in peripherals {
            // Try to get properties
            if let Ok(properties) = peripheral.properties().await {
                if let Some(properties) = properties {
                    // Create discovered device
                    let device = DiscoveredDevice {
                        name: properties.local_name,
                        address: properties.address,
                        rssi: properties.rssi,
                        is_connected: false, // We'll check this later
                        is_potential_airpods: false, // We'll check this later
                        manufacturer_data: properties.manufacturer_data,
                        service_data: properties.service_data,
                        services: properties.services,
                        last_seen: std::time::Instant::now(),
                    };
                    
                    devices.push(device);
                }
            }
        }
        
        Ok(devices)
    }
} 