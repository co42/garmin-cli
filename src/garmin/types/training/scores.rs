use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct TrainingScore {
    #[serde(default)]
    pub date: String,
    pub vo2max: Option<f64>,
    pub fitness_age: Option<f64>,
}

/// API wrapper — shape is `{"generic": {...}, "calendarDate": ...}` per entry.
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct TrainingScoreRaw {
    pub generic: Option<TrainingScoreGeneric>,
    pub calendar_date: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct TrainingScoreGeneric {
    pub calendar_date: Option<String>,
    pub vo2_max_precise_value: Option<f64>,
    pub vo2_max_value: Option<f64>,
    pub fitness_age: Option<f64>,
}

impl From<TrainingScoreRaw> for TrainingScore {
    fn from(r: TrainingScoreRaw) -> Self {
        let g = r.generic;
        Self {
            date: g
                .as_ref()
                .and_then(|g| g.calendar_date.clone())
                .or(r.calendar_date)
                .unwrap_or_default(),
            vo2max: g.as_ref().and_then(|g| g.vo2_max_precise_value.or(g.vo2_max_value)),
            fitness_age: g.and_then(|g| g.fitness_age),
        }
    }
}

impl HumanReadable for TrainingScore {
    fn print_human(&self) {
        println!("{}", self.date.bold());
        let vo2 = self.vo2max.map(|v| format!("{v:.1}")).unwrap_or_else(|| "–".into());
        println!("  {:<LABEL_WIDTH$}{}", "VO2max:", vo2.cyan());
        if let Some(age) = self.fitness_age {
            println!("  {:<LABEL_WIDTH$}{age:.0}", "Fitness age:");
        }
    }
}
