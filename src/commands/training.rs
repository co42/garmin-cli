use crate::client::GarminClient;
use crate::error::Result;
use crate::output::{HumanReadable, LABEL_WIDTH, Output};
use crate::util::{parse_date, today};
use colored::Colorize;
use serde::Serialize;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn fmt_duration(secs: f64) -> String {
    let total = secs.round() as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

fn fmt_pace(secs: f64, distance_m: f64) -> String {
    let pace_secs = secs / (distance_m / 1000.0);
    let m = pace_secs as u64 / 60;
    let s = pace_secs as u64 % 60;
    format!("{m}:{s:02}")
}

fn fitness_trend_label(code: i64) -> &'static str {
    match code {
        1 => "improving",
        0 => "stable",
        -1 => "declining",
        _ => "unknown",
    }
}

fn classification_label(code: i64) -> &'static str {
    match code {
        1 => "Base",
        2 => "Intermediate",
        3 => "Trained",
        4 => "Well-Trained",
        5 => "Expert",
        6 => "Superior",
        7 => "Elite",
        _ => "Unknown",
    }
}

// ---------------------------------------------------------------------------
// 1. TrainingStatus
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct TrainingStatus {
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fitness_trend: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fitness_trend_sport: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub training_paused: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_date: Option<String>,
    // ACWR
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acute_load: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chronic_load: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acwr: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acwr_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_training_load_chronic: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_training_load_chronic: Option<f64>,
    // Load balance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly_load_aerobic_high: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly_load_aerobic_high_target_min: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly_load_aerobic_high_target_max: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly_load_aerobic_low: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly_load_aerobic_low_target_min: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly_load_aerobic_low_target_max: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly_load_anaerobic: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly_load_anaerobic_target_min: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly_load_anaerobic_target_max: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_balance_feedback: Option<String>,
    // VO2max — precise value if available, else integer-rounded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vo2max: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vo2max_date: Option<String>,
}

fn training_status_from(v: &serde_json::Value, date: &str) -> TrainingStatus {
    // Navigate to the first device entry in the nested maps
    let status_data = v["mostRecentTrainingStatus"]["latestTrainingStatusData"]
        .as_object()
        .and_then(|m| m.values().next());

    let load_balance = v["mostRecentTrainingLoadBalance"]["metricsTrainingLoadBalanceDTOMap"]
        .as_object()
        .and_then(|m| m.values().next());

    let vo2 = &v["mostRecentVO2Max"]["generic"];

    let sd = status_data.cloned().unwrap_or(serde_json::Value::Null);
    let lb = load_balance.cloned().unwrap_or(serde_json::Value::Null);

    TrainingStatus {
        date: date.to_string(),
        status: sd["trainingStatusFeedbackPhrase"].as_str().map(Into::into),
        fitness_trend: sd["fitnessTrend"]
            .as_i64()
            .map(|c| fitness_trend_label(c).to_string()),
        fitness_trend_sport: sd["fitnessTrendSport"].as_str().map(Into::into),
        training_paused: sd["trainingPaused"].as_bool(),
        since_date: sd["sinceDate"].as_str().map(Into::into),
        acute_load: sd["acuteTrainingLoadDTO"]["dailyTrainingLoadAcute"].as_f64(),
        chronic_load: sd["acuteTrainingLoadDTO"]["dailyTrainingLoadChronic"].as_f64(),
        acwr: sd["acuteTrainingLoadDTO"]["dailyAcuteChronicWorkloadRatio"].as_f64(),
        acwr_status: sd["acuteTrainingLoadDTO"]["acwrStatus"]
            .as_str()
            .map(Into::into),
        min_training_load_chronic: sd["acuteTrainingLoadDTO"]["minTrainingLoadChronic"].as_f64(),
        max_training_load_chronic: sd["acuteTrainingLoadDTO"]["maxTrainingLoadChronic"].as_f64(),
        monthly_load_aerobic_high: lb["monthlyLoadAerobicHigh"].as_f64(),
        monthly_load_aerobic_high_target_min: lb["monthlyLoadAerobicHighTargetMin"].as_i64(),
        monthly_load_aerobic_high_target_max: lb["monthlyLoadAerobicHighTargetMax"].as_i64(),
        monthly_load_aerobic_low: lb["monthlyLoadAerobicLow"].as_f64(),
        monthly_load_aerobic_low_target_min: lb["monthlyLoadAerobicLowTargetMin"].as_i64(),
        monthly_load_aerobic_low_target_max: lb["monthlyLoadAerobicLowTargetMax"].as_i64(),
        monthly_load_anaerobic: lb["monthlyLoadAnaerobic"].as_f64(),
        monthly_load_anaerobic_target_min: lb["monthlyLoadAnaerobicTargetMin"].as_i64(),
        monthly_load_anaerobic_target_max: lb["monthlyLoadAnaerobicTargetMax"].as_i64(),
        load_balance_feedback: lb["trainingBalanceFeedbackPhrase"].as_str().map(Into::into),
        vo2max: vo2["vo2MaxPreciseValue"]
            .as_f64()
            .or(vo2["vo2MaxValue"].as_f64()),
        vo2max_date: vo2["calendarDate"].as_str().map(Into::into),
    }
}

