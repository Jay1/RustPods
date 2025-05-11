//! BLE scanning and device management

mod scanner;
mod adapter;
mod examples;
mod scanner_config;
pub mod events;
pub mod battery;

pub use scanner::{
    BleScanner, BleError, DiscoveredDevice,
};

pub use adapter::{
    AdapterManager, AdapterInfo,
};

pub use scanner_config::ScanConfig;

pub use events::{
    EventBroker, EventFilter, SubscriberId, receiver_to_stream, BleEvent
};

pub use battery::{
    AirPodsBatteryStatus, extract_battery_status, start_battery_monitoring
};

// Export examples for testing
pub use examples::{
    discover_adapters, scan_with_adapter, interval_scanning,
    airpods_filtering
};

pub struct BluetoothManager;

pub trait BluetoothDevice {} 