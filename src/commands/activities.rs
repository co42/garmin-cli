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
    let pace_secs_per_km = duration_seconds / (dist / 1000.0);
    let mins = (pace_secs_per_km / 60.0).floor() as u32;
    let secs = (pace_secs_per_km % 60.0).round() as u32;
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
    pub avg_pace: Option<String>,
    pub pace_min_km: Option<String>,
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
        } else if let Some(ref pace) = self.avg_pace {
            println!("  Avg pace: {pace}");
        }
        println!();
    }
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
        start_time: a["startTimeLocal"].as_str().unwrap_or("").into(),
        duration_seconds,
        distance_meters,
        calories: a["calories"].as_f64(),
        avg_hr: a["averageHR"].as_f64(),
        max_hr: a["maxHR"].as_f64(),
        avg_pace: a["averagePace"].as_str().map(Into::into),
        pace_min_km: compute_pace(distance_meters, duration_seconds),
    }
}

fn activity_from_detail(id: u64, v: &serde_json::Value) -> ActivitySummary {
    let duration_seconds = v["summary"]["duration"]["value"].as_f64().unwrap_or(0.0);
    let distance_meters = v["summary"]["distance"]["value"].as_f64();
    ActivitySummary {
        id,
        name: v["activityName"].as_str().unwrap_or("Untitled").into(),
        activity_type: v["activityType"]["typeKey"]
            .as_str()
            .unwrap_or("unknown")
            .into(),
        start_time: v["startTimeLocal"].as_str().unwrap_or("").into(),
        duration_seconds,
        distance_meters,
        calories: v["summary"]["calories"]["value"].as_f64(),
        avg_hr: v["summary"]["averageHR"]["value"].as_f64(),
        max_hr: v["summary"]["maxHR"]["value"].as_f64(),
        avg_pace: None,
        pace_min_km: compute_pace(distance_meters, duration_seconds),
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
    let path = format!("/activity-service/activity/{id}");
    let v: serde_json::Value = client.get_json(&path).await?;

    if output.is_json() {
        // Inject computed pace into JSON output
        let mut val = v.clone();
        let duration = v["summary"]["duration"]["value"].as_f64().unwrap_or(0.0);
        let distance = v["summary"]["distance"]["value"].as_f64();
        if let Some(pace) = compute_pace(distance, duration) {
            val["pace_min_km"] = serde_json::Value::String(pace);
        }
        output.print_value(&val);
    } else {
        let summary = activity_from_detail(id, &v);
        output.print(&summary);
    }
    Ok(())
}

pub async fn details(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    let path = format!("/activity-service/activity/{id}/details");
    let v: serde_json::Value = client.get_json(&path).await?;
    output.print_value(&v);
    Ok(())
}

pub async fn hr_zones(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    let path = format!("/activity-service/activity/{id}/hrTimeInZones");
    let v: serde_json::Value = client.get_json(&path).await?;
    output.print_value(&v);
    Ok(())
}

pub async fn splits(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    let path = format!("/activity-service/activity/{id}/splits");
    let v: serde_json::Value = client.get_json(&path).await?;
    output.print_value(&v);
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
