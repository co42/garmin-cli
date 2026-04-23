use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Respiration {
    #[serde(default)]
    pub calendar_date: String,
    pub avg_waking_respiration_value: Option<f64>,
    pub avg_sleep_respiration_value: Option<f64>,
    pub highest_respiration_value: Option<f64>,
    pub lowest_respiration_value: Option<f64>,
}

impl HumanReadable for Respiration {
    fn print_human(&self) {
        println!("{}", self.calendar_date.bold());
        if let Some(w) = self.avg_waking_respiration_value {
            println!("  {:<LABEL_WIDTH$}{w:.1} br/min", "Waking:");
        }
        if let Some(s) = self.avg_sleep_respiration_value {
            println!("  {:<LABEL_WIDTH$}{s:.1} br/min", "Sleeping:");
        }
        if let (Some(lo), Some(hi)) = (self.lowest_respiration_value, self.highest_respiration_value) {
            println!("  {:<LABEL_WIDTH$}{lo:.1}\u{2013}{hi:.1} br/min", "Range:");
        }
    }
}
