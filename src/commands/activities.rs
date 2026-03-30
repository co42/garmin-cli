use crate::client::GarminClient;
use crate::error::Result;
use crate::output::{HumanReadable, Output};
use colored::Colorize;
use serde::Serialize;

/// Format pace as "MM:SS /km" from distance_meters and duration_seconds.
fn compute_pace(distance_meters: Option<f64>, duration_seconds: f64) -> Option<String> {
    let dist = distance_meters?;
    if dist <= 0.0 || duration_seconds <= 0.0 {
        return None;
    }
    let pace_secs_per_km = (duration_seconds / (dist / 1000.0)).round() as u32;
    let mins = pace_secs_per_km / 60;
    let secs = pace_secs_per_km % 60;
    Some(format!("{mins}:{secs:02} /km"))
}

#[derive(Debug, Serialize)]
pub struct ActivitySummary {
    pub id: u64,
    pub name: String,
    pub activity_type: String,
    pub start_time: String,
    pub duration_seconds: f64,
    pub distance_meters: Option<f64>,
    pub calories: Option<f64>,
    pub avg_hr: Option<f64>,
    pub max_hr: Option<f64>,
    pub moving_duration_seconds: Option<f64>,
    pub pace_min_km: Option<String>,

    // Training Effect & Load
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aerobic_training_effect: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anaerobic_training_effect: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aerobic_training_effect_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anaerobic_training_effect_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub training_effect_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity_training_load: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impact_load: Option<f64>,

    // Performance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vo2max_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_power: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub norm_power: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_power: Option<f64>,

    // Running dynamics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_running_cadence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_stride_length: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_ground_contact_time: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_vertical_oscillation: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_vertical_ratio: Option<f64>,

    // Elevation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elevation_gain: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elevation_loss: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_grade_adjusted_speed: Option<f64>,

    // Splits
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fastest_split_1000: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fastest_split_1609: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fastest_split_5000: Option<f64>,

    // Misc
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moderate_intensity_minutes: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vigorous_intensity_minutes: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub difference_body_battery: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_latitude: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_longitude: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workout_id: Option<u64>,

    // HR zones
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hr_time_in_zone_1: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hr_time_in_zone_2: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hr_time_in_zone_3: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hr_time_in_zone_4: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hr_time_in_zone_5: Option<f64>,

    // Power zones
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power_time_in_zone_1: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power_time_in_zone_2: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power_time_in_zone_3: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power_time_in_zone_4: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power_time_in_zone_5: Option<f64>,
}

impl HumanReadable for ActivitySummary {
    fn print_human(&self) {
        let duration_min = self.duration_seconds / 60.0;
        let dist = self
            .distance_meters
            .map(|d| format!("{:.2} km", d / 1000.0))
            .unwrap_or_else(|| "\u{2014}".into());

        println!(
            "{} {} [{}]",
            self.start_time.dimmed(),
            self.name.bold(),
            self.activity_type.cyan()
        );
        println!(
            "  ID: {}  Duration: {:.0}min  Distance: {}",
            self.id, duration_min, dist
        );
        if let Some(hr) = self.avg_hr {
            print!("  Avg HR: {:.0}", hr);
            if let Some(max) = self.max_hr {
                print!("  Max HR: {:.0}", max);
            }
            println!();
        }
        if let Some(ref pace) = self.pace_min_km {
            println!("  Pace: {pace}");
        }

        // Training Effect / Load / VO2max line
        {
            let mut parts = Vec::new();
            if let (Some(aero), Some(anaero)) =
                (self.aerobic_training_effect, self.anaerobic_training_effect)
            {
                parts.push(format!("TE: {aero:.1} aero / {anaero:.1} anaero"));
            } else if let Some(aero) = self.aerobic_training_effect {
                parts.push(format!("TE: {aero:.1} aero"));
            }
            if let Some(load) = self.activity_training_load {
                parts.push(format!("Load: {load:.0}"));
            }
            if let Some(vo2) = self.vo2max_value {
                parts.push(format!("VO2max: {vo2:.0}"));
            }
            if !parts.is_empty() {
                println!("  {}", parts.join("  "));
            }
        }

        // Elevation / Power line
        {
            let mut parts = Vec::new();
            if let Some(gain) = self.elevation_gain {
                let loss_str = self
                    .elevation_loss
                    .map(|l| format!(" / -{l:.0}m"))
                    .unwrap_or_default();
                parts.push(format!("Elev: +{gain:.0}m{loss_str}"));
            }
            if let Some(pwr) = self.avg_power {
                parts.push(format!("Power: {pwr:.0}W"));
            }
            if !parts.is_empty() {
                println!("  {}", parts.join("  "));
            }
        }

        // Running dynamics line
        {
            let mut parts = Vec::new();
            if let Some(cad) = self.avg_running_cadence {
                parts.push(format!("Cadence: {cad:.0} spm"));
            }
            if let Some(gct) = self.avg_ground_contact_time {
                parts.push(format!("GCT: {gct:.0}ms"));
            }
            if let Some(stride) = self.avg_stride_length {
                parts.push(format!("Stride: {stride:.0}cm"));
            }
            if !parts.is_empty() {
                println!("  {}", parts.join("  "));
            }
        }

        println!();
    }
}

