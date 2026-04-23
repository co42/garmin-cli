use super::helpers::{DateRangeArgs, fetch_range};
use super::output::Output;
use crate::error::{Error, Result};
use crate::garmin::{
    BiometricDataPoint, DailyReadiness, EnduranceScore, FitnessAge, GarminClient, HillScore, HrZoneBoundary,
    LactateThreshold, RacePredictions, TrainingScore, TrainingStatus, correct_lt_speed,
};
use chrono::NaiveDate;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum TrainingCommands {
    /// Training status
    Status {
        #[command(flatten)]
        range: DateRangeArgs,
    },
    /// Training readiness
    Readiness {
        #[command(flatten)]
        range: DateRangeArgs,
    },
    /// VO2max history
    #[command(alias = "scores")]
    Vo2max {
        #[command(flatten)]
        range: DateRangeArgs,
    },
    /// Race predictions (5K, 10K, half, marathon)
    RacePredictions {
        #[command(flatten)]
        range: DateRangeArgs,
    },
    /// Endurance score
    EnduranceScore {
        #[command(flatten)]
        range: DateRangeArgs,
    },
    /// Hill score
    HillScore {
        #[command(flatten)]
        range: DateRangeArgs,
    },
    /// Fitness age
    FitnessAge {
        #[command(flatten)]
        range: DateRangeArgs,
    },
    /// Lactate threshold (speed and HR)
    LactateThreshold {
        #[command(flatten)]
        range: DateRangeArgs,
    },
    /// Heart rate zones (from most recent running activity)
    #[command(alias = "zones")]
    HrZones,
}

pub async fn run(command: TrainingCommands, output: &Output) -> Result<()> {
    let client = GarminClient::new(super::helpers::require_auth()?)?;
    match command {
        TrainingCommands::Status { range } => {
            let (start, end) = range.resolve(1)?;
            status(&client, output, start, end).await
        }
        TrainingCommands::Readiness { range } => {
            let (start, end) = range.resolve(1)?;
            readiness(&client, output, start, end).await
        }
        TrainingCommands::Vo2max { range } => {
            let (start, end) = range.resolve(7)?;
            scores(&client, output, start, end).await
        }
        TrainingCommands::RacePredictions { range } => {
            let (start, end) = range.resolve(1)?;
            race_predictions(&client, output, start, end).await
        }
        TrainingCommands::EnduranceScore { range } => {
            let (start, end) = range.resolve(1)?;
            endurance_score(&client, output, start, end).await
        }
        TrainingCommands::HillScore { range } => {
            let (start, end) = range.resolve(1)?;
            hill_score(&client, output, start, end).await
        }
        TrainingCommands::FitnessAge { range } => {
            let (start, end) = range.resolve(1)?;
            fitness_age(&client, output, start, end).await
        }
        TrainingCommands::LactateThreshold { range } => {
            let (start, end) = range.resolve(7)?;
            lactate_threshold(&client, output, start, end).await
        }
        TrainingCommands::HrZones => zones(&client, output).await,
    }
}

async fn status(client: &GarminClient, output: &Output, start: NaiveDate, end: NaiveDate) -> Result<()> {
    let items: Vec<TrainingStatus> =
        fetch_range(start, end, |ds| async move { client.training_status(&ds).await }).await?;
    output.print_list(&items, "Training Status");
    Ok(())
}

async fn readiness(client: &GarminClient, output: &Output, start: NaiveDate, end: NaiveDate) -> Result<()> {
    let items: Vec<DailyReadiness> = fetch_range(start, end, |ds| async move {
        let entries = client.training_readiness(&ds).await?;
        Ok(DailyReadiness::from_entries(entries, &ds))
    })
    .await?;
    output.print_list(&items, "Training Readiness");
    Ok(())
}

async fn scores(client: &GarminClient, output: &Output, start: NaiveDate, end: NaiveDate) -> Result<()> {
    let raw = client.vo2max_daily(start, end).await?;
    let items: Vec<TrainingScore> = raw.into_iter().map(TrainingScore::from).collect();
    output.print_list(&items, "Training Scores (VO2max)");
    Ok(())
}

async fn race_predictions(client: &GarminClient, output: &Output, start: NaiveDate, end: NaiveDate) -> Result<()> {
    let raw = client.race_predictions(start, end).await?;
    let items: Vec<RacePredictions> = raw.into_iter().map(RacePredictions::from).collect();
    output.print_list(&items, "Race Predictions");
    Ok(())
}

