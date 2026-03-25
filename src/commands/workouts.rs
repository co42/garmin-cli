use crate::client::GarminClient;
use crate::error::Result;
use crate::output::{HumanReadable, Output};
use colored::Colorize;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Workout {
    pub id: u64,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sport_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_duration_seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_distance_meters: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_date: Option<String>,
}

fn workout_from_json(v: &serde_json::Value) -> Workout {
    let format_ts = |key: &str| -> Option<String> {
        v[key]
            .as_str()
            .map(|s| s[..s.len().min(19)].to_string())
            .or_else(|| {
                v[key].as_i64().map(|ts| {
                    // Garmin sometimes returns epoch millis
                    let secs = ts / 1000;
                    let dt = chrono::DateTime::from_timestamp(secs, 0);
                    dt.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| ts.to_string())
                })
            })
    };

    Workout {
        id: v["workoutId"].as_u64().unwrap_or(0),
        name: v["workoutName"].as_str().unwrap_or("Untitled").into(),
        sport_type: v["sportType"]["sportTypeKey"].as_str().map(Into::into),
        description: v["description"].as_str().map(Into::into),
        estimated_duration_seconds: v["estimatedDurationInSecs"].as_f64(),
        estimated_distance_meters: v["estimatedDistanceInMeters"].as_f64(),
        created_date: format_ts("createdDate"),
        updated_date: format_ts("updateDate"),
    }
}

impl HumanReadable for Workout {
    fn print_human(&self) {
        let sport = self.sport_type.as_deref().unwrap_or("unknown");
        println!("{} [{}]", self.name.bold(), sport.cyan(),);
        println!("  ID: {}", self.id);
        if let Some(ref desc) = self.description
            && !desc.is_empty()
        {
            println!("  {}", desc.dimmed());
        }
        if let Some(dur) = self.estimated_duration_seconds {
            let mins = (dur / 60.0).round() as u32;
            println!("  Est. duration: {mins} min");
        }
        if let Some(dist) = self.estimated_distance_meters {
            println!("  Est. distance: {:.2} km", dist / 1000.0);
        }
        if let Some(ref date) = self.created_date {
            println!("  Created: {date}");
        }
        println!();
    }
}

pub async fn list(client: &GarminClient, output: &Output, limit: u32, start: u32) -> Result<()> {
    let path = format!("/workout-service/workouts?start={start}&limit={limit}");
    let v: serde_json::Value = client.get_json(&path).await?;

    let workouts: Vec<Workout> = v
        .as_array()
        .map(|arr| arr.iter().map(workout_from_json).collect())
        .unwrap_or_default();

    output.print_list(&workouts, "Workouts");
    Ok(())
}

pub async fn get(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    let path = format!("/workout-service/workout/{id}");
    let v: serde_json::Value = client.get_json(&path).await?;
    if output.is_json() {
        // In JSON mode, return the full API response (includes steps)
        output.print_value(&v);
    } else {
        let workout = workout_from_json(&v);
        workout.print_human();
        // Print step structure
        if let Some(segments) = v["workoutSegments"].as_array() {
            for seg in segments {
                if let Some(steps) = seg["workoutSteps"].as_array() {
                    print_steps(steps, 1);
                }
            }
        }
    }
    Ok(())
}

