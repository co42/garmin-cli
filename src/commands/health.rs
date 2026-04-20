use crate::client::GarminClient;
use crate::error::Result;
use crate::output::{HumanReadable, LABEL_WIDTH, Output};
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
    pub sleep_score: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sleep_score_qualifier: Option<String>,
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
        sleep_score: dto["sleepScores"]["overall"]["value"].as_i64(),
        sleep_score_qualifier: dto["sleepScores"]["overall"]["qualifierKey"]
            .as_str()
            .map(Into::into),
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
        println!("{}", self.date.bold());
        if let Some(score) = self.sleep_score {
            let qualifier = self
                .sleep_score_qualifier
                .as_deref()
                .map(|q| format!(" ({q})"))
                .unwrap_or_default();
            println!(
                "  {:<LABEL_WIDTH$}{}{}",
                "Score:",
                score.to_string().cyan(),
                qualifier
            );
        }
        if let Some(s) = self.sleep_seconds {
            println!("  {:<LABEL_WIDTH$}{}", "Duration:", fmt_duration(s).cyan());
        }
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
            println!("  {:<LABEL_WIDTH$}{}", "Stages:", parts.join("  "));
        }
        if let (Some(start), Some(end)) = (&self.sleep_start, &self.sleep_end) {
            println!("  {:<LABEL_WIDTH$}{start} \u{2013} {end}", "Window:");
        }
        if let Some(s) = self.sleep_need_seconds {
            println!("  {:<LABEL_WIDTH$}{}", "Need:", fmt_duration(s));
        }
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
        println!("{}", self.date.bold());
        let score_str = self
            .score
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".into());
        println!("  {:<LABEL_WIDTH$}{}", "Score:", score_str.cyan());
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
    StressSummary {
        date: v["calendarDate"].as_str().unwrap_or(date).to_string(),
        avg_stress: v["avgStressLevel"].as_i64(),
        max_stress: v["maxStressLevel"].as_i64(),
    }
}

