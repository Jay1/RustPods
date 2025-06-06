//! Bluetooth battery status monitoring for AirPods devices

use std::fmt;
use std::time::Instant;
use btleplug::platform::Peripheral;
use btleplug::api::Peripheral as _;  // Import the Peripheral trait
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use crate::airpods::{AirPodsBattery, parse_airpods_data, AirPodsChargingState};
use crate::error::BluetoothError;

/// Battery status information for AirPods devices
#[derive(Debug, Clone, PartialEq)]
pub struct AirPodsBatteryStatus {
    /// Battery levels for the AirPods
    pub battery: AirPodsBattery,
    /// Timestamp of the last update
    pub last_updated: Instant,
}

// Custom serialization implementation for AirPodsBatteryStatus
impl Serialize for AirPodsBatteryStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AirPodsBatteryStatus", 1)?;
        state.serialize_field("battery", &self.battery)?;
        // Skip serializing last_updated as Instant can't be serialized
        state.end()
    }
}

// Custom deserialization implementation for AirPodsBatteryStatus
impl<'de> Deserialize<'de> for AirPodsBatteryStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct AirPodsBatteryStatusHelper {
            battery: AirPodsBattery,
        }

        let helper = AirPodsBatteryStatusHelper::deserialize(deserializer)?;
        
        Ok(AirPodsBatteryStatus {
            battery: helper.battery,
            last_updated: Instant::now(),
        })
    }
}

impl Default for AirPodsBatteryStatus {
    fn default() -> Self {
        Self {
            battery: AirPodsBattery::default(),
            last_updated: Instant::now(),
        }
    }
}

impl fmt::Display for AirPodsBatteryStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Left: {}%, Right: {}%, Case: {}%, Charging: {}",
            self.battery.left.map_or_else(|| "Unknown".to_string(), |v| format!("{}", v)),
            self.battery.right.map_or_else(|| "Unknown".to_string(), |v| format!("{}", v)),
            self.battery.case.map_or_else(|| "Unknown".to_string(), |v| format!("{}", v)),
            self.battery.charging.map_or_else(
                || "Unknown".to_string(), 
                |state| format!("{:?}", state)
            )
        )
    }
}

impl AirPodsBatteryStatus {
    /// Create a new battery status with the given battery information
    pub fn new(battery: AirPodsBattery) -> Self {
        Self {
            battery,
            last_updated: Instant::now(),
        }
    }
    
    /// Update the battery status with new values
    pub fn update(&mut self, battery: AirPodsBattery) {
        self.battery = battery;
        self.last_updated = Instant::now();
    }
    
    /// Check if the status has any battery information
    pub fn has_battery_info(&self) -> bool {
        self.battery.left.is_some() || self.battery.right.is_some() || self.battery.case.is_some()
    }
    
    /// Check if the status is stale (older than the given duration)
    pub fn is_stale(&self, duration: std::time::Duration) -> bool {
        self.last_updated.elapsed() > duration
    }
}

/// Extract battery status from a peripheral device
pub async fn extract_battery_status(peripheral: &Peripheral) -> AirPodsBatteryStatus {
    match extract_battery_data(peripheral).await {
        Ok(battery) => AirPodsBatteryStatus::new(battery),
        Err(_) => AirPodsBatteryStatus::default(),
    }
}

/// Extract battery data from a peripheral device
pub async fn extract_battery_data(peripheral: &Peripheral) -> Result<AirPodsBattery, BluetoothError> {
    let properties = match peripheral.properties().await {
        Ok(Some(props)) => props,
        Ok(None) => return Err(BluetoothError::Other("No device properties found".to_string())),
        Err(e) => return Err(BluetoothError::ApiError(e.to_string())),
    };
    
    // Check if manufacturer data exists and if it contains Apple data
    const APPLE_COMPANY_ID: u16 = 0x004C;
    
    let apple_data = match properties.manufacturer_data.get(&APPLE_COMPANY_ID) {
        Some(data) => data,
        None => return Err(BluetoothError::Other("No Apple manufacturer data found".to_string())),
    };
    
    // Parse the AirPods data to extract battery information
    match parse_airpods_data(apple_data) {
        Ok(battery) => Ok(battery),
        Err(err) => Err(BluetoothError::Other(format!("Failed to parse AirPods battery data: {}", err))),
    }
}

