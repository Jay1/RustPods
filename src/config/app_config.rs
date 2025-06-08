use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

use crate::bluetooth::ScanConfig;

/// Application configuration
///
/// The configuration file is always stored in the OS-standard config directory:
/// - Windows: %APPDATA%/rustpods/settings.json
/// - macOS: ~/Library/Application Support/rustpods/settings.json
/// - Linux: ~/.config/rustpods/settings.json
///
/// The `settings_path` field is used internally at runtime and is not persisted or user-configurable.
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

    /// Path to save settings (runtime only, not serialized)
    #[serde(skip)]
    pub(crate) settings_path: PathBuf,
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
    pub battery_refresh_interval: Duration,

    /// ID of the currently paired device
    #[serde(default)]
    pub paired_device_id: Option<String>,

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

    /// Auto-save interval in seconds (default: 5 minutes)
    #[serde(default)]
    pub auto_save_interval: Option<u64>,

    /// Create crash recovery snapshots
    #[serde(default = "default_true")]
    pub enable_crash_recovery: bool,
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
#[derive(Default)]
pub enum Theme {
    /// Light theme
    Light,
    /// Dark theme
    Dark,
    /// System theme (follows OS settings)
    #[serde(rename = "system")]
    #[default]
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

/// Log level
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum LogLevel {
    /// Error level - only errors are logged
    Error,
    /// Warning level - warnings and errors are logged
    Warn,
    /// Info level - informational messages, warnings, and errors are logged
    #[default]
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
fn default_true() -> bool {
    true
}
fn default_false() -> bool {
    false
}
fn default_scan_duration_secs() -> Duration {
    Duration::from_secs(5)
}
fn default_scan_interval_secs() -> Duration {
    Duration::from_secs(30)
}
fn default_auto_scan() -> bool {
    true
}
fn default_battery_refresh_interval() -> Duration {
    Duration::from_secs(10)
}
fn default_reconnect_attempts() -> u32 {
    3
}
fn default_low_battery_threshold() -> u8 {
    20
}
fn default_change_threshold() -> u8 {
    5
}

// Custom serialization for Duration
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

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
            paired_device_id: None,
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
            log_level: LogLevel::default(),
            enable_telemetry: false,
            auto_save_interval: Some(300), // 5 minutes default
            enable_crash_recovery: true,
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
        log::info!("Loading configuration from: {}", config_path.display());

        // If the default config file doesn't exist, return the default configuration
        if !config_path.exists() {
            log::info!(
                "Default config file does not exist, using defaults: {:?}",
                config_path
            );
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
        let path = path.as_ref();

        // If the file doesn't exist, return default config
        if !path.exists() {
            log::info!(
                "Configuration file not found at {}, using defaults",
                path.display()
            );
            return Ok(Self::default());
        }

        let file_content = match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => {
                    return Err(ConfigError::FileNotFound(path.to_path_buf()))
                }
                std::io::ErrorKind::PermissionDenied => {
                    return Err(ConfigError::PermissionDenied(path.to_path_buf()))
                }
                _ => return Err(ConfigError::IoError(e)),
            },
        };

        let mut config: Self =
            serde_json::from_str(&file_content).map_err(ConfigError::SerializationError)?;

        // Update the settings path
        config.settings_path = path.to_path_buf();

        // Validate the config
        config.validate()?;

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
        let path = path.as_ref();

        // Extra debug log for parent directory
        if let Some(parent) = path.parent() {
            log::debug!("AppConfig save: parent directory: {}", parent.display());
            if !parent.exists() {
                log::debug!(
                    "AppConfig save: parent directory does not exist, attempting to create: {}",
                    parent.display()
                );
                if let Err(e) = std::fs::create_dir_all(parent) {
                    log::error!(
                        "AppConfig save: failed to create directory {}: {}",
                        parent.display(),
                        e
                    );
                    return Err(ConfigError::IoError(e));
                }
            }
        } else {
            log::error!(
                "AppConfig save: no parent directory for config path: {}",
                path.display()
            );
        }

        // Validate before saving
        self.validate()?;

        // Convert to JSON
        let json =
            serde_json::to_string_pretty(self).map_err(|e| ConfigError::SerializationError(e))?;

        // Write to file with error handling
        std::fs::write(path, json).map_err(|e| match e.kind() {
            std::io::ErrorKind::PermissionDenied => {
                ConfigError::PermissionDenied(path.to_path_buf())
            }
            _ => ConfigError::IoError(e),
        })?;

        log::info!("Configuration saved to {}", path.display());
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
        // Validate each section
        self.bluetooth.validate().map_err(|e| match e {
            ConfigError::ValidationFailed(field, msg) => {
                ConfigError::ValidationFailed(format!("bluetooth.{}", field), msg)
            }
            _ => e,
        })?;

        self.ui.validate().map_err(|e| match e {
            ConfigError::ValidationFailed(field, msg) => {
                ConfigError::ValidationFailed(format!("ui.{}", field), msg)
            }
            _ => e,
        })?;

        self.system.validate().map_err(|e| match e {
            ConfigError::ValidationFailed(field, msg) => {
                ConfigError::ValidationFailed(format!("system.{}", field), msg)
            }
            _ => e,
        })?;

        self.battery.validate().map_err(|e| match e {
            ConfigError::ValidationFailed(field, msg) => {
                ConfigError::ValidationFailed(format!("battery.{}", field), msg)
            }
            _ => e,
        })?;

        Ok(())
    }
}

