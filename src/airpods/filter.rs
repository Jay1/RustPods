use std::collections::HashMap;
use btleplug::api::BDAddr;

use crate::bluetooth::DiscoveredDevice;
use super::{AirPodsType, detect_airpods, identify_airpods_type};

/// Apple company identifier for manufacturer data
pub const APPLE_COMPANY_ID: u16 = 0x004C;

/// Advanced filter options for AirPods device detection
#[derive(Debug, Clone)]
pub struct AirPodsFilter {
    /// Filter by specific device models
    pub model_types: Option<Vec<AirPodsType>>,
    /// Minimum RSSI (signal strength) value
    pub min_rssi: Option<i16>,
    /// Enable name-based filtering
    pub name_contains: Option<String>,
    /// Maximum number of devices to return
    pub max_devices: Option<usize>,
    /// Sort devices by RSSI strength (strongest first)
    pub sort_by_signal_strength: bool,
    /// Only include devices seen within this many seconds
    pub max_age_seconds: Option<u64>,
    /// Exclude devices with unknown battery levels
    pub require_battery_info: bool,
}

impl Default for AirPodsFilter {
    fn default() -> Self {
        Self {
            model_types: None,
            min_rssi: Some(-80), // Reasonable default to filter very weak signals
            name_contains: None,
            max_devices: None,
            sort_by_signal_strength: true,
            max_age_seconds: Some(60), // 1 minute by default
            require_battery_info: false,
        }
    }
}

impl AirPodsFilter {
    /// Create a new default AirPods filter
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Filter for only specific AirPods models
    pub fn with_models(mut self, models: Vec<AirPodsType>) -> Self {
        self.model_types = Some(models);
        self
    }
    
    /// Set minimum signal strength to include
    pub fn with_min_rssi(mut self, rssi: i16) -> Self {
        self.min_rssi = Some(rssi);
        self
    }
    
    /// Filter by device name containing specific text
    pub fn with_name_containing(mut self, text: &str) -> Self {
        self.name_contains = Some(text.to_string());
        self
    }
    
    /// Limit maximum number of results
    pub fn with_max_devices(mut self, count: usize) -> Self {
        self.max_devices = Some(count);
        self
    }
    
    /// Set whether to sort by signal strength
    pub fn with_signal_strength_sorting(mut self, enabled: bool) -> Self {
        self.sort_by_signal_strength = enabled;
        self
    }
    
    /// Only include devices seen within the specified time period
    pub fn with_max_age(mut self, seconds: u64) -> Self {
        self.max_age_seconds = Some(seconds);
        self
    }
    
    /// Only include devices with valid battery information
    pub fn with_battery_info(mut self, required: bool) -> Self {
        self.require_battery_info = required;
        self
    }
    
