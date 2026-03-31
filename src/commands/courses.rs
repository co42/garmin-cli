use crate::client::GarminClient;
use crate::error::Result;
use crate::output::{HumanReadable, Output};
use colored::Colorize;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct GeoPoint {
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: f64,
    pub distance: f64,
}

#[derive(Debug, Serialize)]
pub struct StartPoint {
    pub latitude: f64,
    pub longitude: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elevation: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct LatLon {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Serialize)]
pub struct BoundingBox {
    pub lower_left: LatLon,
    pub upper_right: LatLon,
}

#[derive(Debug, Serialize)]
pub struct CourseSegment {
    pub sort_order: u64,
    pub distance_meters: f64,
    pub num_points: u64,
}

#[derive(Debug, Serialize)]
pub struct Course {
    pub id: u64,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance_meters: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elevation_gain_meters: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elevation_loss_meters: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_latitude: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_longitude: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_point: Option<StartPoint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounding_box: Option<BoundingBox>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favorite: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_pace_band: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_power_guide: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_turn_detection_disabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed_meters_per_second: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elapsed_seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elevation_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_laps: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_to_segments: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cutoff_duration: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_date: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub course_segments: Vec<CourseSegment>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub geo_points: Vec<GeoPoint>,
}

/// Map the Garmin elevation source integer to a human-readable label.
fn elevation_source_label(v: &serde_json::Value) -> Option<String> {
    v["elevationSource"].as_i64().map(|code| match code {
        1 => "device".into(),
        3 => "DEM corrected".into(),
        other => format!("unknown ({other})"),
    })
}

/// Map `activityTypePk` (detail endpoint) to a type key string.
/// The detail endpoint only returns an integer, not the nested activityType object.
/// Common values: 1=running, 2=cycling, 6=trail_running, 5=swimming, etc.
fn activity_type_from_pk(pk: i64) -> &'static str {
    match pk {
        1 => "running",
        2 => "cycling",
        3 => "other",
        4 => "fitness_equipment",
        5 => "swimming",
        6 => "trail_running",
        9 => "walking",
        10 => "hiking",
        11 => "strength_training",
        13 => "open_water_swimming",
        15 => "whitewater_kayaking_rafting",
        17 => "mountaineering",
        26 => "indoor_cycling",
        29 => "elliptical",
        37 => "indoor_rowing",
        46 => "yoga",
        _ => "unknown",
    }
}

fn geo_points_from_json(v: &serde_json::Value) -> Vec<GeoPoint> {
    let Some(arr) = v["geoPoints"].as_array() else {
        return vec![];
    };
    arr.iter()
        .filter_map(|p| {
            Some(GeoPoint {
                latitude: p["latitude"].as_f64()?,
                longitude: p["longitude"].as_f64()?,
                elevation: p["elevation"].as_f64().unwrap_or(0.0),
                distance: p["distance"].as_f64().unwrap_or(0.0),
            })
        })
        .collect()
}

fn start_point_from_json(v: &serde_json::Value) -> Option<StartPoint> {
    let sp = &v["startPoint"];
    let lat = sp["latitude"].as_f64()?;
    let lon = sp["longitude"].as_f64()?;
    Some(StartPoint {
        latitude: lat,
        longitude: lon,
        elevation: sp["elevation"].as_f64(),
    })
}

fn bounding_box_from_json(v: &serde_json::Value) -> Option<BoundingBox> {
    let bb = &v["boundingBox"];
    Some(BoundingBox {
        lower_left: LatLon {
            latitude: bb["lowerLeft"]["latitude"].as_f64()?,
            longitude: bb["lowerLeft"]["longitude"].as_f64()?,
        },
        upper_right: LatLon {
            latitude: bb["upperRight"]["latitude"].as_f64()?,
            longitude: bb["upperRight"]["longitude"].as_f64()?,
        },
    })
}

fn course_segments_from_json(v: &serde_json::Value) -> Vec<CourseSegment> {
    let Some(arr) = v["courseLines"].as_array() else {
        return vec![];
    };
    arr.iter()
        .filter_map(|cl| {
            Some(CourseSegment {
                sort_order: cl["sortOrder"].as_u64()?,
                distance_meters: cl["distanceInMeters"].as_f64().unwrap_or(0.0),
                num_points: cl["numberOfPoints"].as_u64().unwrap_or(0),
            })
        })
        .collect()
}