/// Start monitoring battery status for a peripheral device
pub async fn start_battery_monitoring(
    peripheral: &Peripheral,
    callback: impl Fn(AirPodsBatteryStatus) + Send + 'static,
    refresh_interval: std::time::Duration,
) -> Result<tokio::task::JoinHandle<()>, BluetoothError> {
    // Create a clone of the peripheral to move into the task
    let peripheral_clone = peripheral.clone();
    
    // Spawn a task to monitor the battery status
    let handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(refresh_interval);
        
        loop {
            // Wait for the next interval tick
            interval.tick().await;
            
            // Extract the battery status
            let status = extract_battery_status(&peripheral_clone).await;
            
            // Call the callback with the battery status
            callback(status);
        }
    });
    
    Ok(handle)
}

#[cfg(test)]
mod tests {
    use super::*;
    // ChargingStatus import removed as it's no longer used
    
    // Helper to create test battery status with specified values
    fn create_test_battery_status(left: Option<u8>, right: Option<u8>, case: Option<u8>) -> AirPodsBatteryStatus {
        let battery = AirPodsBattery {
            left,
            right,
            case,
            charging: None,
        };
        
        AirPodsBatteryStatus::new(battery)
    }
    
    // Helper to create test battery with charging status
    fn create_test_battery_with_charging(
        left: Option<u8>, 
        right: Option<u8>, 
        case: Option<u8>,
        charging_state: Option<AirPodsChargingState>
    ) -> AirPodsBattery {
        AirPodsBattery {
            left,
            right,
            case,
            charging: charging_state,
        }
    }
    
    #[test]
    fn test_battery_status_display() {
        let battery = AirPodsBattery {
            left: Some(80),
            right: Some(75),
            case: Some(90),
            charging: None,
        };
        
        let status = AirPodsBatteryStatus::new(battery);
        assert_eq!(status.to_string(), "Left: 80%, Right: 75%, Case: 90%, Charging: Unknown");
        
        // Test with missing values
        let battery = AirPodsBattery {
            left: Some(80),
            right: None,
            case: Some(90),
            charging: None,
        };
        
        let status = AirPodsBatteryStatus::new(battery);
        assert_eq!(status.to_string(), "Left: 80%, Right: Unknown%, Case: 90%, Charging: Unknown");
        
        // Test with all values missing
        let status = AirPodsBatteryStatus::default();
        assert_eq!(status.to_string(), "Left: Unknown%, Right: Unknown%, Case: Unknown%, Charging: Unknown");
    }
    
    #[test]
    fn test_has_battery_info() {
        let battery = AirPodsBattery {
            left: Some(80),
            right: None,
            case: None,
            charging: None,
        };
        
        let status = AirPodsBatteryStatus::new(battery);
        assert!(status.has_battery_info());
        
        // Test with no battery info
        let status = AirPodsBatteryStatus::default();
        assert!(!status.has_battery_info());
        
        // Test with only right earbud
        let status = create_test_battery_status(None, Some(50), None);
        assert!(status.has_battery_info());
        
        // Test with only case
        let status = create_test_battery_status(None, None, Some(75));
        assert!(status.has_battery_info());
    }
    