    /// Apply the filter to a list of discovered devices
    pub fn apply_filter(&self, devices: &[DiscoveredDevice]) -> Vec<DiscoveredDevice> {
        let now = std::time::Instant::now();
        
        let mut filtered_devices: Vec<DiscoveredDevice> = devices
            .iter()
            .filter(|device| {
                // Only include devices with Apple manufacturer data
                if !device.manufacturer_data.contains_key(&APPLE_COMPANY_ID) {
                    return false;
                }
                
                // Apply RSSI filter if configured
                if let Some(min_rssi) = self.min_rssi {
                    if let Some(rssi) = device.rssi {
                        if rssi < min_rssi {
                            return false;
                        }
                    } else if self.min_rssi.is_some() {
                        // If we require RSSI filtering but the device has no RSSI, exclude it
                        return false;
                    }
                }
                
                // Apply name filter if configured
                if let Some(name_filter) = &self.name_contains {
                    if let Some(name) = &device.name {
                        if !name.to_lowercase().contains(&name_filter.to_lowercase()) {
                            return false;
                        }
                    } else {
                        // If we require name filtering but the device has no name, exclude it
                        return false;
                    }
                }
                
                // Apply age filter if configured
                if let Some(max_age) = self.max_age_seconds {
                    let age = now.duration_since(device.last_seen);
                    if age.as_secs() > max_age {
                        return false;
                    }
                }
                
                // Apply model type filter if configured
                if let Some(model_types) = &self.model_types {
                    if let Some(data) = device.manufacturer_data.get(&APPLE_COMPANY_ID) {
                        let device_type = identify_airpods_type(data);
                        if !model_types.contains(&device_type) || device_type == AirPodsType::Unknown {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                
                // Apply battery info filter if configured
                if self.require_battery_info {
                    if let Some(data) = device.manufacturer_data.get(&APPLE_COMPANY_ID) {
                        if super::parse_airpods_data(data).is_none() {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                
                // If we passed all filters, include the device
                true
            })
            .cloned()
            .collect();
            
        // Sort results if requested
        if self.sort_by_signal_strength {
            filtered_devices.sort_by(|a, b| {
                // Compare RSSI values, stronger signals first
                let a_rssi = a.rssi.unwrap_or(i16::MIN);
                let b_rssi = b.rssi.unwrap_or(i16::MIN);
                // Reverse ordering to get strongest signals first
                b_rssi.cmp(&a_rssi)
            });
        }
        
        // Limit to max devices if configured
        if let Some(max) = self.max_devices {
            filtered_devices.truncate(max);
        }
        
        filtered_devices
    }
    
    /// Creates a filter function for use with the BleScanner
    pub fn create_filter_function(&self) -> impl Fn(&DiscoveredDevice) -> bool + '_ {
        move |device| {
            // Basic Apple manufacturer data check - minimum requirement
            if !device.manufacturer_data.contains_key(&APPLE_COMPANY_ID) {
                return false;
            }
            
            // Apply RSSI filter if configured
            if let Some(min_rssi) = self.min_rssi {
                if let Some(rssi) = device.rssi {
                    if rssi < min_rssi {
                        return false;
                    }
                } else if self.min_rssi.is_some() {
                    return false;
                }
            }
            
            // Apply name filter if configured
            if let Some(name_filter) = &self.name_contains {
                if let Some(name) = &device.name {
                    if !name.to_lowercase().contains(&name_filter.to_lowercase()) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            
            // Apply model type filter if configured
            if let Some(model_types) = &self.model_types {
                if let Some(data) = device.manufacturer_data.get(&APPLE_COMPANY_ID) {
                    let device_type = identify_airpods_type(data);
                    if !model_types.contains(&device_type) || device_type == AirPodsType::Unknown {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            
            // If we passed all filters, include the device
            true
        }
    }
}

/// Preset filter for all AirPods models
pub fn airpods_all_models_filter() -> AirPodsFilter {
    AirPodsFilter::new()
}

/// Preset filter for AirPods Pro models only
pub fn airpods_pro_filter() -> AirPodsFilter {
    AirPodsFilter::new().with_models(vec![AirPodsType::AirPodsPro, AirPodsType::AirPodsPro2])
}

/// Preset filter for stronger signal AirPods (likely to be current user's)
pub fn airpods_nearby_filter() -> AirPodsFilter {
    AirPodsFilter::new()
        .with_min_rssi(-65) // Only include strong signals
        .with_max_devices(3) // Limit to 3 closest devices
}

/// Preset filter for AirPods with battery information
pub fn airpods_with_battery_filter() -> AirPodsFilter {
    AirPodsFilter::new()
        .with_battery_info(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use btleplug::api::BDAddr;
    
    fn create_mock_device(
        name: Option<&str>,
        rssi: Option<i16>,
        data: Option<Vec<u8>>,
        is_airpods: bool,
    ) -> DiscoveredDevice {
        let mut manufacturer_data = HashMap::new();
        if let Some(d) = data {
            manufacturer_data.insert(APPLE_COMPANY_ID, d);
        }
        
        DiscoveredDevice {
            address: BDAddr::default(),
            name: name.map(String::from),
            rssi,
            manufacturer_data,
            is_potential_airpods: is_airpods,
            last_seen: std::time::Instant::now(),
        }
    }
    
    #[test]
    fn test_filter_by_rssi() {
        let devices = vec![
            create_mock_device(Some("AirPods"), Some(-50), Some(vec![0x07, 0x19, 0x01]), true),
            create_mock_device(Some("AirPods"), Some(-90), Some(vec![0x07, 0x19, 0x01]), true),
        ];
        
        let filter = AirPodsFilter::new().with_min_rssi(-60);
        let result = filter.apply_filter(&devices);
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].rssi, Some(-50));
    }
    
    #[test]
    fn test_filter_by_name() {
        let devices = vec![
            create_mock_device(Some("AirPods Pro"), Some(-50), Some(vec![0x0E, 0x19, 0x01]), true),
            create_mock_device(Some("Other Device"), Some(-50), Some(vec![0x07, 0x19, 0x01]), true),
        ];
        
        let filter = AirPodsFilter::new().with_name_containing("Pro");
        let result = filter.apply_filter(&devices);
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, Some("AirPods Pro".to_string()));
    }
    
    #[test]
    fn test_filter_by_model() {
        let airpods_pro_data = vec![0x0E, 0x19, 0x01];
        let airpods_regular_data = vec![0x07, 0x19, 0x01];
        
        let devices = vec![
            create_mock_device(Some("AirPods Pro"), Some(-50), Some(airpods_pro_data.clone()), true),
            create_mock_device(Some("AirPods"), Some(-50), Some(airpods_regular_data.clone()), true),
        ];
        
        let filter = AirPodsFilter::new().with_models(vec![AirPodsType::AirPodsPro]);
        let result = filter.apply_filter(&devices);
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, Some("AirPods Pro".to_string()));
    }
    
    #[test]
    fn test_filter_function() {
        let airpods_pro_data = vec![0x0E, 0x19, 0x01];
        let device = create_mock_device(Some("AirPods Pro"), Some(-50), Some(airpods_pro_data), true);
        
        let filter = AirPodsFilter::new().with_models(vec![AirPodsType::AirPodsPro]);
        let filter_fn = filter.create_filter_function();
        
        assert!(filter_fn(&device));
        
        // Test with a device that shouldn't pass the filter
        let non_matching_device = create_mock_device(Some("AirPods"), Some(-50), Some(vec![0x07, 0x19, 0x01]), true);
        assert!(!filter_fn(&non_matching_device));
    }
} 