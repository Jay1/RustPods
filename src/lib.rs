// Module exports for library users
pub mod bluetooth;
pub mod airpods;
pub mod ui;
pub mod config;
pub mod app;
pub mod error;

// Re-export common items for convenience
pub use bluetooth::{BleScanner, BleEvent, EventBroker, EventFilter};
pub use airpods::{AirPodsType, AirPodsFilter, DetectedAirPods};

// Re-exports for convenience
pub use ui::{AppState, Message, run_ui};
pub use config::AppConfig;
pub use error::RustPodsError; 