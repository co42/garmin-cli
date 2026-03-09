use crate::client::GarminClient;
use crate::error::Result;
use crate::output::Output;

fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

fn date_range(date: Option<&str>, days: Option<u32>) -> (String, u32) {
    let end = date.map(String::from).unwrap_or_else(today);
    (end, days.unwrap_or(1))
}

fn parse_date(s: &str) -> std::result::Result<chrono::NaiveDate, crate::error::Error> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map_err(|e| crate::error::Error::Api(format!("Invalid date: {e}")))
}

// Most health endpoints return rich nested JSON. We output it directly
// rather than trying to cherry-pick fields into typed structs.

fn print_value(v: &serde_json::Value) {
    println!("{}", serde_json::to_string_pretty(v).unwrap_or_default());
}

async fn fetch_health_multi(
    client: &GarminClient,
    _output: &Output,
    date: Option<&str>,
    days: Option<u32>,
    path_fn: impl Fn(&str) -> String,
) -> Result<()> {
    let (end_date, days) = date_range(date, days);
    let end = parse_date(&end_date)?;

    if days == 1 {
        let path = path_fn(&end_date);
        let v: serde_json::Value = client.get_json(&path).await?;
        print_value(&v);
    } else {
        let mut results = Vec::new();
        for i in (0..days).rev() {
            let d = end - chrono::Duration::days(i as i64);
            let date_str = d.format("%Y-%m-%d").to_string();
            let path = path_fn(&date_str);
            let v: serde_json::Value = client.get_json(&path).await?;
            results.push(v);
        }
        print_value(&serde_json::Value::Array(results));
    }
    Ok(())
}

async fn fetch_health_single(
    client: &GarminClient,
    _output: &Output,
    date: Option<&str>,
    path_fn: impl Fn(&str) -> String,
) -> Result<()> {
    let date_str = date.map(String::from).unwrap_or_else(today);
    let path = path_fn(&date_str);
    let v: serde_json::Value = client.get_json(&path).await?;
    print_value(&v);
    Ok(())
}

pub async fn sleep(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let display_name = client.display_name().await?;
    fetch_health_multi(client, output, date, days, |d| {
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
    fetch_health_multi(client, output, date, days, |d| {
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
    fetch_health_multi(client, output, date, days, |d| {
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
    fetch_health_single(client, output, date, |d| {
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
    fetch_health_multi(client, output, date, days, |d| {
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
    fetch_health_multi(client, output, date, days, |d| {
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
    fetch_health_multi(client, output, date, days, |d| {
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
    fetch_health_multi(client, output, date, days, |d| {
        format!("/usersummary-service/usersummary/hydration/daily/{d}")
    })
    .await
}

pub async fn spo2(client: &GarminClient, output: &Output, date: Option<&str>) -> Result<()> {
    fetch_health_single(client, output, date, |d| {
        format!("/wellness-service/wellness/dailySpo2/{d}")
    })
    .await
}

pub async fn respiration(client: &GarminClient, output: &Output, date: Option<&str>) -> Result<()> {
    fetch_health_single(client, output, date, |d| {
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
    fetch_health_multi(client, output, date, days, |d| {
        format!("/usersummary-service/stats/im/daily/{d}/{d}")
    })
    .await
}
