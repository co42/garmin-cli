use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct HeartRateDay {
    #[serde(default)]
    pub calendar_date: String,
    pub resting_heart_rate: Option<i64>,
    pub min_heart_rate: Option<i64>,
    pub max_heart_rate: Option<i64>,
    pub last_seven_days_avg_resting_heart_rate: Option<i64>,
}

impl HumanReadable for HeartRateDay {
    fn print_human(&self) {
        println!("{}", self.calendar_date.bold());
        if let Some(v) = self.resting_heart_rate {
            println!("  {:<LABEL_WIDTH$}{} bpm", "Resting:", v.to_string().cyan());
        }
        if let (Some(lo), Some(hi)) = (self.min_heart_rate, self.max_heart_rate) {
            println!("  {:<LABEL_WIDTH$}{lo}\u{2013}{hi} bpm", "Range:");
        }
        if let Some(v) = self.last_seven_days_avg_resting_heart_rate {
            println!("  {:<LABEL_WIDTH$}{v} bpm", "7-day avg:");
        }
    }
}
