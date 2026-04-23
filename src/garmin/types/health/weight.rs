use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// `/weight-service/weight/dateRange` returns `{dateWeightList: [...]}`.
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct WeightRange {
    #[serde(default)]
    pub date_weight_list: Vec<WeightEntry>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct WeightEntry {
    #[serde(default)]
    pub calendar_date: String,
    #[serde(
        rename(deserialize = "weight"),
        default,
        deserialize_with = "crate::garmin::types::helpers::deser_g_to_kg"
    )]
    pub weight_kg: Option<f64>,
    pub bmi: Option<f64>,
    pub body_fat_percent: Option<f64>,
    #[serde(
        rename(deserialize = "muscleMass"),
        default,
        deserialize_with = "crate::garmin::types::helpers::deser_g_to_kg"
    )]
    pub muscle_mass_kg: Option<f64>,
    #[serde(
        rename(deserialize = "boneMass"),
        default,
        deserialize_with = "crate::garmin::types::helpers::deser_g_to_kg"
    )]
    pub bone_mass_kg: Option<f64>,
    #[serde(rename(deserialize = "bodyWater"))]
    pub body_water_percent: Option<f64>,
}

impl HumanReadable for WeightEntry {
    fn print_human(&self) {
        println!("{}", self.calendar_date.bold());
        let Some(w) = self.weight_kg else {
            println!("  No data");
            return;
        };
        println!("  {:<LABEL_WIDTH$}{} kg", "Weight:", format!("{w:.1}").cyan());
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
