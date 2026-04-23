use super::helpers::{compute_pace, fmt_hms};
use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::HashMap;

/// Entry from `/personalrecord-service/personalrecord/prs/{name}`.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PersonalRecordEntry {
    #[serde(default)]
    pub type_id: i64,
    pub value: Option<f64>,
    pub activity_id: Option<u64>,
    pub activity_type: Option<String>,
    pub activity_name: Option<String>,
    /// API: `actStartDateTimeInGMTFormatted` — excessively verbose; renamed for usability.
    #[serde(rename(deserialize = "actStartDateTimeInGMTFormatted"))]
    pub start_date_time: Option<String>,
}

/// Entry from `/personalrecord-service/personalrecordtype/prtypes/{name}`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PersonalRecordType {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub sport: String,
    #[serde(default)]
    pub min_value: Option<f64>,
    #[serde(default)]
    pub max_value: Option<f64>,
}

impl PersonalRecordType {
    /// Midpoint distance in meters (for time-based PRs with a defined distance range).
    pub fn distance_m(&self) -> Option<f64> {
        let min = self.min_value.unwrap_or(0.0);
        let max = self.max_value.unwrap_or(0.0);
        (min > 0.0 && max > 0.0).then_some((min + max) / 2.0)
    }
}

/// Domain type for human display — combines record + type metadata.
/// Built by the command layer since it needs both endpoints' data.
#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct PersonalRecord {
    pub record_type: String,
    pub sport: String,
    pub value: Option<f64>,
    pub formatted_value: Option<String>,
    pub pace_min_km: Option<String>,
    pub activity_id: Option<u64>,
    pub activity_type: Option<String>,
    pub activity_name: Option<String>,
    pub date: Option<String>,
}

impl PersonalRecord {
    pub fn from_entry(entry: &PersonalRecordEntry, types: &HashMap<i64, PersonalRecordType>) -> Self {
        let rt = types.get(&entry.type_id);
        let key = rt.map(|t| t.key.as_str()).unwrap_or("");

        let date = entry
            .start_date_time
            .as_deref()
            .map(|s| s[..s.len().min(10)].to_string());
        let activity_id = entry.activity_id.filter(|&id| id != 0);

        let value = entry.value;
        let formatted_value = value.map(|val| format_value(key, val));
        let pace = value.and_then(|val| compute_pace(rt.and_then(|t| t.distance_m()), val));

        Self {
            record_type: label_from_key(key),
            sport: rt.map(|t| t.sport.to_lowercase()).unwrap_or_else(|| "unknown".into()),
            value,
            formatted_value,
            pace_min_km: pace,
            activity_id,
            activity_type: entry.activity_type.clone(),
            activity_name: entry.activity_name.clone(),
            date,
        }
    }
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

fn format_value(key: &str, value: f64) -> String {
    match value_kind(key) {
        ValueKind::Time => fmt_hms(value),
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
            let id = self.activity_id.map(|i| format!(" (#{i})")).unwrap_or_default();
            println!("  {:<LABEL_WIDTH$}{n}{id}", "Activity:");
        }
    }
}
