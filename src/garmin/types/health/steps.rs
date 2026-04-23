use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// `/usersummary-service/stats/steps/daily/{d}/{d}` returns `[{...}]`.
/// We expose the single-entry shape here.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Steps {
    #[serde(default)]
    pub calendar_date: String,
    pub total_steps: Option<u64>,
    pub step_goal: Option<u64>,
    #[serde(rename(deserialize = "totalDistance"))]
    pub total_distance_meters: Option<f64>,
}

impl HumanReadable for Steps {
    fn print_human(&self) {
        println!("{}", self.calendar_date.bold());
        if let Some(s) = self.total_steps {
            let goal_str = self.step_goal.map(|g| format!(" / {g}")).unwrap_or_default();
            println!("  {:<LABEL_WIDTH$}{}{}", "Steps:", s.to_string().cyan(), goal_str);
        }
        if let Some(d) = self.total_distance_meters {
            println!("  {:<LABEL_WIDTH$}{:.2} km", "Distance:", d / 1000.0);
        }
    }
}
