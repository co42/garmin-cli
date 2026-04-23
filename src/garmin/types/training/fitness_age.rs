use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct FitnessAgeRaw {
    pub fitness_age: Option<f64>,
    pub chronological_age: Option<i64>,
    pub achievable_fitness_age: Option<f64>,
    pub components: Option<FitnessAgeComponents>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct FitnessAgeComponents {
    pub bmi: Option<FitnessAgeComponent<f64>>,
    pub rhr: Option<FitnessAgeComponent<i64>>,
    pub vigorous_days_avg: Option<FitnessAgeComponent<f64>>,
    pub vigorous_minutes_avg: Option<FitnessAgeComponent<f64>>,
}

#[derive(Debug, Deserialize)]
pub struct FitnessAgeComponent<T> {
    pub value: Option<T>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct FitnessAge {
    pub date: String,
    pub fitness_age: Option<f64>,
    pub chronological_age: Option<i64>,
    pub achievable_fitness_age: Option<f64>,
    pub bmi: Option<f64>,
    pub resting_heart_rate: Option<i64>,
    pub vigorous_days_avg: Option<f64>,
    pub vigorous_minutes_avg: Option<f64>,
}

impl FitnessAge {
    pub fn from_raw(r: FitnessAgeRaw, date: &str) -> Self {
        let c = r.components;
        Self {
            date: date.to_string(),
            fitness_age: r.fitness_age,
            chronological_age: r.chronological_age,
            achievable_fitness_age: r.achievable_fitness_age,
            bmi: c.as_ref().and_then(|c| c.bmi.as_ref()).and_then(|c| c.value),
            resting_heart_rate: c.as_ref().and_then(|c| c.rhr.as_ref()).and_then(|c| c.value),
            vigorous_days_avg: c
                .as_ref()
                .and_then(|c| c.vigorous_days_avg.as_ref())
                .and_then(|c| c.value),
            vigorous_minutes_avg: c
                .as_ref()
                .and_then(|c| c.vigorous_minutes_avg.as_ref())
                .and_then(|c| c.value),
        }
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
        println!("  {:<LABEL_WIDTH$}{} (chronological: {ca})", "Age:", fa.cyan());
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
