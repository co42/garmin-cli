use crate::client::GarminClient;
use crate::error::Result;
use crate::output::{HumanReadable, Output};
use colored::Colorize;
use serde::Serialize;

// --- Coach workout (list item) ---

#[derive(Debug, Serialize)]
pub struct CoachWorkout {
    pub uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workout_phrase: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub training_effect_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_training_effect: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_anaerobic_training_effect: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_duration_seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_distance_meters: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sport_type: Option<String>,
}

fn coach_workout_from_json(v: &serde_json::Value) -> CoachWorkout {
    CoachWorkout {
        uuid: v["workoutUuid"].as_str().unwrap_or("unknown").into(),
        name: v["workoutName"].as_str().map(Into::into),
        description: v["description"].as_str().map(Into::into),
        workout_phrase: v["workoutPhrase"].as_str().map(Into::into),
        training_effect_label: v["trainingEffectLabel"].as_str().map(Into::into),
        priority_type: v["priorityType"].as_str().map(Into::into),
        estimated_training_effect: v["estimatedTrainingEffect"].as_f64(),
        estimated_anaerobic_training_effect: v["estimatedAnaerobicTrainingEffect"].as_f64(),
        estimated_duration_seconds: v["estimatedDurationInSecs"].as_f64(),
        estimated_distance_meters: v["estimatedDistanceInMeters"].as_f64(),
        sport_type: v["sportType"]["sportTypeKey"].as_str().map(Into::into),
    }
}

fn humanize_phrase(phrase: &str) -> &str {
    match phrase {
        "ANAEROBIC_SPEED" => "anaerobic speed",
        "BASE" => "base",
        "LONG_WORKOUT" => "long run",
        "RUNNING_HISTORY_SHORTENED_BASE" => "shortened base",
        "FORCED_REST" => "rest",
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
        let name = self.name.as_deref().unwrap_or("Rest");
        let priority = self.priority_type.as_deref().unwrap_or("");
        let alt_tag = match priority {
            "REQUIRED" => "",
            _ => " [alt]",
        };

        let phrase = self
            .workout_phrase
            .as_deref()
            .map(humanize_phrase)
            .unwrap_or("");

        let te = format_te(
            self.estimated_training_effect,
            self.estimated_anaerobic_training_effect,
        );

        // Single line: UUID  Name [phrase] TE  description  duration  distance
        print!("{} {}", self.uuid.dimmed(), name.bold());
        if !alt_tag.is_empty() {
            print!("{}", alt_tag.dimmed());
        }
        if !phrase.is_empty() && phrase != "rest" {
            print!(" [{}]", phrase.cyan());
        }
        if !te.is_empty() {
            print!("  {te}");
        }
        if let Some(ref desc) = self.description {
            print!("  {}", desc.dimmed());
        }
        if let Some(dur) = self.estimated_duration_seconds {
            let mins = (dur / 60.0).round() as u32;
            print!("  {mins} min");
        }
        if let Some(dist) = self.estimated_distance_meters {
            print!("  {:.1} km", dist / 1000.0);
        }
        println!();
    }
}

// --- Coach plan ---

