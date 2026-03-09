use crate::client::GarminClient;
use crate::error::Result;
use crate::output::{HumanReadable, Output};
use colored::Colorize;
use serde::Serialize;

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
        if let Some(ref pace) = self.avg_pace {
            println!("  Avg pace: {pace}");
        }
        println!();
    }
}

pub async fn list(
    client: &GarminClient,
    output: &Output,
    limit: u32,
    start: u32,
    activity_type: Option<&str>,
) -> Result<()> {
    let mut path =
        format!("/activitylist-service/activities/search/activities?limit={limit}&start={start}");
    if let Some(t) = activity_type {
        path.push_str(&format!("&activityType={}", urlencoding::encode(t)));
    }
    let activities: Vec<serde_json::Value> = client.get_json(&path).await?;

    let summaries: Vec<ActivitySummary> = activities
        .iter()
        .map(|a| ActivitySummary {
            id: a["activityId"].as_u64().unwrap_or(0),
            name: a["activityName"].as_str().unwrap_or("Untitled").into(),
            activity_type: a["activityType"]["typeKey"]
                .as_str()
                .unwrap_or("unknown")
                .into(),
            start_time: a["startTimeLocal"].as_str().unwrap_or("").into(),
            duration_seconds: a["duration"].as_f64().unwrap_or(0.0),
            distance_meters: a["distance"].as_f64(),
            calories: a["calories"].as_f64(),
            avg_hr: a["averageHR"].as_f64(),
            max_hr: a["maxHR"].as_f64(),
            avg_pace: a["averagePace"].as_str().map(Into::into),
        })
        .collect();

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
        output.print_value(&v);
    } else {
        // Summary view for human output
        let summary = ActivitySummary {
            id,
            name: v["activityName"].as_str().unwrap_or("Untitled").into(),
            activity_type: v["activityType"]["typeKey"]
                .as_str()
                .unwrap_or("unknown")
                .into(),
            start_time: v["startTimeLocal"].as_str().unwrap_or("").into(),
            duration_seconds: v["summary"]["duration"]["value"].as_f64().unwrap_or(0.0),
            distance_meters: v["summary"]["distance"]["value"].as_f64(),
            calories: v["summary"]["calories"]["value"].as_f64(),
            avg_hr: v["summary"]["averageHR"]["value"].as_f64(),
            max_hr: v["summary"]["maxHR"]["value"].as_f64(),
            avg_pace: None,
        };
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
