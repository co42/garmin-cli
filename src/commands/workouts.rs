use super::output::Output;
use crate::error::Result;
use crate::garmin::GarminClient;
use clap::{Subcommand, ValueEnum};

#[derive(Subcommand)]
pub enum WorkoutCommands {
    /// List saved workouts
    List {
        /// Max workouts to return
        #[arg(long, default_value = "20")]
        limit: u32,
        /// Start index for pagination
        #[arg(long, default_value = "0")]
        start: u32,
        /// Show step details for each workout
        #[arg(long)]
        steps: bool,
    },
    /// Get workout details
    Get {
        /// Workout ID
        id: u64,
    },
    /// Create workout from JSON file
    Create {
        /// Path to workout JSON file
        #[arg(long, short)]
        file: String,
    },
    /// Schedule workout on a date
    Schedule {
        /// Workout ID
        id: u64,
        /// Date (YYYY-MM-DD)
        date: String,
    },
    /// Update an existing workout from JSON file
    Update {
        /// Workout ID
        id: u64,
        /// Path to workout JSON file
        #[arg(long, short)]
        file: String,
    },
    /// Delete a workout
    Delete {
        /// Workout ID
        id: u64,
    },
    /// Generate a workout template
    Template {
        /// Template type
        #[arg(long, default_value = "interval")]
        r#type: TemplateType,
    },
}

#[derive(Clone, ValueEnum)]
pub enum TemplateType {
    Interval,
    Tempo,
    Easy,
    LongRun,
}

pub async fn run(command: WorkoutCommands, output: &Output) -> Result<()> {
    // Template is local-only — no client needed.
    if let WorkoutCommands::Template { r#type } = &command {
        let kind = match r#type {
            TemplateType::Interval => "interval",
            TemplateType::Tempo => "tempo",
            TemplateType::Easy => "easy",
            TemplateType::LongRun => "long_run",
        };
        template(output, kind);
        return Ok(());
    }

    let client = GarminClient::new(super::helpers::require_auth()?)?;
    match command {
        WorkoutCommands::List { limit, start, steps } => list(&client, output, limit, start, steps).await,
        WorkoutCommands::Get { id } => get(&client, output, id).await,
        WorkoutCommands::Create { file } => create(&client, output, &file).await,
        WorkoutCommands::Schedule { id, date } => schedule(&client, output, id, &date).await,
        WorkoutCommands::Update { id, file } => update(&client, output, id, &file).await,
        WorkoutCommands::Delete { id } => delete(&client, output, id).await,
        WorkoutCommands::Template { .. } => unreachable!("handled above"),
    }
}

async fn list(client: &GarminClient, output: &Output, limit: u32, start: u32, steps: bool) -> Result<()> {
    let summaries = client.list_workouts(limit, start).await?;
    if !steps {
        output.print_list(&summaries, "Workouts");
        return Ok(());
    }
    // `--steps` upgrades each list entry to a detail fetch so step structure is printed.
    let details = futures::future::try_join_all(summaries.iter().map(|s| client.workout(s.workout_id))).await?;
    output.print_list(&details, "Workouts");
    Ok(())
}

async fn get(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    let workout = client.workout(id).await?;
    output.print(&workout);
    Ok(())
}

async fn create(client: &GarminClient, output: &Output, file: &str) -> Result<()> {
    let data = std::fs::read_to_string(file)?;
    let body: serde_json::Value = serde_json::from_str(&data)?;
    let result = client.create_workout(&body).await?;
    output.print_value(&result);
    Ok(())
}

async fn schedule(client: &GarminClient, output: &Output, id: u64, date: &str) -> Result<()> {
    client.schedule_workout(id, date).await?;
    output.print_value(&serde_json::json!({
        "workoutId": id,
        "date": date,
        "scheduled": true,
    }));
    Ok(())
}

async fn update(client: &GarminClient, output: &Output, id: u64, file: &str) -> Result<()> {
    let data = std::fs::read_to_string(file)?;
    let body: serde_json::Value = serde_json::from_str(&data)?;
    client.update_workout(id, &body).await?;
    let updated = client.workout(id).await?;
    output.print(&updated);
    Ok(())
}

async fn delete(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    client.delete_workout(id).await?;
    output.print_value(&serde_json::json!({
        "workoutId": id,
        "deleted": true,
    }));
    Ok(())
}

// Workout templates are local-only; render them via print_value.
fn template(output: &Output, kind: &str) {
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
                                "targetValueOne": 4.444, "targetValueTwo": 3.922,
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
                        "targetValueOne": 3.774, "targetValueTwo": 3.509,
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
