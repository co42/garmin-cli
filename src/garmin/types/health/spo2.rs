use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct SpO2 {
    #[serde(default)]
    pub calendar_date: String,
    /// API: `averageSpO2` — unusual `SpO2` casing that `rename_all` can't produce.
    #[serde(rename(deserialize = "averageSpO2"))]
    pub average_spo2: Option<f64>,
    #[serde(rename(deserialize = "lowestSpO2"))]
    pub lowest_spo2: Option<f64>,
}

impl HumanReadable for SpO2 {
    fn print_human(&self) {
        println!("{}", self.calendar_date.bold());
        if let Some(a) = self.average_spo2 {
            println!("  {:<LABEL_WIDTH$}{}%", "Average:", format!("{a:.0}").cyan());
        }
        if let Some(l) = self.lowest_spo2 {
            println!("  {:<LABEL_WIDTH$}{l:.0}%", "Lowest:");
        }
    }
}