impl HumanReadable for TrainingStatus {
    fn print_human(&self) {
        println!("{}", self.date.bold());

        if let Some(ref s) = self.status {
            let since = self
                .since_date
                .as_deref()
                .map(|d| format!(" (since {d})"))
                .unwrap_or_default();
            println!("  {:<LABEL_WIDTH$}{}{since}", "Status:", s.yellow());
        }

        if let Some(ref trend) = self.fitness_trend {
            let sport = self
                .fitness_trend_sport
                .as_deref()
                .map(|s| format!(" ({})", s.to_lowercase()))
                .unwrap_or_default();
            println!("  {:<LABEL_WIDTH$}{trend}{sport}", "Fitness trend:");
        }

        if let Some(acwr) = self.acwr {
            let status = self.acwr_status.as_deref().unwrap_or("?");
            let acute = self
                .acute_load
                .map(|v| format!("{v:.0}"))
                .unwrap_or_default();
            let chronic = self
                .chronic_load
                .map(|v| format!("{v:.0}"))
                .unwrap_or_default();
            println!(
                "  {:<LABEL_WIDTH$}{acwr:.1} ({status}) - acute: {acute} / chronic: {chronic}",
                "ACWR:"
            );
        }

        if let Some(ref fb) = self.load_balance_feedback {
            println!("  {:<LABEL_WIDTH$}{fb}", "Load balance:");
            if let Some(ah) = self.monthly_load_aerobic_high {
                let min = self.monthly_load_aerobic_high_target_min.unwrap_or(0);
                let max = self.monthly_load_aerobic_high_target_max.unwrap_or(0);
                println!(
                    "    {:<LABEL_WIDTH$}{ah:.0} (target: {min}–{max})",
                    "Aerobic high:"
                );
            }
            if let Some(al) = self.monthly_load_aerobic_low {
                let min = self.monthly_load_aerobic_low_target_min.unwrap_or(0);
                let max = self.monthly_load_aerobic_low_target_max.unwrap_or(0);
                println!(
                    "    {:<LABEL_WIDTH$}{al:.0} (target: {min}–{max})",
                    "Aerobic low:"
                );
            }
            if let Some(an) = self.monthly_load_anaerobic {
                let min = self.monthly_load_anaerobic_target_min.unwrap_or(0);
                let max = self.monthly_load_anaerobic_target_max.unwrap_or(0);
                println!(
                    "    {:<LABEL_WIDTH$}{an:.0} (target: {min}–{max})",
                    "Anaerobic:"
                );
            }
        }

        if let Some(vo2) = self.vo2max {
            let date_part = self
                .vo2max_date
                .as_deref()
                .map(|d| format!(" ({d})"))
                .unwrap_or_default();
            println!("  {:<LABEL_WIDTH$}{vo2:.1}{date_part}", "VO2max:");
        }
    }
}

pub async fn status(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let end_date = date.map(String::from).unwrap_or_else(today);
    let days = days.unwrap_or(1);
    let end = parse_date(&end_date)?;

    let futs: Vec<_> = (0..days)
        .rev()
        .map(|i| {
            let d = end - chrono::Duration::days(i as i64);
            let ds = d.format("%Y-%m-%d").to_string();
            let path = format!("/metrics-service/metrics/trainingstatus/aggregated/{ds}");
            async move {
                let v: serde_json::Value = client.get_json(&path).await?;
                Ok(training_status_from(&v, &ds)) as Result<TrainingStatus>
            }
        })
        .collect();
    let items: Vec<TrainingStatus> = futures::future::join_all(futs)
        .await
        .into_iter()
        .collect::<Result<_>>()?;
    output.print_list(&items, "Training Status");
    Ok(())
}

