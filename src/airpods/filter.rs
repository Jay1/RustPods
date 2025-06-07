// Remove unused imports
// use std::collections::HashMap;
// use btleplug::api::BDAddr;

use crate::bluetooth::scanner::DiscoveredDevice;
// use crate::error::AirPodsError;
// Remove unused detect_airpods import
use super::{AirPodsType, identify_airpods_type, Result};

/// Apple company identifier for manufacturer data
pub const APPLE_COMPANY_ID: u16 = 0x004C;

/// Filter function type for AirPods device detection
pub type AirPodsFilter = Box<dyn Fn(&DiscoveredDevice) -> bool + Send + Sync>;

/// Advanced filter options for AirPods device detection
#[derive(Debug, Clone)]
pub struct AirPodsFilterOptions {
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

impl Default for AirPodsFilterOptions {
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

impl AirPodsFilterOptions {
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
    pub fn apply_filter(&self, devices: &[DiscoveredDevice]) -> Result<Vec<DiscoveredDevice>> {
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
                        if let Ok(device_type) = identify_airpods_type(&device.name, data) {
                            if !model_types.contains(&device_type) || device_type == AirPodsType::Unknown {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                
                // Apply battery info filter if configured
                if self.require_battery_info {
                    if let Some(data) = device.manufacturer_data.get(&APPLE_COMPANY_ID) {
                        if super::parse_airpods_data(data).is_err() {
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
        
        Ok(filtered_devices)
    }
    
    /// Creates a filter function for use with the BleScanner
    pub fn create_filter_function(&self) -> AirPodsFilter {
        let filter = self.clone();
        
        Box::new(move |device: &DiscoveredDevice| {
            // Basic Apple manufacturer data check - minimum requirement
            if !device.manufacturer_data.contains_key(&APPLE_COMPANY_ID) {
                return false;
            }
            
            // Apply RSSI filter if configured
            if let Some(min_rssi) = filter.min_rssi {
                if let Some(rssi) = device.rssi {
                    if rssi < min_rssi {
                        return false;
                    }
                } else if filter.min_rssi.is_some() {
                    return false;
                }
            }
            
            // Apply name filter if configured
            if let Some(name_filter) = &filter.name_contains {
                if let Some(name) = &device.name {
                    if !name.to_lowercase().contains(&name_filter.to_lowercase()) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            
            // Apply model type filter if configured
            if let Some(model_types) = &filter.model_types {
                if let Some(data) = device.manufacturer_data.get(&APPLE_COMPANY_ID) {
                    // Handle errors from identify_airpods_type gracefully
                    match identify_airpods_type(&device.name, data) {
                        Ok(device_type) => {
                            if !model_types.contains(&device_type) || device_type == AirPodsType::Unknown {
                                return false;
                            }
                        },
                        Err(_) => return false, // Skip devices we can't identify
                    }
                } else {
                    return false;
                }
            }
            
            // Apply battery info filter if configured
            if filter.require_battery_info {
                if let Some(data) = device.manufacturer_data.get(&APPLE_COMPANY_ID) {
                    // Use the parse_airpods_data function to check if battery information is available
                    if super::parse_airpods_data(data).is_err() {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            
            // If we passed all filters, include the device
            true
        })
    }
}

/// Create a filter for all AirPods models
pub fn airpods_all_models_filter() -> AirPodsFilter {
    let options = AirPodsFilterOptions::new()
        .with_models(vec![
            AirPodsType::AirPods1,
            AirPodsType::AirPods2,
            AirPodsType::AirPods3,
            AirPodsType::AirPodsPro,
            AirPodsType::AirPodsPro2,
            AirPodsType::AirPodsMax,
        ]);
    options.create_filter_function()
}

/// Create a filter for AirPods Pro models only
pub fn airpods_pro_filter() -> AirPodsFilter {
    let options = AirPodsFilterOptions::new()
        .with_models(vec![AirPodsType::AirPodsPro, AirPodsType::AirPodsPro2]);
    options.create_filter_function()
}

/// Create a filter for nearby AirPods (based on signal strength)
pub fn airpods_nearby_filter(min_rssi: i16) -> AirPodsFilter {
    let options = AirPodsFilterOptions::new().with_min_rssi(min_rssi);
    options.create_filter_function()
}

/// Create a filter for AirPods with battery information
pub fn airpods_with_battery_filter() -> AirPodsFilter {
    let options = AirPodsFilterOptions::new().with_battery_info(true);
    options.create_filter_function()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    fn create_mock_device(
        name: Option<&str>,
        rssi: Option<i16>,
        data: Option<Vec<u8>>,
        is_airpods: bool,
    ) -> DiscoveredDevice {
        let mut manufacturer_data = HashMap::new();
        
        if let Some(bytes) = data {
            manufacturer_data.insert(APPLE_COMPANY_ID, bytes);
        }
        
        DiscoveredDevice {
            address: btleplug::api::BDAddr::default(),
            name: name.map(String::from),
            rssi,
            tx_power_level: None,
            manufacturer_data,
            services: vec![],
            is_potential_airpods: is_airpods,
            last_seen: std::time::Instant::now(),
            is_connected: false,
            service_data: HashMap::new(),
        }
    }
    
    #[test]
    fn test_filter_by_rssi() {
        let filter = AirPodsFilterOptions::new().with_min_rssi(-70).create_filter_function();
        
        // Create test devices with different RSSI values
        let strong_device = create_mock_device(Some("AirPods"), Some(-60), Some(vec![0x07, 0x19, 0x01, 0x02]), true);
        let weak_device = create_mock_device(Some("AirPods"), Some(-80), Some(vec![0x07, 0x19, 0x01, 0x02]), true);
        
        assert!(filter(&strong_device));
        assert!(!filter(&weak_device));
    }
    
    #[test]
    fn test_filter_by_name() {
        let filter = AirPodsFilterOptions::new().with_name_containing("Pro").create_filter_function();
        
        // Create test devices with different names
        let pro_device = create_mock_device(Some("AirPods Pro"), Some(-60), Some(vec![0x0E, 0x19, 0x01, 0x02]), true);
        let regular_device = create_mock_device(Some("AirPods"), Some(-60), Some(vec![0x07, 0x19, 0x01, 0x02]), true);
        
        assert!(filter(&pro_device));
        assert!(!filter(&regular_device));
    }
    
    #[test]
    fn test_filter_by_model() {
        let filter = AirPodsFilterOptions::new()
            .with_models(vec![AirPodsType::AirPodsPro])
            .create_filter_function();
        
        // Create test devices for different models
        let pro_device = create_mock_device(
            Some("AirPods Pro"),
            Some(-60),
            Some(vec![0x0E, 0x19, 0x01, 0x02]),
            true
        );
        
        let regular_device = create_mock_device(
            Some("AirPods"),
            Some(-60),
            Some(vec![0x07, 0x19, 0x01, 0x02]),
            true
        );
        
        assert!(filter(&pro_device));
        assert!(!filter(&regular_device));
    }
    
    #[test]
    fn test_apply_filter() {
        let options = AirPodsFilterOptions::new()
            .with_min_rssi(-70)
            .with_models(vec![AirPodsType::AirPodsPro]);
        
        // Create test devices
        let devices = vec![
            create_mock_device(Some("AirPods Pro"), Some(-60), Some(vec![0x0E, 0x19, 0x01, 0x02]), true),
            create_mock_device(Some("AirPods"), Some(-80), Some(vec![0x07, 0x19, 0x01, 0x02]), true),
        ];
        
        // Apply should normally succeed
        let filtered = options.apply_filter(&devices).expect("Apply filter failed");
        
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, Some("AirPods Pro".to_string()));
    }
    
    #[test]
    fn test_filter_function() {
        let options = AirPodsFilterOptions::new()
            .with_min_rssi(-70)
            .with_models(vec![AirPodsType::AirPodsPro, AirPodsType::AirPodsPro2]);
        
        // Create the filter function
        let filter = options.create_filter_function();
        
        // Test with various devices
        let pro_device = create_mock_device(Some("AirPods Pro"), Some(-60), Some(vec![0x0E, 0x19, 0x01, 0x02]), true);
        let weak_pro = create_mock_device(Some("AirPods Pro"), Some(-80), Some(vec![0x0E, 0x19, 0x01, 0x02]), true);
        let regular = create_mock_device(Some("AirPods"), Some(-60), Some(vec![0x07, 0x19, 0x01, 0x02]), true);
        
        assert!(filter(&pro_device));  // Should match
        assert!(!filter(&weak_pro));   // Too weak signal
        assert!(!filter(&regular));    // Wrong model
    }
    
    #[test]
    fn test_filter_with_invalid_data() {
        // Test with invalid or corrupt data
        let filter = AirPodsFilterOptions::new()
            .with_models(vec![AirPodsType::AirPodsPro])
            .create_filter_function();
        
        // Create a device with too-short data
        let device_with_invalid_data = create_mock_device(
            Some("AirPods Pro"), 
            Some(-60), 
            Some(vec![0x0E]), // Too short for proper identification
            true
        );
        
        // Should filter out invalid data
        assert!(!filter(&device_with_invalid_data));
    }
    
    #[test]
    fn test_preset_filters() {
        // Test the preset filter functions
        let all_filter = airpods_all_models_filter();
        let pro_filter = airpods_pro_filter();
        let nearby_filter = airpods_nearby_filter(-70);
        let _battery_filter = airpods_with_battery_filter();
        
        // Create test devices
        let pro_device = create_mock_device(
            Some("AirPods Pro"), 
            Some(-60), 
            Some(vec![0x0E, 0x19, 0x01, 0x02]), 
            true
        );
        
        let regular_device = create_mock_device(
            Some("AirPods"), 
            Some(-60), 
            Some(vec![0x07, 0x19, 0x01, 0x02]), 
            true
        );
        
        // Test all_filter
        assert!(all_filter(&pro_device));
        assert!(all_filter(&regular_device));
        
        // Test pro_filter
        assert!(pro_filter(&pro_device));
        assert!(!pro_filter(&regular_device));
        
        // Test nearby_filter
        let weak_device = create_mock_device(Some("AirPods"), Some(-80), Some(vec![0x07, 0x19, 0x01, 0x02]), true);
        assert!(nearby_filter(&pro_device));     // Strong signal
        assert!(!nearby_filter(&weak_device));   // Weak signal
        
        // Test battery_filter - would need special setup to test battery data parsing
    }
} 