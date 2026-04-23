use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct HrvSummary {
    /// The API returns either a datetime string (`startTimestampLocal`) or
    /// `calendarDate`; take whichever is present.
    #[serde(default, alias = "startTimestampLocal")]
    pub calendar_date: String,
    pub hrv_summary: Option<HrvSummaryInner>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct HrvSummaryInner {
    #[serde(alias = "lastNight")]
    pub last_night_avg: Option<i64>,
    pub last_night5_min_high: Option<i64>,
    pub weekly_avg: Option<i64>,
    pub status: Option<String>,
    pub baseline: Option<HrvBaseline>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct HrvBaseline {
    pub balanced_low: Option<i64>,
    pub balanced_upper: Option<i64>,
}

impl HumanReadable for HrvSummary {
    fn print_human(&self) {
        let date = if self.calendar_date.len() >= 10 {
            &self.calendar_date[..10]
        } else {
            &self.calendar_date
        };
        println!("{}", date.bold());
        if let Some(s) = &self.hrv_summary {
            if let Some(v) = s.last_night_avg {
                println!("  {:<LABEL_WIDTH$}{} ms", "Last night:", v.to_string().cyan());
            }
            if let Some(v) = s.last_night5_min_high {
                println!("  {:<LABEL_WIDTH$}{v} ms", "5-min high:");
            }
            if let Some(v) = s.weekly_avg {
                println!("  {:<LABEL_WIDTH$}{v} ms", "Weekly avg:");
            }
            if let Some(ref st) = s.status {
                println!("  {:<LABEL_WIDTH$}{st}", "Status:");
            }
            if let Some(b) = &s.baseline
                && let (Some(lo), Some(hi)) = (b.balanced_low, b.balanced_upper)
            {
                println!("  {:<LABEL_WIDTH$}{lo}\u{2013}{hi} ms", "Baseline:");
            }
        }
    }
}