// ---------------------------------------------------------------------------
// 2. TrainingReadiness
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct TrainingReadiness {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp_local: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_time_minutes: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hrv_weekly_average: Option<i64>,
    // Factor breakdown
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hrv_score: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hrv_feedback: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sleep_history_score: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sleep_history_feedback: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sleep_score: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sleep_feedback: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_score: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_feedback: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acwr_score: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acwr_feedback: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stress_score: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stress_feedback: Option<String>,
}

/// Wraps a day's readiness into morning (wake-up), post-activity, and latest
/// (real-time) scores.
#[derive(Debug, Serialize)]
pub struct DailyReadiness {
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub morning: Option<TrainingReadiness>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_activity: Option<TrainingReadiness>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest: Option<TrainingReadiness>,
}

fn readiness_entry_from(entry: &serde_json::Value) -> TrainingReadiness {
    TrainingReadiness {
        timestamp_local: entry["timestampLocal"].as_str().map(Into::into),
        score: entry["score"].as_i64(),
        level: entry["level"].as_str().map(Into::into),
        feedback: entry["feedbackShort"].as_str().map(Into::into),
        recovery_time_minutes: entry["recoveryTime"].as_i64(),
        hrv_weekly_average: entry["hrvWeeklyAverage"].as_i64(),
        hrv_score: entry["hrvFactorPercent"].as_i64(),
        hrv_feedback: entry["hrvFactorFeedback"].as_str().map(Into::into),
        sleep_history_score: entry["sleepHistoryFactorPercent"].as_i64(),
        sleep_history_feedback: entry["sleepHistoryFactorFeedback"].as_str().map(Into::into),
        sleep_score: entry["sleepScoreFactorPercent"].as_i64(),
        sleep_feedback: entry["sleepScoreFactorFeedback"].as_str().map(Into::into),
        recovery_score: entry["recoveryTimeFactorPercent"].as_i64(),
        recovery_feedback: entry["recoveryTimeFactorFeedback"].as_str().map(Into::into),
        acwr_score: entry["acwrFactorPercent"].as_i64(),
        acwr_feedback: entry["acwrFactorFeedback"].as_str().map(Into::into),
        stress_score: entry["stressHistoryFactorPercent"].as_i64(),
        stress_feedback: entry["stressHistoryFactorFeedback"]
            .as_str()
            .map(Into::into),
    }
}

fn daily_readiness_from(v: &serde_json::Value, date: &str) -> DailyReadiness {
    let entries: Vec<&serde_json::Value> = match v.as_array() {
        Some(arr) => arr.iter().collect(),
        None if v.is_object() => vec![v],
        _ => vec![],
    };

    let mut morning: Option<TrainingReadiness> = None;
    let mut post_activity: Option<TrainingReadiness> = None;
    let mut latest: Option<TrainingReadiness> = None;

    for entry in &entries {
        match entry["inputContext"].as_str() {
            Some("AFTER_WAKEUP_RESET") => morning = Some(readiness_entry_from(entry)),
            Some("AFTER_POST_EXERCISE_RESET") => post_activity = Some(readiness_entry_from(entry)),
            Some("UPDATE_REALTIME_VARIABLES") => latest = Some(readiness_entry_from(entry)),
            _ => {}
        }
    }

    // Drop latest if its timestamp is not after the morning/post-activity snapshot
    // (stale realtime entry carried over from the previous day).
    if let Some(ref l) = latest {
        let ref_ts = post_activity
            .as_ref()
            .or(morning.as_ref())
            .and_then(|r| r.timestamp_local.as_deref());
        let keep = match (l.timestamp_local.as_deref(), ref_ts) {
            (Some(lt), Some(rt)) => lt > rt,
            _ => true,
        };
        if !keep {
            latest = None;
        }
    }

    // Fallback: if no inputContext was found (old firmware), treat the entry with
    // the earliest timestamp as morning.
    if morning.is_none()
        && post_activity.is_none()
        && latest.is_none()
        && let Some(first) = entries.last()
    {
        morning = Some(readiness_entry_from(first));
    }

    DailyReadiness {
        date: date.to_string(),
        morning,
        post_activity,
        latest,
    }
}

