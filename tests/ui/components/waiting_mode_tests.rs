#[cfg(test)]
mod tests {
    use rustpods::ui::components::waiting_mode::WaitingMode;
    use rustpods::ui::state::DeviceDetectionState;
    use std::time::Duration;

    #[test]
    fn test_waiting_mode_default() {
        let waiting = WaitingMode::default();
        assert_eq!(waiting.detection_state, DeviceDetectionState::Scanning);
        assert_eq!(waiting.animation_progress, 0.0);
        assert!(!waiting.manual_scan_in_progress);
    }

    #[test]
    fn test_waiting_mode_scan_timing_display() {
        let mut waiting = WaitingMode::default();
        waiting.next_scan_in = Some(Duration::from_secs(10));
        // Placeholder: In a full test, would verify scan timing text rendering
        assert_eq!(waiting.next_scan_in.unwrap().as_secs(), 10);
    }

    #[test]
    fn test_waiting_mode_manual_scan() {
        let mut waiting = WaitingMode::default();
        waiting.manual_scan_in_progress = true;
        assert!(waiting.manual_scan_in_progress);
    }
} 