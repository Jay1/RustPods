// Module exports for project structure
pub mod bluetooth;
pub mod airpods;
pub mod ui;
pub mod config;
pub mod app;
pub mod error;

#[tokio::main]
async fn main() {
    println!("RustPodsMon - AirPods Battery Monitor");
    
    // Get command line args to determine what to run
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "adapters" => {
                println!("Discovering Bluetooth adapters...");
                if let Err(e) = bluetooth::discover_adapters().await {
                    eprintln!("Error discovering adapters: {}", e);
                }
            },
            "scan" => {
                println!("Running Bluetooth scan...");
                if let Err(e) = bluetooth::scan_with_adapter().await {
                    eprintln!("Error scanning for devices: {}", e);
                }
            },
            "interval" => {
                println!("Running interval-based scanning...");
                if let Err(e) = bluetooth::interval_scanning().await {
                    eprintln!("Error during interval scanning: {}", e);
                }
            },
            "airpods" => {
                println!("Running AirPods filtering demo...");
                if let Err(e) = bluetooth::airpods_filtering().await {
                    eprintln!("Error during AirPods filtering: {}", e);
                }
            },
            "events" => {
                println!("Running event system demo...");
                // This runs as a separate example now
                println!("To run the event system demo, use: cargo run --example event_system");
            },
            "ui" => {
                println!("Launching UI application...");
                if let Err(e) = ui::run_ui() {
                    eprintln!("Error launching UI: {}", e);
                }
            },
            _ => {
                print_usage();
            }
        }
    } else {
        // Default: discover adapters
        println!("Discovering Bluetooth adapters...");
        if let Err(e) = bluetooth::discover_adapters().await {
            eprintln!("Error discovering adapters: {}", e);
        }
        
        print_usage();
    }
}

fn print_usage() {
    println!("\nUsage:");
    println!("  rustpodsmon adapters    - Discover Bluetooth adapters");
    println!("  rustpodsmon scan        - Run a basic Bluetooth scan");
    println!("  rustpodsmon interval    - Run interval-based scanning");
    println!("  rustpodsmon airpods     - Run AirPods filtering demo");
    println!("  rustpodsmon events      - Run event system demo (use cargo run --example event_system)");
    println!("  rustpodsmon ui          - Launch the UI application");
}
