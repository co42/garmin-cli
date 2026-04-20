use crate::client::GarminClient;
use crate::error::Result;
use crate::output::{HumanReadable, LABEL_WIDTH, Output};
use colored::Colorize;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct PersonalRecord {
    pub record_type: String,
    pub sport: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatted_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pace_min_km: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
}

struct RecordType {
    key: String,
    sport: String,
    /// Midpoint distance in meters (for time-based PRs with a defined distance range).
    distance_m: Option<f64>,
}

fn build_type_map(v: &serde_json::Value) -> HashMap<i64, RecordType> {
    let mut map = HashMap::new();
    if let Some(arr) = v.as_array() {
        for entry in arr {
            if let Some(id) = entry["id"].as_i64() {
                let min = entry["minValue"].as_f64().unwrap_or(0.0);
                let max = entry["maxValue"].as_f64().unwrap_or(0.0);
                let distance_m = if min > 0.0 && max > 0.0 {
                    Some((min + max) / 2.0)
                } else {
                    None
                };
                map.insert(
                    id,
                    RecordType {
                        key: entry["key"].as_str().unwrap_or("").to_string(),
                        sport: entry["sport"].as_str().unwrap_or("").to_string(),
                        distance_m,
                    },
                );
            }
        }
    }
    map
}

fn label_from_key(key: &str) -> String {
    let s = key.strip_prefix("pr.label.").unwrap_or(key);
    s.split('.')
        .map(|part| match part {
            "1k" => "1K".into(),
            "5k" => "5K".into(),
            "10k" => "10K".into(),
            "40k" => "40K".into(),
            "1mile" => "1 Mile".into(),
            "100m" => "100m".into(),
            "100yd" => "100yd".into(),
            "400m" => "400m".into(),
            "500yd" => "500yd".into(),
            "750m" => "750m".into(),
            "1000m" => "1000m".into(),
            "1000yd" => "1000yd".into(),
            "1500m" => "1500m".into(),
            "1650yd" => "1650yd".into(),
            "poolswim" => "Pool Swim".into(),
            "elev" => "Elevation".into(),
            other => {
                let mut c = other.chars();
                match c.next() {
                    Some(first) => first.to_uppercase().to_string() + c.as_str(),
                    None => String::new(),
                }
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

/// Determine how to format the value based on the PR type key.
enum ValueKind {
    Time,
    Distance,
    Count,
}

fn value_kind(key: &str) -> ValueKind {
    if key.contains("steps") || key.contains("pushes") || key.contains("max.rep.weight") {
        ValueKind::Count
    } else if key.contains("farthest") || key.contains("longest") {
        ValueKind::Distance
    } else {
        ValueKind::Time
    }
}

fn format_time(secs: f64) -> String {
    let total_secs = secs.round() as u64;
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;
    if hours > 0 {
        format!("{hours}:{mins:02}:{secs:02}")
    } else {
        format!("{mins}:{secs:02}")
    }
}

fn format_value(key: &str, value: f64) -> String {
    match value_kind(key) {
        ValueKind::Time => format_time(value),
        ValueKind::Distance => {
            let km = value / 1000.0;
            if km >= 1.0 {
                format!("{km:.2} km")
            } else {
                format!("{:.0} m", value)
            }
        }
        ValueKind::Count => format!("{}", value as u64),
    }
}

fn pace_min_km(secs: f64, distance_m: f64) -> String {
    let pace_secs = secs / (distance_m / 1000.0);
    let mins = pace_secs as u64 / 60;
    let s = pace_secs as u64 % 60;
    format!("{mins}:{s:02} /km")
}

fn record_from_json(v: &serde_json::Value, types: &HashMap<i64, RecordType>) -> PersonalRecord {
    let type_id = v["typeId"].as_i64().unwrap_or(0);
    let rt = types.get(&type_id);
    let key = rt.map(|t| t.key.as_str()).unwrap_or("");

    let date = v["actStartDateTimeInGMTFormatted"]
        .as_str()
        .map(|s| s[..s.len().min(10)].to_string());
    let activity_id = v["activityId"].as_u64().filter(|&id| id != 0);

    let value = v["value"].as_f64();
    let formatted_value = value.map(|val| format_value(key, val));

    // Compute pace for time-based records with a known distance
    let pace = value.and_then(|val| {
        rt.and_then(|t| t.distance_m)
            .map(|dist| pace_min_km(val, dist))
    });

    PersonalRecord {
        record_type: label_from_key(key),
        sport: rt
            .map(|t| t.sport.to_lowercase())
            .unwrap_or_else(|| "unknown".into()),
        value,
        formatted_value,
        pace_min_km: pace,
        activity_id,
        activity_type: v["activityType"].as_str().map(Into::into),
        activity_name: v["activityName"].as_str().map(Into::into),
        date,
    }
}

impl HumanReadable for PersonalRecord {
    fn print_human(&self) {
        println!("{}", self.record_type.bold());
        let val = self.formatted_value.as_deref().unwrap_or("\u{2014}");
        println!("  {:<LABEL_WIDTH$}{}", "Value:", val.cyan());
        if let Some(ref pace) = self.pace_min_km {
            println!("  {:<LABEL_WIDTH$}{pace}", "Pace:");
        }
        if let Some(ref d) = self.date {
            println!("  {:<LABEL_WIDTH$}{d}", "Date:");
        }
        if let Some(ref n) = self.activity_name {
            let id = self
                .activity_id
                .map(|i| format!(" (#{i})"))
                .unwrap_or_default();
            println!("  {:<LABEL_WIDTH$}{n}{id}", "Activity:");
        }
    }
}

pub async fn list(client: &GarminClient, output: &Output) -> Result<()> {
    let display_name = client.display_name().await?;

    let records_path = format!("/personalrecord-service/personalrecord/prs/{display_name}");
    let types_path = format!("/personalrecord-service/personalrecordtype/prtypes/{display_name}");

    let (rv, tv) = tokio::try_join!(
        client.get_json::<serde_json::Value>(&records_path),
        client.get_json::<serde_json::Value>(&types_path),
    )?;

    let types = build_type_map(&tv);

    let records: Vec<PersonalRecord> = rv
        .as_array()
        .map(|arr| arr.iter().map(|v| record_from_json(v, &types)).collect())
        .unwrap_or_default();

    output.print_list(&records, "Personal Records");
    Ok(())
}
