//! BLE scanning and device management

pub mod scanner;
pub mod adapter;
pub mod examples;
mod scanner_config;
mod filter;
mod peripheral;
pub mod events;
pub mod battery;
pub mod battery_monitor;

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

pub use battery_monitor::{
    BatteryMonitor, BatteryMonitorOptions, BatteryAlert
};

// Export examples for testing
pub use examples::{
    discover_adapters, scan_with_adapter, interval_scanning,
    airpods_filtering
};

pub struct BluetoothManager;

pub trait BluetoothDevice {}

pub use adapter::BluetoothAdapter;
pub use adapter::BleAdapterEvent;
pub use adapter::AdapterCapabilities;
pub use adapter::AdapterStatus;
pub use filter::*;
pub use peripheral::*; 