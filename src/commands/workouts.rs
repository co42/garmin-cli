use crate::client::GarminClient;
use crate::error::Result;
use crate::output::Output;

pub async fn list(client: &GarminClient, output: &Output, limit: u32, start: u32) -> Result<()> {
    let path = format!("/workout-service/workouts?start={start}&limit={limit}");
    let v: serde_json::Value = client.get_json(&path).await?;
    output.print_value(&v);
    Ok(())
}

pub async fn get(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    let path = format!("/workout-service/workout/{id}");
    let v: serde_json::Value = client.get_json(&path).await?;
    output.print_value(&v);
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
