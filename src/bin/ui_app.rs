use rustpods::ui;

fn main() {
    println!("Launching RustPods UI...");
    match ui::run_ui() {
        Ok(_) => {}
        Err(e) => eprintln!("Error launching UI: {}", e),
    }
}
