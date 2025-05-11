use std::time::Duration;

/// Configuration for BLE scanning
#[derive(Debug, Clone)]
pub struct ScanConfig {
    /// How long to actively scan for devices
    pub scan_duration: Duration,
    
    /// How long to wait between scan cycles
    pub interval_between_scans: Duration,
    
    /// Whether to automatically stop scanning after scan_duration
    pub auto_stop_scan: bool,
    
    /// Maximum number of scan cycles (None for unlimited)
    pub max_scan_cycles: Option<usize>,
    
    /// Whether to keep device history between scans
    pub maintain_device_history: bool,
    
    /// Whether to use active scanning (more data, but higher power usage)
    pub active_scanning: bool,
    
    /// Minimum signal strength to consider devices (None for no filtering)
    pub min_rssi: Option<i16>,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            scan_duration: Duration::from_secs(10),
            interval_between_scans: Duration::from_secs(20),
            auto_stop_scan: true,
            max_scan_cycles: None,
            maintain_device_history: true,
            active_scanning: true,
            min_rssi: None,
        }
    }
}

impl ScanConfig {
    /// Create a new scan configuration with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create a configuration for continuous scanning with minimal delays
    pub fn continuous() -> Self {
        Self {
            scan_duration: Duration::from_secs(5),
            interval_between_scans: Duration::from_secs(2),
            auto_stop_scan: true,
            max_scan_cycles: None,
            maintain_device_history: true,
            active_scanning: true,
            min_rssi: None,
        }
    }
    
    /// Create a configuration for power-efficient scanning
    pub fn power_efficient() -> Self {
        Self {
            scan_duration: Duration::from_secs(3),
            interval_between_scans: Duration::from_secs(60),
            auto_stop_scan: true,
            max_scan_cycles: None,
            maintain_device_history: true,
            active_scanning: false,
            min_rssi: Some(-80), // Filter out weak signals
        }
    }
    
    /// Create a configuration optimized for finding AirPods quickly
    pub fn airpods_optimized() -> Self {
        Self {
            scan_duration: Duration::from_secs(5),
            interval_between_scans: Duration::from_secs(10),
            auto_stop_scan: true,
            max_scan_cycles: None,
            maintain_device_history: true,
            active_scanning: true,
            min_rssi: Some(-70), // AirPods are usually nearby
        }
    }
    
    /// Create a configuration for a one-time scan
    pub fn one_time_scan(duration: Duration) -> Self {
        Self {
            scan_duration: duration,
            interval_between_scans: Duration::from_secs(0),
            auto_stop_scan: true,
            max_scan_cycles: Some(1),
            maintain_device_history: false,
            active_scanning: true,
            min_rssi: None,
        }
    }
    
    /// Set the scan duration
    pub fn with_scan_duration(mut self, duration: Duration) -> Self {
        self.scan_duration = duration;
        self
    }
    
