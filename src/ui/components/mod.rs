//! UI components

// Expose modules for direct access
pub mod airpods_popup;
pub mod battery_icon;
pub mod battery_indicator;
pub mod settings_view;
pub mod svg_icons;

// Re-export components for convenience
pub use airpods_popup::AirPodsPopup;
pub use battery_icon::{battery_display_row, battery_icon_display, battery_with_label, view_circular_battery_widget};
pub use battery_indicator::view as battery_indicator_view;
pub use settings_view::{BluetoothSetting, SettingsView, SystemSetting, UiSetting};
pub use svg_icons::{battery_icon_svg_string, refresh_icon_svg_string};
