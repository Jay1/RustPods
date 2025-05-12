//! UI application using the improved state management architecture

use iced::widget::{text, Column};
use iced::{Settings, Subscription, Application, Element, Command};
use iced::window;
use tokio::sync::mpsc;
use std::{thread, sync::Arc, time::Duration};
use std::sync::Mutex;

use crate::ui::{Message, UiComponent, MainWindow, SettingsWindow};
use crate::ui::components::RealTimeBatteryDisplay;
use crate::ui::system_tray_controller::SystemTrayController;
use crate::ui::state_manager::{StateManager, Action, DeviceState, UiState};
use crate::ui::window_visibility::{WindowVisibilityManager, WindowPosition};
use crate::ui::theme::Theme;
use crate::config::AppConfig;
// Import the AppController from the appropriate path
use crate::app_controller::AppController;

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
        let device_state = state_manager.get_device_state();
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
            window::change_mode(window::Mode::Hidden)
        } else {
            Command::none()
        };
        
        // Create a system tray controller
        let system_tray_controller = None; // Will be initialized in update
        
        (
            Self {
                state_manager,
                main_window,
                settings_window,
                visibility_manager,
                bounds: iced::Rectangle::default(),
                system_tray_controller,
            },
            start_command
        )
    }
    
    fn title(&self) -> String {
        let config = self.state_manager.get_config();
        
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
        let config = self.state_manager.get_config();
        match config.ui.theme {
            crate::config::Theme::Light => Theme::Light,
            crate::config::Theme::Dark => Theme::Dark,
            crate::config::Theme::System => Theme::System,
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        // Reference to current bounds for visibility operations
        let bounds = self.bounds;
        
        match message {
            Message::ToggleVisibility => {
                // Use the visibility manager to toggle
                self.visibility_manager.toggle(bounds)
            },
            Message::Exit => {
                // Cleanup and exit
                if let Some(controller) = &mut self.system_tray_controller {
                    controller.stop();
                }
                
                // Save settings before exit
                let config = self.state_manager.get_config();
                if let Err(e) = config.save() {
                    log::error!("Failed to save settings on exit: {}", e);
                }
                
                window::close()
            },
            Message::WindowMove(position) => {
                // Create a WindowPosition and update the visibility manager
                let window_pos = WindowPosition {
                    x: position.x,
                    y: position.y,
                    width: self.bounds.width,
                    height: self.bounds.height,
                };
                
                self.visibility_manager.set_position(window_pos)
            },
            // Initialize system tray controller if it doesn't exist
            Message::InitializeSystemTray(tx) => {
                // Create the system tray controller with the state manager
                let config = self.state_manager.get_config();
                
                // Create controller and start it
                match SystemTrayController::new(
                    tx,
                    config.clone(),
                    Arc::clone(&self.state_manager)
                ) {
                    Ok(mut controller) => {
                        // Start the controller
                        if let Err(e) = controller.start() {
                            log::error!("Failed to start system tray controller: {}", e);
                        } else {
                            // Store the controller if successful
                            self.system_tray_controller = Some(controller);
                            log::info!("System tray controller started successfully");
                        }
                    },
                    Err(e) => {
                        log::error!("Failed to create system tray controller: {}", e);
                    }
                }
                
                Command::none()
            },
            // Toggle display mode in the main window
            Message::ToggleDisplayMode => {
                // Toggle advanced display mode in main window
                self.main_window.toggle_advanced_display()
            },
            // Update the window size for responsive design
            Message::WindowUpdate => {
                // Update main window with current bounds
                self.main_window = self.main_window.clone().with_window_size(
                    (self.bounds.width as u32, self.bounds.height as u32)
                );
                Command::none()
            },
            // Animation tick for smooth transitions
            Message::AnimationTick => {
                // Update animation progress
                let progress = (self.state_manager.get_animation_progress() + 0.016) % 1.0;
                self.state_manager.set_animation_progress(progress);
                
                // Update main window with animation progress
                self.main_window = self.main_window.clone()
                    .with_animation_progress(progress)
                    .with_connection_transition(progress);
                    
                Command::none()
            },
            // Other messages can be routed to the state manager
            _ => {
                // Handle other message types if needed
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
        // Combine subscriptions from different sources
        Subscription::batch([
            // Window events subscription for handling visibility
            iced::subscription::events().map(Message::RawEvent),
            
            // Add window resize subscription for responsive design
            iced::window::frames()
                .map(|_| Message::WindowUpdate),
            
            // Add animation subscription for smooth transitions
            iced::time::every(std::time::Duration::from_millis(16))
                .map(|_| Message::AnimationTick),
            
            // Add any other subscriptions from components
            // ...
        ])
    }
}

/// Run the state-based UI application
pub fn run_state_ui() -> Result<(), iced::Error> {
    // Initialize logging if not already initialized
    if let Err(_) = env_logger::try_init() {
        // Logging already initialized
    }
    
    log::info!("Starting RustPods UI with state management");
    
    // Create a channel to communicate between the UI and state manager
    let (sender, _receiver) = mpsc::unbounded_channel();
    let sender_clone = sender.clone(); // Clone it before first use
    
    // Create state manager
    let state_manager = Arc::new(StateManager::new(sender));
    
    // Create lifecycle manager
    let mut lifecycle_manager = crate::lifecycle_manager::LifecycleManager::new(
        Arc::clone(&state_manager),
        sender_clone.clone()
    );
    
    // Start lifecycle manager
    if let Err(e) = lifecycle_manager.start() {
        log::error!("Failed to start lifecycle manager: {}", e);
    }
    
    // Create a separate thread for the AppStateController
    let state_manager_clone = Arc::clone(&state_manager);
    let controller_thread = thread::spawn(move || {
        // Create a tokio runtime for the controller
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        
        // Run the controller with the state manager
        let mut controller = crate::app_state_controller::AppStateController::new(sender_clone);
        
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
        state_manager_for_ctrlc.dispatch(Action::Shutdown);
        
        // Shutdown lifecycle manager
        let mut manager = lifecycle_manager_for_ctrlc.lock().unwrap();
        if let Err(e) = manager.shutdown() {
            log::error!("Error during lifecycle manager shutdown: {}", e);
        }
        
        // Allow some time for cleanup before forcing exit
        thread::sleep(Duration::from_millis(500));
        std::process::exit(0);
    }).expect("Failed to set Ctrl+C handler");
    
    // Run the application using the standard iced settings
    let flags = state_manager;
    let settings = iced::Settings::with_flags(flags);
    let result = crate::ui::state::AppState::run(settings);
    
    // Handle any errors during UI execution
    if let Err(ref e) = result {
        log::error!("Error during UI execution: {}", e);
    }
    
    log::info!("UI has exited, performing cleanup");
    
    // Join controller thread (will only happen if UI has exited)
    if controller_thread.is_finished() {
        let _ = controller_thread.join();
    }
    
    result
} 