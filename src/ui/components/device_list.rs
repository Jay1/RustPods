use iced::widget::{button, column, row, scrollable, text, container};
use iced::{Element, Length};
use iced::theme;

use crate::bluetooth::DiscoveredDevice;
use crate::ui::{Message, UiComponent};

/// Component for displaying and selecting devices
pub struct DeviceList {
    /// The devices to display
    devices: Vec<DiscoveredDevice>,
    /// The currently selected device address
    selected: Option<String>,
}

impl DeviceList {
    /// Create a new device list
    pub fn new(devices: Vec<DiscoveredDevice>, selected: Option<String>) -> Self {
        Self { devices, selected }
    }
}

impl UiComponent for DeviceList {
    fn view(&self) -> Element<'_, Message> {
        // Create a column of devices
        let mut device_list = column![]
            .spacing(10)
            .padding(20)
            .width(Length::Fill);

        // Add header
        device_list = device_list.push(
            row![
                text("Device").width(Length::FillPortion(3)),
                text("Type").width(Length::FillPortion(2)),
                text("RSSI").width(Length::FillPortion(1)),
            ]
            .spacing(10)
            .padding(5),
        );

        // Sort devices by RSSI (strongest signal first)
        let mut sorted_devices = self.devices.clone();
        sorted_devices.sort_by(|a, b| {
            let a_rssi = a.rssi.unwrap_or(-100);
            let b_rssi = b.rssi.unwrap_or(-100);
            b_rssi.cmp(&a_rssi)
        });

        // Add each device to the list
        for device in &sorted_devices {
            let address = device.address.to_string();
            let is_selected = self
                .selected
                .as_ref()
                .map_or(false, |selected| selected == &address);

            let device_name = device
                .name
                .clone()
                .unwrap_or_else(|| address.clone());

            let device_type = if device.is_potential_airpods {
                "AirPods"
            } else {
                "Bluetooth"
            };

            let rssi = device.rssi.map_or("-".to_string(), |r| format!("{} dBm", r));

            let btn = button(
                row![
                    text(device_name).width(Length::FillPortion(3)),
                    text(device_type).width(Length::FillPortion(2)),
                    text(rssi).width(Length::FillPortion(1)),
                ]
                .spacing(10),
            )
            .width(Length::Fill)
            .on_press(Message::SelectDevice(address.clone()));

            // Use a different style based on selection
            let styled_container = if is_selected {
                container(btn).padding(5).style(theme::Container::Box)
            } else {
                container(btn).padding(5)
            };

            device_list = device_list.push(styled_container);
        }

        // If no devices are found, show a message
        if sorted_devices.is_empty() {
            device_list = device_list.push(
                text("No devices found. Start scanning to discover nearby devices.")
                    .width(Length::Fill),
            );
        }

        // Wrap in a scrollable container
        scrollable(container(device_list).width(Length::Fill)).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use btleplug::api::BDAddr;
    use std::collections::HashMap;
    use std::time::Instant;

    #[test]
    fn test_device_list_creation() {
        // Create some test devices
        let device1 = DiscoveredDevice {
            address: BDAddr::from([1, 2, 3, 4, 5, 6]),
            name: Some("Device 1".to_string()),
            rssi: Some(-60),
            manufacturer_data: HashMap::new(),
            is_potential_airpods: false,
            last_seen: Instant::now(),
        };

        let device2 = DiscoveredDevice {
            address: BDAddr::from([6, 5, 4, 3, 2, 1]),
            name: Some("AirPods".to_string()),
            rssi: Some(-50),
            manufacturer_data: HashMap::new(),
            is_potential_airpods: true,
            last_seen: Instant::now(),
        };

        let devices = vec![device1, device2];
        let selected = Some(BDAddr::from([6, 5, 4, 3, 2, 1]).to_string());

        // Create the device list component
        let device_list = DeviceList::new(devices, selected.clone());

        // Verify device count
        assert_eq!(device_list.devices.len(), 2);
        
        // Verify selected device
        assert_eq!(device_list.selected, selected);
    }
} 