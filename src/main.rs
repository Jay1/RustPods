//! To run the main UI: cargo run
//! To run the state-managed UI: cargo run --bin rustpods stateui
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
pub mod assets;

use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::path::PathBuf;

use log::{info, error};
use error::{ErrorManager, RustPodsError, ErrorContext};
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
    // Load or create a configuration file first to get logging settings
    let config = match config::load_or_create_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error loading configuration: {}", e);
            AppConfig::default()
        }
    };
    
    // Initialize structured logging with config settings
    let log_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("RustPods")
        .join("logs");
    
    let log_file = log_dir.join(format!("rustpods_{}.log", 
        chrono::Local::now().format("%Y%m%d_%H%M%S")));
    
    if let Err(e) = logging::configure_logging(config.system.log_level, Some(log_file), true) {
        eprintln!("Failed to setup logging: {}", e);
    }
    
    // Create error context for application startup
    let ctx = ErrorContext::new("Main", "startup")
        .with_metadata("version", env!("CARGO_PKG_VERSION"));
    
    info!("RustPods v{} - Starting up application", env!("CARGO_PKG_VERSION"));
    
    // Create a Tokio runtime
    let rt = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(e) => {
            let _ctx = ctx.with_metadata("error", e.to_string());
            error!("Failed to create Tokio runtime: {}", e);
            std::process::exit(1);
        }
    };
    
    // Run the main app code inside the runtime
    rt.block_on(async {
        main_async().await;
    });
}

async fn main_async() {
    // Create error context for async initialization
    let ctx = ErrorContext::new("Main", "main_async")
        .with_metadata("runtime", "tokio");
    
    // Parse command line arguments
    let command = match parse_args() {
        Ok(cmd) => cmd,
        Err(e) => {
            eprintln!("Error parsing arguments: {}", e);
            print_usage();
            std::process::exit(1);
        }
    };
    
    // Load or create a configuration file
    let config = match config::load_or_create_config() {
        Ok(cfg) => {
            info!("Configuration loaded successfully");
            cfg
        },
        Err(e) => {
            let _ctx = ctx.clone().with_metadata("error", e.to_string());
            logging::log_error(&e, &ctx);
            error!("Error loading configuration: {}", e);
            AppConfig::default()
        }
    };
    
    // Handle special commands first
    match command {
        AppCommand::Help => {
            print_usage();
            return;
        },
        AppCommand::UI => {
            info!("Launching UI...");
            if let Err(e) = ui::run_ui() {
                error!("Failed to run UI: {}", e);
                std::process::exit(1);
            }
            return;
        },
        AppCommand::StateUI => {
            info!("Launching State UI...");
            if let Err(e) = ui::run_state_ui() {
                error!("Failed to run State UI: {}", e);
                std::process::exit(1);
            }
            return;
        },
        _ => {
            // Handle other commands with the existing system
        }
    }
    
    // Create UI message channel
    let (ui_sender, _ui_receiver) = mpsc::unbounded_channel::<Message>();
    
    // Create the state manager
    let state_manager = Arc::new(StateManager::new(ui_sender.clone()));
    
    // Create error manager
    let error_manager = Arc::new(Mutex::new(ErrorManager::new()));
    
    // Create telemetry manager
    let telemetry_manager = Arc::new(Mutex::new(TelemetryManager::new(&config)));
    
    // Configure logger with settings from config
    init_logging_from_config(&config);
    
    // Execute the remaining commands
    let config = Arc::new(Mutex::new(config));
    if let Err(exit_code) = execute_command(command, config, error_manager, telemetry_manager).await {
        std::process::exit(exit_code);
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
    let config_ref = match config.lock() {
        Ok(guard) => Arc::new(guard.clone()),
        Err(_) => return Err("Failed to lock config mutex".to_string()),
    };
    
    let mut diagnostics = diagnostics::DiagnosticsManager::new(config_ref, error_manager);
    
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
    E: std::fmt::Display + std::fmt::Debug,
{
    // Create error context
    let _ctx = ErrorContext::new("CommandExecution", operation)
        .with_metadata("operation", operation.to_string());
    
    // Log the error with context
    logging::log_error(&error, &_ctx);
    
    // Log to console as well
    error!("Error while {}: {:?}", operation, error);
    
    // Register the error with the error manager
    if let Ok(mut manager) = error_manager.lock() {
        manager.record_error(&RustPodsError::System(format!(
            "Command execution error while {}: {}", 
            operation, 
            error
        )));
    } else {
        // If we can't lock the error manager, just log the error
        error!("Failed to lock error manager to register error: {:?}", error);
    }
    
    // Suggest recovery action based on the operation
    let recovery_action = match operation {
        "discovering Bluetooth adapters" => 
            "Check if Bluetooth is enabled on your system and you have administrator privileges",
        "scanning for devices" => 
            "Ensure Bluetooth is enabled and try restarting the Bluetooth service",
        "interval scanning" => 
            "Check if there are any Bluetooth devices in range and try again",
        "AirPods filtering" => 
            "Make sure your AirPods are in pairing mode and try again",
        _ => 
            "Try running the command again or check system logs for more details",
    };
    
    logging::log_error_with_recovery(&error, &_ctx, recovery_action);
    println!("Suggested action: {}", recovery_action);
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

/// Initialize logging from the application configuration
fn init_logging_from_config(config: &AppConfig) {
    // Use default log path in data directory if enabled in config
    let log_path = if true { // Using hardcoded true since log_to_file doesn't exist
        let mut path = dirs::data_local_dir()
            .unwrap_or_else(|| std::env::temp_dir())
            .join("RustPods")
            .join("logs");
            
        if !path.exists() {
            match std::fs::create_dir_all(&path) {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("Failed to create log directory: {}", e);
                    path = std::env::temp_dir();
                }
            }
        }
        path.push("rustpods.log");
        Some(path)
    } else {
        None
    };
    
    if let Err(e) = logging::configure_logging(config.system.log_level.clone(), log_path, true) {
        eprintln!("Failed to configure logging: {}", e);
    }
}
