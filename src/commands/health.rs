use crate::client::GarminClient;
use crate::error::Result;
use crate::output::{HumanReadable, Output};
use crate::util::{parse_date, today};
use colored::Colorize;
use serde::Serialize;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn fmt_duration(secs: u64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    if h > 0 {
        format!("{h}h {m:02}m")
    } else {
        format!("{m}m")
    }
}

fn fmt_timestamp(ts: Option<i64>) -> Option<String> {
    ts.and_then(|ms| {
        let secs = ms / 1000;
        let dt = chrono::DateTime::from_timestamp(secs, 0)?;
        // The API gives "local" timestamps, so we just format the UTC interpretation
        Some(dt.format("%H:%M").to_string())
    })
}

// ---------------------------------------------------------------------------
// Structs
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct SleepSummary {
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sleep_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deep_sleep_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub light_sleep_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rem_sleep_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub awake_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sleep_start: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sleep_end: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sleep_need_seconds: Option<u64>,
}

fn sleep_summary_from(v: &serde_json::Value, date: &str) -> SleepSummary {
    let dto = &v["dailySleepDTO"];
    SleepSummary {
        date: dto["calendarDate"].as_str().unwrap_or(date).to_string(),
        sleep_seconds: dto["sleepTimeSeconds"].as_u64(),
        deep_sleep_seconds: dto["deepSleepSeconds"].as_u64(),
        light_sleep_seconds: dto["lightSleepSeconds"].as_u64(),
        rem_sleep_seconds: dto["remSleepSeconds"].as_u64(),
        awake_seconds: dto["awakeSleepSeconds"].as_u64(),
        sleep_start: fmt_timestamp(dto["sleepStartTimestampLocal"].as_i64()),
        sleep_end: fmt_timestamp(dto["sleepEndTimestampLocal"].as_i64()),
        sleep_need_seconds: dto["sleepNeed"]["actual"].as_u64().map(|m| m * 60),
    }
}