    /// Set the interval between scans
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval_between_scans = interval;
        self
    }
    
    /// Set whether to automatically stop scanning
    pub fn with_auto_stop(mut self, auto_stop: bool) -> Self {
        self.auto_stop_scan = auto_stop;
        self
    }
    
    /// Set the maximum number of scan cycles
    pub fn with_max_cycles(mut self, max_cycles: Option<usize>) -> Self {
        self.max_scan_cycles = max_cycles;
        self
    }
    
    /// Set whether to maintain device history between scans
    pub fn with_history(mut self, maintain_history: bool) -> Self {
        self.maintain_device_history = maintain_history;
        self
    }
    
    /// Set whether to use active scanning
    pub fn with_active_scanning(mut self, active: bool) -> Self {
        self.active_scanning = active;
        self
    }
    
    /// Set the minimum signal strength
    pub fn with_min_rssi(mut self, min_rssi: Option<i16>) -> Self {
        self.min_rssi = min_rssi;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ScanConfig::default();
        
        assert_eq!(config.scan_duration, Duration::from_secs(10));
        assert_eq!(config.interval_between_scans, Duration::from_secs(20));
        assert!(config.auto_stop_scan);
        assert_eq!(config.max_scan_cycles, None);
        assert!(config.maintain_device_history);
        assert!(config.active_scanning);
        assert_eq!(config.min_rssi, None);
    }

    #[test]
    fn test_new_config() {
        let config = ScanConfig::new();
        // new() should return default values
        let default_config = ScanConfig::default();
        
        assert_eq!(config.scan_duration, default_config.scan_duration);
        assert_eq!(config.interval_between_scans, default_config.interval_between_scans);
        assert_eq!(config.auto_stop_scan, default_config.auto_stop_scan);
        assert_eq!(config.max_scan_cycles, default_config.max_scan_cycles);
        assert_eq!(config.maintain_device_history, default_config.maintain_device_history);
        assert_eq!(config.active_scanning, default_config.active_scanning);
        assert_eq!(config.min_rssi, default_config.min_rssi);
    }

    #[test]
    fn test_continuous_config() {
        let config = ScanConfig::continuous();
        
        assert_eq!(config.scan_duration, Duration::from_secs(5));
        assert_eq!(config.interval_between_scans, Duration::from_secs(2));
        assert!(config.auto_stop_scan);
        assert_eq!(config.max_scan_cycles, None);
        assert!(config.maintain_device_history);
        assert!(config.active_scanning);
        assert_eq!(config.min_rssi, None);
    }

    #[test]
    fn test_power_efficient_config() {
        let config = ScanConfig::power_efficient();
        
        assert_eq!(config.scan_duration, Duration::from_secs(3));
        assert_eq!(config.interval_between_scans, Duration::from_secs(60));
        assert!(config.auto_stop_scan);
        assert_eq!(config.max_scan_cycles, None);
        assert!(config.maintain_device_history);
        assert!(!config.active_scanning);
        assert_eq!(config.min_rssi, Some(-80));
    }

    #[test]
    fn test_airpods_optimized_config() {
        let config = ScanConfig::airpods_optimized();
        
        assert_eq!(config.scan_duration, Duration::from_secs(5));
        assert_eq!(config.interval_between_scans, Duration::from_secs(10));
        assert!(config.auto_stop_scan);
        assert_eq!(config.max_scan_cycles, None);
        assert!(config.maintain_device_history);
        assert!(config.active_scanning);
        assert_eq!(config.min_rssi, Some(-70));
    }

    #[test]
    fn test_one_time_scan_config() {
        let duration = Duration::from_secs(15);
        let config = ScanConfig::one_time_scan(duration);
        
        assert_eq!(config.scan_duration, duration);
        assert_eq!(config.interval_between_scans, Duration::from_secs(0));
        assert!(config.auto_stop_scan);
        assert_eq!(config.max_scan_cycles, Some(1));
        assert!(!config.maintain_device_history);
        assert!(config.active_scanning);
        assert_eq!(config.min_rssi, None);
    }

    #[test]
    fn test_with_scan_duration() {
        let duration = Duration::from_secs(30);
        let config = ScanConfig::default().with_scan_duration(duration);
        
        assert_eq!(config.scan_duration, duration);
    }

    #[test]
    fn test_with_interval() {
        let interval = Duration::from_secs(45);
        let config = ScanConfig::default().with_interval(interval);
        
        assert_eq!(config.interval_between_scans, interval);
    }

    #[test]
    fn test_with_auto_stop() {
        let config = ScanConfig::default().with_auto_stop(false);
        
        assert!(!config.auto_stop_scan);
    }

    #[test]
    fn test_with_max_cycles() {
        let max_cycles = Some(5);
        let config = ScanConfig::default().with_max_cycles(max_cycles);
        
        assert_eq!(config.max_scan_cycles, max_cycles);
    }

    #[test]
    fn test_with_history() {
        let config = ScanConfig::default().with_history(false);
        
        assert!(!config.maintain_device_history);
    }

    #[test]
    fn test_with_active_scanning() {
        let config = ScanConfig::default().with_active_scanning(false);
        
        assert!(!config.active_scanning);
    }

    #[test]
    fn test_with_min_rssi() {
        let min_rssi = Some(-65);
        let config = ScanConfig::default().with_min_rssi(min_rssi);
        
        assert_eq!(config.min_rssi, min_rssi);
    }

    #[test]
    fn test_builder_pattern_chaining() {
        let config = ScanConfig::default()
            .with_scan_duration(Duration::from_secs(15))
            .with_interval(Duration::from_secs(30))
            .with_auto_stop(false)
            .with_max_cycles(Some(3))
            .with_history(false)
            .with_active_scanning(false)
            .with_min_rssi(Some(-75));
        
        assert_eq!(config.scan_duration, Duration::from_secs(15));
        assert_eq!(config.interval_between_scans, Duration::from_secs(30));
        assert!(!config.auto_stop_scan);
        assert_eq!(config.max_scan_cycles, Some(3));
        assert!(!config.maintain_device_history);
        assert!(!config.active_scanning);
        assert_eq!(config.min_rssi, Some(-75));
    }
} 