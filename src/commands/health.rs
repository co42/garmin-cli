use super::helpers::{DateRangeArgs, fetch_range};
use super::output::Output;
use crate::error::Result;
use crate::garmin::{
    BodyBattery, GarminClient, HeartRateDay, HrvSummary, Hydration, IntensityMinutes, Respiration, SleepScore,
    SleepSummary, SpO2, Steps, StressSummary, WeightEntry,
};
use chrono::NaiveDate;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum HealthCommands {
    /// Sleep data
    Sleep {
        #[command(flatten)]
        range: DateRangeArgs,
    },
    /// Sleep score trends
    SleepScores {
        #[command(flatten)]
        range: DateRangeArgs,
    },
    /// Stress levels
    Stress {
        #[command(flatten)]
        range: DateRangeArgs,
    },
    /// Heart rate
    HeartRate {
        #[command(flatten)]
        range: DateRangeArgs,
    },
    /// Body battery
    BodyBattery {
        #[command(flatten)]
        range: DateRangeArgs,
    },
    /// Heart rate variability
    Hrv {
        #[command(flatten)]
        range: DateRangeArgs,
    },
    /// Step count
    Steps {
        #[command(flatten)]
        range: DateRangeArgs,
    },
    /// Weight
    Weight {
        #[command(flatten)]
        range: DateRangeArgs,
    },
    /// Hydration
    Hydration {
        #[command(flatten)]
        range: DateRangeArgs,
    },
    /// Blood oxygen (SpO2)
    Spo2 {
        #[command(flatten)]
        range: DateRangeArgs,
    },
    /// Respiration rate
    Respiration {
        #[command(flatten)]
        range: DateRangeArgs,
    },
    /// Intensity minutes
    IntensityMinutes {
        #[command(flatten)]
        range: DateRangeArgs,
    },
}

pub async fn run(command: HealthCommands, output: &Output) -> Result<()> {
    let client = GarminClient::new(super::helpers::require_auth()?)?;
    match command {
        HealthCommands::Sleep { range } => {
            let (start, end) = range.resolve(1)?;
            sleep(&client, output, start, end).await
        }
        HealthCommands::SleepScores { range } => {
            let (start, end) = range.resolve(7)?;
            sleep_scores(&client, output, start, end).await
        }
        HealthCommands::Stress { range } => {
            let (start, end) = range.resolve(1)?;
            stress(&client, output, start, end).await
        }
        HealthCommands::HeartRate { range } => {
            let (start, end) = range.resolve(1)?;
            heart_rate(&client, output, start, end).await
        }
        HealthCommands::BodyBattery { range } => {
            let (start, end) = range.resolve(1)?;
            body_battery(&client, output, start, end).await
        }
        HealthCommands::Hrv { range } => {
            let (start, end) = range.resolve(1)?;
            hrv(&client, output, start, end).await
        }
        HealthCommands::Steps { range } => {
            let (start, end) = range.resolve(1)?;
            steps(&client, output, start, end).await
        }
        HealthCommands::Weight { range } => {
            let (start, end) = range.resolve(1)?;
            weight(&client, output, start, end).await
        }
        HealthCommands::Hydration { range } => {
            let (start, end) = range.resolve(1)?;
            hydration(&client, output, start, end).await
        }
        HealthCommands::Spo2 { range } => {
            let (start, end) = range.resolve(1)?;
            spo2(&client, output, start, end).await
        }
        HealthCommands::Respiration { range } => {
            let (start, end) = range.resolve(1)?;
            respiration(&client, output, start, end).await
        }
        HealthCommands::IntensityMinutes { range } => {
            let (start, end) = range.resolve(1)?;
            intensity_minutes(&client, output, start, end).await
        }
    }
}

async fn sleep(client: &GarminClient, output: &Output, start: NaiveDate, end: NaiveDate) -> Result<()> {
    let items: Vec<SleepSummary> = fetch_range(start, end, |ds| async move { client.daily_sleep(&ds).await }).await?;
    output.print_list(&items, "Sleep");
    Ok(())
}

