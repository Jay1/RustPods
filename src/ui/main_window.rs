//! Main application window for RustPods
//! 
//! Implements the main UI window component with device list and battery status display.

use std::sync::Arc;
use iced::{
    widget::{button, column, container, row, text, tooltip, scrollable, vertical_space},
    Element, Length, Command, alignment, Alignment
};
use iced::alignment::Horizontal;

use crate::bluetooth::DiscoveredDevice;
use crate::airpods::{AirPodsType, DetectedAirPods, AirPodsBattery};
use crate::ui::components::{battery_display_row, battery_icon_display, battery_with_label};
use crate::ui::components::{ConnectionStatusWrapper, RealTimeBatteryDisplay};
use crate::ui::Message;
use crate::ui::theme::Theme;
use crate::config::AppConfig;
use crate::ui::UiComponent;

/// Main window component
#[derive(Debug, Clone)]
pub struct MainWindow {
    /// Selected device (if any)
    pub selected_device: Option<DetectedAirPods>,
    
    /// List of discovered devices
    pub devices: Vec<DiscoveredDevice>,
    
    /// Whether a scan is currently in progress
    pub is_scanning: bool,
    
    /// Animation progress (0.0-1.0)
    pub animation_progress: f32,
    
    /// Application configuration
    pub config: AppConfig,
    
    /// Whether advanced display mode is enabled
    pub advanced_display_mode: bool,
}

impl MainWindow {
    /// Create an empty main window
    pub fn empty() -> Self {
        Self::new()
    }
    
    /// Create a new main window
    pub fn new() -> Self {
        Self {
            selected_device: None,
            devices: Vec::new(),
            is_scanning: false,
            animation_progress: 0.0,
            config: AppConfig::default(),
            advanced_display_mode: false,
        }
    }
    
    /// Set the animation progress and return a new instance
    pub fn with_animation_progress(mut self, progress: f32) -> Self {
        self.animation_progress = progress;
        self
    }
    
    /// Set the window size and return a new instance
    pub fn with_window_size(self, _size: (u32, u32)) -> Self {
        // Window size not directly used in MainWindow, but needed for compatibility
        self
    }
    
    /// Toggle advanced display mode
    pub fn toggle_advanced_display(&mut self) -> Command<Message> {
        self.advanced_display_mode = !self.advanced_display_mode;
        Command::none()
    }
    
    /// Toggle advanced display mode and return a command
    pub fn toggle_display_mode(&mut self) -> Command<Message> {
        self.toggle_advanced_display();
        Command::none()
    }
    
    /// Update the window with a new device
    pub fn update_device(&mut self, device: DiscoveredDevice) -> Command<Message> {
        // Find and update existing device or add new one
        if let Some(existing) = self.devices.iter_mut().find(|d| d.address == device.address) {
            *existing = device;
        } else {
            self.devices.push(device);
        }
        Command::none()
    }
    
    /// Remove a device from the list
    pub fn remove_device(&mut self, address: String) -> Command<Message> {
        self.devices.retain(|d| d.address.to_string() != address);
        Command::none()
    }
    
    /// Connect to a device
    pub fn connect_to_device(&mut self, airpods: DetectedAirPods) -> Command<Message> {
        self.selected_device = Some(airpods);
        Command::none()
    }
    
    /// Disconnect from the current device
    pub fn disconnect(&mut self) -> Command<Message> {
        self.selected_device = None;
        Command::none()
    }
    
    /// Update scan status
    pub fn set_scanning(&mut self, scanning: bool) -> Command<Message> {
        self.is_scanning = scanning;
        Command::none()
    }
    
    /// Update animation progress
    pub fn update_animation(&mut self, progress: f32) -> Command<Message> {
        self.animation_progress = progress;
        Command::none()
    }
    
    /// Update battery status
    pub fn update_battery(&mut self, battery: AirPodsBattery) -> Command<Message> {
        if let Some(device) = &mut self.selected_device {
            device.battery = battery;
        }
        Command::none()
    }
    
    /// Set connection transition animation progress and return a new instance
    pub fn with_connection_transition(mut self, progress: f32) -> Self {
        self.animation_progress = progress;
        self
    }
    
    // Add a helper method to create the header section containing title and connection status
    fn create_header(&self) -> Element<'_, Message, iced::Renderer<Theme>> {
        let title = text("RustPods")
            .size(30)
            .horizontal_alignment(Horizontal::Center);
        
        column![
            title,
            iced::widget::Space::new(Length::Fill, Length::Fixed(10.0)),
        ]
        .width(Length::Fill)
        .align_items(iced::Alignment::Center)
        .into()
    }
    
    // Update the view method to use the helper methods
    fn view_content(&self) -> Element<'_, Message, iced::Renderer<Theme>> {
        if let Some(device) = &self.selected_device {
            // Display connected device info
            self.render_connected_device(device)
        } else {
            // Display device list
            self.render_device_list()
        }
    }
}

impl UiComponent for MainWindow {
    fn view(&self) -> Element<'_, Message, iced::Renderer<Theme>> {
        let header = self.create_header();
        