fn course_from_list(v: &serde_json::Value) -> Course {
    Course {
        id: v["courseId"].as_u64().unwrap_or(0),
        name: v["courseName"].as_str().unwrap_or("Untitled").into(),
        description: v["courseDescription"].as_str().map(Into::into),
        activity_type: v["activityType"]["typeKey"].as_str().map(Into::into),
        distance_meters: v["distanceInMeters"].as_f64(),
        elevation_gain_meters: v["elevationGainInMeters"].as_f64(),
        elevation_loss_meters: v["elevationLossInMeters"].as_f64(),
        start_latitude: v["startLatitude"].as_f64(),
        start_longitude: v["startLongitude"].as_f64(),
        start_point: None,
        bounding_box: None,
        favorite: v["favorite"].as_bool(),
        has_pace_band: v["hasPaceBand"].as_bool(),
        has_power_guide: v["hasPowerGuide"].as_bool(),
        has_turn_detection_disabled: v["hasTurnDetectionDisabled"].as_bool(),
        public: v["public"].as_bool(),
        speed_meters_per_second: v["speedInMetersPerSecond"].as_f64().filter(|&s| s > 0.0),
        elapsed_seconds: v["elapsedSeconds"].as_f64(),
        elevation_source: elevation_source_label(v),
        include_laps: None,
        matched_to_segments: None,
        start_note: v["startNote"].as_str().map(Into::into),
        finish_note: v["finishNote"].as_str().map(Into::into),
        cutoff_duration: v["cutoffDuration"].as_f64(),
        created_date: v["createdDateFormatted"].as_str().map(Into::into),
        update_date: v["updatedDateFormatted"].as_str().map(Into::into),
        course_segments: vec![],
        geo_points: vec![],
    }
}

fn course_from_detail(v: &serde_json::Value) -> Course {
    // Detail endpoint uses slightly different field names than list.
    // It has activityTypePk (int) instead of activityType (nested object).
    let activity_type = v["activityType"]["typeKey"]
        .as_str()
        .map(String::from)
        .or_else(|| {
            v["activityTypePk"]
                .as_i64()
                .map(|pk| activity_type_from_pk(pk).into())
        });

    Course {
        id: v["courseId"].as_u64().unwrap_or(0),
        name: v["courseName"].as_str().unwrap_or("Untitled").into(),
        description: v["description"]
            .as_str()
            .or_else(|| v["courseDescription"].as_str())
            .map(Into::into),
        activity_type,
        distance_meters: v["distanceMeter"]
            .as_f64()
            .or_else(|| v["distanceInMeters"].as_f64()),
        elevation_gain_meters: v["elevationGainMeter"]
            .as_f64()
            .or_else(|| v["elevationGainInMeters"].as_f64()),
        elevation_loss_meters: v["elevationLossMeter"]
            .as_f64()
            .or_else(|| v["elevationLossInMeters"].as_f64()),
        start_latitude: v["startPoint"]["latitude"]
            .as_f64()
            .or_else(|| v["startLatitude"].as_f64()),
        start_longitude: v["startPoint"]["longitude"]
            .as_f64()
            .or_else(|| v["startLongitude"].as_f64()),
        start_point: start_point_from_json(v),
        bounding_box: bounding_box_from_json(v),
        favorite: v["favorite"].as_bool(),
        has_pace_band: v["hasPaceBand"].as_bool(),
        has_power_guide: v["hasPowerGuide"].as_bool(),
        has_turn_detection_disabled: v["hasTurnDetectionDisabled"].as_bool(),
        public: None, // not returned by detail endpoint
        speed_meters_per_second: v["speedMeterPerSecond"]
            .as_f64()
            .or_else(|| v["speedInMetersPerSecond"].as_f64())
            .filter(|&s| s > 0.0),
        elapsed_seconds: v["elapsedSeconds"].as_f64(),
        elevation_source: elevation_source_label(v),
        include_laps: v["includeLaps"].as_bool(),
        matched_to_segments: v["matchedToSegments"].as_bool(),
        start_note: v["startNote"].as_str().map(Into::into),
        finish_note: v["finishNote"].as_str().map(Into::into),
        cutoff_duration: v["cutoffDuration"].as_f64(),
        created_date: v["createDate"].as_str().map(Into::into),
        update_date: v["updateDate"].as_str().map(Into::into),
        course_segments: course_segments_from_json(v),
        geo_points: geo_points_from_json(v),
    }
}