fn normalize_timestamp(s: &str) -> String {
    // "2026-03-28 09:56:14" → "2026-03-28T09:56:14"
    s.replacen(' ', "T", 1).trim_end_matches(".0").to_string()
}

fn activity_from_list(a: &serde_json::Value) -> ActivitySummary {
    let duration_seconds = a["duration"].as_f64().unwrap_or(0.0);
    let distance_meters = a["distance"].as_f64();
    ActivitySummary {
        id: a["activityId"].as_u64().unwrap_or(0),
        name: a["activityName"].as_str().unwrap_or("Untitled").into(),
        activity_type: a["activityType"]["typeKey"]
            .as_str()
            .unwrap_or("unknown")
            .into(),
        start_time: a["startTimeLocal"]
            .as_str()
            .map(normalize_timestamp)
            .unwrap_or_default(),
        duration_seconds,
        distance_meters,
        calories: a["calories"].as_f64(),
        avg_hr: a["averageHR"].as_f64(),
        max_hr: a["maxHR"].as_f64(),
        moving_duration_seconds: a["movingDuration"].as_f64(),
        pace_min_km: compute_pace(distance_meters, duration_seconds),

        // Training Effect & Load
        aerobic_training_effect: a["aerobicTrainingEffect"].as_f64(),
        anaerobic_training_effect: a["anaerobicTrainingEffect"].as_f64(),
        aerobic_training_effect_message: a["aerobicTrainingEffectMessage"].as_str().map(Into::into),
        anaerobic_training_effect_message: a["anaerobicTrainingEffectMessage"]
            .as_str()
            .map(Into::into),
        training_effect_label: a["trainingEffectLabel"].as_str().map(Into::into),
        activity_training_load: a["activityTrainingLoad"].as_f64(),
        impact_load: a["impactLoad"].as_f64(),

        // Performance
        vo2max_value: a["vO2MaxValue"].as_f64(),
        avg_power: a["avgPower"].as_f64(),
        norm_power: a["normPower"].as_f64(),
        max_power: a["maxPower"].as_f64(),

        // Running dynamics
        avg_running_cadence: a["avgRunningCadenceInStepsPerMinute"].as_f64(),
        avg_stride_length: a["avgStrideLength"].as_f64(),
        avg_ground_contact_time: a["avgGroundContactTime"].as_f64(),
        avg_vertical_oscillation: a["avgVerticalOscillation"].as_f64(),
        avg_vertical_ratio: a["avgVerticalRatio"].as_f64(),

        // Elevation
        elevation_gain: a["elevationGain"].as_f64(),
        elevation_loss: a["elevationLoss"].as_f64(),
        avg_grade_adjusted_speed: a["avgGradeAdjustedSpeed"].as_f64(),

        // Splits
        fastest_split_1000: a["fastestSplit_1000"].as_f64(),
        fastest_split_1609: a["fastestSplit_1609"].as_f64(),
        fastest_split_5000: a["fastestSplit_5000"].as_f64(),

        // Misc
        moderate_intensity_minutes: a["moderateIntensityMinutes"].as_f64(),
        vigorous_intensity_minutes: a["vigorousIntensityMinutes"].as_f64(),
        difference_body_battery: a["differenceBodyBattery"].as_f64(),
        steps: a["steps"].as_u64(),
        location_name: a["locationName"].as_str().map(Into::into),
        start_latitude: a["startLatitude"].as_f64(),
        start_longitude: a["startLongitude"].as_f64(),
        workout_id: a["workoutId"].as_u64(),

        // HR zones
        hr_time_in_zone_1: a["hrTimeInZone_1"].as_f64(),
        hr_time_in_zone_2: a["hrTimeInZone_2"].as_f64(),
        hr_time_in_zone_3: a["hrTimeInZone_3"].as_f64(),
        hr_time_in_zone_4: a["hrTimeInZone_4"].as_f64(),
        hr_time_in_zone_5: a["hrTimeInZone_5"].as_f64(),

        // Power zones
        power_time_in_zone_1: a["powerTimeInZone_1"].as_f64(),
        power_time_in_zone_2: a["powerTimeInZone_2"].as_f64(),
        power_time_in_zone_3: a["powerTimeInZone_3"].as_f64(),
        power_time_in_zone_4: a["powerTimeInZone_4"].as_f64(),
        power_time_in_zone_5: a["powerTimeInZone_5"].as_f64(),
    }
}

