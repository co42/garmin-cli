use serde::{Deserialize, Serialize};

/// One segment of the adaptive training plan timeline. `training_phase` is a
/// short enum: `BUILD` | `PEAK` | `TAPER` | `TARGET_EVENT_DAY`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct TrainingPhase {
    pub start_date: String,
    pub end_date: String,
    pub training_phase: String,
    #[serde(default)]
    pub current_phase: bool,
}
