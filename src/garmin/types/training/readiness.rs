use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

// TODO: The API returns a heterogeneous array with different inputContexts
// (AFTER_WAKEUP_RESET / AFTER_POST_EXERCISE_RESET / UPDATE_REALTIME_VARIABLES).
// The command used to pick morning/post-activity/latest. For now we return
// all entries and expose a helper on DailyReadiness.

pub type TrainingReadinessResponse = Vec<TrainingReadinessEntry>;

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct TrainingReadinessEntry {
    pub input_context: Option<String>,
    pub timestamp_local: Option<String>,
    pub score: Option<i64>,
    pub level: Option<String>,
    pub feedback_short: Option<String>,
    pub recovery_time: Option<i64>,
    pub hrv_weekly_average: Option<i64>,
    pub hrv_factor_percent: Option<i64>,
    pub hrv_factor_feedback: Option<String>,
    pub sleep_history_factor_percent: Option<i64>,
    pub sleep_history_factor_feedback: Option<String>,
    pub sleep_score_factor_percent: Option<i64>,
    pub sleep_score_factor_feedback: Option<String>,
    pub recovery_time_factor_percent: Option<i64>,
    pub recovery_time_factor_feedback: Option<String>,
    pub acwr_factor_percent: Option<i64>,
    pub acwr_factor_feedback: Option<String>,
    pub stress_history_factor_percent: Option<i64>,
    pub stress_history_factor_feedback: Option<String>,
}

/// Display-only wrapper showing one day's readiness entries grouped by context.
#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct DailyReadiness {
    pub date: String,
    pub morning: Option<TrainingReadinessEntry>,
    pub post_activity: Option<TrainingReadinessEntry>,
    pub latest: Option<TrainingReadinessEntry>,
}

impl DailyReadiness {
    pub fn from_entries(entries: Vec<TrainingReadinessEntry>, date: &str) -> Self {
        let mut morning = None;
        let mut post_activity = None;
        let mut latest = None;

        for entry in entries {
            match entry.input_context.as_deref() {
                Some("AFTER_WAKEUP_RESET") => morning = Some(entry),
                Some("AFTER_POST_EXERCISE_RESET") => post_activity = Some(entry),
                Some("UPDATE_REALTIME_VARIABLES") => latest = Some(entry),
                _ => {
                    if morning.is_none() {
                        morning = Some(entry);
                    }
                }
            }
        }

        // Drop latest if older than morning/post-activity (stale carry-over).
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

        Self {
            date: date.to_string(),
            morning,
            post_activity,
            latest,
        }
    }
}

impl TrainingReadinessEntry {
    fn print_section(&self, label: &str) {
        let score_str = self.score.map(|s| format!("{s}/100")).unwrap_or_else(|| "?".into());
        let level = self.level.as_deref().unwrap_or("?");
        println!("  {:<LABEL_WIDTH$}{} ({})", label, score_str.cyan(), level);
        if let Some(ref fb) = self.feedback_short {
            println!("  {:<LABEL_WIDTH$}{fb}", "");
        }
        let mut parts = Vec::new();
        if let Some(rt) = self.recovery_time {
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
        print_factor("HRV", self.hrv_factor_percent, self.hrv_factor_feedback.as_deref());
        print_factor(
            "Sleep history",
            self.sleep_history_factor_percent,
            self.sleep_history_factor_feedback.as_deref(),
        );
        print_factor(
            "Sleep",
            self.sleep_score_factor_percent,
            self.sleep_score_factor_feedback.as_deref(),
        );
        print_factor(
            "Recovery",
            self.recovery_time_factor_percent,
            self.recovery_time_factor_feedback.as_deref(),
        );
        print_factor("ACWR", self.acwr_factor_percent, self.acwr_factor_feedback.as_deref());
        print_factor(
            "Stress",
            self.stress_history_factor_percent,
            self.stress_history_factor_feedback.as_deref(),
        );
    }
}

fn print_factor(name: &str, score: Option<i64>, feedback: Option<&str>) {
    if let Some(s) = score {
        let fb = feedback.unwrap_or("?");
        println!("    {name:<LABEL_WIDTH$}{s:>3}% ({fb})");
    }
}

impl HumanReadable for TrainingReadinessEntry {
    fn print_human(&self) {
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
