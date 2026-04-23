use super::helpers::{deser_cm_to_m, deser_ms_to_s, deser_nullable_u64, unknown_key};
use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// The calendar-service month endpoint returns `{ "calendarItems": [...], ... }`.
/// TODO: some queries may return a bare array instead; unhandled for now.
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CalendarMonth {
    #[serde(default)]
    pub calendar_items: Vec<CalendarItem>,
}

impl CalendarMonth {
    pub fn into_items(self) -> Vec<CalendarItem> {
        self.calendar_items
    }
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CalendarItem {
    #[serde(default, deserialize_with = "deser_nullable_u64")]
    pub id: u64,
    #[serde(alias = "calendarItemType", default = "unknown_key")]
    pub item_type: String,
    pub workout_id: Option<u64>,
    pub workout_uuid: Option<String>,
    pub title: Option<String>,
    /// API returns both `date` and `startTimestampLocal` for activities; `date`
    /// is always the date-only form. TODO: some shapes only include
    /// `startTimestampLocal` — those will show no date.
    pub date: Option<String>,
    /// API sometimes returns a nested `activityTypeDTO.typeKey`; only the flat
    /// `activityType: "running"` form is captured here.
    /// TODO: revisit if activity-type display regresses for scheduled activities.
    pub activity_type: Option<String>,
    #[serde(rename(deserialize = "duration"), deserialize_with = "deser_ms_to_s", default)]
    pub duration_seconds: Option<f64>,
    #[serde(rename(deserialize = "distance"), deserialize_with = "deser_cm_to_m", default)]
    pub distance_meters: Option<f64>,
}

impl HumanReadable for CalendarItem {
    fn print_human(&self) {
        let date = self
            .date
            .as_deref()
            .map(|d| &d[..d.len().min(10)])
            .unwrap_or("(no date)");
        let title = self.title.as_deref().unwrap_or("\u{2014}");
        println!("{}  {}", date.bold(), title);
        let (kind, id_line) = if let Some(uuid) = &self.workout_uuid {
            ("coach", Some(uuid.clone()))
        } else if let Some(id) = self.workout_id {
            ("workout", Some(id.to_string()))
        } else if self.item_type == "activity" {
            ("activity", Some(self.id.to_string()))
        } else {
            (self.activity_type.as_deref().unwrap_or(&self.item_type), None)
        };
        println!("  {:<LABEL_WIDTH$}{}", "Type:", kind.cyan());
        if let Some(id) = id_line {
            println!("  {:<LABEL_WIDTH$}{}", "Ref:", id.dimmed());
        }
        if let Some(meters) = self.distance_meters {
            println!("  {:<LABEL_WIDTH$}{:.2} km", "Distance:", meters / 1000.0);
        }
        if let Some(secs) = self.duration_seconds {
            let mins = (secs / 60.0).round() as u32;
            println!("  {:<LABEL_WIDTH$}{mins} min", "Duration:");
        }
    }
}
