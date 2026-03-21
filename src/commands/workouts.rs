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
    let workout = workout_from_json(&v);
    output.print(&workout);
    Ok(())
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

pub async fn delete(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    let path = format!("/workout-service/workout/{id}");
    client.delete(&path).await?;
    output.print_value(&serde_json::json!({
        "workoutId": id,
        "deleted": true,
    }));
    Ok(())
}

/// Print a hardcoded workout template to stdout.
pub fn template(output: &Output, kind: &str) {
    let value = match kind {
        "interval" => serde_json::json!({
            "workoutName": "Interval Workout",
            "sportType": { "sportTypeId": 1, "sportTypeKey": "running" },
            "workoutSegments": [
                {
                    "segmentOrder": 1,
                    "sportType": { "sportTypeId": 1, "sportTypeKey": "running" },
                    "workoutSteps": [
                        { "type": "ExecutableStepDTO", "stepOrder": 1, "stepType": { "stepTypeKey": "warmup" }, "endCondition": { "conditionTypeKey": "time" }, "endConditionValue": 600 },
                        { "type": "RepeatGroupDTO", "stepOrder": 2, "numberOfIterations": 6, "workoutSteps": [
                            { "type": "ExecutableStepDTO", "stepOrder": 1, "stepType": { "stepTypeKey": "interval" }, "endCondition": { "conditionTypeKey": "distance" }, "endConditionValue": 400 },
                            { "type": "ExecutableStepDTO", "stepOrder": 2, "stepType": { "stepTypeKey": "recovery" }, "endCondition": { "conditionTypeKey": "time" }, "endConditionValue": 90 }
                        ]},
                        { "type": "ExecutableStepDTO", "stepOrder": 3, "stepType": { "stepTypeKey": "cooldown" }, "endCondition": { "conditionTypeKey": "time" }, "endConditionValue": 600 }
                    ]
                }
            ]
        }),
        "tempo" => serde_json::json!({
            "workoutName": "Tempo Run",
            "sportType": { "sportTypeId": 1, "sportTypeKey": "running" },
            "workoutSegments": [
                {
                    "segmentOrder": 1,
                    "sportType": { "sportTypeId": 1, "sportTypeKey": "running" },
                    "workoutSteps": [
                        { "type": "ExecutableStepDTO", "stepOrder": 1, "stepType": { "stepTypeKey": "warmup" }, "endCondition": { "conditionTypeKey": "time" }, "endConditionValue": 900 },
                        { "type": "ExecutableStepDTO", "stepOrder": 2, "stepType": { "stepTypeKey": "interval" }, "endCondition": { "conditionTypeKey": "time" }, "endConditionValue": 1200, "targetType": { "workoutTargetTypeKey": "pace.zone" } },
                        { "type": "ExecutableStepDTO", "stepOrder": 3, "stepType": { "stepTypeKey": "cooldown" }, "endCondition": { "conditionTypeKey": "time" }, "endConditionValue": 600 }
                    ]
                }
            ]
        }),
        "easy" => serde_json::json!({
            "workoutName": "Easy Run",
            "sportType": { "sportTypeId": 1, "sportTypeKey": "running" },
            "workoutSegments": [
                {
                    "segmentOrder": 1,
                    "sportType": { "sportTypeId": 1, "sportTypeKey": "running" },
                    "workoutSteps": [
                        { "type": "ExecutableStepDTO", "stepOrder": 1, "stepType": { "stepTypeKey": "warmup" }, "endCondition": { "conditionTypeKey": "time" }, "endConditionValue": 300 },
                        { "type": "ExecutableStepDTO", "stepOrder": 2, "stepType": { "stepTypeKey": "interval" }, "endCondition": { "conditionTypeKey": "time" }, "endConditionValue": 2400, "targetType": { "workoutTargetTypeKey": "heart.rate.zone" }, "targetValueOne": 1, "targetValueTwo": 2 },
                        { "type": "ExecutableStepDTO", "stepOrder": 3, "stepType": { "stepTypeKey": "cooldown" }, "endCondition": { "conditionTypeKey": "time" }, "endConditionValue": 300 }
                    ]
                }
            ]
        }),
        _ /* long_run */ => serde_json::json!({
            "workoutName": "Long Run",
            "sportType": { "sportTypeId": 1, "sportTypeKey": "running" },
            "workoutSegments": [
                {
                    "segmentOrder": 1,
                    "sportType": { "sportTypeId": 1, "sportTypeKey": "running" },
                    "workoutSteps": [
                        { "type": "ExecutableStepDTO", "stepOrder": 1, "stepType": { "stepTypeKey": "warmup" }, "endCondition": { "conditionTypeKey": "time" }, "endConditionValue": 600 },
                        { "type": "ExecutableStepDTO", "stepOrder": 2, "stepType": { "stepTypeKey": "interval" }, "endCondition": { "conditionTypeKey": "distance" }, "endConditionValue": 20000 },
                        { "type": "ExecutableStepDTO", "stepOrder": 3, "stepType": { "stepTypeKey": "cooldown" }, "endCondition": { "conditionTypeKey": "time" }, "endConditionValue": 600 }
                    ]
                }
            ]
        }),
    };

    output.print_value(&value);
}
