use super::output::Output;
use crate::error::Result;
use crate::garmin::GarminClient;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum DeviceCommands {
    /// List registered devices
    List,
    /// Get device details
    Get {
        /// Device ID
        id: u64,
    },
}

pub async fn run(command: DeviceCommands, output: &Output) -> Result<()> {
    let client = GarminClient::new(super::helpers::require_auth()?)?;
    match command {
        DeviceCommands::List => list(&client, output).await,
        DeviceCommands::Get { id } => get(&client, output, id).await,
    }
}

async fn list(client: &GarminClient, output: &Output) -> Result<()> {
    let devices = client.list_devices().await?;
    output.print_list(&devices, "Devices");
    Ok(())
}

async fn get(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    let mut device = client.device(id).await?;
    // API sometimes omits ID on single-device lookup; populate from caller.
    if device.device_id == 0 {
        device.device_id = id;
    }
    output.print(&device);
    Ok(())
}
