use super::labels::{phase_display_label, sport_type_label, supplemental_label};
use super::phase::TrainingPhase;
use super::task::CoachTask;
use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use crate::garmin::types::helpers::{fmt_hm, unknown};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// The adaptive plan response includes both `adaptivePlanPhases` (populated)
/// and `planPhases` (legacy mirror, sometimes populated, sometimes not).
/// Non-adaptive plans return `planPhases` only. A manual Deserialize impl
/// folds them into a single `phases` field, preferring the adaptive one.
#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct CoachPlan {
    pub training_plan_id: u64,
    pub name: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub duration_weeks: Option<u32>,
    pub avg_weekly_workouts: Option<u32>,
    pub training_status: Option<String>,
    pub training_level: Option<String>,
    pub training_version: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub supplemental_sports: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub phases: Vec<TrainingPhase>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub task_list: Vec<CoachTask>,
}

impl<'de> Deserialize<'de> for CoachPlan {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all(deserialize = "camelCase"))]
        struct Raw {
            #[serde(default)]
            training_plan_id: u64,
            #[serde(default = "unknown")]
            name: String,
            start_date: Option<String>,
            end_date: Option<String>,
            duration_in_weeks: Option<u32>,
            training_level: Option<TrainingLevel>,
            avg_weekly_workouts: Option<u32>,
            training_version: Option<TrainingVersion>,
            training_status: Option<TrainingStatusRef>,
            #[serde(default)]
            task_list: Vec<CoachTask>,
            #[serde(default)]
            adaptive_plan_phases: Vec<TrainingPhase>,
            #[serde(default)]
            plan_phases: Vec<TrainingPhase>,
            #[serde(default)]
            supplemental_sports: Vec<String>,
        }
        let r = Raw::deserialize(d)?;
        let phases = if !r.adaptive_plan_phases.is_empty() {
            r.adaptive_plan_phases
        } else {
            r.plan_phases
        };
        Ok(CoachPlan {
            training_plan_id: r.training_plan_id,
            name: r.name,
            start_date: r.start_date.map(trim_date_owned),
            end_date: r.end_date.map(trim_date_owned),
            duration_weeks: r.duration_in_weeks,
            avg_weekly_workouts: r.avg_weekly_workouts,
            training_status: r.training_status.map(|s| s.status_key),
            training_level: r.training_level.map(|s| s.level_key),
            training_version: r.training_version.map(|s| s.version_name),
            supplemental_sports: r.supplemental_sports,
            phases,
            task_list: r.task_list,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
struct TrainingLevel {
    level_key: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
struct TrainingVersion {
    version_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
struct TrainingStatusRef {
    status_key: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct TrainingPlanListResponse {
    #[serde(default)]
    pub training_plan_list: Vec<TrainingPlanSummary>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct TrainingPlanSummary {
    pub training_plan_id: u64,
    pub name: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub duration_weeks: Option<u32>,
    pub training_plan_category: Option<String>,
    pub training_status: Option<String>,
    pub training_level: Option<String>,
    pub training_version: Option<String>,
}

impl<'de> Deserialize<'de> for TrainingPlanSummary {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all(deserialize = "camelCase"))]
        struct Raw {
            #[serde(default)]
            training_plan_id: u64,
            #[serde(default = "unknown")]
            name: String,
            start_date: Option<String>,
            end_date: Option<String>,
            duration_in_weeks: Option<u32>,
            training_plan_category: Option<String>,
            training_level: Option<TrainingLevel>,
            training_version: Option<TrainingVersion>,
            training_status: Option<TrainingStatusRef>,
        }
        let r = Raw::deserialize(d)?;
        Ok(TrainingPlanSummary {
            training_plan_id: r.training_plan_id,
            name: r.name,
            start_date: r.start_date.map(trim_date_owned),
            end_date: r.end_date.map(trim_date_owned),
            duration_weeks: r.duration_in_weeks,
            training_plan_category: r.training_plan_category,
            training_status: r.training_status.map(|s| s.status_key),
            training_level: r.training_level.map(|s| s.level_key),
            training_version: r.training_version.map(|s| s.version_name),
        })
    }
}

fn trim_date_owned(s: String) -> String {
    s[..s.len().min(10)].to_string()
}

impl HumanReadable for CoachPlan {
    fn print_human(&self) {
        println!("{}", self.name.bold());
        println!("{}", "\u{2500}".repeat(40).dimmed());
        println!("  {:<LABEL_WIDTH$}{}", "ID:", self.training_plan_id);
        if let Some(ref level) = self.training_level {
            println!("  {:<LABEL_WIDTH$}{}", "Level:", level.cyan());
        }
        if let Some(ref version) = self.training_version {
            println!("  {:<LABEL_WIDTH$}{}", "Target:", version);
        }
        if let (Some(start), Some(end)) = (&self.start_date, &self.end_date) {
            let weeks = self.duration_weeks.map(|w| format!(" ({w} weeks)")).unwrap_or_default();
            println!("  {:<LABEL_WIDTH$}{start} \u{2192} {end}{weeks}", "Range:");
        }
        if let Some(avg) = self.avg_weekly_workouts {
            println!("  {:<LABEL_WIDTH$}{avg}", "Workouts/wk:");
        }
        if let Some(ref status) = self.training_status {
            println!("  {:<LABEL_WIDTH$}{status}", "Status:");
        }
        if !self.supplemental_sports.is_empty() {
            let joined = self
                .supplemental_sports
                .iter()
                .map(|s| supplemental_label(s))
                .collect::<Vec<_>>()
                .join(", ");
            println!("  {:<LABEL_WIDTH$}{joined}", "Supplemental:");
        }
        if !self.phases.is_empty() {
            println!();
            println!("  {}", "Phases".bold());
            print_phases(&self.phases);
        }
        if !self.task_list.is_empty() {
            println!();
            let header = format!("Upcoming ({} scheduled)", self.task_list.len());
            println!("  {}", header.bold());
            print_tasks(&self.task_list);
        }
    }
}

impl HumanReadable for TrainingPlanSummary {
    fn print_human(&self) {
        let status = self.training_status.as_deref().unwrap_or("");
        let glyph = match status {
            "Completed" => "\u{2713}",
            "Paused" => "\u{2016}",
            "Scheduled" | "Active" | "" => "\u{25CF}",
            _ => "\u{25E6}",
        };
        let range = match (&self.start_date, &self.end_date) {
            (Some(a), Some(b)) => format!("{a} \u{2192} {b}"),
            _ => String::new(),
        };
        let weeks = self.duration_weeks.map(|w| format!("{w} wk")).unwrap_or_default();
        let level = self.training_level.as_deref().unwrap_or("");
        let version = self.training_version.as_deref().unwrap_or("");
        let level_target = match (level.is_empty(), version.is_empty()) {
            (false, false) => format!("{level} \u{00B7} {version}"),
            (false, true) => level.into(),
            (true, false) => version.into(),
            (true, true) => String::new(),
        };
        println!(
            "  {glyph}  {name:<26} {status:<12} {range:<26} {weeks:>5}   {level_target}",
            name = self.name,
        );
    }
}

fn print_phases(phases: &[TrainingPhase]) {
    for phase in phases {
        let label = phase_display_label(&phase.training_phase);
        let range = if phase.start_date == phase.end_date {
            phase.start_date.clone()
        } else {
            format!("{} \u{2192} {}", phase.start_date, phase.end_date)
        };
        let marker = if phase.current_phase {
            "   \u{25CF} current".to_string()
        } else {
            String::new()
        };
        println!("    {label:<8} {range}{marker}");
    }
}

fn print_tasks(tasks: &[super::task::CoachTask]) {
    let mut ordered: Vec<&super::task::CoachTask> = tasks.iter().collect();
    ordered.sort_by(|a, b| {
        a.calendar_date
            .cmp(&b.calendar_date)
            .then(a.workout_order.cmp(&b.workout_order))
    });
    for task in ordered {
        let day = day_of_week(&task.calendar_date);
        if task.task_workout.rest_day {
            println!("    {day} {date}   Rest", date = task.calendar_date);
            continue;
        }
        let sport = task
            .task_workout
            .sport_type
            .as_ref()
            .map(|s| sport_type_label(&s.sport_type_key))
            .unwrap_or_else(|| "\u{2014}".into());
        let name = task.task_workout.workout_name.as_deref().unwrap_or("\u{2014}");
        let desc = task.task_workout.workout_description.as_deref().unwrap_or("");
        let dur = task
            .task_workout
            .estimated_duration_seconds
            .map(|s| fmt_hm(s.round() as u64))
            .unwrap_or_default();
        let dist = task
            .task_workout
            .estimated_distance_meters
            .filter(|&m| m > 0.0)
            .map(|m| format!(" \u{00B7} {:.1} km", m / 1000.0))
            .unwrap_or_default();
        println!(
            "    {day} {date}   {sport:<9}  {name:<20}  {desc:<18}  {dur}{dist}",
            date = task.calendar_date,
        );
    }
}

/// YYYY-MM-DD → 3-letter weekday abbreviation (en_US). Falls back to three
/// dashes if the date doesn't parse.
fn day_of_week(iso_date: &str) -> String {
    chrono::NaiveDate::parse_from_str(iso_date, "%Y-%m-%d")
        .map(|d| d.format("%a").to_string())
        .unwrap_or_else(|_| "---".into())
}
