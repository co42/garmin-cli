use crate::client::GarminClient;
use crate::error::Result;
use crate::output::{HumanReadable, Output};
use chrono::{Datelike, NaiveDate};
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

/// Fetch calendar items for a single month from the Garmin API.
async fn fetch_month(client: &GarminClient, year: u32, month: u32) -> Result<Vec<CalendarItem>> {
    // Garmin calendar API uses 0-indexed months (0=Jan, 11=Dec)
    let api_month = month - 1;
    let path = format!("/calendar-service/year/{year}/month/{api_month}");
    let v: serde_json::Value = client.get_json(&path).await?;

    let items_arr = v["calendarItems"].as_array().or_else(|| v.as_array());
    Ok(items_arr
        .map(|arr| arr.iter().map(calendar_item_from_json).collect())
        .unwrap_or_default())
}

/// Advance (year, month) by one month.
fn next_month(year: u32, month: u32) -> (u32, u32) {
    if month >= 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    }
}

pub async fn list(
    client: &GarminClient,
    output: &Output,
    year: Option<u32>,
    month: Option<u32>,
    weeks: Option<u32>,
) -> Result<()> {
    let now = chrono::Local::now();
    let y = year.unwrap_or(now.year() as u32);
    let m = month.unwrap_or(now.month());

    if let Some(w) = weeks {
        // Fetch enough months to cover the requested weeks
        let start = NaiveDate::from_ymd_opt(y as i32, m, now.day()).unwrap_or(now.date_naive());
        let end = start + chrono::Duration::weeks(w as i64);

        let mut all_items = Vec::new();
        let mut cur_y = y;
        let mut cur_m = m;

        loop {
            all_items.extend(fetch_month(client, cur_y, cur_m).await?);
            let last_day =
                NaiveDate::from_ymd_opt(cur_y as i32, cur_m, 28).unwrap_or(now.date_naive());
            if last_day >= end {
                break;
            }
            (cur_y, cur_m) = next_month(cur_y, cur_m);
        }

        // Filter to date range and deduplicate by id
        let start_str = start.format("%Y-%m-%d").to_string();
        let end_str = end.format("%Y-%m-%d").to_string();
        let mut seen = std::collections::HashSet::new();
        let items: Vec<CalendarItem> = all_items
            .into_iter()
            .filter(|item| {
                if !seen.insert(item.id) {
                    return false;
                }
                item.date
                    .as_deref()
                    .is_some_and(|d| d >= start_str.as_str() && d <= end_str.as_str())
            })
            .collect();

        output.print_list(&items, &format!("Calendar {start_str} to {end_str}"));
    } else {
        let items = fetch_month(client, y, m).await?;
        output.print_list(&items, &format!("Calendar {y}-{m:02}"));
    }

    Ok(())
}

pub async fn delete(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    let path = format!("/workout-service/schedule/{id}");
    client.delete(&path).await?;
    output.print_value(&serde_json::json!({
        "calendarEntryId": id,
        "deleted": true,
    }));
    Ok(())
}
