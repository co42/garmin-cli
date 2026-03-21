use crate::client::GarminClient;
use crate::error::Result;
use crate::output::{HumanReadable, Output};
use chrono::Datelike;
use colored::Colorize;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CalendarItem {
    pub id: u64,
    pub item_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance_meters: Option<f64>,
}

fn calendar_item_from_json(v: &serde_json::Value) -> CalendarItem {
    CalendarItem {
        id: v["id"].as_u64().unwrap_or(0),
        item_type: v["itemType"]
            .as_str()
            .or_else(|| v["calendarItemType"].as_str())
            .unwrap_or("unknown")
            .into(),
        title: v["title"]
            .as_str()
            .or_else(|| v["activityName"].as_str())
            .map(Into::into),
        date: v["date"]
            .as_str()
            .or_else(|| v["startTimestampLocal"].as_str())
            .map(Into::into),
        activity_type: v["activityType"]
            .as_str()
            .or_else(|| v["activityTypeDTO"]["typeKey"].as_str())
            .map(Into::into),
        duration_seconds: v["duration"].as_f64(),
        distance_meters: v["distance"].as_f64(),
    }
}

impl HumanReadable for CalendarItem {
    fn print_human(&self) {
        let title = self.title.as_deref().unwrap_or("\u{2014}");
        let date = self.date.as_deref().unwrap_or("");
        let kind = self.activity_type.as_deref().unwrap_or(&self.item_type);
        print!("{} {} [{}]", date.dimmed(), title.bold(), kind.cyan());
        if let Some(dist) = self.distance_meters {
            print!("  {:.2} km", dist / 1000.0);
        }
        if let Some(dur) = self.duration_seconds {
            let mins = (dur / 60.0).round() as u32;
            print!("  {mins} min");
        }
        println!();
    }
}

pub async fn month(
    client: &GarminClient,
    output: &Output,
    year: Option<u32>,
    month: Option<u32>,
) -> Result<()> {
    let now = chrono::Local::now();
    let y = year.unwrap_or(now.year() as u32);
    let m = month.unwrap_or(now.month());
    let path = format!("/calendar-service/year/{y}/month/{m}");
    let v: serde_json::Value = client.get_json(&path).await?;

    let items_arr = v["calendarItems"].as_array().or_else(|| v.as_array());
    let items: Vec<CalendarItem> = items_arr
        .map(|arr| arr.iter().map(calendar_item_from_json).collect())
        .unwrap_or_default();

    output.print_list(&items, &format!("Calendar {y}-{m:02}"));
    Ok(())
}
