// Root module exports
pub mod config;
pub mod bluetooth;
pub mod airpods;
pub mod ui;
pub mod errors;
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

// Re-export common items for convenience
pub use bluetooth::{BleScanner, BleEvent, EventBroker, EventFilter};
pub use airpods::{AirPodsType, AirPodsFilter, DetectedAirPods};

// Re-exports for convenience
pub use ui::{AppState, Message, run_ui};
pub use config::AppConfig;
pub use error::{RustPodsError, ErrorManager, ErrorSeverity, RecoveryAction};
pub use app_controller::AppController;
pub use logging::init_logger;
pub use telemetry::TelemetryManager;
pub use diagnostics::{DiagnosticsManager, DiagnosticLevel}; 