impl TrainingReadiness {
    fn print_section(&self, label: &str) {
        let score_str = self
            .score
            .map(|s| format!("{s}/100"))
            .unwrap_or_else(|| "?".into());
        let level = self.level.as_deref().unwrap_or("?");
        println!("  {:<LABEL_WIDTH$}{} ({})", label, score_str.cyan(), level);

        if let Some(ref fb) = self.feedback {
            println!("  {:<LABEL_WIDTH$}{fb}", "");
        }

        let mut parts = Vec::new();
        if let Some(rt) = self.recovery_time_minutes {
            let h = rt / 60;
            let m = rt % 60;
            parts.push(format!("Recovery: {h}h{m:02}"));
        }
        if let Some(hrv) = self.hrv_weekly_average {
            parts.push(format!("HRV 7d: {hrv}ms"));
        }
        if !parts.is_empty() {
            println!("  {:<LABEL_WIDTH$}{}", "", parts.join("  "));
        }

        println!("  Factors:");
        print_factor("HRV", self.hrv_score, self.hrv_feedback.as_deref());
        print_factor(
            "Sleep history",
            self.sleep_history_score,
            self.sleep_history_feedback.as_deref(),
        );
        print_factor("Sleep", self.sleep_score, self.sleep_feedback.as_deref());
        print_factor(
            "Recovery",
            self.recovery_score,
            self.recovery_feedback.as_deref(),
        );
        print_factor("ACWR", self.acwr_score, self.acwr_feedback.as_deref());
        print_factor("Stress", self.stress_score, self.stress_feedback.as_deref());
    }
}

impl HumanReadable for TrainingReadiness {
    fn print_human(&self) {
        // Standalone printing (not used directly, but required by trait)
        self.print_section("Readiness");
    }
}

impl HumanReadable for DailyReadiness {
    fn print_human(&self) {
        println!("{}", self.date.bold());

        if let Some(ref m) = self.morning {
            m.print_section("Morning");
        }

        if let Some(ref pa) = self.post_activity {
            if self.morning.is_some() {
                println!();
            }
            pa.print_section("Post-activity");
        }

        if let Some(ref l) = self.latest {
            if self.morning.is_some() || self.post_activity.is_some() {
                println!();
            }
            l.print_section("Latest");
        }

        if self.morning.is_none() && self.post_activity.is_none() && self.latest.is_none() {
            println!("  No readiness data");
        }
    }
}

fn print_factor(name: &str, score: Option<i64>, feedback: Option<&str>) {
    if let Some(s) = score {
        let fb = feedback.unwrap_or("?");
        println!("    {name:<LABEL_WIDTH$}{s:>3}% ({fb})");
    }
}

pub async fn readiness(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let end_date = date.map(String::from).unwrap_or_else(today);
    let days = days.unwrap_or(1);
    let end = parse_date(&end_date)?;

    let futs: Vec<_> = (0..days)
        .rev()
        .map(|i| {
            let d = end - chrono::Duration::days(i as i64);
            let ds = d.format("%Y-%m-%d").to_string();
            let path = format!("/metrics-service/metrics/trainingreadiness/{ds}");
            async move {
                let v: serde_json::Value = client.get_json(&path).await?;
                Ok(daily_readiness_from(&v, &ds)) as Result<DailyReadiness>
            }
        })
        .collect();
    let items: Vec<DailyReadiness> = futures::future::join_all(futs)
        .await
        .into_iter()
        .collect::<Result<_>>()?;
    output.print_list(&items, "Training Readiness");
    Ok(())
}

// ---------------------------------------------------------------------------
// 3. TrainingScore (VO2max daily)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct TrainingScore {
    pub date: String,
    /// VO2max — precise value when available, else the integer-rounded value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vo2max: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fitness_age: Option<f64>,
}

fn training_score_from(v: &serde_json::Value) -> TrainingScore {
    let g = &v["generic"];
    TrainingScore {
        date: g["calendarDate"]
            .as_str()
            .or(v["calendarDate"].as_str())
            .unwrap_or("")
            .to_string(),
        vo2max: g["vo2MaxPreciseValue"]
            .as_f64()
            .or(g["vo2MaxValue"].as_f64()),
        fitness_age: g["fitnessAge"].as_f64(),
    }
}

impl HumanReadable for TrainingScore {
    fn print_human(&self) {
        println!("{}", self.date.bold());
        let vo2 = self
            .vo2max
            .map(|v| format!("{v:.1}"))
            .unwrap_or_else(|| "–".into());
        println!("  {:<LABEL_WIDTH$}{}", "VO2max:", vo2.cyan());
        if let Some(age) = self.fitness_age {
            println!("  {:<LABEL_WIDTH$}{age:.0}", "Fitness age:");
        }
    }
}

