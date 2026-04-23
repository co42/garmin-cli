use super::helpers::{pace_from_speed, untitled};
use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

// The list and detail endpoints return meaningfully different shapes
// - List (`/workout-service/workouts`) is flatter
// - Detail (`/workout-service/workout/{id}`) adds `workoutSegments`

/// List-endpoint shape — no step structure.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct WorkoutSummary {
    #[serde(default)]
    pub workout_id: u64,
    #[serde(default = "untitled")]
    pub workout_name: String,
    pub sport_type: Option<SportTypeRef>,
    pub description: Option<String>,
    pub estimated_duration_in_secs: Option<f64>,
    pub estimated_distance_in_meters: Option<f64>,
    pub created_date: Option<String>,
    /// List endpoint uses `updateDate`; detail endpoint uses `updatedDate`.
    /// Rename to the common snake_case `updated_date` so JSON output is consistent
    /// across `workouts list` and `workouts get`.
    #[serde(rename(deserialize = "updateDate"))]
    pub updated_date: Option<String>,
}

impl HumanReadable for WorkoutSummary {
    fn print_human(&self) {
        print_header(
            self.workout_id,
            &self.workout_name,
            self.sport_type.as_ref(),
            self.created_date.as_deref(),
        );
        print_body(
            self.description.as_deref(),
            self.estimated_duration_in_secs,
            self.estimated_distance_in_meters,
        );
    }
}

/// Detail-endpoint shape — includes `workout_segments` with step structure.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Workout {
    #[serde(default)]
    pub workout_id: u64,
    #[serde(default = "untitled")]
    pub workout_name: String,
    pub sport_type: Option<SportTypeRef>,
    pub sub_sport_type: Option<SportTypeRef>,
    pub description: Option<String>,
    pub estimated_duration_in_secs: Option<f64>,
    pub estimated_distance_in_meters: Option<f64>,
    #[serde(rename(deserialize = "avgTrainingSpeed"))]
    pub avg_training_speed_mps: Option<f64>,
    pub created_date: Option<String>,
    pub updated_date: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub workout_segments: Vec<WorkoutSegment>,
}