fn activity_from_detail(id: u64, v: &serde_json::Value) -> ActivitySummary {
    let s = &v["summaryDTO"];
    let duration_seconds = s["duration"].as_f64().unwrap_or(0.0);
    let distance_meters = s["distance"].as_f64();
    ActivitySummary {
        id,
        name: v["activityName"].as_str().unwrap_or("Untitled").into(),
        activity_type: v["activityTypeDTO"]["typeKey"]
            .as_str()
            .unwrap_or("unknown")
            .into(),
        start_time: s["startTimeLocal"]
            .as_str()
            .map(normalize_timestamp)
            .unwrap_or_default(),
        duration_seconds,
        distance_meters,
        calories: s["calories"].as_f64(),
        avg_hr: s["averageHR"].as_f64(),
        max_hr: s["maxHR"].as_f64(),
        moving_duration_seconds: s["movingDuration"].as_f64(),
        pace_min_km: compute_pace(distance_meters, duration_seconds),

        // Training Effect & Load
        // Note: summaryDTO uses "trainingEffect" for aerobic (not "aerobicTrainingEffect")
        aerobic_training_effect: s["trainingEffect"].as_f64(),
        anaerobic_training_effect: s["anaerobicTrainingEffect"].as_f64(),
        aerobic_training_effect_message: s["aerobicTrainingEffectMessage"].as_str().map(Into::into),
        anaerobic_training_effect_message: s["anaerobicTrainingEffectMessage"]
            .as_str()
            .map(Into::into),
        training_effect_label: s["trainingEffectLabel"].as_str().map(Into::into),
        activity_training_load: s["activityTrainingLoad"].as_f64(),
        impact_load: s["impactLoad"].as_f64(),

        // Performance
        vo2max_value: s["vO2MaxValue"].as_f64(),
        // summaryDTO uses "averagePower" / "normalizedPower" / "maxPower"
        avg_power: s["averagePower"].as_f64(),
        norm_power: s["normalizedPower"].as_f64(),
        max_power: s["maxPower"].as_f64(),

        // Running dynamics
        // summaryDTO uses "averageRunCadence" not "avgRunningCadenceInStepsPerMinute"
        avg_running_cadence: s["averageRunCadence"].as_f64(),
        avg_stride_length: s["strideLength"].as_f64(),
        avg_ground_contact_time: s["groundContactTime"].as_f64(),
        avg_vertical_oscillation: s["verticalOscillation"].as_f64(),
        avg_vertical_ratio: s["verticalRatio"].as_f64(),

        // Elevation
        elevation_gain: s["elevationGain"].as_f64(),
        elevation_loss: s["elevationLoss"].as_f64(),
        avg_grade_adjusted_speed: s["avgGradeAdjustedSpeed"].as_f64(),

        // Splits -- not available from detail endpoint
        fastest_split_1000: None,
        fastest_split_1609: None,
        fastest_split_5000: None,

        // Misc
        moderate_intensity_minutes: s["moderateIntensityMinutes"].as_f64(),
        vigorous_intensity_minutes: s["vigorousIntensityMinutes"].as_f64(),
        difference_body_battery: s["differenceBodyBattery"].as_f64(),
        steps: s["steps"].as_u64(),
        location_name: None,
        start_latitude: s["startLatitude"].as_f64(),
        start_longitude: s["startLongitude"].as_f64(),
        workout_id: v["workoutId"].as_u64(),

        // HR zones -- not available from detail endpoint
        hr_time_in_zone_1: None,
        hr_time_in_zone_2: None,
        hr_time_in_zone_3: None,
        hr_time_in_zone_4: None,
        hr_time_in_zone_5: None,

        // Power zones -- not available from detail endpoint
        power_time_in_zone_1: None,
        power_time_in_zone_2: None,
        power_time_in_zone_3: None,
        power_time_in_zone_4: None,
        power_time_in_zone_5: None,
    }
}

