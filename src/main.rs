// Module exports for project structure
pub mod bluetooth;
pub mod airpods;
pub mod ui;
pub mod config;
pub mod app;
pub mod error;
pub mod app_controller;
pub mod app_state_controller;
pub mod lifecycle_manager;
pub mod state_persistence;
pub mod logging;
pub mod telemetry;
pub mod diagnostics;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use log::info;
use error::{ErrorManager, RustPodsError};
use telemetry::TelemetryManager;
use config::AppConfig;
use ui::state_manager::StateManager;
use ui::Message;
use tokio::sync::mpsc;
use crate::lifecycle_manager::LifecycleManager;

enum AppCommand {
    Adapters,
    Scan,
    Interval,
    AirPods,
    Events,
    UI,
    StateUI, // New command for using the state-based UI
    Diagnostic, // New command for running diagnostics
    Help,
}

fn main() {
    // Initialize logging first
    logging::init_logger(&AppConfig::default()).expect("Failed to setup logging");
    
    info!("RustPods v{} - Starting up application", env!("CARGO_PKG_VERSION"));
    
    // Create a Tokio runtime
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    
    // Run the main app code inside the runtime
    rt.block_on(async {
        main_async().await;
    });
}

async fn main_async() {
    // Load or create a configuration file
    let config = match config::load_or_create_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error loading configuration: {}", e);
            AppConfig::default()
        }
    };
    
    // Create UI message channel
    let (ui_sender, _ui_receiver) = mpsc::unbounded_channel::<Message>();
    
    // Create the state manager
    let state_manager = Arc::new(StateManager::new(ui_sender.clone()));

    // Initialize error manager
    let _error_manager = Arc::new(Mutex::new(ErrorManager::new()));
    
    // Initialize telemetry manager
    let _telemetry_manager = Arc::new(Mutex::new(TelemetryManager::new(&config)));
    
    // Create lifecycle manager
    let mut lifecycle_manager = LifecycleManager::new(
        Arc::clone(&state_manager),
        ui_sender,
    );
    
    // Start the life cycle manager
    if let Err(e) = lifecycle_manager.start() {
        eprintln!("Failed to start lifecycle manager: {}", e);
    }
    
    // Keep the application running
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

fn parse_args() -> Result<AppCommand, String> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() <= 1 {
        // No command provided, default to StateUI - Using our new state management system
        return Ok(AppCommand::StateUI);
    }

    // Parse the command
    match args[1].to_lowercase().as_str() {
        "adapters" => Ok(AppCommand::Adapters),
        "scan" => Ok(AppCommand::Scan),
        "interval" => Ok(AppCommand::Interval),
        "airpods" => Ok(AppCommand::AirPods),
        "events" => Ok(AppCommand::Events),
        "ui" => Ok(AppCommand::UI),
        "stateui" => Ok(AppCommand::StateUI),
        "diagnostic" | "diagnostics" => Ok(AppCommand::Diagnostic),
        "help" | "--help" | "-h" => Ok(AppCommand::Help),
        _ => Err(format!("Unknown command: '{}'", args[1]))
    }
}

async fn execute_command(
    command: AppCommand, 
    config: Arc<Mutex<AppConfig>>,
    error_manager: Arc<Mutex<ErrorManager>>,
    _telemetry_manager: Arc<Mutex<telemetry::TelemetryManager>>,
) -> Result<(), i32> {
    match command {
        AppCommand::Adapters => {
            println!("Discovering Bluetooth adapters...");
            if let Err(e) = bluetooth::discover_adapters().await {
                handle_command_error(e, "discovering Bluetooth adapters", &error_manager);
                return Err(2); // Error code 2 for Bluetooth adapter issues
            }
        },
        AppCommand::Scan => {
            println!("Running Bluetooth scan...");
            if let Err(e) = bluetooth::scan_with_adapter().await {
                handle_command_error(e, "scanning for devices", &error_manager);
                return Err(3); // Error code 3 for scanning issues
            }
        },
        AppCommand::Interval => {
            println!("Running interval-based scanning...");
            if let Err(e) = bluetooth::interval_scanning().await {
                handle_command_error(e, "interval scanning", &error_manager);
                return Err(4); // Error code 4 for interval scanning issues
            }
        },
        AppCommand::AirPods => {
            println!("Running AirPods filtering demo...");
            if let Err(e) = bluetooth::airpods_filtering().await {
                handle_command_error(e, "AirPods filtering", &error_manager);
                return Err(5); // Error code 5 for AirPods filtering issues
            }
        },
        AppCommand::Events => {
            println!("Running event system demo...");
            println!("To run the event system demo, use: cargo run --example event_system");
        },
        AppCommand::Diagnostic => {
            println!("Running system diagnostics...");
            if let Err(e) = run_diagnostics(Arc::clone(&config), Arc::clone(&error_manager)).await {
                eprintln!("Error running diagnostics: {}", e);
                return Err(7); // Error code 7 for diagnostic issues
            }
        },
        AppCommand::UI | AppCommand::StateUI | AppCommand::Help => {
            // These are handled in main() before this function
            unreachable!("These commands should be handled before calling execute_command");
        }
    }
    
    Ok(())
}