impl BluetoothConfig {
    /// Validate Bluetooth configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.scan_duration.as_secs() == 0 {
            return Err(ConfigError::ValidationFailed(
                "scan_duration".to_string(),
                "Scan duration must be greater than zero".to_string(),
            ));
        }

        if self.scan_interval.as_secs() == 0 {
            return Err(ConfigError::ValidationFailed(
                "scan_interval".to_string(),
                "Scan interval must be greater than zero".to_string(),
            ));
        }

        if self.battery_refresh_interval.as_secs() == 0 {
            return Err(ConfigError::ValidationFailed(
                "battery_refresh_interval".to_string(),
                "Battery refresh interval must be greater than zero".to_string(),
            ));
        }

        if let Some(rssi) = self.min_rssi {
            // RSSI is a negative value, and smaller values (e.g. -100) are weaker than larger values (e.g. -60)
            if rssi > 0 {
                return Err(ConfigError::ValidationFailed(
                    "min_rssi".to_string(),
                    "RSSI minimum value should be negative".to_string(),
                ));
            }

            if rssi < -100 {
                return Err(ConfigError::ValidationFailed(
                    "min_rssi".to_string(),
                    "RSSI minimum value should be greater than -100".to_string(),
                ));
            }
        }

        if self.reconnect_attempts > 10 {
            log::warn!(
                "High reconnect_attempts value ({}), this could cause delays",
                self.reconnect_attempts
            );
        }

        Ok(())
    }
}

impl UiConfig {
    /// Validate UI configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.low_battery_threshold > 100 {
            return Err(ConfigError::ValidationFailed(
                "low_battery_threshold".to_string(),
                "Low battery threshold cannot exceed 100%".to_string(),
            ));
        }

        if let Some(timeout) = self.auto_hide_timeout {
            if timeout < 5 {
                return Err(ConfigError::ValidationFailed(
                    "auto_hide_timeout".to_string(),
                    "Auto hide timeout should be at least 5 seconds".to_string(),
                ));
            }

            if timeout > 3600 {
                log::warn!(
                    "Very long auto-hide timeout ({}s), consider a shorter value",
                    timeout
                );
            }
        }

        Ok(())
    }
}

impl SystemConfig {
    /// Validate system configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        if let Some(interval) = self.auto_save_interval {
            if interval < 10 {
                return Err(ConfigError::ValidationFailed(
                    "auto_save_interval".to_string(),
                    "Auto-save interval should be at least 10 seconds".to_string(),
                ));
            }

            if interval > 3600 {
                log::warn!(
                    "Very long auto-save interval ({}s), consider a shorter value",
                    interval
                );
            }
        }

        Ok(())
    }
}

impl BatteryConfig {
    /// Validate the battery configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.low_threshold > 100 {
            return Err(ConfigError::ValidationFailed(
                "low_threshold".to_string(),
                "Low battery threshold cannot exceed 100%".to_string(),
            ));
        }

        if self.change_threshold > 50 {
            return Err(ConfigError::ValidationFailed(
                "change_threshold".to_string(),
                "Change threshold should not exceed 50%".to_string(),
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

    /// Path error
    #[error("Path error: {0}")]
    PathError(String),

    /// File not found error
    #[error("Configuration file not found: {0}")]
    FileNotFound(PathBuf),

    /// Permission denied
    #[error("Permission denied when accessing configuration file: {0}")]
    PermissionDenied(PathBuf),

    /// File system error
    #[error("File system error: {0}")]
    FileSystemError(String),
}

impl From<ConfigError> for crate::error::RustPodsError {
    fn from(err: ConfigError) -> Self {
        match err {
            ConfigError::IoError(e) => crate::error::RustPodsError::IoError(e.to_string()),
            ConfigError::SerializationError(e) => {
                crate::error::RustPodsError::ParseError(e.to_string())
            }
            ConfigError::LockError => {
                crate::error::RustPodsError::State("Failed to lock configuration".to_string())
            }
            ConfigError::InvalidConfig(msg) => crate::error::RustPodsError::Config(msg),
            ConfigError::ValidationFailed(field, msg) => {
                crate::error::RustPodsError::Validation(format!("{}: {}", field, msg))
            }
            ConfigError::PathError(msg) => crate::error::RustPodsError::Path(msg),
            ConfigError::FileNotFound(path) => crate::error::RustPodsError::FileNotFound(path),
            ConfigError::PermissionDenied(path) => {
                crate::error::RustPodsError::PermissionDenied(path.to_string_lossy().to_string())
            }
            ConfigError::FileSystemError(msg) => crate::error::RustPodsError::IoError(msg),
        }
    }
}

/// Get the default settings path
///
/// Returns the OS-standard config directory for RustPods:
/// - Windows: %APPDATA%/rustpods/settings.json
/// - macOS: ~/Library/Application Support/rustpods/settings.json
/// - Linux: ~/.config/rustpods/settings.json
fn default_settings_path() -> PathBuf {
    let path = dirs_next::config_dir()
        .map(|config_dir| {
            let path = config_dir.join("rustpods").join("settings.json");
            // Try to ensure the directory exists but don't fail if it doesn't
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    let _ = std::fs::create_dir_all(parent);
                }
            }
            path
        })
        .unwrap_or_else(|| PathBuf::from("settings.json")); // Fallback to current directory

    path
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
        assert_eq!(
            deserialized.bluetooth.scan_duration,
            config.bluetooth.scan_duration
        );
        assert_eq!(deserialized.ui.theme, config.ui.theme);
    }
}
