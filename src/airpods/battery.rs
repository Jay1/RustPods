use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct AirPodsBatteryInfo {
    pub address: u64,
    pub name: String,
    pub model_id: u16,
    pub left_battery: i32,
    pub left_charging: bool,
    pub right_battery: i32,
    pub right_charging: bool,
    pub case_battery: i32,
    pub case_charging: bool,
    pub left_in_ear: Option<bool>,
    pub right_in_ear: Option<bool>,
    pub case_lid_open: Option<bool>,
    pub side: Option<i32>,
    pub both_in_case: Option<bool>,
    pub color: Option<i32>,
    pub switch_count: Option<i32>,
    pub rssi: Option<i32>,
    pub timestamp: Option<u64>,
    pub raw_manufacturer_data: Option<String>,
}

pub fn get_airpods_battery_info(helper_path: &str) -> Vec<AirPodsBatteryInfo> {
    let output = Command::new(helper_path)
        .output()
        .expect("Failed to run airpods_battery_helper.exe");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut infos = Vec::new();

    // Each JSON object is on its own line
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || !trimmed.starts_with('{') {
            continue;
        }
        match serde_json::from_str::<AirPodsBatteryInfo>(trimmed) {
            Ok(info) => infos.push(info),
            Err(e) => eprintln!(
                "[airpods_battery] Failed to parse line as AirPodsBatteryInfo: {}\nLine: {}",
                e, trimmed
            ),
        }
    }
    infos
}

pub fn model_name(model_id: u16) -> &'static str {
    match model_id {
        0x2002 => "AirPods 1",
        0x200F => "AirPods 2",
        0x2013 => "AirPods 3",
        0x200E => "AirPods Pro",
        0x2014 => "AirPods Pro 2",
        0x2024 => "AirPods Pro 2 (USB-C)",
        0x200A => "AirPods Max",
        0x2012 => "Beats Fit Pro",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_compact_json_lines() {
        let sample = r#"
{"address":123,"name":"AirPods","model_id":8206,"left_battery":90,"left_charging":true,"right_battery":80,"right_charging":false,"case_battery":70,"case_charging":true}
{"address":456,"name":"","model_id":8211,"left_battery":100,"left_charging":false,"right_battery":100,"right_charging":false,"case_battery":100,"case_charging":false}
        "#;
        let mut infos = Vec::new();
        for line in sample.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let info: AirPodsBatteryInfo = serde_json::from_str(trimmed).unwrap();
            infos.push(info);
        }
        assert_eq!(infos.len(), 2);
        assert_eq!(infos[0].address, 123);
        assert_eq!(infos[0].name, "AirPods");
        assert_eq!(infos[0].model_id, 0x200E);
        assert!(infos[0].left_charging);
        assert_eq!(infos[1].address, 456);
        assert_eq!(infos[1].model_id, 0x2013);
    }
}
