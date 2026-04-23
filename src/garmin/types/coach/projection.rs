use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// One day's race-time projection snapshot for a target event. All durations
/// are in seconds, all speeds in m/s. Upper-bound means slower time / lower
/// speed; lower-bound means faster time / higher speed.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct EventProjection {
    pub calendar_date: String,
    pub sporting_event_id: u64,

    #[serde(rename(deserialize = "predictedRaceTime"))]
    pub predicted_race_time_seconds: Option<f64>,
    #[serde(rename(deserialize = "projectionRaceTime"))]
    pub projection_race_time_seconds: Option<f64>,
    #[serde(rename(deserialize = "upperBoundProjectionRaceTime"))]
    pub upper_bound_projection_race_time_seconds: Option<f64>,
    #[serde(rename(deserialize = "lowerBoundProjectionRaceTime"))]
    pub lower_bound_projection_race_time_seconds: Option<f64>,

    #[serde(rename(deserialize = "speedPrediction"))]
    pub speed_prediction_mps: Option<f64>,
    #[serde(rename(deserialize = "speedProjection"))]
    pub speed_projection_mps: Option<f64>,
    #[serde(rename(deserialize = "upperBoundProjectionSpeed"))]
    pub upper_bound_projection_speed_mps: Option<f64>,
    #[serde(rename(deserialize = "lowerBoundProjectionSpeed"))]
    pub lower_bound_projection_speed_mps: Option<f64>,

    pub event_race_predictions_feedback_phrase: Option<String>,
}
