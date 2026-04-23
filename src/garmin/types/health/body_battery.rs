use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::stress::DailyStressResponse;

/// One sample from `bodyBatteryValuesArray`. Fixed positional layout:
/// `(timestamp_ms, status, level, version)`. `status` is typically
/// `MEASURED` (sensor reading), `MODELED` (algorithmic fill), or
/// `RESET` (baseline anchor — e.g. morning wake-up).
#[derive(Debug, Serialize, Deserialize)]
pub struct BodyBatteryEntry(
    pub i64,
    #[serde(default)] pub Option<String>,
    #[serde(default)] pub Option<i64>,
    #[serde(default)] pub Option<f64>,
);

impl BodyBatteryEntry {
    pub fn timestamp_ms(&self) -> i64 {
        self.0
    }
    pub fn status(&self) -> Option<&str> {
        self.1.as_deref()
    }
    pub fn level(&self) -> Option<i64> {
        self.2.filter(|&l| l >= 0)
    }
}

/// Summary of a day's body-battery time series.
#[derive(Debug, Clone, Copy)]
pub struct BodyBatterySummary {
    pub high: Option<i64>,
    pub low: Option<i64>,
    pub latest: Option<i64>,
    /// Morning baseline — the level at the first RESET sample of the day.
    pub reset_level: Option<i64>,
    pub reset_timestamp_ms: Option<i64>,
}

/// Body-battery view derived from DailyStressResponse.
#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct BodyBattery {
    pub calendar_date: String,
    pub body_battery_high: Option<i64>,
    pub body_battery_low: Option<i64>,
    pub body_battery_latest: Option<i64>,
    /// Level at the first `RESET` sample of the day — the sleep baseline
    /// anchor written by the watch as sleep modeling kicks in.
    pub body_battery_reset_level: Option<i64>,
    /// UTC epoch ms of the matching RESET sample.
    pub body_battery_reset_timestamp_ms: Option<i64>,
}

impl From<&DailyStressResponse> for BodyBattery {
    fn from(r: &DailyStressResponse) -> Self {
        let s = r.body_battery();
        Self {
            calendar_date: r.calendar_date.clone(),
            body_battery_high: s.high,
            body_battery_low: s.low,
            body_battery_latest: s.latest,
            body_battery_reset_level: s.reset_level,
            body_battery_reset_timestamp_ms: s.reset_timestamp_ms,
        }
    }
}

impl HumanReadable for BodyBattery {
    fn print_human(&self) {
        println!("{}", self.calendar_date.bold());
        if let Some(level) = self.body_battery_reset_level {
            println!("  {:<LABEL_WIDTH$}{level}", "Reset:");
        }
        if let (Some(lo), Some(hi)) = (self.body_battery_low, self.body_battery_high) {
            println!("  {:<LABEL_WIDTH$}{lo}\u{2013}{hi}", "Range:");
        }
        if let Some(v) = self.body_battery_latest {
            println!("  {:<LABEL_WIDTH$}{}", "Latest:", v.to_string().cyan());
        }
    }
}