pub async fn list(
    client: &GarminClient,
    output: &Output,
    limit: u32,
    start: u32,
    activity_type: Option<&str>,
    after: Option<&str>,
    before: Option<&str>,
) -> Result<()> {
    // If filtering by date, fetch more to compensate for client-side filtering.
    let has_date_filter = after.is_some() || before.is_some();
    let fetch_limit = if has_date_filter {
        (limit * 3).max(100)
    } else {
        limit
    };

    let mut path = format!(
        "/activitylist-service/activities/search/activities?limit={fetch_limit}&start={start}"
    );
    if let Some(t) = activity_type {
        path.push_str(&format!("&activityType={}", urlencoding::encode(t)));
    }
    let activities: Vec<serde_json::Value> = client.get_json(&path).await?;

    let after_date = after.map(|s| s.to_string());
    let before_date = before.map(|s| s.to_string());

    let mut summaries: Vec<ActivitySummary> = activities
        .iter()
        .map(activity_from_list)
        .filter(|a| {
            // start_time is like "2025-03-01 08:30:00"
            let date_part = &a.start_time[..a.start_time.len().min(10)];
            if let Some(ref after_d) = after_date
                && date_part <= after_d.as_str()
            {
                return false;
            }
            if let Some(ref before_d) = before_date
                && date_part >= before_d.as_str()
            {
                return false;
            }
            true
        })
        .collect();

    if has_date_filter {
        summaries.truncate(limit as usize);
    }

    output.print_list(
        &summaries,
        &format!("Activities ({} results)", summaries.len()),
    );
    Ok(())
}

pub async fn get(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    // Fetch both detail and list views to get a complete superset of fields.
    // Detail has summaryDTO (running dynamics, etc.), list has splits/zones/location.
    let detail_path = format!("/activity-service/activity/{id}");
    let list_path = format!(
        "/activitylist-service/activities/search/activities?limit=1&start=0&activityId={id}"
    );
    let (detail, list_result) = tokio::join!(
        client.get_json::<serde_json::Value>(&detail_path),
        client.get_json::<serde_json::Value>(&list_path),
    );
    let detail = detail?;
    let mut summary = activity_from_detail(id, &detail);

    // Merge list-only fields if the list call succeeded
    if let Ok(list_val) = list_result
        && let Some(a) = list_val.as_array().and_then(|arr| arr.first())
    {
        if summary.vo2max_value.is_none() {
            summary.vo2max_value = a["vO2MaxValue"].as_f64();
        }
        if summary.location_name.is_none() {
            summary.location_name = a["locationName"].as_str().map(Into::into);
        }
        if summary.fastest_split_1000.is_none() {
            summary.fastest_split_1000 = a["fastestSplit_1000"].as_f64();
        }
        if summary.fastest_split_1609.is_none() {
            summary.fastest_split_1609 = a["fastestSplit_1609"].as_f64();
        }
        if summary.fastest_split_5000.is_none() {
            summary.fastest_split_5000 = a["fastestSplit_5000"].as_f64();
        }
        if summary.hr_time_in_zone_1.is_none() {
            summary.hr_time_in_zone_1 = a["hrTimeInZone_1"].as_f64();
            summary.hr_time_in_zone_2 = a["hrTimeInZone_2"].as_f64();
            summary.hr_time_in_zone_3 = a["hrTimeInZone_3"].as_f64();
            summary.hr_time_in_zone_4 = a["hrTimeInZone_4"].as_f64();
            summary.hr_time_in_zone_5 = a["hrTimeInZone_5"].as_f64();
        }
        if summary.power_time_in_zone_1.is_none() {
            summary.power_time_in_zone_1 = a["powerTimeInZone_1"].as_f64();
            summary.power_time_in_zone_2 = a["powerTimeInZone_2"].as_f64();
            summary.power_time_in_zone_3 = a["powerTimeInZone_3"].as_f64();
            summary.power_time_in_zone_4 = a["powerTimeInZone_4"].as_f64();
            summary.power_time_in_zone_5 = a["powerTimeInZone_5"].as_f64();
        }
    }

    output.print(&summary);
    Ok(())
}

pub async fn details(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    let path = format!("/activity-service/activity/{id}/details");
    let v: serde_json::Value = client.get_json(&path).await?;
    output.print_value(&v);
    Ok(())
}

// ── HR Zones ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct HrZone {
    pub zone: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_hr: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seconds_in_zone: Option<f64>,
}

fn hr_zones_from_json(v: &serde_json::Value) -> Vec<HrZone> {
    let arr = v.as_array();
    let Some(items) = arr else {
        return vec![];
    };
    items
        .iter()
        .map(|z| HrZone {
            zone: z["zoneNumber"].as_i64().unwrap_or(0),
            min_hr: z["zoneLowBoundary"].as_i64(),
            seconds_in_zone: z["secsInZone"].as_f64(),
        })
        .collect()
}

impl HumanReadable for HrZone {
    fn print_human(&self) {
        let hr_label = self
            .min_hr
            .map(|h| format!("{h}+ bpm"))
            .unwrap_or_else(|| "-".into());
        let time = self
            .seconds_in_zone
            .map(|s| {
                let m = (s / 60.0).floor() as u32;
                let sec = (s % 60.0).round() as u32;
                format!("{m}:{sec:02}")
            })
            .unwrap_or_else(|| "-".into());
        println!(
            "  Zone {}  {:>10}  {}",
            format!("{}", self.zone).cyan(),
            hr_label,
            time.dimmed(),
        );
    }
}

