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
    pub time_10k_seconds: Option<f64>,
    pub time_half_marathon_seconds: Option<f64>,
    pub time_marathon_seconds: Option<f64>,
}

impl From<RacePredictionsRaw> for RacePredictions {
    fn from(r: RacePredictionsRaw) -> Self {
        Self {
            date: r.calendar_date.or(r.last_updated).unwrap_or_default(),
            time_5k_seconds: r.time_5k,
            time_10k_seconds: r.time_10k,
            time_half_marathon_seconds: r.time_half_marathon,
            time_marathon_seconds: r.time_marathon,
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
        print_race_line("5K:", self.time_5k_seconds, 5_000.0);
        print_race_line("10K:", self.time_10k_seconds, 10_000.0);
        print_race_line("Half Marathon:", self.time_half_marathon_seconds, 21_097.5);
        print_race_line("Marathon:", self.time_marathon_seconds, 42_195.0);
    }
}

fn print_race_line(name: &str, secs: Option<f64>, distance_m: f64) {
    if let Some(s) = secs {
        let time = fmt_hms(s);
        let pace_str = compute_pace(Some(distance_m), s)
            .map(|p| format!(" ({p})"))
            .unwrap_or_default();
        println!("  {name:<LABEL_WIDTH$}{}{pace_str}", time.cyan());
    }
}
