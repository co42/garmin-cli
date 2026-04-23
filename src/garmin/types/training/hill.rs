use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct HillScore {
    #[serde(default)]
    pub calendar_date: String,
    pub overall_score: Option<i64>,
    pub strength_score: Option<i64>,
    pub endurance_score: Option<i64>,
    pub vo2_max: Option<f64>,
}

impl HumanReadable for HillScore {
    fn print_human(&self) {
        println!("{}", self.calendar_date.bold());
        if let Some(v) = self.overall_score {
            println!("  {:<LABEL_WIDTH$}{}", "Overall:", v.to_string().cyan());
        }
        if let Some(v) = self.strength_score {
            println!("  {:<LABEL_WIDTH$}{v}", "Strength:");
        }
        if let Some(v) = self.endurance_score {
            println!("  {:<LABEL_WIDTH$}{v}", "Endurance:");
        }
        if let Some(v) = self.vo2_max {
            println!("  {:<LABEL_WIDTH$}{v:.1}", "VO2max:");
        }
    }
}