pub async fn hr_zones(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    let path = format!("/activity-service/activity/{id}/hrTimeInZones");
    let v: serde_json::Value = client.get_json(&path).await?;
    let zones = hr_zones_from_json(&v);
    output.print_list(&zones, "HR Zones");
    Ok(())
}

// ── Splits ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ActivitySplit {
    pub split: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance_meters: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moving_duration_seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_hr: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_hr: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_power: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub norm_power: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_cadence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_stride_length: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elevation_gain: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elevation_loss: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_ground_contact_time: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_vertical_oscillation: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_vertical_ratio: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calories: Option<f64>,
}

fn splits_from_json(v: &serde_json::Value) -> Vec<ActivitySplit> {
    let arr = v["lapDTOs"].as_array().or_else(|| v.as_array());
    let Some(items) = arr else {
        return vec![];
    };
    items
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let distance = s["distance"].as_f64();
            let duration = s["duration"].as_f64();
            let pace = duration.and_then(|d| compute_pace(distance, d));
            ActivitySplit {
                split: (i + 1) as i64,
                distance_meters: distance,
                duration_seconds: duration,
                moving_duration_seconds: s["movingDuration"].as_f64(),
                pace,
                avg_hr: s["averageHR"].as_f64(),
                max_hr: s["maxHR"].as_f64(),
                avg_power: s["averagePower"].as_f64(),
                norm_power: s["normalizedPower"].as_f64(),
                avg_cadence: s["averageRunCadence"].as_f64(),
                avg_stride_length: s["strideLength"].as_f64(),
                elevation_gain: s["elevationGain"].as_f64(),
                elevation_loss: s["elevationLoss"].as_f64(),
                avg_ground_contact_time: s["groundContactTime"].as_f64(),
                avg_vertical_oscillation: s["verticalOscillation"].as_f64(),
                avg_vertical_ratio: s["verticalRatio"].as_f64(),
                calories: s["calories"].as_f64(),
            }
        })
        .collect()
}

impl HumanReadable for ActivitySplit {
    fn print_human(&self) {
        let dist = self
            .distance_meters
            .map(|d| format!("{:.0}m", d))
            .unwrap_or_else(|| "-".into());
        let dur = self
            .duration_seconds
            .map(|s| {
                let m = (s / 60.0).floor() as u32;
                let sec = (s % 60.0).round() as u32;
                format!("{m}:{sec:02}")
            })
            .unwrap_or_else(|| "-".into());
        let pace = self.pace.as_deref().unwrap_or("-");
        let hr = self
            .avg_hr
            .map(|h| format!("{:.0} bpm", h))
            .unwrap_or_else(|| "-".into());
        let elev = match (self.elevation_gain, self.elevation_loss) {
            (Some(g), Some(l)) => format!("+{:.0}/-{:.0}m", g, l),
            (Some(g), None) => format!("+{:.0}m", g),
            _ => String::new(),
        };
        println!(
            "  {:>3}  {:>7}  {:>6}  {:>10}  {:>8}  {}",
            format!("#{}", self.split).cyan(),
            dist,
            dur,
            pace,
            hr.dimmed(),
            elev.dimmed(),
        );
    }
}

pub async fn splits(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    let path = format!("/activity-service/activity/{id}/splits");
    let v: serde_json::Value = client.get_json(&path).await?;
    let items = splits_from_json(&v);
    output.print_list(&items, "Splits");
    Ok(())
}

// ── Weather ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ActivityWeather {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature_celsius: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feels_like_celsius: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dew_point_celsius: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub humidity_percent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wind_speed_kmh: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wind_gust_kmh: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wind_direction_degrees: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wind_direction_compass: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weather_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub station_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

fn fahrenheit_to_celsius(f: f64) -> f64 {
    ((f - 32.0) * 5.0 / 9.0 * 10.0).round() / 10.0
}

fn mph_to_kmh(mph: f64) -> f64 {
    (mph * 1.60934 * 10.0).round() / 10.0
}

fn weather_from_json(v: &serde_json::Value) -> ActivityWeather {
    ActivityWeather {
        temperature_celsius: v["temp"].as_f64().map(fahrenheit_to_celsius),
        feels_like_celsius: v["apparentTemp"].as_f64().map(fahrenheit_to_celsius),
        dew_point_celsius: v["dewPoint"].as_f64().map(fahrenheit_to_celsius),
        humidity_percent: v["relativeHumidity"].as_f64(),
        wind_speed_kmh: v["windSpeed"].as_f64().map(mph_to_kmh),
        wind_gust_kmh: v["windGust"].as_f64().map(mph_to_kmh),
        wind_direction_degrees: v["windDirection"].as_i64(),
        wind_direction_compass: v["windDirectionCompassPoint"].as_str().map(Into::into),
        weather_description: v["weatherTypeDTO"]["desc"].as_str().map(Into::into),
        station_name: v["weatherStationDTO"]["name"].as_str().map(Into::into),
        latitude: v["latitude"].as_f64(),
        longitude: v["longitude"].as_f64(),
        timestamp: v["issueDate"].as_str().map(Into::into),
    }
}

