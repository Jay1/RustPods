# RustPodsMon

A Rust application for monitoring and controlling Apple AirPods and other Bluetooth headphones.

## Setup

1. Ensure you have Rust and Cargo installed. See https://www.rust-lang.org/tools/install
2. Clone this repository or create the project using `cargo new rustpodsmon`.
3. Add the required dependencies in `Cargo.toml`:
   - btleplug = "0.10"
   - iced = "0.13"
   - tray-item = "0.7"
   - serde = { version = "1.0", features = ["derive"] }
   - serde_json = "1.0"
   - tokio = { version = "1.0", features = ["full"] }
4. Build the project:
   ```sh
   cargo build
   ```

## Usage

- Run the application:
  ```sh
  cargo run
  ```
- The app will provide a native UI for interacting with Bluetooth headphones.

## License

MIT 