impl HumanReadable for StressSummary {
    fn print_human(&self) {
        println!("{}", self.date.bold());
        if let Some(avg) = self.avg_stress {
            println!("  {:<LABEL_WIDTH$}{}", "Average:", avg.to_string().cyan());
        }
        if let Some(max) = self.max_stress {
            println!("  {:<LABEL_WIDTH$}{max}", "Max:");
        }
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
        println!("{}", self.date.bold());
        if let Some(v) = self.resting_hr {
            println!("  {:<LABEL_WIDTH$}{} bpm", "Resting:", v.to_string().cyan());
        }
        if let (Some(lo), Some(hi)) = (self.min_hr, self.max_hr) {
            println!("  {:<LABEL_WIDTH$}{lo}\u{2013}{hi} bpm", "Range:");
        }
        if let Some(v) = self.avg_7day_resting {
            println!("  {:<LABEL_WIDTH$}{v} bpm", "7-day avg:");
        }
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct BodyBattery {
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_battery_high: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_battery_low: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_battery_latest: Option<i64>,
}

fn body_battery_from(v: &serde_json::Value, date: &str) -> BodyBattery {
    let (high, low, latest) = extract_body_battery(v);
    BodyBattery {
        date: v["calendarDate"].as_str().unwrap_or(date).to_string(),
        body_battery_high: high,
        body_battery_low: low,
        body_battery_latest: latest,
    }
}

impl HumanReadable for BodyBattery {
    fn print_human(&self) {
        println!("{}", self.date.bold());
        if let (Some(lo), Some(hi)) = (self.body_battery_low, self.body_battery_high) {
            println!("  {:<LABEL_WIDTH$}{lo}\u{2013}{hi}", "Range:");
        }
        if let Some(v) = self.body_battery_latest {
            println!("  {:<LABEL_WIDTH$}{}", "Latest:", v.to_string().cyan());
        }
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct HrvSummary {
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_night_avg: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_night_5min_high: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weekly_average: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_balanced_low: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_balanced_upper: Option<i64>,
}

fn hrv_summary_from(v: &serde_json::Value, date: &str) -> HrvSummary {
    let s = &v["hrvSummary"];
    let raw_date = v["startTimestampLocal"]
        .as_str()
        .or_else(|| v["calendarDate"].as_str())
        .unwrap_or(date);
    // Normalize ISO datetime to YYYY-MM-DD
    let normalized_date = if raw_date.len() >= 10 {
        &raw_date[..10]
    } else {
        raw_date
    };
    HrvSummary {
        date: normalized_date.to_string(),
        last_night_avg: s["lastNightAvg"].as_i64().or(s["lastNight"].as_i64()),
        last_night_5min_high: s["lastNight5MinHigh"].as_i64(),
        weekly_average: s["weeklyAvg"].as_i64(),
        status: s["status"].as_str().map(String::from),
        baseline_balanced_low: s["baseline"]["balancedLow"]
            .as_i64()
            .or(s["baselineLowUpper"].as_i64()),
        baseline_balanced_upper: s["baseline"]["balancedUpper"]
            .as_i64()
            .or(s["baselineBalancedUpper"].as_i64()),
    }
}

impl HumanReadable for HrvSummary {
    fn print_human(&self) {
        println!("{}", self.date.bold());
        if let Some(v) = self.last_night_avg {
            println!(
                "  {:<LABEL_WIDTH$}{} ms",
                "Last night:",
                v.to_string().cyan()
            );
        }
        if let Some(v) = self.last_night_5min_high {
            println!("  {:<LABEL_WIDTH$}{v} ms", "5-min high:");
        }
        if let Some(v) = self.weekly_average {
            println!("  {:<LABEL_WIDTH$}{v} ms", "Weekly avg:");
        }
        if let Some(ref s) = self.status {
            println!("  {:<LABEL_WIDTH$}{s}", "Status:");
        }
        if let (Some(lo), Some(hi)) = (self.baseline_balanced_low, self.baseline_balanced_upper) {
            println!("  {:<LABEL_WIDTH$}{lo}\u{2013}{hi} ms", "Baseline:");
        }
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
        println!("{}", self.date.bold());
        if let Some(s) = self.total_steps {
            let goal_str = self
                .step_goal
                .map(|g| format!(" / {g}"))
                .unwrap_or_default();
            println!(
                "  {:<LABEL_WIDTH$}{}{}",
                "Steps:",
                s.to_string().cyan(),
                goal_str
            );
        }
        if let Some(d) = self.total_distance_meters {
            println!("  {:<LABEL_WIDTH$}{:.2} km", "Distance:", d / 1000.0);
        }
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
        println!("{}", self.date.bold());
        let Some(w) = self.weight_kg else {
            println!("  No data");
            return;
        };
        println!(
            "  {:<LABEL_WIDTH$}{} kg",
            "Weight:",
            format!("{w:.1}").cyan()
        );
        if let Some(b) = self.bmi {
            println!("  {:<LABEL_WIDTH$}{b:.1}", "BMI:");
        }
        if let Some(f) = self.body_fat_percent {
            println!("  {:<LABEL_WIDTH$}{f:.1}%", "Body fat:");
        }
        if let Some(m) = self.muscle_mass_kg {
            println!("  {:<LABEL_WIDTH$}{m:.1} kg", "Muscle:");
        }
        if let Some(b) = self.bone_mass_kg {
            println!("  {:<LABEL_WIDTH$}{b:.1} kg", "Bone:");
        }
        if let Some(w) = self.body_water_percent {
            println!("  {:<LABEL_WIDTH$}{w:.1}%", "Water:");
        }
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct SpO2 {
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_spo2: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lowest_spo2: Option<f64>,
}

fn spo2_from(v: &serde_json::Value, date: &str) -> SpO2 {
    SpO2 {
        date: v["calendarDate"].as_str().unwrap_or(date).to_string(),
        avg_spo2: v["averageSpO2"].as_f64(),
        lowest_spo2: v["lowestSpO2"].as_f64(),
    }
}

impl HumanReadable for SpO2 {
    fn print_human(&self) {
        println!("{}", self.date.bold());
        if let Some(a) = self.avg_spo2 {
            println!(
                "  {:<LABEL_WIDTH$}{}%",
                "Average:",
                format!("{a:.0}").cyan()
            );
        }
        if let Some(l) = self.lowest_spo2 {
            println!("  {:<LABEL_WIDTH$}{l:.0}%", "Lowest:");
        }
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct Respiration {
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_waking_br: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_sleeping_br: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highest_br: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lowest_br: Option<f64>,
}

fn respiration_from(v: &serde_json::Value, date: &str) -> Respiration {
    Respiration {
        date: v["calendarDate"].as_str().unwrap_or(date).to_string(),
        avg_waking_br: v["avgWakingRespirationValue"].as_f64(),
        avg_sleeping_br: v["avgSleepRespirationValue"].as_f64(),
        highest_br: v["highestRespirationValue"].as_f64(),
        lowest_br: v["lowestRespirationValue"].as_f64(),
    }
}

impl HumanReadable for Respiration {
    fn print_human(&self) {
        println!("{}", self.date.bold());
        if let Some(w) = self.avg_waking_br {
            println!("  {:<LABEL_WIDTH$}{w:.1} br/min", "Waking:");
        }
        if let Some(s) = self.avg_sleeping_br {
            println!("  {:<LABEL_WIDTH$}{s:.1} br/min", "Sleeping:");
        }
        if let (Some(lo), Some(hi)) = (self.lowest_br, self.highest_br) {
            println!("  {:<LABEL_WIDTH$}{lo:.1}\u{2013}{hi:.1} br/min", "Range:");
        }
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct IntensityMinutes {
    pub date: String,
    pub moderate: i64,
    pub vigorous: i64,
    pub total: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weekly_goal: Option<i64>,
}

fn intensity_minutes_from(v: &serde_json::Value, date: &str) -> IntensityMinutes {
    let moderate = v["moderateValue"].as_i64().unwrap_or(0);
    let vigorous = v["vigorousValue"].as_i64().unwrap_or(0);
    IntensityMinutes {
        date: v["calendarDate"].as_str().unwrap_or(date).to_string(),
        moderate,
        vigorous,
        total: moderate + vigorous,
        weekly_goal: v["weeklyGoal"].as_i64(),
    }
}

impl HumanReadable for IntensityMinutes {
    fn print_human(&self) {
        println!("{}", self.date.bold());
        let goal_str = self
            .weekly_goal
            .map(|g| format!(" / {g}"))
            .unwrap_or_default();
        println!(
            "  {:<LABEL_WIDTH$}{} min{}",
            "Total:",
            self.total.to_string().cyan(),
            goal_str
        );
        println!("  {:<LABEL_WIDTH$}{} min", "Moderate:", self.moderate);
        println!("  {:<LABEL_WIDTH$}{} min", "Vigorous:", self.vigorous);
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
            .or_else(|| v["dailyGoalInML"].as_f64())
            .map(|g| g.round()),
    }
}

impl HumanReadable for Hydration {
    fn print_human(&self) {
        println!("{}", self.date.bold());
        match (self.intake_ml, self.goal_ml) {
            (Some(intake), Some(goal)) => {
                println!(
                    "  {:<LABEL_WIDTH$}{:.0} / {:.0} ml",
                    "Intake:", intake, goal
                );
            }
            (Some(intake), None) => {
                println!("  {:<LABEL_WIDTH$}{:.0} ml", "Intake:", intake);
            }
            (None, Some(goal)) => {
                println!("  {:<LABEL_WIDTH$}{:.0} ml", "Goal:", goal);
            }
            (None, None) => {
                println!("  No data");
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// Fetch one JSON blob per date in `[end - days + 1 .. end]` in parallel
/// and map each with `mk_item`. Produces items in chronological order.
async fn fetch_daily<T, F, M>(
    client: &GarminClient,
    end: chrono::NaiveDate,
    days: u32,
    mk_path: F,
    mk_item: M,
) -> Result<Vec<T>>
where
    F: Fn(&str) -> String,
    M: Fn(&serde_json::Value, &str) -> T,
{
    let dates: Vec<String> = (0..days)
        .rev()
        .map(|i| {
            (end - chrono::Duration::days(i as i64))
                .format("%Y-%m-%d")
                .to_string()
        })
        .collect();
    let futs = dates.iter().map(|ds| {
        let path = mk_path(ds);
        async move { client.get_json::<serde_json::Value>(&path).await }
    });
    let results: Vec<serde_json::Value> = futures::future::join_all(futs)
        .await
        .into_iter()
        .collect::<Result<_>>()?;
    Ok(results
        .iter()
        .zip(dates.iter())
        .map(|(v, ds)| mk_item(v, ds))
        .collect())
}

pub async fn sleep(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let display_name = client.display_name().await?;
    let end = parse_date(&date.map(String::from).unwrap_or_else(today))?;
    let days = days.unwrap_or(1);
    let items = fetch_daily(
        client,
        end,
        days,
        |ds| format!("/wellness-service/wellness/dailySleepData/{display_name}?date={ds}"),
        sleep_summary_from,
    )
    .await?;
    output.print_list(&items, "Sleep");
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

    output.print_list(&items, "Sleep Scores");
    Ok(())
}

pub async fn stress(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let end = parse_date(&date.map(String::from).unwrap_or_else(today))?;
    let days = days.unwrap_or(1);
    let items = fetch_daily(
        client,
        end,
        days,
        |ds| format!("/wellness-service/wellness/dailyStress/{ds}"),
        stress_summary_from,
    )
    .await?;
    output.print_list(&items, "Stress");
    Ok(())
}

pub async fn heart_rate(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let display_name = client.display_name().await?;
    let end = parse_date(&date.map(String::from).unwrap_or_else(today))?;
    let days = days.unwrap_or(1);
    let items = fetch_daily(
        client,
        end,
        days,
        |ds| format!("/wellness-service/wellness/dailyHeartRate/{display_name}?date={ds}"),
        heart_rate_from,
    )
    .await?;
    output.print_list(&items, "Heart Rate");
    Ok(())
}

pub async fn body_battery(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let end = parse_date(&date.map(String::from).unwrap_or_else(today))?;
    let days = days.unwrap_or(1);
    let items = fetch_daily(
        client,
        end,
        days,
        |ds| format!("/wellness-service/wellness/dailyStress/{ds}"),
        body_battery_from,
    )
    .await?;
    output.print_list(&items, "Body Battery");
    Ok(())
}

pub async fn hrv(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let end = parse_date(&date.map(String::from).unwrap_or_else(today))?;
    let days = days.unwrap_or(1);
    let items = fetch_daily(
        client,
        end,
        days,
        |ds| format!("/hrv-service/hrv/{ds}"),
        hrv_summary_from,
    )
    .await?;
    output.print_list(&items, "HRV");
    Ok(())
}

pub async fn steps(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let end = parse_date(&date.map(String::from).unwrap_or_else(today))?;
    let days = days.unwrap_or(1);
    let items = fetch_daily(
        client,
        end,
        days,
        |ds| format!("/usersummary-service/stats/steps/daily/{ds}/{ds}"),
        steps_from,
    )
    .await?;
    output.print_list(&items, "Steps");
    Ok(())
}

pub async fn weight(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let end = parse_date(&date.map(String::from).unwrap_or_else(today))?;
    let days = days.unwrap_or(1);
    let items = fetch_daily(
        client,
        end,
        days,
        |ds| format!("/weight-service/weight/dateRange?startDate={ds}&endDate={ds}"),
        weight_from,
    )
    .await?;
    output.print_list(&items, "Weight");
    Ok(())
}

pub async fn hydration(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let end = parse_date(&date.map(String::from).unwrap_or_else(today))?;
    let days = days.unwrap_or(1);
    let items = fetch_daily(
        client,
        end,
        days,
        |ds| format!("/usersummary-service/usersummary/hydration/daily/{ds}"),
        hydration_from,
    )
    .await?;
    output.print_list(&items, "Hydration");
    Ok(())
}

pub async fn spo2(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let end = parse_date(&date.map(String::from).unwrap_or_else(today))?;
    let days = days.unwrap_or(1);
    let items = fetch_daily(
        client,
        end,
        days,
        |ds| format!("/wellness-service/wellness/dailySpo2/{ds}"),
        spo2_from,
    )
    .await?;
    output.print_list(&items, "SpO2");
    Ok(())
}

pub async fn respiration(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let end = parse_date(&date.map(String::from).unwrap_or_else(today))?;
    let days = days.unwrap_or(1);
    let items = fetch_daily(
        client,
        end,
        days,
        |ds| format!("/wellness-service/wellness/daily/respiration/{ds}"),
        respiration_from,
    )
    .await?;
    output.print_list(&items, "Respiration");
    Ok(())
}

pub async fn intensity_minutes(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let end = parse_date(&date.map(String::from).unwrap_or_else(today))?;
    let days = days.unwrap_or(1);
    let items = fetch_daily(
        client,
        end,
        days,
        |ds| format!("/usersummary-service/stats/im/daily/{ds}/{ds}"),
        intensity_minutes_from,
    )
    .await?;
    output.print_list(&items, "Intensity Minutes");
    Ok(())
}
