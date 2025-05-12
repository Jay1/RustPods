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

use std::process;

enum AppCommand {
    Adapters,
    Scan,
    Interval,
    AirPods,
    Events,
    UI,
    StateUI, // New command for using the state-based UI
    Help,
}

fn main() {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();
    
    log::info!("RustPods - Starting up application");
    
    // Parse command-line arguments
    match parse_args() {
        Ok(command) => {
            // For UI commands, use a different execution path to avoid Tokio runtime conflicts
            match command {
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
                        eprintln!("Error launching state-based UI: {}", e);
                        process::exit(6);
                    }
                    return;
                },
                _ => {
                    // For other commands, use tokio runtime
                    let runtime = tokio::runtime::Builder::new_multi_thread()
                        .enable_all()
                        .build()
                        .expect("Failed to create tokio runtime");
                        
                    let result = runtime.block_on(async {
                        execute_command(command).await
                    });
                    
                    if let Err(err_code) = result {
                        log::error!("Command failed with error code: {}", err_code);
                        process::exit(err_code);
                    }
                }
            }
        },
        Err(err_msg) => {
            log::error!("Error parsing arguments: {}", err_msg);
            eprintln!("Error: {}", err_msg);
            print_usage();
            process::exit(1);
        }
    }
    
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
        "help" | "--help" | "-h" => Ok(AppCommand::Help),
        _ => Err(format!("Unknown command: '{}'", args[1]))
    }
}

async fn execute_command(command: AppCommand) -> Result<(), i32> {
    match command {
        AppCommand::Adapters => {
            println!("Discovering Bluetooth adapters...");
            if let Err(e) = bluetooth::discover_adapters().await {
                eprintln!("Error discovering adapters: {}", e);
                return Err(2); // Error code 2 for Bluetooth adapter issues
            }
        },
        AppCommand::Scan => {
            println!("Running Bluetooth scan...");
            if let Err(e) = bluetooth::scan_with_adapter().await {
                eprintln!("Error scanning for devices: {}", e);
                return Err(3); // Error code 3 for scanning issues
            }
        },
        AppCommand::Interval => {
            println!("Running interval-based scanning...");
            if let Err(e) = bluetooth::interval_scanning().await {
                eprintln!("Error during interval scanning: {}", e);
                return Err(4); // Error code 4 for interval scanning issues
            }
        },
        AppCommand::AirPods => {
            println!("Running AirPods filtering demo...");
            if let Err(e) = bluetooth::airpods_filtering().await {
                eprintln!("Error during AirPods filtering: {}", e);
                return Err(5); // Error code 5 for AirPods filtering issues
            }
        },
        AppCommand::Events => {
            println!("Running event system demo...");
            println!("To run the event system demo, use: cargo run --example event_system");
        },
        AppCommand::UI | AppCommand::StateUI => {
            // UI is handled separately in main to avoid Tokio runtime conflicts
            unreachable!("UI commands should be handled outside this function");
        },
        AppCommand::Help => {
            print_usage();
        }
    }
    
    Ok(())
}

fn print_usage() {
    println!("\nUsage:");
    println!("  rustpods adapters    - Discover Bluetooth adapters");
    println!("  rustpods scan        - Run a basic Bluetooth scan");
    println!("  rustpods interval    - Run interval-based scanning");
    println!("  rustpods airpods     - Run AirPods filtering demo");
    println!("  rustpods events      - Run event system demo (use cargo run --example event_system)");
    println!("  rustpods ui          - Launch the UI application with original state management");
    println!("  rustpods stateui     - Launch the UI application with new state management");
    println!("  rustpods help        - Show this help message");
}
