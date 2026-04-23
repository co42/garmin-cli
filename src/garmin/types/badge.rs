use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Badge {
    pub badge_id: i64,
    pub badge_name: String,
    pub badge_key: String,
    pub badge_earned_date: Option<String>,
    pub badge_earned_number: Option<i64>,
    pub badge_points: Option<i64>,
    pub badge_progress_value: Option<f64>,
    pub badge_target_value: Option<f64>,
    pub badge_category_id: Option<i64>,
    pub badge_difficulty_id: Option<i64>,
}

impl HumanReadable for Badge {
    fn print_human(&self) {
        let count_str = self
            .badge_earned_number
            .filter(|&c| c > 1)
            .map(|c| format!(" x{c}"))
            .unwrap_or_default();
        println!("{}{count_str}", self.badge_name.bold());
        if let Some(ref d) = self.badge_earned_date {
            let short = &d[..d.len().min(10)];
            println!("  {:<LABEL_WIDTH$}{short}", "Earned:");
        }
        if let Some(pts) = self.badge_points {
            println!("  {:<LABEL_WIDTH$}{pts}", "Points:");
        }
        if let (Some(prog), Some(target)) = (self.badge_progress_value, self.badge_target_value)
            && target > 0.0
        {
            println!("  {:<LABEL_WIDTH$}{:.0} / {:.0}", "Progress:", prog, target);
        }
    }
}