impl HumanReadable for ActivityWeather {
    fn print_human(&self) {
        println!("{}", "Weather".bold());
        if let Some(temp) = self.temperature_celsius {
            let feels = self
                .feels_like_celsius
                .map(|f| format!(" (feels like {f:.0}\u{b0}C)"))
                .unwrap_or_default();
            println!(
                "  {:<14}{:.0}\u{b0}C{}",
                "Temperature:".dimmed(),
                temp,
                feels
            );
        }
        if let Some(hum) = self.humidity_percent {
            println!("  {:<14}{:.0}%", "Humidity:".dimmed(), hum);
        }
        if let Some(wind) = self.wind_speed_kmh {
            let dir = self.wind_direction_compass.as_deref().unwrap_or("");
            println!("  {:<14}{:.0} km/h {}", "Wind:".dimmed(), wind, dir);
        }
        if let Some(ref desc) = self.weather_description {
            println!("  {:<14}{}", "Conditions:".dimmed(), desc);
        }
        if let Some(ref station) = self.station_name {
            println!("  {:<14}{}", "Station:".dimmed(), station);
        }
        println!();
    }
}

pub async fn weather(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    let path = format!("/activity-service/activity/{id}/weather");
    let v: serde_json::Value = client.get_json(&path).await?;
    let w = weather_from_json(&v);
    output.print(&w);
    Ok(())
}

// ── Laps ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ActivityLap {
    pub lap_number: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance_meters: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_hr: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_hr: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elevation_gain: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_cadence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_power: Option<f64>,
}

fn laps_from_json(v: &serde_json::Value) -> Vec<ActivityLap> {
    let arr = v["lapDTOs"].as_array().or_else(|| v.as_array());
    let Some(items) = arr else {
        return vec![];
    };
    items
        .iter()
        .enumerate()
        .map(|(i, lap)| {
            let distance = lap["distance"].as_f64();
            let duration = lap["duration"].as_f64();
            let pace = duration.and_then(|d| compute_pace(distance, d));
            ActivityLap {
                lap_number: (i + 1) as i64,
                distance_meters: distance,
                duration_seconds: duration,
                pace,
                avg_hr: lap["averageHR"].as_f64(),
                max_hr: lap["maxHR"].as_f64(),
                elevation_gain: lap["elevationGain"].as_f64(),
                avg_cadence: lap["averageRunCadence"].as_f64(),
                avg_power: lap["averagePower"].as_f64(),
            }
        })
        .collect()
}

impl HumanReadable for ActivityLap {
    fn print_human(&self) {
        let dist = self
            .distance_meters
            .map(|d| format!("{:.0}m", d))
            .unwrap_or_else(|| "\u{2014}".into());
        let dur = self
            .duration_seconds
            .map(|s| {
                let m = (s / 60.0).floor() as u32;
                let sec = (s % 60.0).round() as u32;
                format!("{m}:{sec:02}")
            })
            .unwrap_or_else(|| "\u{2014}".into());
        let pace = self.pace.as_deref().unwrap_or("\u{2014}");
        let hr = self
            .avg_hr
            .map(|h| format!("{:.0} bpm", h))
            .unwrap_or_else(|| "\u{2014}".into());
        println!(
            "  {:>3}  {:>7}  {:>6}  {:>10}  {}",
            format!("#{}", self.lap_number).cyan(),
            dist,
            dur,
            pace,
            hr.dimmed(),
        );
    }
}

pub async fn laps(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    let path = format!("/activity-service/activity/{id}/laps");
    let v: serde_json::Value = client.get_json(&path).await?;
    let items = laps_from_json(&v);
    output.print_list(&items, "Laps");
    Ok(())
}

// ── Exercises (raw passthrough - too variable) ───────────────────────

pub async fn exercises(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    let path = format!("/activity-service/activity/{id}/exerciseSets");
    let v: serde_json::Value = client.get_json(&path).await?;
    output.print_value(&v);
    Ok(())
}

// ── Power Zones ──────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct PowerZone {
    pub zone: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_watts: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_watts: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seconds_in_zone: Option<f64>,
}

