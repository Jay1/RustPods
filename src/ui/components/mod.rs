//! UI components

// Expose modules for direct access
pub mod device_list;
pub mod battery_display;
pub mod enhanced_battery_display;
pub mod battery_icon;
pub mod header;
pub mod settings_view;
pub mod connection_status;
pub mod connection_status_wrapper;
pub mod svg_icons;
pub mod refresh_button;
pub mod real_time_battery_display;
pub mod context_menu;

// Re-export components for convenience
pub use device_list::DeviceList;
pub use battery_display::BatteryDisplay;
pub use enhanced_battery_display::EnhancedBatteryDisplay;
pub use battery_icon::{battery_display_row, battery_icon_display, battery_with_label};
pub use header::Header;
pub use settings_view::{SettingsView, BluetoothSetting, UiSetting, SystemSetting};
pub use connection_status::ConnectionStatus;
pub use connection_status_wrapper::ConnectionStatusWrapper;
pub use refresh_button::RefreshButton;
pub use svg_icons::{refresh_icon_svg_string, battery_icon_svg_string};
pub use real_time_battery_display::RealTimeBatteryDisplay;
pub use context_menu::{ContextMenu, ContextMenuItem}; 