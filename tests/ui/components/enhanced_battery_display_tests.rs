#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::components::enhanced_battery_display::{EnhancedBatteryDisplay, battery_level_low};
    use crate::airpods::AirPodsBattery;
    use crate::airpods::AirPodsChargingState;

    #[test]
    fn test_enhanced_battery_display_creation() {
        // Create with battery info
        let battery = AirPodsBattery {
            left: Some(80),
            right: Some(75),
            case: Some(90),
            charging: Some(AirPodsChargingState::CaseCharging),
        };
        let display = EnhancedBatteryDisplay::new(Some(battery.clone()));
        assert!(display.battery.is_some());
        if let Some(b) = display.battery {
            assert_eq!(b.left, Some(80));
            assert_eq!(b.right, Some(75));
            assert_eq!(b.case, Some(90));
            assert!(b.charging.is_some());
            assert_eq!(*b.charging.as_ref().unwrap(), AirPodsChargingState::CaseCharging);
        }
        // Test empty creation
        let empty_display = EnhancedBatteryDisplay::empty();
        assert!(empty_display.battery.is_none());
    }

    #[test]
    fn test_battery_level_low_detection() {
        // Test with low battery
        let low_battery = AirPodsBattery {
            left: Some(15),
            right: Some(50),
            case: Some(75),
            charging: Some(AirPodsChargingState::NotCharging),
        };
        assert!(battery_level_low(&low_battery));
        // Test with no low battery
        let good_battery = AirPodsBattery {
            left: Some(60),
            right: Some(50),
            case: Some(75),
            charging: Some(AirPodsChargingState::NotCharging),
        };
        assert!(!battery_level_low(&good_battery));
    }
} 