//! Bluetooth device filtering functionality
//! 
//! Provides filters for Bluetooth scanning to identify specific devices

use std::collections::HashSet;
use btleplug::api::BDAddr;

use crate::bluetooth::DiscoveredDevice;

/// Filter for Bluetooth devices
pub trait DeviceFilter: Send + Sync {
    /// Apply the filter to a list of devices
    fn apply_filter(&self, devices: &[DiscoveredDevice]) -> Vec<DiscoveredDevice>;
    
    /// Check if a device passes the filter
    fn matches(&self, device: &DiscoveredDevice) -> bool;
}

/// Filter devices by name
pub struct NameFilter {
    /// Names to match (case-insensitive)
    names: HashSet<String>,
}

impl NameFilter {
    /// Create a new name filter
    pub fn new(names: Vec<String>) -> Self {
        Self {
            names: names.into_iter().map(|n| n.to_lowercase()).collect(),
        }
    }
}

impl DeviceFilter for NameFilter {
    fn apply_filter(&self, devices: &[DiscoveredDevice]) -> Vec<DiscoveredDevice> {
        devices.iter()
            .filter(|d| self.matches(d))
            .cloned()
            .collect()
    }
    
    fn matches(&self, device: &DiscoveredDevice) -> bool {
        if let Some(name) = &device.name {
            self.names.contains(&name.to_lowercase())
        } else {
            false
        }
    }
}

/// Filter devices by address
pub struct AddressFilter {
    /// Addresses to match
    addresses: HashSet<BDAddr>,
}

impl AddressFilter {
    /// Create a new address filter
    pub fn new(addresses: Vec<BDAddr>) -> Self {
        Self {
            addresses: addresses.into_iter().collect(),
        }
    }
}

impl DeviceFilter for AddressFilter {
    fn apply_filter(&self, devices: &[DiscoveredDevice]) -> Vec<DiscoveredDevice> {
        devices.iter()
            .filter(|d| self.matches(d))
            .cloned()
            .collect()
    }
    
    fn matches(&self, device: &DiscoveredDevice) -> bool {
        self.addresses.contains(&device.address)
    }
}

/// Filter devices by RSSI (signal strength)
pub struct RssiFilter {
    /// Minimum RSSI value
    min_rssi: i16,
}

impl RssiFilter {
    /// Create a new RSSI filter
    pub fn new(min_rssi: i16) -> Self {
        Self { min_rssi }
    }
}

impl DeviceFilter for RssiFilter {
    fn apply_filter(&self, devices: &[DiscoveredDevice]) -> Vec<DiscoveredDevice> {
        devices.iter()
            .filter(|d| self.matches(d))
            .cloned()
            .collect()
    }
    
    fn matches(&self, device: &DiscoveredDevice) -> bool {
        if let Some(rssi) = device.rssi {
            rssi >= self.min_rssi
        } else {
            false
        }
    }
}

/// Composite filter that combines multiple filters (AND logic)
pub struct CompositeFilter {
    /// Filters to apply
    filters: Vec<Box<dyn DeviceFilter>>,
}

impl CompositeFilter {
    /// Create a new composite filter
    pub fn new(filters: Vec<Box<dyn DeviceFilter>>) -> Self {
        Self { filters }
    }
}

impl DeviceFilter for CompositeFilter {
    fn apply_filter(&self, devices: &[DiscoveredDevice]) -> Vec<DiscoveredDevice> {
        devices.iter()
            .filter(|d| self.matches(d))
            .cloned()
            .collect()
    }
    
    fn matches(&self, device: &DiscoveredDevice) -> bool {
        self.filters.iter().all(|f| f.matches(device))
    }
}

/// Function-based filter
pub struct FunctionFilter {
    /// Filter function
    filter_fn: Box<dyn Fn(&DiscoveredDevice) -> bool + Send + Sync>,
}

impl FunctionFilter {
    /// Create a new function filter
    pub fn new<F>(filter_fn: F) -> Self 
    where 
        F: Fn(&DiscoveredDevice) -> bool + Send + Sync + 'static
    {
        Self {
            filter_fn: Box::new(filter_fn),
        }
    }
}

impl DeviceFilter for FunctionFilter {
    fn apply_filter(&self, devices: &[DiscoveredDevice]) -> Vec<DiscoveredDevice> {
        devices.iter()
            .filter(|d| self.matches(d))
            .cloned()
            .collect()
    }
    
    fn matches(&self, device: &DiscoveredDevice) -> bool {
        (self.filter_fn)(device)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::time::Instant;

    use crate::bluetooth::scanner::parse_bdaddr;
    
    fn create_test_device(name: Option<&str>, addr: &str, rssi: Option<i16>) -> DiscoveredDevice {
        DiscoveredDevice {
            address: match parse_bdaddr(addr) {
                Ok(addr) => addr,
                Err(_) => BDAddr::default(),
            },
            name: name.map(|s| s.to_string()),
            rssi,
            manufacturer_data: HashMap::new(),
            is_potential_airpods: false,
            last_seen: Instant::now(),
            is_connected: false,
            service_data: HashMap::new(),
            services: Vec::new(),
            tx_power_level: None,
        }
    }
    
    #[test]
    fn test_name_filter() {
        let filter = NameFilter::new(vec!["AirPods Pro".to_string()]);
        
        let devices = vec![
            create_test_device(Some("AirPods Pro"), "00:11:22:33:44:55", Some(-60)),
            create_test_device(Some("Random Device"), "AA:BB:CC:DD:EE:FF", Some(-70)),
        ];
        
        let filtered = filter.apply_filter(&devices);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, Some("AirPods Pro".to_string()));
    }
    
    #[test]
    fn test_rssi_filter() {
        let filter = RssiFilter::new(-65);
        
        let devices = vec![
            create_test_device(Some("Strong Signal"), "00:11:22:33:44:55", Some(-60)),
            create_test_device(Some("Weak Signal"), "AA:BB:CC:DD:EE:FF", Some(-70)),
        ];
        
        let filtered = filter.apply_filter(&devices);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, Some("Strong Signal".to_string()));
    }
} 