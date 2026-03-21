use crate::client::GarminClient;
use crate::error::Result;
use crate::output::{HumanReadable, Output};
use colored::Colorize;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PersonalRecord {
    pub type_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
}

fn record_from_json(v: &serde_json::Value) -> PersonalRecord {
    let date = v["actStartDateTimeInGMTFormatted"]
        .as_str()
        .map(|s| s[..s.len().min(10)].to_string());
    PersonalRecord {
        type_id: v["typeId"].as_i64().unwrap_or(0),
        activity_type: v["activityType"].as_str().map(Into::into),
        activity_name: v["activityName"].as_str().map(Into::into),
        activity_id: v["activityId"].as_u64(),
        value: v["value"].as_f64(),
        date,
    }
}

fn format_record_value(type_id: i64, value: f64) -> String {
    // Time-based records: value is in seconds
    // Distance-based records: value is in meters
    // Heuristic: type_ids for pace/time records have values that make sense as seconds
    // If value > 100_000, likely meters; otherwise likely seconds
    match type_id {
        // Known time-based type_ids (1km, 1mi, 5km, 10km, half, marathon, etc.)
        _ if value < 100_000.0 => {
            let total_secs = value.round() as u64;
            let hours = total_secs / 3600;
            let mins = (total_secs % 3600) / 60;
            let secs = total_secs % 60;
            if hours > 0 {
                format!("{hours}:{mins:02}:{secs:02}")
            } else {
                format!("{mins}:{secs:02}")
            }
        }
        _ => format!("{:.0}m", value),
    }
}

impl HumanReadable for PersonalRecord {
    fn print_human(&self) {
        let name = self.activity_name.as_deref().unwrap_or("\u{2014}");
        let val = self
            .value
            .map(|v| format_record_value(self.type_id, v))
            .unwrap_or_else(|| "\u{2014}".into());
        let date = self.date.as_deref().unwrap_or("");
        println!("  {:>8}  ({}, {})", val.bold(), date.dimmed(), name,);
    }
}

pub async fn list(client: &GarminClient, output: &Output) -> Result<()> {
    let display_name = client.display_name().await?;
    let path = format!("/personalrecord-service/personalrecord/prs/{display_name}");
    let v: serde_json::Value = client.get_json(&path).await?;

    let records: Vec<PersonalRecord> = v
        .as_array()
        .map(|arr| arr.iter().map(record_from_json).collect())
        .unwrap_or_default();

    output.print_list(&records, "Personal Records");
    Ok(())
}
