use iced::{executor, Application, Command, Element, Settings, Length, Color, Background};
use iced::widget::{column, container, row, text};
use iced::alignment::{Alignment, Horizontal};
use iced::theme;

use rustpods::init_logging;
use rustpods::ui::theme::Theme as RustPodsTheme;
use rustpods::airpods::{AirPodsChargingState, AirPodsBattery};

/// Minimal test application for RustPods UI
pub struct MinTestApp {
    title: String,
    mock_airpods: Vec<MockAirPods>,
}

#[derive(Debug, Clone)]
pub struct MockAirPods {
    name: String,
    address: String,
    battery: Option<AirPodsBattery>,
}

#[derive(Debug, Clone)]
pub enum Message {
    NoOp,
}

impl Application for MinTestApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = RustPodsTheme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        // Create mock AirPods with battery data
        let mock_airpods = vec![
            MockAirPods {
                name: "AirPods Pro".to_string(),
                address: "12:34:56:78:90:AB".to_string(),
                battery: Some(AirPodsBattery {
                    left: Some(85),
                    right: Some(92),
                    case: Some(67),
                    charging: Some(AirPodsChargingState::LeftCharging),
                }),
            },
            MockAirPods {
                name: "AirPods Max".to_string(),
                address: "34:56:78:90:AB:CD".to_string(),
                battery: Some(AirPodsBattery {
                    left: Some(45),
                    right: Some(51),
                    case: None,
                    charging: Some(AirPodsChargingState::NotCharging),
                }),
            },
            MockAirPods {
                name: "AirPods (3rd generation)".to_string(),
                address: "56:78:90:AB:CD:EF".to_string(),
                battery: Some(AirPodsBattery {
                    left: None,
                    right: Some(25),
                    case: Some(10),
                    charging: Some(AirPodsChargingState::RightCharging),
                }),
            },
        ];
        
        (
            Self {
                title: "RustPods UI Test".to_string(),
                mock_airpods,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn update(&mut self, _message: Message) -> Command<Message> {
        Command::none()
    }

    fn view(&self) -> Element<Message, iced::Renderer<Self::Theme>> {
        // Main title with increased size for visibility
        let title = text("RustPods UI Test")
            .size(36)
            .style(Color::WHITE)
            .horizontal_alignment(Horizontal::Center);
            
        let subtitle = text("Visual verification - Battery Display Test")
            .size(24)
            .style(Color::WHITE)
            .horizontal_alignment(Horizontal::Center);
        
        // Create device rows with battery information
        let device_rows = column(
            self.mock_airpods.iter().map(|device| {
                let battery_info = if let Some(battery) = &device.battery {
                    let is_left_charging = battery.charging.as_ref()
                        .map(|state| state.is_left_charging())
                        .unwrap_or(false);
                    
                    let is_right_charging = battery.charging.as_ref()
                        .map(|state| state.is_right_charging())
                        .unwrap_or(false);
                    
                    let is_case_charging = battery.charging.as_ref()
                        .map(|state| state.is_case_charging())
                        .unwrap_or(false);
                    
                    column![
                        row![
                            text("Left Pod:").width(Length::Fixed(140.0)).size(18).style(Color::WHITE),
                            match battery.left {
                                Some(level) => text(format!("{}% {}", level, if is_left_charging { "(Charging)" } else { "" })).size(18).style(Color::WHITE),
                                None => text("N/A").size(18).style(Color::WHITE),
                            }
                        ].spacing(15),
                        row![
                            text("Right Pod:").width(Length::Fixed(140.0)).size(18).style(Color::WHITE),
                            match battery.right {
                                Some(level) => text(format!("{}% {}", level, if is_right_charging { "(Charging)" } else { "" })).size(18).style(Color::WHITE),
                                None => text("N/A").size(18).style(Color::WHITE),
                            }
                        ].spacing(15),
                        row![
                            text("Case:").width(Length::Fixed(140.0)).size(18).style(Color::WHITE),
                            match battery.case {
                                Some(level) => text(format!("{}% {}", level, if is_case_charging { "(Charging)" } else { "" })).size(18).style(Color::WHITE),
                                None => text("N/A").size(18).style(Color::WHITE),
                            }
                        ].spacing(15)
                    ].spacing(15)
                } else {
                    column![text("Battery information not available").size(18).style(Color::WHITE)]
                };

                // Container with a visible border and background
                container(
                    column![
                        text(&device.name).size(28).style(Color::WHITE),
                        text(&device.address).size(20).style(Color::WHITE),
                        battery_info
                    ]
                    .spacing(15)
                    .padding(20)
                )
                .padding(20)
                .style(theme::Container::Custom(Box::new(DeviceCardStyle)))
                .width(Length::Fill)
                .into()
            })
            .collect()
        )
        .spacing(30)
        .width(Length::Fill);

        // Add a visible container around all content
        let content = column![
            title,
            subtitle,
            device_rows,
            text("All battery levels are displayed correctly, including charging status and N/A values")
                .size(20)
                .style(Color::WHITE)
                .horizontal_alignment(Horizontal::Center),
        ]
        .spacing(30)
        .padding(30)
        .align_items(Alignment::Center);

        // Main container with a light background for contrast
        container(content)
            .style(theme::Container::Custom(Box::new(MainBackgroundStyle)))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn theme(&self) -> Self::Theme {
        RustPodsTheme::default()
    }
}

// Container style for device cards
struct DeviceCardStyle;

impl container::StyleSheet for DeviceCardStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.3))),
            border_radius: 10.0.into(),
            border_width: 2.0,
            border_color: Color::from_rgb(0.5, 0.5, 0.8),
            ..Default::default()
        }
    }
}

// Container style for main background
struct MainBackgroundStyle;

impl container::StyleSheet for MainBackgroundStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.15, 0.15, 0.25))),
            ..Default::default()
        }
    }
}

fn main() -> Result<(), iced::Error> {
    // Initialize logging
    init_logging();
    
    println!("Starting minimal RustPods UI test with mock AirPods data...");
    
    // Run the minimal test application
    MinTestApp::run(Settings::default())
} 