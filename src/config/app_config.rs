use std::path::PathBuf;
use std::time::Duration;

use crate::bluetooth::ScanConfig;

/// Application configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Automatically start scanning on startup
    pub auto_scan_on_startup: bool,
    
    /// Scan duration
    pub scan_duration: Duration,
    
    /// Interval between scans
    pub scan_interval: Duration,
    
    /// Minimum RSSI to consider a device
    pub min_rssi: Option<i16>,
    
    /// Show battery notifications
    pub show_notifications: bool,
    
    /// Start minimized to system tray
    pub start_minimized: bool,
    
    /// Theme (light or dark)
    pub theme: Theme,
    
    /// Path to save settings
    pub settings_path: PathBuf,
}

/// UI theme
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Theme {
    /// Light theme
    Light,
    /// Dark theme
    Dark,
    /// System theme
    System,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            auto_scan_on_startup: true,
            scan_duration: Duration::from_secs(5),
            scan_interval: Duration::from_secs(30),
            min_rssi: Some(-70),
            show_notifications: true,
            start_minimized: true,
            theme: Theme::System,
            settings_path: default_settings_path(),
        }
    }
}

impl AppConfig {
    /// Convert to scan config for the bluetooth scanner
    pub fn to_scan_config(&self) -> ScanConfig {
        ScanConfig::new()
            .with_scan_duration(self.scan_duration)
            .with_interval(self.scan_interval)
            .with_min_rssi(self.min_rssi)
            .with_active_scanning(true)
    }
    
    /// Load configuration from file
    pub fn load() -> Result<Self, std::io::Error> {
        // For now, just return the default config
        // In a real implementation, this would load from a file
        Ok(Self::default())
    }
    
    /// Save configuration to file
    pub fn save(&self) -> Result<(), std::io::Error> {
        // For now, do nothing
        // In a real implementation, this would save to a file
        Ok(())
    }
}

/// Get the default settings path
fn default_settings_path() -> PathBuf {
    dirs_next::config_dir()
        .map(|config_dir| config_dir.join("rustpods").join("settings.json"))
        .unwrap_or_else(|| PathBuf::from("settings.json")) // Fallback to current directory
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        
        assert!(config.auto_scan_on_startup);
        assert_eq!(config.scan_duration, Duration::from_secs(5));
        assert_eq!(config.scan_interval, Duration::from_secs(30));
        assert_eq!(config.min_rssi, Some(-70));
        assert!(config.show_notifications);
        assert!(config.start_minimized);
        assert_eq!(config.theme, Theme::System);
    }
    
    #[test]
    fn test_to_scan_config() {
        let config = AppConfig::default();
        let scan_config = config.to_scan_config();
        
        assert_eq!(scan_config.scan_duration, Duration::from_secs(5));
        assert_eq!(scan_config.interval_between_scans, Duration::from_secs(30));
        assert!(scan_config.active_scanning);
        assert_eq!(scan_config.min_rssi, Some(-70));
    }
} 