pub async fn scores(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let end_date = date.map(String::from).unwrap_or_else(today);
    let end = parse_date(&end_date)?;
    let days = days.unwrap_or(7);
    let start = end - chrono::Duration::days(days as i64 - 1);
    let start_str = start.format("%Y-%m-%d").to_string();
    let path = format!("/metrics-service/metrics/maxmet/daily/{start_str}/{end_date}");
    let v: serde_json::Value = client.get_json(&path).await?;

    let items: Vec<TrainingScore> = v
        .as_array()
        .map(|arr| arr.iter().map(training_score_from).collect())
        .unwrap_or_default();

    output.print_list(&items, "Training Scores (VO2max)");
    Ok(())
}

// ---------------------------------------------------------------------------
// 4. RacePredictions
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct RacePredictions {
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_5k_seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_5k: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pace_5k: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_10k_seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_10k: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pace_10k: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_half_marathon_seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_half_marathon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pace_half_marathon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_marathon_seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_marathon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pace_marathon: Option<String>,
}

fn race_predictions_from(v: &serde_json::Value) -> RacePredictions {
    let t5k = v["time5K"].as_f64();
    let t10k = v["time10K"].as_f64();
    let thm = v["timeHalfMarathon"].as_f64();
    let tm = v["timeMarathon"].as_f64();

    RacePredictions {
        date: v["calendarDate"]
            .as_str()
            .or(v["lastUpdated"].as_str())
            .unwrap_or("")
            .to_string(),
        time_5k_seconds: t5k,
        time_5k: t5k.map(fmt_duration),
        pace_5k: t5k.map(|s| fmt_pace(s, 5000.0)),
        time_10k_seconds: t10k,
        time_10k: t10k.map(fmt_duration),
        pace_10k: t10k.map(|s| fmt_pace(s, 10_000.0)),
        time_half_marathon_seconds: thm,
        time_half_marathon: thm.map(fmt_duration),
        pace_half_marathon: thm.map(|s| fmt_pace(s, 21_097.5)),
        time_marathon_seconds: tm,
        time_marathon: tm.map(fmt_duration),
        pace_marathon: tm.map(|s| fmt_pace(s, 42_195.0)),
    }
}

impl HumanReadable for RacePredictions {
    fn print_human(&self) {
        if self.date.is_empty() {
            println!("{}", "(no date)".dimmed());
        } else {
            println!("{}", self.date.bold());
        }
        print_race_line("5K:", self.time_5k_seconds, self.pace_5k.as_deref());
        print_race_line("10K:", self.time_10k_seconds, self.pace_10k.as_deref());
        print_race_line(
            "Half Marathon:",
            self.time_half_marathon_seconds,
            self.pace_half_marathon.as_deref(),
        );
        print_race_line(
            "Marathon:",
            self.time_marathon_seconds,
            self.pace_marathon.as_deref(),
        );
    }
}

fn print_race_line(name: &str, secs: Option<f64>, pace: Option<&str>) {
    if let Some(s) = secs {
        let time = fmt_duration(s);
        let pace_str = pace.map(|p| format!(" ({p} /km)")).unwrap_or_default();
        println!("  {name:<LABEL_WIDTH$}{}{pace_str}", time.cyan());
    }
}

pub async fn race_predictions(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let display_name = client.display_name().await?;
    let end_date = date.map(String::from).unwrap_or_else(today);
    let end = parse_date(&end_date)?;
    let days = days.unwrap_or(1);
    let start = end - chrono::Duration::days(days as i64 - 1);
    let start_str = start.format("%Y-%m-%d").to_string();

    let path = format!(
        "/metrics-service/metrics/racepredictions/daily/{display_name}?fromCalendarDate={start_str}&toCalendarDate={end_date}"
    );
    let v: serde_json::Value = client.get_json(&path).await?;

    let items: Vec<RacePredictions> = v
        .as_array()
        .map(|arr| arr.iter().map(race_predictions_from).collect())
        .unwrap_or_default();

    output.print_list(&items, "Race Predictions");
    Ok(())
}

// ---------------------------------------------------------------------------
// 5. EnduranceScore
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct EnduranceScore {
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classification: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback: Option<String>,
}

fn endurance_score_from(v: &serde_json::Value) -> EnduranceScore {
    let classification_int = v["classificationId"]
        .as_i64()
        .or(v["classification"].as_i64());
    EnduranceScore {
        date: v["calendarDate"].as_str().unwrap_or("").to_string(),
        score: v["overallScore"].as_i64(),
        classification: classification_int.map(|c| classification_label(c).to_string()),
        feedback: v["feedbackPhrase"].as_str().map(Into::into),
    }
}

