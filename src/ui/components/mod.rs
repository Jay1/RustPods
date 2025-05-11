//! UI components

// Expose modules for direct access
pub mod device_list;
pub mod battery_display;
pub mod header;

// Re-export components for convenience
pub use device_list::DeviceList;
pub use battery_display::BatteryDisplay;
pub use header::Header; 