#[derive(Debug, Serialize)]
pub struct CoachPlan {
    pub id: u64,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_weeks: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub training_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_weekly_workouts: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub training_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

fn coach_plan_from_json(v: &serde_json::Value) -> CoachPlan {
    let truncate_date =
        |key: &str| -> Option<String> { v[key].as_str().map(|s| s[..s.len().min(10)].to_string()) };

    CoachPlan {
        id: v["trainingPlanId"].as_u64().unwrap_or(0),
        name: v["name"].as_str().unwrap_or("Unknown").into(),
        start_date: truncate_date("startDate"),
        end_date: truncate_date("endDate"),
        duration_weeks: v["durationInWeeks"].as_u64().map(|w| w as u32),
        training_level: v["trainingLevel"]["levelKey"].as_str().map(Into::into),
        avg_weekly_workouts: v["avgWeeklyWorkouts"].as_u64().map(|w| w as u32),
        training_version: v["trainingVersion"]["versionName"].as_str().map(Into::into),
        status: v["trainingStatus"]["statusKey"].as_str().map(Into::into),
    }
}

impl HumanReadable for CoachPlan {
    fn print_human(&self) {
        println!("{}", self.name.bold());
        println!("  ID: {}", self.id);
        if let Some(ref level) = self.training_level {
            println!("  Level: {}", level.cyan());
        }
        if let Some(ref version) = self.training_version {
            println!("  Target: {version}");
        }
        if let (Some(start), Some(end)) = (&self.start_date, &self.end_date) {
            print!("  {start} \u{2192} {end}");
            if let Some(weeks) = self.duration_weeks {
                print!("  ({weeks} weeks)");
            }
            println!();
        }
        if let Some(avg) = self.avg_weekly_workouts {
            println!("  {avg} workouts/week");
        }
        if let Some(ref status) = self.status {
            println!("  Status: {status}");
        }
        println!();
    }
}

// --- Commands ---

pub async fn list(client: &GarminClient, output: &Output, all: bool, verbose: bool) -> Result<()> {
    let v: serde_json::Value = client.get_json("/workout-service/fbt-adaptive").await?;

    if output.is_json() {
        output.print_value(&v);
    } else {
        let arr = v.as_array().map(|a| a.as_slice()).unwrap_or_default();
        let workouts: Vec<CoachWorkout> = arr.iter().map(coach_workout_from_json).collect();

        if all {
            if verbose {
                print_coach_list_verbose(arr, &workouts);
            } else {
                output.print_list(&workouts, "Coach Workouts (all variants)");
            }
        } else {
            // Show only REQUIRED workouts by default (alternates are noise)
            let (filtered_raw, filtered): (Vec<_>, Vec<_>) = arr
                .iter()
                .zip(workouts.into_iter())
                .filter(|(_, w)| w.priority_type.as_deref() == Some("REQUIRED"))
                .unzip();
            if verbose {
                print_coach_list_verbose(
                    &filtered_raw.into_iter().cloned().collect::<Vec<_>>(),
                    &filtered,
                );
            } else {
                output.print_list(&filtered, "Coach Workouts");
            }
        }
    }
    Ok(())
}

fn print_coach_list_verbose(raw: &[serde_json::Value], workouts: &[CoachWorkout]) {
    for (item, workout) in raw.iter().zip(workouts.iter()) {
        workout.print_human();
        if let Some(segments) = item["workoutSegments"].as_array() {
            for seg in segments {
                if let Some(steps) = seg["workoutSteps"].as_array() {
                    super::workouts::print_steps(steps, 1);
                }
            }
        }
        println!();
    }
    println!("{} items", workouts.len());
}

pub async fn get(client: &GarminClient, output: &Output, uuid: &str) -> Result<()> {
    let path = format!("/workout-service/fbt-adaptive/{uuid}");
    let v: serde_json::Value = client.get_json(&path).await?;

    if output.is_json() {
        output.print_value(&v);
    } else {
        let workout = coach_workout_from_json(&v);
        workout.print_human();
        // Print step structure (reuse from workouts module)
        if let Some(segments) = v["workoutSegments"].as_array() {
            for seg in segments {
                if let Some(steps) = seg["workoutSteps"].as_array() {
                    super::workouts::print_steps(steps, 1);
                }
            }
        }
    }
    Ok(())
}

pub async fn plan(client: &GarminClient, output: &Output) -> Result<()> {
    // Get training plan ID from the first FBT workout
    let workouts: serde_json::Value = client.get_json("/workout-service/fbt-adaptive").await?;

    let plan_id = workouts
        .as_array()
        .and_then(|arr| arr.iter().find_map(|w| w["trainingPlanId"].as_u64()));

    let Some(plan_id) = plan_id else {
        return Err(crate::error::Error::Api(
            "No active Coach training plan found".into(),
        ));
    };

    let path = format!("/trainingplan-service/trainingplan/{plan_id}");
    let v: serde_json::Value = client.get_json(&path).await?;

    if output.is_json() {
        output.print_value(&v);
    } else {
        let plan = coach_plan_from_json(&v);
        plan.print_human();
    }
    Ok(())
}
