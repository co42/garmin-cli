use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use crate::garmin::types::helpers::unknown_key;
use crate::garmin::types::workout::{SportTypeRef, WorkoutSegment, print_steps};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CoachWorkout {
    #[serde(default = "unknown_key")]
    pub workout_uuid: String,
    pub workout_name: Option<String>,
    pub description: Option<String>,
    pub workout_phrase: Option<String>,
    pub training_effect_label: Option<String>,
    pub priority_type: Option<String>,
    pub estimated_training_effect: Option<f64>,
    pub estimated_anaerobic_training_effect: Option<f64>,
    #[serde(rename(deserialize = "estimatedDurationInSecs"))]
    pub estimated_duration_seconds: Option<f64>,
    #[serde(rename(deserialize = "estimatedDistanceInMeters"))]
    pub estimated_distance_meters: Option<f64>,
    pub sport_type: Option<SportTypeRef>,
    /// Populated on both list and detail endpoints (unlike regular workouts,
    /// where only the detail endpoint returns segments). Empty for rest days.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub workout_segments: Vec<WorkoutSegment>,
    /// Present on list items — a workout belongs to a training plan.
    pub training_plan_id: Option<u64>,
}

fn humanize_phrase(phrase: &str) -> &str {
    match phrase {
        "ANAEROBIC_SPEED" => "anaerobic speed",
        "BASE" => "base",
        "LONG_WORKOUT" => "long run",
        "RUNNING_HISTORY_SHORTENED_BASE" => "shortened base",
        "FORCED_REST" | "EASY_WEEK_LOAD_REST" => "rest",
        "UNKNOWN" => "other",
        s if s.starts_with("STRENGTH_") => "strength",
        _ => phrase,
    }
}

fn format_te(aero: Option<f64>, anaero: Option<f64>) -> String {
    match (aero.filter(|&v| v > 0.0), anaero.filter(|&v| v > 0.0)) {
        (Some(a), Some(an)) => format!("aero {a:.1} | anaero {an:.1}"),
        (Some(a), None) => format!("aero {a:.1}"),
        (None, Some(an)) => format!("anaero {an:.1}"),
        (None, None) => String::new(),
    }
}

impl HumanReadable for CoachWorkout {
    fn print_human(&self) {
        let name = self.workout_name.as_deref().unwrap_or("Rest");
        let priority = self.priority_type.as_deref().unwrap_or("");
        let alt = if priority == "REQUIRED" { "" } else { " (alt)" };
        println!("{}{}", name.bold(), alt.dimmed());
        println!("  {:<LABEL_WIDTH$}{}", "UUID:", self.workout_uuid.dimmed());
        if let Some(phrase) = self.workout_phrase.as_deref().map(humanize_phrase)
            && !phrase.is_empty()
        {
            println!("  {:<LABEL_WIDTH$}{}", "Type:", phrase.cyan());
        }
        let te = format_te(self.estimated_training_effect, self.estimated_anaerobic_training_effect);
        if !te.is_empty() {
            println!("  {:<LABEL_WIDTH$}{te}", "TE:");
        }
        if let Some(ref desc) = self.description {
            println!("  {:<LABEL_WIDTH$}{desc}", "Target:");
        }
        if let Some(dur) = self.estimated_duration_seconds {
            let mins = (dur / 60.0).round() as u32;
            println!("  {:<LABEL_WIDTH$}{mins} min", "Duration:");
        }
        if let Some(dist) = self.estimated_distance_meters {
            println!("  {:<LABEL_WIDTH$}{:.1} km", "Distance:", dist / 1000.0);
        }
        for seg in &self.workout_segments {
            print_steps(&seg.workout_steps, 1);
        }
    }
}
