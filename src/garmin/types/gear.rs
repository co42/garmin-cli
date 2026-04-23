use super::helpers::unknown;
use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct GearItem {
    #[serde(default)]
    pub uuid: String,
    #[serde(default = "unknown")]
    pub display_name: String,
    #[serde(alias = "gearType")]
    pub gear_type_name: Option<String>,
    /// API name `customMakeModel` is a cryptic internal label; `brand` is what
    /// users expect. Alias keeps the alternative brand-key compatible.
    #[serde(rename(deserialize = "customMakeModel"), alias = "brand")]
    pub brand: Option<String>,
    pub model: Option<String>,
    /// Bulk listing does not include distance; filled in from stats endpoint by the command.
    pub distance_meters: Option<f64>,
    /// Same — filled in from stats endpoint.
    pub activities: Option<i64>,
    pub date_begin: Option<String>,
    pub maximum_meters: Option<f64>,
    pub gear_status_name: Option<String>,
    pub date_end: Option<String>,
}

impl HumanReadable for GearItem {
    fn print_human(&self) {
        let retired = self.gear_status_name.as_deref() == Some("retired");
        if retired {
            println!("{} {}", self.display_name.bold().dimmed(), "(retired)".dimmed());
        } else {
            println!("{}", self.display_name.bold());
        }
        println!("  {:<LABEL_WIDTH$}{}", "UUID:", self.uuid.dimmed());
        match (self.distance_meters, self.maximum_meters) {
            (Some(d), Some(m)) if m > 0.0 => println!(
                "  {:<LABEL_WIDTH$}{:.0} / {:.0} km",
                "Distance:",
                d / 1000.0,
                m / 1000.0
            ),
            (Some(d), _) => println!("  {:<LABEL_WIDTH$}{:.0} km", "Distance:", d / 1000.0),
            _ => {}
        }
        if let Some(count) = self.activities {
            println!("  {:<LABEL_WIDTH$}{count}", "Activities:");
        }
        if let Some(ref date) = self.date_begin {
            let short = &date[..date.len().min(10)];
            println!("  {:<LABEL_WIDTH$}{short}", "Since:");
        }
    }
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct GearStats {
    /// Not returned by the API — filled in by the client after deserialization.
    #[serde(default)]
    pub uuid: String,
    #[serde(rename(deserialize = "totalDistance"), alias = "distance")]
    pub total_distance_meters: Option<f64>,
    #[serde(alias = "activities")]
    pub total_activities: Option<i64>,
    #[serde(rename(deserialize = "totalDuration"), alias = "duration")]
    pub total_duration_seconds: Option<f64>,
}

impl HumanReadable for GearStats {
    fn print_human(&self) {
        println!("{}", "Gear Stats".bold());
        println!("{}", "\u{2500}".repeat(40).dimmed());
        println!("  {:<LABEL_WIDTH$}{}", "UUID:", self.uuid.dimmed());
        if let Some(dist) = self.total_distance_meters {
            println!("  {:<LABEL_WIDTH$}{:.1} km", "Distance:", dist / 1000.0);
        }
        if let Some(count) = self.total_activities {
            println!("  {:<LABEL_WIDTH$}{count}", "Activities:");
        }
        if let Some(dur) = self.total_duration_seconds {
            let hours = (dur / 3600.0).floor() as u32;
            let mins = ((dur % 3600.0) / 60.0).round() as u32;
            println!("  {:<LABEL_WIDTH$}{hours}h {mins}min", "Duration:");
        }
    }
}