impl HumanReadable for EnduranceScore {
    fn print_human(&self) {
        println!("{}", self.date.bold());
        let score = self
            .score
            .map(|s| s.to_string())
            .unwrap_or_else(|| "\u{2013}".into());
        let class = self.classification.as_deref().unwrap_or("?");
        println!("  {:<LABEL_WIDTH$}{} ({})", "Score:", score.cyan(), class);
        if let Some(ref fb) = self.feedback {
            println!("  {:<LABEL_WIDTH$}{fb}", "Feedback:");
        }
    }
}

pub async fn endurance_score(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let end_date = date.map(String::from).unwrap_or_else(today);
    let days = days.unwrap_or(1);
    let end = parse_date(&end_date)?;

    let futs: Vec<_> = (0..days)
        .rev()
        .map(|i| {
            let d = end - chrono::Duration::days(i as i64);
            let ds = d.format("%Y-%m-%d").to_string();
            let path = format!("/metrics-service/metrics/endurancescore?calendarDate={ds}");
            async move {
                let v: serde_json::Value = client.get_json(&path).await?;
                Ok(endurance_score_from(&v)) as Result<EnduranceScore>
            }
        })
        .collect();
    let items: Vec<EnduranceScore> = futures::future::join_all(futs)
        .await
        .into_iter()
        .collect::<Result<_>>()?;
    output.print_list(&items, "Endurance Score");
    Ok(())
}

// ---------------------------------------------------------------------------
// 6. HillScore
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct HillScore {
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overall: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strength: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endurance: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vo2max: Option<f64>,
}

fn hill_score_from(v: &serde_json::Value) -> HillScore {
    HillScore {
        date: v["calendarDate"].as_str().unwrap_or("").to_string(),
        overall: v["overallScore"].as_i64(),
        strength: v["strengthScore"].as_i64(),
        endurance: v["enduranceScore"].as_i64(),
        vo2max: v["vo2Max"].as_f64(),
    }
}

impl HumanReadable for HillScore {
    fn print_human(&self) {
        println!("{}", self.date.bold());
        if let Some(v) = self.overall {
            println!("  {:<LABEL_WIDTH$}{}", "Overall:", v.to_string().cyan());
        }
        if let Some(v) = self.strength {
            println!("  {:<LABEL_WIDTH$}{v}", "Strength:");
        }
        if let Some(v) = self.endurance {
            println!("  {:<LABEL_WIDTH$}{v}", "Endurance:");
        }
        if let Some(v) = self.vo2max {
            println!("  {:<LABEL_WIDTH$}{v:.1}", "VO2max:");
        }
    }
}

pub async fn hill_score(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let end_date = date.map(String::from).unwrap_or_else(today);
    let days = days.unwrap_or(1);
    let end = parse_date(&end_date)?;

    let futs: Vec<_> = (0..days)
        .rev()
        .map(|i| {
            let d = end - chrono::Duration::days(i as i64);
            let ds = d.format("%Y-%m-%d").to_string();
            let path = format!("/metrics-service/metrics/hillscore?calendarDate={ds}");
            async move {
                let v: serde_json::Value = client.get_json(&path).await?;
                Ok(hill_score_from(&v)) as Result<HillScore>
            }
        })
        .collect();
    let items: Vec<HillScore> = futures::future::join_all(futs)
        .await
        .into_iter()
        .collect::<Result<_>>()?;
    output.print_list(&items, "Hill Score");
    Ok(())
}

// ---------------------------------------------------------------------------
// 7. FitnessAge
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct FitnessAge {
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fitness_age: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chronological_age: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub achievable_fitness_age: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bmi: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resting_heart_rate: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vigorous_days_avg: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vigorous_minutes_avg: Option<f64>,
}

fn fitness_age_from(v: &serde_json::Value, date: &str) -> FitnessAge {
    FitnessAge {
        date: date.to_string(),
        fitness_age: v["fitnessAge"].as_f64(),
        chronological_age: v["chronologicalAge"].as_i64(),
        achievable_fitness_age: v["achievableFitnessAge"].as_f64(),
        bmi: v["components"]["bmi"]["value"].as_f64(),
        resting_heart_rate: v["components"]["rhr"]["value"].as_i64(),
        vigorous_days_avg: v["components"]["vigorousDaysAvg"]["value"].as_f64(),
        vigorous_minutes_avg: v["components"]["vigorousMinutesAvg"]["value"].as_f64(),
    }
}

