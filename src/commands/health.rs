use crate::client::GarminClient;
use crate::error::Result;
use crate::output::Output;
use crate::util::{fetch_date_range, parse_date, today};

async fn fetch_single(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    path_fn: impl Fn(&str) -> String,
) -> Result<()> {
    let date_str = date.map(String::from).unwrap_or_else(today);
    let path = path_fn(&date_str);
    let v: serde_json::Value = client.get_json(&path).await?;
    output.print_value(&v);
    Ok(())
}

pub async fn sleep(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let display_name = client.display_name().await?;
    fetch_date_range(client, output, date, days, |d| {
        format!("/wellness-service/wellness/dailySleepData/{display_name}?date={d}")
    })
    .await
}

pub async fn stress(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    fetch_date_range(client, output, date, days, |d| {
        format!("/wellness-service/wellness/dailyStress/{d}")
    })
    .await
}

pub async fn heart_rate(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let display_name = client.display_name().await?;
    fetch_date_range(client, output, date, days, |d| {
        format!("/wellness-service/wellness/dailyHeartRate/{display_name}?date={d}")
    })
    .await
}

pub async fn body_battery(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
) -> Result<()> {
    // Body battery data is embedded in the stress endpoint response
    fetch_single(client, output, date, |d| {
        format!("/wellness-service/wellness/dailyStress/{d}")
    })
    .await
}

pub async fn hrv(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    fetch_date_range(client, output, date, days, |d| {
        format!("/hrv-service/hrv/{d}")
    })
    .await
}

pub async fn steps(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    fetch_date_range(client, output, date, days, |d| {
        format!("/usersummary-service/stats/steps/daily/{d}/{d}")
    })
    .await
}

pub async fn weight(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    fetch_date_range(client, output, date, days, |d| {
        format!("/weight-service/weight/dateRange?startDate={d}&endDate={d}")
    })
    .await
}

pub async fn hydration(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    fetch_date_range(client, output, date, days, |d| {
        format!("/usersummary-service/usersummary/hydration/daily/{d}")
    })
    .await
}

pub async fn spo2(client: &GarminClient, output: &Output, date: Option<&str>) -> Result<()> {
    fetch_single(client, output, date, |d| {
        format!("/wellness-service/wellness/dailySpo2/{d}")
    })
    .await
}

pub async fn respiration(client: &GarminClient, output: &Output, date: Option<&str>) -> Result<()> {
    fetch_single(client, output, date, |d| {
        format!("/wellness-service/wellness/daily/respiration/{d}")
    })
    .await
}

pub async fn intensity_minutes(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    fetch_date_range(client, output, date, days, |d| {
        format!("/usersummary-service/stats/im/daily/{d}/{d}")
    })
    .await
}

pub async fn sleep_scores(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let end_date = date.map(String::from).unwrap_or_else(today);
    let days = days.unwrap_or(7);
    let end = parse_date(&end_date)?;
    let start = end - chrono::Duration::days(days as i64 - 1);
    let start_str = start.format("%Y-%m-%d").to_string();
    let path = format!("/wellness-service/stats/daily/sleep/score/{start_str}/{end_date}");
    let v: serde_json::Value = client.get_json(&path).await?;
    output.print_value(&v);
    Ok(())
}