fn power_zones_from_json(v: &serde_json::Value) -> Vec<PowerZone> {
    // API returns an array; each element may have a "zones" sub-array, or be the zone itself.
    let items = v.as_array().and_then(|arr| {
        if arr.is_empty() {
            return None;
        }
        // Try nested "zones" first (common response shape)
        arr[0]["zones"]
            .as_array()
            .cloned()
            .or_else(|| Some(arr.clone()))
    });
    let Some(zones) = items else {
        return vec![];
    };
    zones
        .iter()
        .enumerate()
        .map(|(i, z)| PowerZone {
            zone: z["zone"].as_i64().unwrap_or((i + 1) as i64),
            min_watts: z["zoneLowBoundary"]
                .as_f64()
                .or_else(|| z["minWatts"].as_f64()),
            max_watts: z["zoneHighBoundary"]
                .as_f64()
                .or_else(|| z["maxWatts"].as_f64()),
            seconds_in_zone: z["secsInZone"]
                .as_f64()
                .or_else(|| z["secondsInZone"].as_f64()),
        })
        .collect()
}

impl HumanReadable for PowerZone {
    fn print_human(&self) {
        let range = match (self.min_watts, self.max_watts) {
            (Some(lo), Some(hi)) => format!("{:.0}–{:.0}W", lo, hi),
            (Some(lo), None) => format!("{:.0}W+", lo),
            _ => "\u{2014}".into(),
        };
        let time = self
            .seconds_in_zone
            .map(|s| {
                let m = (s / 60.0).floor() as u32;
                let sec = (s % 60.0).round() as u32;
                format!("{m}:{sec:02}")
            })
            .unwrap_or_else(|| "\u{2014}".into());
        println!(
            "  Zone {:>1}  {:>14}  {}",
            format!("{}", self.zone).cyan(),
            range,
            time.dimmed(),
        );
    }
}

pub async fn power_zones(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    let path = format!("/activity-service/activity/{id}/powerTimeInZones");
    let v: serde_json::Value = client.get_json(&path).await?;
    let zones = power_zones_from_json(&v);
    output.print_list(&zones, "Power Zones");
    Ok(())
}

pub async fn download(
    client: &GarminClient,
    id: u64,
    format: &str,
    output_path: Option<&str>,
) -> Result<()> {
    let path = match format {
        "gpx" => format!("/download-service/export/gpx/activity/{id}"),
        "tcx" => format!("/download-service/export/tcx/activity/{id}"),
        _ => format!("/download-service/files/activity/{id}"),
    };

    let bytes = client.get_bytes(&path).await?;

    let out = output_path
        .map(String::from)
        .unwrap_or_else(|| format!("activity_{id}.{format}"));

    if out == "-" {
        use std::io::Write;
        std::io::stdout().write_all(&bytes)?;
    } else {
        std::fs::write(&out, &bytes)?;
        eprintln!("Saved to {out} ({} bytes)", bytes.len());
    }
    Ok(())
}

pub async fn upload(client: &GarminClient, output: &Output, file: &str) -> Result<()> {
    let bytes = std::fs::read(file)?;
    let filename = std::path::Path::new(file)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "upload.fit".into());

    let ext = std::path::Path::new(file)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("fit");
    let upload_path = format!("/upload-service/upload/.{ext}");

    let result = client.put_file(&upload_path, bytes, &filename).await?;

    if output.is_json() {
        output.print_value(&result);
    } else {
        output.success(&format!("Uploaded {filename}"));
    }
    Ok(())
}

/// Compare two activities side-by-side.
pub async fn compare(client: &GarminClient, output: &Output, id1: u64, id2: u64) -> Result<()> {
    let path1 = format!("/activity-service/activity/{id1}");
    let path2 = format!("/activity-service/activity/{id2}");

    let (v1, v2) = tokio::try_join!(
        client.get_json::<serde_json::Value>(&path1),
        client.get_json::<serde_json::Value>(&path2),
    )?;

    let a1 = activity_from_detail(id1, &v1);
    let a2 = activity_from_detail(id2, &v2);

    if output.is_json() {
        let delta = build_delta(&a1, &a2);
        let obj = serde_json::json!({
            "activity_1": a1,
            "activity_2": a2,
            "delta": delta,
        });
        output.print_value(&obj);
    } else {
        print_comparison_human(&a1, &a2);
    }

    Ok(())
}

