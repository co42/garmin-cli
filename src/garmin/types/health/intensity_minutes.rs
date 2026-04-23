use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct IntensityMinutes {
    #[serde(default)]
    pub calendar_date: String,
    #[serde(default)]
    pub moderate_value: i64,
    #[serde(default)]
    pub vigorous_value: i64,
    pub weekly_goal: Option<i64>,
}

impl IntensityMinutes {
    pub fn total(&self) -> i64 {
        self.moderate_value + self.vigorous_value
    }
}

impl HumanReadable for IntensityMinutes {
    fn print_human(&self) {
        println!("{}", self.calendar_date.bold());
        let goal_str = self.weekly_goal.map(|g| format!(" / {g}")).unwrap_or_default();
        println!(
            "  {:<LABEL_WIDTH$}{} min{}",
            "Total:",
            self.total().to_string().cyan(),
            goal_str
        );
        println!("  {:<LABEL_WIDTH$}{} min", "Moderate:", self.moderate_value);
        println!("  {:<LABEL_WIDTH$}{} min", "Vigorous:", self.vigorous_value);
    }
}
