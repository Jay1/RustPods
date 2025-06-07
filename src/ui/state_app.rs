//! UI application using the improved state management architecture

use iced::{Subscription, Application, Element, Command};
use iced::window;
use tokio::sync::mpsc;
use std::{sync::Arc, time::Duration};

use crate::ui::{Message, UiComponent, MainWindow, SettingsWindow};
// Temporarily disable system tray
// use crate::ui::system_tray_controller::SystemTrayController;
use crate::ui::state_manager::StateManager;
use crate::ui::window_visibility::WindowVisibilityManager;
use crate::ui::theme::Theme;
use crate::config::AppConfig;
use crate::config::ConfigManager;
use crate::bluetooth::DiscoveredDevice;
// Import the AppController from the appropriate path

/// State-based application implementation with improved state management
pub struct StateApp {
    /// State manager
    state_manager: Arc<StateManager>,
    
    /// Main window component
    main_window: MainWindow,
    
    /// Settings window component
    settings_window: SettingsWindow,
    
    /// Window visibility manager
    visibility_manager: WindowVisibilityManager,
    
    /// Current application bounds
    bounds: iced::Rectangle,
    
    // System tray controller (temporarily disabled)
    // system_tray_controller: Option<SystemTrayController>,
}

impl Application for StateApp {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = Arc<StateManager>;

    fn new(flags: Self::Flags) -> (Self, Command<Message>) {
        let state_manager = flags;
        
        // Refresh config
        let _config = state_manager.get_config();
        
        // Get device state
        let _device_state = state_manager.get_device_state();
        let ui_state = state_manager.get_ui_state();
        
        // Create main and settings windows
        let main_window = MainWindow::empty();
        let settings_window = SettingsWindow::new(_config.clone());
        
        // Create window visibility manager with the current config
        let visibility_manager = WindowVisibilityManager::new(_config.clone())
            .with_state_manager(Arc::clone(&state_manager));
        
        // Create initial command
        let start_command = if !ui_state.visible {
            // Start hidden if UI state is not visible
            window::change_mode(window::Mode::Hidden)
        } else {
            Command::none()
        };
        
        // Temporarily disable system tray controller
        // let (tray_tx, _tray_rx) = std::sync::mpsc::channel();
        // let mut system_tray_controller = Some(SystemTrayController::new(tray_tx, _config.clone(), Arc::clone(&state_manager)).unwrap());
        // if let Some(controller) = &mut system_tray_controller {
        //     match controller.start() {
        //         Ok(_) => log::info!("System tray controller started successfully"),
        //         Err(e) => log::error!("Failed to start system tray controller: {}", e),
        //     }
        // }
        
        (
            Self {
                state_manager,
                main_window,
                settings_window,
                visibility_manager,
                bounds: iced::Rectangle::default(),
                // system_tray_controller, // Temporarily disabled
            },
            start_command
        )
    }
    
    fn title(&self) -> String {
        let _config = self.state_manager.get_config();
        
        // Show battery percentage in title if available
        let device_state = self.state_manager.get_device_state();
        if let Some(battery) = device_state.battery_status.as_ref() {
            let left = battery.battery.left.map(|v| v.to_string()).unwrap_or_else(|| "?".to_string());
            let right = battery.battery.right.map(|v| v.to_string()).unwrap_or_else(|| "?".to_string());
            let case = battery.battery.case.map(|v| v.to_string()).unwrap_or_else(|| "?".to_string());
            
            format!("RustPods - L: {}% R: {}% Case: {}%", left, right, case)
        } else {
            String::from("RustPods - AirPods Battery Monitor")
        }
    }

    fn theme(&self) -> Self::Theme {
        // Use the theme from config
        let _config = self.state_manager.get_config();
        match _config.ui.theme {
            crate::config::Theme::Light => Theme::Light,
            crate::config::Theme::Dark => Theme::Dark,
            crate::config::Theme::System => Theme::System,
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        let bounds = self.bounds;
        match message {
            Message::ToggleVisibility => {
                log::info!("StateApp: ToggleVisibility received");
                self.visibility_manager.toggle(bounds)
            },
            Message::Exit => {
                // Temporarily disable system tray controller cleanup
                // if let Some(controller) = &mut self.system_tray_controller {
                //     let _ = controller.stop();
                // }
                let config = self.state_manager.get_config();
                if let Err(e) = config.save() {
                    log::error!("Failed to save settings on exit: {}", e);
                }
                window::close()
            },
            Message::WindowCloseRequested => {
                log::info!("StateApp: WindowCloseRequested received, minimizing to tray");
                // Minimize to tray instead of exiting
                self.visibility_manager.hide(bounds)
            },
            Message::AnimationTick => {
                let progress = (self.state_manager.get_animation_progress() + 0.016) % 1.0;
                self.state_manager.set_animation_progress(progress);
                self.main_window = self.main_window.clone()
                    .with_animation_progress(progress)
                    .with_connection_transition(progress);
                Command::none()
            },
            _ => {
                Command::none()
            }
        }
    }
    
    fn view(&self) -> Element<'_, Message, iced::Renderer<Theme>> {
        if !self.visibility_manager.is_visible() {
            iced::widget::container::Container::new(
                iced::widget::text("")
            ).into()
        } else if self.state_manager.get_ui_state().show_settings {
            self.settings_window.view()
        } else {
            self.main_window.view()
        }
    }
    
