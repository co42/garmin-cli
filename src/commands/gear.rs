use super::output::Output;
use crate::error::Result;
use crate::garmin::GarminClient;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum GearCommands {
    /// List all gear
    List,
    /// Get gear usage statistics
    Stats {
        /// Gear UUID
        uuid: String,
    },
    /// Link gear to an activity
    Link {
        /// Gear UUID
        uuid: String,
        /// Activity ID
        activity_id: u64,
    },
}

pub async fn run(command: GearCommands, output: &Output) -> Result<()> {
    let client = GarminClient::new(super::helpers::require_auth()?)?;
    match command {
        GearCommands::List => list(&client, output).await,
        GearCommands::Stats { uuid } => stats(&client, output, &uuid).await,
        GearCommands::Link { uuid, activity_id } => link(&client, output, &uuid, activity_id).await,
    }
}

async fn list(client: &GarminClient, output: &Output) -> Result<()> {
    let mut items = client.list_gear().await?;

    // Bulk listing doesn't include distance/activities — enrich from the stats endpoint.
    let stats_futs = items
        .iter()
        .map(|g| {
            let uuid = g.uuid.clone();
            async move { client.gear_stats(&uuid).await.ok() }
        })
        .collect::<Vec<_>>();
    let stats = futures::future::join_all(stats_futs).await;
    for (item, stat) in items.iter_mut().zip(stats.into_iter()) {
        if let Some(s) = stat {
            item.distance_meters = s.total_distance_meters;
            item.activities = s.total_activities;
        }
    }

    output.print_list(&items, "Gear");
    Ok(())
}

async fn stats(client: &GarminClient, output: &Output, uuid: &str) -> Result<()> {
    let s = client.gear_stats(uuid).await?;
    output.print(&s);
    Ok(())
}

async fn link(client: &GarminClient, output: &Output, uuid: &str, activity_id: u64) -> Result<()> {
    client.link_gear(uuid, activity_id).await?;
    output.print_value(&serde_json::json!({
        "gearUUID": uuid,
        "activityId": activity_id,
        "linked": true,
    }));
    Ok(())
}