fn print_steps(steps: &[serde_json::Value], indent: usize) {
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
            let desc = step["description"].as_str().unwrap_or("");
            let end_type = step["endCondition"]["conditionTypeKey"]
                .as_str()
                .unwrap_or("?");
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

            let kind_colored = match kind {
                "warmup" => "Warm Up".green().to_string(),
                "cooldown" => "Cool Down".green().to_string(),
                "interval" => "Run".yellow().bold().to_string(),
                "recovery" => "Recover".cyan().to_string(),
                "rest" => "Rest".dimmed().to_string(),
                other => other.to_string(),
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

fn format_target(step: &serde_json::Value) -> String {
    let target_key = step["targetType"]["workoutTargetTypeKey"]
        .as_str()
        .unwrap_or("");
    let v1 = step["targetValueOne"].as_f64();
    let v2 = step["targetValueTwo"].as_f64();

    match target_key {
        "pace.zone" => {
            let fmt_pace = |secs: f64| -> String {
                let s = secs as u64;
                format!("{}:{:02}/km", s / 60, s % 60)
            };
            match (v1, v2) {
                (Some(a), Some(b)) => format!("{}-{}", fmt_pace(a), fmt_pace(b)),
                _ => "pace target".into(),
            }
        }
        "heart.rate.zone" => match (v1, v2) {
            (Some(a), Some(b)) if a == b => format!("{} bpm", a as u32),
            (Some(a), Some(b)) => format!("{}-{} bpm", a as u32, b as u32),
            _ => "HR target".into(),
        },
        "" => String::new(),
        other => other.to_string(),
    }
}

pub async fn create(client: &GarminClient, output: &Output, file: &str) -> Result<()> {
    let data = std::fs::read_to_string(file)?;
    let body: serde_json::Value = serde_json::from_str(&data)?;
    let result: serde_json::Value = client.post_json("/workout-service/workout", &body).await?;
    output.print_value(&result);
    Ok(())
}

pub async fn schedule(client: &GarminClient, output: &Output, id: u64, date: &str) -> Result<()> {
    let body = serde_json::json!({
        "date": date,
    });
    let path = format!("/workout-service/schedule/{id}");
    client.post(&path, &body).await?;
    output.print_value(&serde_json::json!({
        "workoutId": id,
        "date": date,
        "scheduled": true,
    }));
    Ok(())
}

pub async fn update(client: &GarminClient, output: &Output, id: u64, file: &str) -> Result<()> {
    let data = std::fs::read_to_string(file)?;
    let body: serde_json::Value = serde_json::from_str(&data)?;
    let path = format!("/workout-service/workout/{id}");
    // PUT returns 204 No Content on success
    client.put(&path, &body).await?;
    // Fetch the updated workout to show the result
    let updated: serde_json::Value = client
        .get_json(&format!("/workout-service/workout/{id}"))
        .await?;
    if output.is_json() {
        output.print_value(&updated);
    } else {
        let workout = workout_from_json(&updated);
        workout.print_human();
        if let Some(segments) = updated["workoutSegments"].as_array() {
            for seg in segments {
                if let Some(steps) = seg["workoutSteps"].as_array() {
                    print_steps(steps, 1);
                }
            }
        }
    }
    Ok(())
}

pub async fn delete(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    let path = format!("/workout-service/workout/{id}");
    client.delete(&path).await?;
    output.print_value(&serde_json::json!({
        "workoutId": id,
        "deleted": true,
    }));
    Ok(())
}

// Garmin API IDs reference:
//   stepTypeId:     1=warmup, 2=cooldown, 3=interval, 4=recovery, 5=rest
//   conditionTypeId: 1=lap.button, 2=time, 3=distance
//   targetTypeId:   4=heart.rate.zone, 6=pace.zone
//   Pace targets:   m/s (e.g. 3.774 = 4:25/km).  Convert: 1000 / seconds_per_km = m/s
//   HR targets:     BPM values (e.g. targetValueOne=120, targetValueTwo=150)
//                   Zone numbers (1-5) do NOT work - the watch interprets them
//                   as literal BPM. Always use actual BPM values.
//                   Use `garmin training zones` to get your HR zone boundaries.

/// Print a hardcoded workout template to stdout.
pub fn template(output: &Output, kind: &str) {
    let value = match kind {
        "interval" => serde_json::json!({
            "workoutName": "Interval Workout",
            "sportType": { "sportTypeId": 1, "sportTypeKey": "running" },
            "workoutSegments": [{
                "segmentOrder": 1,
                "sportType": { "sportTypeId": 1, "sportTypeKey": "running" },
                "workoutSteps": [
                    {
                        "type": "ExecutableStepDTO", "stepOrder": 1,
                        "stepType": { "stepTypeId": 1, "stepTypeKey": "warmup" },
                        "endCondition": { "conditionTypeId": 3, "conditionTypeKey": "distance" },
                        "endConditionValue": 2000,
                        "description": "Z1-Z2 Warm up"
                    },
                    {
                        "type": "RepeatGroupDTO", "stepOrder": 2,
                        "numberOfIterations": 6,
                        "workoutSteps": [
                            {
                                "type": "ExecutableStepDTO", "stepOrder": 1,
                                "stepType": { "stepTypeId": 3, "stepTypeKey": "interval" },
                                "endCondition": { "conditionTypeId": 3, "conditionTypeKey": "distance" },
                                "endConditionValue": 400,
                                "targetType": { "workoutTargetTypeId": 6, "workoutTargetTypeKey": "pace.zone" },
                                "targetValueOne": 3.922, "targetValueTwo": 4.444,
                                "description": "Z5 VO2max (~3:45-4:15/km)"
                            },
                            {
                                "type": "ExecutableStepDTO", "stepOrder": 2,
                                "stepType": { "stepTypeId": 4, "stepTypeKey": "recovery" },
                                "endCondition": { "conditionTypeId": 2, "conditionTypeKey": "time" },
                                "endConditionValue": 90,
                                "description": "Recovery jog"
                            }
                        ]
                    },
                    {
                        "type": "ExecutableStepDTO", "stepOrder": 3,
                        "stepType": { "stepTypeId": 2, "stepTypeKey": "cooldown" },
                        "endCondition": { "conditionTypeId": 3, "conditionTypeKey": "distance" },
                        "endConditionValue": 2000,
                        "description": "Z1-Z2 Cool down"
                    }
                ]
            }]
        }),
        "tempo" => serde_json::json!({
            "workoutName": "Tempo Run",
            "sportType": { "sportTypeId": 1, "sportTypeKey": "running" },
            "workoutSegments": [{
                "segmentOrder": 1,
                "sportType": { "sportTypeId": 1, "sportTypeKey": "running" },
                "workoutSteps": [
                    {
                        "type": "ExecutableStepDTO", "stepOrder": 1,
                        "stepType": { "stepTypeId": 1, "stepTypeKey": "warmup" },
                        "endCondition": { "conditionTypeId": 3, "conditionTypeKey": "distance" },
                        "endConditionValue": 2000,
                        "description": "Z1-Z2 Warm up"
                    },
                    {
                        "type": "ExecutableStepDTO", "stepOrder": 2,
                        "stepType": { "stepTypeId": 3, "stepTypeKey": "interval" },
                        "endCondition": { "conditionTypeId": 2, "conditionTypeKey": "time" },
                        "endConditionValue": 1200,
                        "targetType": { "workoutTargetTypeId": 6, "workoutTargetTypeKey": "pace.zone" },
                        "targetValueOne": 3.509, "targetValueTwo": 3.774,
                        "description": "Z4 Threshold (~4:25-4:45/km)"
                    },
                    {
                        "type": "ExecutableStepDTO", "stepOrder": 3,
                        "stepType": { "stepTypeId": 2, "stepTypeKey": "cooldown" },
                        "endCondition": { "conditionTypeId": 3, "conditionTypeKey": "distance" },
                        "endConditionValue": 2000,
                        "description": "Z1-Z2 Cool down"
                    }
                ]
            }]
        }),
        "easy" => serde_json::json!({
            "workoutName": "Easy Run",
            "sportType": { "sportTypeId": 1, "sportTypeKey": "running" },
            "workoutSegments": [{
                "segmentOrder": 1,
                "sportType": { "sportTypeId": 1, "sportTypeKey": "running" },
                "workoutSteps": [
                    {
                        "type": "ExecutableStepDTO", "stepOrder": 1,
                        "stepType": { "stepTypeId": 3, "stepTypeKey": "interval" },
                        "endCondition": { "conditionTypeId": 3, "conditionTypeKey": "distance" },
                        "endConditionValue": 8000,
                        "targetType": { "workoutTargetTypeId": 4, "workoutTargetTypeKey": "heart.rate.zone" },
                        "targetValueOne": 120, "targetValueTwo": 150,
                        "description": "Z1-Z2 Easy - adjust BPM to your zones"
                    }
                ]
            }]
        }),
        _ /* long_run */ => serde_json::json!({
            "workoutName": "Long Run",
            "sportType": { "sportTypeId": 1, "sportTypeKey": "running" },
            "workoutSegments": [{
                "segmentOrder": 1,
                "sportType": { "sportTypeId": 1, "sportTypeKey": "running" },
                "workoutSteps": [
                    {
                        "type": "ExecutableStepDTO", "stepOrder": 1,
                        "stepType": { "stepTypeId": 1, "stepTypeKey": "warmup" },
                        "endCondition": { "conditionTypeId": 3, "conditionTypeKey": "distance" },
                        "endConditionValue": 2000,
                        "description": "Z1-Z2 Warm up"
                    },
                    {
                        "type": "ExecutableStepDTO", "stepOrder": 2,
                        "stepType": { "stepTypeId": 3, "stepTypeKey": "interval" },
                        "endCondition": { "conditionTypeId": 3, "conditionTypeKey": "distance" },
                        "endConditionValue": 16000,
                        "targetType": { "workoutTargetTypeId": 4, "workoutTargetTypeKey": "heart.rate.zone" },
                        "targetValueOne": 130, "targetValueTwo": 150,
                        "description": "Z2 Endurance - adjust BPM to your zones"
                    },
                    {
                        "type": "ExecutableStepDTO", "stepOrder": 3,
                        "stepType": { "stepTypeId": 2, "stepTypeKey": "cooldown" },
                        "endCondition": { "conditionTypeId": 3, "conditionTypeKey": "distance" },
                        "endConditionValue": 2000,
                        "description": "Z1 Cool down"
                    }
                ]
            }]
        }),
    };

    output.print_value(&value);
}
