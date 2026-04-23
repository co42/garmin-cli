use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct DailySummary {
    /// Command layer may override after construction for multi-day fetches.
    #[serde(default)]
    pub calendar_date: String,
    pub total_steps: Option<u64>,
    pub total_distance_meters: Option<f64>,
    pub active_kilocalories: Option<f64>,
    pub total_kilocalories: Option<f64>,
    pub resting_heart_rate: Option<u32>,
    pub max_heart_rate: Option<u32>,
    pub average_stress_level: Option<f64>,
    pub max_stress_level: Option<u32>,
    pub body_battery_highest_value: Option<u32>,
    pub body_battery_lowest_value: Option<u32>,
    pub sleeping_seconds: Option<u64>,
    pub floors_ascended: Option<f64>,
    pub floors_descended: Option<f64>,
    pub moderate_intensity_minutes: Option<u64>,
    pub vigorous_intensity_minutes: Option<u64>,
}

impl DailySummary {
    pub fn intensity_minutes(&self) -> Option<u64> {
        let m = self.moderate_intensity_minutes.unwrap_or(0);
        let v = self.vigorous_intensity_minutes.unwrap_or(0);
        let total = m + v;
        (total > 0).then_some(total)
    }
}

impl HumanReadable for DailySummary {
    fn print_human(&self) {
        println!("{}", self.calendar_date.bold());
        if let Some(v) = self.total_steps {
            println!("  {:<LABEL_WIDTH$}{}", "Steps:", v.to_string().cyan());
        }
        if let Some(v) = self.total_distance_meters {
            println!("  {:<LABEL_WIDTH$}{:.1} km", "Distance:", v / 1000.0);
        }
        if let Some(v) = self.active_kilocalories {
            println!("  {:<LABEL_WIDTH$}{:.0}", "Active cal:", v);
        }
        if let Some(v) = self.total_kilocalories {
            println!("  {:<LABEL_WIDTH$}{:.0}", "Total cal:", v);
        }
        if let Some(v) = self.resting_heart_rate {
            println!("  {:<LABEL_WIDTH$}{} bpm", "Resting HR:", v);
        }
        if let Some(v) = self.average_stress_level {
            let max_str = self.max_stress_level.map(|m| format!("  Max: {m}")).unwrap_or_default();
            println!("  {:<LABEL_WIDTH$}{:.0}{}", "Stress:", v, max_str);
        }
        if let (Some(lo), Some(hi)) = (self.body_battery_lowest_value, self.body_battery_highest_value) {
            println!("  {:<LABEL_WIDTH$}{lo}\u{2013}{hi}", "Body battery:");
        }
        if let Some(v) = self.sleeping_seconds {
            let h = v / 3600;
            let m = (v % 3600) / 60;
            println!("  {:<LABEL_WIDTH$}{h}h {m:02}m", "Sleep:");
        }
        if let Some(v) = self.floors_ascended {
            println!("  {:<LABEL_WIDTH$}{v:.0}", "Floors up:");
        }
        if let Some(v) = self.intensity_minutes() {
            println!("  {:<LABEL_WIDTH$}{v}", "Intensity min:");
        }
    }
}
