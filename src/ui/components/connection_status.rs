use iced::widget::{container, row, column, text, button, Space, horizontal_rule};use iced::{Length, Color, Alignment};

use crate::ui::{Message, UiComponent};

/// Component for displaying detailed connection status for AirPods devices
#[derive(Debug, Clone)]
pub struct ConnectionStatus {
    /// Whether a device is connected
    is_connected: bool,
    /// Name of the connected device
    device_name: Option<String>,
    /// Signal strength (RSSI) if available (-30 to -100 dBm)
    signal_strength: Option<i16>,
    /// Time connected in seconds (if applicable)
    connected_time: Option<u64>,
}

impl ConnectionStatus {
    /// Create a new connection status component
    pub fn new(
        is_connected: bool,
        device_name: Option<String>,
        signal_strength: Option<i16>,
        connected_time: Option<u64>,
    ) -> Self {
        Self {
            is_connected,
            device_name,
            signal_strength,
            connected_time,
        }
    }
    
    /// Create an empty connection status component (not connected)
    pub fn empty() -> Self {
        Self {
            is_connected: false,
            device_name: None,
            signal_strength: None,
            connected_time: None,
        }
    }

    /// Get a formatted connection time string
    fn format_connected_time(&self) -> String {
        if let Some(seconds) = self.connected_time {
            if seconds < 60 {
                format!("{}s", seconds)
            } else if seconds < 3600 {
                format!("{}m {}s", seconds / 60, seconds % 60)
            } else {
                format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
            }
        } else {
            "".to_string()
        }
    }
    
    /// Get signal quality description based on RSSI
    fn signal_quality_text(&self) -> &'static str {
        match self.signal_strength {
            Some(rssi) if rssi >= -50 => "Excellent",
            Some(rssi) if rssi >= -65 => "Good",
            Some(rssi) if rssi >= -75 => "Fair",
            Some(rssi) if rssi < -75 => "Poor",
            _ => "Unknown",
        }
    }
    
    /// Get signal quality color based on RSSI
    fn signal_quality_color(&self) -> Color {
        match self.signal_strength {
            Some(rssi) if rssi >= -50 => Color::from_rgb(0.2, 0.8, 0.2), // Green
            Some(rssi) if rssi >= -65 => Color::from_rgb(0.6, 0.8, 0.2), // Yellow-Green
            Some(rssi) if rssi >= -75 => Color::from_rgb(0.9, 0.6, 0.1), // Orange
            Some(rssi) if rssi < -75 => Color::from_rgb(0.8, 0.2, 0.2),  // Red
            _ => Color::from_rgb(0.7, 0.7, 0.7),                         // Gray
        }
    }
    
    /// Create signal bars based on RSSI
    fn signal_bars(&self) -> iced::Element<'static, Message, iced::Renderer<crate::ui::theme::Theme>> {
        // Number of bars to show (0-4)
        let bars = match self.signal_strength {
            Some(rssi) if rssi >= -50 => 4,
            Some(rssi) if rssi >= -65 => 3,
            Some(rssi) if rssi >= -75 => 2,
            Some(rssi) if rssi >= -85 => 1,
            _ => 0,
        };
        
        let _color = self.signal_quality_color();
        
        // Bar heights (tallest to shortest)
        let heights = [16, 12, 8, 4];
        let bar_width = 4.0;
        let bar_spacing = 2.0;
        
        let mut bar_row = row![].spacing(bar_spacing);
        
        // Add filled bars
        for i in 0..bars {
            bar_row = bar_row.push(
                container(Space::new(Length::Fixed(bar_width), Length::Fixed(heights[i] as f32)))
            );
        }
        
        // Add empty bars
        for i in bars..4 {
            bar_row = bar_row.push(
                container(Space::new(Length::Fixed(bar_width), Length::Fixed(heights[i] as f32)))
            );
        }
        
        bar_row.into()
    }
}

impl UiComponent for ConnectionStatus {
    fn view(&self) -> iced::Element<'static, Message, iced::Renderer<crate::ui::theme::Theme>> {
        if self.is_connected {
            // Connected state with detailed information
            let device_name = self.device_name.clone().unwrap_or_else(|| "AirPods".to_string());
            
            let mut content = column![
                // Connection header with device name
                row![
                    text("● "),
                    text(format!("Connected to {}", device_name))
                ]
                .align_items(Alignment::Center)
                .spacing(5),
                
                horizontal_rule(1),
                
                // Signal quality row
                row![
                    // Signal bars visualization
                    self.signal_bars(),
                    
                    Space::with_width(Length::Fixed(5.0)),
                    
                    // Signal quality text
                    text(format!("Signal: {}", self.signal_quality_text())),
                    
                    // Show RSSI value if available
                    if let Some(rssi) = self.signal_strength {
                        text(format!(" ({}dBm)", rssi))
                            .size(12)
                    } else {
                        text("")
                    }
                ]
                .align_items(Alignment::Center)
                .spacing(5),
            ];
            
            // Add connection time if available
            if self.connected_time.is_some() {
                content = content.push(
                    text(format!("Connected for: {}", self.format_connected_time()))
                        .size(14)
                );
            }
            
            // Action button
            content = content.push(
                button(text("Disconnect"))
                    .on_press(Message::RetryConnection)
            );
            
            container(content)
                .width(Length::Fill)
                .padding(10)
                .into()
        } else {
            // Not connected state
            let content = column![
                text("Not Connected").size(16),
                text("Connect to a device to see status").size(14),
            ]
            .spacing(10)
            .padding(10)
            .align_items(Alignment::Center);
            
            container(content)
                .width(Length::Fill)
                .padding(10)
                .into()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_connection_status_creation() {
        // Test connected state
        let status = ConnectionStatus::new(
            true,
            Some("My AirPods Pro".to_string()),
            Some(-65),
            Some(120),
        );
        
        assert!(status.is_connected);
        assert_eq!(status.device_name, Some("My AirPods Pro".to_string()));
        assert_eq!(status.signal_strength, Some(-65));
        assert_eq!(status.connected_time, Some(120));
        
        // Test empty creation
        let empty_status = ConnectionStatus::empty();
        assert!(!empty_status.is_connected);
        assert_eq!(empty_status.device_name, None);
        assert_eq!(empty_status.signal_strength, None);
        assert_eq!(empty_status.connected_time, None);
    }
    
    #[test]
    fn test_signal_quality_text() {
        // Test various signal strengths
        let excellent = ConnectionStatus::new(true, None, Some(-40), None);
        let good = ConnectionStatus::new(true, None, Some(-60), None);
        let fair = ConnectionStatus::new(true, None, Some(-70), None);
        let poor = ConnectionStatus::new(true, None, Some(-85), None);
        let unknown = ConnectionStatus::new(true, None, None, None);
        
        assert_eq!(excellent.signal_quality_text(), "Excellent");
        assert_eq!(good.signal_quality_text(), "Good");
        assert_eq!(fair.signal_quality_text(), "Fair");
        assert_eq!(poor.signal_quality_text(), "Poor");
        assert_eq!(unknown.signal_quality_text(), "Unknown");
    }
    
    #[test]
    fn test_format_connected_time() {
        // Test time formatting
        let seconds = ConnectionStatus::new(true, None, None, Some(45));
        let minutes = ConnectionStatus::new(true, None, None, Some(125));
        let hours = ConnectionStatus::new(true, None, None, Some(3725));
        
        assert_eq!(seconds.format_connected_time(), "45s");
        assert_eq!(minutes.format_connected_time(), "2m 5s");
        assert_eq!(hours.format_connected_time(), "1h 2m");
    }
} 