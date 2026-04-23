use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use colored::Colorize;
use serde::Serialize;
use serde_with::skip_serializing_none;

/// Derived from an activity's `hrTimeInZones`.
#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct HrZoneBoundary {
    pub zone: i64,
    pub min_bpm: i64,
    pub max_bpm: Option<i64>,
}

impl HumanReadable for HrZoneBoundary {
    fn print_human(&self) {
        let range = match self.max_bpm {
            Some(max) => format!("{}\u{2013}{} bpm", self.min_bpm, max),
            None => format!("{}+ bpm", self.min_bpm),
        };
        let label = format!("Zone {}", self.zone);
        println!("  {:<LABEL_WIDTH$}{}", label, range.cyan());
    }
}
