use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use crate::garmin::types::helpers::fmt_hms;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct HrZone {
    #[serde(default)]
    pub zone_number: i64,
    #[serde(rename(deserialize = "zoneLowBoundary"))]
    pub zone_low_boundary_bpm: Option<i64>,
    pub secs_in_zone: Option<f64>,
}

impl HumanReadable for HrZone {
    fn print_human(&self) {
        let hr_label = self
            .zone_low_boundary_bpm
            .map(|h| format!("{h}+ bpm"))
            .unwrap_or_else(|| "\u{2013}".into());
        let time = self.secs_in_zone.map(fmt_hms).unwrap_or_else(|| "\u{2013}".into());
        let label = format!("Zone {}", self.zone_number);
        println!("  {:<LABEL_WIDTH$}{hr_label}  {}", label, time.dimmed());
    }
}