    fn subscription(&self) -> Subscription<Message> {
        iced::Subscription::batch(vec![
            iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick),
            iced::subscription::events_with(|event, _status| {
                if let iced::Event::Window(window_event) = &event {
                    if let window::Event::CloseRequested = window_event {
                        return Some(Message::WindowCloseRequested);
                    }
                }
                None
            }),
        ])
    }
}

/// Run the state-based UI application
pub fn run_state_ui() -> Result<(), iced::Error> {
    // Initialize logging if not already initialized
    if env_logger::try_init().is_err() {
        // Logging already initialized
    }
    log::info!("Starting RustPods UI with state management");

    // Create a channel to communicate between the UI and state manager
    let (sender, _receiver) = mpsc::unbounded_channel();
    let sender_clone = sender.clone(); // Clone it before first use

    // Create state manager
    let state_manager = Arc::new(StateManager::new(sender));

    // Create lifecycle manager with appropriate auto-save interval based on config
    let config = state_manager.get_config();
    let auto_save_interval = match config.system.auto_save_interval {
        Some(seconds) if seconds >= 60 => Duration::from_secs(seconds),
        _ => Duration::from_secs(300), // Default 5 minutes
    };

    let mut lifecycle_manager = crate::lifecycle_manager::LifecycleManager::new(
        Arc::clone(&state_manager),
        sender_clone.clone()
    ).with_auto_save_interval(auto_save_interval);

    // Start lifecycle manager with proper error handling
    match lifecycle_manager.start() {
        Ok(_) => {
            log::info!("Lifecycle manager started successfully");
        },
        Err(e) => {
            log::error!("Failed to start lifecycle manager: {}", e);
            // Continue without full lifecycle management, but still try basic features
        }
    }

    // Create a separate thread for the AppStateController
    let _state_manager_clone = Arc::clone(&state_manager);
    let controller_thread = std::thread::spawn(move || {
        // Create a tokio runtime for the controller
        let runtime = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                log::error!("Failed to create tokio runtime for controller: {}", e);
                return;
            }
        };
        // Run the controller with the state manager
        let mut controller = crate::app_state_controller::AppStateController::new(sender_clone);
        log::info!("Starting app state controller");
        // Use the runtime to run the controller
        runtime.block_on(async {
            if let Err(e) = controller.initialize().await {
                log::error!("Failed to initialize app state controller: {}", e);
                return;
            }
            if let Err(e) = controller.start().await {
                log::error!("Failed to start app state controller: {}", e);
            }
            // Keep the controller running until shutdown
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
                // TODO: Add proper shutdown detection
            }
        });
    });

    // --- FIX: Wrap the UI in a Tokio runtime ---
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime for UI thread");

    let result = rt.block_on(async {
        StateApp::run(iced::Settings {
            id: Some("rustpods".to_string()),
            window: iced::window::Settings {
                size: (800, 600),
                position: iced::window::Position::Default,
                min_size: Some((400, 300)),
                max_size: None,
                visible: true,
                resizable: true,
                decorations: true,
                transparent: false,
                icon: None,
                level: iced::window::Level::Normal,
                platform_specific: Default::default(),
            },
            flags: state_manager,
            default_font: iced::Font::with_name("SpaceMono Nerd Font"),
            default_text_size: 16.0,
            antialiasing: true,
            exit_on_close_request: false,
        })
    });

    // Join controller thread (will only happen if UI has exited)
    if controller_thread.is_finished() {
        if let Err(_e) = controller_thread.join() {
            log::error!("Failed to join controller thread: thread panicked");
        }
    } else {
        log::warn!("Controller thread is still running after UI exit");
    }

    result
} 