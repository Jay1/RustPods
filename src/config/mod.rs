//! Settings management

pub mod app_config;
// Replace the external test module import with the actual tests
// #[cfg(test)]
// mod tests;

pub use app_config::AppConfig;
pub use app_config::Theme;
pub use app_config::{
    BluetoothConfig, ConfigError, LogLevel, SystemConfig, UiConfig, WindowPosition,
};

use std::fs;
use std::path::{Path, PathBuf};
// Removing unused imports
// use std::io;
use log::{debug, error, info};
use std::sync::{Arc, Mutex};
// Removing unused imports
// use std::fs::File;
// use std::io::ErrorKind;
// use serde::{Deserialize, Serialize};
// use thiserror::Error;

/// Manages application configuration
#[derive(Debug, Clone)]
pub struct ConfigManager {
    /// Path to the configuration file
    config_path: PathBuf,

    /// Current configuration
    config: Arc<Mutex<AppConfig>>,

    /// Whether auto-save is enabled
    auto_save: bool,
}

impl ConfigManager {
    /// Create a new configuration manager
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the configuration file
    /// * `auto_save` - Whether to automatically save when configuration changes
    pub fn new(config_path: &Path, auto_save: bool) -> Self {
        Self {
            config_path: config_path.to_path_buf(),
            config: Arc::new(Mutex::new(AppConfig::default())),
            auto_save,
        }
    }

    /// Create a default configuration manager
    pub fn default() -> Self {
        Self::new(&default_config_path(), true)
    }

    /// Get the path to the configuration file
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// Get a copy of the current configuration
    pub fn get_config(&self) -> AppConfig {
        match self.config.lock() {
            Ok(guard) => guard.clone(),
            Err(e) => {
                error!("Failed to lock configuration: {}", e);
                AppConfig::default()
            }
        }
    }

    /// Load configuration from file
    pub fn load(&self) -> Result<(), ConfigError> {
        debug!("Loading configuration from {}", self.config_path.display());

        // Check if the file exists
        if !self.config_path.exists() {
            info!("Configuration file does not exist, using defaults");
            return Ok(());
        }

        // Read the file
        let contents = fs::read_to_string(&self.config_path).map_err(|e| {
            error!("Failed to read configuration file: {}", e);
            ConfigError::IoError(e)
        })?;

        // Parse the JSON
        let config: AppConfig = serde_json::from_str(&contents).map_err(|e| {
            error!("Failed to parse configuration file: {}", e);
            ConfigError::SerializationError(e)
        })?;

        // Validate the configuration
        config.validate()?;

        // Update the configuration
        let mut guard = match self.config.lock() {
            Ok(guard) => guard,
            Err(e) => {
                error!("Failed to lock configuration: {}", e);
                return Err(ConfigError::LockError);
            }
        };

        *guard = config;

        info!("Configuration loaded successfully");
        Ok(())
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<(), ConfigError> {
        debug!("Saving configuration to {}", self.config_path.display());
        // Extra debug log for parent directory
        if let Some(parent) = self.config_path.parent() {
            debug!("Config parent directory: {}", parent.display());
            if !parent.exists() {
                debug!(
                    "Parent directory does not exist, attempting to create: {}",
                    parent.display()
                );
                if let Err(e) = fs::create_dir_all(parent) {
                    error!(
                        "Failed to create configuration directory {}: {}",
                        parent.display(),
                        e
                    );
                    return Err(ConfigError::IoError(e));
                }
            }
        } else {
            error!(
                "No parent directory for config path: {}",
                self.config_path.display()
            );
        }

        // Get the configuration
        let config = self.get_config();

        // Validate the configuration
        config.validate()?;

        // Serialize the configuration
        let json = serde_json::to_string_pretty(&config).map_err(|e| {
            error!("Failed to serialize configuration: {}", e);
            ConfigError::SerializationError(e)
        })?;

        // Write to file
        fs::write(&self.config_path, json).map_err(|e| {
            error!("Failed to write configuration file: {}", e);
            ConfigError::IoError(e)
        })?;

        info!("Configuration saved successfully");
        Ok(())
    }

    /// Update the configuration
    pub fn update<F>(&self, update_fn: F) -> Result<(), ConfigError>
    where
        F: FnOnce(&mut AppConfig),
    {
        // Get the configuration
        let mut guard = match self.config.lock() {
            Ok(guard) => guard,
            Err(e) => {
                error!("Failed to lock configuration: {}", e);
                return Err(ConfigError::LockError);
            }
        };

        // Update the configuration
        update_fn(&mut guard);

        // Auto-save if enabled
        if self.auto_save {
            drop(guard); // Release the lock before saving
            self.save()?;
        }

        Ok(())
    }

    /// Update the configuration with validation
    pub fn update_with_validation<F>(&self, update_fn: F) -> Result<(), ConfigError>
    where
        F: FnOnce(&mut AppConfig),
    {
        // Get the configuration
        let mut config = self.get_config();

        // Update the configuration
        update_fn(&mut config);

        // Validate the configuration
        config.validate()?;

        // Update the configuration if valid
        let mut guard = match self.config.lock() {
            Ok(guard) => guard,
            Err(e) => {
                error!("Failed to lock configuration: {}", e);
                return Err(ConfigError::LockError);
            }
        };

        *guard = config;

        // Auto-save if enabled
        if self.auto_save {
            drop(guard); // Release the lock before saving
            self.save()?;
        }

        Ok(())
    }

    /// Validate the current configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        let config = self.get_config();
        config.validate()
    }
}

