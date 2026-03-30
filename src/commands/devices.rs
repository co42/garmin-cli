use crate::client::GarminClient;
use crate::error::Result;
use crate::output::{HumanReadable, Output};
use colored::Colorize;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Device {
    pub id: u64,
    pub device_name: String,
    pub device_type: String,
    pub serial_number: Option<String>,
    pub firmware_version: Option<String>,
    pub last_sync: Option<String>,
}

impl HumanReadable for Device {
    fn print_human(&self) {
        if self.device_type.is_empty() {
            println!("{}", self.device_name.bold());
        } else {
            println!(
                "{} ({})",
                self.device_name.bold(),
                self.device_type.dimmed()
            );
        }
        println!("  {:<14}{}", "ID:", self.id);
        if let Some(ref sn) = self.serial_number {
            println!("  {:<14}{sn}", "Serial:");
        }
        if let Some(ref fw) = self.firmware_version {
            println!("  {:<14}{fw}", "Firmware:");
        }
        if let Some(ref sync) = self.last_sync {
            println!("  {:<14}{sync}", "Last sync:");
        }
        println!();
    }
}

pub async fn list(client: &GarminClient, output: &Output) -> Result<()> {
    let v: Vec<serde_json::Value> = client
        .get_json("/device-service/deviceregistration/devices")
        .await?;

    let devices: Vec<Device> = v
        .iter()
        .map(|d| Device {
            id: d["deviceId"].as_u64().unwrap_or(0),
            device_name: d["displayName"].as_str().unwrap_or("Unknown").into(),
            device_type: d["deviceTypeName"].as_str().unwrap_or("").into(),
            serial_number: d["serialNumber"].as_str().map(Into::into),
            firmware_version: d["currentFirmwareVersion"].as_str().map(Into::into),
            last_sync: d["lastSyncTime"].as_str().map(Into::into),
        })
        .collect();

    output.print_list(&devices, "Devices");
    Ok(())
}

pub async fn get(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    let v: serde_json::Value = client
        .get_json(&format!("/device-service/deviceregistration/devices/{id}"))
        .await?;

    let device = Device {
        id,
        device_name: v["displayName"].as_str().unwrap_or("Unknown").into(),
        device_type: v["deviceTypeName"].as_str().unwrap_or("").into(),
        serial_number: v["serialNumber"].as_str().map(Into::into),
        firmware_version: v["currentFirmwareVersion"].as_str().map(Into::into),
        last_sync: v["lastSyncTime"].as_str().map(Into::into),
    };
    output.print(&device);
    Ok(())
}
