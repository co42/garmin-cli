use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use crate::garmin::types::helpers::{compute_pace, fmt_hms};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RacePredictionsRaw {
    pub calendar_date: Option<String>,
    pub last_updated: Option<String>,
    #[serde(rename(deserialize = "time5K"))]
    pub time_5k: Option<f64>,
    #[serde(rename(deserialize = "time10K"))]
    pub time_10k: Option<f64>,
    pub time_half_marathon: Option<f64>,
    pub time_marathon: Option<f64>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct RacePredictions {
    pub date: String,
    pub time_5k_seconds: Option<f64>,
    pub time_5k: Option<String>,
    pub pace_5k: Option<String>,
    pub time_10k_seconds: Option<f64>,
    pub time_10k: Option<String>,
    pub pace_10k: Option<String>,
    pub time_half_marathon_seconds: Option<f64>,
    pub time_half_marathon: Option<String>,
    pub pace_half_marathon: Option<String>,
    pub time_marathon_seconds: Option<f64>,
    pub time_marathon: Option<String>,
    pub pace_marathon: Option<String>,
}

impl From<RacePredictionsRaw> for RacePredictions {
    fn from(r: RacePredictionsRaw) -> Self {
        Self {
            date: r.calendar_date.or(r.last_updated).unwrap_or_default(),
            time_5k_seconds: r.time_5k,
            time_5k: r.time_5k.map(fmt_hms),
            pace_5k: r.time_5k.and_then(|s| compute_pace(Some(5000.0), s)),
            time_10k_seconds: r.time_10k,
            time_10k: r.time_10k.map(fmt_hms),
            pace_10k: r.time_10k.and_then(|s| compute_pace(Some(10_000.0), s)),
            time_half_marathon_seconds: r.time_half_marathon,
            time_half_marathon: r.time_half_marathon.map(fmt_hms),
            pace_half_marathon: r.time_half_marathon.and_then(|s| compute_pace(Some(21_097.5), s)),
            time_marathon_seconds: r.time_marathon,
            time_marathon: r.time_marathon.map(fmt_hms),
            pace_marathon: r.time_marathon.and_then(|s| compute_pace(Some(42_195.0), s)),
        }
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
        print_race_line("Marathon:", self.time_marathon_seconds, self.pace_marathon.as_deref());
    }
}

fn print_race_line(name: &str, secs: Option<f64>, pace: Option<&str>) {
    if let Some(s) = secs {
        let time = fmt_hms(s);
        let pace_str = pace.map(|p| format!(" ({p})")).unwrap_or_default();
        println!("  {name:<LABEL_WIDTH$}{}{pace_str}", time.cyan());
    }
}
