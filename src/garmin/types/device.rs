use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Device {
    pub device_id: u64,
    pub display_name: String,
    #[serde(default)]
    pub device_type_name: String,
    pub serial_number: Option<String>,
    pub current_firmware_version: Option<String>,
    pub last_sync_time: Option<String>,
}

impl HumanReadable for Device {
    fn print_human(&self) {
        if self.device_type_name.is_empty() {
            println!("{}", self.display_name.bold());
        } else {
            println!("{} ({})", self.display_name.bold(), self.device_type_name.dimmed());
        }
        println!("  {:<LABEL_WIDTH$}{}", "ID:", self.device_id);
        if let Some(ref sn) = self.serial_number {
            println!("  {:<LABEL_WIDTH$}{sn}", "Serial:");
        }
        if let Some(ref fw) = self.current_firmware_version {
            println!("  {:<LABEL_WIDTH$}{fw}", "Firmware:");
        }
        if let Some(ref sync) = self.last_sync_time {
            println!("  {:<LABEL_WIDTH$}{sync}", "Last sync:");
        }
    }
}
