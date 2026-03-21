use crate::client::GarminClient;
use crate::error::Result;
use crate::output::{HumanReadable, Output};
use colored::Colorize;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct GearItem {
    pub uuid: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gear_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance_meters: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activities: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_begin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_distance_meters: Option<f64>,
}

fn gear_from_json(v: &serde_json::Value) -> GearItem {
    GearItem {
        uuid: v["uuid"].as_str().unwrap_or("").into(),
        display_name: v["displayName"]
            .as_str()
            .or_else(|| v["gearPk"].as_str())
            .unwrap_or("Unknown")
            .into(),
        gear_type: v["gearTypeName"]
            .as_str()
            .or_else(|| v["gearType"].as_str())
            .map(Into::into),
        brand: v["customMakeModel"]
            .as_str()
            .or_else(|| v["brand"].as_str())
            .map(Into::into),
        model: v["model"].as_str().map(Into::into),
        distance_meters: v["totalDistance"]
            .as_f64()
            .or_else(|| v["distanceInMeters"].as_f64()),
        activities: v["totalActivities"]
            .as_i64()
            .or_else(|| v["activities"].as_i64()),
        date_begin: v["dateBegin"].as_str().map(Into::into),
        max_distance_meters: v["maximumMeters"]
            .as_f64()
            .or_else(|| v["maxDistanceInMeters"].as_f64()),
    }
}

impl HumanReadable for GearItem {
    fn print_human(&self) {
        let kind = self.gear_type.as_deref().unwrap_or("GEAR");
        println!("{} [{}]", self.display_name.bold(), kind.cyan(),);
        println!("  UUID: {}", self.uuid.dimmed());
        if let Some(ref brand) = self.brand {
            print!("  Brand: {brand}");
            if let Some(ref model) = self.model {
                print!(" / {model}");
            }
            println!();
        }
        if let Some(dist) = self.distance_meters {
            println!("  Distance: {:.1} km", dist / 1000.0);
        }
        if let Some(count) = self.activities {
            println!("  Activities: {count}");
        }
        if let Some(ref date) = self.date_begin {
            println!("  Since: {date}");
        }
        if let Some(max) = self.max_distance_meters {
            println!("  Max distance alert: {:.0} km", max / 1000.0);
        }
        println!();
    }
}

pub async fn list(client: &GarminClient, output: &Output) -> Result<()> {
    let pk = client.profile_pk().await?;
    let path = format!("/gear-service/gear/filterGear?userProfilePk={pk}");
    let v: serde_json::Value = client.get_json(&path).await?;

    let items: Vec<GearItem> = v
        .as_array()
        .map(|arr| arr.iter().map(gear_from_json).collect())
        .unwrap_or_default();

    output.print_list(&items, "Gear");
    Ok(())
}

// ── Gear Stats ───────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct GearStats {
    pub uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_distance_meters: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_activities: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_duration_seconds: Option<f64>,
}

fn gear_stats_from_json(uuid: &str, v: &serde_json::Value) -> GearStats {
    GearStats {
        uuid: uuid.into(),
        total_distance_meters: v["totalDistance"]
            .as_f64()
            .or_else(|| v["distance"].as_f64()),
        total_activities: v["totalActivities"]
            .as_i64()
            .or_else(|| v["activities"].as_i64()),
        total_duration_seconds: v["totalDuration"]
            .as_f64()
            .or_else(|| v["duration"].as_f64()),
    }
}

impl HumanReadable for GearStats {
    fn print_human(&self) {
        println!("{}", "Gear Stats".bold());
        println!("  UUID: {}", self.uuid.dimmed());
        if let Some(dist) = self.total_distance_meters {
            println!("  Total distance: {:.1} km", dist / 1000.0);
        }
        if let Some(count) = self.total_activities {
            println!("  Total activities: {count}");
        }
        if let Some(dur) = self.total_duration_seconds {
            let hours = (dur / 3600.0).floor() as u32;
            let mins = ((dur % 3600.0) / 60.0).round() as u32;
            println!("  Total duration: {hours}h {mins}min");
        }
        println!();
    }
}

pub async fn stats(client: &GarminClient, output: &Output, uuid: &str) -> Result<()> {
    let path = format!("/gear-service/gear/stats/{uuid}");
    let v: serde_json::Value = client.get_json(&path).await?;
    let s = gear_stats_from_json(uuid, &v);
    output.print(&s);
    Ok(())
}

pub async fn link(
    client: &GarminClient,
    output: &Output,
    uuid: &str,
    activity_id: u64,
) -> Result<()> {
    let path = format!("/gear-service/gear/link/{uuid}/activity/{activity_id}");
    client.request(reqwest::Method::PUT, &path, None).await?;
    output.print_value(&serde_json::json!({
        "gearUUID": uuid,
        "activityId": activity_id,
        "linked": true,
    }));
    Ok(())
}
