use std::time::Duration;

/// Configuration for the Bluetooth scanner
#[derive(Debug, Clone)]
pub struct ScanConfig {
    /// Duration of each scan
    pub scan_duration: Duration,
    /// Interval between scans
    pub interval_between_scans: Duration,
    /// Maximum number of scan cycles before auto-stop
    /// None means indefinite scanning
    pub max_scan_cycles: Option<usize>,
    /// Whether to automatically stop scanning after the first cycle
    pub auto_stop_scan: bool,
    /// Minimum RSSI (signal strength) filter
    /// Devices with lower RSSI will be ignored
    pub min_rssi: Option<i16>,
    /// Timeout after which inactive devices are removed
    pub device_inactive_timeout: Option<Duration>,
    /// Whether to continue scanning in a loop
    pub continuous: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            scan_duration: Duration::from_secs(10),
            interval_between_scans: Duration::from_secs(20),
            auto_stop_scan: true,
            max_scan_cycles: None,
            device_inactive_timeout: None,
            continuous: false,
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
            device_inactive_timeout: None,
            continuous: true,
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
            device_inactive_timeout: None,
            continuous: false,
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
            device_inactive_timeout: None,
            continuous: false,
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
            device_inactive_timeout: None,
            continuous: false,
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
    
    /// Set the device inactive timeout
    pub fn with_device_inactive_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.device_inactive_timeout = timeout;
        self
    }
    
    /// Set whether to continue scanning in a loop
    pub fn with_continuous(mut self, continuous: bool) -> Self {
        self.continuous = continuous;
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
        assert_eq!(config.device_inactive_timeout, None);
        assert!(!config.continuous);
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
        assert_eq!(config.device_inactive_timeout, default_config.device_inactive_timeout);
        assert_eq!(config.continuous, default_config.continuous);
        assert_eq!(config.min_rssi, default_config.min_rssi);
    }

    #[test]
    fn test_continuous_config() {
        let config = ScanConfig::continuous();
        
        assert_eq!(config.scan_duration, Duration::from_secs(5));
        assert_eq!(config.interval_between_scans, Duration::from_secs(2));
        assert!(config.auto_stop_scan);
        assert_eq!(config.max_scan_cycles, None);
        assert_eq!(config.device_inactive_timeout, None);
        assert!(config.continuous);
        assert_eq!(config.min_rssi, None);
    }

    #[test]
    fn test_power_efficient_config() {
        let config = ScanConfig::power_efficient();
        
        assert_eq!(config.scan_duration, Duration::from_secs(3));
        assert_eq!(config.interval_between_scans, Duration::from_secs(60));
        assert!(config.auto_stop_scan);
        assert_eq!(config.max_scan_cycles, None);
        assert_eq!(config.device_inactive_timeout, None);
        assert!(!config.continuous);
        assert_eq!(config.min_rssi, Some(-80));
    }

    #[test]
    fn test_airpods_optimized_config() {
        let config = ScanConfig::airpods_optimized();
        
        assert_eq!(config.scan_duration, Duration::from_secs(5));
        assert_eq!(config.interval_between_scans, Duration::from_secs(10));
        assert!(config.auto_stop_scan);
        assert_eq!(config.max_scan_cycles, None);
        assert_eq!(config.device_inactive_timeout, None);
        assert!(!config.continuous);
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
        assert_eq!(config.device_inactive_timeout, None);
        assert!(!config.continuous);
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
    fn test_with_device_inactive_timeout() {
        let timeout = Some(Duration::from_secs(60));
        let config = ScanConfig::default().with_device_inactive_timeout(timeout);
        
        assert_eq!(config.device_inactive_timeout, timeout);
    }

    #[test]
    fn test_with_continuous() {
        let config = ScanConfig::default().with_continuous(true);
        
        assert!(config.continuous);
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
            .with_device_inactive_timeout(Some(Duration::from_secs(60)))
            .with_continuous(true)
            .with_min_rssi(Some(-75));
        
        assert_eq!(config.scan_duration, Duration::from_secs(15));
        assert_eq!(config.interval_between_scans, Duration::from_secs(30));
        assert!(!config.auto_stop_scan);
        assert_eq!(config.max_scan_cycles, Some(3));
        assert_eq!(config.device_inactive_timeout, Some(Duration::from_secs(60)));
        assert!(config.continuous);
        assert_eq!(config.min_rssi, Some(-75));
    }
} 