impl HumanReadable for FitnessAge {
    fn print_human(&self) {
        println!("{}", self.date.bold());
        let fa = self
            .fitness_age
            .map(|v| format!("{v:.0}"))
            .unwrap_or_else(|| "\u{2013}".into());
        let ca = self
            .chronological_age
            .map(|v| v.to_string())
            .unwrap_or_else(|| "?".into());
        println!(
            "  {:<LABEL_WIDTH$}{} (chronological: {ca})",
            "Age:",
            fa.cyan()
        );
        if let Some(v) = self.achievable_fitness_age {
            println!("  {:<LABEL_WIDTH$}{v:.0}", "Achievable:");
        }
        if let Some(v) = self.bmi {
            println!("  {:<LABEL_WIDTH$}{v:.1}", "BMI:");
        }
        if let Some(v) = self.resting_heart_rate {
            println!("  {:<LABEL_WIDTH$}{v} bpm", "Resting HR:");
        }
        if let Some(v) = self.vigorous_days_avg {
            println!("  {:<LABEL_WIDTH$}{v:.1}", "Vigorous d/wk:");
        }
        if let Some(v) = self.vigorous_minutes_avg {
            println!("  {:<LABEL_WIDTH$}{v:.0}", "Vigorous m/d:");
        }
    }
}

pub async fn fitness_age(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let end_date = date.map(String::from).unwrap_or_else(today);
    let days = days.unwrap_or(1);
    let end = parse_date(&end_date)?;

    let futs: Vec<_> = (0..days)
        .rev()
        .map(|i| {
            let d = end - chrono::Duration::days(i as i64);
            let ds = d.format("%Y-%m-%d").to_string();
            let path = format!("/fitnessage-service/fitnessage/{ds}");
            async move {
                let v: serde_json::Value = client.get_json(&path).await?;
                Ok(fitness_age_from(&v, &ds)) as Result<FitnessAge>
            }
        })
        .collect();
    let items: Vec<FitnessAge> = futures::future::join_all(futs)
        .await
        .into_iter()
        .collect::<Result<_>>()?;
    output.print_list(&items, "Fitness Age");
    Ok(())
}

// ---------------------------------------------------------------------------
// 8. LactateThreshold
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct LactateThreshold {
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heart_rate: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed_meters_per_second: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pace: Option<String>,
}

/// Garmin's stats endpoint returns LT speed ~10x too low (e.g. 0.386 instead of 3.86 m/s).
/// Correct if value is implausibly small (< 1 m/s is walking speed).
fn correct_lt_speed(s: f64) -> f64 {
    if s > 0.0 && s < 1.0 { s * 10.0 } else { s }
}

fn lt_pace(speed: f64) -> String {
    let pace_secs = 1000.0 / speed;
    let m = pace_secs as u64 / 60;
    let sec = pace_secs as u64 % 60;
    format!("{m}:{sec:02}")
}

impl HumanReadable for LactateThreshold {
    fn print_human(&self) {
        println!("{}", self.date.bold());
        let hr = self
            .heart_rate
            .map(|v| format!("{v} bpm"))
            .unwrap_or_else(|| "–".into());
        let pace = self
            .pace
            .as_deref()
            .map(|p| format!("{p}/km"))
            .unwrap_or_else(|| "–".into());
        println!("  {:<LABEL_WIDTH$}{}", "Heart rate:", hr.cyan());
        println!("  {:<LABEL_WIDTH$}{}", "Pace:", pace.cyan());
    }
}

