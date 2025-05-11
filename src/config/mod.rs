//! Settings management

mod app_config;

pub use app_config::AppConfig;
pub use app_config::Theme;

/// Manages application configuration
pub struct ConfigManager;

/// Trait for configurable components
pub trait Configurable {
    /// Apply configuration
    fn apply_config(&mut self, config: &AppConfig);
} 