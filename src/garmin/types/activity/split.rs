use crate::commands::output::HumanReadable;
use crate::garmin::types::helpers::{compute_pace, fmt_hms};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct SplitsResponse {
    /// API key is `lapDTOs`; strip the DTO suffix.
    #[serde(rename(deserialize = "lapDTOs"), default)]
    pub laps: Vec<ActivitySplit>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ActivitySplit {
    /// Not returned by API — filled in by the client after deserialization.
    #[serde(skip, default)]
    pub split: i64,
    #[serde(rename(deserialize = "distance"))]
    pub distance_meters: Option<f64>,
    #[serde(rename(deserialize = "duration"))]
    pub duration_seconds: Option<f64>,
    #[serde(rename(deserialize = "movingDuration"))]
    pub moving_duration_seconds: Option<f64>,
    #[serde(rename(deserialize = "averageHR"))]
    pub average_hr: Option<f64>,
    #[serde(rename(deserialize = "maxHR"))]
    pub max_hr: Option<f64>,
    pub average_power: Option<f64>,
    pub normalized_power: Option<f64>,
    pub average_run_cadence: Option<f64>,
    #[serde(rename(deserialize = "strideLength"))]
    pub stride_length_cm: Option<f64>,
    #[serde(rename(deserialize = "elevationGain"))]
    pub elevation_gain_meters: Option<f64>,
    #[serde(rename(deserialize = "elevationLoss"))]
    pub elevation_loss_meters: Option<f64>,
    #[serde(rename(deserialize = "groundContactTime"))]
    pub ground_contact_time_ms: Option<f64>,
    #[serde(rename(deserialize = "verticalOscillation"))]
    pub vertical_oscillation_cm: Option<f64>,
    #[serde(rename(deserialize = "verticalRatio"))]
    pub vertical_ratio_percent: Option<f64>,
    pub calories: Option<f64>,
}

impl ActivitySplit {
    pub fn pace(&self) -> Option<String> {
        self.duration_seconds
            .and_then(|d| compute_pace(self.distance_meters, d))
    }
}

impl HumanReadable for ActivitySplit {
    fn print_human(&self) {
        let dist = self
            .distance_meters
            .map(|d| format!("{:.0}m", d))
            .unwrap_or_else(|| "\u{2013}".into());
        let dur = self.duration_seconds.map(fmt_hms).unwrap_or_else(|| "\u{2013}".into());
        let pace = self.pace().unwrap_or_else(|| "\u{2013}".into());
        let hr = self
            .average_hr
            .map(|h| format!("{:.0} bpm", h))
            .unwrap_or_else(|| "\u{2013}".into());
        let elev = match (self.elevation_gain_meters, self.elevation_loss_meters) {
            (Some(g), Some(l)) => format!("+{:.0}/-{:.0}m", g, l),
            (Some(g), None) => format!("+{:.0}m", g),
            _ => String::new(),
        };
        let label = format!("#{}", self.split);
        println!(
            "  {:<6}{:>7}  {:>6}  {:>10}  {:>8}  {}",
            label.cyan(),
            dist,
            dur,
            pace,
            hr.dimmed(),
            elev.dimmed(),
        );
    }
}