fn build_delta(a1: &ActivitySummary, a2: &ActivitySummary) -> serde_json::Value {
    let mut delta = serde_json::Map::new();

    delta.insert(
        "duration_seconds".into(),
        serde_json::json!(a2.duration_seconds - a1.duration_seconds),
    );

    if let (Some(d1), Some(d2)) = (a1.distance_meters, a2.distance_meters) {
        delta.insert("distance_meters".into(), serde_json::json!(d2 - d1));
    }
    if let (Some(h1), Some(h2)) = (a1.avg_hr, a2.avg_hr) {
        delta.insert("avg_hr".into(), serde_json::json!(h2 - h1));
    }
    if let (Some(h1), Some(h2)) = (a1.max_hr, a2.max_hr) {
        delta.insert("max_hr".into(), serde_json::json!(h2 - h1));
    }
    if let (Some(c1), Some(c2)) = (a1.calories, a2.calories) {
        delta.insert("calories".into(), serde_json::json!(c2 - c1));
    }
    if let (Some(e1), Some(e2)) = (a1.elevation_gain, a2.elevation_gain) {
        delta.insert("elevation_gain".into(), serde_json::json!(e2 - e1));
    }
    if let (Some(e1), Some(e2)) = (a1.elevation_loss, a2.elevation_loss) {
        delta.insert("elevation_loss".into(), serde_json::json!(e2 - e1));
    }
    if let (Some(t1), Some(t2)) = (a1.aerobic_training_effect, a2.aerobic_training_effect) {
        delta.insert("aerobic_training_effect".into(), serde_json::json!(t2 - t1));
    }
    if let (Some(t1), Some(t2)) = (a1.anaerobic_training_effect, a2.anaerobic_training_effect) {
        delta.insert(
            "anaerobic_training_effect".into(),
            serde_json::json!(t2 - t1),
        );
    }
    if let (Some(l1), Some(l2)) = (a1.activity_training_load, a2.activity_training_load) {
        delta.insert("activity_training_load".into(), serde_json::json!(l2 - l1));
    }
    if let (Some(p1), Some(p2)) = (a1.avg_power, a2.avg_power) {
        delta.insert("avg_power".into(), serde_json::json!(p2 - p1));
    }

    serde_json::Value::Object(delta)
}

fn print_comparison_human(a1: &ActivitySummary, a2: &ActivitySummary) {
    println!(
        "{:>20} {:>20} {:>20}",
        "".bold(),
        format!("#{}", a1.id).cyan(),
        format!("#{}", a2.id).cyan(),
    );
    print_row("Name", &a1.name, &a2.name);
    print_row("Type", &a1.activity_type, &a2.activity_type);
    print_row(
        "Distance",
        &fmt_dist(a1.distance_meters),
        &fmt_dist(a2.distance_meters),
    );
    print_row(
        "Duration",
        &fmt_duration(a1.duration_seconds),
        &fmt_duration(a2.duration_seconds),
    );
    print_row(
        "Pace",
        a1.pace_min_km.as_deref().unwrap_or("\u{2014}"),
        a2.pace_min_km.as_deref().unwrap_or("\u{2014}"),
    );
    print_row("Avg HR", &fmt_opt_f64(a1.avg_hr), &fmt_opt_f64(a2.avg_hr));
    print_row("Max HR", &fmt_opt_f64(a1.max_hr), &fmt_opt_f64(a2.max_hr));
    print_row(
        "Calories",
        &fmt_opt_f64(a1.calories),
        &fmt_opt_f64(a2.calories),
    );
    print_row(
        "Elev Gain",
        &fmt_opt_f64(a1.elevation_gain),
        &fmt_opt_f64(a2.elevation_gain),
    );
    print_row(
        "Elev Loss",
        &fmt_opt_f64(a1.elevation_loss),
        &fmt_opt_f64(a2.elevation_loss),
    );
    print_row(
        "Aero TE",
        &fmt_opt_f64(a1.aerobic_training_effect),
        &fmt_opt_f64(a2.aerobic_training_effect),
    );
    print_row(
        "Anaero TE",
        &fmt_opt_f64(a1.anaerobic_training_effect),
        &fmt_opt_f64(a2.anaerobic_training_effect),
    );
    print_row(
        "Load",
        &fmt_opt_f64(a1.activity_training_load),
        &fmt_opt_f64(a2.activity_training_load),
    );
    print_row(
        "Avg Power",
        &fmt_opt_f64(a1.avg_power),
        &fmt_opt_f64(a2.avg_power),
    );
}

fn print_row(label: &str, v1: &str, v2: &str) {
    println!("{:>20} {:>20} {:>20}", label.dimmed(), v1, v2);
}

fn fmt_dist(d: Option<f64>) -> String {
    d.map(|m| format!("{:.2} km", m / 1000.0))
        .unwrap_or_else(|| "\u{2014}".into())
}

fn fmt_duration(s: f64) -> String {
    let mins = (s / 60.0).floor() as u32;
    let secs = (s % 60.0).round() as u32;
    format!("{mins}:{secs:02}")
}

fn fmt_opt_f64(v: Option<f64>) -> String {
    v.map(|x| format!("{x:.0}"))
        .unwrap_or_else(|| "\u{2014}".into())
}
