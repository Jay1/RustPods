//! AirPods Popup Component - Compact, visual AirPods interface
//! 
//! Provides a macOS-style popup interface for displaying AirPods information

use iced::{
    widget::{button, column, container, row, text, Space, image, Svg},
    Element, Length, Alignment, Color
};
use iced::alignment::Horizontal;

use crate::ui::Message;
use crate::ui::theme::{self, Theme};
use crate::ui::UiComponent;
use crate::ui::state::MergedBluetoothDevice;
use crate::ui::components::battery_indicator;

/// Determine the correct image paths based on the AirPods model
fn get_airpods_image_paths(device_name: &str) -> (String, String) {
    if device_name.to_lowercase().contains("pro") {
        // AirPods Pro (any generation)
        ("assets/icons/hw/airpodspro.png".to_string(), 
         "assets/icons/hw/airpodsprocase.png".to_string())
    } else if device_name.to_lowercase().contains("max") {
        // AirPods Max
        ("assets/icons/hw/AirpodsMax.png".to_string(), 
         "assets/icons/hw/AirpodsMax.png".to_string()) // Max doesn't have a separate case
    } else {
        // Regular AirPods (1st, 2nd, 3rd gen)
        ("assets/icons/hw/airpods.png".to_string(), 
         "assets/icons/hw/airpodscase.png".to_string())
    }
}

/// Create a graphical popup for displaying AirPods device information
/// 
/// This function creates a styled container with Catppuccin theme colors
/// that will serve as the main container for the new graphical AirPods interface.
pub fn view_device_popup(device: &MergedBluetoothDevice) -> Element<'static, Message, iced::Renderer<Theme>> {
    // Header row with device name and close button
    let header_row = row![
        // Device name title
        text(device.name.clone())
            .size(24.0)
            .style(theme::TEXT),
        
        // Spacer to push elements to opposite ends
        Space::with_width(Length::Fill),
        
        // Close button with temporary text (for debugging)
        button(text("X").size(16).style(Color::from_rgb(1.0, 0.0, 0.0)))
        .on_press(Message::ClosePopup)
        .style(iced::theme::Button::Text)
        .padding(8)
    ]
    .align_items(Alignment::Center);

    // Get the correct image paths based on device model
    let (earbuds_image_path, case_image_path) = get_airpods_image_paths(&device.name);
    
    // Device images row
    let device_images_row = row![
        // Earbuds image - dynamically determined
        image(earbuds_image_path)
            .height(Length::Fixed(128.0)),
        
        // Case image - dynamically determined
        image(case_image_path)
            .height(Length::Fixed(128.0))
    ]
    .align_items(Alignment::Center)
    .spacing(40);

    // Battery indicators row
    let battery_indicators_row = row![
        // Earbuds group: Left and Right together
        row![
            battery_indicator::view(
                "Left", 
                device.left_battery, 
                false // For now, charging state is not available in MergedBluetoothDevice
            ),
            battery_indicator::view(
                "Right", 
                device.right_battery, 
                false // For now, charging state is not available in MergedBluetoothDevice
            )
        ]
        .spacing(20)
        .align_items(Alignment::Center),
        
        // Case indicator
        battery_indicator::view(
            "Case", 
            device.case_battery, 
            false // For now, charging state is not available in MergedBluetoothDevice
        )
    ]
    .spacing(40)
    .align_items(Alignment::Center);

    // Main content column
    let main_content = column![
        header_row,
        device_images_row,
        battery_indicators_row,
        // Additional content will be added here in future steps
    ]
    .spacing(15);
    
    // Return the main content directly
    Element::from(main_content)
}

/// AirPods popup component for compact device display
#[derive(Debug, Clone)]
pub struct AirPodsPopup {
    /// The AirPods device to display
    pub device: MergedBluetoothDevice,
}

impl AirPodsPopup {
    /// Create a new AirPods popup
    pub fn new(device: MergedBluetoothDevice) -> Self {
        Self { device }
    }

    /// Create battery display with visual bar
    fn create_battery_display(&self, battery_level: Option<u8>, label: &str) -> Element<'_, Message, iced::Renderer<Theme>> {
        let battery_text = match battery_level {
            Some(level) => format!("{}%", level),
            None => "0%".to_string(),
        };

