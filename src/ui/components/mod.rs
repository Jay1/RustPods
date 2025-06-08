//! UI components

// Expose modules for direct access
pub mod battery_display;
pub mod battery_icon;
pub mod connection_status;
pub mod connection_status_wrapper;
pub mod context_menu;
pub mod device_list;
pub mod enhanced_battery_display;
pub mod header;
pub mod real_time_battery_display;
pub mod refresh_button;
pub mod settings_view;
pub mod svg_icons;

// Re-export components for convenience
pub use battery_display::BatteryDisplay;
pub use battery_icon::{battery_display_row, battery_icon_display, battery_with_label};
pub use connection_status::ConnectionStatus;
pub use connection_status_wrapper::ConnectionStatusWrapper;
pub use context_menu::{ContextMenu, ContextMenuItem};
pub use device_list::DeviceList;
pub use enhanced_battery_display::EnhancedBatteryDisplay;
pub use header::Header;
pub use real_time_battery_display::RealTimeBatteryDisplay;
pub use refresh_button::RefreshButton;
pub use settings_view::{BluetoothSetting, SettingsView, SystemSetting, UiSetting};
pub use svg_icons::{battery_icon_svg_string, refresh_icon_svg_string};
