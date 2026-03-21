use crate::client::GarminClient;
use crate::error::Result;
use crate::output::{HumanReadable, Output};
use colored::Colorize;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Course {
    pub id: u64,
    pub name: String,
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
    pub created_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_pace_band: Option<bool>,
}

fn course_from_list(v: &serde_json::Value) -> Course {
    Course {
        id: v["courseId"].as_u64().unwrap_or(0),
        name: v["courseName"].as_str().unwrap_or("Untitled").into(),
        activity_type: v["activityType"]["typeKey"].as_str().map(Into::into),
        distance_meters: v["distanceInMeters"].as_f64(),
        elevation_gain_meters: v["elevationGainInMeters"].as_f64(),
        elevation_loss_meters: v["elevationLossInMeters"].as_f64(),
        start_latitude: v["startLatitude"].as_f64(),
        start_longitude: v["startLongitude"].as_f64(),
        created_date: v["createdDateFormatted"].as_str().map(Into::into),
        has_pace_band: v["hasPaceBand"].as_bool(),
    }
}

fn course_from_detail(v: &serde_json::Value) -> Course {
    // Detail endpoint uses slightly different field names
    Course {
        id: v["courseId"].as_u64().unwrap_or(0),
        name: v["courseName"].as_str().unwrap_or("Untitled").into(),
        activity_type: v["activityType"]["typeKey"].as_str().map(Into::into),
        distance_meters: v["distanceMeter"]
            .as_f64()
            .or_else(|| v["distanceInMeters"].as_f64()),
        elevation_gain_meters: v["elevationGainMeter"]
            .as_f64()
            .or_else(|| v["elevationGainInMeters"].as_f64()),
        elevation_loss_meters: v["elevationLossMeter"]
            .as_f64()
            .or_else(|| v["elevationLossInMeters"].as_f64()),
        start_latitude: v["startPoint"]["lat"]
            .as_f64()
            .or_else(|| v["startLatitude"].as_f64()),
        start_longitude: v["startPoint"]["lon"]
            .as_f64()
            .or_else(|| v["startLongitude"].as_f64()),
        created_date: v["createdDateFormatted"].as_str().map(Into::into),
        has_pace_band: v["hasPaceBand"].as_bool(),
    }
}

impl HumanReadable for Course {
    fn print_human(&self) {
        let kind = self.activity_type.as_deref().unwrap_or("unknown");
        println!("{} [{}]", self.name.bold(), kind.cyan(),);
        println!("  ID: {}", self.id);
        if let Some(dist) = self.distance_meters {
            println!("  Distance: {:.2} km", dist / 1000.0);
        }
        if let Some(gain) = self.elevation_gain_meters {
            let loss_str = self
                .elevation_loss_meters
                .map(|l| format!(" / -{l:.0}m"))
                .unwrap_or_default();
            println!("  Elevation: +{gain:.0}m{loss_str}");
        }
        if let Some(ref date) = self.created_date {
            println!("  Created: {date}");
        }
        if let (Some(lat), Some(lon)) = (self.start_latitude, self.start_longitude) {
            println!("  Start: {lat:.5}, {lon:.5}");
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