        // Create visual battery bar using repeated characters
        let battery_bar = match battery_level {
            Some(level) if level > 0 => {
                let filled_bars = (level as f32 / 100.0 * 10.0).ceil() as usize;
                let empty_bars = 10 - filled_bars;
                format!("{}{}", "█".repeat(filled_bars), "░".repeat(empty_bars))
            }
            _ => "░░░░░░░░░░".to_string(),
        };

        column![
            text(label)
                .size(14)
                .horizontal_alignment(Horizontal::Center)
                .style(theme::TEXT),
            iced::widget::Space::with_height(Length::Fixed(8.0)),
            text(battery_bar)
                .size(12)
                .horizontal_alignment(Horizontal::Center)
                .style(theme::BLUE),
            iced::widget::Space::with_height(Length::Fixed(4.0)),
            text(battery_text)
                .size(16)
                .horizontal_alignment(Horizontal::Center)
                .style(theme::TEXT),
        ]
        .spacing(2)
        .align_items(Alignment::Center)
        .width(Length::FillPortion(1))
        .into()
    }

    /// Create case battery display
    fn create_case_display(&self) -> Element<'_, Message, iced::Renderer<Theme>> {
        let battery_text = match self.device.case_battery {
            Some(level) => format!("{}%", level),
            None => "0%".to_string(),
        };

        // Create visual battery bar for case
        let battery_bar = match self.device.case_battery {
            Some(level) if level > 0 => {
                let filled_bars = (level as f32 / 100.0 * 10.0).ceil() as usize;
                let empty_bars = 10 - filled_bars;
                format!("{}{}", "█".repeat(filled_bars), "░".repeat(empty_bars))
            }
            _ => "░░░░░░░░░░".to_string(),
        };

        column![
            text("Case")
                .size(14)
                .horizontal_alignment(Horizontal::Center)
                .style(theme::TEXT),
            iced::widget::Space::with_height(Length::Fixed(8.0)),
            text(battery_bar)
                .size(12)
                .horizontal_alignment(Horizontal::Center)
                .style(theme::BLUE),
            iced::widget::Space::with_height(Length::Fixed(4.0)),
            text(battery_text)
                .size(16)
                .horizontal_alignment(Horizontal::Center)
                .style(theme::TEXT),
        ]
        .spacing(2)
        .align_items(Alignment::Center)
        .width(Length::FillPortion(1))
        .into()
    }
}

impl UiComponent for AirPodsPopup {
    fn view(&self) -> Element<'_, Message, iced::Renderer<Theme>> {
        // Header with device name and close button
        let header = row![
            text(&self.device.name)
                .size(20)
                .style(theme::TEXT),
            iced::widget::Space::with_width(Length::Fill),
            button(text("X").size(14).style(Color::from_rgb(1.0, 0.0, 0.0)))
            .padding(6)
            .on_press(Message::ClosePopup)
            .style(iced::theme::Button::Text)
        ]
        .align_items(Alignment::Center)
        .padding([20, 20, 10, 20]);

        // Battery displays in a row
        let battery_row = row![
            self.create_battery_display(self.device.left_battery, "Left"),
            self.create_battery_display(self.device.right_battery, "Right"),
            self.create_case_display(),
        ]
        .spacing(24)
        .align_items(Alignment::Start)
        .padding([10, 20]);

        // Connect/Disconnect button
        let action_button = if self.device.connected {
            button(text("Disconnect").horizontal_alignment(Horizontal::Center))
                .width(Length::Fill)
                .padding(10)
                .on_press(Message::DisconnectDevice)
                .style(iced::theme::Button::Destructive)
        } else {
            button(text("Connect").horizontal_alignment(Horizontal::Center))
                .width(Length::Fill)
                .padding(10)
                .on_press(Message::ConnectDevice)
                .style(iced::theme::Button::Primary)
        };

        // Main popup container
        container(
            column![
                header,
                battery_row,
                iced::widget::Space::with_height(Length::Fixed(10.0)),
                container(action_button).padding([0, 20, 20, 20])
            ]
            .spacing(0)
        )
        .style(iced::theme::Container::Box)
        .width(350)
        .into()
    }
} 