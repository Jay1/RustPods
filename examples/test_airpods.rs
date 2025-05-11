use rustpods::bluetooth;

#[tokio::main]
async fn main() {
    println!("Testing AirPods filter functionality...");
    
    if let Err(e) = bluetooth::airpods_filtering().await {
        eprintln!("Error during AirPods filtering test: {}", e);
    } else {
        println!("AirPods filtering test completed successfully.");
    }
} 