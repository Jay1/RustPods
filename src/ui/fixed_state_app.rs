use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::sync::Mutex;


use iced::{Command, Element, Subscription, Application};

use crate::ui::{MainWindow, SettingsWindow};
use crate::ui::message::Message;
use crate::ui::state_manager::StateManager;
use crate::ui::theme::Theme;
use crate::ui::window_visibility::WindowVisibilityManager;
use crate::ui::UiComponent;
use crate::ui::utils::load_window_icon;
use crate::ui::window_management::{DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT};

/// State-based UI application
pub struct StateApp {
    /// State manager
    state_manager: Arc<StateManager>,
    
    /// Main window component
    main_window: MainWindow,
    
    /// Settings window component
    settings_window: SettingsWindow,
    
    /// Window visibility manager
    #[allow(dead_code)]
    visibility_manager: WindowVisibilityManager,
    
    /// Current application bounds
    #[allow(dead_code)]
    bounds: iced::Rectangle,
    
    // System tray controller (temporarily disabled)
    // #[allow(dead_code)]
    // system_tray_controller: Option<SystemTrayController>,
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
        
        // Create window state manager
        let visibility_manager = WindowVisibilityManager::new(config.clone());
        
        // Create settings window
        let settings_window = SettingsWindow::new(config.clone());
        
        // Create the main window with initial state
        let main_window = MainWindow::new();
        
        // Get UI state to check if window should be visible on startup
        let start_command: iced::Command<Message> = if !ui_state.visible {
            // For now, just use Command::none() since window minimize requires proper window ID
            Command::none()
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
                iced::Size::new(DEFAULT_WINDOW_WIDTH as f32, DEFAULT_WINDOW_HEIGHT as f32),
            ),
            // system_tray_controller: None, // Temporarily disabled
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
                // Temporarily disable system tray controller cleanup
                // if let Some(controller) = &mut self.system_tray_controller {
                //     let _ = controller.stop();
                // }
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
    let (controller_sender, _controller_receiver) = tokio::sync::mpsc::unbounded_channel();
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
    
    // Don't start lifecycle manager yet - defer until Iced runtime is available
    // lifecycle_manager.start() will be called from within the Iced application
    log::info!("Lifecycle manager initialized, will start after Iced runtime is available");
    
    // Start the sync phase of lifecycle manager
    if let Err(e) = lifecycle_manager.start() {
        log::error!("Failed to start lifecycle manager sync phase: {}", e);
        eprintln!("Error: Failed to start lifecycle manager: {}", e);
        std::process::exit(1);
    }
    
    // Store lifecycle manager for async task startup
    let lifecycle_manager_for_async = Arc::new(Mutex::new(lifecycle_manager));
    
    // Start the AppStateController in a background thread with its own async context
    // We can't use tokio::spawn here because we're not in an async context yet
    let _state_manager_clone = Arc::clone(&state_manager);
    let controller_handle = std::thread::spawn(move || {
        // Create a minimal runtime for the controller background thread
        let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime for controller");
        rt.block_on(async move {
            // Run the controller with the state manager
            let mut controller = crate::app_state_controller::AppStateController::new(sender_for_controller_thread);
            
            log::info!("Starting app state controller");
            if let Err(e) = controller.start().await {
                log::error!("Failed to start app state controller: {}", e);
            }
        });
    });
    
    // Register Ctrl+C handler for graceful shutdown
    let state_manager_for_ctrlc = Arc::clone(&state_manager);
    let lifecycle_manager_for_ctrlc = Arc::clone(&lifecycle_manager_for_async);
    
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
    
    // Run the StateApp using the standard iced settings with proper icon
    let flags = Arc::clone(&state_manager);
    
    // Load the window icon with proper error handling
    let icon = load_window_icon();
    
    // Start async task startup in a deferred manner
    let lifecycle_manager_for_startup = Arc::clone(&lifecycle_manager_for_async);
    
    // Spawn a task to start lifecycle async tasks after a delay
    std::thread::spawn(move || {
        // Wait a moment for Iced to fully initialize its runtime
        thread::sleep(Duration::from_millis(500));
        
        log::info!("Starting lifecycle manager async tasks after delay");
        if let Ok(mut manager) = lifecycle_manager_for_startup.lock() {
            if let Err(e) = manager.start_async_tasks() {
                log::error!("Failed to start lifecycle manager async tasks: {}", e);
            } else {
                log::info!("Successfully started lifecycle manager async tasks");
            }
        } else {
            log::error!("Failed to acquire lock on lifecycle manager for async task startup");
        }
    });
    
    let settings = iced::Settings {
        id: None,
        window: iced::window::Settings {
            visible: true,
            icon,
            ..Default::default()
        },
        flags: flags,
        default_font: iced::Font::default(),
        default_text_size: 16.0,
        antialiasing: false,
        exit_on_close_request: true,
    };
    let result = StateApp::run(settings);
    
    // Handle any errors during UI execution
    if let Err(ref e) = result {
        log::error!("Error during UI execution: {}", e);
    }
    
    log::info!("UI has exited, performing cleanup");
    
    // Check controller thread status (will only happen if UI has exited)
    if controller_handle.is_finished() {
        log::info!("Controller thread has completed");
        if let Err(e) = controller_handle.join() {
            log::error!("Controller thread panicked: {:?}", e);
        }
    } else {
        log::info!("Controller thread is still running after UI exit");
        // Note: We can't gracefully stop the thread from here, but that's okay
        // as the process will terminate anyway
    }
    
    result
} 