// Helper function to run system diagnostics
async fn run_diagnostics(
    config: Arc<Mutex<AppConfig>>,
    error_manager: Arc<Mutex<ErrorManager>>,
) -> Result<(), String> {
    // Create diagnostics manager
    let mut diagnostics = diagnostics::DiagnosticsManager::new(config, error_manager);
    
    // Set to complete diagnostic level
    diagnostics.set_level(diagnostics::DiagnosticLevel::Complete);
    
    // Run diagnostics
    let result = match diagnostics.run_diagnostics().await {
        Ok(r) => r,
        Err(e) => return Err(format!("Failed to run diagnostics: {}", e)),
    };
    
    // Save diagnostic report
    let report_path = match diagnostics.save_diagnostics() {
        Ok(path) => path,
        Err(e) => return Err(format!("Failed to save diagnostic report: {}", e)),
    };
    
    // Print summary
    println!("\n==== Diagnostic Results ====");
    println!("Issues found: {}", result.issues.len());
    println!("Critical issues: {}", result.issues.iter().filter(|i| i.severity == diagnostics::IssueSeverity::Critical).count());
    println!("Major issues: {}", result.issues.iter().filter(|i| i.severity == diagnostics::IssueSeverity::Major).count());
    println!("Minor issues: {}", result.issues.iter().filter(|i| i.severity == diagnostics::IssueSeverity::Minor).count());
    println!("\nDiagnostic report saved to: {}", report_path.display());
    
    // Print issues
    if !result.issues.is_empty() {
        println!("\n==== Issues ====");
        for (i, issue) in result.issues.iter().enumerate() {
            let severity = match issue.severity {
                diagnostics::IssueSeverity::Critical => "CRITICAL",
                diagnostics::IssueSeverity::Major => "MAJOR",
                diagnostics::IssueSeverity::Minor => "MINOR",
                diagnostics::IssueSeverity::Info => "INFO",
            };
            println!("{}. [{}] {}", i + 1, severity, issue.title);
            println!("   {}", issue.description);
        }
    }
    
    // Print recommendations
    if !result.recommendations.is_empty() {
        println!("\n==== Recommendations ====");
        for (i, rec) in result.recommendations.iter().enumerate() {
            println!("{}. {}", i + 1, rec);
        }
    }
    
    Ok(())
}

// Helper function to handle command errors consistently
fn handle_command_error<E>(
    error: E,
    operation: &str,
    error_manager: &Arc<Mutex<ErrorManager>>,
) where
    E: std::fmt::Display,
{
    log::error!("Error while {}: {}", operation, error);
    eprintln!("Error while {}: {}", operation, error);
    
    // Create a RustPodsError from the error string
    let rustpods_error = RustPodsError::System(format!("Error during {}: {}", operation, error));
    
    // Record in error manager
    if let Ok(mut em) = error_manager.lock() {
        em.record_error(&rustpods_error);
    }
}

fn print_usage() {
    println!("\nRustPods v{} - AirPods Battery Monitor for Windows", env!("CARGO_PKG_VERSION"));
    println!("\nUsage:");
    println!("  rustpods                - Launch the UI application (default)");
    println!("  rustpods adapters       - Discover Bluetooth adapters");
    println!("  rustpods scan           - Run a basic Bluetooth scan");
    println!("  rustpods interval       - Run interval-based scanning");
    println!("  rustpods airpods        - Run AirPods filtering demo");
    println!("  rustpods events         - Run event system demo (use cargo run --example event_system)");
    println!("  rustpods ui             - Launch the UI application with original state management");
    println!("  rustpods stateui        - Launch the UI application with new state management");
    println!("  rustpods diagnostic     - Run system diagnostics");
    println!("  rustpods help           - Show this help message");
}