pub async fn lactate_threshold(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let end_date = date.map(String::from).unwrap_or_else(today);
    let end = parse_date(&end_date)?;
    let days = days.unwrap_or(7);
    let start = end - chrono::Duration::days(days as i64 - 1);
    let start_str = start.format("%Y-%m-%d").to_string();

    // Garmin's stats endpoint only returns rows on dates when LT changed, and
    // caps ranges at 366 days. Use the widest allowed lookback so we can
    // carry-forward the latest known value when the user's window is empty.
    let lookback = end - chrono::Duration::days(365);
    let lookback_str = lookback.format("%Y-%m-%d").to_string();

    let hr_path = format!(
        "/biometric-service/stats/lactateThresholdHeartRate/range/{lookback_str}/{end_date}?aggregation=daily&aggregationStrategy=LATEST&sport=RUNNING"
    );
    let speed_path = format!(
        "/biometric-service/stats/lactateThresholdSpeed/range/{lookback_str}/{end_date}?aggregation=daily&aggregationStrategy=LATEST&sport=RUNNING"
    );

    let (hr_v, speed_v) = tokio::try_join!(
        client.get_json::<serde_json::Value>(&hr_path),
        client.get_json::<serde_json::Value>(&speed_path),
    )?;

    // Merge HR and speed change-point series by their `updatedDate`.
    // HR and speed update independently, so we keyed-merge them.
    type LtRow = (Option<i64>, Option<f64>);
    let mut by_date: std::collections::BTreeMap<String, LtRow> = std::collections::BTreeMap::new();

    let row_date = |e: &serde_json::Value| -> Option<String> {
        e["updatedDate"]
            .as_str()
            .or(e["from"].as_str())
            .map(|s| s.chars().take(10).collect())
    };

    if let Some(arr) = hr_v.as_array() {
        for e in arr {
            if let (Some(d), Some(v)) = (row_date(e), e["value"].as_f64()) {
                by_date.entry(d).or_default().0 = Some(v as i64);
            }
        }
    }
    if let Some(arr) = speed_v.as_array() {
        for e in arr {
            if let (Some(d), Some(v)) = (row_date(e), e["value"].as_f64()) {
                by_date.entry(d).or_default().1 = Some(correct_lt_speed(v));
            }
        }
    }

    // Split into in-window change points and prior history. If the window has
    // changes, return them. Otherwise fall back to the most recent prior
    // change point so "current LT" is always reported.
    let (prior, in_window): (Vec<_>, Vec<_>) = by_date
        .into_iter()
        .partition(|(d, _)| d.as_str() < start_str.as_str());

    let rows: Vec<(String, LtRow)> = if in_window.is_empty() {
        prior.into_iter().last().into_iter().collect()
    } else {
        in_window
    };

    let items: Vec<LactateThreshold> = rows
        .into_iter()
        .map(|(date, (hr, speed))| LactateThreshold {
            date,
            heart_rate: hr,
            speed_meters_per_second: speed,
            pace: speed.map(lt_pace),
        })
        .collect();

    output.print_list(&items, "Lactate Threshold");
    Ok(())
}

// ---------------------------------------------------------------------------
// Heart Rate Zones
// ---------------------------------------------------------------------------
// Garmin doesn't expose user HR zone boundaries directly. We fetch them from
// the most recent running activity's hrTimeInZones data, which contains the
// zoneLowBoundary for each zone as configured on the device.

#[derive(Debug, Serialize)]
pub struct HrZoneBoundary {
    pub zone: i64,
    pub min_bpm: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_bpm: Option<i64>,
}

impl HumanReadable for HrZoneBoundary {
    fn print_human(&self) {
        let range = match self.max_bpm {
            Some(max) => format!("{}\u{2013}{} bpm", self.min_bpm, max),
            None => format!("{}+ bpm", self.min_bpm),
        };
        let label = format!("Zone {}", self.zone);
        println!("  {:<LABEL_WIDTH$}{}", label, range.cyan());
    }
}

pub async fn zones(client: &GarminClient, output: &Output) -> Result<()> {
    // Find the most recent running activity
    let activities: serde_json::Value = client
        .get_json("/activitylist-service/activities/search/activities?activityType=running&limit=1&start=0")
        .await?;
    let activity_id = activities
        .as_array()
        .and_then(|a| a.first())
        .and_then(|a| a["activityId"].as_u64())
        .ok_or_else(|| crate::error::Error::Api("No running activities found".into()))?;

    // Get HR zones from that activity
    let path = format!("/activity-service/activity/{activity_id}/hrTimeInZones");
    let v: serde_json::Value = client.get_json(&path).await?;
    let raw_zones: Vec<(i64, i64)> = v
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|z| {
                    let zone = z["zoneNumber"].as_i64()?;
                    let low = z["zoneLowBoundary"].as_i64()?;
                    Some((zone, low))
                })
                .collect()
        })
        .unwrap_or_default();

    // Build boundaries: each zone's max is the next zone's min - 1
    let mut boundaries: Vec<HrZoneBoundary> = Vec::new();
    for (i, &(zone, min_bpm)) in raw_zones.iter().enumerate() {
        let max_bpm = raw_zones.get(i + 1).map(|&(_, next_min)| next_min - 1);
        boundaries.push(HrZoneBoundary {
            zone,
            min_bpm,
            max_bpm,
        });
    }

    output.print_table(&boundaries, "HR Zones");
    Ok(())
}
