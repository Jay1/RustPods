use iced::widget::{text, button, row, container, Column, Row};
use iced::{Element, Length, alignment};

use crate::ui::{Message, UiComponent};
use crate::bluetooth::AirPodsBatteryStatus;
use crate::ui::theme::Theme;
use crate::ui::components::enhanced_battery_display::EnhancedBatteryDisplay;

/// Component for the main application window
pub struct MainWindow {
    /// Battery status for the connected AirPods
    battery_status: Option<AirPodsBatteryStatus>,
    /// Whether the application is currently scanning
    is_scanning: bool,
    /// Whether a device is connected
    is_connected: bool,
    /// Name of the connected device
    device_name: Option<String>,
    /// Signal strength (RSSI) if available
    signal_strength: Option<i16>,
    /// Time connected in seconds
    connected_time: Option<u64>,
    /// Animation progress for refresh button
    refresh_animation_progress: f32,
}

impl MainWindow {
    /// Create a new main window with the given state
    pub fn new(
        battery_status: Option<AirPodsBatteryStatus>,
        is_scanning: bool,
        is_connected: bool,
        device_name: Option<String>,
        signal_strength: Option<i16>,
        connected_time: Option<u64>,
    ) -> Self {
        Self {
            battery_status,
            is_scanning,
            is_connected,
            device_name,
            signal_strength,
            connected_time,
            refresh_animation_progress: 0.0,
        }
    }
    
    /// Create an empty main window
    pub fn empty() -> Self {
        Self {
            battery_status: None,
            is_scanning: false,
            is_connected: false,
            device_name: None,
            signal_strength: None,
            connected_time: None,
            refresh_animation_progress: 0.0,
        }
    }
    
    /// Set the animation progress for refresh button
    pub fn with_animation_progress(mut self, progress: f32) -> Self {
        self.refresh_animation_progress = progress;
        self
    }
}

impl UiComponent for MainWindow {
    fn view(&self) -> Element<'static, Message, iced::Renderer<Theme>> {
        // Create header with title
        let title = text("RustPods Battery Monitor")
            .size(28)
            .width(Length::Fill)
            .horizontal_alignment(alignment::Horizontal::Center);
            
        // Create status display
        let status_text = if self.is_scanning {
            "Scanning for devices..."
        } else if self.is_connected {
            "Connected"
        } else {
            "No device connected"
        };
        
        let status = text(status_text)
            .size(16)
            .width(Length::Fill)
            .horizontal_alignment(alignment::Horizontal::Center);
            
        // Create device information if connected
        let device_info: Element<'static, Message, iced::Renderer<Theme>> = if let Some(name) = &self.device_name {
            let name_text = format!("Device: {}", name);
            let mut device_row: Row<'static, Message, iced::Renderer<Theme>> = Row::new()
                .spacing(5)
                .width(Length::Fill)
                .push(text(name_text).size(16));
                
            // Add signal strength if available
            if let Some(rssi) = self.signal_strength {
                let signal_text = format!("Signal: {}dBm", rssi);
                device_row = device_row.push(text(signal_text).size(16));
            }
            
            // Add connected time if available
            if let Some(time) = self.connected_time {
                let time_text = format!("Connected for: {}s", time);
                device_row = device_row.push(text(time_text).size(16));
            }
            
            device_row.into()
        } else {
            // No device connected
            text("No device selected")
                .size(16)
                .width(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Center)
                .into()
        };
        
        // Create action buttons
        let action_text = if self.is_scanning {
            "Stop Scan"
        } else {
            "Start Scan"
        };
        
        let scan_button = button(text(action_text))
            .on_press(
                if self.is_scanning {
                    Message::StopScan
                } else {
                    Message::StartScan
                }
            );
            
        let settings_button = button(text("Settings"))
            .on_press(Message::OpenSettings);
            
        let exit_button = button(text("Exit"))
            .on_press(Message::Exit);
            
        let action_buttons = row![
            scan_button,
            iced::widget::Space::with_width(Length::Fill),
            settings_button,
            exit_button,
        ]
        .spacing(10)
        .padding(5);
        
        // Create battery display
        let battery_display = if let Some(status) = &self.battery_status {
            // If we have battery status, show the enhanced display
            let display = EnhancedBatteryDisplay::new(Some(status.battery.clone()));
            display.view()
        } else {
            // No battery status available
            text("Battery information not available")
                .size(16)
                .width(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Center)
                .into()
        };
        
        // Combine all elements into the main container
        let content = Column::new()
            .push(title)
            .push(status)
            .push(device_info)
            .push(battery_display)
            .push(action_buttons)
            .spacing(20)
            .padding(20)
            .width(Length::Fill);
            
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::airpods::{AirPodsBattery, ChargingStatus};
    
    #[test]
    fn test_main_window_creation() {
        // Create with battery status
        let battery = AirPodsBattery {
            left: Some(80),
            right: Some(75),
            case: Some(90),
            charging: ChargingStatus {
                left: false,
                right: false,
                case: true,
            },
        };
        
        let battery_status = AirPodsBatteryStatus::new(battery);
        
        let window = MainWindow::new(
            Some(battery_status),
            false,
            true,
            Some("My AirPods".to_string()),
            Some(-65),
            Some(360),
        );
        
        // Verify fields
        assert!(window.battery_status.is_some());
        assert!(!window.is_scanning);
        assert!(window.is_connected);
        assert_eq!(window.device_name, Some("My AirPods".to_string()));
        assert_eq!(window.signal_strength, Some(-65));
        assert_eq!(window.connected_time, Some(360));
        assert_eq!(window.refresh_animation_progress, 0.0);
        
        // Test empty creation
        let empty_window = MainWindow::empty();
        assert!(empty_window.battery_status.is_none());
        assert!(!empty_window.is_scanning);
        assert!(!empty_window.is_connected);
        assert!(empty_window.device_name.is_none());
        assert!(empty_window.signal_strength.is_none());
        assert!(empty_window.connected_time.is_none());
    }
} 