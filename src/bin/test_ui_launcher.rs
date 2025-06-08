use std::process;

use rustpods::init_logging;

fn main() {
    // Initialize logging
    init_logging();

    println!("Starting RustPods UI...");

    // Since the application creates its own Tokio runtime internally,
    // we should just call the state UI directly without wrapping it in another runtime
    // This avoids the runtime shutdown panic we were seeing
    match rustpods::ui::run_state_ui() {
        Ok(_) => {
            println!("UI closed successfully");
            process::exit(0);
        }
        Err(e) => {
            eprintln!("Error running UI: {}", e);
            process::exit(1);
        }
    }
}
