use std::collections::BTreeMap;

use super::labels::fitness_trend_label;
use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

// The API nests the payload by device id under
// `mostRecentTrainingStatus.latestTrainingStatusData.{device-id}`. Custom
// deserializer picks the first device entry and flattens the three top-level
// blocks (status / load balance / VO2max) into `TrainingStatus`.

#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct TrainingStatus {
    pub date: String,
    pub status: Option<String>,
    pub fitness_trend: Option<String>,
    pub fitness_trend_sport: Option<String>,
    pub training_paused: Option<bool>,
    pub since_date: Option<String>,
    pub acute_load: Option<f64>,
    pub chronic_load: Option<f64>,
    pub acwr: Option<f64>,
    pub acwr_status: Option<String>,
    pub min_training_load_chronic: Option<f64>,
    pub max_training_load_chronic: Option<f64>,
    pub monthly_load_aerobic_high: Option<f64>,
    pub monthly_load_aerobic_high_target_min: Option<i64>,
    pub monthly_load_aerobic_high_target_max: Option<i64>,
    pub monthly_load_aerobic_low: Option<f64>,
    pub monthly_load_aerobic_low_target_min: Option<i64>,
    pub monthly_load_aerobic_low_target_max: Option<i64>,
    pub monthly_load_anaerobic: Option<f64>,
    pub monthly_load_anaerobic_target_min: Option<i64>,
    pub monthly_load_anaerobic_target_max: Option<i64>,
    pub load_balance_feedback: Option<String>,
    pub vo2max: Option<f64>,
    pub vo2max_date: Option<String>,
}