fn format_duration(seconds: f64) -> String {
    let total = seconds.round() as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

impl HumanReadable for Course {
    fn print_human(&self) {
        let kind = self.activity_type.as_deref().unwrap_or("unknown");
        let mut tags = String::new();
        if self.favorite == Some(true) {
            tags.push_str(" *");
        }
        if self.public == Some(true) {
            tags.push_str(" (public)");
        }
        println!("{}{} [{}]", self.name.bold(), tags, kind.cyan());
        println!("  ID: {}", self.id);
        if let Some(ref desc) = self.description {
            println!("  Description: {desc}");
        }
        if let Some(dist) = self.distance_meters {
            let duration_str = self
                .elapsed_seconds
                .map(|s| format!(" in {}", format_duration(s)))
                .unwrap_or_default();
            println!("  Distance: {:.2} km{duration_str}", dist / 1000.0);
        }
        if let Some(speed) = self.speed_meters_per_second {
            println!("  Speed: {:.2} km/h", speed * 3.6);
        }
        if let Some(gain) = self.elevation_gain_meters {
            let loss_str = self
                .elevation_loss_meters
                .map(|l| format!(" / -{l:.0}m"))
                .unwrap_or_default();
            println!("  Elevation: +{gain:.0}m{loss_str}");
        }
        if let Some(ref source) = self.elevation_source {
            println!("  Elevation source: {source}");
        }
        if let Some(ref date) = self.created_date {
            let update_str = self
                .update_date
                .as_ref()
                .map(|u| format!(" (updated {u})"))
                .unwrap_or_default();
            println!("  Created: {date}{update_str}");
        }
        if let (Some(lat), Some(lon)) = (self.start_latitude, self.start_longitude) {
            let elev_str = self
                .start_point
                .as_ref()
                .and_then(|sp| sp.elevation)
                .map(|e| format!(" ({e:.0}m)"))
                .unwrap_or_default();
            println!("  Start: {lat:.5}, {lon:.5}{elev_str}");
        }
        if let Some(ref bb) = self.bounding_box {
            println!(
                "  Bounds: ({:.5}, {:.5}) - ({:.5}, {:.5})",
                bb.lower_left.latitude,
                bb.lower_left.longitude,
                bb.upper_right.latitude,
                bb.upper_right.longitude
            );
        }

        // Feature flags line
        {
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
                println!("  Features: {}", flags.join(", "));
            }
        }

        if let Some(ref note) = self.start_note {
            println!("  Start note: {note}");
        }
        if let Some(ref note) = self.finish_note {
            println!("  Finish note: {note}");
        }
        if let Some(cutoff) = self.cutoff_duration {
            println!("  Cutoff: {}", format_duration(cutoff));
        }
        if !self.course_segments.is_empty() {
            let total_points: u64 = self.course_segments.iter().map(|s| s.num_points).sum();
            println!(
                "  Segments: {} ({} points)",
                self.course_segments.len(),
                total_points
            );
        }
        if !self.geo_points.is_empty() {
            println!("  Track: {} points", self.geo_points.len());
        }
        println!();
    }
}

pub async fn list(client: &GarminClient, output: &Output) -> Result<()> {
    let v: serde_json::Value = client.get_json("/course-service/course").await?;

    let courses: Vec<Course> = v
        .as_array()
        .map(|arr| arr.iter().map(course_from_list).collect())
        .unwrap_or_default();

    output.print_list(&courses, "Courses");
    Ok(())
}

pub async fn get(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    let path = format!("/course-service/course/{id}");
    let v: serde_json::Value = client.get_json(&path).await?;
    let course = course_from_detail(&v);
    output.print(&course);
    Ok(())
}
