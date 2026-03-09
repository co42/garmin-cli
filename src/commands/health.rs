use crate::client::GarminClient;
use crate::error::Result;
use crate::output::{HumanReadable, Output};
use colored::Colorize;
use serde::Serialize;

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

// Generic wrapper for single-date health data -- just output the raw JSON
// with a date label. Most health endpoints return rich nested data that
// doesn't benefit from cherry-picking into typed structs.

#[derive(Debug, Serialize)]
struct HealthData {
    date: String,
    #[serde(flatten)]
    data: serde_json::Value,
}

impl HumanReadable for HealthData {
    fn print_human(&self) {
        println!("{}", self.date.bold());
        // Pretty-print the nested JSON for human consumption
        if let Some(obj) = self.data.as_object() {
            for (k, v) in obj {
                if !v.is_null() {
                    println!("  {}: {}", k.dimmed(), format_value(v));
                }
            }
        }
        println!();
    }
}

fn format_value(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                if f == f.floor() {
                    format!("{}", f as i64)
                } else {
                    format!("{:.1}", f)
                }
            } else {
                n.to_string()
            }
        }
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "\u{2014}".into(),
        _ => serde_json::to_string(v).unwrap_or_default(),
    }
}

async fn fetch_health_multi(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
    path_fn: impl Fn(&str) -> String,
    title: &str,
) -> Result<()> {
    let (end_date, days) = date_range(date, days);
    let end = parse_date(&end_date)?;
    let mut results = Vec::new();

    for i in 0..days {
        let d = end - chrono::Duration::days(i as i64);
        let date_str = d.format("%Y-%m-%d").to_string();
        let path = path_fn(&date_str);
        let v: serde_json::Value = client.get_json(&path).await?;
        results.push(HealthData {
            date: date_str,
            data: v,
        });
    }

    results.reverse();
    if results.len() == 1 {
        output.print(&results[0]);
    } else {
        output.print_list(&results, title);
    }
    Ok(())
}

async fn fetch_health_single(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    path_fn: impl Fn(&str) -> String,
) -> Result<()> {
    let date_str = date.map(String::from).unwrap_or_else(today);
    let path = path_fn(&date_str);
    let v: serde_json::Value = client.get_json(&path).await?;
    let data = HealthData {
        date: date_str,
        data: v,
    };
    output.print(&data);
    Ok(())
}

pub async fn sleep(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let display_name = client.display_name().await?;
    fetch_health_multi(
        client,
        output,
        date,
        days,
        |d| format!("/wellness-service/wellness/dailySleepData/{display_name}?date={d}"),
        "Sleep",
    )
    .await
}

pub async fn stress(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    fetch_health_multi(
        client,
        output,
        date,
        days,
        |d| format!("/wellness-service/wellness/dailyStress/{d}"),
        "Stress",
    )
    .await
}

pub async fn heart_rate(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let display_name = client.display_name().await?;
    fetch_health_multi(
        client,
        output,
        date,
        days,
        |d| format!("/wellness-service/wellness/dailyHeartRate/{display_name}?date={d}"),
        "Heart Rate",
    )
    .await
}

pub async fn body_battery(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
) -> Result<()> {
    fetch_health_single(client, output, date, |d| {
        format!("/wellness-service/wellness/bodyBattery/dates/{d}?startDate={d}&endDate={d}")
    })
    .await
}

pub async fn hrv(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    fetch_health_multi(
        client,
        output,
        date,
        days,
        |d| format!("/hrv-service/hrv/{d}"),
        "HRV",
    )
    .await
}

pub async fn steps(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    fetch_health_multi(
        client,
        output,
        date,
        days,
        |d| format!("/usersummary-service/stats/steps/daily/{d}/{d}"),
        "Steps",
    )
    .await
}

pub async fn weight(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    fetch_health_multi(
        client,
        output,
        date,
        days,
        |d| format!("/weight-service/weight/dateRange?startDate={d}&endDate={d}"),
        "Weight",
    )
    .await
}

pub async fn hydration(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    fetch_health_multi(
        client,
        output,
        date,
        days,
        |d| format!("/usersummary-service/usersummary/hydration/daily/{d}"),
        "Hydration",
    )
    .await
}

pub async fn spo2(client: &GarminClient, output: &Output, date: Option<&str>) -> Result<()> {
    fetch_health_single(client, output, date, |d| {
        format!("/wellness-service/wellness/pulse-ox/daily/{d}/{d}")
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
    fetch_health_multi(
        client,
        output,
        date,
        days,
        |d| format!("/wellness-service/wellness/dailyIntensityMinutes?calendarDate={d}"),
        "Intensity Minutes",
    )
    .await
}
