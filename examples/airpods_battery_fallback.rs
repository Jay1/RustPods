use tokio::runtime::Runtime;

fn main() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        use rustpods::bluetooth::airpods_bridge::get_airpods_battery_via_python;
        println!("Querying AirPods battery via Python Bleak fallback...");
        match get_airpods_battery_via_python().await {
            Ok(airpods) => {
                if airpods.is_empty() {
                    println!("No AirPods found.");
                } else {
                    for ap in airpods {
                        println!("Found: {} ({}), battery: {}%", ap.name, ap.address, ap.battery);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: {e}");
            }
        }
    });
} 