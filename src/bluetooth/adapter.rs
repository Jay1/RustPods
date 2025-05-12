use std::sync::Arc;
use std::fmt;
use std::default::Default;
use std::collections::HashMap;
use std::time::Duration;

use btleplug::api::{
    BDAddr, Central, Manager as _, 
    ScanFilter, Peripheral as _,
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use tokio::time::sleep;

use crate::bluetooth::BleError;
use crate::bluetooth::scanner::DiscoveredDevice;

/// Adapter capabilities
#[derive(Debug, Clone)]
pub struct AdapterCapabilities {
    /// Whether the adapter supports scanning
    pub supports_scanning: bool,
    /// Whether the adapter supports connecting to peripherals
    pub supports_connecting: bool,
    /// Whether the adapter is powered on
    pub is_powered_on: bool,
    /// Maximum number of concurrent connections
    pub max_connections: Option<usize>,
    /// Last time capabilities were checked
    pub last_checked: std::time::Instant,
    /// Last known status
    pub status: AdapterStatus,
}

impl Default for AdapterCapabilities {
    fn default() -> Self {
        Self {
            supports_scanning: false,
            supports_connecting: false,
            is_powered_on: false,
            max_connections: None,
            last_checked: std::time::Instant::now(),
            status: AdapterStatus::default(),
        }
    }
}

/// Status of an adapter
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterStatus {
    /// Adapter is working normally
    Normal,
    /// Adapter is having issues
    Troubled,
    /// Adapter is unavailable
    Unavailable,
    /// Adapter status is unknown
    Unknown,
}

impl Default for AdapterStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Information about a Bluetooth adapter
#[derive(Debug, Clone)]
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
                AdapterStatus::Troubled => " (troubled)",
                AdapterStatus::Unavailable => " (unavailable)",
                AdapterStatus::Unknown => " (unknown status)",
            }
        )
    }
}

impl Default for AdapterInfo {
    fn default() -> Self {
        Self {
            index: 0,
            name: String::new(),
            address: None,
            is_default: false,
            capabilities: AdapterCapabilities::default(),
            vendor: None,
        }
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
        caps.last_checked = now;
        
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
            AdapterStatus::Troubled
        } else if !caps.is_powered_on {
            AdapterStatus::Unavailable
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
        let history = self.adapter_history.entry(adapter_id.to_string()).or_insert_with(Vec::new);
        
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
    async fn get_adapter_address(adapter: &Adapter) -> Option<BDAddr> {
        // Note: This implementation is limited by btleplug's API
        // We currently can't reliably get adapter addresses on all platforms
        // For now, we'll return None, but platform-specific code could be added later
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
        match adapter.adapter_info().await {
            Ok(info) => {
                // Different BLE libraries might have different field names for the identifier
                // Using info as a string directly instead of trying to access a specific field
                return info.to_string();
            },
            Err(_) => {}
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
            Err(e) => Err(BleError::BtlePlugError(e)),
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
                info.capabilities.last_checked = std::time::Instant::now();
                
                // Get a copy of the adapter ID before we need to borrow self mutably again
                let adapter_id = if let Some(addr) = info.address {
                    addr.to_string()
                } else {
                    format!("adapter_{}", index)
                };
                
                // Get a copy of the last_checked timestamp before releasing the borrow
                let timestamp = info.capabilities.last_checked;
                
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
        let manager = Manager::new().await.map_err(BleError::BtlePlugError)?;
        
        // Get the adapter list
        let adapters = manager.adapters().await.map_err(BleError::BtlePlugError)?;
        
        // Find the first adapter
        let adapter = adapters.into_iter().next()
            .ok_or(BleError::AdapterNotFound)?;
        
        // Get adapter capabilities
        let capabilities = AdapterCapabilities::default();
        
        Ok(Self {
            adapter: Arc::new(adapter),
            status: AdapterStatus::Normal,
            capabilities,
        })
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
} 