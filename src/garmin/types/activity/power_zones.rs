use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use crate::garmin::types::helpers::fmt_hms;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

// API may return `[{zones: [...]}]` OR `[...]` directly. TODO: accept both;
// for now we target the nested-zones shape via PowerZoneGroup.

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PowerZoneGroup {
    #[serde(default)]
    pub zones: Vec<PowerZone>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PowerZone {
    #[serde(default)]
    pub zone_number: i64,
    /// Alternate API shape uses `minWatts`.
    #[serde(alias = "minWatts")]
    pub zone_low_boundary: Option<f64>,
    /// Alternate API shape uses `maxWatts`.
    #[serde(alias = "maxWatts")]
    pub zone_high_boundary: Option<f64>,
    /// Alternate API shape uses `secondsInZone`.
    #[serde(alias = "secondsInZone")]
    pub secs_in_zone: Option<f64>,
}

impl HumanReadable for PowerZone {
    fn print_human(&self) {
        let range = match (self.zone_low_boundary, self.zone_high_boundary) {
            (Some(lo), Some(hi)) => format!("{:.0}\u{2013}{:.0} W", lo, hi),
            (Some(lo), None) => format!("{:.0} W+", lo),
            _ => "\u{2014}".into(),
        };
        let time = self.secs_in_zone.map(fmt_hms).unwrap_or_else(|| "\u{2014}".into());
        let label = format!("Zone {}", self.zone_number);
        println!("  {:<LABEL_WIDTH$}{range}  {}", label, time.dimmed());
    }
}
