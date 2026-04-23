use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Hydration {
    #[serde(default)]
    pub calendar_date: String,
    /// API: `valueInML` (all-caps ML) — also ambiguous ("value" of what?); renamed
    /// for clarity. Alias accepts the alternative `intakeInML` form.
    #[serde(rename(deserialize = "valueInML"), alias = "intakeInML")]
    pub intake_ml: Option<f64>,
    /// API: `goalInML` (all-caps ML).
    #[serde(rename(deserialize = "goalInML"), alias = "dailyGoalInML")]
    pub goal_ml: Option<f64>,
}

impl HumanReadable for Hydration {
    fn print_human(&self) {
        println!("{}", self.calendar_date.bold());
        match (self.intake_ml, self.goal_ml) {
            (Some(intake), Some(goal)) => {
                println!("  {:<LABEL_WIDTH$}{:.0} / {:.0} ml", "Intake:", intake, goal.round());
            }
            (Some(intake), None) => {
                println!("  {:<LABEL_WIDTH$}{:.0} ml", "Intake:", intake);
            }
            (None, Some(goal)) => {
                println!("  {:<LABEL_WIDTH$}{:.0} ml", "Goal:", goal.round());
            }
            (None, None) => {
                println!("  No data");
            }
        }
    }
}