impl<'de> Deserialize<'de> for TrainingStatus {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Raw {
            most_recent_training_status: Option<StatusWrapper>,
            most_recent_training_load_balance: Option<LoadWrapper>,
            #[serde(rename = "mostRecentVO2Max")]
            most_recent_vo2_max: Option<Vo2Wrapper>,
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct StatusWrapper {
            #[serde(default)]
            latest_training_status_data: BTreeMap<String, StatusData>,
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct StatusData {
            calendar_date: Option<String>,
            training_status_feedback_phrase: Option<String>,
            fitness_trend: Option<i64>,
            fitness_trend_sport: Option<String>,
            training_paused: Option<bool>,
            since_date: Option<String>,
            #[serde(rename = "acuteTrainingLoadDTO")]
            acute_training_load: Option<AcuteLoad>,
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct AcuteLoad {
            daily_training_load_acute: Option<f64>,
            daily_training_load_chronic: Option<f64>,
            daily_acute_chronic_workload_ratio: Option<f64>,
            acwr_status: Option<String>,
            min_training_load_chronic: Option<f64>,
            max_training_load_chronic: Option<f64>,
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct LoadWrapper {
            #[serde(default, rename = "metricsTrainingLoadBalanceDTOMap")]
            metrics_training_load_balance: BTreeMap<String, LoadBalance>,
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct LoadBalance {
            monthly_load_aerobic_high: Option<f64>,
            monthly_load_aerobic_high_target_min: Option<i64>,
            monthly_load_aerobic_high_target_max: Option<i64>,
            monthly_load_aerobic_low: Option<f64>,
            monthly_load_aerobic_low_target_min: Option<i64>,
            monthly_load_aerobic_low_target_max: Option<i64>,
            monthly_load_anaerobic: Option<f64>,
            monthly_load_anaerobic_target_min: Option<i64>,
            monthly_load_anaerobic_target_max: Option<i64>,
            training_balance_feedback_phrase: Option<String>,
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Vo2Wrapper {
            generic: Option<Vo2Generic>,
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Vo2Generic {
            vo2_max_precise_value: Option<f64>,
            vo2_max_value: Option<f64>,
            calendar_date: Option<String>,
        }

        let raw = Raw::deserialize(d)?;
        // Pick the first device entry from each keyed map.
        let sd = raw
            .most_recent_training_status
            .and_then(|w| w.latest_training_status_data.into_values().next());
        let lb = raw
            .most_recent_training_load_balance
            .and_then(|w| w.metrics_training_load_balance.into_values().next());
        let vo2 = raw.most_recent_vo2_max.and_then(|v| v.generic);
        let acute = sd.as_ref().and_then(|d| d.acute_training_load.as_ref());

        Ok(TrainingStatus {
            date: sd.as_ref().and_then(|d| d.calendar_date.clone()).unwrap_or_default(),
            status: sd.as_ref().and_then(|d| d.training_status_feedback_phrase.clone()),
            fitness_trend: sd
                .as_ref()
                .and_then(|d| d.fitness_trend)
                .map(|c| fitness_trend_label(c).to_string()),
            fitness_trend_sport: sd.as_ref().and_then(|d| d.fitness_trend_sport.clone()),
            training_paused: sd.as_ref().and_then(|d| d.training_paused),
            since_date: sd.as_ref().and_then(|d| d.since_date.clone()),
            acute_load: acute.and_then(|a| a.daily_training_load_acute),
            chronic_load: acute.and_then(|a| a.daily_training_load_chronic),
            acwr: acute.and_then(|a| a.daily_acute_chronic_workload_ratio),
            acwr_status: acute.and_then(|a| a.acwr_status.clone()),
            min_training_load_chronic: acute.and_then(|a| a.min_training_load_chronic),
            max_training_load_chronic: acute.and_then(|a| a.max_training_load_chronic),
            monthly_load_aerobic_high: lb.as_ref().and_then(|b| b.monthly_load_aerobic_high),
            monthly_load_aerobic_high_target_min: lb.as_ref().and_then(|b| b.monthly_load_aerobic_high_target_min),
            monthly_load_aerobic_high_target_max: lb.as_ref().and_then(|b| b.monthly_load_aerobic_high_target_max),
            monthly_load_aerobic_low: lb.as_ref().and_then(|b| b.monthly_load_aerobic_low),
            monthly_load_aerobic_low_target_min: lb.as_ref().and_then(|b| b.monthly_load_aerobic_low_target_min),
            monthly_load_aerobic_low_target_max: lb.as_ref().and_then(|b| b.monthly_load_aerobic_low_target_max),
            monthly_load_anaerobic: lb.as_ref().and_then(|b| b.monthly_load_anaerobic),
            monthly_load_anaerobic_target_min: lb.as_ref().and_then(|b| b.monthly_load_anaerobic_target_min),
            monthly_load_anaerobic_target_max: lb.as_ref().and_then(|b| b.monthly_load_anaerobic_target_max),
            load_balance_feedback: lb.and_then(|b| b.training_balance_feedback_phrase),
            vo2max: vo2.as_ref().and_then(|v| v.vo2_max_precise_value.or(v.vo2_max_value)),
            vo2max_date: vo2.and_then(|v| v.calendar_date),
        })
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
            let acute = self.acute_load.map(|v| format!("{v:.0}")).unwrap_or_default();
            let chronic = self.chronic_load.map(|v| format!("{v:.0}")).unwrap_or_default();
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
                println!("    {:<LABEL_WIDTH$}{ah:.0} (target: {min}–{max})", "Aerobic high:");
            }
            if let Some(al) = self.monthly_load_aerobic_low {
                let min = self.monthly_load_aerobic_low_target_min.unwrap_or(0);
                let max = self.monthly_load_aerobic_low_target_max.unwrap_or(0);
                println!("    {:<LABEL_WIDTH$}{al:.0} (target: {min}–{max})", "Aerobic low:");
            }
            if let Some(an) = self.monthly_load_anaerobic {
                let min = self.monthly_load_anaerobic_target_min.unwrap_or(0);
                let max = self.monthly_load_anaerobic_target_max.unwrap_or(0);
                println!("    {:<LABEL_WIDTH$}{an:.0} (target: {min}–{max})", "Anaerobic:");
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
