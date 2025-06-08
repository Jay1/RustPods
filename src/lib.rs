// Root module exports
pub mod airpods;
pub mod app_state_controller;
pub mod bluetooth;
pub mod config;
pub mod diagnostics;
pub mod lifecycle_manager;
pub mod logging;
pub mod state_persistence;
pub mod telemetry;
pub mod ui;

// Module exports for library users
pub mod app;
pub mod app_controller;
pub mod assets;
pub mod error;

// Re-export common items for convenience
pub use airpods::{AirPodsFilter, AirPodsType, DetectedAirPods};
pub use bluetooth::{BleEvent, BleScanner, EventBroker, EventFilter};

// Re-exports for convenience
pub use app_controller::AppController;
pub use config::AppConfig;
pub use diagnostics::{DiagnosticLevel, DiagnosticsManager};
pub use error::{ErrorManager, ErrorSeverity, RecoveryAction, RustPodsError};
pub use logging::configure_logging;
pub use telemetry::TelemetryManager;
pub use ui::{run_ui, AppState, Message};

/// Initialize logging with default settings
pub fn init_logging() {
    if let Err(e) = logging::configure_logging(config::LogLevel::Info, None, true) {
        eprintln!("Failed to initialize logging: {}", e);
    }
}
