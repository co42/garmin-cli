use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// One day's race-time projection snapshot for a target event. All durations
/// are in seconds, all speeds in m/s. Upper-bound means slower time / lower
/// speed; lower-bound means faster time / higher speed.
///
/// Speeds and `sporting_event_id` are kept internally (used by HumanReadable
/// and for cross-referencing) but not serialized to JSON — callers can derive
/// pace from time + known distance, and the outer `event.id` already carries
/// the event id.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct EventProjection {
    #[serde(rename(serialize = "date"))]
    pub calendar_date: String,

    #[serde(rename(deserialize = "predictedRaceTime"))]
    pub predicted_race_time_seconds: Option<f64>,
    #[serde(rename(deserialize = "projectionRaceTime"))]
    pub projection_race_time_seconds: Option<f64>,
    #[serde(rename(deserialize = "upperBoundProjectionRaceTime"))]
    pub upper_bound_projection_race_time_seconds: Option<f64>,
    #[serde(rename(deserialize = "lowerBoundProjectionRaceTime"))]
    pub lower_bound_projection_race_time_seconds: Option<f64>,

    /// Speed-based projections used for human-readable pace display.
    /// Redundant with time + known distance, so not serialized to JSON —
    /// callers should compute pace client-side from time + event distance.
    #[serde(rename(deserialize = "speedPrediction"), skip_serializing)]
    pub speed_prediction_mps: Option<f64>,
    #[serde(rename(deserialize = "speedProjection"), skip_serializing)]
    pub speed_projection_mps: Option<f64>,

    #[serde(rename(serialize = "feedback_phrase"))]
    pub event_race_predictions_feedback_phrase: Option<String>,
}