async fn sleep_scores(client: &GarminClient, output: &Output, start: NaiveDate, end: NaiveDate) -> Result<()> {
    let items: Vec<SleepScore> = client.sleep_scores(start, end).await?;
    output.print_list(&items, "Sleep Scores");
    Ok(())
}

async fn stress(client: &GarminClient, output: &Output, start: NaiveDate, end: NaiveDate) -> Result<()> {
    let items: Vec<StressSummary> = fetch_range(start, end, |ds| async move {
        let r = client.daily_stress(&ds).await?;
        Ok(StressSummary::from(&r))
    })
    .await?;
    output.print_list(&items, "Stress");
    Ok(())
}

async fn heart_rate(client: &GarminClient, output: &Output, start: NaiveDate, end: NaiveDate) -> Result<()> {
    let items: Vec<HeartRateDay> =
        fetch_range(start, end, |ds| async move { client.daily_heart_rate(&ds).await }).await?;
    output.print_list(&items, "Heart Rate");
    Ok(())
}

async fn body_battery(client: &GarminClient, output: &Output, start: NaiveDate, end: NaiveDate) -> Result<()> {
    let items: Vec<BodyBattery> = fetch_range(start, end, |ds| async move {
        let r = client.daily_stress(&ds).await?;
        Ok(BodyBattery::from(&r))
    })
    .await?;
    output.print_list(&items, "Body Battery");
    Ok(())
}

async fn hrv(client: &GarminClient, output: &Output, start: NaiveDate, end: NaiveDate) -> Result<()> {
    let items: Vec<HrvSummary> = fetch_range(start, end, |ds| async move { client.daily_hrv(&ds).await }).await?;
    output.print_list(&items, "HRV");
    Ok(())
}

async fn steps(client: &GarminClient, output: &Output, start: NaiveDate, end: NaiveDate) -> Result<()> {
    let grouped: Vec<Vec<Steps>> = fetch_range(start, end, |ds| async move { client.daily_steps(&ds).await }).await?;
    let items: Vec<Steps> = grouped.into_iter().flatten().collect();
    output.print_list(&items, "Steps");
    Ok(())
}

async fn weight(client: &GarminClient, output: &Output, start: NaiveDate, end: NaiveDate) -> Result<()> {
    let range = client.weight_range(start, end).await?;
    let items: Vec<WeightEntry> = range.date_weight_list;
    output.print_list(&items, "Weight");
    Ok(())
}

async fn hydration(client: &GarminClient, output: &Output, start: NaiveDate, end: NaiveDate) -> Result<()> {
    let items: Vec<Hydration> = fetch_range(start, end, |ds| async move { client.daily_hydration(&ds).await }).await?;
    output.print_list(&items, "Hydration");
    Ok(())
}

async fn spo2(client: &GarminClient, output: &Output, start: NaiveDate, end: NaiveDate) -> Result<()> {
    let items: Vec<SpO2> = fetch_range(start, end, |ds| async move { client.daily_spo2(&ds).await }).await?;
    output.print_list(&items, "SpO2");
    Ok(())
}

async fn respiration(client: &GarminClient, output: &Output, start: NaiveDate, end: NaiveDate) -> Result<()> {
    let items: Vec<Respiration> =
        fetch_range(start, end, |ds| async move { client.daily_respiration(&ds).await }).await?;
    output.print_list(&items, "Respiration");
    Ok(())
}

async fn intensity_minutes(client: &GarminClient, output: &Output, start: NaiveDate, end: NaiveDate) -> Result<()> {
    let grouped: Vec<Vec<IntensityMinutes>> =
        fetch_range(
            start,
            end,
            |ds| async move { client.daily_intensity_minutes(&ds).await },
        )
        .await?;
    let items: Vec<IntensityMinutes> = grouped.into_iter().flatten().collect();
    output.print_list(&items, "Intensity Minutes");
    Ok(())
}