        // Create the connection status wrapper that can be used directly in the column
        let connection_status = ConnectionStatusWrapper::new(
            self.selected_device.is_some(),
            self.is_scanning
        )
        .with_animation_progress(self.animation_progress);
        
        let content = self.view_content();
        
        // Create column that contains all the main window content
        let main_content = column![
            header,
            connection_status,
            iced::widget::Space::new(Length::Fill, Length::Fixed(20.0)),
            content
        ]
        .padding(20)
        .spacing(10)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_items(iced::Alignment::Center);

        container(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
    
/// Render connected device information
impl MainWindow {
    fn render_connected_device(&self, device: &DetectedAirPods) -> Element<Message, iced::Renderer<Theme>> {
        let device_name = text(device.name.as_ref().unwrap_or(&"Unknown Device".to_string()))
            .size(24);
            
        let device_type = text(format!("Type: {:?}", device.device_type))
            .size(16);
            
        // Battery status rows
        let left_pod = battery_display_row(
            "Left", 
            device.battery.left, 
            device.battery.charging.left,
            self.animation_progress
        );
        
        let right_pod = battery_display_row(
            "Right", 
            device.battery.right, 
            device.battery.charging.right,
            self.animation_progress
        );
        
        let case = battery_display_row(
            "Case", 
            device.battery.case, 
            device.battery.charging.case,
            self.animation_progress
        );
        
        // Battery icons if in advanced mode
        let battery_icons = if self.advanced_display_mode {
            row![
                battery_with_label("L", device.battery.left, device.battery.charging.left, 60.0, self.animation_progress),
                battery_with_label("R", device.battery.right, device.battery.charging.right, 60.0, self.animation_progress),
                battery_with_label("C", device.battery.case, device.battery.charging.case, 60.0, self.animation_progress),
            ]
            .spacing(20)
            .padding(10)
        } else {
            row![]
        };
        
        // Disconnect button
        let disconnect_button = button(text("Disconnect"))
            .on_press(Message::DeviceDisconnected);
        
        // Toggle display mode button
        let toggle_mode_button = button(
            text(if self.advanced_display_mode { "Simple View" } else { "Advanced View" })
        )
        .on_press(Message::ToggleDisplayMode);
        
        // Combine all elements
        column![
            device_name,
            device_type,
            iced::widget::Space::new(Length::Fill, Length::Fixed(20.0)),
            left_pod,
            right_pod,
            case,
            iced::widget::Space::new(Length::Fill, Length::Fixed(20.0)),
            battery_icons,
            iced::widget::Space::new(Length::Fill, Length::Fixed(20.0)),
            row![
                disconnect_button,
                iced::widget::Space::new(Length::Fill, Length::Fixed(10.0)),
                toggle_mode_button,
            ],
        ]
        .padding(20)
        .spacing(10)
        .width(Length::Fill)
        .into()
    }
    
    /// Render device list
    fn render_device_list(&self) -> Element<Message, iced::Renderer<Theme>> {
        let devices_list = if self.devices.is_empty() {
            // No devices found message
            let empty_message: Element<Message, iced::Renderer<Theme>> = text("No devices found. Press Scan to search for devices.")
                .size(16)
                .into();
                
            empty_message
        } else {
            let devices_column = self.devices.iter().fold(
                column![].spacing(10),
                |column, device| {
                    column.push(self.render_device_item(device))
                }
            );
            
            scrollable(devices_column)
                .height(Length::Fill)
                .into()
        };
        
        // Scan button
        let scan_button = button(
            text(if self.is_scanning { "Stop Scan" } else { "Scan for Devices" })
        )
        .on_press(
            if self.is_scanning { 
                Message::StopScan 
            } else { 
                Message::StartScan 
            }
        );
        
        column![
            text("Available Devices")
                .size(20),
            iced::widget::Space::new(Length::Fill, Length::Fixed(10.0)),
            devices_list,
            iced::widget::Space::new(Length::Fill, Length::Fixed(20.0)),
            scan_button,
        ]
        .padding(20)
        .spacing(10)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
    
    /// Render a device item in the list
    fn render_device_item(&self, device: &DiscoveredDevice) -> Element<Message, iced::Renderer<Theme>> {
        let device_name = device.name.as_ref()
            .map(|s| s.clone())
            .unwrap_or_else(|| "Unknown Device".to_string());
        let address = device.address.to_string();
        
        // Device name and address
        let device_info = column![
            text(&device_name).size(16),
            text(&address).size(12),
        ]
        .spacing(5)
        .width(Length::Fill);
        
        // Signal strength indicator (RSSI)
        let rssi_text = match device.rssi {
            Some(rssi) => format!("{}dBm", rssi),
            None => "Unknown".to_string(),
        };
        let rssi = text(rssi_text).size(14);
        
        // Connect button
        let connect_button = button(text("Connect"))
            .on_press(Message::SelectDevice(address.clone()));
        
        // Combine elements
        container(
            row![
                device_info,
                rssi,
                connect_button,
            ]
            .spacing(10)
            .align_items(Alignment::Center)
        )
        .padding(10)
        .width(Length::Fill)
        .into()
    }
}
