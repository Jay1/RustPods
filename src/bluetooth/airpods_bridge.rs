//! Bridge to call the Python Bleak fallback for AirPods battery on Windows

use std::process::Stdio;
use std::path::Path;
use serde::Deserialize;
use tokio::process::Command;
use tracing::{info, error};

#[derive(Debug, Clone, Deserialize)]
pub struct AirPodsBatteryInfo {
    pub name: String,
    pub address: String,
    pub battery: u8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PairedBluetoothDevice {
    pub name: String,
    pub address: String,
    pub paired: bool,
    pub connected: bool,
    pub device_type: Option<String>,
}

/// Calls the Python Bleak script and returns a list of AirPods with battery info
pub async fn get_airpods_battery_via_python() -> Result<Vec<AirPodsBatteryInfo>, String> {
    let exe_path = Path::new("scripts/airpods_ble_scan.exe");
    if !exe_path.exists() {
        return Err("Bundled BLE scanner not found: scripts/airpods_ble_scan.exe".to_string());
    }
    let output = Command::new(exe_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to run BLE scanner .exe: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("BLE scanner .exe error: {stderr}");
        return Err(format!("BLE scanner .exe failed: {stderr}"));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    info!("BLE scanner .exe output: {stdout}");
    let airpods: Vec<AirPodsBatteryInfo> = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse JSON output: {e}\nOutput: {stdout}"))?;
    Ok(airpods)
}

/// Calls the Python Win32/WinRT script and returns a list of paired/connected Bluetooth devices
pub async fn get_paired_devices_via_python() -> Result<Vec<PairedBluetoothDevice>, String> {
    let exe_path = Path::new("scripts/bt_paired_devices.exe");
    if !exe_path.exists() {
        return Err("Bundled paired device scanner not found: scripts/bt_paired_devices.exe".to_string());
    }
    let output = Command::new(exe_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to run paired device scanner .exe: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("Paired device scanner .exe error: {stderr}");
        return Err(format!("Paired device scanner .exe failed: {stderr}"));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    info!("Paired device scanner .exe output: {stdout}");
    let devices: Vec<PairedBluetoothDevice> = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse JSON output: {e}\nOutput: {stdout}"))?;
    Ok(devices)
} 