impl HumanReadable for SleepSummary {
    fn print_human(&self) {
        println!("{}  {}", self.date.bold(), "Sleep".dimmed());
        if let Some(s) = self.sleep_seconds {
            println!("  Duration:  {}", fmt_duration(s).cyan());
        }
        // Deep / Light / REM / Awake on one line
        let parts: Vec<String> = [
            self.deep_sleep_seconds
                .map(|s| format!("Deep: {}", fmt_duration(s))),
            self.light_sleep_seconds
                .map(|s| format!("Light: {}", fmt_duration(s))),
            self.rem_sleep_seconds
                .map(|s| format!("REM: {}", fmt_duration(s))),
            self.awake_seconds
                .map(|s| format!("Awake: {}", fmt_duration(s))),
        ]
        .into_iter()
        .flatten()
        .collect();
        if !parts.is_empty() {
            println!("  Stages:    {}", parts.join("  "));
        }
        if let (Some(start), Some(end)) = (&self.sleep_start, &self.sleep_end) {
            println!("  Window:    {start} — {end}");
        }
        if let Some(s) = self.sleep_need_seconds {
            println!("  Need:      {}", fmt_duration(s));
        }
        println!();
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct SleepScore {
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<i64>,
}

impl HumanReadable for SleepScore {
    fn print_human(&self) {
        let score_str = self
            .score
            .map(|s| s.to_string())
            .unwrap_or_else(|| "—".into());
        println!("{}  Score: {}", self.date.bold(), score_str.cyan());
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct StressSummary {
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_stress: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_stress: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_battery_high: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_battery_low: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_battery_latest: Option<i64>,
}

fn extract_body_battery(v: &serde_json::Value) -> (Option<i64>, Option<i64>, Option<i64>) {
    let arr = match v["bodyBatteryValuesArray"].as_array() {
        Some(a) if !a.is_empty() => a,
        _ => return (None, None, None),
    };
    let levels: Vec<i64> = arr
        .iter()
        .filter_map(|entry| {
            entry
                .as_array()
                .and_then(|a| a.get(2))
                .and_then(|v| v.as_i64())
        })
        .filter(|&l| l >= 0) // skip sentinel values
        .collect();
    if levels.is_empty() {
        return (None, None, None);
    }
    let high = levels.iter().copied().max();
    let low = levels.iter().copied().min();
    let latest = levels.last().copied();
    (high, low, latest)
}

fn stress_summary_from(v: &serde_json::Value, date: &str) -> StressSummary {
    let (bb_high, bb_low, bb_latest) = extract_body_battery(v);
    StressSummary {
        date: v["calendarDate"].as_str().unwrap_or(date).to_string(),
        avg_stress: v["avgStressLevel"].as_i64(),
        max_stress: v["maxStressLevel"].as_i64(),
        body_battery_high: bb_high,
        body_battery_low: bb_low,
        body_battery_latest: bb_latest,
    }
}

impl HumanReadable for StressSummary {
    fn print_human(&self) {
        println!("{}  {}", self.date.bold(), "Stress".dimmed());
        if let Some(avg) = self.avg_stress {
            let max_str = self
                .max_stress
                .map(|m| format!("  Max: {m}"))
                .unwrap_or_default();
            println!("  Avg:       {}{}", avg.to_string().cyan(), max_str);
        }
        if let (Some(lo), Some(hi)) = (self.body_battery_low, self.body_battery_high) {
            let latest_str = self
                .body_battery_latest
                .map(|l| format!("  Latest: {l}"))
                .unwrap_or_default();
            println!("  Battery:   {lo}–{hi}{latest_str}");
        }
        println!();
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct HeartRateDay {
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resting_hr: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_hr: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_hr: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_7day_resting: Option<i64>,
}

fn heart_rate_from(v: &serde_json::Value, date: &str) -> HeartRateDay {
    HeartRateDay {
        date: v["calendarDate"].as_str().unwrap_or(date).to_string(),
        resting_hr: v["restingHeartRate"].as_i64(),
        min_hr: v["minHeartRate"].as_i64(),
        max_hr: v["maxHeartRate"].as_i64(),
        avg_7day_resting: v["lastSevenDaysAvgRestingHeartRate"].as_i64(),
    }
}

impl HumanReadable for HeartRateDay {
    fn print_human(&self) {
        println!("{}  {}", self.date.bold(), "Heart Rate".dimmed());
        if let Some(v) = self.resting_hr {
            println!("  Resting:   {} bpm", v.to_string().red());
        }
        if let (Some(lo), Some(hi)) = (self.min_hr, self.max_hr) {
            println!("  Range:     {lo}–{hi} bpm");
        }
        if let Some(v) = self.avg_7day_resting {
            println!("  7-day avg: {v} bpm");
        }
        println!();
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct BodyBattery {
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub high: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub low: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest: Option<i64>,
}

fn body_battery_from(v: &serde_json::Value, date: &str) -> BodyBattery {
    let (high, low, latest) = extract_body_battery(v);
    BodyBattery {
        date: v["calendarDate"].as_str().unwrap_or(date).to_string(),
        high,
        low,
        latest,
    }
}

impl HumanReadable for BodyBattery {
    fn print_human(&self) {
        println!("{}  {}", self.date.bold(), "Body Battery".dimmed());
        if let (Some(lo), Some(hi)) = (self.low, self.high) {
            println!("  Range:     {lo}–{hi}");
        }
        if let Some(v) = self.latest {
            println!("  Latest:    {}", v.to_string().cyan());
        }
        println!();
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct HrvSummary {
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weekly_average: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_night: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_low: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_high: Option<i64>,
}

fn hrv_summary_from(v: &serde_json::Value, date: &str) -> HrvSummary {
    let s = &v["hrvSummary"];
    HrvSummary {
        date: v["startTimestampLocal"]
            .as_str()
            .or_else(|| v["calendarDate"].as_str())
            .unwrap_or(date)
            .to_string(),
        weekly_average: s["weeklyAvg"].as_i64(),
        last_night: s["lastNight"].as_i64(),
        status: s["status"].as_str().map(String::from),
        baseline_low: s["baselineLowUpper"].as_i64(),
        baseline_high: s["baselineBalancedUpper"].as_i64(),
    }
}

impl HumanReadable for HrvSummary {
    fn print_human(&self) {
        println!("{}  {}", self.date.bold(), "HRV".dimmed());
        if let Some(v) = self.last_night {
            println!("  Last night:  {} ms", v.to_string().cyan());
        }
        if let Some(v) = self.weekly_average {
            println!("  Weekly avg:  {v} ms");
        }
        if let Some(ref s) = self.status {
            println!("  Status:      {s}");
        }
        if let (Some(lo), Some(hi)) = (self.baseline_low, self.baseline_high) {
            println!("  Baseline:    {lo}–{hi} ms");
        }
        println!();
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct Steps {
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_steps: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step_goal: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_distance_meters: Option<f64>,
}

fn steps_from(v: &serde_json::Value, date: &str) -> Steps {
    // API returns an array; take first entry, or use value directly if object.
    let entry = v.as_array().and_then(|a| a.first()).unwrap_or(v);
    Steps {
        date: entry["calendarDate"].as_str().unwrap_or(date).to_string(),
        total_steps: entry["totalSteps"].as_u64(),
        step_goal: entry["stepGoal"].as_u64(),
        total_distance_meters: entry["totalDistance"].as_f64(),
    }
}

impl HumanReadable for Steps {
    fn print_human(&self) {
        println!("{}  {}", self.date.bold(), "Steps".dimmed());
        if let Some(s) = self.total_steps {
            let goal_str = self
                .step_goal
                .map(|g| format!(" / {g}"))
                .unwrap_or_default();
            println!("  Steps:     {}{}", s.to_string().cyan(), goal_str);
        }
        if let Some(d) = self.total_distance_meters {
            println!("  Distance:  {:.2} km", d / 1000.0);
        }
        println!();
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct Weight {
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight_kg: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bmi: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_fat_percent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub muscle_mass_kg: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bone_mass_kg: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_water_percent: Option<f64>,
}

fn weight_from(v: &serde_json::Value, date: &str) -> Weight {
    let list = v["dateWeightList"].as_array();
    let entry = list.and_then(|a| a.last());
    match entry {
        Some(e) => Weight {
            date: e["calendarDate"].as_str().unwrap_or(date).to_string(),
            weight_kg: e["weight"].as_f64().map(|g| g / 1000.0),
            bmi: e["bmi"].as_f64(),
            body_fat_percent: e["bodyFat"].as_f64(),
            muscle_mass_kg: e["muscleMass"].as_f64().map(|g| g / 1000.0),
            bone_mass_kg: e["boneMass"].as_f64().map(|g| g / 1000.0),
            body_water_percent: e["bodyWater"].as_f64(),
        },
        None => Weight {
            date: date.to_string(),
            weight_kg: None,
            bmi: None,
            body_fat_percent: None,
            muscle_mass_kg: None,
            bone_mass_kg: None,
            body_water_percent: None,
        },
    }
}

impl HumanReadable for Weight {
    fn print_human(&self) {
        println!("{}  {}", self.date.bold(), "Weight".dimmed());
        if let Some(w) = self.weight_kg {
            println!("  Weight:    {} kg", format!("{w:.1}").cyan());
        }
        if let Some(b) = self.bmi {
            println!("  BMI:       {b:.1}");
        }
        if let Some(f) = self.body_fat_percent {
            println!("  Body fat:  {f:.1}%");
        }
        if let Some(m) = self.muscle_mass_kg {
            println!("  Muscle:    {m:.1} kg");
        }
        if let Some(b) = self.bone_mass_kg {
            println!("  Bone:      {b:.1} kg");
        }
        if let Some(w) = self.body_water_percent {
            println!("  Water:     {w:.1}%");
        }
        println!();
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct SpO2 {
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lowest: Option<f64>,
}

fn spo2_from(v: &serde_json::Value, date: &str) -> SpO2 {
    SpO2 {
        date: v["calendarDate"].as_str().unwrap_or(date).to_string(),
        average: v["averageSpO2"].as_f64(),
        lowest: v["lowestSpO2"].as_f64(),
    }
}

impl HumanReadable for SpO2 {
    fn print_human(&self) {
        println!("{}  {}", self.date.bold(), "SpO2".dimmed());
        if let Some(a) = self.average {
            println!("  Average:   {}%", format!("{a:.0}").cyan());
        }
        if let Some(l) = self.lowest {
            println!("  Lowest:    {l:.0}%");
        }
        println!();
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct Respiration {
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_waking: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_sleeping: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highest: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lowest: Option<f64>,
}

fn respiration_from(v: &serde_json::Value, date: &str) -> Respiration {
    Respiration {
        date: v["calendarDate"].as_str().unwrap_or(date).to_string(),
        avg_waking: v["avgWakingRespirationValue"].as_f64(),
        avg_sleeping: v["avgSleepRespirationValue"].as_f64(),
        highest: v["highestRespirationValue"].as_f64(),
        lowest: v["lowestRespirationValue"].as_f64(),
    }
}

impl HumanReadable for Respiration {
    fn print_human(&self) {
        println!("{}  {}", self.date.bold(), "Respiration".dimmed());
        if let Some(w) = self.avg_waking {
            println!("  Waking:    {w:.1} br/min");
        }
        if let Some(s) = self.avg_sleeping {
            println!("  Sleeping:  {s:.1} br/min");
        }
        if let (Some(lo), Some(hi)) = (self.lowest, self.highest) {
            println!("  Range:     {lo:.1}–{hi:.1} br/min");
        }
        println!();
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct IntensityMinutes {
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moderate: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vigorous: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weekly_goal: Option<i64>,
}

fn intensity_minutes_from(v: &serde_json::Value, date: &str) -> IntensityMinutes {
    IntensityMinutes {
        date: v["calendarDate"].as_str().unwrap_or(date).to_string(),
        moderate: v["moderateValue"].as_i64(),
        vigorous: v["vigorousValue"].as_i64(),
        weekly_goal: v["weeklyGoal"].as_i64(),
    }
}

impl HumanReadable for IntensityMinutes {
    fn print_human(&self) {
        println!("{}  {}", self.date.bold(), "Intensity Minutes".dimmed());
        let m = self.moderate.unwrap_or(0);
        let v = self.vigorous.unwrap_or(0);
        let total = m + v;
        let goal_str = self
            .weekly_goal
            .map(|g| format!(" (weekly goal: {g})"))
            .unwrap_or_default();
        println!("  Total:     {} min{}", total.to_string().cyan(), goal_str);
        println!("  Moderate:  {m} min  Vigorous: {v} min");
        println!();
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct Hydration {
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intake_ml: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_ml: Option<f64>,
}

fn hydration_from(v: &serde_json::Value, date: &str) -> Hydration {
    Hydration {
        date: v["calendarDate"].as_str().unwrap_or(date).to_string(),
        intake_ml: v["valueInML"].as_f64().or_else(|| v["intakeInML"].as_f64()),
        goal_ml: v["goalInML"]
            .as_f64()
            .or_else(|| v["dailyGoalInML"].as_f64()),
    }
}

impl HumanReadable for Hydration {
    fn print_human(&self) {
        println!("{}  {}", self.date.bold(), "Hydration".dimmed());
        if let Some(ml) = self.intake_ml {
            let goal_str = self
                .goal_ml
                .map(|g| format!(" / {:.0} ml", g))
                .unwrap_or_default();
            println!("  Intake:    {:.0} ml{}", ml, goal_str);
        }
        println!();
    }
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

pub async fn sleep(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let display_name = client.display_name().await?;
    let end_date = date.map(String::from).unwrap_or_else(today);
    let days = days.unwrap_or(1);
    let end = parse_date(&end_date)?;

    if days == 1 {
        let path =
            format!("/wellness-service/wellness/dailySleepData/{display_name}?date={end_date}");
        let v: serde_json::Value = client.get_json(&path).await?;
        let item = sleep_summary_from(&v, &end_date);
        output.print(&item);
    } else {
        let futs: Vec<_> = (0..days)
            .rev()
            .map(|i| {
                let d = end - chrono::Duration::days(i as i64);
                let ds = d.format("%Y-%m-%d").to_string();
                let path =
                    format!("/wellness-service/wellness/dailySleepData/{display_name}?date={ds}");
                async move { client.get_json::<serde_json::Value>(&path).await }
            })
            .collect();
        let results: Vec<serde_json::Value> = futures::future::join_all(futs)
            .await
            .into_iter()
            .collect::<Result<_>>()?;
        let items: Vec<SleepSummary> = results
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let d = end - chrono::Duration::days((days - 1 - i as u32) as i64);
                sleep_summary_from(v, &d.format("%Y-%m-%d").to_string())
            })
            .collect();
        output.print_list(&items, "Sleep");
    }
    Ok(())
}

pub async fn sleep_scores(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let end_date = date.map(String::from).unwrap_or_else(today);
    let days = days.unwrap_or(7);
    let end = parse_date(&end_date)?;
    let start = end - chrono::Duration::days(days as i64 - 1);
    let start_str = start.format("%Y-%m-%d").to_string();
    let path = format!("/wellness-service/stats/daily/sleep/score/{start_str}/{end_date}");
    let v: serde_json::Value = client.get_json(&path).await?;

    let items: Vec<SleepScore> = match v.as_array() {
        Some(arr) => arr
            .iter()
            .map(|entry| SleepScore {
                date: entry["calendarDate"].as_str().unwrap_or("").to_string(),
                score: entry["value"].as_i64(),
            })
            .collect(),
        None => vec![SleepScore {
            date: end_date,
            score: v["value"].as_i64(),
        }],
    };

    if items.len() == 1 {
        output.print(&items[0]);
    } else {
        output.print_list(&items, "Sleep Scores");
    }
    Ok(())
}

pub async fn stress(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let end_date = date.map(String::from).unwrap_or_else(today);
    let days = days.unwrap_or(1);
    let end = parse_date(&end_date)?;

    if days == 1 {
        let path = format!("/wellness-service/wellness/dailyStress/{end_date}");
        let v: serde_json::Value = client.get_json(&path).await?;
        let item = stress_summary_from(&v, &end_date);
        output.print(&item);
    } else {
        let futs: Vec<_> = (0..days)
            .rev()
            .map(|i| {
                let d = end - chrono::Duration::days(i as i64);
                let path = format!(
                    "/wellness-service/wellness/dailyStress/{}",
                    d.format("%Y-%m-%d")
                );
                async move { client.get_json::<serde_json::Value>(&path).await }
            })
            .collect();
        let results: Vec<serde_json::Value> = futures::future::join_all(futs)
            .await
            .into_iter()
            .collect::<Result<_>>()?;
        let items: Vec<StressSummary> = results
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let d = end - chrono::Duration::days((days - 1 - i as u32) as i64);
                stress_summary_from(v, &d.format("%Y-%m-%d").to_string())
            })
            .collect();
        output.print_list(&items, "Stress");
    }
    Ok(())
}

pub async fn heart_rate(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let display_name = client.display_name().await?;
    let end_date = date.map(String::from).unwrap_or_else(today);
    let days = days.unwrap_or(1);
    let end = parse_date(&end_date)?;

    if days == 1 {
        let path =
            format!("/wellness-service/wellness/dailyHeartRate/{display_name}?date={end_date}");
        let v: serde_json::Value = client.get_json(&path).await?;
        let item = heart_rate_from(&v, &end_date);
        output.print(&item);
    } else {
        let futs: Vec<_> = (0..days)
            .rev()
            .map(|i| {
                let d = end - chrono::Duration::days(i as i64);
                let path = format!(
                    "/wellness-service/wellness/dailyHeartRate/{display_name}?date={}",
                    d.format("%Y-%m-%d")
                );
                async move { client.get_json::<serde_json::Value>(&path).await }
            })
            .collect();
        let results: Vec<serde_json::Value> = futures::future::join_all(futs)
            .await
            .into_iter()
            .collect::<Result<_>>()?;
        let items: Vec<HeartRateDay> = results
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let d = end - chrono::Duration::days((days - 1 - i as u32) as i64);
                heart_rate_from(v, &d.format("%Y-%m-%d").to_string())
            })
            .collect();
        output.print_list(&items, "Heart Rate");
    }
    Ok(())
}

pub async fn body_battery(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
) -> Result<()> {
    let date_str = date.map(String::from).unwrap_or_else(today);
    let path = format!("/wellness-service/wellness/dailyStress/{date_str}");
    let v: serde_json::Value = client.get_json(&path).await?;
    let item = body_battery_from(&v, &date_str);
    output.print(&item);
    Ok(())
}

pub async fn hrv(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let end_date = date.map(String::from).unwrap_or_else(today);
    let days = days.unwrap_or(1);
    let end = parse_date(&end_date)?;

    if days == 1 {
        let path = format!("/hrv-service/hrv/{end_date}");
        let v: serde_json::Value = client.get_json(&path).await?;
        let item = hrv_summary_from(&v, &end_date);
        output.print(&item);
    } else {
        let futs: Vec<_> = (0..days)
            .rev()
            .map(|i| {
                let d = end - chrono::Duration::days(i as i64);
                let path = format!("/hrv-service/hrv/{}", d.format("%Y-%m-%d"));
                async move { client.get_json::<serde_json::Value>(&path).await }
            })
            .collect();
        let results: Vec<serde_json::Value> = futures::future::join_all(futs)
            .await
            .into_iter()
            .collect::<Result<_>>()?;
        let items: Vec<HrvSummary> = results
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let d = end - chrono::Duration::days((days - 1 - i as u32) as i64);
                hrv_summary_from(v, &d.format("%Y-%m-%d").to_string())
            })
            .collect();
        output.print_list(&items, "HRV");
    }
    Ok(())
}

pub async fn steps(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let end_date = date.map(String::from).unwrap_or_else(today);
    let days = days.unwrap_or(1);
    let end = parse_date(&end_date)?;

    if days == 1 {
        let path = format!("/usersummary-service/stats/steps/daily/{end_date}/{end_date}");
        let v: serde_json::Value = client.get_json(&path).await?;
        let item = steps_from(&v, &end_date);
        output.print(&item);
    } else {
        let futs: Vec<_> = (0..days)
            .rev()
            .map(|i| {
                let d = end - chrono::Duration::days(i as i64);
                let ds = d.format("%Y-%m-%d").to_string();
                let path = format!("/usersummary-service/stats/steps/daily/{ds}/{ds}");
                async move { client.get_json::<serde_json::Value>(&path).await }
            })
            .collect();
        let results: Vec<serde_json::Value> = futures::future::join_all(futs)
            .await
            .into_iter()
            .collect::<Result<_>>()?;
        let items: Vec<Steps> = results
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let d = end - chrono::Duration::days((days - 1 - i as u32) as i64);
                steps_from(v, &d.format("%Y-%m-%d").to_string())
            })
            .collect();
        output.print_list(&items, "Steps");
    }
    Ok(())
}

pub async fn weight(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let end_date = date.map(String::from).unwrap_or_else(today);
    let days = days.unwrap_or(1);
    let end = parse_date(&end_date)?;

    if days == 1 {
        let path =
            format!("/weight-service/weight/dateRange?startDate={end_date}&endDate={end_date}");
        let v: serde_json::Value = client.get_json(&path).await?;
        let item = weight_from(&v, &end_date);
        output.print(&item);
    } else {
        let futs: Vec<_> = (0..days)
            .rev()
            .map(|i| {
                let d = end - chrono::Duration::days(i as i64);
                let ds = d.format("%Y-%m-%d").to_string();
                let path = format!("/weight-service/weight/dateRange?startDate={ds}&endDate={ds}");
                async move { client.get_json::<serde_json::Value>(&path).await }
            })
            .collect();
        let results: Vec<serde_json::Value> = futures::future::join_all(futs)
            .await
            .into_iter()
            .collect::<Result<_>>()?;
        let items: Vec<Weight> = results
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let d = end - chrono::Duration::days((days - 1 - i as u32) as i64);
                weight_from(v, &d.format("%Y-%m-%d").to_string())
            })
            .collect();
        output.print_list(&items, "Weight");
    }
    Ok(())
}

pub async fn hydration(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let end_date = date.map(String::from).unwrap_or_else(today);
    let days = days.unwrap_or(1);
    let end = parse_date(&end_date)?;

    if days == 1 {
        let path = format!("/usersummary-service/usersummary/hydration/daily/{end_date}");
        let v: serde_json::Value = client.get_json(&path).await?;
        let item = hydration_from(&v, &end_date);
        output.print(&item);
    } else {
        let futs: Vec<_> = (0..days)
            .rev()
            .map(|i| {
                let d = end - chrono::Duration::days(i as i64);
                let path = format!(
                    "/usersummary-service/usersummary/hydration/daily/{}",
                    d.format("%Y-%m-%d")
                );
                async move { client.get_json::<serde_json::Value>(&path).await }
            })
            .collect();
        let results: Vec<serde_json::Value> = futures::future::join_all(futs)
            .await
            .into_iter()
            .collect::<Result<_>>()?;
        let items: Vec<Hydration> = results
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let d = end - chrono::Duration::days((days - 1 - i as u32) as i64);
                hydration_from(v, &d.format("%Y-%m-%d").to_string())
            })
            .collect();
        output.print_list(&items, "Hydration");
    }
    Ok(())
}

pub async fn spo2(client: &GarminClient, output: &Output, date: Option<&str>) -> Result<()> {
    let date_str = date.map(String::from).unwrap_or_else(today);
    let path = format!("/wellness-service/wellness/dailySpo2/{date_str}");
    let v: serde_json::Value = client.get_json(&path).await?;
    let item = spo2_from(&v, &date_str);
    output.print(&item);
    Ok(())
}

pub async fn respiration(client: &GarminClient, output: &Output, date: Option<&str>) -> Result<()> {
    let date_str = date.map(String::from).unwrap_or_else(today);
    let path = format!("/wellness-service/wellness/daily/respiration/{date_str}");
    let v: serde_json::Value = client.get_json(&path).await?;
    let item = respiration_from(&v, &date_str);
    output.print(&item);
    Ok(())
}

pub async fn intensity_minutes(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let end_date = date.map(String::from).unwrap_or_else(today);
    let days = days.unwrap_or(1);
    let end = parse_date(&end_date)?;

    if days == 1 {
        let path = format!("/usersummary-service/stats/im/daily/{end_date}/{end_date}");
        let v: serde_json::Value = client.get_json(&path).await?;
        let item = intensity_minutes_from(&v, &end_date);
        output.print(&item);
    } else {
        let futs: Vec<_> = (0..days)
            .rev()
            .map(|i| {
                let d = end - chrono::Duration::days(i as i64);
                let ds = d.format("%Y-%m-%d").to_string();
                let path = format!("/usersummary-service/stats/im/daily/{ds}/{ds}");
                async move { client.get_json::<serde_json::Value>(&path).await }
            })
            .collect();
        let results: Vec<serde_json::Value> = futures::future::join_all(futs)
            .await
            .into_iter()
            .collect::<Result<_>>()?;
        let items: Vec<IntensityMinutes> = results
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let d = end - chrono::Duration::days((days - 1 - i as u32) as i64);
                intensity_minutes_from(v, &d.format("%Y-%m-%d").to_string())
            })
            .collect();
        output.print_list(&items, "Intensity Minutes");
    }
    Ok(())
}
