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

use std::process;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use error::{ErrorManager, ErrorSeverity, RustPodsError};

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
    // Parse command-line arguments
    let command = match parse_args() {
        Ok(cmd) => cmd,
        Err(err_msg) => {
            // Basic logging initialization for error reporting
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("error"))
                .format_timestamp_millis()
                .init();
            
            log::error!("Error parsing arguments: {}", err_msg);
            eprintln!("Error: {}", err_msg);
            print_usage();
            process::exit(1);
        }
    };

    // Initialize configuration
    let config = match config::load_or_create_config() {
        Ok(cfg) => {
            Arc::new(cfg)
        },
        Err(e) => {
            // Basic logging initialization for error reporting
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("error"))
                .format_timestamp_millis()
                .init();
                
            log::error!("Failed to load configuration: {}", e);
            eprintln!("Failed to load configuration: {}", e);
            process::exit(2);
        }
    };
    
    // Initialize enhanced logging system
    if let Err(e) = logging::init_logger(&config) {
        // Fall back to basic logging
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .format_timestamp_millis()
            .init();
            
        log::error!("Failed to initialize enhanced logging system: {}", e);
        eprintln!("Failed to initialize enhanced logging system: {}", e);
        // Continue with basic logging
    }
    
    log::info!("RustPods v{} - Starting up application", env!("CARGO_PKG_VERSION"));
    
    // Initialize error manager
    let error_manager = Arc::new(Mutex::new(ErrorManager::new()));
    
    // Initialize telemetry if enabled
    let telemetry_manager = telemetry::init_telemetry(&config);
    
    // Handle special commands that don't need a tokio runtime
    match command {
        AppCommand::Help => {
            print_usage();
            return;
        },
        AppCommand::UI => {
            log::info!("Launching UI mode");
            if let Err(e) = ui::run_ui() {
                log::error!("Error launching UI: {}", e);
                eprintln!("Error launching UI: {}", e);
                process::exit(6);
            }
            return;
        },
        AppCommand::StateUI => {
            log::info!("Launching state-based UI mode with lifecycle management");
            if let Err(e) = ui::run_state_ui() {
                log::error!("Error launching state-based UI: {}", e);
                
                // Record error in telemetry if enabled
                if let Ok(mut tm) = telemetry_manager.lock() {
                    let ui_error = RustPodsError::ui(
                        format!("Failed to start UI: {}", e),
                        ErrorSeverity::Critical
                    );
                    tm.record_error(&ui_error);
                }
                
                eprintln!("Error launching state-based UI: {}", e);
                process::exit(6);
            }
            return;
        },
        AppCommand::Diagnostic => {
            println!("Running system diagnostics...");
            if let Err(e) = run_diagnostics(Arc::clone(&config), Arc::clone(&error_manager), Arc::clone(&telemetry_manager)).await {
                eprintln!("Error running diagnostics: {}", e);
                process::exit(7); // Error code 7 for diagnostic issues
            }
        },
        _ => {
            // For other commands, use tokio runtime
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime");
                
            let result = runtime.block_on(async {
                execute_command(command, Arc::clone(&config), Arc::clone(&error_manager), Arc::clone(&telemetry_manager)).await
            });
            
            if let Err(err_code) = result {
                log::error!("Command failed with error code: {}", err_code);
                process::exit(err_code);
            }
        }
    }
    
    // Wait for logs to be flushed
    std::thread::sleep(Duration::from_millis(100));
    
    // Log application exit
    log::info!("RustPods - Application exiting normally");
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
    config: Arc<config::AppConfig>,
    error_manager: Arc<Mutex<ErrorManager>>,
    telemetry_manager: Arc<Mutex<telemetry::TelemetryManager>>,
) -> Result<(), i32> {
    match command {
        AppCommand::Adapters => {
            println!("Discovering Bluetooth adapters...");
            if let Err(e) = bluetooth::discover_adapters().await {
                handle_command_error(e, "discovering Bluetooth adapters", &error_manager, &telemetry_manager);
                return Err(2); // Error code 2 for Bluetooth adapter issues
            }
        },
        AppCommand::Scan => {
            println!("Running Bluetooth scan...");
            if let Err(e) = bluetooth::scan_with_adapter().await {
                handle_command_error(e, "scanning for devices", &error_manager, &telemetry_manager);
                return Err(3); // Error code 3 for scanning issues
            }
        },
        AppCommand::Interval => {
            println!("Running interval-based scanning...");
            if let Err(e) = bluetooth::interval_scanning().await {
                handle_command_error(e, "interval scanning", &error_manager, &telemetry_manager);
                return Err(4); // Error code 4 for interval scanning issues
            }
        },
        AppCommand::AirPods => {
            println!("Running AirPods filtering demo...");
            if let Err(e) = bluetooth::airpods_filtering().await {
                handle_command_error(e, "AirPods filtering", &error_manager, &telemetry_manager);
                return Err(5); // Error code 5 for AirPods filtering issues
            }
        },
        AppCommand::Events => {
            println!("Running event system demo...");
            println!("To run the event system demo, use: cargo run --example event_system");
        },
        AppCommand::Diagnostic => {
            println!("Running system diagnostics...");
            if let Err(e) = run_diagnostics(Arc::clone(&config), Arc::clone(&error_manager), Arc::clone(&telemetry_manager)).await {
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
    config: Arc<config::AppConfig>,
    error_manager: Arc<Mutex<ErrorManager>>,
    telemetry_manager: Arc<Mutex<telemetry::TelemetryManager>>,
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
fn handle_command_error(
    error: impl std::error::Error,
    operation: &str,
    error_manager: &Arc<Mutex<ErrorManager>>,
    telemetry_manager: &Arc<Mutex<telemetry::TelemetryManager>>,
) {
    eprintln!("Error during {}: {}", operation, error);
    
    // Convert error to RustPodsError if needed
    let rustpods_error = match error.downcast_ref::<RustPodsError>() {
        Some(e) => e.clone(),
        None => RustPodsError::system(
            format!("Error during {}: {}", operation, error),
            ErrorSeverity::Error,
        ),
    };
    
    // Log the error properly
    rustpods_error.log();
    
    // Record in telemetry
    if let Ok(mut tm) = telemetry_manager.lock() {
        tm.record_error(&rustpods_error);
    }
    
    // Store in error manager
    if let Ok(mut em) = error_manager.lock() {
        em.add_to_history(rustpods_error);
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