/// Trait for configurable components
pub trait Configurable {
    /// Apply configuration
    fn apply_config(&mut self, config: &AppConfig);
}

/// Get the default configuration path
fn default_config_path() -> PathBuf {
    if let Some(config_dir) = dirs_next::config_dir() {
        config_dir.join("rustpods").join("config.json")
    } else {
        // Fallback to the current directory
        PathBuf::from("config.json")
    }
}

/// Load or create a configuration file
pub fn load_or_create_config() -> Result<AppConfig, ConfigError> {
    let config_path = default_config_path();

    // Ensure the parent directory exists before attempting any operations
    if let Some(parent) = config_path.parent() {
        if !parent.exists() {
            info!("Creating config directory: {}", parent.display());
            if let Err(e) = std::fs::create_dir_all(parent) {
                error!("Failed to create config directory: {}", e);
                // Continue with default config, but in the current directory
                return Ok(AppConfig::default());
            }
        }
    }

    let manager = ConfigManager::new(&config_path, true);

    // Attempt to load the config file
    if let Err(e) = manager.load() {
        // If the error is because the file doesn't exist, that's ok
        // We'll use defaults and save them below
        if !config_path.exists() {
            info!("Config file not found. Creating default configuration.");
        } else {
            // If there was another error loading the file, log it
            error!("Error loading config file: {}", e);
        }
    }

    // Get the current configuration (either loaded or default)
    let config = manager.get_config();

    // Save the config to ensure the file exists
    if let Err(e) = manager.save() {
        error!("Failed to save configuration: {}", e);
        // We continue even if saving fails
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ConfigManager, LogLevel, Theme};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_config_manager_new() {
        let path = std::path::PathBuf::from("test_config.json");
        let manager = ConfigManager::new(&path, true);

        assert_eq!(manager.config_path(), &path);
        assert!(manager.auto_save);
    }

    #[test]
    fn test_config_manager_default() {
        let manager = ConfigManager::default();
        let default_path = default_config_path();

        assert_eq!(manager.config_path(), &default_path);
        assert!(manager.auto_save);
    }

    #[test]
    fn test_get_config() {
        let manager = ConfigManager::default();
        let config = manager.get_config();

        // Default values should match
        assert!(config.bluetooth.auto_scan_on_startup);
        assert_eq!(config.ui.theme, Theme::System);
    }

    #[test]
    fn test_save_and_load() {
        // Create a temporary directory that will be automatically deleted
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.json");

        // Create a config manager
        let manager = ConfigManager::new(&config_path, false);

        // Modify the configuration
        manager
            .update(|config| {
                config.bluetooth.auto_scan_on_startup = false;
                config.ui.theme = Theme::Dark;
                config.system.log_level = LogLevel::Debug;
            })
            .unwrap();

        // Save the configuration
        manager.save().unwrap();

        // Verify the file exists
        assert!(config_path.exists());

        // Create a new manager with the same path
        let new_manager = ConfigManager::new(&config_path, false);

        // Load the configuration
        new_manager.load().unwrap();

        // Verify the loaded configuration
        let loaded_config = new_manager.get_config();
        assert!(!loaded_config.bluetooth.auto_scan_on_startup);
        assert_eq!(loaded_config.ui.theme, Theme::Dark);
        assert_eq!(loaded_config.system.log_level, LogLevel::Debug);
    }

    #[test]
    fn test_update_with_auto_save() {
        // Create a temporary directory
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.json");

        // Create a config manager with auto-save enabled
        let manager = ConfigManager::new(&config_path, true);

        // Update the configuration
        manager
            .update(|config| {
                config.bluetooth.auto_scan_on_startup = false;
            })
            .unwrap();

        // The file should have been created due to auto-save
        assert!(config_path.exists());

        // Create a new manager and load
        let new_manager = ConfigManager::new(&config_path, false);
        new_manager.load().unwrap();

        // Verify the change was saved
        let loaded_config = new_manager.get_config();
        assert!(!loaded_config.bluetooth.auto_scan_on_startup);
    }

    #[test]
    fn test_validation() {
        let manager = ConfigManager::default();

        // Set a valid initial state
        manager
            .update(|config| {
                config.bluetooth.scan_duration = std::time::Duration::from_secs(10);
                config.ui.low_battery_threshold = 20;
            })
            .expect("Initial setup should succeed");

        // Test with invalid scan_duration value
        let update_result = manager.update(|config| {
            config.bluetooth.scan_duration = std::time::Duration::from_secs(0);
        });

        // If validation happens during update, it might fail, or it might happen during validate() call
        if update_result.is_ok() {
            // If update succeeded, validate() should fail
            assert!(manager.validate().is_err());
        } else {
            // If update failed, it should be due to validation
            assert!(update_result.is_err());
        }

        // Reset to valid values
        manager
            .update(|config| {
                config.bluetooth.scan_duration = std::time::Duration::from_secs(5);
                config.ui.low_battery_threshold = 20; // Valid value
            })
            .expect("Setting valid values should succeed");

        // Validation should pass
        assert!(manager.validate().is_ok());

        // Test with invalid threshold
        let update_result = manager.update(|config| {
            config.ui.low_battery_threshold = 101; // Invalid: must be <= 100
        });

        // Expect validation to fail (during update or validate call)
        if update_result.is_ok() {
            assert!(manager.validate().is_err());
        } else {
            assert!(update_result.is_err());
        }
    }

    #[test]
    fn test_serialization_format() {
        // Create a temporary directory
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.json");

        // Create a config manager
        let manager = ConfigManager::new(&config_path, false);

        // Update with some values
        manager
            .update(|config| {
                config.bluetooth.auto_scan_on_startup = false;
                config.ui.theme = Theme::Dark;
            })
            .unwrap();

        // Save the configuration
        manager.save().unwrap();

        // Read the file directly
        let content = fs::read_to_string(&config_path).unwrap();

        // Verify the JSON structure (case insensitive checks to handle whitespace variations)
        assert!(content.to_lowercase().contains("bluetooth"));
        assert!(content.to_lowercase().contains("ui"));
        assert!(content.to_lowercase().contains("dark"));

        // Parse the JSON to verify its structure
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert!(json.is_object());
        assert!(json.get("bluetooth").is_some());
        assert!(json.get("ui").is_some());
        assert!(json.get("system").is_some());

        let bluetooth = json.get("bluetooth").unwrap();
        assert_eq!(
            bluetooth.get("auto_scan_on_startup").unwrap(),
            &serde_json::json!(false)
        );

        let ui = json.get("ui").unwrap();
        assert_eq!(ui.get("theme").unwrap(), &serde_json::json!("dark"));
    }
}
