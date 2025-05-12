use std::path::PathBuf;
use std::time::Duration;
use serde::{Serialize, Deserialize};

use crate::bluetooth::ScanConfig;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppConfig {
    /// Bluetooth scanning configuration
    #[serde(default)]
    pub bluetooth: BluetoothConfig,
    
    /// User interface configuration
    #[serde(default)]
    pub ui: UiConfig,
    
    /// System configuration
    #[serde(default)]
    pub system: SystemConfig,
    
    /// Battery monitoring configuration
    #[serde(default)]
    pub battery: BatteryConfig,
    
    /// Path to save settings (not serialized)
    #[serde(skip)]
    pub settings_path: PathBuf,
}

/// Bluetooth scanning and connection configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BluetoothConfig {
    /// Automatically start scanning on startup
    #[serde(default = "default_auto_scan")]
    pub auto_scan_on_startup: bool,
    
    /// Scan duration in seconds
    #[serde(default = "default_scan_duration_secs", with = "duration_serde")]
    pub scan_duration: Duration,
    
    /// Interval between scans in seconds
    #[serde(default = "default_scan_interval_secs", with = "duration_serde")]
    pub scan_interval: Duration,
    
    /// Minimum RSSI to consider a device
    #[serde(default)]
    pub min_rssi: Option<i16>,
    
    /// Battery status refresh interval in seconds
    #[serde(default = "default_battery_refresh_interval")]
    pub battery_refresh_interval: u64,
    
    /// Auto-reconnect to last connected device
    #[serde(default = "default_true")]
    pub auto_reconnect: bool,
    
    /// Reconnect attempts before giving up
    #[serde(default = "default_reconnect_attempts")]
    pub reconnect_attempts: u32,
    
    /// Use adaptive polling for battery status
    #[serde(default = "default_true")]
    pub adaptive_polling: bool,
}

/// Window position information
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct WindowPosition {
    /// X coordinate
    pub x: i32,
    /// Y coordinate
    pub y: i32,
}

impl From<iced::Point> for WindowPosition {
    fn from(point: iced::Point) -> Self {
        Self {
            x: point.x as i32,
            y: point.y as i32,
        }
    }
}

impl From<WindowPosition> for iced::Point {
    fn from(pos: WindowPosition) -> Self {
        iced::Point::new(pos.x as f32, pos.y as f32)
    }
}

/// User interface configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UiConfig {
    /// Show battery notifications
    #[serde(default = "default_true")]
    pub show_notifications: bool,
    
    /// Start minimized to system tray
    #[serde(default = "default_true")]
    pub start_minimized: bool,
    
    /// Theme (light, dark, or system)
    #[serde(default)]
    pub theme: Theme,
    
    /// Show battery percentage in system tray icon
    #[serde(default = "default_true")]
    pub show_percentage_in_tray: bool,
    
    /// Show a warning notification when battery is low
    #[serde(default = "default_true")]
    pub show_low_battery_warning: bool,
    
    /// Low battery threshold percentage
    #[serde(default = "default_low_battery_threshold")]
    pub low_battery_threshold: u8,
    
    /// Remember window position
    #[serde(default = "default_true")]
    pub remember_window_position: bool,
    
    /// Last window position
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_window_position: Option<WindowPosition>,
    
    /// Minimize to tray when closed
    #[serde(default = "default_true")]
    pub minimize_to_tray_on_close: bool,
    
    /// Minimize to tray when window loses focus
    #[serde(default = "default_false")]
    pub minimize_on_blur: bool,
    
    /// Auto-hide window after inactivity timeout (in seconds)
    #[serde(default)]
    pub auto_hide_timeout: Option<u64>,
}

/// System configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemConfig {
    /// Launch application at system startup
    #[serde(default)]
    pub launch_at_startup: bool,
    
    /// Log level (error, warn, info, debug, trace)
    #[serde(default)]
    pub log_level: LogLevel,
    
    /// Enable application telemetry
    #[serde(default)]
    pub enable_telemetry: bool,
}

/// Battery monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BatteryConfig {
    /// Low battery threshold percentage
    #[serde(default = "default_low_battery_threshold")]
    pub low_threshold: u8,
    
    /// Enable smoothing of battery readings to prevent display fluctuations
    #[serde(default = "default_true")]
    pub smoothing_enabled: bool,
    
    /// Minimum level change to trigger faster polling (percentage)
    #[serde(default = "default_change_threshold")]
    pub change_threshold: u8,
    
    /// Send notifications for low battery
    #[serde(default = "default_true")]
    pub notify_low: bool,
    
    /// Send notifications for charging completed
    #[serde(default = "default_true")]
    pub notify_charged: bool,
}

