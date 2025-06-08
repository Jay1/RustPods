use iced::alignment::{Alignment, Horizontal};
use iced::theme;
use iced::widget::{column, container, row, text};
use iced::{executor, Application, Background, Color, Command, Element, Length, Settings};

use rustpods::airpods::{AirPodsBattery, AirPodsChargingState};
use rustpods::init_logging;
use rustpods::ui::theme::Theme as RustPodsTheme;

/// Edge case test application for RustPods UI
pub struct EdgeCaseTestApp {
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

impl Application for EdgeCaseTestApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = RustPodsTheme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        // Create mock AirPods with various edge cases for battery data
        let mock_airpods = vec![
            // Edge Case 1: Completely missing battery data
            MockAirPods {
                name: "AirPods (No Battery Data)".to_string(),
                address: "00:11:22:33:44:55".to_string(),
                battery: None,
            },
            // Edge Case 2: Only case battery available
            MockAirPods {
                name: "AirPods (Case Only)".to_string(),
                address: "AA:BB:CC:DD:EE:FF".to_string(),
                battery: Some(AirPodsBattery {
                    left: None,
                    right: None,
                    case: Some(95),
                    charging: Some(AirPodsChargingState::CaseCharging),
                }),
            },
            // Edge Case 3: Extremely low battery levels
            MockAirPods {
                name: "AirPods (Low Battery)".to_string(),
                address: "11:22:33:44:55:66".to_string(),
                battery: Some(AirPodsBattery {
                    left: Some(5),
                    right: Some(2),
                    case: Some(1),
                    charging: Some(AirPodsChargingState::NotCharging),
                }),
            },
            // Edge Case 4: All components charging simultaneously
            MockAirPods {
                name: "AirPods (Multiple Charging)".to_string(),
                address: "BB:CC:DD:EE:FF:00".to_string(),
                battery: Some(AirPodsBattery {
                    left: Some(45),
                    right: Some(52),
                    case: Some(78),
                    charging: Some(AirPodsChargingState::BothBudsCharging),
                }),
            },
            // Edge Case 5: Missing charging state
            MockAirPods {
                name: "AirPods (Unknown Charging)".to_string(),
                address: "CC:DD:EE:FF:00:11".to_string(),
                battery: Some(AirPodsBattery {
                    left: Some(67),
                    right: Some(69),
                    case: Some(72),
                    charging: None,
                }),
            },
        ];

        (
            Self {
                title: "RustPods Edge Case Test".to_string(),
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
        let title = text("RustPods Edge Case Test")
            .size(36)
            .style(Color::WHITE)
            .horizontal_alignment(Horizontal::Center);

        let subtitle = text("Visual verification - Battery Display Edge Cases")
            .size(24)
            .style(Color::WHITE)
            .horizontal_alignment(Horizontal::Center);

        // Create device rows with battery information
        let device_rows = column(
            self.mock_airpods
                .iter()
                .map(|device| {
                    let battery_info = if let Some(battery) = &device.battery {
                        let is_left_charging = battery
                            .charging
                            .as_ref()
                            .map(|state| state.is_left_charging())
                            .unwrap_or(false);

                        let is_right_charging = battery
                            .charging
                            .as_ref()
                            .map(|state| state.is_right_charging())
                            .unwrap_or(false);

                        let is_case_charging = battery
                            .charging
                            .as_ref()
                            .map(|state| state.is_case_charging())
                            .unwrap_or(false);

                        column![
                            row![
                                text("Left Pod:")
                                    .width(Length::Fixed(140.0))
                                    .size(18)
                                    .style(Color::WHITE),
                                match battery.left {
                                    Some(level) => text(format!(
                                        "{}% {}",
                                        level,
                                        if is_left_charging { "(Charging)" } else { "" }
                                    ))
                                    .size(18)
                                    .style(Color::WHITE),
                                    None => text("N/A").size(18).style(Color::WHITE),
                                }
                            ]
                            .spacing(15),
                            row![
                                text("Right Pod:")
                                    .width(Length::Fixed(140.0))
                                    .size(18)
                                    .style(Color::WHITE),
                                match battery.right {
                                    Some(level) => text(format!(
                                        "{}% {}",
                                        level,
                                        if is_right_charging { "(Charging)" } else { "" }
                                    ))
                                    .size(18)
                                    .style(Color::WHITE),
                                    None => text("N/A").size(18).style(Color::WHITE),
                                }
                            ]
                            .spacing(15),
                            row![
                                text("Case:")
                                    .width(Length::Fixed(140.0))
                                    .size(18)
                                    .style(Color::WHITE),
                                match battery.case {
                                    Some(level) => text(format!(
                                        "{}% {}",
                                        level,
                                        if is_case_charging { "(Charging)" } else { "" }
                                    ))
                                    .size(18)
                                    .style(Color::WHITE),
                                    None => text("N/A").size(18).style(Color::WHITE),
                                }
                            ]
                            .spacing(15)
                        ]
                        .spacing(15)
                    } else {
                        column![text("Battery information not available")
                            .size(18)
                            .style(Color::WHITE)]
                    };

                    // Container with a visible border and background
                    container(
                        column![
                            text(&device.name).size(28).style(Color::WHITE),
                            text(&device.address).size(20).style(Color::WHITE),
                            battery_info
                        ]
                        .spacing(15)
                        .padding(20),
                    )
                    .padding(20)
                    .style(theme::Container::Custom(Box::new(DeviceCardStyle)))
                    .width(Length::Fill)
                    .into()
                })
                .collect(),
        )
        .spacing(30)
        .width(Length::Fill);

        // Add a visible container around all content
        let content = column![
            title,
            subtitle,
            device_rows,
            text(
                "Verifying edge cases: missing data, low battery, all charging, and unknown states"
            )
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

    println!("\nStarting RustPods edge case test with mock AirPods data...");

    let mut settings = Settings::with_flags(());
    settings.window.size = (800, 800);

    EdgeCaseTestApp::run(settings)
}
