//! Main application window for RustPods
//! 
//! Implements the main UI window component with device list and battery status display.

use iced::{
    widget::{button, column, container, row, text, scrollable, Svg},
    Element, Length, Command
};
use iced::alignment::Horizontal;
use iced::widget::svg::Handle;

use crate::bluetooth::AirPodsBatteryStatus;
use crate::airpods::DetectedAirPods;
use crate::ui::Message;
use crate::ui::theme;
use crate::config::AppConfig;
use crate::ui::UiComponent;
use crate::ui::components::svg_icons::{settings_icon_svg_string, headset_icon_svg_string};
use crate::ui::theme::Theme;
use crate::ui::state::MergedBluetoothDevice;

/// Main window component
#[derive(Debug, Clone)]
pub struct MainWindow {
    /// Selected device (if any)
    pub selected_device: Option<DetectedAirPods>,
    
    /// List of discovered devices
    pub merged_devices: Vec<MergedBluetoothDevice>,
    
    /// Whether a scan is currently in progress
    pub is_scanning: bool,
    
    /// Animation progress (0.0-1.0)
    pub animation_progress: f32,
    
    /// Application configuration
    pub config: AppConfig,
    
    /// Whether advanced display mode is enabled
    pub advanced_display_mode: bool,
}

impl Default for MainWindow {
    fn default() -> Self {
        Self::new()
    }
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
            merged_devices: Vec::new(),
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
        let _ = self.toggle_advanced_display();
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
    pub fn update_battery(&mut self, battery_status: AirPodsBatteryStatus) -> Command<Message> {
        if let Some(device) = &mut self.selected_device {
            device.battery = Some(battery_status.battery);
        }
        Command::none()
    }
    
    /// Set connection transition animation progress and return a new instance
    pub fn with_connection_transition(mut self, progress: f32) -> Self {
        self.animation_progress = progress;
        self
    }
    
    /// Set battery status and return a new instance
    pub fn with_battery_status(mut self, battery_status: AirPodsBatteryStatus) -> Self {
        if let Some(device) = &mut self.selected_device {
            device.battery = Some(battery_status.battery);
        }
        self
    }
    
    // Add a helper method to create the header section containing title and connection status
    fn create_header(&self) -> Element<'_, Message, iced::Renderer<Theme>> {
        let title = text("RustPods")
            .size(30)
            .horizontal_alignment(Horizontal::Center)
            .style(crate::ui::theme::TEXT);
        
        // SVG settings icon with theme-driven color
        let svg_data = settings_icon_svg_string(theme::settings_icon_color(&Theme::CatppuccinMocha)).into_bytes();
        let svg_icon = Svg::new(Handle::from_memory(svg_data))
            .width(24)
            .height(24);
        let settings_button = button(svg_icon)
            .padding(8)
            .on_press(Message::OpenSettings)
            .style(crate::ui::theme::button_style());
        let exit_button = button(text("Exit")
            .size(16)
            .style(theme::TEXT))
            .padding([6, 16])
            .on_press(Message::Exit)
            .style(crate::ui::theme::lavender_button_style());
        row![
            title,
            iced::widget::Space::with_width(Length::Fill),
            settings_button,
            iced::widget::Space::with_width(8),
            exit_button
        ]
        .width(Length::Fill)
        .align_items(iced::Alignment::Center)
        .padding(10)
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
        let show_no_device = self.merged_devices.is_empty() && self.selected_device.is_none();
        let no_device_msg = if show_no_device {
            Some(
                iced::widget::text("No device connected")
                    .size(20)
                    .style(crate::ui::theme::ROSEWATER)
            )
        } else {
            None
        };
        let content = self.view_content();
        let mut main_content = column![header];
        if let Some(msg) = no_device_msg {
            main_content = main_content.push(msg);
        }
        main_content = main_content
            .push(content)
            .padding(20)
            .spacing(10)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(iced::Alignment::Center);
        container(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(crate::ui::theme::device_row_style())
            .into()
    }
}
    