/// UI theme
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    /// Light theme
    Light,
    /// Dark theme
    Dark,
    /// System theme (follows OS settings)
    #[serde(rename = "system")]
    System,
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Theme::Light => write!(f, "Light"),
            Theme::Dark => write!(f, "Dark"),
            Theme::System => write!(f, "System"),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::System
    }
}

/// Log level
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// Error level - only errors are logged
    Error,
    /// Warning level - warnings and errors are logged
    Warn,
    /// Info level - informational messages, warnings, and errors are logged
    Info,
    /// Debug level - debug information and all above are logged
    Debug,
    /// Trace level - verbose tracing information and all above are logged
    Trace,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Error => write!(f, "Error"),
            LogLevel::Warn => write!(f, "Warning"),
            LogLevel::Info => write!(f, "Info"),
            LogLevel::Debug => write!(f, "Debug"),
            LogLevel::Trace => write!(f, "Trace"),
        }
    }
}

// Default functions for serde
fn default_true() -> bool { true }
fn default_false() -> bool { false }
fn default_scan_duration_secs() -> Duration { Duration::from_secs(5) }
fn default_scan_interval_secs() -> Duration { Duration::from_secs(30) }
fn default_auto_scan() -> bool { true }
fn default_battery_refresh_interval() -> u64 { 10 }
fn default_reconnect_attempts() -> u32 { 3 }
fn default_low_battery_threshold() -> u8 { 20 }
fn default_change_threshold() -> u8 { 5 }

// Custom serialization for Duration
mod duration_serde {
    use std::time::Duration;
    use serde::{Deserialize, Deserializer, Serializer, Serialize};

    // Serialize Duration as seconds
    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let secs = duration.as_secs();
        secs.serialize(serializer)
    }

    // Deserialize Duration from seconds
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            bluetooth: BluetoothConfig::default(),
            ui: UiConfig::default(),
            system: SystemConfig::default(),
            battery: BatteryConfig::default(),
            settings_path: default_settings_path(),
        }
    }
}

impl Default for BluetoothConfig {
    fn default() -> Self {
        Self {
            auto_scan_on_startup: default_auto_scan(),
            scan_duration: default_scan_duration_secs(),
            scan_interval: default_scan_interval_secs(),
            min_rssi: Some(-70),
            battery_refresh_interval: default_battery_refresh_interval(),
            auto_reconnect: default_true(),
            reconnect_attempts: default_reconnect_attempts(),
            adaptive_polling: default_true(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            show_notifications: default_true(),
            start_minimized: default_true(),
            theme: Theme::System,
            show_percentage_in_tray: default_true(),
            show_low_battery_warning: default_true(),
            low_battery_threshold: default_low_battery_threshold(),
            remember_window_position: default_true(),
            last_window_position: None,
            minimize_to_tray_on_close: default_true(),
            minimize_on_blur: default_false(),
            auto_hide_timeout: None,
        }
    }
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            launch_at_startup: false,
            log_level: LogLevel::Info,
            enable_telemetry: false,
        }
    }
}

impl Default for BatteryConfig {
    fn default() -> Self {
        Self {
            low_threshold: default_low_battery_threshold(),
            smoothing_enabled: default_true(),
            change_threshold: default_change_threshold(),
            notify_low: default_true(),
            notify_charged: default_true(),
        }
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}

impl AppConfig {
    /// Convert to scan config for the bluetooth scanner
    pub fn to_scan_config(&self) -> ScanConfig {
        ScanConfig::new()
            .with_scan_duration(self.bluetooth.scan_duration)
            .with_interval(self.bluetooth.scan_interval)
            .with_min_rssi(self.bluetooth.min_rssi)
            .with_continuous(true)
    }
    
    /// Load configuration from file, using a path derived from the default settings path
    ///
    /// This is a convenience wrapper that creates a ConfigManager internally.
    /// For more control, use ConfigManager directly.
    ///
    /// # Returns
    ///
    /// Result containing the loaded configuration or an error
    pub fn load() -> Result<Self, ConfigError> {
        let config_path = default_settings_path();
        
        // If the default config file doesn't exist, return the default configuration
        if !config_path.exists() {
            log::info!("Default config file does not exist, using defaults: {:?}", config_path);
            return Ok(Self::default());
        }
        
        Self::load_from_path(&config_path)
    }
    
