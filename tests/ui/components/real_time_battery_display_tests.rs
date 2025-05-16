#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::components::real_time_battery_display::RealTimeBatteryDisplay;
    use crate::airpods::{AirPodsBattery, AirPodsChargingState};
    use crate::bluetooth::AirPodsBatteryStatus;

    #[test]
    fn test_real_time_battery_display_creation() {
        // Create battery status
        let battery = AirPodsBattery {
            left: Some(80),
            right: Some(75),
            case: Some(90),
            charging: Some(AirPodsChargingState::CaseCharging),
        };
        let status = AirPodsBatteryStatus::new(battery);
        // Create display
        let display = RealTimeBatteryDisplay::new(Some(status));
        // Verify it has the status
        assert!(display.battery_status.is_some());
        // Test empty creation
        let empty_display = RealTimeBatteryDisplay::new(None);
        assert!(empty_display.battery_status.is_none());
    }

    #[test]
    fn test_time_remaining_calculation() {
        // Create battery status with different levels
        let battery = AirPodsBattery {
            left: Some(50),
            right: Some(60),
            case: Some(90),
            charging: Some(AirPodsChargingState::NotCharging),
        };
        let status = AirPodsBatteryStatus::new(battery);
        // Create display
        let display = RealTimeBatteryDisplay::new(Some(status));
        // Should use the lower value between left and right
        let time = display.calculate_time_remaining();
        assert!(time.is_some());
        if let Some(minutes) = time {
            // 50% should be 150 minutes (50% of 300)
            assert_eq!(minutes, 150);
        }
    }
} 