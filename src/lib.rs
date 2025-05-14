// Root module exports
pub mod config;
pub mod bluetooth;
pub mod airpods;
pub mod ui;
pub mod state_persistence;
pub mod lifecycle_manager;
pub mod app_state_controller;
pub mod logging;
pub mod telemetry;
pub mod diagnostics;

// Module exports for library users
pub mod app;
pub mod error;
pub mod app_controller;
pub mod assets;

// Test modules
#[cfg(test)]
mod tests_mod;
#[cfg(test)]
mod airpods_tests;
#[cfg(test)]
mod bluetooth_tests;

// Re-export common items for convenience
pub use bluetooth::{BleScanner, BleEvent, EventBroker, EventFilter};
pub use airpods::{AirPodsType, AirPodsFilter, DetectedAirPods};

// Re-exports for convenience
pub use ui::{AppState, Message, run_ui};
pub use config::AppConfig;
pub use error::{RustPodsError, ErrorManager, ErrorSeverity, RecoveryAction};
pub use app_controller::AppController;
pub use logging::configure_logging;
pub use telemetry::TelemetryManager;
pub use diagnostics::{DiagnosticsManager, DiagnosticLevel};

/// Initialize logging with default settings
pub fn init_logging() {
    if let Err(e) = logging::configure_logging(config::LogLevel::Info, None, true) {
        eprintln!("Failed to initialize logging: {}", e);
    }
} 