    /// Load configuration from a specific file path
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file
    ///
    /// # Returns
    ///
    /// Result containing the loaded configuration or an error
    pub fn load_from_path<P: AsRef<std::path::Path>>(path: P) -> Result<Self, ConfigError> {
        use std::fs::File;
        use std::io::Read;
        
        let path = path.as_ref();
        
        // Check if the file exists - return IoError for non-existent files
        if !path.exists() {
            log::error!("Config file does not exist: {:?}", path);
            return Err(ConfigError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Config file not found: {:?}", path)
            )));
        }
        
        // Read the file
        let mut file = File::open(path)
            .map_err(|e| {
                log::error!("Failed to open config file: {}", e);
                ConfigError::IoError(e)
            })?;
        
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| {
                log::error!("Failed to read config file: {}", e);
                ConfigError::IoError(e)
            })?;
        
        // Deserialize
        let mut config: Self = serde_json::from_str(&contents)
            .map_err(|e| {
                log::error!("Failed to parse config file: {}", e);
                ConfigError::SerializationError(e)
            })?;
        
        // Set the settings path
        config.settings_path = path.to_path_buf();
        
        log::debug!("Loaded configuration from {:?}", path);
        Ok(config)
    }
    
    /// Save configuration to file
    ///
    /// This is a convenience wrapper that creates a ConfigManager internally.
    /// For more control, use ConfigManager directly.
    ///
    /// # Returns
    ///
    /// Result indicating success or an error
    pub fn save(&self) -> Result<(), ConfigError> {
        self.save_to_path(&self.settings_path)
    }
    
    /// Save configuration to a specific file path
    ///
    /// # Arguments
    ///
    /// * `path` - Path to save the configuration file
    ///
    /// # Returns
    ///
    /// Result indicating success or an error
    pub fn save_to_path<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), ConfigError> {
        use std::fs::{self, File};
        use std::io::Write;
        
        let path = path.as_ref();
        
        // Ensure the directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                log::debug!("Creating config directory: {:?}", parent);
                fs::create_dir_all(parent)
                    .map_err(|e| {
                        log::error!("Failed to create config directory: {}", e);
                        ConfigError::IoError(e)
                    })?;
            }
        }
        
        // Serialize
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| {
                log::error!("Failed to serialize config: {}", e);
                ConfigError::SerializationError(e)
            })?;
        
        // Write to file
        let mut file = File::create(path)
            .map_err(|e| {
                log::error!("Failed to create config file: {}", e);
                ConfigError::IoError(e)
            })?;
        
        file.write_all(json.as_bytes())
            .map_err(|e| {
                log::error!("Failed to write config file: {}", e);
                ConfigError::IoError(e)
            })?;
        
        log::debug!("Saved configuration to {:?}", path);
        Ok(())
    }
    
    /// Get the Bluetooth configuration section
    pub fn bluetooth(&self) -> &BluetoothConfig {
        &self.bluetooth
    }
    
    /// Get the UI configuration section
    pub fn ui(&self) -> &UiConfig {
        &self.ui
    }
    
    /// Get the system configuration section
    pub fn system(&self) -> &SystemConfig {
        &self.system
    }
    
    /// Get the battery configuration section
    pub fn battery(&self) -> &BatteryConfig {
        &self.battery
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate Bluetooth configuration
        self.bluetooth.validate()?;
        
        // Validate UI configuration
        self.ui.validate()?;
        
        // Validate system configuration
        self.system.validate()?;
        
        // Validate battery configuration
        self.battery.validate()?;
        
        Ok(())
    }
}

impl BluetoothConfig {
    /// Validate Bluetooth configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Scan duration should be between 1 and 60 seconds
        if self.scan_duration.as_secs() < 1 || self.scan_duration.as_secs() > 60 {
            return Err(ConfigError::ValidationFailed(
                "scan_duration".to_string(),
                format!("Scan duration must be between 1 and 60 seconds, got {}", self.scan_duration.as_secs())
            ));
        }
        
