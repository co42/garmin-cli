use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

// Extra fields from `/activity-service/activity/{id}` (`summaryDTO`) that are
// not present on the list endpoint. Deserialized from the summaryDTO object
// only; see `GarminClient::activity_detail`.

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ActivityDetail {
    // HR / power min + normalized
    #[serde(rename(deserialize = "minHR"))]
    pub min_hr: Option<f64>,
    pub min_power: Option<f64>,
    pub normalized_power: Option<f64>,
    pub impact_load: Option<f64>,
    #[serde(rename(deserialize = "totalWork"))]
    pub total_work_joules: Option<f64>,
    // Speeds (m/s; list has `averageSpeed` too but names differ by endpoint)
    #[serde(rename(deserialize = "averageSpeed"))]
    pub average_speed_mps: Option<f64>,
    #[serde(rename(deserialize = "averageMovingSpeed"))]
    pub average_moving_speed_mps: Option<f64>,
    #[serde(rename(deserialize = "maxSpeed"))]
    pub max_speed_mps: Option<f64>,
    // Cadence peak
    pub max_run_cadence: Option<f64>,
    // Durations
    #[serde(rename(deserialize = "elapsedDuration"))]
    pub elapsed_duration_seconds: Option<f64>,
    // Altitude range (m)
    #[serde(rename(deserialize = "avgElevation"))]
    pub avg_elevation_meters: Option<f64>,
    #[serde(rename(deserialize = "maxElevation"))]
    pub max_elevation_meters: Option<f64>,
    #[serde(rename(deserialize = "minElevation"))]
    pub min_elevation_meters: Option<f64>,
    #[serde(rename(deserialize = "maxVerticalSpeed"))]
    pub max_vertical_speed_mps: Option<f64>,
    // Calorie breakdown
    pub bmr_calories: Option<f64>,
    // Location / time extras
    /// All-caps GMT; rename_all would produce `startTimeGmt`.
    #[serde(rename(deserialize = "startTimeGMT"))]
    pub start_time_gmt: Option<String>,
    pub end_latitude: Option<f64>,
    pub end_longitude: Option<f64>,
    // Stamina (range across the activity)
    pub begin_potential_stamina: Option<f64>,
    pub end_potential_stamina: Option<f64>,
    pub min_available_stamina: Option<f64>,
    // Subjective inputs logged on the watch
    pub direct_workout_feel: Option<i64>,
    pub direct_workout_rpe: Option<i64>,
    pub direct_workout_compliance_score: Option<f64>,
}
