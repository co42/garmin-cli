use std::io::Write;
use std::{fmt, fs, io};

use clap::{Subcommand, ValueEnum};

use super::helpers::DateRangeArgs;
use super::output::Output;
use crate::error::{Error, Result};
use crate::garmin::GarminClient;

#[derive(Subcommand)]
pub enum ActivityCommands {
    /// List recent activities
    List {
        /// Max activities to return
        #[arg(long, default_value = "20")]
        limit: u32,
        /// Start index for pagination
        #[arg(long, default_value = "0")]
        start: u32,
        /// Filter by activity type (e.g. running, trail_running, cycling)
        #[arg(long, short = 't')]
        r#type: Option<String>,
        #[command(flatten)]
        range: DateRangeArgs,
    },
    /// Get activity summary
    Get {
        /// Activity ID
        id: u64,
    },
    /// Get full activity details (metrics, polyline, time-series)
    Details {
        /// Activity ID
        id: u64,
    },
    /// Get per-km lap splits (pace, HR, elevation per lap)
    Splits {
        /// Activity ID
        id: u64,
    },
    /// Get HR time in zones for an activity
    HrZones {
        /// Activity ID
        id: u64,
    },
    /// Get weather conditions during an activity
    Weather {
        /// Activity ID
        id: u64,
    },
    /// Get raw laps for an activity
    Laps {
        /// Activity ID
        id: u64,
    },
    /// Get exercise sets (structured intervals)
    Exercises {
        /// Activity ID
        id: u64,
    },
    /// Get power time in zones
    PowerZones {
        /// Activity ID
        id: u64,
    },
    /// Download activity file
    Download {
        /// Activity ID
        id: u64,
        /// File format
        #[arg(long, default_value = "fit")]
        format: DownloadFormat,
        /// Output file path
        #[arg(long, short)]
        output: Option<String>,
    },
    /// Upload activity file
    Upload {
        /// Path to FIT/GPX/TCX file
        file: String,
    },
}

#[derive(Clone, ValueEnum)]
pub enum DownloadFormat {
    Fit,
    Gpx,
    Tcx,
}

impl fmt::Display for DownloadFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fit => write!(f, "fit"),
            Self::Gpx => write!(f, "gpx"),
            Self::Tcx => write!(f, "tcx"),
        }
    }
}

pub async fn run(command: ActivityCommands, output: &Output) -> Result<()> {
    let client = GarminClient::new(super::helpers::require_auth()?)?;
    match command {
        ActivityCommands::List {
            limit,
            start,
            r#type,
            range,
        } => list(&client, output, limit, start, r#type.as_deref(), range).await,
        ActivityCommands::Get { id } => get(&client, output, id).await,
        ActivityCommands::Details { id } => details(&client, output, id).await,
        ActivityCommands::Splits { id } => splits(&client, output, id).await,
        ActivityCommands::HrZones { id } => hr_zones(&client, output, id).await,
        ActivityCommands::Weather { id } => weather(&client, output, id).await,
        ActivityCommands::Laps { id } => laps(&client, output, id).await,
        ActivityCommands::Exercises { id } => exercises(&client, output, id).await,
        ActivityCommands::PowerZones { id } => power_zones(&client, output, id).await,
        ActivityCommands::Download {
            id,
            format,
            output: out,
        } => download(&client, output, id, &format.to_string(), out.as_deref()).await,
        ActivityCommands::Upload { file } => upload(&client, output, &file).await,
    }
}

async fn list(
    client: &GarminClient,
    output: &Output,
    limit: u32,
    start: u32,
    activity_type: Option<&str>,
    range: DateRangeArgs,
) -> Result<()> {
    let (from, to) = match range.resolve_optional()? {
        Some((s, e)) => (
            Some(s.format("%Y-%m-%d").to_string()),
            Some(e.format("%Y-%m-%d").to_string()),
        ),
        None => (None, None),
    };
    let activities = client
        .list_activities(limit, start, activity_type, from.as_deref(), to.as_deref())
        .await?;
    output.print_list(&activities, "Activities");
    Ok(())
}

async fn get(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    let activity = client
        .activity(id)
        .await?
        .ok_or_else(|| Error::NotFound(format!("activity {id}")))?;
    output.print(&activity);
    Ok(())
}

async fn details(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    let v = client.activity_details(id).await?;
    output.print_value(&v);
    Ok(())
}

async fn splits(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    let items = client.activity_splits(id).await?;
    output.print_table(&items, "Splits");
    Ok(())
}

async fn hr_zones(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    let zones = client.activity_hr_zones(id).await?;
    output.print_table(&zones, "HR Zones");
    Ok(())
}

async fn weather(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    let w = client.activity_weather(id).await?;
    output.print(&w);
    Ok(())
}

async fn laps(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    let items = client.activity_laps(id).await?;
    output.print_table(&items, "Laps");
    Ok(())
}

async fn exercises(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    let v = client.activity_exercises(id).await?;
    output.print_value(&v);
    Ok(())
}

async fn power_zones(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    let zones = client.activity_power_zones(id).await?;
    output.print_table(&zones, "Power Zones");
    Ok(())
}

async fn download(
    client: &GarminClient,
    output: &Output,
    id: u64,
    format: &str,
    output_path: Option<&str>,
) -> Result<()> {
    let bytes = client.download_activity(id, format).await?;
    let out = output_path
        .map(String::from)
        .unwrap_or_else(|| format!("activity_{id}.{format}"));

    if out == "-" {
        io::stdout().write_all(&bytes)?;
        return Ok(());
    }

    fs::write(&out, &bytes)?;
    if output.is_json() {
        output.print_value(&serde_json::json!({
            "activityId": id,
            "path": out,
            "bytes": bytes.len(),
        }));
    } else {
        output.success(&format!("Saved to {out} ({} bytes)", bytes.len()));
    }
    Ok(())
}

async fn upload(client: &GarminClient, output: &Output, file: &str) -> Result<()> {
    let bytes = fs::read(file)?;
    let filename = std::path::Path::new(file)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "upload.fit".into());

    let ext = std::path::Path::new(file)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("fit");

    let result = client.upload_activity(bytes, &filename, ext).await?;

    if output.is_json() {
        output.print_value(&result);
    } else {
        output.success(&format!("Uploaded {filename}"));
    }
    Ok(())
}
