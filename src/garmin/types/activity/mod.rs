mod detail;
mod hr_zones;
mod lap;
mod power_zones;
mod split;
mod summary;
mod weather;

pub use detail::*;
pub use hr_zones::*;
pub use lap::*;
pub use power_zones::*;
pub use split::*;
pub use summary::*;
pub use weather::*;

use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use crate::garmin::types::helpers::fmt_hms;
use colored::Colorize;
use serde::Serialize;

/// Merged view returned by `activities get`. Both halves are flattened so the
/// JSON output is a single object instead of `{summary: {...}, detail: {...}}`.
#[derive(Debug, Serialize)]
pub struct Activity {
    #[serde(flatten)]
    pub summary: ActivitySummary,
    #[serde(flatten)]
    pub detail: ActivityDetail,
}

fn workout_feel_label(code: i64) -> &'static str {
    // Values on the watch are 0 (Very Weak) to 100 (Very Strong) in steps of 25.
    match code {
        0..=10 => "very weak",
        11..=35 => "weak",
        36..=60 => "normal",
        61..=85 => "strong",
        _ => "very strong",
    }
}

impl HumanReadable for Activity {
    fn print_human(&self) {
        self.summary.print_human();
        let d = &self.detail;

        // Collect each detail line lazily so we can skip the section entirely
        // when nothing is populated (e.g. old activities without stamina data).
        let mut lines: Vec<String> = Vec::new();
        if let Some(min) = d.min_hr {
            lines.push(format!("  {:<LABEL_WIDTH$}{min:.0} bpm", "Min HR:"));
        }
        if let Some(norm) = d.normalized_power {
            let min = d.min_power.map(|p| format!(" (min {p:.0} W)")).unwrap_or_default();
            lines.push(format!("  {:<LABEL_WIDTH$}{norm:.0} W{min}", "Norm Power:"));
        }
        if let Some(work) = d.total_work_joules {
            lines.push(format!("  {:<LABEL_WIDTH$}{:.0} kJ", "Work:", work / 1000.0));
        }
        if let Some(impact) = d.impact_load {
            lines.push(format!("  {:<LABEL_WIDTH$}{impact:.0}", "Impact Load:"));
        }
        // Speed & cadence peaks (m/s → km/h for display).
        if let Some(max) = d.max_speed_mps {
            lines.push(format!("  {:<LABEL_WIDTH$}{:.1} km/h", "Max Speed:", max * 3.6));
        }
        if let Some(max) = d.max_run_cadence {
            lines.push(format!("  {:<LABEL_WIDTH$}{max:.0} spm", "Max Cadence:"));
        }
        // Altitude range (meters).
        if let (Some(min), Some(max)) = (d.min_elevation_meters, d.max_elevation_meters) {
            let avg = d
                .avg_elevation_meters
                .map(|a| format!(" (avg {a:.0})"))
                .unwrap_or_default();
            lines.push(format!(
                "  {:<LABEL_WIDTH$}{min:.0}\u{2013}{max:.0} m{avg}",
                "Altitude:"
            ));
        }
        if let Some(vs) = d.max_vertical_speed_mps {
            // m/s → m/h for more intuitive climbing rate.
            lines.push(format!("  {:<LABEL_WIDTH$}{:.0} m/h", "Max Vert:", vs * 3600.0));
        }
        // Elapsed vs moving — show stopped time if any.
        if let Some(elapsed) = d.elapsed_duration_seconds {
            let stopped = elapsed - self.summary.duration_seconds;
            let extra = if stopped > 1.0 {
                format!(" ({} stopped)", fmt_hms(stopped))
            } else {
                String::new()
            };
            lines.push(format!("  {:<LABEL_WIDTH$}{}{extra}", "Elapsed:", fmt_hms(elapsed)));
        }
        // Calorie breakdown (Active = Total − BMR).
        if let (Some(total), Some(bmr)) = (self.summary.calories, d.bmr_calories) {
            lines.push(format!(
                "  {:<LABEL_WIDTH$}{:.0} kcal ({:.0} active + {:.0} BMR)",
                "Calories:",
                total,
                total - bmr,
                bmr
            ));
        }
        if let (Some(start), Some(end)) = (d.begin_potential_stamina, d.end_potential_stamina) {
            let min_str = d
                .min_available_stamina
                .map(|m| format!(" (min {m:.0})"))
                .unwrap_or_default();
            lines.push(format!(
                "  {:<LABEL_WIDTH$}{start:.0} \u{2192} {end:.0}{min_str}",
                "Stamina:"
            ));
        }
        if let Some(feel) = d.direct_workout_feel {
            let rpe = d.direct_workout_rpe.map(|r| format!(", RPE {r}")).unwrap_or_default();
            lines.push(format!(
                "  {:<LABEL_WIDTH$}{} ({feel}{rpe})",
                "Feel:",
                workout_feel_label(feel)
            ));
        }
        if let Some(score) = d.direct_workout_compliance_score {
            lines.push(format!("  {:<LABEL_WIDTH$}{score:.0}%", "Compliance:"));
        }

        if !lines.is_empty() {
            println!();
            println!("  {}", "Details".bold());
            println!("  {}", "\u{2500}".repeat(38).dimmed());
            for line in lines {
                println!("{line}");
            }
        }
    }
}
