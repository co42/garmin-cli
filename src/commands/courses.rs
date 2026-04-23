use super::output::Output;
use crate::error::Result;
use crate::garmin::GarminClient;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum CourseCommands {
    /// List saved courses
    List,
    /// Get course details
    Get {
        /// Course ID
        id: u64,
    },
}

pub async fn run(command: CourseCommands, output: &Output) -> Result<()> {
    let client = GarminClient::new(super::helpers::require_auth()?)?;
    match command {
        CourseCommands::List => list(&client, output).await,
        CourseCommands::Get { id } => get(&client, output, id).await,
    }
}

async fn list(client: &GarminClient, output: &Output) -> Result<()> {
    let courses = client.list_courses().await?;
    output.print_list(&courses, "Courses");
    Ok(())
}

async fn get(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    let course = client.course(id).await?;
    output.print(&course);
    Ok(())
}
