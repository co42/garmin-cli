use crate::client::GarminClient;
use crate::error::Result;
use crate::output::{HumanReadable, Output};
use colored::Colorize;
use serde::Serialize;

fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

fn parse_date(s: &str) -> std::result::Result<chrono::NaiveDate, crate::error::Error> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map_err(|e| crate::error::Error::Api(format!("Invalid date: {e}")))
}

#[derive(Debug, Serialize)]
struct TrainingData {
    date: String,
    #[serde(flatten)]
    data: serde_json::Value,
}

impl HumanReadable for TrainingData {
    fn print_human(&self) {
        println!("{}", self.date.bold());
        if let Some(obj) = self.data.as_object() {
            for (k, v) in obj {
                if !v.is_null() && !v.is_object() && !v.is_array() {
                    println!("  {}: {}", k.dimmed(), v);
                }
            }
        }
        println!();
    }
}

async fn fetch_multi(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
    path_fn: impl Fn(&str) -> String,
    title: &str,
) -> Result<()> {
    let end_date = date.map(String::from).unwrap_or_else(today);
    let days = days.unwrap_or(1);
    let end = parse_date(&end_date)?;
    let mut results = Vec::new();

    for i in 0..days {
        let d = end - chrono::Duration::days(i as i64);
        let date_str = d.format("%Y-%m-%d").to_string();
        let path = path_fn(&date_str);
        let v: serde_json::Value = client.get_json(&path).await?;
        results.push(TrainingData {
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

pub async fn status(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    fetch_multi(
        client,
        output,
        date,
        days,
        |d| format!("/training-status-service/trainingStatus/aggregated/{d}"),
        "Training Status",
    )
    .await
}

pub async fn readiness(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    fetch_multi(
        client,
        output,
        date,
        days,
        |d| format!("/training-readiness-service/trainingReadiness/{d}"),
        "Training Readiness",
    )
    .await
}

pub async fn scores(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    fetch_multi(
        client,
        output,
        date,
        days,
        |d| format!("/metrics-service/metrics/maxmet/daily/{d}/{d}"),
        "Training Scores",
    )
    .await
}
