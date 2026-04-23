use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::body_battery::{BodyBatteryEntry, BodyBatterySummary};

/// The `/wellness-service/wellness/dailyStress/{date}` endpoint returns a rich
/// object that the CLI uses for both stress and body battery views.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct DailyStressResponse {
    #[serde(default)]
    pub calendar_date: String,
    pub avg_stress_level: Option<i64>,
    pub max_stress_level: Option<i64>,
    /// Per-sample time series. Position order is fixed by Garmin's
    /// `bodyBatteryValueDescriptorsDTOList` — see `BodyBatteryEntry`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub body_battery_values_array: Vec<BodyBatteryEntry>,
}

impl DailyStressResponse {
    /// Walk the time series once, collecting min/max/latest level plus the
    /// first RESET anchor if any.
    pub fn body_battery(&self) -> BodyBatterySummary {
        let mut high: Option<i64> = None;
        let mut low: Option<i64> = None;
        let mut latest: Option<i64> = None;
        let mut reset_level: Option<i64> = None;
        let mut reset_ts: Option<i64> = None;
        for entry in &self.body_battery_values_array {
            if let Some(level) = entry.level() {
                high = Some(high.map_or(level, |h| h.max(level)));
                low = Some(low.map_or(level, |l| l.min(level)));
                latest = Some(level);
                if reset_level.is_none() && entry.status() == Some("RESET") {
                    reset_level = Some(level);
                    reset_ts = Some(entry.timestamp_ms());
                }
            }
        }
        BodyBatterySummary {
            high,
            low,
            latest,
            reset_level,
            reset_timestamp_ms: reset_ts,
        }
    }
}

/// Stress-only view derived from DailyStressResponse for human-readable display.
#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct StressSummary {
    pub calendar_date: String,
    pub avg_stress_level: Option<i64>,
    pub max_stress_level: Option<i64>,
}

impl From<&DailyStressResponse> for StressSummary {
    fn from(r: &DailyStressResponse) -> Self {
        Self {
            calendar_date: r.calendar_date.clone(),
            avg_stress_level: r.avg_stress_level,
            max_stress_level: r.max_stress_level,
        }
    }
}

impl HumanReadable for StressSummary {
    fn print_human(&self) {
        println!("{}", self.calendar_date.bold());
        if let Some(avg) = self.avg_stress_level {
            println!("  {:<LABEL_WIDTH$}{}", "Average:", avg.to_string().cyan());
        }
        if let Some(max) = self.max_stress_level {
            println!("  {:<LABEL_WIDTH$}{max}", "Max:");
        }
    }
}
