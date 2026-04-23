use crate::garmin::types::workout::SportTypeRef;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// One entry in the adaptive plan `taskList`.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CoachTask {
    pub calendar_date: String,
    pub week_id: Option<u32>,
    pub day_of_week_id: Option<u32>,
    pub workout_order: Option<u32>,
    pub task_workout: CoachTaskWorkout,
}

/// The workout embedded in a task. `sport_type` is `null` for rest days.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CoachTaskWorkout {
    pub workout_uuid: Option<String>,
    pub workout_name: Option<String>,
    pub workout_description: Option<String>,
    pub sport_type: Option<SportTypeRef>,
    #[serde(rename(deserialize = "estimatedDurationInSecs"))]
    pub estimated_duration_seconds: Option<f64>,
    #[serde(rename(deserialize = "estimatedDistanceInMeters"))]
    pub estimated_distance_meters: Option<f64>,
    pub training_effect_label: Option<String>,
    pub workout_phrase: Option<String>,
    #[serde(default)]
    pub rest_day: bool,
    pub adaptive_coaching_workout_status: Option<String>,
}