impl HumanReadable for Workout {
    fn print_human(&self) {
        print_header(
            self.workout_id,
            &self.workout_name,
            self.sport_type.as_ref(),
            self.created_date.as_deref(),
        );
        print_body(
            self.description.as_deref(),
            self.estimated_duration_in_secs,
            self.estimated_distance_in_meters,
        );
        for seg in &self.workout_segments {
            print_steps(&seg.workout_steps, 1);
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct SportTypeRef {
    #[serde(default)]
    pub sport_type_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct WorkoutSegment {
    #[serde(default)]
    pub workout_steps: Vec<serde_json::Value>,
}

fn print_header(id: u64, name: &str, sport: Option<&SportTypeRef>, created: Option<&str>) {
    println!("{}", name.bold());
    println!("  {:<LABEL_WIDTH$}{}", "ID:", id.to_string().dimmed());
    if let Some(sport) = sport {
        println!("  {:<LABEL_WIDTH$}{}", "Sport:", sport.sport_type_key.cyan());
    }
    if let Some(date) = created {
        let short = &date[..date.len().min(19)];
        println!("  {:<LABEL_WIDTH$}{short}", "Created:");
    }
}

fn print_body(description: Option<&str>, duration: Option<f64>, distance: Option<f64>) {
    if let Some(desc) = description
        && !desc.is_empty()
    {
        println!("  {:<LABEL_WIDTH$}{desc}", "Description:");
    }
    if let Some(dur) = duration {
        let mins = (dur / 60.0).round() as u32;
        println!("  {:<LABEL_WIDTH$}{mins} min", "Duration:");
    }
    if let Some(dist) = distance {
        println!("  {:<LABEL_WIDTH$}{:.1} km", "Distance:", dist / 1000.0);
    }
}

/// Render workout steps recursively. Used by both workouts and coach commands.
pub fn print_steps(steps: &[serde_json::Value], indent: usize) {
    let pad = "  ".repeat(indent);
    for step in steps {
        let step_type = step["type"].as_str().unwrap_or("");
        if step_type == "RepeatGroupDTO" {
            let iters = step["numberOfIterations"].as_u64().unwrap_or(0);
            println!("{pad}{}", format!("Repeat {iters}x:").yellow());
            if let Some(sub) = step["workoutSteps"].as_array() {
                print_steps(sub, indent + 1);
            }
        } else {
            let kind = step["stepType"]["stepTypeKey"].as_str().unwrap_or("?");
            let exercise_name = step["exerciseName"].as_str();
            let desc = step["description"].as_str().unwrap_or("");
            let end_type = step["endCondition"]["conditionTypeKey"].as_str().unwrap_or("?");
            let end_val = step["endConditionValue"].as_f64().unwrap_or(0.0);

            let end_str = match end_type {
                "distance" => format!("{:.0}m", end_val),
                "time" => {
                    let secs = end_val as u64;
                    if secs >= 60 {
                        format!("{}:{:02}", secs / 60, secs % 60)
                    } else {
                        format!("{secs}s")
                    }
                }
                "lap.button" => "lap button".into(),
                _ => format!("{end_val} {end_type}"),
            };

            let target_str = format_target(step);

            let kind_colored = if let Some(name) = exercise_name {
                humanize_exercise(name).yellow().bold().to_string()
            } else {
                match kind {
                    "warmup" => "Warm Up".green().to_string(),
                    "cooldown" => "Cool Down".green().to_string(),
                    "interval" => "Run".yellow().bold().to_string(),
                    "recovery" => "Recover".cyan().to_string(),
                    "rest" => "Rest".dimmed().to_string(),
                    other => other.to_string(),
                }
            };

            let mut line = format!("{pad}{kind_colored} - {end_str}");
            if !target_str.is_empty() {
                line.push_str(&format!(" @ {target_str}"));
            }
            if !desc.is_empty() {
                line.push_str(&format!("  {}", desc.dimmed()));
            }
            println!("{line}");
        }
    }
}

fn humanize_exercise(name: &str) -> String {
    name.split('_')
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(c) => {
                    let mut s = c.to_uppercase().to_string();
                    s.extend(chars.map(|c| c.to_ascii_lowercase()));
                    s
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_target(step: &serde_json::Value) -> String {
    let target_key = step["targetType"]["workoutTargetTypeKey"].as_str().unwrap_or("");
    let v1 = step["targetValueOne"].as_f64();
    let v2 = step["targetValueTwo"].as_f64();

    match target_key {
        "pace.zone" => match (v1, v2) {
            (Some(a), Some(b)) => {
                let (fast, slow) = if a > b { (a, b) } else { (b, a) };
                format!("{}-{}", pace_from_speed(fast), pace_from_speed(slow))
            }
            _ => "pace target".into(),
        },
        "heart.rate.zone" => match (v1, v2) {
            (Some(a), Some(b)) if a == b => format!("{} bpm", a as u32),
            (Some(a), Some(b)) => {
                let (lo, hi) = if a < b { (a, b) } else { (b, a) };
                format!("{}-{} bpm", lo as u32, hi as u32)
            }
            _ => "HR target".into(),
        },
        "instruction" => match v1.map(|v| v as u32) {
            Some(0) => "no instruction".into(),
            Some(1) => "easy".into(),
            Some(2) => "moderate".into(),
            Some(3) => "hard".into(),
            Some(4) => "very hard".into(),
            Some(5) => "max effort".into(),
            Some(6) => "warm up".into(),
            Some(7) => "cool down".into(),
            Some(8) => "recovery".into(),
            Some(9) => "tempo".into(),
            Some(10) => "steady".into(),
            Some(11) => "race pace".into(),
            Some(12) => "all out".into(),
            Some(n) => format!("instruction({n})"),
            None => String::new(),
        },
        "power.zone" => match (v1, v2) {
            (Some(a), Some(b)) => {
                let (lo, hi) = if a < b { (a, b) } else { (b, a) };
                format!("{}-{} W", lo as u32, hi as u32)
            }
            _ => "power target".into(),
        },
        "no.target" | "" => String::new(),
        other => other.to_string(),
    }
}