async fn endurance_score(client: &GarminClient, output: &Output, start: NaiveDate, end: NaiveDate) -> Result<()> {
    let items: Vec<EnduranceScore> = fetch_range(start, end, |ds| async move {
        let raw = client.endurance_score(&ds).await?;
        Ok(EnduranceScore::from(raw))
    })
    .await?;
    output.print_list(&items, "Endurance Score");
    Ok(())
}

async fn hill_score(client: &GarminClient, output: &Output, start: NaiveDate, end: NaiveDate) -> Result<()> {
    let items: Vec<HillScore> = fetch_range(start, end, |ds| async move { client.hill_score(&ds).await }).await?;
    output.print_list(&items, "Hill Score");
    Ok(())
}

async fn fitness_age(client: &GarminClient, output: &Output, start: NaiveDate, end: NaiveDate) -> Result<()> {
    let items: Vec<FitnessAge> = fetch_range(start, end, |ds| async move {
        let raw = client.fitness_age(&ds).await?;
        Ok(FitnessAge::from_raw(raw, &ds))
    })
    .await?;
    output.print_list(&items, "Fitness Age");
    Ok(())
}

async fn lactate_threshold(client: &GarminClient, output: &Output, start: NaiveDate, end: NaiveDate) -> Result<()> {
    // Garmin caps range at 366 days; use a wider lookback so we can carry
    // forward the latest known value when the in-window response is empty.
    let lookback = end - chrono::Duration::days(365);
    let start_str = start.format("%Y-%m-%d").to_string();

    let (hr_points, speed_points) = tokio::try_join!(
        client.lactate_threshold_hr(lookback, end),
        client.lactate_threshold_speed(lookback, end),
    )?;

    // Key HR and speed change-points by their updatedDate.
    type LtRow = (Option<i64>, Option<f64>);
    let mut by_date: std::collections::BTreeMap<String, LtRow> = std::collections::BTreeMap::new();
    let row_date = |p: &BiometricDataPoint| -> Option<String> {
        p.updated_date
            .as_deref()
            .or(p.from_date.as_deref())
            .map(|s| s.chars().take(10).collect())
    };
    for p in &hr_points {
        if let (Some(d), Some(v)) = (row_date(p), p.value) {
            by_date.entry(d).or_default().0 = Some(v as i64);
        }
    }
    for p in &speed_points {
        if let (Some(d), Some(v)) = (row_date(p), p.value) {
            by_date.entry(d).or_default().1 = Some(correct_lt_speed(v));
        }
    }

    // Split into in-window and prior; fall back to most recent prior if window is empty.
    let (prior, in_window): (Vec<_>, Vec<_>) = by_date.into_iter().partition(|(d, _)| d.as_str() < start_str.as_str());
    let rows: Vec<(String, LtRow)> = if in_window.is_empty() {
        prior.into_iter().last().into_iter().collect()
    } else {
        in_window
    };

    let items: Vec<LactateThreshold> = rows
        .into_iter()
        .map(|(date, (hr, speed))| LactateThreshold {
            date,
            heart_rate: hr,
            speed_mps: speed,
        })
        .collect();

    output.print_list(&items, "Lactate Threshold");
    Ok(())
}

async fn zones(client: &GarminClient, output: &Output) -> Result<()> {
    // Find the most recent running activity.
    let activities = client.list_activities(1, 0, Some("running"), None, None).await?;
    let activity_id = activities
        .first()
        .map(|a| a.activity_id)
        .ok_or_else(|| Error::NotFound("no running activities".into()))?;

    let zones = client.activity_hr_zones(activity_id).await?;
    let raw: Vec<(i64, i64)> = zones
        .iter()
        .filter_map(|z| Some((z.zone_number, z.zone_low_boundary_bpm?)))
        .collect();

    // Build boundaries: each zone's max is the next zone's min - 1.
    let mut boundaries: Vec<HrZoneBoundary> = Vec::new();
    for (i, &(zone, min_bpm)) in raw.iter().enumerate() {
        let max_bpm = raw.get(i + 1).map(|&(_, next_min)| next_min - 1);
        boundaries.push(HrZoneBoundary { zone, min_bpm, max_bpm });
    }

    output.print_table(&boundaries, "HR Zones");
    Ok(())
}
