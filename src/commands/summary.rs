use crate::client::GarminClient;
use crate::error::Result;
use crate::output::{HumanReadable, Output};
use colored::Colorize;
use serde::Serialize;

fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

#[derive(Debug, Serialize)]
pub struct DailySummary {
    pub date: String,
    pub total_steps: Option<u64>,
    pub total_distance_meters: Option<f64>,
    pub active_calories: Option<f64>,
    pub total_calories: Option<f64>,
    pub resting_heart_rate: Option<u32>,
    pub max_heart_rate: Option<u32>,
    pub avg_stress: Option<f64>,
    pub max_stress: Option<u32>,
    pub body_battery_high: Option<u32>,
    pub body_battery_low: Option<u32>,
    pub sleep_seconds: Option<u64>,
    pub floors_ascended: Option<u32>,
    pub floors_descended: Option<u32>,
    pub intensity_minutes: Option<u32>,
}

impl HumanReadable for DailySummary {
    fn print_human(&self) {
        println!("{}", self.date.bold());
        if let Some(v) = self.total_steps {
            println!("  Steps:          {}", v.to_string().cyan());
        }
        if let Some(v) = self.total_distance_meters {
            println!("  Distance:       {:.1} km", v / 1000.0);
        }
        if let Some(v) = self.active_calories {
            println!("  Active cal:     {:.0}", v);
        }
        if let Some(v) = self.total_calories {
            println!("  Total cal:      {:.0}", v);
        }
        if let Some(v) = self.resting_heart_rate {
            println!("  Resting HR:     {} bpm", v.to_string().red());
        }
        if let Some(v) = self.avg_stress {
            println!("  Avg stress:     {:.0}", v);
        }
        if let (Some(hi), Some(lo)) = (self.body_battery_high, self.body_battery_low) {
            println!("  Body battery:   {lo}--{hi}");
        }
        if let Some(v) = self.sleep_seconds {
            let h = v / 3600;
            let m = (v % 3600) / 60;
            println!("  Sleep:          {h}h {m}m");
        }
        if let Some(v) = self.floors_ascended {
            println!("  Floors up:      {v}");
        }
        if let Some(v) = self.intensity_minutes {
            println!("  Intensity min:  {v}");
        }
        println!();
    }
}

pub async fn summary(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let display_name = client.display_name().await?;
    let end_date = date.map(String::from).unwrap_or_else(today);
    let days = days.unwrap_or(1);

    let mut summaries = Vec::new();
    let end = chrono::NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
        .map_err(|e| crate::error::Error::Api(format!("Invalid date: {e}")))?;

    for i in 0..days {
        let d = end - chrono::Duration::days(i as i64);
        let date_str = d.format("%Y-%m-%d").to_string();
        let path = format!(
            "/usersummary-service/usersummary/daily/{display_name}?calendarDate={date_str}"
        );
        let v: serde_json::Value = client.get_json(&path).await?;

        summaries.push(DailySummary {
            date: date_str,
            total_steps: v["totalSteps"].as_u64(),
            total_distance_meters: v["totalDistanceMeters"].as_f64(),
            active_calories: v["activeKilocalories"].as_f64(),
            total_calories: v["totalKilocalories"].as_f64(),
            resting_heart_rate: v["restingHeartRate"].as_u64().map(|v| v as u32),
            max_heart_rate: v["maxHeartRate"].as_u64().map(|v| v as u32),
            avg_stress: v["averageStressLevel"].as_f64(),
            max_stress: v["maxStressLevel"].as_u64().map(|v| v as u32),
            body_battery_high: v["bodyBatteryHighestValue"].as_u64().map(|v| v as u32),
            body_battery_low: v["bodyBatteryLowestValue"].as_u64().map(|v| v as u32),
            sleep_seconds: v["sleepingSeconds"].as_u64(),
            floors_ascended: v["floorsAscended"].as_u64().map(|v| v as u32),
            floors_descended: v["floorsDescended"].as_u64().map(|v| v as u32),
            intensity_minutes: {
                let moderate = v["moderateIntensityMinutes"].as_u64().unwrap_or(0);
                let vigorous = v["vigorousIntensityMinutes"].as_u64().unwrap_or(0);
                let total = moderate + vigorous;
                if total > 0 { Some(total as u32) } else { None }
            },
        });
    }

    summaries.reverse();
    if summaries.len() == 1 {
        output.print(&summaries[0]);
    } else {
        output.print_list(&summaries, "Daily Summary");
    }
    Ok(())
}
