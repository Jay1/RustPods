//! UI application and related functionality

use iced::widget::text;
use iced::{Settings, Subscription, Application, Element, Command};
use tokio::sync::mpsc;
use crate::ui::{Message, UiComponent};
use crate::app_controller::AppController;
use crate::ui::state::AppState;
use crate::ui::theme::Theme;
use std::sync::Arc;

use crate::ui::state_manager::StateManager;
use crate::ui::window_management::{DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH};
use crate::ui::state_app::run_state_ui;

/// Application entry point using Iced
pub struct App {
    state_manager: Arc<StateManager>,
}

impl Application for App {
    type Message = Message;
    type Theme = crate::ui::theme::Theme;
    type Executor = iced::executor::Default;
    type Flags = Arc<StateManager>;

    fn new(flags: Self::Flags) -> (Self, Command<Message>) {
        let state_manager = flags;
        
        (
            Self {
                state_manager,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "RustPods".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Exit => iced::window::close(),
            _ => Command::none(),
        }
    }

    fn view(&self) -> Element<'_, Message, iced::Renderer<Self::Theme>> {
        // This is just a placeholder - the actual UI is managed by the system tray
        text("RustPods running in system tray").into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}

/// Create application settings for Iced
pub fn create_app_settings(state_manager: Arc<StateManager>) -> Settings<Arc<StateManager>> {
    Settings {
        id: Some("rustpods".to_string()),
        window: iced::window::Settings {
            size: (DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT),
            position: iced::window::Position::Default,
            min_size: Some((400, 300)),
            max_size: None,
            visible: false,
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
    }
}

/// Runs the UI application
pub fn run_ui() -> iced::Result {
    // Create a channel for communication with the AppController
    let (sender, receiver) = mpsc::unbounded_channel();
    
    // Create the AppController but don't initialize it yet
    let app_controller = AppController::new(sender.clone());
    
    // Create a synchronous channel to handle controller initialization
    let (controller_tx, controller_rx) = std::sync::mpsc::channel();
    
    // Spawn a separate thread to manage the controller and tokio runtime
    std::thread::spawn(move || {
        // Create a dedicated tokio runtime for the controller thread
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime");
            
        // Box the controller so we can send it to the processing function
        let mut controller = Box::new(app_controller);
        
        // Initialize the controller inside this runtime
        let init_result = rt.block_on(async {
            controller.initialize().await
        });
        
        if let Err(e) = init_result {
            eprintln!("Failed to initialize AppController: {}", e);
            controller_tx.send(Err(e)).expect("Failed to send initialization result");
            return;
        }
        
        // Send back success
        controller_tx.send(Ok(())).expect("Failed to send initialization result");
        
        // Now run the message processing loop in this runtime
        rt.block_on(async {
            let mut rx = receiver;
            process_messages(&mut rx, &mut controller).await;
        });
    });
    
    // Wait for the controller initialization to complete
    if let Err(e) = controller_rx.recv().expect("Failed to receive initialization result") {
        return Err(iced::Error::WindowCreationFailed(e.into()));
    }
    
    // Create the state manager
    let state_manager = Arc::new(StateManager::new(sender));
    
    // Run the Iced application using state UI
    run_state_ui()
}

/// Process messages received from the UI
async fn process_messages(
    receiver: &mut mpsc::UnboundedReceiver<Message>,
    controller: &mut Box<AppController>,
) {
    while let Some(message) = receiver.recv().await {
        match message {
            Message::StartScan => {
                if let Err(e) = controller.start_scanning() {
                    eprintln!("Failed to start scanning: {}", e);
                }
            }
            Message::StopScan => {
                if let Err(e) = controller.stop_scanning() {
                    eprintln!("Failed to stop scanning: {}", e);
                }
            }
            Message::SelectDevice(address) => {
                if let Err(e) = controller.connect_device(address) {
                    eprintln!("Failed to connect to device: {}", e);
                }
            }
            Message::Exit => {
                if let Err(e) = controller.shutdown() {
                    eprintln!("Failed to shutdown controller: {}", e);
                }
                // Actually exit the application
                std::process::exit(0);
            }
            _ => {
                // Other messages are not handled by the controller
            }
        }
    }
}

/// Creates a subscription for updating the UI
pub fn subscription(state: &AppState) -> Subscription<Message> {
    if state.is_scanning {
        // If we're scanning, create a subscription for animation updates
        // In animation mode, send animation tick events
        iced::time::every(std::time::Duration::from_millis(16))
            .map(move |_instant| {
                // Animation tick without progress calculation
                Message::AnimationTick
            })
    } else {
        // Regular tick for other updates (1 second)
        iced::time::every(std::time::Duration::from_secs(1))
            .map(|_instant| Message::Tick)
    }
}

/// Creates the user interface
pub fn view(state: &AppState) -> Element<'_, Message, iced::Renderer<Theme>> {
    if !state.visible {
        // Return an empty container when not visible (system tray mode)
        text("").into()
    } else if state.show_settings {
        // Use our new settings window component
        state.settings_window.view()
    } else {
        state.main_window.view()
    }
} 