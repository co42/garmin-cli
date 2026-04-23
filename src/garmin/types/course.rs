use super::helpers::{fmt_hms, untitled};
use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Debug, Serialize, Deserialize)]
pub struct GeoPoint {
    pub latitude: f64,
    pub longitude: f64,
    #[serde(rename(deserialize = "elevation"), default)]
    pub elevation_meters: f64,
    #[serde(rename(deserialize = "distance"), default)]
    pub distance_meters: f64,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct StartPoint {
    pub latitude: f64,
    pub longitude: f64,
    #[serde(rename(deserialize = "elevation"))]
    pub elevation_meters: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LatLon {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct BoundingBox {
    pub lower_left: LatLon,
    pub upper_right: LatLon,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CourseSegment {
    #[serde(default)]
    pub sort_order: u64,
    #[serde(rename(deserialize = "distanceInMeters"), default)]
    pub distance_meters: f64,
    #[serde(default)]
    pub number_of_points: u64,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Course {
    #[serde(default)]
    pub course_id: u64,
    #[serde(default = "untitled")]
    pub course_name: String,
    #[serde(alias = "description")]
    pub course_description: Option<String>,
    /// Populated only for the list endpoint; detail uses `activityTypePk` (int).
    /// TODO: the detail endpoint returns a numeric `activityTypePk` instead of
    /// the nested `activityType.typeKey`. Not yet mapped — `get` may show None.
    pub activity_type: Option<ActivityTypeRef>,
    #[serde(rename(deserialize = "distanceInMeters"), alias = "distanceMeter")]
    pub distance_meters: Option<f64>,
    #[serde(rename(deserialize = "elevationGainInMeters"), alias = "elevationGainMeter")]
    pub elevation_gain_meters: Option<f64>,
    #[serde(rename(deserialize = "elevationLossInMeters"), alias = "elevationLossMeter")]
    pub elevation_loss_meters: Option<f64>,
    pub start_latitude: Option<f64>,
    pub start_longitude: Option<f64>,
    pub start_point: Option<StartPoint>,
    pub bounding_box: Option<BoundingBox>,
    pub favorite: Option<bool>,
    pub has_pace_band: Option<bool>,
    pub has_power_guide: Option<bool>,
    pub has_turn_detection_disabled: Option<bool>,
    pub public: Option<bool>,
    #[serde(rename(deserialize = "speedInMetersPerSecond"), alias = "speedMeterPerSecond")]
    pub speed_mps: Option<f64>,
    pub elapsed_seconds: Option<f64>,
    /// Encoded as int by the API: 1=device, 3=DEM corrected.
    pub elevation_source: Option<i64>,
    pub include_laps: Option<bool>,
    pub matched_to_segments: Option<bool>,
    pub start_note: Option<String>,
    pub finish_note: Option<String>,
    #[serde(rename(deserialize = "cutoffDuration"))]
    pub cutoff_duration_seconds: Option<f64>,
    /// API: `createdDateFormatted` — "Formatted" suffix is an implementation
    /// detail; the alternative endpoint uses plain `createDate`.
    #[serde(rename(deserialize = "createdDateFormatted"), alias = "createDate")]
    pub created_date: Option<String>,
    #[serde(rename(deserialize = "updatedDateFormatted"), alias = "updateDate")]
    pub updated_date: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub course_lines: Vec<CourseSegment>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub geo_points: Vec<GeoPoint>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ActivityTypeRef {
    pub type_key: String,
}

impl Course {
    fn elevation_source_label(&self) -> Option<String> {
        self.elevation_source.map(|code| match code {
            1 => "device".into(),
            3 => "DEM corrected".into(),
            other => format!("unknown ({other})"),
        })
    }
}

impl HumanReadable for Course {
    fn print_human(&self) {
        let mut tags = String::new();
        if self.favorite == Some(true) {
            tags.push_str(" *");
        }
        if self.public == Some(true) {
            tags.push_str(" (public)");
        }
        println!("{}{}", self.course_name.bold(), tags);
        println!("  {:<LABEL_WIDTH$}{}", "ID:", self.course_id);
        if let Some(ref kind) = self.activity_type {
            println!("  {:<LABEL_WIDTH$}{}", "Type:", kind.type_key.cyan());
        }
        if let Some(ref desc) = self.course_description {
            println!("  {:<LABEL_WIDTH$}{desc}", "Description:");
        }
        if let Some(dist) = self.distance_meters {
            let duration_str = self
                .elapsed_seconds
                .map(|s| format!(" in {}", fmt_hms(s)))
                .unwrap_or_default();
            println!("  {:<LABEL_WIDTH$}{:.2} km{duration_str}", "Distance:", dist / 1000.0);
        }
        if let Some(speed) = self.speed_mps.filter(|&s| s > 0.0) {
            println!("  {:<LABEL_WIDTH$}{:.2} km/h", "Speed:", speed * 3.6);
        }
        if let Some(gain) = self.elevation_gain_meters {
            let loss_str = self
                .elevation_loss_meters
                .map(|l| format!(" / -{l:.0}m"))
                .unwrap_or_default();
            println!("  {:<LABEL_WIDTH$}+{gain:.0}m{loss_str}", "Elevation:");
        }
        if let Some(source) = self.elevation_source_label() {
            println!("  {:<LABEL_WIDTH$}{source}", "Elev source:");
        }
        if let Some(ref date) = self.created_date {
            let update_str = self
                .updated_date
                .as_ref()
                .map(|u| format!(" (updated {u})"))
                .unwrap_or_default();
            println!("  {:<LABEL_WIDTH$}{date}{update_str}", "Created:");
        }
        if let (Some(lat), Some(lon)) = (self.start_latitude, self.start_longitude) {
            let elev_str = self
                .start_point
                .as_ref()
                .and_then(|sp| sp.elevation_meters)
                .map(|e| format!(" ({e:.0}m)"))
                .unwrap_or_default();
            println!("  {:<LABEL_WIDTH$}{lat:.5}, {lon:.5}{elev_str}", "Start:");
        }
        if let Some(ref bb) = self.bounding_box {
            println!(
                "  {:<LABEL_WIDTH$}({:.5}, {:.5}) \u{2013} ({:.5}, {:.5})",
                "Bounds:",
                bb.lower_left.latitude,
                bb.lower_left.longitude,
                bb.upper_right.latitude,
                bb.upper_right.longitude
            );
        }
        let mut flags = Vec::new();
        if self.has_pace_band == Some(true) {
            flags.push("pace band");
        }
        if self.has_power_guide == Some(true) {
            flags.push("power guide");
        }
        if self.include_laps == Some(true) {
            flags.push("laps");
        }
        if self.matched_to_segments == Some(true) {
            flags.push("matched to segments");
        }
        if self.has_turn_detection_disabled == Some(true) {
            flags.push("turn detection disabled");
        }
        if !flags.is_empty() {
            println!("  {:<LABEL_WIDTH$}{}", "Features:", flags.join(", "));
        }
        if let Some(ref note) = self.start_note {
            println!("  {:<LABEL_WIDTH$}{note}", "Start note:");
        }
        if let Some(ref note) = self.finish_note {
            println!("  {:<LABEL_WIDTH$}{note}", "Finish note:");
        }
        if let Some(cutoff) = self.cutoff_duration_seconds {
            println!("  {:<LABEL_WIDTH$}{}", "Cutoff:", fmt_hms(cutoff));
        }
        if !self.course_lines.is_empty() {
            let total_points: u64 = self.course_lines.iter().map(|s| s.number_of_points).sum();
            println!(
                "  {:<LABEL_WIDTH$}{} ({total_points} points)",
                "Segments:",
                self.course_lines.len()
            );
        }
        if !self.geo_points.is_empty() {
            println!("  {:<LABEL_WIDTH$}{} points", "Track:", self.geo_points.len());
        }
    }
}
