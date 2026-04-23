use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use crate::garmin::types::helpers::{fmt_hm, fmt_local_time};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Garmin returns sleep data under a `dailySleepDTO` key alongside other
/// arrays we don't consume; the custom deserializer unwraps that so callers
/// see the fields flat, matching every other health endpoint.
#[skip_serializing_none]
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SleepSummary {
    #[serde(default)]
    pub calendar_date: String,
    pub sleep_scores: Option<SleepScores>,
    pub sleep_time_seconds: Option<u64>,
    pub deep_sleep_seconds: Option<u64>,
    pub light_sleep_seconds: Option<u64>,
    pub rem_sleep_seconds: Option<u64>,
    pub awake_sleep_seconds: Option<u64>,
    pub sleep_start_timestamp_local: Option<i64>,
    pub sleep_end_timestamp_local: Option<i64>,
    pub sleep_need: Option<SleepNeed>,
}

impl<'de> Deserialize<'de> for SleepSummary {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Inner {
            #[serde(default)]
            calendar_date: String,
            sleep_scores: Option<SleepScores>,
            sleep_time_seconds: Option<u64>,
            deep_sleep_seconds: Option<u64>,
            light_sleep_seconds: Option<u64>,
            rem_sleep_seconds: Option<u64>,
            awake_sleep_seconds: Option<u64>,
            sleep_start_timestamp_local: Option<i64>,
            sleep_end_timestamp_local: Option<i64>,
            sleep_need: Option<SleepNeed>,
        }
        #[derive(Deserialize)]
        struct Wrapper {
            #[serde(rename = "dailySleepDTO")]
            inner: Inner,
        }
        let Wrapper { inner } = Wrapper::deserialize(d)?;
        Ok(SleepSummary {
            calendar_date: inner.calendar_date,
            sleep_scores: inner.sleep_scores,
            sleep_time_seconds: inner.sleep_time_seconds,
            deep_sleep_seconds: inner.deep_sleep_seconds,
            light_sleep_seconds: inner.light_sleep_seconds,
            rem_sleep_seconds: inner.rem_sleep_seconds,
            awake_sleep_seconds: inner.awake_sleep_seconds,
            sleep_start_timestamp_local: inner.sleep_start_timestamp_local,
            sleep_end_timestamp_local: inner.sleep_end_timestamp_local,
            sleep_need: inner.sleep_need,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SleepScores {
    pub overall: Option<SleepScoreValue>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct SleepScoreValue {
    pub value: Option<i64>,
    pub qualifier_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SleepNeed {
    /// Minutes.
    pub actual: Option<u64>,
}

impl HumanReadable for SleepSummary {
    fn print_human(&self) {
        let d = self;
        println!("{}", d.calendar_date.bold());
        if let Some(score) = d.sleep_scores.as_ref().and_then(|s| s.overall.as_ref()) {
            let qualifier = score
                .qualifier_key
                .as_deref()
                .map(|q| format!(" ({q})"))
                .unwrap_or_default();
            if let Some(v) = score.value {
                println!("  {:<LABEL_WIDTH$}{}{}", "Score:", v.to_string().cyan(), qualifier);
            }
        }
        if let Some(s) = d.sleep_time_seconds {
            println!("  {:<LABEL_WIDTH$}{}", "Duration:", fmt_hm(s).cyan());
        }
        let parts: Vec<String> = [
            d.deep_sleep_seconds.map(|s| format!("Deep: {}", fmt_hm(s))),
            d.light_sleep_seconds.map(|s| format!("Light: {}", fmt_hm(s))),
            d.rem_sleep_seconds.map(|s| format!("REM: {}", fmt_hm(s))),
            d.awake_sleep_seconds.map(|s| format!("Awake: {}", fmt_hm(s))),
        ]
        .into_iter()
        .flatten()
        .collect();
        if !parts.is_empty() {
            println!("  {:<LABEL_WIDTH$}{}", "Stages:", parts.join("  "));
        }
        let start = fmt_local_time(d.sleep_start_timestamp_local);
        let end = fmt_local_time(d.sleep_end_timestamp_local);
        if let (Some(s), Some(e)) = (start, end) {
            println!("  {:<LABEL_WIDTH$}{s} \u{2013} {e}", "Window:");
        }
        if let Some(need_min) = d.sleep_need.as_ref().and_then(|n| n.actual) {
            println!("  {:<LABEL_WIDTH$}{}", "Need:", fmt_hm(need_min * 60));
        }
    }
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct SleepScore {
    #[serde(default)]
    pub calendar_date: String,
    pub value: Option<i64>,
}

impl HumanReadable for SleepScore {
    fn print_human(&self) {
        println!("{}", self.calendar_date.bold());
        let score_str = self.value.map(|s| s.to_string()).unwrap_or_else(|| "-".into());
        println!("  {:<LABEL_WIDTH$}{}", "Score:", score_str.cyan());
    }
}