    #[test]
    fn test_is_stale() {
        // Create a status with a timestamp exactly 60 seconds in the past
        let past_time = Instant::now() - std::time::Duration::from_secs(60);
        let status = AirPodsBatteryStatus {
            battery: AirPodsBattery::default(),
            last_updated: past_time,
        };
        
        // Should be stale if older than 30 seconds
        assert!(status.is_stale(std::time::Duration::from_secs(30)));
        
        // Should not be stale if younger than 90 seconds
        assert!(!status.is_stale(std::time::Duration::from_secs(90)));
        
        // Test with current timestamp
        let current_status = AirPodsBatteryStatus::new(AirPodsBattery::default());
        assert!(!current_status.is_stale(std::time::Duration::from_secs(5)));
        
        // Test with exactly matching duration
        // Create a timestamp exactly 30 seconds in the past
        let exact_past_time = Instant::now() - std::time::Duration::from_secs(30);
        let exact_status = AirPodsBatteryStatus {
            battery: AirPodsBattery::default(),
            last_updated: exact_past_time,
        };
        
        // Should be stale with exactly 30 seconds
        assert!(exact_status.is_stale(std::time::Duration::from_secs(29)));
        
        // Edge case: exactly matching the staleness threshold
        // This is technically implementation-dependent due to precision issues
        // So we won't test the exact boundary case
    }
    
    #[test]
    fn test_update_battery_status() {
        let initial_battery = AirPodsBattery {
            left: Some(50),
            right: Some(60),
            case: Some(70),
            charging: Some(AirPodsChargingState::NotCharging),
        };
        
        let mut status = AirPodsBatteryStatus::new(initial_battery);
        
        // Update with new values
        let new_battery = AirPodsBattery {
            left: Some(40),
            right: Some(30),
            case: Some(80),
            charging: Some(AirPodsChargingState::LeftCharging),
        };
        
        status.update(new_battery.clone());
        
        // Check the values were updated
        assert_eq!(status.battery.left, Some(40));
        assert_eq!(status.battery.right, Some(30));
        assert_eq!(status.battery.case, Some(80));
        assert_eq!(status.battery.charging, Some(AirPodsChargingState::LeftCharging));
        
        // Update with partial data (only left earbud)
        let partial_battery = AirPodsBattery {
            left: Some(35),
            right: None,  // This will replace the existing value with None
            case: None,   // This will replace the existing value with None
            charging: None,
        };
        
        status.update(partial_battery);
        
        // Check that the battery was replaced with the new values
        assert_eq!(status.battery.left, Some(35));
        assert_eq!(status.battery.right, None, "Right earbud value should be None");
        assert_eq!(status.battery.case, None, "Case value should be None");
        
        // Verify charging status was also replaced
        assert_eq!(status.battery.charging, None);
    }
    
    #[test]
    fn test_default_battery_status() {
        let default_status = AirPodsBatteryStatus::default();
        
        assert_eq!(default_status.battery.left, None);
        assert_eq!(default_status.battery.right, None);
        assert_eq!(default_status.battery.case, None);
        assert_eq!(default_status.battery.charging, None);
        
        // Should not have any battery info
        assert!(!default_status.has_battery_info());
        
        // We don't test the timestamp since it's implementation-dependent
        // and can lead to flaky tests
    }
    
    #[test]
    fn test_charging_status_changes() {
        // Create battery with charging
        let charging_battery = create_test_battery_with_charging(
            Some(50), Some(60), Some(70), 
            Some(AirPodsChargingState::BothBudsCharging)
        );
        
        let mut status = AirPodsBatteryStatus::new(charging_battery);
        
        // Verify initial charging state
        assert_eq!(status.battery.charging, Some(AirPodsChargingState::BothBudsCharging));
        
        // Update with non-charging battery
        let non_charging_battery = create_test_battery_with_charging(
            Some(55), Some(65), Some(75), 
            Some(AirPodsChargingState::NotCharging)
        );
        
        status.update(non_charging_battery);
        
        // Verify charging status changed
        assert_eq!(status.battery.charging, Some(AirPodsChargingState::NotCharging));
        
        // Update with mixed charging (only case charging)
        let mixed_charging_battery = create_test_battery_with_charging(
            Some(60), Some(70), Some(80),
            Some(AirPodsChargingState::CaseCharging)
        );
        
        status.update(mixed_charging_battery);
        
        // Verify mixed charging status
        assert_eq!(status.battery.charging, Some(AirPodsChargingState::CaseCharging));
        
        // Check that the display string includes the charging status
        assert!(status.to_string().contains("CaseCharging"));
    }
} 