        // Scan interval should be between 5 seconds and 10 minutes
        if self.scan_interval.as_secs() < 5 || self.scan_interval.as_secs() > 600 {
            return Err(ConfigError::ValidationFailed(
                "scan_interval".to_string(),
                format!("Scan interval must be between 5 and 600 seconds, got {}", self.scan_interval.as_secs())
            ));
        }
        
        // If min_rssi is set, it should be in a reasonable range (-100 to 0)
        if let Some(rssi) = self.min_rssi {
            if rssi < -100 || rssi > 0 {
                return Err(ConfigError::ValidationFailed(
                    "min_rssi".to_string(),
                    format!("RSSI must be between -100 and 0, got {}", rssi)
                ));
            }
        }
        
        // Battery refresh interval should be between 1 and 300 seconds
        if self.battery_refresh_interval < 1 || self.battery_refresh_interval > 300 {
            return Err(ConfigError::ValidationFailed(
                "battery_refresh_interval".to_string(),
                format!("Battery refresh interval must be between 1 and 300 seconds, got {}", self.battery_refresh_interval)
            ));
        }
        
        // Reconnect attempts should be between 1 and 10
        if self.reconnect_attempts < 1 || self.reconnect_attempts > 10 {
            return Err(ConfigError::ValidationFailed(
                "reconnect_attempts".to_string(),
                format!("Reconnect attempts must be between 1 and 10, got {}", self.reconnect_attempts)
            ));
        }
        
        Ok(())
    }
}

impl UiConfig {
    /// Validate UI configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Low battery threshold should be between 1 and 100
        if self.low_battery_threshold < 1 || self.low_battery_threshold > 100 {
            return Err(ConfigError::ValidationFailed(
                "low_battery_threshold".to_string(),
                format!("Low battery threshold must be between 1 and 100, got {}", self.low_battery_threshold)
            ));
        }
        
        // Auto-hide timeout should be reasonable if set
        if let Some(timeout) = self.auto_hide_timeout {
            if timeout < 5 || timeout > 3600 {
                return Err(ConfigError::ValidationFailed(
                    "auto_hide_timeout".to_string(),
                    format!("Auto-hide timeout must be between 5 and 3600 seconds, got {}", timeout)
                ));
            }
        }
        
        Ok(())
    }
}

impl SystemConfig {
    /// Validate system configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // No specific validation for system config yet
        Ok(())
    }
}

impl BatteryConfig {
    /// Validate the battery configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate low threshold
        if self.low_threshold > 100 {
            return Err(ConfigError::ValidationFailed(
                "battery.low_threshold".to_string(),
                "Low battery threshold must be between 0 and 100".to_string(),
            ));
        }
        
        // Validate change threshold
        if self.change_threshold > 50 {
            return Err(ConfigError::ValidationFailed(
                "battery.change_threshold".to_string(),
                "Level change threshold must be between 0 and 50".to_string(),
            ));
        }
        
        Ok(())
    }
}

/// Configuration error
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    /// Lock error
    #[error("Failed to lock configuration")]
    LockError,
    
    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    /// Validation error
    #[error("Validation failed for {0}: {1}")]
    ValidationFailed(String, String),
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
    use serde_json;
    
    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        
        assert!(config.bluetooth.auto_scan_on_startup);
        assert_eq!(config.bluetooth.scan_duration, Duration::from_secs(5));
        assert_eq!(config.bluetooth.scan_interval, Duration::from_secs(30));
        assert_eq!(config.bluetooth.min_rssi, Some(-70));
        assert!(config.ui.show_notifications);
        assert!(config.ui.start_minimized);
        assert_eq!(config.ui.theme, Theme::System);
    }
    
    #[test]
    fn test_to_scan_config() {
        let config = AppConfig::default();
        let scan_config = config.to_scan_config();
        
        assert_eq!(scan_config.scan_duration, Duration::from_secs(5));
        assert_eq!(scan_config.interval_between_scans, Duration::from_secs(30));
        assert!(scan_config.continuous);
        assert_eq!(scan_config.min_rssi, Some(-70));
    }
    
    #[test]
    fn test_serialization() {
        let config = AppConfig::default();
        let json = serde_json::to_string_pretty(&config).unwrap();
        
        // Ensure json contains our fields
        assert!(json.contains("bluetooth"));
        assert!(json.contains("ui"));
        assert!(json.contains("system"));
        
        // Deserialize back and verify it matches
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.bluetooth.scan_duration, config.bluetooth.scan_duration);
        assert_eq!(deserialized.ui.theme, config.ui.theme);
    }
} 