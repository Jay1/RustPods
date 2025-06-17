//! AirPods Popup Component - Compact, visual AirPods interface
//! 
//! Provides a macOS-style popup interface for displaying AirPods information

use iced::{
    widget::{button, column, container, row, text, Space, image},
    Element, Length, Alignment, Color
};
use iced::alignment::Horizontal;

use crate::ui::Message;
use crate::ui::theme::{self, Theme};
use crate::ui::UiComponent;
use crate::ui::state::MergedBluetoothDevice;
use crate::ui::components::view_circular_battery_widget;

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

    // Battery indicators row with circular widgets
    let battery_indicators_row = row![
        // Left earbud circular widget
        column![
            view_circular_battery_widget(
                device.left_battery_fractional
                    .unwrap_or(device.left_battery.unwrap_or(0) as f32),
                false // For now, charging state is not available in MergedBluetoothDevice
            ),
            text("Left")
                .size(14)
                .style(theme::TEXT)
                .horizontal_alignment(Horizontal::Center)
        ]
        .align_items(Alignment::Center)
        .spacing(5),
        
        // Right earbud circular widget
        column![
            view_circular_battery_widget(
                device.right_battery_fractional
                    .unwrap_or(device.right_battery.unwrap_or(0) as f32),
                false // For now, charging state is not available in MergedBluetoothDevice
            ),
            text("Right")
                .size(14)
                .style(theme::TEXT)
                .horizontal_alignment(Horizontal::Center)
        ]
        .align_items(Alignment::Center)
        .spacing(5),
        
        // Case circular widget
        column![
            view_circular_battery_widget(
                device.case_battery_fractional
                    .unwrap_or(device.case_battery.unwrap_or(0) as f32),
                false // For now, charging state is not available in MergedBluetoothDevice
            ),
            text("Case")
                .size(14)
                .style(theme::TEXT)
                .horizontal_alignment(Horizontal::Center)
        ]
        .align_items(Alignment::Center)
        .spacing(5)
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

        // Battery displays in a row with circular widgets
        let battery_row = row![
            // Left earbud circular widget
            column![
                view_circular_battery_widget(
                    self.device.left_battery_fractional
                        .unwrap_or(self.device.left_battery.unwrap_or(0) as f32),
                    false // For now, charging state is not available in MergedBluetoothDevice
                ),
                text("Left")
                    .size(14)
                    .style(theme::TEXT)
                    .horizontal_alignment(Horizontal::Center)
            ]
            .align_items(Alignment::Center)
            .spacing(5),
            
            // Right earbud circular widget
            column![
                view_circular_battery_widget(
                    self.device.right_battery_fractional
                        .unwrap_or(self.device.right_battery.unwrap_or(0) as f32),
                    false // For now, charging state is not available in MergedBluetoothDevice
                ),
                text("Right")
                    .size(14)
                    .style(theme::TEXT)
                    .horizontal_alignment(Horizontal::Center)
            ]
            .align_items(Alignment::Center)
            .spacing(5),
            
            // Case circular widget
            column![
                view_circular_battery_widget(
                    self.device.case_battery_fractional
                        .unwrap_or(self.device.case_battery.unwrap_or(0) as f32),
                    false // For now, charging state is not available in MergedBluetoothDevice
                ),
                text("Case")
                    .size(14)
                    .style(theme::TEXT)
                    .horizontal_alignment(Horizontal::Center)
            ]
            .align_items(Alignment::Center)
            .spacing(5)
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