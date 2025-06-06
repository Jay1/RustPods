use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::sync::Mutex;

use iced::{Command, Element, Subscription, Application, window};

use crate::ui::SystemTrayController;
use crate::ui::{MainWindow, SettingsWindow};
use crate::ui::message::Message;
use crate::ui::state_manager::StateManager;
use crate::ui::theme::Theme;
use crate::ui::window_visibility::WindowVisibilityManager;
use crate::ui::UiComponent;

/// State-managed application UI
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
    
    /// System tray controller
    #[allow(dead_code)]
    system_tray_controller: Option<SystemTrayController>,
}

impl Application for StateApp {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = Arc<StateManager>;
    
    fn new(flags: Self::Flags) -> (Self, Command<Message>) {
        let state_manager = flags;
        
        // Get current state
        let _device_state = state_manager.get_device_state();
        let ui_state = state_manager.get_ui_state();
        let config = state_manager.get_config();
        
        // Create main and settings windows
        let main_window = MainWindow::empty();
        let settings_window = SettingsWindow::new(config.clone());
        
        // Create window visibility manager with the current config
        let visibility_manager = WindowVisibilityManager::new(config.clone())
            .with_state_manager(Arc::clone(&state_manager));
        
        // Create initial command
        let start_command = if !ui_state.visible {
            // Start hidden if UI state is not visible
            window::minimize(true)
        } else {
            Command::none()
        };
        
        // Create app state
        let app = Self {
            state_manager,
            main_window,
            settings_window,
            visibility_manager,
            bounds: iced::Rectangle::new(
                iced::Point::new(0.0, 0.0),
                iced::Size::new(800.0, 600.0),
            ),
            system_tray_controller: None,
        };
        
        (app, start_command)
    }
    
    fn title(&self) -> String {
        let mut title = String::from("RustPods");
        
        // Add device information if connected
        let device_state = self.state_manager.get_device_state();
        if let Some(address) = &device_state.selected_device {
            if let Some(device) = device_state.devices.get(address) {
                title.push_str(" - ");
                title.push_str(device.name.as_deref().unwrap_or("Unknown Device"));
            }
        }
        
        title
    }
    
    fn theme(&self) -> Self::Theme {
        Theme::default()
    }
    
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Exit => {
                if let Some(controller) = &mut self.system_tray_controller {
                    let _ = controller.stop();
                }
                iced::window::close()
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
        // Use the UI state to determine which view to show
        let ui_state = self.state_manager.get_ui_state();
        
        if ui_state.show_settings {
            // Show settings window
            self.settings_window.view()
        } else {
            // Show main window
            self.main_window.view()
        }
    }
    
    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            iced::time::every(std::time::Duration::from_millis(16)).map(|_| Message::AnimationTick),
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
    let (controller_sender, controller_receiver) = tokio::sync::mpsc::unbounded_channel();
    let sender_for_state_manager = controller_sender.clone();
    let sender_for_controller_thread = controller_sender.clone();
    let sender_for_lifecycle_manager = controller_sender.clone();
    let _sender_for_ui = controller_sender.clone();
    
    // Create state manager
    let state_manager = Arc::new(StateManager::new(sender_for_state_manager));
    
    // Create lifecycle manager with appropriate auto-save interval based on config
    let config = state_manager.get_config();
    let auto_save_interval = match config.system.auto_save_interval {
        Some(seconds) if seconds >= 60 => Duration::from_secs(seconds),
        _ => Duration::from_secs(300), // Default 5 minutes
    };
    
    let mut lifecycle_manager = crate::lifecycle_manager::LifecycleManager::new(
        Arc::clone(&state_manager),
        sender_for_lifecycle_manager
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
    let controller_thread = thread::spawn(move || {
        // Create a tokio runtime for the controller
        let runtime = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                log::error!("Failed to create tokio runtime for controller: {}", e);
                return;
            }
        };
        
        // Run the controller with the state manager
        let mut controller = crate::app_state_controller::AppStateController::new(sender_for_controller_thread);
        
        log::info!("Starting app state controller");
        // Use the runtime to run the async start method
        if let Err(e) = runtime.block_on(controller.start()) {
            log::error!("Failed to start app state controller: {}", e);
        }
        
        // Keep the thread alive while the application is running
        loop {
            // Sleep to avoid high CPU usage
            thread::sleep(Duration::from_millis(100));
            
            // Check if we should exit the thread
            // In a real implementation, we would use a proper shutdown signal
            if thread::panicking() {
                break;
            }
        }
    });
    
    // Register Ctrl+C handler for graceful shutdown
    let state_manager_for_ctrlc = Arc::clone(&state_manager);
    let lifecycle_manager_for_ctrlc = Arc::new(Mutex::new(lifecycle_manager));
    
    ctrlc::set_handler(move || {
        log::info!("Received Ctrl+C, initiating graceful shutdown");
        
        // Trigger shutdown actions
        state_manager_for_ctrlc.dispatch(crate::ui::state_manager::Action::Shutdown);
        
        // Shutdown lifecycle manager
        if let Ok(mut manager) = lifecycle_manager_for_ctrlc.lock() {
            if let Err(e) = manager.shutdown() {
                log::error!("Error during lifecycle manager shutdown: {}", e);
            } else {
                log::info!("Lifecycle manager shutdown completed");
            }
        } else {
            log::error!("Failed to acquire lock on lifecycle manager for shutdown");
        }
        
        // Allow some time for cleanup before forcing exit
        thread::sleep(Duration::from_millis(500));
        std::process::exit(0);
    }).expect("Failed to set Ctrl+C handler");
    
    // Run the application using the standard iced settings
    let flags = (controller_sender, controller_receiver);
    let settings = iced::Settings {
        id: None,
        window: iced::window::Settings {
            visible: true,
            ..Default::default()
        },
        flags: flags,
        default_font: iced::Font::default(),
        default_text_size: 16.0,
        antialiasing: false,
        exit_on_close_request: true,
    };
    let result = crate::ui::state::AppState::run(settings);
    
    // Handle any errors during UI execution
    if let Err(ref e) = result {
        log::error!("Error during UI execution: {}", e);
    }
    
    log::info!("UI has exited, performing cleanup");
    
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