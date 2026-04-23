use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use crate::garmin::types::helpers::{
    compute_pace, deser_norm_ts, deser_type_key, fmt_dist, fmt_hms, unknown_key, untitled,
};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

// Deserializes from the list endpoint (`/activitylist-service/...`), which has
// the flatter shape. `compute_pace` is derived post-deserialization.
// TODO: `get_activity` used to merge the detail endpoint (with running dynamics
// under `summaryDTO`). That merge is not done any more — some detail-only
// fields won't appear for `activities get`. Revisit if users care.

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ActivitySummary {
    #[serde(default)]
    pub activity_id: u64,
    #[serde(default = "untitled")]
    pub activity_name: String,
    #[serde(default = "unknown_key", deserialize_with = "deser_type_key")]
    pub activity_type: String,
    #[serde(default, deserialize_with = "deser_norm_ts")]
    pub start_time_local: String,
    #[serde(rename(deserialize = "duration"), default)]
    pub duration_seconds: f64,
    #[serde(rename(deserialize = "distance"))]
    pub distance_meters: Option<f64>,
    pub calories: Option<f64>,
    #[serde(rename(deserialize = "averageHR"))]
    pub average_hr: Option<f64>,
    #[serde(rename(deserialize = "maxHR"))]
    pub max_hr: Option<f64>,
    #[serde(rename(deserialize = "movingDuration"))]
    pub moving_duration_seconds: Option<f64>,

    // Training Effect & Load
    pub aerobic_training_effect: Option<f64>,
    pub anaerobic_training_effect: Option<f64>,
    pub aerobic_training_effect_message: Option<String>,
    pub anaerobic_training_effect_message: Option<String>,
    pub training_effect_label: Option<String>,
    pub activity_training_load: Option<f64>,
    pub impact_load: Option<f64>,

    // Performance
    #[serde(rename(deserialize = "vO2MaxValue"))]
    pub vo2_max_value: Option<f64>,
    pub average_power: Option<f64>,
    pub norm_power: Option<f64>,
    pub max_power: Option<f64>,

    // Running dynamics
    #[serde(rename(deserialize = "avgRunningCadenceInStepsPerMinute"))]
    pub avg_running_cadence: Option<f64>,
    #[serde(rename(deserialize = "avgStrideLength"))]
    pub avg_stride_length_cm: Option<f64>,
    #[serde(rename(deserialize = "avgGroundContactTime"))]
    pub avg_ground_contact_time_ms: Option<f64>,
    #[serde(rename(deserialize = "avgVerticalOscillation"))]
    pub avg_vertical_oscillation_cm: Option<f64>,
    #[serde(rename(deserialize = "avgVerticalRatio"))]
    pub avg_vertical_ratio_percent: Option<f64>,

    // Elevation
    #[serde(rename(deserialize = "elevationGain"))]
    pub elevation_gain_meters: Option<f64>,
    #[serde(rename(deserialize = "elevationLoss"))]
    pub elevation_loss_meters: Option<f64>,
    #[serde(rename(deserialize = "avgGradeAdjustedSpeed"))]
    pub avg_grade_adjusted_speed_mps: Option<f64>,

    // Splits (API name uses underscore+digit, rename_all can't reproduce)
    #[serde(rename(deserialize = "fastestSplit_1000"))]
    pub fastest_split_1000_seconds: Option<f64>,
    #[serde(rename(deserialize = "fastestSplit_1609"))]
    pub fastest_split_1609_seconds: Option<f64>,
    #[serde(rename(deserialize = "fastestSplit_5000"))]
    pub fastest_split_5000_seconds: Option<f64>,

    // Misc
    pub moderate_intensity_minutes: Option<f64>,
    pub vigorous_intensity_minutes: Option<f64>,
    pub difference_body_battery: Option<f64>,
    pub steps: Option<u64>,
    pub location_name: Option<String>,
    pub start_latitude: Option<f64>,
    pub start_longitude: Option<f64>,
    pub workout_id: Option<u64>,

    // HR zones (underscore+digit + all-caps HR) — values are seconds
    #[serde(rename(deserialize = "hrTimeInZone_1"))]
    pub hr_time_in_zone_1_seconds: Option<f64>,
    #[serde(rename(deserialize = "hrTimeInZone_2"))]
    pub hr_time_in_zone_2_seconds: Option<f64>,
    #[serde(rename(deserialize = "hrTimeInZone_3"))]
    pub hr_time_in_zone_3_seconds: Option<f64>,
    #[serde(rename(deserialize = "hrTimeInZone_4"))]
    pub hr_time_in_zone_4_seconds: Option<f64>,
    #[serde(rename(deserialize = "hrTimeInZone_5"))]
    pub hr_time_in_zone_5_seconds: Option<f64>,

    // Power zones (underscore+digit) — values are seconds
    #[serde(rename(deserialize = "powerTimeInZone_1"))]
    pub power_time_in_zone_1_seconds: Option<f64>,
    #[serde(rename(deserialize = "powerTimeInZone_2"))]
    pub power_time_in_zone_2_seconds: Option<f64>,
    #[serde(rename(deserialize = "powerTimeInZone_3"))]
    pub power_time_in_zone_3_seconds: Option<f64>,
    #[serde(rename(deserialize = "powerTimeInZone_4"))]
    pub power_time_in_zone_4_seconds: Option<f64>,
    #[serde(rename(deserialize = "powerTimeInZone_5"))]
    pub power_time_in_zone_5_seconds: Option<f64>,
}

