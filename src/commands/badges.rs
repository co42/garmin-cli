use super::output::Output;
use crate::error::Result;
use crate::garmin::GarminClient;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum BadgeCommands {
    /// List earned badges
    List,
}

pub async fn run(command: BadgeCommands, output: &Output) -> Result<()> {
    let client = GarminClient::new(super::helpers::require_auth()?)?;
    match command {
        BadgeCommands::List => list(&client, output).await,
    }
}

async fn list(client: &GarminClient, output: &Output) -> Result<()> {
    let badges = client.earned_badges().await?;
    output.print_list(&badges, "Badges");
    Ok(())
}