/// Render connected device information
impl MainWindow {
    fn render_connected_device(&self, device: &DetectedAirPods) -> Element<Message, iced::Renderer<Theme>> {
        let device_name = text(device.name.as_ref().unwrap_or(&"Unknown Device".to_string()))
            .size(24)
            .style(crate::ui::theme::TEXT);
            
        let device_type = text(format!("Type: {:?}", device.device_type))
            .size(16)
            .style(crate::ui::theme::TEXT);
            
        // Battery status rows - handle Option<AirPodsBattery> properly
        let mut rows = vec![];
        
        rows.push(device_name.into());
        rows.push(device_type.into());
        
        // Add battery information if available
        if let Some(battery) = &device.battery {
            // Left earbud
            if let Some(left) = battery.left {
                let left_charging = battery.charging.as_ref()
                    .and_then(|c| c.is_left_charging().then_some(true))
                    .unwrap_or(false);
                    
                let left_status = format!(
                    "Left Earbud: {}% {}", 
                    left,
                    if left_charging { "(Charging)" } else { "" }
                );
                rows.push(text(left_status).size(16).style(crate::ui::theme::TEXT).into());
            }
            
            // Right earbud
            if let Some(right) = battery.right {
                let right_charging = battery.charging.as_ref()
                    .and_then(|c| c.is_right_charging().then_some(true))
                    .unwrap_or(false);
                    
                let right_status = format!(
                    "Right Earbud: {}% {}", 
                    right,
                    if right_charging { "(Charging)" } else { "" }
                );
                rows.push(text(right_status).size(16).style(crate::ui::theme::TEXT).into());
            }
            
            // Case
            if let Some(case) = battery.case {
                let case_charging = battery.charging.as_ref()
                    .and_then(|c| c.is_case_charging().then_some(true))
                    .unwrap_or(false);
                    
                let case_status = format!(
                    "Case: {}% {}", 
                    case,
                    if case_charging { "(Charging)" } else { "" }
                );
                rows.push(text(case_status).size(16).style(crate::ui::theme::TEXT).into());
            }
        } else {
            // No battery information available
            rows.push(text("Battery information not available").size(16).style(crate::ui::theme::TEXT).into());
        }
        
        // Add address at the bottom
        rows.push(text(format!("Address: {}", device.address)).size(12).style(crate::ui::theme::TEXT).into());
        
        // Disconnect button at the bottom
        let disconnect_button = button(
            text("Disconnect")
                .horizontal_alignment(iced::alignment::Horizontal::Center)
                .size(16)
                .style(crate::ui::theme::TEXT)
        )
        .padding(10)
        .width(Length::Fixed(120.0))
        .on_press(Message::DeviceDisconnected);
        
        rows.push(disconnect_button.into());
        
        // Create column with all the rows
        container(
            column(rows)
                .spacing(10)
                .align_items(iced::Alignment::Center)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .padding(20)
        .into()
    }
    
    /// Render device list
    fn render_device_list(&self) -> Element<Message, iced::Renderer<Theme>> {
        let mut col = column![];
        let devices_list = if self.merged_devices.is_empty() {
            let empty_message: Element<Message, iced::Renderer<Theme>> = text("No devices found.")
                .size(16)
                .style(crate::ui::theme::TEXT)
                .into();
            empty_message
        } else {
            let devices_column = self.merged_devices.iter().fold(
                column![].spacing(10),
                |column, device| {
                    column.push(self.render_merged_device_item(device))
                }
            );
            scrollable(devices_column)
                .height(Length::Fill)
                .into()
        };
        col = col
            .push(text("Available Devices").size(20).style(crate::ui::theme::TEXT))
            .push(iced::widget::Space::new(Length::Fill, Length::Fixed(10.0)))
            .push(devices_list)
            .push(iced::widget::Space::new(Length::Fill, Length::Fixed(20.0)));
        col
            .padding(20)
            .spacing(10)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
    
    /// Render a device item in the list
    fn render_merged_device_item(&self, device: &MergedBluetoothDevice) -> Element<Message, iced::Renderer<Theme>> {
        use iced::widget::{Image, Svg, Space, Container, Row, Column, Button};
        use iced::widget::svg::Handle;
        use iced::{Length, Alignment};
        use crate::ui::theme;
        // --- Icon selection logic ---
        let device_type = device.device_type.as_deref().unwrap_or("Unknown").to_lowercase();
        let name_lower = device.name.to_lowercase();
        let icon_element: iced::Element<'_, Message, iced::Renderer<Theme>> = if device.device_subtype.as_deref() == Some("case") {
            Image::new("assets/icons/hw/airpodscase.png").width(32).height(32).into()
        } else if name_lower.contains("airpods") {
            Image::new("assets/icons/hw/airpods.png").width(32).height(32).into()
        } else if name_lower.contains("sony") {
            Image::new("assets/icons/hw/sony.png").width(32).height(32).into()
        } else if name_lower.contains("sennheiser") {
            Image::new("assets/icons/hw/Sennheiser.png").width(32).height(32).into()
        } else if name_lower.contains("bose") {
            Image::new("assets/icons/hw/Bose.png").width(32).height(32).into()
        } else if name_lower.contains("beats") {
            Image::new("assets/icons/hw/beats.png").width(32).height(32).into()
        } else if name_lower.contains("jabra") {
            Image::new("assets/icons/hw/jabra.png").width(32).height(32).into()
        } else if name_lower.contains("anker") {
            Image::new("assets/icons/hw/anker.png").width(32).height(32).into()
        } else if name_lower.contains("earfun") {
            Image::new("assets/icons/hw/earfun.png").width(32).height(32).into()
        } else if name_lower.contains("technics") {
            Image::new("assets/icons/hw/technics.png").width(32).height(32).into()
        } else if name_lower.contains("google") {
            Image::new("assets/icons/hw/google.png").width(32).height(32).into()
        } else if name_lower.contains("samsung") {
            Image::new("assets/icons/hw/samsung.png").width(32).height(32).into()
        } else {
            let svg_data = headset_icon_svg_string();
            let svg_handle = Handle::from_memory(svg_data.into_bytes());
            Svg::new(svg_handle).width(32).height(32).into()
        };
        // --- Device info ---
        let device_name = if device.name.trim().is_empty() {
            "Unknown Device".to_string()
        } else {
            device.name.clone()
        };
        let address = &device.address;
        let paired = device.paired;
        let connected = device.connected;
        // Status badge
        let (status_str, status_color) = if connected {
            ("Connected", theme::GREEN)
        } else if paired {
            ("Paired", theme::YELLOW)
        } else {
            ("Not Paired", theme::RED)
        };
        // Battery badge(s)
        let mut battery_row = Row::new();
        if let Some(left) = device.left_battery {
            battery_row = battery_row.push(Container::new(text(format!("L: {}%", left)).size(12).style(theme::TEXT)).padding([2, 8]).style(theme::badge_style(theme::BLUE)));
            if let Some(in_ear) = device.left_in_ear {
                battery_row = battery_row.push(Container::new(text(format!("L in-ear: {}", if in_ear { "Yes" } else { "No" })).size(12).style(theme::TEXT)).padding([2, 8]).style(theme::badge_style(theme::GREEN)));
            }
        }
        if let Some(right) = device.right_battery {
            battery_row = battery_row.push(Container::new(text(format!("R: {}%", right)).size(12).style(theme::TEXT)).padding([2, 8]).style(theme::badge_style(theme::BLUE)));
            if let Some(in_ear) = device.right_in_ear {
                battery_row = battery_row.push(Container::new(text(format!("R in-ear: {}", if in_ear { "Yes" } else { "No" })).size(12).style(theme::TEXT)).padding([2, 8]).style(theme::badge_style(theme::GREEN)));
            }
        }
        if let Some(case) = device.case_battery {
            battery_row = battery_row.push(Container::new(text(format!("Case: {}%", case)).size(12).style(theme::TEXT)).padding([2, 8]).style(theme::badge_style(theme::BLUE)));
            if let Some(lid_open) = device.case_lid_open {
                battery_row = battery_row.push(Container::new(text(format!("Lid open: {}", if lid_open { "Yes" } else { "No" })).size(12).style(theme::TEXT)).padding([2, 8]).style(theme::badge_style(theme::GREEN)));
            }
        }
        // Advanced/diagnostic fields
        if self.advanced_display_mode {
            if let Some(side) = &device.side {
                battery_row = battery_row.push(Container::new(text(format!("Side: {}", side)).size(12).style(theme::TEXT)).padding([2, 8]).style(theme::badge_style(theme::YELLOW)));
            }
            if let Some(both_in_case) = device.both_in_case {
                battery_row = battery_row.push(Container::new(text(format!("Both in case: {}", if both_in_case { "Yes" } else { "No" })).size(12).style(theme::TEXT)).padding([2, 8]).style(theme::badge_style(theme::YELLOW)));
            }
            if let Some(color) = &device.color {
                battery_row = battery_row.push(Container::new(text(format!("Color: {}", color)).size(12).style(theme::TEXT)).padding([2, 8]).style(theme::badge_style(theme::YELLOW)));
            }
            if let Some(switch_count) = device.switch_count {
                battery_row = battery_row.push(Container::new(text(format!("Switches: {}", switch_count)).size(12).style(theme::TEXT)).padding([2, 8]).style(theme::badge_style(theme::YELLOW)));
            }
        }
        // --- Layout ---
        let info_col = Column::new()
            .push(
                text(device_name)
                    .size(16)
                    .style(theme::TEXT)
            )
            .push(
                text(address)
                    .size(12)
                    .style(theme::SUBTLE_TEXT)
            )
            .push(
                Row::new()
                    .push(Container::new(text(status_str).size(12).style(theme::TEXT)).padding([2, 8]).style(theme::badge_style(status_color)))
                    .push(Space::with_width(Length::Fixed(8.0)))
                    .push(battery_row)
                    .spacing(8)
            )
            .spacing(3)
            .width(Length::Fill);
        let select_button = Button::new(text("Select").style(theme::TEXT))
            .on_press(Message::SelectDevice(address.clone()))
            .padding([6, 16])
            .style(theme::button_style());
        let row_content = Row::new()
            .push(icon_element)
            .push(Space::with_width(Length::Fixed(16.0)))
            .push(info_col)
            .push(Space::with_width(Length::Fixed(16.0)))
            .push(select_button)
            .spacing(16)
            .align_items(Alignment::Center);
        Container::new(row_content)
            .padding(12)
            .width(Length::Fill)
            .style(theme::device_row_style())
            .into()
    }
}