impl ActivitySummary {
    pub fn pace_min_km(&self) -> Option<String> {
        compute_pace(self.distance_meters, self.duration_seconds)
    }
}

impl HumanReadable for ActivitySummary {
    fn print_human(&self) {
        println!("{}  {}", self.start_time_local.bold(), self.activity_name);
        println!("  {:<LABEL_WIDTH$}{}", "ID:", self.activity_id.to_string().dimmed());
        println!("  {:<LABEL_WIDTH$}{}", "Type:", self.activity_type.cyan());
        println!("  {:<LABEL_WIDTH$}{}", "Distance:", fmt_dist(self.distance_meters));
        println!("  {:<LABEL_WIDTH$}{}", "Duration:", fmt_hms(self.duration_seconds));
        if let Some(pace) = self.pace_min_km() {
            println!("  {:<LABEL_WIDTH$}{pace}", "Pace:");
        }
        if let Some(hr) = self.average_hr {
            let max = self.max_hr.map(|m| format!(" (max {m:.0})")).unwrap_or_default();
            println!("  {:<LABEL_WIDTH$}{hr:.0} bpm{max}", "Avg HR:");
        }
        if let (Some(aero), Some(anaero)) = (self.aerobic_training_effect, self.anaerobic_training_effect) {
            println!("  {:<LABEL_WIDTH$}{aero:.1} aero / {anaero:.1} anaero", "TE:");
        } else if let Some(aero) = self.aerobic_training_effect {
            println!("  {:<LABEL_WIDTH$}{aero:.1} aero", "TE:");
        }
        if let Some(load) = self.activity_training_load {
            println!("  {:<LABEL_WIDTH$}{load:.0}", "Load:");
        }
        if let Some(vo2) = self.vo2_max_value {
            println!("  {:<LABEL_WIDTH$}{vo2:.0}", "VO2max:");
        }
        if let Some(gain) = self.elevation_gain_meters {
            let loss_str = self
                .elevation_loss_meters
                .map(|l| format!(" / -{l:.0}m"))
                .unwrap_or_default();
            println!("  {:<LABEL_WIDTH$}+{gain:.0}m{loss_str}", "Elevation:");
        }
        if let Some(pwr) = self.average_power {
            println!("  {:<LABEL_WIDTH$}{pwr:.0} W", "Power:");
        }
        if let Some(cad) = self.avg_running_cadence {
            println!("  {:<LABEL_WIDTH$}{cad:.0} spm", "Cadence:");
        }
        if let Some(gct) = self.avg_ground_contact_time_ms {
            println!("  {:<LABEL_WIDTH$}{gct:.0} ms", "GCT:");
        }
        if let Some(stride) = self.avg_stride_length_cm {
            println!("  {:<LABEL_WIDTH$}{stride:.0} cm", "Stride:");
        }
    }
}
