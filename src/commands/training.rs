use crate::client::GarminClient;
use crate::error::Result;
use crate::output::Output;
use crate::util::{fetch_date_range, parse_date, today};

pub async fn status(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    fetch_date_range(client, output, date, days, |d| {
        format!("/metrics-service/metrics/trainingstatus/aggregated/{d}")
    })
    .await
}

pub async fn readiness(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    fetch_date_range(client, output, date, days, |d| {
        format!("/metrics-service/metrics/trainingreadiness/{d}")
    })
    .await
}

pub async fn race_predictions(client: &GarminClient, output: &Output) -> Result<()> {
    let display_name = client.display_name().await?;
    let path = format!("/metrics-service/metrics/racepredictions/latest/{display_name}");
    let v: serde_json::Value = client.get_json(&path).await?;
    output.print_value(&v);
    Ok(())
}

pub async fn endurance_score(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let end_date = date.map(String::from).unwrap_or_else(today);
    let days = days.unwrap_or(1);
    if days == 1 {
        let path = format!("/metrics-service/metrics/endurancescore?calendarDate={end_date}");
        let v: serde_json::Value = client.get_json(&path).await?;
        output.print_value(&v);
    } else {
        let end = parse_date(&end_date)?;
        let start = end - chrono::Duration::days(days as i64 - 1);
        let start_str = start.format("%Y-%m-%d").to_string();
        let path = format!(
            "/metrics-service/metrics/endurancescore/stats?startDate={start_str}&endDate={end_date}&aggregation=daily"
        );
        let v: serde_json::Value = client.get_json(&path).await?;
        output.print_value(&v);
    }
    Ok(())
}

pub async fn hill_score(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let end_date = date.map(String::from).unwrap_or_else(today);
    let days = days.unwrap_or(1);
    if days == 1 {
        let path = format!("/metrics-service/metrics/hillscore?calendarDate={end_date}");
        let v: serde_json::Value = client.get_json(&path).await?;
        output.print_value(&v);
    } else {
        let end = parse_date(&end_date)?;
        let start = end - chrono::Duration::days(days as i64 - 1);
        let start_str = start.format("%Y-%m-%d").to_string();
        let path = format!(
            "/metrics-service/metrics/hillscore/stats?startDate={start_str}&endDate={end_date}&aggregation=daily"
        );
        let v: serde_json::Value = client.get_json(&path).await?;
        output.print_value(&v);
    }
    Ok(())
}

pub async fn fitness_age(client: &GarminClient, output: &Output, date: Option<&str>) -> Result<()> {
    let date_str = date.map(String::from).unwrap_or_else(today);
    let path = format!("/fitnessage-service/fitnessage/{date_str}");
    let v: serde_json::Value = client.get_json(&path).await?;
    output.print_value(&v);
    Ok(())
}

pub async fn lactate_threshold(client: &GarminClient, output: &Output) -> Result<()> {
    let v: serde_json::Value = client
        .get_json("/biometric-service/biometric/latestLactateThreshold")
        .await?;
    output.print_value(&v);
    Ok(())
}

pub async fn scores(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let end_date = date.map(String::from).unwrap_or_else(today);
    let end = parse_date(&end_date)?;
    let days = days.unwrap_or(7);
    let start = end - chrono::Duration::days(days as i64 - 1);
    let start_str = start.format("%Y-%m-%d").to_string();
    let path = format!("/metrics-service/metrics/maxmet/daily/{start_str}/{end_date}");
    let v: serde_json::Value = client.get_json(&path).await?;
    output.print_value(&v);
    Ok(())
}
