use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

// HR and speed come from two separate endpoints (`lactateThresholdHeartRate`
// and `lactateThresholdSpeed`); the command merges them by `updatedDate`.

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct BiometricDataPoint {
    pub updated_date: Option<String>,
    #[serde(alias = "from")]
    pub from_date: Option<String>,
    pub value: Option<f64>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct LactateThreshold {
    pub date: String,
    pub heart_rate: Option<i64>,
    pub speed_meters_per_second: Option<f64>,
    pub pace: Option<String>,
}

impl HumanReadable for LactateThreshold {
    fn print_human(&self) {
        println!("{}", self.date.bold());
        let hr = self
            .heart_rate
            .map(|v| format!("{v} bpm"))
            .unwrap_or_else(|| "\u{2013}".into());
        let pace = self.pace.as_deref().unwrap_or("\u{2013}");
        println!("  {:<LABEL_WIDTH$}{}", "Heart rate:", hr.cyan());
        println!("  {:<LABEL_WIDTH$}{}", "Pace:", pace.cyan());
    }
}
